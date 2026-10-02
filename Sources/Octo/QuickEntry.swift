import AppKit
import ApplicationServices

/// Вызов quick entry Claude: имитируем двойное нажатие Option (стандартный хоткей на Mac).
@MainActor
enum QuickEntry {
    static let claudeBundleID = "com.anthropic.claudefordesktop"
    private static let optionKeyCode: CGKeyCode = 58 // kVK_Option

    @discardableResult
    static func ensureAccessibility(prompt: Bool) -> Bool {
        let options = ["AXTrustedCheckOptionPrompt": prompt] as CFDictionary
        return AXIsProcessTrustedWithOptions(options)
    }

    static var isClaudeRunning: Bool {
        !NSRunningApplication.runningApplications(withBundleIdentifier: claudeBundleID).isEmpty
    }

    static func open() {
        guard isClaudeRunning else {
            // Quick entry работает только при запущенном приложении — сначала поднимем его.
            openMainApp()
            return
        }
        guard ensureAccessibility(prompt: true) else { return }
        tapOption()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.08) { tapOption() }
    }

    static func openMainApp() {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: claudeBundleID) else {
            NSSound.beep()
            return
        }
        NSWorkspace.shared.openApplication(at: url, configuration: NSWorkspace.OpenConfiguration())
    }

    private static func tapOption() {
        let source = CGEventSource(stateID: .hidSystemState)
        if let down = CGEvent(keyboardEventSource: source, virtualKey: optionKeyCode, keyDown: true) {
            down.type = .flagsChanged
            down.flags = .maskAlternate
            down.post(tap: .cghidEventTap)
        }
        if let up = CGEvent(keyboardEventSource: source, virtualKey: optionKeyCode, keyDown: false) {
            up.type = .flagsChanged
            up.flags = []
            up.post(tap: .cghidEventTap)
        }
    }
}
