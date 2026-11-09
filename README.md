# mywhisper

**mywhisper** is a small **command-line application for Windows and macOS** (Apple Silicon). You run it from a terminal—**PowerShell** / **Command Prompt** on Windows, **Terminal** / iTerm / Ghostty on macOS—or from a shortcut that opens one. It records your microphone, sends the audio to [OpenAI Whisper Large V3 Turbo](https://openrouter.ai/openai/whisper-large-v3-turbo) through [OpenRouter](https://openrouter.ai/), and prints the transcript or copies it to the clipboard.

There is no graphical window—everything happens in the terminal, with keyboard shortcuts for record/stop/send. For day-to-day use, most people bind a hotkey that opens a small terminal running mywhisper (see [Quick launch without a GUI](#quick-launch-without-a-gui) for Windows and [Quick launch on macOS](#quick-launch-on-macos)).

### Background

You can already dictate in Claude, Cursor, VS Code, and Windows—but those flows are someone else’s product. **mywhisper** exists because I wanted something I control end to end: what gets recorded, where audio is stored, when it is sent for transcription, and what happens to the text afterward (stdout, clipboard, files under `%LOCALAPPDATA%` or `~/Library/Application Support`). It is a small CLI I can wire into shortcuts and scripts without a GUI or vendor dictation UI.

Running it is inexpensive. OpenRouter bills Whisper Large V3 Turbo at **$0.000003 per second** of audio (~**$0.011/hour**). If you transcribed **eight hours of speech every weekday** (~22 days), that is about **$1.90/month** in API cost; eight hours **every day** for a 30-day month is roughly **$2.60**. Real dictation is usually a fraction of that unless you are recording continuously. The program prints an estimated cost on stderr after each clip (same rate as above).

---

## Quick start (users)

You do **not** need Rust or a compiler. Use a pre-built binary from [GitHub Releases](https://github.com/mathiasdahl/mywhisper/releases). This section covers Windows; macOS users jump to [Quick start (macOS)](#quick-start-macos).

1. **Download** `mywhisper-x86_64-pc-windows-msvc.zip` and verify it against the matching `.sha256` file on the release (optional but recommended). Alternatively, run the PowerShell installer shown on the release page.
2. **Extract** `mywhisper.exe` to a folder you keep, e.g. `C:\Tools\mywhisper\mywhisper.exe`.
3. **OpenRouter API key** — create one at [openrouter.ai](https://openrouter.ai/), then set it for your Windows user account so every new terminal sees it:
   - Start → type **environment variables** → *Edit environment variables for your account*
   - **New** under *User variables*: name `OPENROUTER_API_KEY`, value `sk-or-v1-...`
   - OK, then **open a new** PowerShell window (existing windows do not pick up the change).

   For a one-off test in the current session only:

   ```powershell
   $env:OPENROUTER_API_KEY = "sk-or-v1-..."
   ```

4. **Run** from the folder where the exe lives, or use the full path:

   ```powershell
   cd C:\Tools\mywhisper
   .\mywhisper.exe
   ```

5. **List all flags** (no recording):

   ```powershell
   .\mywhisper.exe --help
   ```

### What you will see

- Hints and status (device name, recording length, estimated cost) go to **stderr**—the “diagnostic” stream in the terminal.
- The **transcript** goes to **stdout** by default, so you can pipe it to a file: `.\mywhisper.exe > notes.txt`
- Each real recording also saves matching `.wav` and `.txt` files under `%LOCALAPPDATA%\mywhisper\recordings\` (macOS: `~/Library/Application Support/mywhisper/recordings/`).
- Status lines on stderr use **semantic colors** when stderr is a TTY (Windows Terminal, PowerShell, or **cmd.exe** on Windows 10/11 with virtual-terminal support; Terminal.app, iTerm2 and other macOS/Linux terminals). Transcript text on stdout is never colored, so piping stays clean.

#### Terminal colors

| Setting | Behavior |
|---------|----------|
| `--color auto` (default) | Color when stderr is an interactive terminal and `NO_COLOR` is not set |
| `--color never` | Plain text |
| `--color always` | ANSI color on stderr even when redirected (e.g. logging to a file) |
| `NO_COLOR` (any value) | Plain text when `--color` is `auto`; use `--color always` to override |

Disable colors in a session: `$env:NO_COLOR = '1'` (PowerShell) or `set NO_COLOR=1` (cmd); on macOS/Linux, `export NO_COLOR=1`.

### Typical session (interactive)

1. Start `mywhisper.exe`.
2. **Enter** — start recording (unless you used `-i`; see below).
3. **Enter** or **Esc** — stop recording.
4. **Enter** — send for transcription (or **Esc** to discard).
5. Choose **s** (stdout), **c** (clipboard), or **q** / **Esc** (no output)—unless you passed `--stdout`, `--clipboard`, or `-y` earlier.

See [Keyboard controls](#keyboard-controls-interactive-mode) for pause/resume and other keys.

### Examples (command line)

Hands-off: start recording immediately and transcribe when you stop, transcript on stdout:

```powershell
.\mywhisper.exe -i -y
```

Copy to clipboard instead of printing:

```powershell
.\mywhisper.exe --clipboard
```

Swedish/English hint (ISO-639-1):

```powershell
.\mywhisper.exe -l sv
```

Try the flow **without a microphone or API key** (mock transcript):

```powershell
.\mywhisper.exe --dry-run
```

Throughout this document, **`mywhisper`** means `mywhisper.exe` (or the full path to it).

### Quick launch without a GUI

You do not need a separate app icon or tray menu. A **shortcut → small terminal → mywhisper** is the intended UI: one key chord opens a tiny window, you record and transcribe with the keyboard, then close the tab.

1. Install [Windows Terminal](https://apps.microsoft.com/store/detail/windows-terminal/9N0DX20HK701) if it is not already on your machine (`wt.exe` must be on your PATH, which is normal after install).
2. Create a desktop shortcut (Right-click desktop → **New** → **Shortcut**).
3. Set **Target** — adjust the path to your exe; flags after the exe are optional:

   ```text
   wt.exe -w -1 --size 120,10 --pos 500,500 C:\Tools\mywhisper\mywhisper.exe -i -y --clipboard
   ```

   - `-w -1` opens a new window; size/position keep the window small and out of the way (tweak `120,10` and `500,500` to taste).
   - `-i -y` starts recording immediately and sends for transcription when you stop—good for “hotkey dictation”.
   - `--clipboard` (or `--stdout`) avoids the post-transcription **s** / **c** / **q** menu.

4. **Pin to taskbar:** drag the shortcut to the taskbar. Only slots **1**–**9** respond to **Win** + that number—place mywhisper in one of those slots.
5. Press **Win** + **1**–**9** (whichever slot you used) anytime you want a fresh mywhisper session.

Set `OPENROUTER_API_KEY` as a **user** environment variable ([Quick start](#quick-start-users)) so the shortcut inherits it without embedding secrets in the Target field.

---

## Quick start (macOS)

Pre-built binaries are provided for **Apple Silicon** (M1 and later). Intel Macs and Linux can [build from source](#build-and-run-from-source).

1. **Install** with the shell installer (puts `mywhisper` in `~/.cargo/bin`, which the installer adds to your `PATH`):

   ```sh
   curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mathiasdahl/mywhisper/releases/latest/download/mywhisper-installer.sh | sh
   ```

   Or download `mywhisper-aarch64-apple-darwin.tar.xz` from the release, extract it, and move `mywhisper` somewhere on your `PATH`. Files downloaded through a browser are quarantined by Gatekeeper; the binary is not notarized, so clear the flag once:

   ```sh
   xattr -d com.apple.quarantine /path/to/mywhisper
   ```

2. **OpenRouter API key** — add it to your shell profile (e.g. `~/.zshrc`) and open a new terminal:

   ```sh
   export OPENROUTER_API_KEY="sk-or-v1-..."
   ```

3. **Run** `mywhisper` (or `mywhisper --dry-run` to try it without a mic or key). The first real recording makes macOS ask whether **your terminal app** may use the microphone—allow it. If you denied it earlier, enable it under **System Settings › Privacy & Security › Microphone**; mywhisper reports “microphone input was completely silent” when access is blocked.

Recordings and transcripts are saved under `~/Library/Application Support/mywhisper/recordings/`. Everything else (flags, keys, examples) is the same as on Windows—just drop the `.\` and `.exe`.

### Quick launch on macOS

Any launcher that can run a shell command on a hotkey works. Examples:

- **Raycast** / **Alfred** script command, or a **Shortcuts.app** “Run Shell Script” action bound to a keyboard shortcut, running:

  ```sh
  osascript -e 'tell application "Terminal" to do script "mywhisper -i -y --clipboard; exit"'
  ```

- **Hammerspoon** (`~/.hammerspoon/init.lua`), using iTerm or Ghostty instead if you prefer:

  ```lua
  hs.hotkey.bind({"cmd", "alt"}, "D", function()
    hs.execute([[osascript -e 'tell application "Terminal" to do script "mywhisper -i -y --clipboard; exit"']])
  end)
  ```

Terminal's `do script` runs your normal login shell, so `OPENROUTER_API_KEY` from `~/.zshrc` is picked up. If you instead run mywhisper directly from a launcher (no terminal window), wrap it in `zsh -lc '…'` so the profile is read.

---

## Command-line options

| Flag | Description |
|------|-------------|
| `-l`, `--language <code>` | Language hint (ISO-639-1, e.g. `en`, `sv`) |
| `-i`, `--immediate` | Start recording as soon as the program is ready |
| `-y`, `--auto-send` | Transcribe immediately when recording stops |
| `--stdout` | Write transcript to stdout (default if no output flags) |
| `--clipboard` | Copy transcript to the system clipboard |
| `--dry-run` | No mic and no API; simulates record and uses mock text (`OPENROUTER_API_KEY` not required) |
| `--dry-run-text <TEXT>` | Transcript for `--dry-run` (default: `[dry-run] Mock transcript for UI testing.`) |
| `--color <auto\|always\|never>` | When to color stderr (default: `auto`) |
| `-h`, `--help` | Print usage |

With `-y` and no `--stdout` / `--clipboard`, delivery uses the same default as stdout and skips the **s** / **c** / **q** prompt.

**`--dry-run`:** **Enter** starts (unless `-i`), **Esc** stops; **Enter** or **Space** pauses/unpauses while a timer runs on stderr. With `--dry-run -i`, simulated recording starts immediately. No files are written in dry-run mode.

---

## Keyboard controls (interactive mode)

| When | Key |
|------|-----|
| Before recording | **Enter** — start |
| While recording | **Enter** / **Esc** — stop |
| While recording | **Space** — pause / resume |
| After recording | **Enter** — send for transcription |
| After recording | **Esc** — discard |
| After transcription (without `-y`) | **s** — stdout · **c** — clipboard · **q** / **Esc** — no output |

Use `--stdout` and/or `--clipboard` to skip the **s** / **c** / **q** step when appropriate.

---

## Requirements

- **Windows** (x64) or **macOS** (Apple Silicon for pre-built binaries) with microphone access for the terminal you use (Windows privacy settings, or macOS *Privacy & Security › Microphone*).
- **Linux** works when built from source (see below).
- An [OpenRouter](https://openrouter.ai/) API key for real transcription (not needed for `--dry-run`).

---

## For developers

### Build and run from source

- Rust toolchain (see `rust-toolchain.toml` in the repo).
- **Linux only:** ALSA development headers and `pkg-config` (Debian/Ubuntu: `sudo apt install libasound2-dev pkg-config`). On X11/Wayland the clipboard belongs to the process that set it, so `--clipboard` text disappears when mywhisper exits unless a clipboard manager is running—prefer `--stdout` piped to `wl-copy` / `xclip -selection clipboard`.

On macOS or Linux, `cargo install --git https://github.com/mathiasdahl/mywhisper` builds and installs it in one step.

```powershell
git clone https://github.com/mathiasdahl/mywhisper.git
cd mywhisper
$env:OPENROUTER_API_KEY = "sk-or-v1-..."
cargo run
cargo run -- -i -y
cargo run -- --dry-run
```

Release binary:

```powershell
cargo build --release
# optional: strip target\release\mywhisper.exe
```

The installed/command name from Cargo is `mywhisper` (`target\release\mywhisper.exe` on Windows, `target/release/mywhisper` elsewhere).

### Publishing a release

After your changes are on `main`:

1. Optionally bump `version` in `Cargo.toml` so the tag matches what you ship.
2. Create and push a version tag (`v*` triggers [`.github/workflows/release.yml`](.github/workflows/release.yml)):

```powershell
git tag v0.1.1
git push origin v0.1.1
```

The workflow is generated by [cargo-dist](https://opensource.axo.dev/cargo-dist/) from `dist-workspace.toml`—do not hand-edit it; change the config and run `dist generate`. Run `dist plan` locally to preview the artifacts. GitHub Actions builds Windows x64 and macOS arm64, uploads `mywhisper-x86_64-pc-windows-msvc.zip`, `mywhisper-aarch64-apple-darwin.tar.xz`, shell/PowerShell installers and SHA256 checksums, and opens a release. Share the [releases](https://github.com/mathiasdahl/mywhisper/releases) page—users do not need the source tree.

---

## Security & privacy

This is a convenience recorder, not a certified confidential transcription product. Before you record anything sensitive, skim the points below.

- **API key** — Set `OPENROUTER_API_KEY` in your environment (or another secret store you trust). Do not commit it; the repo ignores `.env` and similar files.
- **Cloud transcription** — When you send a recording, the **WAV is uploaded to [OpenRouter](https://openrouter.ai/)** for Whisper. Follow your organization’s rules on third-party APIs and data you are allowed to send off the machine.
- **Local copies** — Every run saves matching `.wav` and `.txt` files under `%LOCALAPPDATA%\mywhisper\recordings\` (Windows), `~/Library/Application Support/mywhisper/recordings/` (macOS) or `~/.local/share/mywhisper/recordings/` (Linux). They remain on disk until you delete them; backup or sync software may copy that folder.
- **Clipboard and stdout** — Transcripts can go to the clipboard or stdout. Other tools, logs, or people with access to your session are outside this app’s control.
- **stderr** — Status output can include your **microphone device name** and cost estimates. That stays on your machine but may show up in terminal recordings or shared screens.

---

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
