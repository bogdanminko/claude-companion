import AppKit
import ApplicationServices

/// Claude desktop app: open it, open a Claude Code session, toggle voice via Caps Lock.
@MainActor
enum ClaudeApp {
    static let claudeBundleID = "com.anthropic.claudefordesktop"

    @discardableResult
    static func ensureAccessibility(prompt: Bool) -> Bool {
        let options = ["AXTrustedCheckOptionPrompt": prompt] as CFDictionary
        return AXIsProcessTrustedWithOptions(options)
    }

    static var isClaudeRunning: Bool {
        !NSRunningApplication.runningApplications(withBundleIdentifier: claudeBundleID).isEmpty
    }

    // MARK: Voice — Caps Lock: press, talk, press again

    private static let capsLockKeyCode: CGKeyCode = 57 // kVK_CapsLock
    private(set) static var isListening = false
    private static var lastSyntheticCaps = Date.distantPast
    private static var capsMonitor: Any?
    /// Called on any voice state change (including Caps Lock pressed by hand).
    static var onListeningChanged: ((Bool) -> Void)?

    static func toggleVoice() {
        guard isClaudeRunning else { openMainApp(); return }
        guard ensureAccessibility(prompt: true) else { return }
        startCapsMonitorIfNeeded()
        isListening.toggle()
        lastSyntheticCaps = Date()
        tapCapsLock(on: isListening)
        onListeningChanged?(isListening)
    }

    /// Watch the physical Caps Lock so Pixel stays in sync with Claude.
    private static func startCapsMonitorIfNeeded() {
        guard capsMonitor == nil else { return }
        capsMonitor = NSEvent.addGlobalMonitorForEvents(matching: .flagsChanged) { event in
            guard event.keyCode == capsLockKeyCode else { return }
            MainActor.assumeIsolated {
                // skip our own press
                guard Date().timeIntervalSince(lastSyntheticCaps) > 0.3 else { return }
                isListening.toggle()
                onListeningChanged?(isListening)
            }
        }
    }

    private static func tapCapsLock(on: Bool) {
        let source = CGEventSource(stateID: .hidSystemState)
        for keyDown in [true, false] {
            if let e = CGEvent(keyboardEventSource: source, virtualKey: capsLockKeyCode, keyDown: keyDown) {
                e.type = .flagsChanged
                e.flags = on ? .maskAlphaShift : []
                e.post(tap: .cghidEventTap)
            }
        }
    }

    // MARK: App

    static func openMainApp() {
        guard let url = NSWorkspace.shared.urlForApplication(withBundleIdentifier: claudeBundleID) else {
            NSSound.beep()
            return
        }
        NSWorkspace.shared.openApplication(at: url, configuration: NSWorkspace.OpenConfiguration())
    }

    static func openCode() { NSWorkspace.shared.open(URL(string: "claude://code/new")!) } // new session in the Code tab
}
