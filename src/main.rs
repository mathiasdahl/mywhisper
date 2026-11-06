mod logging;
mod output;

use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use clap::Parser;
use output::{InteractiveOutput, OutputChannels};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SupportedStreamConfig};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use hound::{SampleFormat as WavSampleFormat, WavSpec, WavWriter};
use serde::Deserialize;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex};
use std::time::{Duration, Instant};

const OPENROUTER_TRANSCRIBE_URL: &str = "https://openrouter.ai/api/v1/audio/transcriptions";
const MODEL: &str = "openai/whisper-large-v3-turbo";
const USD_PER_SECOND: f64 = 0.000003;
const SEK_PER_USD: f64 = 10.0;
const DEFAULT_DRY_RUN_TEXT: &str = "[dry-run] Mock transcript for UI testing.";

#[derive(Parser, Debug)]
#[command(name = "mywhisper", about = "Record voice and transcribe with Whisper via OpenRouter")]
struct Cli {
    /// ISO-639-1 language hint (optional; auto-detected if omitted)
    #[arg(short, long)]
    language: Option<String>,

    /// Start recording as soon as the program is ready (skip Enter to start)
    #[arg(short, long)]
    immediate: bool,

    /// Transcribe as soon as recording stops (skip confirm prompt)
    #[arg(short = 'y', long)]
    auto_send: bool,

    /// Write transcript to stdout (default when no output flags are given)
    #[arg(long)]
    stdout: bool,

    /// Copy transcript to the Windows clipboard
    #[arg(long)]
    clipboard: bool,

    /// Skip recording and API; use mock text to test output channels and prompts (no API key)
    #[arg(long)]
    dry_run: bool,

    /// Transcript text when using --dry-run
    #[arg(long, value_name = "TEXT")]
    dry_run_text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TranscriptionResponse {
    text: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let text = if cli.dry_run {
        eprintln!("Dry-run: no recording, no API call.");

        if !cli.auto_send {
            dry_run_recording_phase(cli.immediate)?;
        }

        if cli.auto_send {
            eprintln!("Dry-run: skipping API call…");
        } else {
            eprintln!("Press Enter to transcribe, Escape to discard.");
            if !wait_enter_or_escape()? {
                eprintln!("Discarded without sending.");
                return Ok(());
            }
            eprintln!("Dry-run: skipping API call…");
        }

        {
            let s = cli
                .dry_run_text
                .as_deref()
                .unwrap_or(DEFAULT_DRY_RUN_TEXT)
                .trim();
            if s.is_empty() {
                DEFAULT_DRY_RUN_TEXT.to_string()
            } else {
                s.to_string()
            }
        }
    } else {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY environment variable is not set")?;

        let wav_bytes = record_wav(cli.immediate)?;
        let session = logging::SessionLog::save_recording(&wav_bytes)?;
        let duration = wav_duration_secs(&wav_bytes)?;
        let usd = duration * USD_PER_SECOND;
        let sek = usd * SEK_PER_USD;

        eprintln!();
        eprintln!("Recording length: {}", format_duration(duration));
        eprintln!(
            "Estimated transcription cost: {} ({})",
            format_sek(sek),
            format_usd(usd)
        );

        if cli.auto_send {
            eprintln!("Sending to OpenRouter…");
        } else {
            eprintln!("Press Enter to transcribe, Escape to discard.");
            if !wait_enter_or_escape()? {
                eprintln!("Discarded without sending.");
                return Ok(());
            }
            eprintln!("Sending to OpenRouter…");
        }
        let text = transcribe(&wav_bytes, &api_key, cli.language.as_deref())?;
        session.save_transcript(&text)?;
        text
    };

    let channels = OutputChannels::from_cli(cli.stdout, cli.clipboard);
    let output_channels_explicit = cli.stdout || cli.clipboard;
    if cli.auto_send || output_channels_explicit {
        channels.deliver(&text)?;
    } else {
        match output::prompt_output_channel(&text)? {
            InteractiveOutput::Channels(choice) => choice.deliver(&text)?,
            InteractiveOutput::None => eprintln!("No output channel selected."),
        }
    }
    Ok(())
}

fn format_duration(secs: f64) -> String {
    if secs >= 60.0 {
        let minutes = (secs / 60.0).floor() as u32;
        let seconds = secs - f64::from(minutes) * 60.0;
        format!("{minutes}:{seconds:05.2}")
    } else {
        format!("{secs:.2} s")
    }
}

fn format_usd(amount: f64) -> String {
    if amount >= 0.01 {
        format!("${amount:.4} USD")
    } else {
        format!("${amount:.8} USD")
    }
}

fn format_sek(amount: f64) -> String {
    if amount >= 0.01 {
        format!("{amount:.4} SEK")
    } else if amount >= 0.000_1 {
        format!("{amount:.6} SEK")
    } else {
        format!("{amount:.8} SEK")
    }
}

fn wait_enter_prompt(message: &str) -> Result<()> {
    eprint!("{message}");
    io::stderr().flush()?;
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .context("failed to read Enter from stdin")?;
    Ok(())
}

fn wait_enter_or_escape() -> Result<bool> {
    enable_raw_mode().context("failed to enable raw terminal mode")?;
    let result = read_enter_or_escape_loop();
    disable_raw_mode().context("failed to disable raw terminal mode")?;
    result
}

fn read_enter_or_escape_loop() -> Result<bool> {
    loop {
        if event::poll(Duration::from_millis(100)).context("failed to poll keyboard")? {
            if let Event::Key(key) = event::read().context("failed to read keyboard event")? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Enter => return Ok(true),
                    KeyCode::Esc => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}

fn run_recording_controls(paused: Arc<AtomicBool>) -> Result<()> {
    enable_raw_mode().context("failed to enable raw terminal mode")?;
    let result = recording_control_loop(&paused);
    disable_raw_mode().context("failed to disable raw terminal mode")?;
    result
}

/// Simulates the pre-record and record UX without a microphone (dry-run only).
fn dry_run_recording_phase(immediate: bool) -> Result<()> {
    eprintln!("Dry-run: no microphone (simulated).");

    if immediate {
        eprintln!("Ready — dry-run recording now. Esc stop · Enter or Space pause/unpause");
    } else {
        eprintln!("Controls: Enter start · Esc stop · Enter or Space pause/unpause");
        wait_enter_prompt("Press Enter to start recording… ")?;
    }

    eprintln!("Dry-run recording… Esc to stop · Enter or Space to pause/unpause");

    struct ActiveTimer {
        paused: bool,
        active_elapsed: Duration,
        segment_start: Instant,
    }

    impl ActiveTimer {
        fn new() -> Self {
            Self {
                paused: false,
                active_elapsed: Duration::ZERO,
                segment_start: Instant::now(),
            }
        }

        fn active_secs(&self) -> f64 {
            let active = if self.paused {
                self.active_elapsed
            } else {
                self.active_elapsed + self.segment_start.elapsed()
            };
            active.as_secs_f64()
        }

        fn toggle_pause(&mut self) {
            let now = Instant::now();
            if self.paused {
                self.segment_start = now;
                self.paused = false;
            } else {
                self.active_elapsed += now.duration_since(self.segment_start);
                self.paused = true;
            }
        }
    }

    let mut timer = ActiveTimer::new();
    let mut last_whole_sec: Option<u64> = None;

    enable_raw_mode().context("failed to enable raw terminal mode")?;
    let result = (|| -> Result<()> {
        loop {
            let secs = timer.active_secs();
            let whole = secs.floor() as u64;
            if !timer.paused && last_whole_sec != Some(whole) {
                last_whole_sec = Some(whole);
                eprintln!(
                    "Dry-run · {} elapsed · Esc stop · Enter/Space pause",
                    format_duration(secs)
                );
            }

            if event::poll(Duration::from_millis(50)).context("failed to poll keyboard")? {
                if let Event::Key(key) = event::read().context("failed to read keyboard event")? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        KeyCode::Esc => break,
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            timer.toggle_pause();
                            if timer.paused {
                                eprintln!(
                                    "Paused · {}",
                                    format_duration(timer.active_secs())
                                );
                            } else {
                                eprintln!("Resumed");
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    })();
    disable_raw_mode().context("failed to disable raw terminal mode")?;
    result
}

fn recording_control_loop(paused: &Arc<AtomicBool>) -> Result<()> {
    eprintln!("Recording… Enter or Escape to stop · Space to pause/unpause");

    loop {
        if event::poll(Duration::from_millis(50)).context("failed to poll keyboard")? {
            if let Event::Key(key) = event::read().context("failed to read keyboard event")? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Enter | KeyCode::Esc => break,
                    KeyCode::Char(' ') => {
                        let now_paused = !paused.load(Ordering::Relaxed);
                        paused.store(now_paused, Ordering::Relaxed);
                        if now_paused {
                            eprintln!("Paused");
                        } else {
                            eprintln!("Resumed");
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn wav_duration_secs(wav: &[u8]) -> Result<f64> {
    let reader = hound::WavReader::new(std::io::Cursor::new(wav)).context("invalid WAV")?;
    let spec = reader.spec();
    let frames = reader.len() as f64 / f64::from(spec.channels);
    Ok(frames / spec.sample_rate as f64)
}

fn record_wav(start_immediately: bool) -> Result<Vec<u8>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("no default input device (microphone) found")?;

    let device_name = device.name().unwrap_or_else(|_| "unknown".into());
    eprintln!("Using input device: {device_name}");

    if start_immediately {
        eprintln!("Ready — recording now. Enter/Esc stop · Space pause");
    } else {
        eprintln!("Controls: Enter start · Enter/Esc stop · Space pause");
        wait_enter_prompt("Press Enter to start recording… ")?;
    }

    let config = device
        .default_input_config()
        .context("failed to read default input config")?;
    let sample_rate = config.sample_rate().0;
    let channels = config.channels();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let paused = Arc::new(AtomicBool::new(false));

    let stream = build_input_stream(&device, config, Arc::clone(&samples), Arc::clone(&paused))?;
    stream.play().context("failed to start audio stream")?;

    run_recording_controls(paused)?;
    drop(stream);

    let samples = samples
        .lock()
        .map_err(|_| anyhow::anyhow!("audio buffer poisoned"))?;
    if samples.is_empty() {
        bail!("no audio captured; check microphone permissions and input level");
    }

    encode_wav(&samples, sample_rate, channels)
}

fn build_input_stream(
    device: &cpal::Device,
    config: SupportedStreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
    paused: Arc<AtomicBool>,
) -> Result<cpal::Stream> {
    let sample_format = config.sample_format();
    let stream_config: cpal::StreamConfig = config.clone().into();

    let err_fn = |err| eprintln!("audio stream error: {err}");

    let stream = match sample_format {
        SampleFormat::F32 => {
            let paused = Arc::clone(&paused);
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    append_samples(data, &samples, &paused);
                },
                err_fn,
                None,
            )
        }
        SampleFormat::I16 => {
            let paused = Arc::clone(&paused);
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let chunk: Vec<f32> = data
                        .iter()
                        .map(|&s| Sample::to_sample::<f32>(s))
                        .collect();
                    append_samples(&chunk, &samples, &paused);
                },
                err_fn,
                None,
            )
        }
        SampleFormat::U16 => {
            let paused = Arc::clone(&paused);
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let chunk: Vec<f32> = data
                        .iter()
                        .map(|&s| Sample::to_sample::<f32>(s))
                        .collect();
                    append_samples(&chunk, &samples, &paused);
                },
                err_fn,
                None,
            )
        }
        other => bail!("unsupported sample format: {other:?}"),
    }
    .context("failed to build input stream")?;

    Ok(stream)
}

fn append_samples(data: &[f32], samples: &Arc<Mutex<Vec<f32>>>, paused: &Arc<AtomicBool>) {
    if paused.load(Ordering::Relaxed) {
        return;
    }
    if let Ok(mut buf) = samples.lock() {
        buf.extend_from_slice(data);
    }
}

fn encode_wav(samples: &[f32], sample_rate: u32, channels: u16) -> Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(Vec::new());
    {
        let spec = WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: WavSampleFormat::Int,
        };
        let mut writer = WavWriter::new(&mut cursor, spec).context("failed to create WAV writer")?;
        for &sample in samples {
            let clamped = sample.clamp(-1.0, 1.0);
            let int_sample = (clamped * i16::MAX as f32) as i16;
            writer
                .write_sample(int_sample)
                .context("failed to write WAV sample")?;
        }
        writer.finalize().context("failed to finalize WAV")?;
    }
    Ok(cursor.into_inner())
}

fn transcribe(wav_bytes: &[u8], api_key: &str, language: Option<&str>) -> Result<String> {
    let audio_b64 = STANDARD.encode(wav_bytes);

    let mut body = serde_json::json!({
        "model": MODEL,
        "input_audio": {
            "data": audio_b64,
            "format": "wav"
        }
    });
    if let Some(lang) = language {
        body["language"] = serde_json::Value::String(lang.to_string());
    }

    let client = reqwest::blocking::Client::new();
    let response = client
        .post(OPENROUTER_TRANSCRIBE_URL)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .context("HTTP request to OpenRouter failed")?;

    let status = response.status();
    let response_text = response
        .text()
        .context("failed to read OpenRouter response body")?;

    if !status.is_success() {
        bail!("OpenRouter API error ({status}): {response_text}");
    }

    let parsed: TranscriptionResponse = serde_json::from_str(&response_text)
        .with_context(|| format!("unexpected response JSON: {response_text}"))?;

    Ok(parsed.text.trim().to_string())
}
