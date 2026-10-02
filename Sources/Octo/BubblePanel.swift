import AppKit

/// Тёмная капсула с двумя кнопками: чат (quick entry) и голос.
@MainActor
final class BubblePanel: NSPanel {
    var onChat: (() -> Void)?
    var onVoice: (() -> Void)?

    private static let size = NSSize(width: 112, height: 50)
    private let voiceButton: NSButton
    private var outsideClickMonitor: Any?
    private var autoHide: DispatchWorkItem?

    init() {
        voiceButton = Self.makeButton(symbol: "waveform", label: "Голос")
        super.init(
            contentRect: NSRect(origin: .zero, size: Self.size),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        level = .floating
        collectionBehavior = [.canJoinAllSpaces, .stationary, .fullScreenAuxiliary, .ignoresCycle]
        hidesOnDeactivate = false
        isReleasedWhenClosed = false

        let capsule = NSView(frame: NSRect(origin: .zero, size: Self.size))
        capsule.wantsLayer = true
        capsule.layer?.backgroundColor = NSColor(srgbRed: 0.16, green: 0.16, blue: 0.17, alpha: 0.96).cgColor
        capsule.layer?.cornerRadius = Self.size.height / 2
        capsule.layer?.borderWidth = 1
        capsule.layer?.borderColor = NSColor(white: 1, alpha: 0.12).cgColor

        let chatButton = Self.makeButton(symbol: "square.and.pencil", label: "Чат")
        chatButton.target = self
        chatButton.action = #selector(chatTapped)
        voiceButton.target = self
        voiceButton.action = #selector(voiceTapped)

        let half = Self.size.width / 2
        chatButton.frame = NSRect(x: 6, y: 5, width: half - 8, height: 40)
        voiceButton.frame = NSRect(x: half + 2, y: 5, width: half - 8, height: 40)

        let divider = NSView(frame: NSRect(x: half - 0.5, y: 13, width: 1, height: 24))
        divider.wantsLayer = true
        divider.layer?.backgroundColor = NSColor(white: 1, alpha: 0.18).cgColor

        capsule.addSubview(chatButton)
        capsule.addSubview(divider)
        capsule.addSubview(voiceButton)
        contentView = capsule
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    private static func makeButton(symbol: String, label: String) -> NSButton {
        let config = NSImage.SymbolConfiguration(pointSize: 19, weight: .medium)
        let image = NSImage(systemSymbolName: symbol, accessibilityDescription: label)?
            .withSymbolConfiguration(config)
        let b = NSButton(image: image ?? NSImage(), target: nil, action: nil)
        b.isBordered = false
        b.contentTintColor = .white
        b.toolTip = label
        b.setAccessibilityLabel(label)
        return b
    }

    // MARK: Показ / скрытие

    func toggle(below anchor: NSRect, listening: Bool) {
        isVisible ? hide() : show(below: anchor, listening: listening)
    }

    func show(below anchor: NSRect, listening: Bool) {
        setListening(listening)
        let w = Self.size.width, h = Self.size.height
        var origin = NSPoint(x: anchor.midX - w / 2, y: anchor.minY - h + 14)
        let screen = NSScreen.screens.first { $0.frame.intersects(anchor) } ?? NSScreen.main
        if let vf = screen?.visibleFrame {
            if origin.y < vf.minY { origin.y = anchor.maxY - 10 }        // нет места снизу — над головой
            origin.x = min(max(origin.x, vf.minX + 4), vf.maxX - w - 4)
        }
        setFrameOrigin(origin)
        alphaValue = 0
        orderFrontRegardless()
        NSAnimationContext.runAnimationGroup { $0.duration = 0.12; animator().alphaValue = 1 }

        outsideClickMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { _ in
            MainActor.assumeIsolated { self.hide() }
        }
        scheduleAutoHide()
    }

    func hide() {
        autoHide?.cancel()
        if let m = outsideClickMonitor { NSEvent.removeMonitor(m); outsideClickMonitor = nil }
        orderOut(nil)
    }

    func setListening(_ on: Bool) {
        voiceButton.contentTintColor = on
            ? NSColor(srgbRed: 1.0, green: 0.42, blue: 0.38, alpha: 1)
            : .white
        voiceButton.toolTip = on ? "Остановить голос" : "Голос"
    }

    private func scheduleAutoHide() {
        autoHide?.cancel()
        let work = DispatchWorkItem { [weak self] in self?.hide() }
        autoHide = work
        DispatchQueue.main.asyncAfter(deadline: .now() + 6, execute: work)
    }

    @objc private func chatTapped() { hide(); onChat?() }
    @objc private func voiceTapped() { hide(); onVoice?() }
}
