import AppKit
import SwiftUI

@MainActor
final class StatusItemController: NSObject {
    private let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    private let panelController = PanelController()

    override init() {
        super.init()
        configureButton()
        configureMenu()
    }

    private func configureButton() {
        guard let button = statusItem.button else {
            return
        }

        button.title = "Voice Input"
        button.image = NSImage(systemSymbolName: "waveform.circle.fill", accessibilityDescription: "Voice Input")
        button.imagePosition = .imageLeading
        button.target = self
        button.action = #selector(handleStatusItemClick(_:))
        button.sendAction(on: [.leftMouseUp, .rightMouseUp])
    }

    private func configureMenu() {
        let menu = NSMenu()
        menu.addItem(withTitle: "Open Panel", action: #selector(openPanel), keyEquivalent: "")
        menu.addItem(withTitle: "Check Rust Core", action: #selector(refreshSmokeStatus), keyEquivalent: "")
        menu.addItem(.separator())
        menu.addItem(withTitle: "Quit Voice Input", action: #selector(quitApp), keyEquivalent: "q")
        menu.items.forEach { $0.target = self }
        statusItem.menu = nil
        statusItem.menu = menu
    }

    @objc
    private func handleStatusItemClick(_ sender: AnyObject?) {
        guard let event = NSApp.currentEvent else {
            panelController.togglePanel(relativeTo: statusItem.button)
            return
        }

        switch event.type {
        case .rightMouseUp:
            statusItem.button?.performClick(nil)
        default:
            statusItem.menu = nil
            panelController.togglePanel(relativeTo: statusItem.button)
            configureMenu()
        }
    }

    @objc
    private func openPanel() {
        panelController.showPanel(relativeTo: statusItem.button)
    }

    @objc
    private func refreshSmokeStatus() {
        panelController.refreshRustStatus()
        showPanel(relativeTo: statusItem.button)
    }

    @objc
    private func quitApp() {
        NSApp.terminate(nil)
    }

    private func showPanel(relativeTo button: NSStatusBarButton?) {
        panelController.showPanel(relativeTo: button)
    }
}
