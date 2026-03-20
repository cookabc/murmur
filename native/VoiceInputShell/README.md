# Murmur — native macOS shell

Swift Package that builds the `Murmur.app` menu bar binary.

## Source layout

```
Sources/VoiceInputShell/
├── App/
│   ├── VoiceInputShellApp.swift    — @main entry, NSApplicationDelegate
│   ├── StatusItemController.swift  — NSStatusItem, recording icon animation
│   └── PanelController.swift       — floating NSPanel lifecycle
├── Engine/
│   ├── AudioSession.swift          — AVAudioEngine PCM recording → /tmp/voice_<ts>.wav
│   ├── LiveSpeechRecognizer.swift  — parallel SFSpeechRecognizer (zh-CN, zh-TW, en-US)
│   ├── ColiTranscriber.swift       — actor subprocess: coli asr → TranscriptionResult
│   ├── LLMPolisher.swift           — actor: OpenAI-compatible grammar/punctuation polish
│   └── VoiceCoreService.swift      — @MainActor façade wiring the above together
├── Support/
│   ├── AppPaths.swift              — coli binary resolution (bundle → env → Homebrew)
│   └── TextInsertionService.swift  — clipboard copy + CGEvent ⌘V paste
└── UI/
    ├── ShellViewModel.swift        — @MainActor ObservableObject state machine
    └── ShellPanelView.swift        — SwiftUI panel (408 × 500, dark/light adaptive)
```

## Development workflow

```bash
# Rebuild, stage, and launch (preferred)
./Scripts/run-dev-app.sh

# Stage only
./Scripts/stage-dev-app.sh [--release]

# Manual build
swift build
```

## Bundle layout

```
Murmur.app/
└── Contents/
    ├── MacOS/
    │   └── Murmur              (Swift executable)
    ├── Helpers/
    │   ├── coli                (shell wrapper → node coli_pkg/distribution/cli.js)
    │   ├── coli_pkg/           (full @marswave/coli node package with node_modules)
    │   └── node                (node binary, resolved from same dir as coli)
    └── Info.plist
```

## Environment overrides

| Variable | Purpose |
|---|---|
| `VOICE_INPUT_HELPERS_DIR` | Override the helpers directory at runtime |
| `VOICE_INPUT_COLI_PATH`   | Override the coli binary path used at staging time |
