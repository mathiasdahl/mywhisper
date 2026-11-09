use clap::ValueEnum;
use crossterm::style::Stylize;
use std::io::{self, IsTerminal, Write};

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Clone, Copy)]
pub struct Console {
    color: bool,
}

impl Console {
    pub fn from_mode(mode: ColorMode) -> Self {
        let color = match mode {
            ColorMode::Never => {
                crossterm::style::force_color_output(false);
                false
            }
            ColorMode::Always => {
                crossterm::style::force_color_output(true);
                true
            }
            ColorMode::Auto => {
                let on =
                    io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none();
                if !on {
                    crossterm::style::force_color_output(false);
                }
                on
            }
        };
        Self { color }
    }

    pub fn blank_line(&self) {
        let _ = writeln!(io::stderr());
    }

    pub fn write_prompt(&self, text: &str) -> io::Result<()> {
        if self.color {
            write!(io::stderr(), "{}", text.yellow().bold())
        } else {
            write!(io::stderr(), "{text}")
        }
    }

    pub fn context_device(&self, device_name: &str) {
        self.eprintln(|c| {
            if c {
                format!("{} {}", "Using input device:".cyan(), device_name)
            } else {
                format!("Using input device: {device_name}")
            }
        });
    }

    pub fn controls(&self, text: &str) {
        self.eprintln(|c| hint_line(c, text, false));
    }

    pub fn prompt(&self, text: &str) {
        self.eprintln(|c| {
            if c {
                text.yellow().bold().to_string()
            } else {
                text.to_string()
            }
        });
    }

    pub fn active(&self, headline: &str, tail: Option<&str>) {
        self.eprintln(|c| {
            if c {
                let head = headline.magenta().bold();
                match tail {
                    Some(t) => format!("{} {}", head, t.dark_grey()),
                    None => head.to_string(),
                }
            } else {
                match tail {
                    Some(t) => format!("{headline} {t}"),
                    None => headline.to_string(),
                }
            }
        });
    }

    pub fn paused(&self) {
        self.eprintln(|c| {
            if c {
                "Paused".yellow().to_string()
            } else {
                "Paused".to_string()
            }
        });
    }

    pub fn paused_with_duration(&self, duration: &str) {
        self.eprintln(|c| {
            if c {
                format!("{} · {}", "Paused".yellow(), duration.dark_grey())
            } else {
                format!("Paused · {duration}")
            }
        });
    }

    pub fn resumed(&self) {
        self.eprintln(|c| {
            if c {
                "Resumed".green().to_string()
            } else {
                "Resumed".to_string()
            }
        });
    }

    pub fn saved(&self, kind: &str, path: &str) {
        self.eprintln(|c| {
            if c {
                format!("{} {}", kind.green(), path.dark_cyan())
            } else {
                format!("{kind} {path}")
            }
        });
    }

    pub fn metric(&self, label: &str, value: &str) {
        self.eprintln(|c| {
            if c {
                format!("{} {}", label.dark_grey(), value.cyan())
            } else {
                format!("{label} {value}")
            }
        });
    }

    pub fn metric_cost(&self, sek: &str, usd_paren: &str) {
        self.eprintln(|c| {
            if c {
                format!(
                    "{} {} {}",
                    "Estimated transcription cost:".dark_grey(),
                    sek.cyan(),
                    usd_paren.dark_grey()
                )
            } else {
                format!("Estimated transcription cost: {sek} {usd_paren}")
            }
        });
    }

    pub fn dry_run(&self, message: &str) {
        self.eprintln(|c| {
            if c {
                format!("{} {}", "Dry-run:".dark_yellow(), message.dark_grey())
            } else {
                format!("Dry-run: {message}")
            }
        });
    }

    pub fn dry_run_progress(&self, duration: &str) {
        let line = format!(
            "Dry-run · {duration} elapsed · Esc stop · Enter/Space pause"
        );
        self.eprintln(|c| hint_line(c, &line, false));
    }

    pub fn neutral(&self, text: &str) {
        self.eprintln(|c| {
            if c {
                text.dark_grey().to_string()
            } else {
                text.to_string()
            }
        });
    }

    pub fn copied_clipboard(&self) {
        self.eprintln(|c| {
            if c {
                "Copied to clipboard.".green().to_string()
            } else {
                "Copied to clipboard.".to_string()
            }
        });
    }

    pub fn stream_error(&self, err: &str) {
        self.eprintln(|c| {
            if c {
                format!("audio stream error: {}", err.red().bold())
            } else {
                format!("audio stream error: {err}")
            }
        });
    }

    pub fn transcript_frame_top(&self) {
        self.eprintln(|c| rule_line(c, "--- Transcript ---"));
    }

    pub fn transcript_frame_bottom(&self) {
        self.eprintln(|c| rule_line(c, "------------------"));
    }

    pub fn transcript_body(&self, text: &str) {
        if text.is_empty() {
            self.eprintln(|c| {
                if c {
                    "(empty)".dark_grey().to_string()
                } else {
                    "(empty)".to_string()
                }
            });
        } else {
            self.eprintln(|_| text.to_string());
        }
    }

    pub fn output_menu(&self) {
        const RAW: &str = "s — stdout · c — clipboard · q/Esc — quit without output";
        self.eprintln(|c| hint_line(c, RAW, true));
    }

    fn eprintln(&self, f: impl FnOnce(bool) -> String) {
        let line = f(self.color);
        // Unix raw mode does not translate `\n` to `\r\n`, so lines printed while a key loop
        // is active would staircase across the terminal.
        let eol = if crossterm::terminal::is_raw_mode_enabled().unwrap_or(false) {
            "\r\n"
        } else {
            "\n"
        };
        let _ = write!(io::stderr(), "{line}{eol}");
    }
}

fn rule_line(color: bool, text: &str) -> String {
    if color {
        text.dark_cyan().to_string()
    } else {
        text.to_string()
    }
}

fn hint_line(color: bool, plain: &str, menu_style: bool) -> String {
    if !color {
        return plain.to_string();
    }

    let mut keys: Vec<&str> = vec![
        "Enter/Esc",
        "Enter or Escape",
        "Enter or Space",
        "Enter",
        "Escape",
        "Esc",
        "Space",
    ];
    if menu_style {
        keys.extend(["q/Esc —", "q/Esc", "s —", "c —"]);
    }
    keys.sort_by_key(|k| std::cmp::Reverse(k.len()));

    let mut out = String::new();
    let mut i = 0;
    while i < plain.len() {
        if let Some(key) = keys.iter().find(|k| plain[i..].starts_with(**k)) {
            let end = i + key.len();
            out.push_str(&plain[i..end].cyan().bold().to_string());
            i = end;
        } else {
            let mut end = i + 1;
            while end < plain.len() {
                if keys.iter().any(|k| plain[end..].starts_with(*k)) {
                    break;
                }
                end += plain[end..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            }
            out.push_str(&plain[i..end].dark_grey().to_string());
            i = end;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Stylize;

    #[test]
    fn subprocess_color_always_writes_ansi() {
        use std::path::Path;
        use std::process::Command;

        let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/mywhisper.exe");
        if !bin.exists() {
            return;
        }
        let out = Command::new(&bin)
            .args(["--dry-run", "-y", "--color", "always"])
            .output()
            .expect("run mywhisper");
        assert!(
            out.stderr.iter().any(|&b| b == 0x1b),
            "stderr {:?}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    #[test]
    fn force_color_emits_ansi() {
        crossterm::style::force_color_output(true);
        let s = "x".red().to_string();
        assert!(
            s.contains('\x1b'),
            "expected ANSI escapes in {s:?}"
        );
    }

    #[test]
    fn dry_run_message_contains_ansi() {
        let c = Console::from_mode(ColorMode::Always);
        let message = "no recording, no API call.";
        let line = if c.color {
            format!("{} {}", "Dry-run:".dark_yellow(), message.dark_grey())
        } else {
            format!("Dry-run: {message}")
        };
        assert!(line.contains('\x1b'), "expected ANSI in {line:?}");
    }

}
