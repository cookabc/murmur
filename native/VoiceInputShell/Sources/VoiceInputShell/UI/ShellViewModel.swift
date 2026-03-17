import Foundation

@MainActor
final class ShellViewModel: ObservableObject {
    @Published var title = "Swift shell ready"
    @Published var detail = "The native menu bar shell is live. Load the Rust core to verify bundling paths."
    @Published var rustVersion = "unknown"
    @Published var ffmpegLine = "ffmpeg: unresolved"
    @Published var coliLine = "coli: unresolved"

    func refreshRuntime() {
        do {
            let bridge = RustCoreBridge.shared
            let summary = try bridge.runtimeSummary()
            rustVersion = bridge.version()
            title = "Rust core connected"
            detail = "The Swift shell loaded the Rust core and passed helper binary paths for smoke verification."
            ffmpegLine = "ffmpeg: \(summary.ffmpegPath ?? "missing") [\(summary.ffmpegExists ? "found" : "not found")]"
            coliLine = "coli: \(summary.coliPath ?? "missing") [\(summary.coliExists ? "found" : "not found")]"
        } catch {
            title = "Rust core unavailable"
            detail = error.localizedDescription
            ffmpegLine = "ffmpeg: unresolved"
            coliLine = "coli: unresolved"
        }
    }
}
