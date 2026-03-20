# Murmur

A privacy-first macOS menu bar dictation app. Click the icon, speak, and your words are pasted directly into whatever you're typing in — no cloud, no telemetry, no subscription.

## Features

- **Menu bar app** — lightweight, always available, never in the Dock
- **Live preview** — parallel on-device speech recognition (zh-CN, zh-TW, en-US) while recording
- **Offline transcription** — final pass via `coli asr` (SenseVoice) bundled inside the app
- **Auto-paste** — transcribed text is pasted into the frontmost app via CGEvent (no AppleScript)
- **LLM Polish** — optional grammar/punctuation cleanup via any OpenAI-compatible API
- **Privacy-first** — microphone audio never leaves your machine

## Requirements

- macOS 14.0+ (Sonoma)
- Swift 5.10+ (Xcode 16+) — for building from source
- [`@marswave/coli`](https://www.npmjs.com/package/@marswave/coli) — SenseVoice transcription CLI, bundled at build time

```bash
npm install -g @marswave/coli
```

## Running in development

```bash
cd native/VoiceInputShell
./Scripts/run-dev-app.sh
```

Builds the Swift package, stages `Murmur.app` under `.stage/`, kills any running instance, and opens the new build.

## Building a distributable bundle

```bash
cd native/VoiceInputShell
./Scripts/stage-dev-app.sh --release
```

The staged bundle is at `native/VoiceInputShell/.stage/Murmur.app`.

```bash
# Package as DMG
hdiutil create -volname "Murmur" \
  -srcfolder native/VoiceInputShell/.stage/Murmur.app \
  -ov -format UDZO \
  Murmur.dmg
```

> **Note:** The bundle is unsigned. For distribution outside your own machine, sign it:
> ```bash
> codesign --deep --force --sign "Developer ID Application: Your Name" \
>   native/VoiceInputShell/.stage/Murmur.app
> ```

## Project structure

```
voice-input-mac/
├── native/VoiceInputShell/          Swift menu bar app (the app)
│   ├── Package.swift
│   ├── Sources/VoiceInputShell/
│   │   ├── App/                     Entry point, NSStatusItem, NSPanel
│   │   ├── Engine/                  Audio recording, live ASR, coli, LLM polish
│   │   ├── Support/                 Path resolution, CGEvent paste
│   │   └── UI/                      SwiftUI panel + view model
│   └── Scripts/
│       ├── run-dev-app.sh           Build + stage + launch
│       └── stage-dev-app.sh         Stage .app bundle
└── docs/
    ├── product-spec.zh-CN.md
    └── technical-assessment.zh-CN.md
```

## Architecture

```
AVAudioEngine ──buffers──▶ LiveSpeechRecognizer  (parallel zh-CN / zh-TW / en-US)
     │                              │
     │                       liveTranscript (real-time SwiftUI preview)
     │
     └── WAV file ──────────▶ ColiTranscriber  (coli asr subprocess)
                                    │
                              transcriptText (SwiftUI)
                                    │
                            LLMPolisher  (optional, OpenAI-compatible API)
                                    │
                             polishedText (SwiftUI)
                                    │
                      TextInsertionService → CGEvent ⌘V → frontmost app
```

## Internal Docs

- [Product Spec](docs/product-spec.zh-CN.md)
- [Technical Assessment](docs/technical-assessment.zh-CN.md)

## License

MIT


