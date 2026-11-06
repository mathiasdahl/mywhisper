# mywhisper

Record voice from your microphone and transcribe it with [OpenAI Whisper Large V3 Turbo](https://openrouter.ai/openai/whisper-large-v3-turbo) via [OpenRouter](https://openrouter.ai/).

## Requirements

- Rust (see `rust-toolchain.toml`)
- Windows microphone access
- An [OpenRouter](https://openrouter.ai/) API key (not needed for `--dry-run`)

## Setup

```powershell
$env:OPENROUTER_API_KEY = "sk-or-v1-..."
```

## Run

```powershell
cargo run
```

### Options

| Flag | Description |
|------|-------------|
| `-l`, `--language <code>` | Language hint (ISO-639-1, e.g. `en`) |
| `-i`, `--immediate` | Start recording as soon as the program is ready |
| `-y`, `--auto-send` | Transcribe immediately when recording stops |
| `--stdout` | Write transcript to stdout (default if no output flags) |
| `--clipboard` | Copy transcript to the Windows clipboard |
| `--dry-run` | No mic and no API; simulates record (Enter to start, Esc to stop, Enter/Space pause) and uses mock text for the rest (no `OPENROUTER_API_KEY`) |
| `--dry-run-text <TEXT>` | Transcript text for `--dry-run` (default: `[dry-run] Mock transcript for UI testing.`) |

Example: hands-off capture and transcription:

```powershell
cargo run -- -i -y
```

Example: test record timing, pause, and output flow without a mic or credits:

```powershell
cargo run -- --dry-run
```

After **Enter** to start, stderr shows an elapsed timer while “recording”; **Esc** stops, then **Enter** continues to mock transcription (same as a real run).

Use `-y` to skip the Enter/Escape confirm and go straight to the output channel step:

```powershell
cargo run -- --dry-run -y
```

### Quick launch (Windows)

Pin a small terminal window on the taskbar and start mywhisper with **Win** + **1**–**9** (slot number).

1. Create a shortcut on your desktop.
2. Set **Target** (adjust the path; requires [Windows Terminal](https://apps.microsoft.com/store/detail/windows-terminal/9N0DX20HK701)):

   ```text
   wt.exe -w -1 --size 120,10 --pos 500,500 C:\path\to\mywhisper.exe
   ```

   Append flags after the exe if you like, e.g. `-i -y` for immediate record and auto-transcribe.

3. Drag the shortcut to the taskbar in slot **1**–**9** (only those slots get **Win** + number).
4. Press **Win** + that number to open mywhisper.

Set `OPENROUTER_API_KEY` in your user environment (see [Setup](#setup)) so the shortcut inherits it.

## Controls (interactive mode)

| When | Key |
|------|-----|
| Before recording | **Enter** — start |
| While recording | **Enter** / **Esc** — stop |
| While recording | **Space** — pause / resume |
| After recording | **Enter** — send for transcription |
| After recording | **Esc** — discard |
| After transcription (without `-y`) | **s** — stdout · **c** — clipboard · **q** / **Esc** — no output |

Use `--stdout` and/or `--clipboard` to choose output channels without the **s** / **c** / **q** prompt (stdout is the default when neither flag is given). With `-y` and no output flags, delivery uses that same default and skips the prompt as well.

**`--dry-run` (without `-y`):** Same as a normal run for **Enter** to start and **Esc** to stop the fake recording. While “recording”, **Enter** or **Space** toggles pause (the timer pauses); stderr prints elapsed time about once per second. With `--dry-run -i`, the simulated recording starts immediately (no initial Enter).

Recordings and transcripts are always saved under `%LOCALAPPDATA%\\mywhisper\\recordings\\` as matching timestamped `.wav` and `.txt` files (normal runs only; `--dry-run` does not write files). Status and cost estimates go to **stderr**.

## Releases

Pre-built Windows binaries are on [GitHub Releases](https://github.com/mathiasdahl/mywhisper/releases). Download `mywhisper-windows-x64.zip`, extract `mywhisper.exe`, set `OPENROUTER_API_KEY`, and run.

Each release lists a SHA256 checksum for the zip file.

### Publishing a release

After your changes are on `main`:

1. Optionally bump `version` in `Cargo.toml` so the tag matches what you ship.
2. Create and push a version tag (`v*` triggers [`.github/workflows/release.yml`](.github/workflows/release.yml)):

```powershell
git tag v0.1.1
git push origin v0.1.1
```

GitHub Actions builds `mywhisper.exe`, uploads `mywhisper-windows-x64.zip`, and opens a release with a SHA256 checksum. Share the [releases](https://github.com/mathiasdahl/mywhisper/releases) page with colleagues—no need to email the binary.

## Build from source

```powershell
cargo build --release
# optional: strip target\release\mywhisper.exe
```

Colleagues can also build locally; everyone needs their own `OPENROUTER_API_KEY`.

## Security & privacy

This is a convenience recorder, not a certified confidential transcription product. Before you record anything sensitive, skim the points below.

- **API key** — Set `OPENROUTER_API_KEY` in your environment (or another secret store you trust). Do not commit it; the repo ignores `.env` and similar files.
- **Cloud transcription** — When you send a recording, the **WAV is uploaded to [OpenRouter](https://openrouter.ai/)** for Whisper. Follow your organization’s rules on third-party APIs and data you are allowed to send off the machine.
- **Local copies** — Every run saves matching `.wav` and `.txt` files under `%LOCALAPPDATA%\mywhisper\recordings\`. They remain on disk until you delete them; backup or sync software may copy that folder.
- **Clipboard and stdout** — Transcripts can go to the clipboard or stdout. Other tools, logs, or people with access to your session are outside this app’s control.
- **stderr** — Status output can include your **microphone device name** and cost estimates. That stays on your machine but may show up in terminal recordings or shared screens.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
