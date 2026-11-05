use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::path::PathBuf;

pub struct SessionLog {
    txt_path: PathBuf,
}

impl SessionLog {
    pub fn save_recording(wav_bytes: &[u8]) -> Result<Self> {
        let dir = log_directory()?;
        fs::create_dir_all(&dir).with_context(|| format!("failed to create log dir {}", dir.display()))?;

        let stem = Local::now().format("%Y%m%d_%H%M%S").to_string();
        let wav_path = dir.join(format!("{stem}.wav"));
        fs::write(&wav_path, wav_bytes)
            .with_context(|| format!("failed to write recording {}", wav_path.display()))?;

        let txt_path = dir.join(format!("{stem}.txt"));
        eprintln!("Saved recording: {}", wav_path.display());

        let _ = wav_path;
        Ok(Self { txt_path })
    }

    pub fn save_transcript(&self, text: &str) -> Result<()> {
        fs::write(&self.txt_path, text)
            .with_context(|| format!("failed to write transcript {}", self.txt_path.display()))?;
        eprintln!("Saved transcript: {}", self.txt_path.display());
        Ok(())
    }
}

fn log_directory() -> Result<PathBuf> {
    let base = dirs::data_local_dir().context("could not resolve local data directory")?;
    Ok(base.join("mywhisper").join("recordings"))
}
