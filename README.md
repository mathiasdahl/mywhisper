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

## Release binary

```powershell
cargo build --release
# optional: strip target\release\mywhisper.exe
```

Share `target\release\mywhisper.exe` with others; they still need their own `OPENROUTER_API_KEY`.
