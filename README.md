# TypeSpeak

Open-source, Windows-first, local-first Lebanese Arabic and English voice typing.

Developed by [NABILNET.AI](https://nabilnet.ai).

TypeSpeak records a short utterance, converts it to a 16 kHz mono WAV, routes it to the model assigned to **Arabic**, **English**, or **Mixed**, optionally translates the result locally, and can paste the final text into the Windows field that was active when recording started.

The local paths use `whisper.cpp` and the bundled CrispASR runtime. Optional model connections can point to an OpenAI-compatible transcription endpoint running on `localhost` or to a cloud API. Cloud use is explicit and the UI warns when the selected route uploads audio.

## Model routing

Each route stores two independent choices: its speech model and its output language. A user can configure, for example:

- Arabic → Whisper local.
- English → Whisper local.
- Mixed → Cohere Transcribe Arabic local.
- Arabic output → English.
- English output → English.
- Mixed output → Mixed · عربي + EN.

Changing one route does not change the other two. `Mixed · عربي + EN` means TypeSpeak does not run translation and asks the selected speech model to retain both scripts. Arabic-to-Arabic and English-to-English also bypass translation. Selecting a different target language uses the local M2M100 translator; selecting it for the first time offers the one-time model download.

Supported connection types:

- Native `whisper.cpp`.
- Managed GGUF models through the bundled CrispASR runtime.
- Local multipart `/audio/transcriptions` endpoint, including runtimes that expose that route directly or through a wrapper.
- Remote multipart transcription API.

API keys are kept in memory for the current session and are not saved to local storage. A custom managed model can be added with a Hugging Face file link or a direct HTTPS `.gguf` URL, a CrispASR backend name, and an optional SHA-256 checksum.

## Managed local catalog

Selecting an uninstalled catalog model asks for confirmation, downloads it, shows progress, resumes an interrupted partial download, verifies its known size and SHA-256, and then assigns it to that route.

| Model | Download | Primary role |
|---|---:|---|
| Whisper large-v3-turbo Q5_0 | 574 MB | Default multilingual Arabic and English model |
| Cohere Transcribe Arabic Q4_K | 1.51 GB | Strong Arabic, Lebanese/Levantine, and Arabic-English code-switch candidate |
| Qwen3-ASR 0.6B Q4_K | 631 MB | Lightweight multilingual model with language detection |
| OmniASR CTC 300M Q4_K | 204 MB | Smallest managed multilingual download |
| M2M100 418M Q8_0 | 526 MB | Local text translation among 100 languages |

Managed weights are stored under `%LOCALAPPDATA%\TypeSpeak\models`. TypeSpeak also accepts normal Hugging Face `blob` links and converts them to downloadable `resolve` links automatically. The bundled native runtime is [CrispASR](https://github.com/CrispStrobe/CrispASR); the catalog weights come from the corresponding `cstr` GGUF repositories on Hugging Face.

## Default local model

The default is **Whisper large-v3-turbo Q5_0**:

- Multilingual Arabic and English recognition with automatic language detection.
- Practical 574 MB quantized model for Windows CPU inference.
- Downloaded on demand to `%LOCALAPPDATA%\TypeSpeak\models`; it is never bundled inside a production installer.
- One runtime family for the future Windows, Android, and iOS versions.
- Personal dictionary terms are supplied as an initial decoding prompt.

The higher-accuracy `large-v3 Q5_0` model can be tested later on powerful PCs, but it is roughly 1.08 GB and slower. Mistral Voxtral Realtime is a future high-end GPU candidate; its 4B-parameter footprint is not the default for a cross-platform consumer app.

## What works

- Hold-only global push-to-talk with a user-configurable shortcut (`Ctrl+Alt+Space` by default).
- Single-key shortcuts including `Ins`, `Prt Scr`, F1–F24, navigation, numpad, volume, and media buttons.
- Quick taps retain the selected button's normal Windows action; holding for 280 ms starts dictation.
- No speech-model weights in either installer. Download only the models you choose from TypeSpeak.
- A click-through, microphone-reactive listening pill at the bottom of the active screen.
- Microphone capture and local 16 kHz PCM WAV encoding.
- Explicit **Arabic**, **English**, and **Mixed** modes.
- Independent **Arabic**, **English**, and **Mixed** model assignments.
- Independent output language for each route: keep the transcript or translate it.
- Local M2M100 translation to 100 supported languages.
- One-click managed Cohere Arabic, Qwen3-ASR, and OmniASR downloads.
- Custom managed GGUF download from Hugging Face or another direct HTTPS URL.
- Download progress, SHA-256 verification, and interrupted-download resume.
- Add/configure a local endpoint or cloud transcription API.
- Offline `whisper.cpp` transcription with no external network call.
- A warm local `whisper-server` process, so the model loads once instead of once per utterance.
- Automatic CUDA acceleration on supported NVIDIA GPUs, with the CPU runtime retained as a fallback.
- Offline CrispASR transcription after the selected weight is installed.
- Browser-preview offline demo for UI testing.
- Verbatim and deterministic light-cleanup modes.
- A persistent local personal dictionary with search and removal.
- A persistent, optional local Recent page with dictation, word, and voice-time metrics.
- A labeled desktop navigation shell with Dictate, Recent, Dictionary, Shortcuts, and Settings pages.
- Close-to-system-tray behavior that keeps push-to-talk active in the background.
- Windows target-window capture, clipboard paste, and text clipboard restoration.
- Temporary audio and transcript files are deleted after each local run.
- No saved audio. Transcript history is local-only and can be disabled or cleared.

## Install the offline engine

Prerequisites:

- Target environment: Windows 10 or 11 with WebView2.
- Rust toolchain (verified with Rust 1.93).
- Microsoft C++ Build Tools.
- Around 700 MB of free disk space for CPU-only setup, or about 2 GB when the NVIDIA CUDA runtime is installed.

From the project folder:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\setup-local-whisper.ps1
```

The setup script detects NVIDIA GPUs automatically and installs the official CUDA 12.4 runtime into `runtime-cuda\`. Pass `-CpuOnly` to skip the larger GPU runtime download.

The setup script downloads:

- Official `whisper.cpp` Windows x64 runtime v1.9.1 into `runtime\`.
- On supported NVIDIA systems, the official CUDA 12.4 runtime into `runtime-cuda\`.
- `ggml-large-v3-turbo-q5_0.bin` into `models\`.

The model download is verified against SHA-256:

```text
394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2
```

The network is only needed for the one-time model installations. Dictation and translation stay offline whenever local routes are selected.

## Install the Windows app

The production build bundles the CPU and NVIDIA CUDA runtimes, but **no speech-model weights**. Install either artifact like a normal Windows app:

- `TypeSpeak_0.1.0_x64-setup.exe` — recommended per-user installer. It asks whether to download the 574 MB default Whisper model now. Choosing **Yes** opens TypeSpeak and displays the download progress; choosing **No** keeps the installer lightweight and leaves a **Download** button beside Whisper inside the app.
- `TypeSpeak_0.1.0_x64_en-US.msi` — Windows Installer package for managed deployment. It includes no speech models; use the in-app **Download** button after deployment.

Closing the window hides TypeSpeak in the Windows system tray and keeps the global shortcut active. Use the tray menu to open Dictate, Recent, or Settings, or choose **Quit TypeSpeak** to stop it completely.

To build both installers from source:

```powershell
cargo install tauri-cli --version "^2" --locked
cargo tauri build --bundles nsis,msi --ci
```

Generated artifacts are written under `src-tauri\target\release\bundle\`.

## Run from source

For normal use, double-click:

```text
Start-TypeSpeak.cmd
```

Do **not** open `src\index.html` directly in Chrome. That file is only a UI preview:
Chrome can record microphone audio, but it cannot launch the native speech runtime,
use the global shortcut, or insert text into other Windows applications.

For development:

```powershell
cargo run --manifest-path .\src-tauri\Cargo.toml
```

After changing Rust or frontend source, rebuild before using the launcher because `Start-TypeSpeak.cmd` opens the existing debug executable when one is already present:

```powershell
cargo build --manifest-path .\src-tauri\Cargo.toml
```

Optional custom locations:

```powershell
$env:TYPESPEAK_WHISPER_EXE="D:\speech\whisper-cli.exe"
$env:TYPESPEAK_WHISPER_MODEL="D:\speech\ggml-large-v3-turbo-q5_0.bin"
cargo run --manifest-path .\src-tauri\Cargo.toml
```

## Use push-to-talk

1. Put the cursor in a normal Windows text field.
2. Hold the configured shortcut (`Ctrl+Alt+Space` by default) for about 0.3 seconds.
3. Speak in Lebanese Arabic, English, or both.
4. Release the shortcut.
5. TypeSpeak runs the model assigned to that route, applies its output-language choice, and pastes the final text into the captured field.

The in-app recording button is intended for testing and preview rather than automatic insertion.
The listening pill appears while push-to-talk is held and does not take keyboard focus.
To change the shortcut, open **Shortcuts → Change shortcut**, then press a standard
Windows key or key combination. `Ins`, `Prt Scr`, function keys, navigation keys,
numpad, volume, and media keys are supported. A quick tap keeps the key's normal
behavior; only a hold activates TypeSpeak. The app saves the new shortcut only
after Windows registers it successfully.

TypeSpeak starts warming the local Whisper model after launch. The first run can
still wait for warm-up to finish; later runs reuse the loaded model.

For Whisper, `Mixed` is intentionally Arabic-first. TypeSpeak sends the Arabic language token, a bilingual script prompt, and no Whisper translate flag. Whisper can still mishandle code-switching; Cohere Arabic is the stronger Mixed candidate. Use the separate output selector when translation is actually wanted.

## Test

```powershell
cargo test --manifest-path .\src-tauri\Cargo.toml
node --check .\src\app.js
node --check .\src\overlay.js
node .\scripts\verify-ui.mjs
```

To preview only the interface:

```powershell
node .\scripts\serve.mjs
```

Then open `http://127.0.0.1:4173/`. Browser preview uses the built-in offline demo because a browser tab cannot launch the native `whisper.cpp` binary.

## Current POC limits

- This pass sends the completed recording to the selected local process or API; partial live text is not implemented yet.
- Managed model weights are downloaded on demand rather than bundled in the repository.
- A custom managed GGUF must use an architecture supported by the bundled CrispASR build and the correct backend name.
- Generic endpoint models must expose an OpenAI-compatible multipart transcription endpoint returning a JSON `text` field.
- Cloud API keys are session-only and must be entered again after restarting the app.
- First transcription is slower because the model must be loaded into memory.
- Whisper can still miss Lebanese-English code-switches.
- Mixed-route translation currently chooses Arabic or English as the predominant source script before running M2M100. Sentence-level multilingual routing is future work.
- Light cleanup only collapses whitespace and immediate duplicate tokens. Translation is a separate, explicit output step.
- Clipboard restoration works when the previous clipboard content is text.
- Windows may block insertion into an elevated application when TypeSpeak runs at a lower privilege level.
- Password and secure-field detection must be added before production use.

## License

TypeSpeak is open-source software released under the [MIT License](./LICENSE), developed by [NABILNET.AI](https://nabilnet.ai). You can use, modify, distribute, and extend it with new speech models and languages.
