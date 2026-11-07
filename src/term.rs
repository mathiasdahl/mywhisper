//! Terminal helpers that behave the same on Windows, macOS and Linux.

use anyhow::{Context, Result};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

/// Enables raw mode for its lifetime; restores the terminal on drop (including early `?` returns).
pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> Result<Self> {
        enable_raw_mode().context("failed to enable raw terminal mode")?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}

/// Raw mode disables SIGINT on Unix, so Ctrl+C arrives as a key event. Exit like a normal interrupt.
pub fn exit_on_ctrl_c(key: &KeyEvent) {
    let is_ctrl_c = key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'));
    if is_ctrl_c {
        let _ = disable_raw_mode();
        eprintln!();
        eprintln!("Interrupted.");
        std::process::exit(130);
    }
}

/// `eprintln!` for use while raw mode is on: Unix raw mode does not translate `\n` to `\r\n`.
macro_rules! rawln {
    ($($arg:tt)*) => {
        eprint!("{}\r\n", format!($($arg)*))
    };
}
pub(crate) use rawln;
