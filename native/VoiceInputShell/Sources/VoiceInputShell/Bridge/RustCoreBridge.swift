import Darwin
import Foundation

typealias VoiceCoreVersionFn = @convention(c) () -> UnsafeMutablePointer<CChar>?
typealias VoiceCoreConfigureToolsFn = @convention(c) (UnsafePointer<CChar>?, UnsafePointer<CChar>?) -> Bool
typealias VoiceCoreSmokeStatusFn = @convention(c) () -> UnsafeMutablePointer<CChar>?
typealias VoiceCoreStringFreeFn = @convention(c) (UnsafeMutablePointer<CChar>?) -> Void

struct RustSmokeStatus: Decodable {
    let name: String
    let version: String
    let ffmpegPath: String?
    let coliPath: String?
    let ffmpegExists: Bool
    let coliExists: Bool
}

enum RustCoreBridgeError: Error {
    case libraryNotFound(String)
    case symbolMissing(String)
    case callFailed(String)
}

final class RustCoreBridge {
    static let shared = RustCoreBridge()

    private let handle: UnsafeMutableRawPointer
    private let versionFn: VoiceCoreVersionFn
    private let configureToolsFn: VoiceCoreConfigureToolsFn
    private let smokeStatusFn: VoiceCoreSmokeStatusFn
    private let stringFreeFn: VoiceCoreStringFreeFn

    private init() {
        let libraryPath = AppPaths.rustCoreLibraryPath
        guard let handle = dlopen(libraryPath, RTLD_NOW | RTLD_LOCAL) else {
            fatalError("Failed to load Rust core: \(libraryPath)")
        }

        self.handle = handle
        versionFn = RustCoreBridge.loadSymbol(handle: handle, name: "voice_input_core_version")
        configureToolsFn = RustCoreBridge.loadSymbol(handle: handle, name: "voice_input_core_configure_tools")
        smokeStatusFn = RustCoreBridge.loadSymbol(handle: handle, name: "voice_input_core_smoke_status_json")
        stringFreeFn = RustCoreBridge.loadSymbol(handle: handle, name: "voice_input_core_string_free")
    }

    deinit {
        dlclose(handle)
    }

    func runtimeSummary() throws -> RustSmokeStatus {
        let configured = AppPaths.ffmpegHelperPath.withCString { ffmpegPtr in
            AppPaths.coliHelperPath.withCString { coliPtr in
                configureToolsFn(ffmpegPtr, coliPtr)
            }
        }

        guard configured else {
            throw RustCoreBridgeError.callFailed("Failed to configure helper binary paths")
        }

        guard let raw = smokeStatusFn() else {
            throw RustCoreBridgeError.callFailed("Rust core returned no smoke status")
        }

        defer { stringFreeFn(raw) }

        let data = Data(bytes: raw, count: strlen(raw))
        return try JSONDecoder().decode(RustSmokeStatus.self, from: data)
    }

    func version() -> String {
        guard let raw = versionFn() else {
            return "unknown"
        }

        defer { stringFreeFn(raw) }
        return String(cString: raw)
    }

    private static func loadSymbol<T>(handle: UnsafeMutableRawPointer, name: String) -> T {
        guard let symbol = dlsym(handle, name) else {
            fatalError("Missing Rust core symbol: \(name)")
        }

        return unsafeBitCast(symbol, to: T.self)
    }
}
