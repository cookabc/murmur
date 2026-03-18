# VoiceInputShell

This is the first native macOS shell scaffold for Voice Input.

Current scope:

- Creates a menu bar status item using AppKit.
- Differentiates left click for toggling the panel and right click for showing the menu.
- Uses a floating native panel instead of the old Tauri window.
- Loads the Rust core dynamically and performs a smoke check for bundled `ffmpeg` and `coli` paths.

Current development workflow:

```bash
cd voice-core && cargo build
cd ../native/VoiceInputShell && swift build
swift run
```

Development app bundle staging:

```bash
cd native/VoiceInputShell
./Scripts/stage-dev-app.sh
open .stage/VoiceInputShell.app
```

Optional overrides:

- `VOICE_INPUT_FFMPEG_PATH=/absolute/path/to/ffmpeg`
- `VOICE_INPUT_COLI_PATH=/absolute/path/to/coli`

The current shell expects the final bundled app layout to provide:

- `Contents/Frameworks/libvoice_input_core.dylib`
- `Contents/Helpers/ffmpeg`
- `Contents/Helpers/coli`

During local development, it falls back to the Rust core debug dylib and `/usr/local/bin/<tool>` helper locations.

The staging script above assembles that same layout under `.stage/VoiceInputShell.app` so the native shell can be exercised against bundled helpers instead of development fallbacks.