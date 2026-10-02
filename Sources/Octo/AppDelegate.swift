import AppKit

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    private var panel: CompanionPanel!
    private var statusItem: NSStatusItem!

    func applicationDidFinishLaunching(_ notification: Notification) {
        panel = CompanionPanel()
        panel.companionView.onClick = { QuickEntry.open() }
        panel.companionView.contextMenuProvider = { [unowned self] in self.makeMenu() }
        panel.orderFrontRegardless()

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
        item("Quick entry", #selector(quickEntry))
        item("Открыть Claude", #selector(openClaude))
        menu.addItem(.separator())
        item("Показать / спрятать Octo", #selector(toggle))
        item("Уложить спать", #selector(sleep))
        item("Вернуть в угол", #selector(resetPosition))
        menu.addItem(.separator())
        item("Выйти", #selector(quit), "q")
        return menu
    }

    @objc private func quickEntry() { QuickEntry.open() }
    @objc private func openClaude() { QuickEntry.openMainApp() }
    @objc private func toggle() { panel.isVisible ? panel.orderOut(nil) : panel.orderFrontRegardless() }
    @objc private func sleep() { panel.companionView.goToSleep() }
    @objc private func resetPosition() { panel.resetPosition() }
    @objc private func quit() { NSApp.terminate(nil) } // exit 0 → launchd не перезапускает
}
