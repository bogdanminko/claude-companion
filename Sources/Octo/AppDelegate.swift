import AppKit

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var panel: CompanionPanel!
    private var statusItem: NSStatusItem!
    private let bubble = BubblePanel()

    func applicationDidFinishLaunching(_ notification: Notification) {
        panel = CompanionPanel()
        panel.companionView.onClick = { [unowned self] in self.handleClick() }
        panel.companionView.onDragStart = { [unowned self] in self.bubble.hide() }
        panel.companionView.contextMenuProvider = { [unowned self] in self.bubble.hide(); return self.makeMenu() }
        panel.orderFrontRegardless()

        bubble.onChat = { [unowned self] in self.quickEntry() }
        bubble.onVoice = { QuickEntry.toggleVoice() }
        QuickEntry.onListeningChanged = { [unowned self] on in
            self.panel.companionView.setListening(on)
            self.bubble.setListening(on)
            self.statusItem.menu = self.makeMenu() // обновить пункт «Голос / Остановить»
        }

        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        statusItem.button?.image = NSImage(systemSymbolName: "terminal", accessibilityDescription: "Octo")
        statusItem.menu = makeMenu()

        // Спросим доступ к Accessibility один раз при старте — он нужен, чтобы нажимать двойной Option.
        QuickEntry.ensureAccessibility(prompt: true)
    }

    private func makeMenu() -> NSMenu {
        let menu = NSMenu()
        func item(_ title: String, _ action: Selector, _ key: String = "") {
            let i = NSMenuItem(title: title, action: action, keyEquivalent: key)
            i.target = self
            menu.addItem(i)
        }
        item("Чат (quick entry)", #selector(quickEntry))
        item(QuickEntry.isListening ? "Остановить голос" : "Голос", #selector(voice))
        item("Открыть Claude", #selector(openClaude))
        menu.addItem(.separator())
        item("Показать / спрятать Octo", #selector(toggle))
        item("Уложить спать", #selector(sleep))
        item("Вернуть в угол", #selector(resetPosition))
        menu.addItem(.separator())
        item("Выйти", #selector(quit), "q")
        return menu
    }

    /// Клик по осьминогу: во время записи голоса — остановить её, иначе показать капсулу.
    private func handleClick() {
        if QuickEntry.isListening {
            QuickEntry.toggleVoice()
        } else {
            bubble.toggle(below: panel.frame, listening: false)
        }
    }

    @objc private func quickEntry() {
        panel.companionView.think()
        QuickEntry.open()
    }
    @objc private func voice() { QuickEntry.toggleVoice() }
    @objc private func openClaude() { QuickEntry.openMainApp() }
    @objc private func toggle() { panel.isVisible ? panel.orderOut(nil) : panel.orderFrontRegardless() }
    @objc private func sleep() { panel.companionView.goToSleep() }
    @objc private func resetPosition() { panel.resetPosition() }
    @objc private func quit() { NSApp.terminate(nil) } // exit 0 → launchd не перезапускает
}
