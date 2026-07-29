# Speakly/Genspark Voice-Typing Research and Product Plan

**Research date:** 2026-07-28
**Product goal:** Build a clean-room, category-equivalent voice-typing assistant for Windows first, followed by Android and iOS. The product must handle Lebanese Arabic, English spoken with a Lebanese accent, and Arabic-English code-switching.

> This plan targets functional parity with the product category. It does not recommend copying Genspark/Speakly source code, branding, visual assets, proprietary prompts, or copyrighted wording.

## Executive recommendation

**Product constraint update:** local transcription remains the privacy-first default, but every model may also have an explicit API connection. The user chooses Local or API; a local route must never switch to cloud silently.

The practical default is **Whisper large-v3-turbo Q5_0 through whisper.cpp**:

- Multilingual Arabic and English recognition with automatic language detection.
- A 574 MB quantized model, versus roughly 1.08 GB for large-v3 Q5_0.
- CPU-only Windows support plus optional hardware acceleration.
- The same runtime family supports Windows, Android, and iOS.
- Initial-prompt support for names and product vocabulary.

Sources: [whisper.cpp](https://github.com/ggml-org/whisper.cpp), [official converted models](https://huggingface.co/ggerganov/whisper.cpp), [Whisper large-v3-turbo Q5_0](https://huggingface.co/ggerganov/whisper.cpp/blob/main/ggml-large-v3-turbo-q5_0.bin).

Mistral’s Apache-2.0 **Voxtral Realtime 4B** is the most interesting future local streaming candidate and includes Arabic among its supported languages. Its much larger memory and GPU requirements make it a high-end mode, not the cross-platform default. Source: [Voxtral Transcribe 2](https://mistral.ai/news/voxtral-transcribe-2/).

For the Windows MVP, use this flow:

```text
Hold hotkey
  → record audio locally
  → release hotkey
  → convert to 16 kHz mono PCM
  → local whisper.cpp inference
  → deterministic light cleanup
  → insert at the active cursor
```

This matches the press-hold-release interaction without requiring streaming in version 1. The Lebanese benchmark remains necessary because broad Arabic results can hide weaknesses in Lebanese pronunciation, English names, technical vocabulary, and code-switching.

## 1. What Speakly currently does

The official product site positions Speakly as an AI voice-to-text layer that works across macOS, Windows, iOS, and Android. Its public workflow and feature set are documented on the [Speakly site](https://speakly.ai/) and in the [Genspark Speakly help center](https://www.genspark.ai/helpcenter/speakly).

### Core interaction

- The user presses and holds a configurable keyboard shortcut, speaks, and releases it to insert text at the current cursor.
- A hands-free mode starts and stops recording with separate presses.
- A double press opens an agent-style mode.
- The result is intended to work system-wide rather than inside one editor.
- The service is cloud-based; its help center says offline use is not supported.

### Transcript processing

- Removes filler words and repeated false starts.
- Corrects basic grammar and punctuation.
- Converts spoken structure into paragraphs and lists.
- Supports custom modes such as translation, professional rewrite, and terminal help.
- Can transform selected text after a spoken instruction.
- Supports chained instructions and replacement of the selected text.

### Personalization

- Personal dictionary for names, technical terms, and uncommon words.
- Automatic dictionary suggestions after a user correction.
- Cross-device dictionary sync.
- The help center states that the dictionary supports up to 8,000 entries.

### Meetings

- Captures microphone and system audio.
- Produces speaker labels, timestamps, summaries, and notes.
- Supports file import and live translation.

### Public product claims

- More than 100 languages.
- Mixed-language dictation within one utterance.
- Desktop and mobile availability.
- A seven-day trial and a free allowance of 4,000 words per week are described in the help center; paid plan details can change and should be rechecked before competitive pricing decisions.

The Android listing was at 10K+ downloads, 4.0 stars, and 51 reviews when reviewed. A public review specifically complained about incomplete Arabic letters and a non-standard Arabic keyboard layout; Genspark replied in June 2026 that it had fixed the issue. This is anecdotal evidence, but it shows that Arabic input quality and keyboard behavior are visible product risks. Source: [Google Play listing](https://play.google.com/store/apps/details?id=ai.mainfunc.speakly).

## 2. Opportunity: where our product can be better

The product should not be “Speakly with a different logo.” A credible wedge is **Lebanese-first bilingual voice typing**:

- Preserve Lebanese dialect instead of silently rewriting it into Modern Standard Arabic.
- Keep embedded English terms in Latin characters.
- Support Arabic-script and Arabizi output modes.
- Learn bilingual names and product vocabulary.
- Offer explicit **Arabic**, **English**, and **Mixed** modes, each with its own model assignment.
- Separate **Verbatim** from **Polished** output.
- Make every AI rewrite undoable.
- Display low-confidence words without inventing replacements.
- Provide reliable insertion fallbacks for desktop applications that reject simulated typing or paste.
- Publish a precise audio-retention policy and make transcript-history storage opt-in.

Speakly’s live privacy page is a general MainFunc/Genspark policy rather than a voice-dictation-specific data contract. It says prompts and outputs may be processed by external providers such as OpenAI or Anthropic and describes Azure storage and US processing in general terms. The Play listing says the company does not store or sell what a user says, while its data-safety disclosure also mentions audio collection or sharing. These statements may describe different processing stages, but the ambiguity is itself a product opportunity. Sources: [Speakly privacy policy](https://speakly.ai/privacy), [Google Play listing](https://play.google.com/store/apps/details?id=ai.mainfunc.speakly).

## 3. Model research

### Recommended shortlist

| Model | Lebanese / Arabic fit | Streaming | Deployment | Important caveat | MVP role |
|---|---|---:|---|---|---|
| **Cohere Transcribe Arabic 07-2026** | Purpose-built for dialectal Arabic, Arabic-English code-switching, and Arabic-accented English | No native streaming documented | Cohere API, Model Vault, or open weights | File API; no timestamps or diarization; eager to transcribe noise | **Final transcript winner to test first** |
| **Deepgram Nova-3 Arabic** | Explicit `ar-LB` support and keyterm prompting | Yes | Cloud API | Arabic-English code-switch quality needs a Lebanese bakeoff | **Live partial-text candidate** |
| **Mistral Voxtral Realtime** | Arabic and English among 13 supported languages | Yes, configurable sub-200 ms | API or Apache-2.0 open weights | No Lebanese-specific public benchmark | **Best open streaming candidate** |
| **Mistral Voxtral Mini Transcribe V2** | General Arabic/English multilingual batch ASR | Batch | Cloud API | Generic multilingual result is not a Lebanese result | Alternative final pass |
| **Azure Speech `ar-LB`** | Explicit Lebanese locale; Custom Speech available | Yes | Azure | No neutral public WER proving it is best | Enterprise/custom-vocabulary candidate |
| **Google Chirp 3 `ar-LB`** | Explicit Lebanese locale | Yes | Google Cloud | `ar-LB` was listed as Preview | Evaluation candidate |
| **ElevenLabs Scribe v2 Realtime** | Arabic among 90+ languages | Yes, about 150 ms claimed | Cloud API | Provider describes Arabic as “Good,” not dialect-specific | Fast comparison candidate |
| **OpenAI `gpt-realtime-whisper`** | Multilingual, streaming, simple API | Yes | OpenAI API | No Lebanese-specific public benchmark | Integration-friendly fallback |
| **OpenAI `gpt-4o-transcribe`** | Higher-accuracy multilingual batch ASR than original Whisper | Batch | OpenAI API | No Lebanese-specific public benchmark | Integration-friendly final-pass candidate |
| **Whisper Large V3 / Turbo** | Established multilingual baseline; can run locally | Pseudo-streaming | Local or hosted | Current Arabic-specialized systems can outperform it | Offline baseline, not default winner |

### 3.1 Cohere Transcribe Arabic 07-2026

Released on 2026-07-07, this is a 2B-parameter FastConformer encoder plus autoregressive Transformer decoder under Apache 2.0. It is explicitly trained for Arabic dialect diversity, Arabic-English code-switching, and Arabic-accented English. It preserves English tokens in Latin script inside Arabic sentences.

Cohere publishes these average WER results:

| Model | Average WER reported by Cohere |
|---|---:|
| Cohere Transcribe Arabic 07-2026 | 25.87 |
| OmniASR LLM 7B | 28.32 |
| Cohere Transcribe base | 30.67 |
| Whisper Large V3 | 36.86 |

Cohere also reports 19.16 WER for Levantine and 27.84 WER for Arabic-English code-switching in its internal dialect evaluation. Human evaluators in that internal evaluation preferred its transcript to Whisper in 95.8% of comparisons. These values are useful for prioritization, but they are **vendor self-reported and not directly comparable to another provider’s benchmark**.

Operational limitations:

- The API accepts an audio file rather than a streaming session.
- The model expects an `ar` or `en` language tag. For mixed speech, select the dominant language: `ar` for mostly Lebanese Arabic with embedded English and `en` for mostly English.
- It may transcribe non-speech noise, so add voice activity detection and a noise gate.
- It does not currently provide timestamps or diarization.
- The documented API accepts multipart uploads up to 25 MB.

Sources: [release](https://cohere.com/blog/transcribe-arabic), [changelog](https://docs.cohere.com/changelog/transcribe-arabic), [quickstart](https://docs.cohere.com/v2/docs/audio-transcription-quickstart), [model card](https://huggingface.co/CohereLabs/cohere-transcribe-arabic-07-2026).

### 3.2 Deepgram Nova-3 Arabic

Nova-3 Arabic is a production real-time and batch option with 17 documented Arabic variants, including Lebanese Arabic (`ar-LB`), and keyterm prompting for names or domain vocabulary. Deepgram reports up to roughly 40% lower WER than competing systems across its Arabic tests. That claim is vendor-published, and Deepgram’s multilingual/code-switch path must be tested specifically with Arabic-English Lebanese speech before selection.

Source: [Deepgram Nova-3 Arabic announcement](https://deepgram.com/learn/nova-3-arabic-speech-to-text-production-grade-stt).

### 3.3 Mistral Voxtral Transcribe 2

Mistral released Voxtral Transcribe 2 on 2026-02-04:

- **Voxtral Realtime:** 4B parameters, Apache 2.0 open weights, Arabic and English support, configurable latency below 200 ms, and a published API price of $0.006 per minute.
- **Voxtral Mini Transcribe V2:** batch transcription with timestamps, diarization, and context biasing at a published $0.003 per minute.

Mistral reports about 4% average WER for Mini V2 on FLEURS, but that is a broad multilingual benchmark rather than proof of Lebanese performance.

Source: [Mistral Voxtral Transcribe 2](https://mistral.ai/news/voxtral-transcribe-2/).

### 3.4 ElevenLabs, Microsoft, Google, and OpenAI

- **ElevenLabs Scribe v2 Realtime:** about 150 ms claimed latency and 90+ languages. The official documentation groups Arabic in its “Good” accuracy band rather than publishing a Lebanese number. [Documentation](https://elevenlabs.io/docs/overview/capabilities/speech-to-text/), [API pricing](https://elevenlabs.io/pricing/api).
- **Azure Speech:** provides an `ar-LB` locale and Custom Speech, making it attractive if a private Lebanese data set is later used for adaptation. [Language support](https://learn.microsoft.com/azure/ai-services/speech-service/language-support).
- **Google Chirp 3:** supports streaming and lists `ar-LB`, but Lebanese was listed as Preview when researched. [Chirp 3 documentation](https://cloud.google.com/speech-to-text/docs/models/chirp-3).
- **OpenAI `gpt-realtime-whisper`:** streaming transcription at a published $0.017 per minute. [Model documentation](https://developers.openai.com/api/docs/models/gpt-realtime-whisper).
- **OpenAI `gpt-4o-transcribe`:** a batch model that OpenAI reports improves word error rate, accents, and noise handling over original Whisper. [Model documentation](https://developers.openai.com/api/docs/models/gpt-4o-transcribe), [audio-model announcement](https://openai.com/index/introducing-our-next-generation-audio-models/).

### 3.5 Transcript cleanup model

ASR and transcript cleanup should be separate services. Start with a fast multilingual language model behind a replaceable interface; a current low-latency candidate is **GPT-5.6 Luna**, but there is no public Lebanese-specific cleanup benchmark proving it is the permanent winner. The post-processor must be evaluated on meaning preservation, not prose fluency. [Current OpenAI model guidance](https://developers.openai.com/api/docs/guides/latest-model).

### 3.6 Lightweight local candidates — July 2026 update

“Small parameter count” and “small practical installation” are not the same. Runtime, precision, tokenizer, audio encoder, and quantization determine the real disk/RAM cost.

| Model | Official footprint / runtime | Arabic evidence | Best TypeSpeak role | Decision |
|---|---|---|---|---|
| **Whisper base / small** | 142 MiB / 466 MiB GGML; multilingual; `whisper.cpp` runs on Windows, Android, and iOS | General multilingual Arabic, not Lebanese-specialized | Low-end local fallback | Add as managed lightweight variants |
| **Whisper large-v3-turbo Q5_0** | 547 MiB GGML | Stable multilingual baseline | Balanced local default today | Keep |
| **Meta OmniASR CTC 300M family** | 325M params; a Q4_K GGUF is about 204 MB; Apache 2.0 | Arabic is covered through the 1,600+ language family, but no Lebanese-specific public result was found | Experimental fast multilingual local route | Implemented as the smallest managed catalog option; benchmark before default |
| **Qwen3-ASR 0.6B** | 0.6B model; a Q4_K GGUF is about 631 MB; Apache 2.0 | Arabic is one of 30 languages and automatic language identification is supported; the official dialect list does not claim Lebanese | Mid-size multilingual / language-ID candidate | Implemented as a managed local option |
| **Cohere Transcribe Arabic 07-2026** | 2B; a Q4_K imatrix GGUF is about 1.51 GB; Apache 2.0 | Purpose-built for major Arabic dialects, Arabic-accented English, and Arabic-English speech | High-accuracy Arabic and Mixed candidate | Implemented through CrispASR; first Arabic bakeoff model, not the light option |
| **Audar ASR V1 Turbo Q4** | ~1.28 GB decoder plus required ~0.64 GB BF16 audio projector; llama.cpp; 2.35B total | Vendor reports Levantine and code-switch training and 24.8 average WER | Accuracy challenger for Arabic/Mixed | Research-only until license and independent benchmark review |
| **Official Distil-Whisper** | 166M–756M depending English checkpoint | Official project still says English-only | Optional lightweight English-only route | Do not present as Arabic |

Sources: [whisper.cpp model sizes](https://github.com/ggml-org/whisper.cpp/blob/master/models/README.md), [Meta Omnilingual ASR](https://github.com/facebookresearch/omnilingual-asr), [Omnilingual ASR paper](https://arxiv.org/abs/2511.09690), [Qwen3-ASR 0.6B](https://huggingface.co/Qwen/Qwen3-ASR-0.6B), [Cohere Arabic model card](https://huggingface.co/CohereLabs/cohere-transcribe-arabic-07-2026), [Audar ASR V1 Turbo](https://huggingface.co/audarai/Audar-ASR-V1-Turbo), [Distil-Whisper](https://huggingface.co/distil-whisper).

Practical recommendation:

- **Weak CPU / mobile:** Whisper base or small quantized. It is the easiest truly small cross-platform package, but Lebanese accuracy must be accepted as lower.
- **Balanced PC:** Whisper large-v3-turbo Q5_0.
- **Arabic accuracy on a capable PC:** Cohere Transcribe Arabic local.
- **Experimental middle tier:** Qwen3-ASR 0.6B Q4_K. Its managed package is about 631 MB.
- **OmniASR CTC 300M:** useful for speed and broad language coverage, not yet proven as the best Lebanese model.

### 3.7 Per-language model router

TypeSpeak must not have one global model selector. It stores three independent assignments:

```text
Arabic route  → any installed/connected model
English route → any installed/connected model
Mixed route   → any installed/connected model
```

The user selects Arabic, English, or Mixed before dictation. The selected lane chooses its assigned model. The same model may be assigned to multiple lanes, and changing one lane must not change the others.

An automatic router is a separate future feature because deciding which ASR model to run requires a language-identification pre-pass. Mixed utterances are harder than single-language LID. Until an accented Arabic-English LID component is benchmarked, the UI must not label manual lane selection as automatic detection.

The model registry supports:

- Native runtimes managed by TypeSpeak, currently `whisper.cpp` and CrispASR.
- Managed local GGUF model packages with download progress, interrupted-download resume, and integrity verification.
- User-added Hugging Face or direct HTTPS GGUF links with a selected CrispASR backend and optional SHA-256.
- Arbitrary local OpenAI-compatible transcription endpoints.
- Remote transcription APIs with an explicit cloud/privacy warning.

No local route may silently fall back to a cloud API. API keys should move from the current session-memory POC to the operating-system credential vault before production.

Use two user-selectable modes:

- **Verbatim:** deterministic punctuation, number normalization, dictionary replacement, and minimal false-start removal.
- **Polished:** conservative LLM editing with an undo path.

Core cleanup constraints:

```text
Do not translate.
Preserve Lebanese dialect and the speaker's meaning.
Keep English words in Latin script unless the user requests Arabic transliteration.
Remove only obvious filler words and repeated false starts.
Add punctuation and paragraphs without adding facts.
Apply the supplied personal dictionary exactly.
When uncertain, keep the original words rather than inventing a replacement.
```

Text-only correction can produce fluent but wrong content. For uncertain segments, retain the ASR text or later provide audio-conditioned correction rather than asking an LLM to guess.

### 3.8 Per-route output translation

Speech recognition and translation are separate choices. Each input route stores its own output:

```text
Arabic route  → Arabic by default, or a target language
English route → English by default, or a target language
Mixed route   → Mixed Arabic + English by default, or a target language
```

The default output for each route must never invoke translation. `Mixed · عربي + EN` explicitly disables the translation stage and requests both scripts from the selected ASR model; it cannot repair an ASR model that already translated or mistranscribed speech. The Windows POC uses the 526 MB Q8_0 conversion of **M2M100 418M** through CrispASR for local text-to-text translation among 100 language codes. The model downloads only when a route first needs actual translation. Arabic-to-Arabic and English-to-English selections bypass the translator.

For Mixed input, the current POC counts Arabic-script and Latin letters and sends the predominant language as M2M100's source language. This is deterministic but not a complete code-switch translation strategy. A production version should benchmark sentence- or span-level language identification so English names and Arabic phrases are not incorrectly translated as if they belonged to one source language.

Whisper.cpp also has an open multilingual-audio limitation: a single primary language can produce skipped or unrelated text after language switches. The proposed upstream direction is segment-level language detection, which supports preferring Cohere for the Mixed lane until that pipeline is implemented.

Sources: [M2M100 418M GGUF](https://huggingface.co/cstr/m2m100-418m-GGUF), [whisper.cpp multilingual-audio issue](https://github.com/ggml-org/whisper.cpp/issues/3334).

### 3.9 Implemented Windows managed catalog

The current one-click catalog is intentionally allowlisted. TypeSpeak knows the direct model URL, exact byte size, and SHA-256 before downloading:

| Catalog item | Runtime backend | Download |
|---|---|---:|
| Cohere Transcribe Arabic Q4_K imatrix | `cohere` | 1,510,365,312 bytes |
| Qwen3-ASR 0.6B Q4_K | `qwen3` | 631,026,336 bytes |
| OmniASR CTC 300M v2 Q4_K | `omniasr` | 203,542,816 bytes |
| M2M100 418M Q8_0 | `m2m100` | 526,331,008 bytes |

The bundled CrispASR 0.8.23 Windows CPU runtime was checked locally with `--list-backends-json`; it reports all four backends. Managed weights live under `%LOCALAPPDATA%\TypeSpeak\models`, not in the source tree. Normal Hugging Face `blob` file links are normalized to `resolve` downloads automatically.

Sources: [CrispASR](https://github.com/CrispStrobe/CrispASR), [Cohere Arabic GGUF](https://huggingface.co/cstr/cohere-transcribe-arabic-07-2026-GGUF), [Qwen3-ASR GGUF](https://huggingface.co/cstr/qwen3-asr-0.6b-GGUF), [OmniASR GGUF](https://huggingface.co/cstr/omniASR-CTC-300M-v2-GGUF).

## 4. Lebanese benchmark before committing to a provider

### Dataset

Collect 1,000–2,000 consented, anonymized clips from at least 30 Lebanese speakers. Cover:

- Pure Lebanese Arabic.
- Pure English spoken with a Lebanese accent.
- Natural Arabic-English code-switching.
- Names, brands, technical vocabulary, and abbreviations.
- Dates, currencies, phone numbers, URLs, and email addresses.
- Quiet rooms, cars, cafés, laptop microphones, wired headsets, and Bluetooth.
- Short commands and 30–90 second dictation.
- Beirut, Mount Lebanon, North, South, and Bekaa accents where possible.

Reference transcripts should preserve the intended script: Arabic in Arabic script and embedded English in Latin script. Define one normalization policy before scoring because Arabic diacritics, alef forms, punctuation, and number formats can distort WER.

### Systems to compare

- Cohere Transcribe Arabic 07-2026.
- Deepgram Nova-3 `ar-LB`.
- Mistral Voxtral Realtime and Mini Transcribe V2.
- Azure Speech `ar-LB`.
- Google Chirp 3 `ar-LB`.
- ElevenLabs Scribe v2 Realtime.
- OpenAI `gpt-4o-transcribe`.
- Whisper Large V3 or Turbo as the local baseline.

### Metrics

- Normalized Arabic WER and character error rate.
- English-token retention F1 on code-switched clips.
- Named-entity accuracy.
- Meaning-changing error rate after cleanup.
- User correction rate and keystrokes saved.
- Release-to-insert latency, p50 and p95.
- Real-time factor and provider failure rate.
- Cost per active dictation hour.

### Suggested launch gates

These are product targets, not claims about current models:

- Pure Lebanese normalized WER at or below 15–20%.
- At least 95% correct retention of clear English tokens in code-switched clips.
- p95 release-to-insert latency at or below 1.5 seconds for short utterances.
- Zero known meaning-changing cleanup patterns in the release regression suite.
- No raw-audio retention by default after successful transcription.

## 5. Windows-first architecture

### Technology choice

Use **Tauri 2 + Rust for the Windows shell and system integration**, with a lightweight web UI. The current proof of concept uses plain HTML, CSS, and JavaScript.

Why:

- Small desktop footprint compared with a full browser runtime.
- Rust is suitable for global hotkeys, audio capture, clipboard handling, and native Windows APIs.
- The UI remains quick to build and easy to iterate.
- `whisper.cpp` can run as a local sidecar now and move behind a native Rust/C interface later.

Flutter is viable if the team already has strong Flutter experience, but desktop insertion and both mobile keyboard systems still need native platform work. The realistic shared layer is the backend protocol, account/dictionary model, and possibly a small Rust core—not one universal UI codebase.

### Components

```text
Windows client
├─ Global hotkey manager
├─ Microphone capture
├─ VAD / noise gate
├─ Recording indicator and cancel
├─ Local whisper.cpp and CrispASR engines
├─ Managed quantized speech models
├─ Optional local M2M100 translation
├─ Transcript cleanup
├─ Personal dictionary
├─ Safe insertion adapter
└─ Local settings

Shared local core
├─ Audio preprocessing
├─ Model selection and inference
├─ Dictionary prompting
├─ Deterministic cleanup
└─ Evaluation and regression harness
```

### Safe Windows text insertion

Use ordered fallbacks:

1. Microsoft UI Automation `ValuePattern` or text APIs when the active control supports them.
2. Clipboard plus paste, restoring the prior clipboard contents after insertion.
3. Unicode keyboard input through `SendInput` when paste is unavailable.

The client must detect password and secure fields and refuse to insert or retain content. Windows privilege isolation can also block a normal application from injecting input into an elevated application; this limitation should be reported clearly rather than silently losing text. Reference: [Microsoft `SendInput`](https://learn.microsoft.com/windows/win32/api/winuser/nf-winuser-sendinput).

### MVP product surface

- Press-and-hold dictation.
- Hands-free start/stop.
- Cancel recording.
- Arabic, English, and Mixed routes with independent model assignments.
- Verbatim and Polished modes.
- Personal dictionary.
- Preview/undo after insertion.
- Per-application mode preference.
- Clear microphone and cloud-processing indicators.
- Local history disabled by default or protected with a retention control.
- Crash-safe audio cleanup.

Selection rewrite, meetings, advanced code-switch translation, and full agent actions should follow after the core dictation loop is reliable.

## 6. Android and iOS implications

### Android

Android can provide the closest system-wide mobile experience through a native `InputMethodService` keyboard. The keyboard can expose a microphone button, stream or upload the recording, then commit the transcript into the focused field. Native Kotlin is the lowest-risk choice for the IME even if the containing settings app uses a cross-platform UI.

Official reference: [Android: create an input method](https://developer.android.com/develop/ui/views/touch-and-input/creating-input-method).

### iOS

iOS is the largest platform risk. Apple custom keyboard extensions **cannot access the microphone**. A custom keyboard can insert text through `textDocumentProxy`, and enabling network access/shared containers requires “Allow Full Access,” but the recording itself cannot happen directly inside the keyboard extension.

Therefore the iOS design must use one of these patterns:

- The containing app records and transcribes, then shares the transcript with the keyboard extension through an App Group.
- A Shortcut/App Intent starts dictation in the containing app, followed by clipboard or keyboard insertion.
- The user records inside the main app and pastes or sends the result to the target app.

This will not be identical to Android’s microphone keyboard. It needs to be designed honestly and validated against App Review rules early.

Official references: [Apple: configuring open access for a custom keyboard](https://developer.apple.com/documentation/uikit/configuring-open-access-for-a-custom-keyboard), [Apple custom keyboard limitations](https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/CustomKeyboard.html).

## 7. Privacy and security baseline

- A local route never uploads dictation audio or transcripts; an API route shows an explicit cloud warning before recording.
- Delete each temporary local recording and generated transcript immediately after inference.
- Make transcript history opt-in, with a visible retention period and delete-all action.
- Encrypt dictionary and local history at rest.
- Do not use dictation content for training.
- Never capture or insert into secure/password fields.
- Show an unambiguous recording indicator and provide a hard cancel shortcut.
- Redact transcript and audio contents from normal application logs.
- Verify model downloads with published checksums and keep the inference runtime pinned per release.

## 8. Delivery roadmap

Estimates assume two engineers plus part-time design/QA and begin after the model bakeoff. A solo implementation will usually take materially longer.

### Phase 0 — evaluation and specification: 1–2 weeks

- Define Lebanese transcription conventions.
- Collect the first benchmark set.
- Build the provider evaluation harness.
- Select final and streaming ASR providers.
- Write the privacy/data-flow specification.

### Phase 1 — Windows MVP: 4–6 weeks

- Tauri/Rust shell, global shortcut, capture, VAD, and cancellation.
- Batch final transcription.
- Conservative cleanup.
- Reliable text insertion with fallbacks.
- Settings, dictionary, minimal history, onboarding, and updater.
- Automated regression set for Arabic, English, and mixed speech.

### Phase 2 — Windows quality/product layer: 3–5 weeks

- Live provisional transcription and dual-pass finalization.
- Selection-based rewrite.
- Per-app profiles.
- Dictionary learning after explicit corrections.
- Usage metering and opt-in quality analytics.

### Phase 3 — Android IME: 4–6 weeks

- Native keyboard service.
- Shared account and dictionary.
- Mobile recording and insertion UX.
- Privacy onboarding and Play policy review.

### Phase 4 — iOS app plus keyboard handoff: 5–8 weeks

- Prototype the containing-app/keyboard handoff first.
- App Group transcript sharing.
- Shortcuts/App Intents.
- Full-access education and App Review hardening.

## 9. Major risks

| Risk | Why it matters | Mitigation |
|---|---|---|
| Lebanese code-switch errors | Generic Arabic scores hide English-token corruption | Lebanese benchmark and dual-language token metric |
| LLM “cleanup” changes meaning | Fluent output can conceal recognition mistakes | Verbatim default, constrained prompt, undo, regression tests |
| Cloud latency/outage | Breaks the feeling of typing | Local buffering, provider fallback, clear retry, later local model |
| Windows insertion incompatibility | Some applications reject paste or simulated keys | UI Automation, clipboard, and Unicode fallback chain |
| iOS keyboard microphone restriction | Prevents Android-like UX | Containing app + App Group/Shortcut design; prototype early |
| Sensitive audio/text handling | Voice typing can capture confidential content | Default deletion, no content logs, explicit subprocessors, opt-in history |
| Vendor lock-in | Model leadership and pricing change quickly | Internal ASR interface, normalized output schema, recurring benchmark |
| Literal copying/IP exposure | Creates avoidable legal and product risk | Clean-room functional specification and original brand/design |

## 10. Latest and most relevant research

### 2026

1. **Arab Voices: Mapping Standard and Dialectal Arabic Speech Technology** — Findings of ACL 2026. It organizes 31 datasets across 14 dialects and compares modern Arabic speech tools, making it a useful map of evaluation gaps. [Paper](https://aclanthology.org/2026.findings-acl.575/)
2. **Zero-Shot Context-Aware ASR for Diverse Arabic Varieties** — Findings of ACL 2026. It shows how textual context and retrieved speech-text examples can improve dialect/accent recognition without updating model weights. This supports personal-dictionary and contextual-prompting work. [Paper](https://aclanthology.org/2026.findings-acl.1296/)
3. **Improving Language Identification for Code-Switched Speech: The Pivotal Role of Accented English** — Findings of EACL 2026. It finds that accented-English examples are especially important for Arabic-English code-switch language identification and that parameter-efficient adaptation can help with limited data. [Paper](https://aclanthology.org/2026.findings-eacl.242/)
4. **Linear Semantic Segmentation for Low-Resource Spoken Dialects** — Findings of ACL 2026. It addresses boundaries in conversational, code-switched dialectal speech, directly relevant to turning unstructured dictation into readable sentences. [Paper](https://aclanthology.org/2026.findings-acl.1740/)

### 2025

5. **Dialectal Coverage and Generalization in Arabic Speech Recognition** — ACL 2025. It evaluates wide dialect coverage and code-switching across many Arabic varieties. Its results warn that multilingual training can improve code-switching while introducing interference on monolingual speech. [Paper](https://aclanthology.org/2025.acl-long.1427/)
6. **Octopus: Towards Building the Arabic Speech LLM Suite** — ArabicNLP 2025. It combines Whisper-v3 with Arabic/English language models and synthetic code-switched data for ASR, dialect identification, and speech translation. [Paper](https://aclanthology.org/2025.arabicnlp-main.35/)
7. **ALADAN at IWSLT25 Low-resource Arabic Dialectal Speech Translation Task** — IWSLT 2025. North Levantine experiments using TDNN-F and Zipformer systems show the value of crowdsourced dialect data in low-resource settings. [Paper](https://aclanthology.org/2025.iwslt-1.24/)
8. **NeKo: Cross-Modality Post-Recognition Error Correction with Tasks-Guided Mixture-of-Experts Language Model** — ACL Industry 2025. It uses multi-task mixture-of-experts correction and reports an average relative 5.0% WER reduction on the Open ASR Leaderboard. It supports specialized post-correction research, while the separate ClozeGER paper below provides the stronger evidence for audio-aware correction. [Paper](https://aclanthology.org/2025.acl-industry.17/)
9. **From Conversational Speech to Readable Text: Post-Processing Noisy Transcripts in a Low-Resource Setting** — WNUT 2025. It studies punctuation, capitalization, disfluency removal, and sentence boundaries, and finds that compact language models can be competitive for transcript post-processing. [Paper](https://aclanthology.org/2025.wnut-1.15/)
10. **Omnilingual ASR: Open-Source Multilingual Speech Recognition for 1,600+ Languages** — Meta AI, 2025. Its 300M–7B open models and few-shot language extension are relevant to a future fine-tuned or local path. [Publication](https://ai.meta.com/research/publications/omnilingual-asr-open-source-multilingual-speech-recognition-for-1600-languages/)

### 2024 foundations

11. **Listen Again and Choose the Right Answer: A New Paradigm for Automatic Speech Recognition with Large Language Models** — Findings of ACL 2024. It demonstrates why acoustic evidence matters in correction: text-only LLMs can select a fluent but incorrect transcript. [Paper](https://aclanthology.org/2024.findings-acl.37/)
12. **Casablanca: Data and Models for Multidialectal Arabic Speech Recognition** — EMNLP 2024. It supplies multi-dialect conversational material with code-switch annotations. [Paper](https://aclanthology.org/2024.emnlp-main.1211/)
13. **ZAEBUC-Spoken: A Multilingual Multidialectal Arabic-English Speech Corpus** — LREC-COLING 2024. Its workplace-like Zoom conversations and code-switch annotations are useful when designing realistic evaluation data. [Paper](https://aclanthology.org/2024.lrec-main.1546/)

## 11. Product decision

Proceed with a Windows proof of concept using:

- **Local ASR baseline:** Whisper large-v3-turbo Q5_0 through whisper.cpp.
- **Model routing:** independent Arabic, English, and Mixed assignments.
- **Arabic bakeoff:** Cohere Transcribe Arabic local vs Whisper vs Qwen3-ASR 0.6B vs OmniASR CTC 300M.
- **Extensibility:** native managed runtimes plus local/API transcription endpoints; never silent local-to-cloud fallback.
- **Initial interaction:** press/hold, transcribe on release, then insert.
- **Cleanup:** deterministic whitespace and duplicate-token cleanup; Verbatim mode always available.
- **Desktop shell:** Tauri 2, Rust, and a lightweight web UI.
- **Insertion:** UI Automation → clipboard/paste → Unicode input fallback.
- **Version 2 live text:** test local Voxtral Realtime on high-end hardware and whisper.cpp streaming/VAD paths.
- **Evaluation:** build the Lebanese test set before fine-tuning or changing the default local model.
- **Mobile:** native Android IME; iOS containing app plus keyboard/Shortcut handoff.

The first engineering milestone is a thin Windows prototype that records a short utterance, runs the local model, and inserts the transcript into common applications. The next milestone is a Lebanese Arabic/English benchmark covering code-switching, names, accents, noise, and hardware latency.
