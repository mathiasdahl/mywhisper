use anyhow::{Context, Result};
use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::time::Duration;

#[derive(Debug, Clone, Copy, Default)]
pub struct OutputChannels {
    pub stdout: bool,
    pub clipboard: bool,
}

impl OutputChannels {
    pub fn from_cli(stdout_flag: bool, clipboard_flag: bool) -> Self {
        if stdout_flag || clipboard_flag {
            Self {
                stdout: stdout_flag,
                clipboard: clipboard_flag,
            }
        } else {
            Self {
                stdout: true,
                clipboard: false,
            }
        }
    }

    pub fn deliver(&self, text: &str) -> Result<()> {
        if self.stdout {
            println!("{text}");
        }
        if self.clipboard {
            copy_to_clipboard(text)?;
            if !self.stdout {
                eprintln!("Copied to clipboard.");
            }
        }
        Ok(())
    }
}

pub enum InteractiveOutput {
    Channels(OutputChannels),
    None,
}

pub fn prompt_output_channel(text: &str) -> Result<InteractiveOutput> {
    eprintln!();
    eprintln!("--- Transcript ---");
    eprintln!("{text}");
    eprintln!("------------------");
    eprintln!("s — stdout · c — clipboard · q/Esc — quit without output");

    enable_raw_mode().context("failed to enable raw terminal mode")?;
    let choice = read_output_channel_choice();
    disable_raw_mode().context("failed to disable raw terminal mode")?;
    choice
}

fn read_output_channel_choice() -> Result<InteractiveOutput> {
    loop {
        if event::poll(Duration::from_millis(100)).context("failed to poll keyboard")? {
            if let Event::Key(key) = event::read().context("failed to read keyboard event")? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                return Ok(match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        InteractiveOutput::Channels(OutputChannels {
                            stdout: true,
                            clipboard: false,
                        })
                    }
                    KeyCode::Char('c') | KeyCode::Char('C') => {
                        InteractiveOutput::Channels(OutputChannels {
                            stdout: false,
                            clipboard: true,
                        })
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                        InteractiveOutput::None
                    }
                    _ => continue,
                });
            }
        }
    }
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    Clipboard::new()
        .context("failed to open clipboard")?
        .set_text(text)
        .context("failed to write to clipboard")?;
    Ok(())
}
