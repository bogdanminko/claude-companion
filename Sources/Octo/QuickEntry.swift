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

    // MARK: Голос — Caps Lock: нажать, говорить, нажать ещё раз

    private static let capsLockKeyCode: CGKeyCode = 57 // kVK_CapsLock
    private(set) static var isListening = false
    private static var lastSyntheticCaps = Date.distantPast
    private static var capsMonitor: Any?
    /// Вызывается при любой смене состояния голоса (в т.ч. если Caps Lock нажали руками).
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

    /// Следим за физическим Caps Lock, чтобы Octo не рассинхронизировался с Claude.
    private static func startCapsMonitorIfNeeded() {
        guard capsMonitor == nil else { return }
        capsMonitor = NSEvent.addGlobalMonitorForEvents(matching: .flagsChanged) { event in
            guard event.keyCode == capsLockKeyCode else { return }
            MainActor.assumeIsolated {
                // наше собственное нажатие пропускаем
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

    // MARK: Приложение

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
