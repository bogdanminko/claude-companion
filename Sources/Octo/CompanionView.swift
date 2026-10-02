import AppKit

/// Пиксельный осьминог с терминалом на пузе: состояние, анимация и мышь.
@MainActor
final class CompanionView: NSView {
    static let px: CGFloat = 2.5                 // 1 пиксель спрайта = 2.5 pt = 5 физ. пикселей на Retina
    static let size = NSSize(width: 110, height: 112)
    static let idleTimeout: TimeInterval = 300   // через 5 минут без внимания засыпает

    enum Mood { case idle, hover, thinking, listening, sleeping }

    var onClick: (() -> Void)?
    var onDragStart: (() -> Void)?
    var contextMenuProvider: (() -> NSMenu)?

    private(set) var mood: Mood = .idle
    private var tick = 0
    private var timer: Timer?
    private var lastInteraction = Date()
    private var blinkUntil = 0
    private var nextBlink = 30

    private var dragStartMouse: NSPoint = .zero
    private var dragStartOrigin: NSPoint = .zero
    private var dragged = false

    private static let zzzColor = NSColor(srgbRed: 0.65, green: 0.70, blue: 0.95, alpha: 1)

    // MARK: Жизненный цикл

    override init(frame: NSRect) {
        super.init(frame: frame)
        let t = Timer(timeInterval: 0.1, repeats: true) { [weak self] _ in
            MainActor.assumeIsolated { self?.step() }
        }
        RunLoop.main.add(t, forMode: .common)
        timer = t
        addTrackingArea(NSTrackingArea(
            rect: .zero,
            options: [.mouseEnteredAndExited, .activeAlways, .inVisibleRect],
            owner: self
        ))
    }

    required init?(coder: NSCoder) { fatalError("not used") }

    override var isFlipped: Bool { true }
    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    private func step() {
        tick += 1
        if mood != .sleeping, mood != .thinking, mood != .listening,
           Date().timeIntervalSince(lastInteraction) > Self.idleTimeout {
            mood = .sleeping
        }
        if tick >= nextBlink {
            blinkUntil = tick + 2
            nextBlink = tick + Int.random(in: 25...70)
        }
        needsDisplay = true
    }

    func goToSleep() { mood = .sleeping }

    private func wake() {
        lastInteraction = Date()
        if mood == .sleeping { mood = .idle }
    }

    func setListening(_ on: Bool) {
        lastInteraction = Date()
        mood = on ? .listening : .idle
    }

    func think() {
        lastInteraction = Date()
        mood = .thinking
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) { [weak self] in
            guard let self, self.mood == .thinking else { return }
            self.mood = .idle
        }
    }

    // MARK: Отрисовка

    override func draw(_ dirtyRect: NSRect) {
        guard let ctx = NSGraphicsContext.current else { return }
        ctx.shouldAntialias = false

        let p = Self.px
        let sleeping = mood == .sleeping
        // плавное покачивание на 1 пиксель спрайта
        let bobPeriod = sleeping ? 40.0 : 16.0
        let bob: CGFloat = sin(Double(tick) / bobPeriod * 2 * .pi) > 0 ? p : 0
        let ox: CGFloat = 15
        let oy: CGFloat = 26 + bob

        let eyes: OctoSprite.Eyes = (sleeping || tick < blinkUntil) ? .closed : .open(lookUp: mood == .hover)
        let screen: OctoSprite.Screen
        switch mood {
        case .idle, .hover: screen = .prompt(cursor: (tick / 5) % 2 == 0)
        case .thinking:     screen = .thinking(active: (tick / 3) % 3)
        case .listening:    screen = .listening(tick: tick)
        case .sleeping:     screen = .off
        }
        let phase = Double(tick) * (sleeping ? 0.05 : 0.18)

        let grid = OctoSprite.render(tentaclePhase: phase, eyes: eyes, blush: mood == .hover, screen: screen)
        for (y, row) in grid.enumerated() {
            for (x, color) in row.enumerated() {
                guard let color else { continue }
                color.setFill()
                NSRect(x: ox + CGFloat(x) * p, y: oy + CGFloat(y) * p, width: p, height: p).fill()
            }
        }

        // zzz над головой
        if sleeping {
            let pattern = ["xxxx", "..x.", ".x..", "xxxx"]
            for k in 0..<2 {
                let phase = (tick / 2 + k * 10) % 20
                let x0 = ox + 25 * p + CGFloat(phase) * 0.6
                let y0 = oy - 2 - CGFloat(phase) * 0.9
                let s: CGFloat = k == 0 ? p : p * 0.8
                Self.zzzColor.withAlphaComponent(1 - CGFloat(phase) / 25).setFill()
                for (r, line) in pattern.enumerated() {
                    for (c, ch) in line.enumerated() where ch == "x" {
                        NSRect(x: x0 + CGFloat(c) * s, y: y0 + CGFloat(r) * s, width: s, height: s).fill()
                    }
                }
            }
        }
    }

    // MARK: Мышь

    override func mouseEntered(with event: NSEvent) {
        wake()
        if mood == .idle { mood = .hover }
    }

    override func mouseExited(with event: NSEvent) {
        if mood == .hover { mood = .idle }
    }

    override func mouseDown(with event: NSEvent) {
        dragStartMouse = NSEvent.mouseLocation
        dragStartOrigin = window?.frame.origin ?? .zero
        dragged = false
    }

    override func mouseDragged(with event: NSEvent) {
        let m = NSEvent.mouseLocation
        let dx = m.x - dragStartMouse.x
        let dy = m.y - dragStartMouse.y
        if !dragged, abs(dx) + abs(dy) > 3 {
            dragged = true
            onDragStart?()
        }
        if dragged {
            window?.setFrameOrigin(NSPoint(x: dragStartOrigin.x + dx, y: dragStartOrigin.y + dy))
        }
    }

    override func mouseUp(with event: NSEvent) {
        if dragged {
            (window as? CompanionPanel)?.savePosition()
            return
        }
        wake()
        onClick?()
    }

    override func rightMouseDown(with event: NSEvent) {
        wake()
        if let menu = contextMenuProvider?() {
            NSMenu.popUpContextMenu(menu, with: event, for: self)
        }
    }
}
