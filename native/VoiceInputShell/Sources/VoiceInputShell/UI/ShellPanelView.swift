import SwiftUI

struct ShellPanelView: View {
    @ObservedObject var viewModel: ShellViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Voice Input")
                        .font(.system(size: 24, weight: .semibold, design: .rounded))
                    Text("Native menu bar shell")
                        .font(.system(size: 12, weight: .medium, design: .rounded))
                        .foregroundStyle(.secondary)
                }
                Spacer()
                Text("Rust \(viewModel.rustVersion)")
                    .font(.system(size: 11, weight: .semibold, design: .rounded))
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .background(.thinMaterial, in: Capsule())
            }

            VStack(alignment: .leading, spacing: 8) {
                Text(viewModel.title)
                    .font(.system(size: 30, weight: .bold, design: .rounded))
                    .lineLimit(2)
                Text(viewModel.detail)
                    .font(.system(size: 14, weight: .medium, design: .rounded))
                    .foregroundStyle(.secondary)
            }

            VStack(alignment: .leading, spacing: 10) {
                Label(viewModel.ffmpegLine, systemImage: "waveform")
                Label(viewModel.coliLine, systemImage: "text.bubble")
            }
            .font(.system(size: 13, weight: .medium, design: .rounded))

            Spacer()

            HStack(spacing: 10) {
                Button("Refresh Rust Core") {
                    viewModel.refreshRuntime()
                }
                .buttonStyle(.borderedProminent)

                Button("Open Migration Notes") {
                    NSWorkspace.shared.open(URL(fileURLWithPath: FileManager.default.currentDirectoryPath))
                }
                .buttonStyle(.bordered)
            }
        }
        .padding(20)
        .frame(width: 408, height: 520)
        .background(
            LinearGradient(
                colors: [Color(red: 0.06, green: 0.11, blue: 0.14), Color(red: 0.11, green: 0.19, blue: 0.23)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .onAppear {
            viewModel.refreshRuntime()
        }
    }
}
