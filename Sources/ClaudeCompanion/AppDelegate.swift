import AppKit

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var panel: CompanionPanel!
    private var statusItem: NSStatusItem!
    private let bubble = BubblePanel()
    private let doubleOption = DoubleOptionDetector()

    func applicationDidFinishLaunching(_ notification: Notification) {
        panel = CompanionPanel()
        panel.companionView.onClick = { [unowned self] in self.handleClick() }
        panel.companionView.onDragStart = { [unowned self] in self.bubble.hide() }
        panel.companionView.contextMenuProvider = { [unowned self] in self.bubble.hide(); return self.makeMenu() }
        panel.orderFrontRegardless()

        panel.companionView.onHover = { [unowned self] inside in
            inside ? self.bubble.show(below: self.panel.frame) : self.bubble.scheduleHide()
        }
        bubble.onClaude = { [unowned self] in self.openClaude() }
        bubble.onCode = { [unowned self] in self.openClaudeCode() }
        ClaudeApp.onListeningChanged = { [unowned self] on in
            self.panel.companionView.setListening(on)
            self.statusItem.menu = self.makeMenu() // refresh the "Voice / Stop" item
        }

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        statusItem.button?.image = NSImage(systemSymbolName: "terminal", accessibilityDescription: "Pixel")
        statusItem.menu = makeMenu()

        // Accessibility is needed both to catch double Option and to press Caps Lock for voice.
        doubleOption.onDoubleTap = { [unowned self] in self.summon() }
        if ClaudeApp.ensureAccessibility(prompt: true) {
            doubleOption.start()
        } else {
            waitForAccessibility()
        }
    }

    /// Until access is granted, poll every 2 seconds and enable the hotkey once it is.
    private func waitForAccessibility() {
        Timer.scheduledTimer(withTimeInterval: 2, repeats: true) { [weak self] timer in
            MainActor.assumeIsolated {
                guard ClaudeApp.ensureAccessibility(prompt: false) else { return }
                timer.invalidate()
                self?.doubleOption.stop()
                self?.doubleOption.start()
            }
        }
    }

    /// Double Option: show Pixel (if hidden) and the capsule; again — hide the capsule.
    private func summon() {
        if !panel.isVisible { panel.orderFrontRegardless() }
        if ClaudeApp.isListening {
            ClaudeApp.toggleVoice()
        } else {
            bubble.toggle(below: panel.frame)
        }
    }

    private func makeMenu() -> NSMenu {
        let menu = NSMenu()
        func item(_ title: String, _ action: Selector, _ key: String = "") {
            let i = NSMenuItem(title: title, action: action, keyEquivalent: key)
            i.target = self
            menu.addItem(i)
        }
        item(ClaudeApp.isListening ? "Stop voice" : "Voice", #selector(voice))
        item("Open Claude", #selector(openClaude))
        item("Open Claude Code", #selector(openClaudeCode))
        menu.addItem(.separator())
        item("Show / hide Pixel", #selector(toggle))
        item("Put to sleep", #selector(sleep))
        item("Fiesta 🎸", #selector(fiesta))
        item("Back to corner", #selector(resetPosition))
        menu.addItem(.separator())
        item("Quit", #selector(quit), "q")
        return menu
    }

    /// Click on Pixel: while recording voice, stop it; otherwise show the capsule.
    private func handleClick() {
        if ClaudeApp.isListening {
            ClaudeApp.toggleVoice()
        } else {
            bubble.toggle(below: panel.frame)
        }
    }

    @objc private func voice() { ClaudeApp.toggleVoice() }
    @objc private func openClaude() { panel.companionView.think(); ClaudeApp.openMainApp() }
    @objc private func openClaudeCode() { panel.companionView.think(); ClaudeApp.openCode() }
    @objc private func toggle() { panel.isVisible ? panel.orderOut(nil) : panel.orderFrontRegardless() }
    @objc private func sleep() { panel.companionView.goToSleep() }
    @objc private func fiesta() { panel.companionView.fiesta() }
    @objc private func resetPosition() { panel.resetPosition() }
    @objc private func quit() { NSApp.terminate(nil) } // exit 0 → launchd won't restart
}
