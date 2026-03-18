import Foundation

@MainActor
final class ShellViewModel: ObservableObject {
    var onRequestDismiss: (() -> Void)?

    @Published var title = "Swift shell ready"
    @Published var detail = "The menu bar shell is live. Checking the local Rust runtime and bundled helpers."
    @Published var rustVersion = "—"
    @Published var runtimeBadge = "Checking"
    @Published var ffmpegLine = "ffmpeg unresolved"
    @Published var coliLine = "coli unresolved"
    @Published var recordingLine = "Idle"
    @Published var recordingPath = ""
    @Published var actionError = ""
    @Published var transcriptText = ""
    @Published var transcriptMeta = ""
    @Published var diagnosticsExpanded = false

    var isReady: Bool {
        runtimeBadge == "Ready"
    }

    var isRecordingActive: Bool {
        recordingLine == "Recording live"
    }

    var primaryActionTitle: String {
        isRecordingActive ? "Stop recording" : "Start recording"
    }

    var primaryActionSubtitle: String {
        if isRecordingActive {
            return "Capture is live. Stop when you're ready to transcribe."
        }

        return isReady ? "Capture a short clip, then transcribe and paste it." : "Fix runtime tools first, then start dictation."
    }

    var canStartRecording: Bool {
        isReady && !isRecordingActive
    }

    var canStopRecording: Bool {
        isRecordingActive
    }

    var canTranscribe: Bool {
        !recordingPath.isEmpty && !isRecordingActive
    }

    var canPasteTranscript: Bool {
        !transcriptText.isEmpty
    }

    var diagnosticsSummary: String {
        "\(runtimeBadge) · \(ffmpegLine) · \(coliLine)"
    }

    func refreshRuntime() {
        do {
            let bridge = RustCoreBridge.shared
            let summary = try bridge.runtimeSummary()
            let recording = try bridge.isRecording()
            rustVersion = bridge.version()
            runtimeBadge = summary.ffmpegExists && summary.coliExists ? "Ready" : "Needs setup"
            title = summary.ffmpegExists && summary.coliExists ? "Voice input is ready" : "Runtime needs attention"
            detail = summary.ffmpegExists && summary.coliExists
                ? "Local capture and transcription helpers are available. Record, transcribe, then paste into the frontmost app."
                : "The shell loaded the Rust core, but one or more helper tools still need attention before dictation is fully ready."
            ffmpegLine = statusLine(name: "ffmpeg", path: summary.ffmpegPath, available: summary.ffmpegExists)
            coliLine = statusLine(name: "coli", path: summary.coliPath, available: summary.coliExists)
            recordingLine = recording ? "Recording live" : "Ready to record"
            actionError = ""
        } catch {
            runtimeBadge = "Offline"
            title = "Rust core unavailable"
            detail = error.localizedDescription
            ffmpegLine = "ffmpeg unresolved"
            coliLine = "coli unresolved"
            recordingLine = "Unavailable"
            transcriptText = ""
            transcriptMeta = ""
        }
    }

    func startRecording() {
        guard canStartRecording else {
            actionError = isReady ? "Recording is already running." : "Runtime is not ready for recording yet."
            return
        }

        do {
            let path = try RustCoreBridge.shared.startRecording()
            recordingPath = path
            recordingLine = "Recording live"
            actionError = ""
            transcriptText = ""
            transcriptMeta = ""
            detail = "Recording through the shared Rust core. Stop when you're ready to transcribe."
        } catch {
            actionError = error.localizedDescription
            recordingLine = "Start failed"
        }
    }

    func stopRecording() {
        guard canStopRecording else {
            actionError = "There is no active recording to stop."
            return
        }

        do {
            try RustCoreBridge.shared.stopRecording()
            recordingLine = "Recorded"
            actionError = ""
            if !recordingPath.isEmpty {
                detail = "Recording finished. You can transcribe the latest clip now."
            }
        } catch {
            actionError = error.localizedDescription
            recordingLine = "Stop failed"
        }
    }

    func transcribeLatestRecording() {
        guard !recordingPath.isEmpty else {
            actionError = "No completed recording available to transcribe yet."
            return
        }

        do {
            let result = try RustCoreBridge.shared.transcribeAudio(at: recordingPath)
            transcriptText = result.text

            var metaParts = [String]()
            if let lang = result.lang, !lang.isEmpty {
                metaParts.append("lang: \(lang)")
            }
            if let duration = result.duration {
                metaParts.append(String(format: "audio: %.1fs", duration))
            }

            transcriptMeta = metaParts.joined(separator: "  |  ")
            actionError = ""
            detail = "Transcription completed. Copy it or paste it straight into the focused app."
        } catch {
            actionError = error.localizedDescription
        }
    }

    func copyTranscript() {
        guard !transcriptText.isEmpty else {
            actionError = "No transcript available to copy."
            return
        }

        TextInsertionService.copyToClipboard(transcriptText)
        actionError = ""
        detail = "Transcript copied to the clipboard."
    }

    func pasteTranscript() {
        guard !transcriptText.isEmpty else {
            actionError = "No transcript available to paste."
            return
        }

        do {
            try TextInsertionService.pasteToFrontmostApp(transcriptText)
            actionError = ""
            detail = "Transcript pasted into the frontmost app."
            onRequestDismiss?()
        } catch {
            actionError = error.localizedDescription
        }
    }

    func toggleDiagnostics() {
        diagnosticsExpanded.toggle()
    }

    private func statusLine(name: String, path: String?, available: Bool) -> String {
        let location = path.flatMap { URL(fileURLWithPath: $0).lastPathComponent.isEmpty ? nil : URL(fileURLWithPath: $0).lastPathComponent } ?? "not found"
        return available ? "\(name) ready · \(location)" : "\(name) missing · \(location)"
    }
}
