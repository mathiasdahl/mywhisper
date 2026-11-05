# mywhisper

Record voice from your microphone and transcribe it with [OpenAI Whisper Large V3 Turbo](https://openrouter.ai/openai/whisper-large-v3-turbo) via [OpenRouter](https://openrouter.ai/).

## Requirements

- Rust (see `rust-toolchain.toml`)
- Windows microphone access
- An [OpenRouter](https://openrouter.ai/) API key

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

Example: hands-off capture and transcription:

```powershell
cargo run -- -i -y
```

## Controls (interactive mode)

| When | Key |
|------|-----|
| Before recording | **Enter** — start |
| While recording | **Enter** / **Esc** — stop |
| While recording | **Space** — pause / resume |
| After recording | **Enter** — send for transcription |
| After recording | **Esc** — discard |
| After transcription (without `-y`) | **s** — stdout · **c** — clipboard · **q** / **Esc** — no output |

With `-y`, use `--stdout` and/or `--clipboard` to choose output channels (stdout is the default when neither flag is given).

Recordings and transcripts are always saved under `%LOCALAPPDATA%\\mywhisper\\recordings\\` as matching timestamped `.wav` and `.txt` files. Status and cost estimates go to **stderr**.

## Releases

Pre-built Windows binaries are on [GitHub Releases](https://github.com/mathiasdahl/mywhisper/releases). Download `mywhisper-windows-x64.zip`, extract `mywhisper.exe`, set `OPENROUTER_API_KEY`, and run.

Each release lists a SHA256 checksum for the zip file.

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
