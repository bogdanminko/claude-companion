import AppKit

/// Pixel Clawd with a terminal for a mouth: state, animation and mouse.
@MainActor
final class CompanionView: NSView {
    static let px: CGFloat = 5                   // 18 sprite px = 90 pt
    static let size = NSSize(width: 130, height: 112)
    static let idleTimeout: TimeInterval = 300   // falls asleep after 5 minutes without attention

    enum Mood { case idle, hover, thinking, listening, sleeping }

    var onClick: (() -> Void)?
    var onHover: ((Bool) -> Void)?
    var onDragStart: (() -> Void)?
    var contextMenuProvider: (() -> NSMenu)?

    private(set) var mood: Mood = .idle
    private var tick = 0
    private var timer: Timer?
    private var lastInteraction = Date()
    private var blinkUntil = 0
    private var nextBlink = 30

    /// Random idle tricks, so Pixel feels alive.
    enum Trick: CaseIterable { case walk, jump, shake, wave, look, glitch, fiesta }
    private var trick: Trick?
    private var trickStart = 0
    private var trickLength = 0
    private var walkDir: CGFloat = 1
    private var nextTrick = Int.random(in: 80...250)

    private var dragStartMouse: NSPoint = .zero
    private var dragStartOrigin: NSPoint = .zero
    private var dragged = false

    private static let glitchColors = [NSColor(srgbRed: 0.2, green: 0.95, blue: 1, alpha: 1), NSColor(srgbRed: 1, green: 0.25, blue: 0.75, alpha: 1)]
    private static let noteColor = NSColor(srgbRed: 0.98, green: 0.85, blue: 0.45, alpha: 1)
    private static let zzzColor = NSColor(srgbRed: 0.65, green: 0.70, blue: 0.95, alpha: 1)

    // MARK: Lifecycle

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

    func step() {
        tick += 1
        if mood != .sleeping, mood != .thinking, mood != .listening,
           Date().timeIntervalSince(lastInteraction) > Self.idleTimeout {
            mood = .sleeping
        }
        updateTrick()
        if tick >= nextBlink {
            blinkUntil = tick + 2
            nextBlink = tick + Int.random(in: 25...70)
        }
        needsDisplay = true
    }

    private func updateTrick() {
        guard mood == .idle || (mood == .hover && trick == .fiesta) else { stopTrick(); return }
        if trick == nil, tick >= nextTrick {
            startTrick(Trick.allCases.randomElement()!)
        }
        guard let trick else { return }
        if tick - trickStart >= trickLength { stopTrick(); return }
        if trick == .walk, let w = window, let vf = (w.screen ?? NSScreen.main)?.visibleFrame {
            var o = w.frame.origin
            o.x += walkDir * 3
            if o.x < vf.minX || o.x + w.frame.width > vf.maxX { walkDir *= -1; o.x += walkDir * 6 }  // bump into the edge, turn around
            w.setFrameOrigin(o)
        }
    }

    func startTrick(_ t: Trick) {
        trick = t
        trickStart = tick
        trickLength = switch t {
        case .walk: Int.random(in: 15...40)
        case .glitch: 15
        case .fiesta: 60
        default: 12
        }
        walkDir = Bool.random() ? 1 : -1
    }

    /// Sombrero on, a few seconds of guitar.
    func fiesta() {
        wake()
        mood = .idle
        startTrick(.fiesta)
    }

    private func stopTrick() {
        guard let t = trick else { return }
        trick = nil
        nextTrick = tick + Int.random(in: 80...250)
        if t == .walk { (window as? CompanionPanel)?.savePosition() }
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

    // MARK: Drawing

    override func draw(_ dirtyRect: NSRect) {
        guard let ctx = NSGraphicsContext.current else { return }
        ctx.shouldAntialias = false

        let p = Self.px
        let sleeping = mood == .sleeping
        // bob by one sprite pixel
        let bobPeriod = sleeping ? 40.0 : 16.0
        let bob: CGFloat = sin(Double(tick) / bobPeriod * 2 * .pi) > 0 ? p : 0
        let t = tick - trickStart
        var ox: CGFloat = 10
        var oy: CGFloat = 26 + bob
        switch trick {
        case .jump:  oy -= CGFloat(sin(Double(t) / Double(trickLength) * .pi)) * 5 * p
        case .shake: ox += t % 2 == 0 ? p : -p
        default: break
        }

        let eyes: PixelSprite.Eyes = (sleeping || tick < blinkUntil) ? .closed : .open(lookUp: mood == .hover)
        let screen: PixelSprite.Screen
        switch mood {
        case .idle where trick == .wave: screen = .listening(tick: tick)
        case .idle where trick == .look: screen = .thinking(active: (t / 4) % 3)
        case .idle, .hover: screen = .prompt(cursor: (tick / 5) % 2 == 0)
        case .thinking:     screen = .thinking(active: (tick / 3) % 3)
        case .listening:    screen = .listening(tick: tick)
        case .sleeping:     screen = .off
        }
        let phase = Double(tick) * (sleeping ? 0.05 : trick == .walk ? 0.9 : 0.18)

        let grid = PixelSprite.render(legPhase: phase, eyes: eyes, blush: mood == .hover, screen: screen,
                                      fiesta: trick == .fiesta ? t / 2 : nil)
        for (y, row) in grid.enumerated() {
            // glitch: rows tear sideways, some get a cyan / magenta fringe like broken video
            let torn = trick == .glitch && Int.random(in: 0..<2) == 0
            let glitch: CGFloat = torn ? CGFloat(Int.random(in: -3...3)) * p : 0
            let fringe: NSColor? = torn && Int.random(in: 0..<3) == 0 ? Self.glitchColors.randomElement() : nil
            for (x, color) in row.enumerated() {
                guard let color else { continue }
                (fringe ?? color).setFill()
                NSRect(x: ox + glitch + CGFloat(x) * p, y: oy + CGFloat(y) * p, width: p, height: p).fill()
            }
        }

        // notes floating up from the guitar
        if trick == .fiesta {
            let note = ["..x", "..x", "xxx", "xx."]
            for k in 0..<2 {
                let ph = (t + k * 8) % 16
                Self.noteColor.withAlphaComponent(1 - CGFloat(ph) / 16).setFill()
                let x0 = ox + 18 * p + 2 + CGFloat(k) * 2 * p, y0 = oy + 8 * p - CGFloat(ph) * 2.5
                let s = p * 0.6
                for (r, line) in note.enumerated() {
                    for (c, ch) in line.enumerated() where ch == "x" {
                        NSRect(x: x0 + CGFloat(c) * s, y: y0 + CGFloat(r) * s, width: s, height: s).fill()
                    }
                }
            }
        }

        // zzz above the head
        if sleeping {
            let pattern = ["xxxx", "..x.", ".x..", "xxxx"]
            for k in 0..<2 {
                let phase = (tick / 2 + k * 10) % 20
                let x0 = ox + 15 * p + CGFloat(phase) * 0.6
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

    // MARK: Mouse

    override func mouseEntered(with event: NSEvent) {
        wake()
        if mood == .idle { mood = .hover }
        onHover?(true)
    }

    override func mouseExited(with event: NSEvent) {
        if mood == .hover { mood = .idle }
        onHover?(false)
    }

    override func mouseDown(with event: NSEvent) {
        stopTrick()
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
