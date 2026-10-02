import AppKit

/// Пиксельный осьминог с терминалом на пузе.
@MainActor
final class CompanionView: NSView {
    static let px: CGFloat = 5                // размер одного «пикселя» спрайта
    static let size = NSSize(width: 16 * 5 + 30, height: 16 * 5 + 30)
    static let idleTimeout: TimeInterval = 300  // через 5 минут без внимания засыпает

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

    // MARK: Палитра

    private enum C {
        static let body = NSColor(srgbRed: 0.851, green: 0.467, blue: 0.341, alpha: 1)   // #D97757
        static let shade = NSColor(srgbRed: 0.722, green: 0.361, blue: 0.251, alpha: 1)  // #B85C40
        static let light = NSColor(srgbRed: 0.937, green: 0.635, blue: 0.533, alpha: 1)  // блик
        static let eye = NSColor(srgbRed: 0.169, green: 0.106, blue: 0.090, alpha: 1)
        static let shine = NSColor.white
        static let screen = NSColor(srgbRed: 0.118, green: 0.118, blue: 0.125, alpha: 1)
        static let glyph = NSColor(srgbRed: 0.486, green: 0.890, blue: 0.545, alpha: 1)
        static let glyphDim = NSColor(srgbRed: 0.486, green: 0.890, blue: 0.545, alpha: 0.3)
        static let zzz = NSColor(srgbRed: 0.65, green: 0.70, blue: 0.95, alpha: 1)
    }

    // MARK: Спрайт (16×15)

    private static let head: [String] = [
        "......OOOO......",
        "....OLLOOOOO....",
        "...OLLOOOOOOO...",
        "..OOLOOOOOOOOO..",
        "..OOOOOOOOOOOO..",
        "..OOOOOOOOOOOO..",
        "..OOOOOOOOOOOO..",
        "..OSSSSSSSSSSO..",
        "..OSSSSSSSSSSO..",
        "..OSSSSSSSSSSO..",
        "..OSSSSSSSSSSO..",
        "..DDDDDDDDDDDD..",
    ]
    private static let legsA: [String] = [
        "..OO.OO..OO.OO..",
        ".OO..O....O..OO.",
        ".O...OO..OO...O.",
    ]
    private static let legsB: [String] = [
        "..OO.OO..OO.OO..",
        "..O..OO..OO..O..",
        ".OO..O....O..OO.",
    ]

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
        NSGraphicsContext.current?.shouldAntialias = false
        NSGraphicsContext.current?.imageInterpolation = .none

        let p = Self.px
        let sleeping = mood == .sleeping
        let half = sleeping ? 15 : 6
        let bob: CGFloat = (tick / half) % 2 == 0 ? 0 : p
        let ox: CGFloat = 15
        let oy: CGFloat = 22 + bob

        func cell(_ col: Int, _ row: Int, _ color: NSColor, size s: CGFloat = p, x0: CGFloat = ox, y0: CGFloat = oy) {
            color.setFill()
            NSRect(x: x0 + CGFloat(col) * s, y: y0 + CGFloat(row) * s, width: s, height: s).fill()
        }

        // тело
        let legs = ((tick / (sleeping ? 15 : 5)) % 2 == 0) ? Self.legsA : Self.legsB
        for (r, line) in (Self.head + legs).enumerated() {
            for (c, ch) in line.enumerated() {
                switch ch {
                case "O": cell(c, r, C.body)
                case "D": cell(c, r, C.shade)
                case "L": cell(c, r, C.light)
                case "S": cell(c, r, C.screen)
                default: break
                }
            }
        }

        // глаза
        let closed = sleeping || tick < blinkUntil
        let eyeRow = mood == .hover ? 3 : 4
        for ex in [4, 10] {
            if closed {
                cell(ex, 5, C.eye); cell(ex + 1, 5, C.eye)
            } else {
                cell(ex, eyeRow, C.eye); cell(ex + 1, eyeRow, C.shine)
                cell(ex, eyeRow + 1, C.eye); cell(ex + 1, eyeRow + 1, C.eye)
            }
        }
        // румянец при наведении
        if mood == .hover {
            cell(3, 6, C.light); cell(12, 6, C.light)
        }

        // экран на пузе
        switch mood {
        case .idle, .hover:
            cell(4, 7, C.glyph); cell(5, 8, C.glyph); cell(4, 9, C.glyph)    // >
            if (tick / 5) % 2 == 0 {                                          // мигающий курсор
                cell(7, 9, C.glyph); cell(8, 9, C.glyph)
            }
        case .thinking:
            let active = (tick / 3) % 3
            for i in 0..<3 {
                cell(5 + i * 2, 9, i == active ? C.glyph : C.glyphDim)
            }
        case .listening:
            // эквалайзер: столбики высотой 1…4 «пикселя»
            for c in 4...11 {
                let h = max(1, 4 - abs(((tick + c * 3) % 8) - 4))
                for r in 0..<h { cell(c, 10 - r, C.glyph) }
            }
        case .sleeping:
            break // экран выключен
        }

        // zzz
        if sleeping {
            let zs: CGFloat = 3
            let pattern = ["xxx", ".x.", "xxx"]
            for k in 0..<2 {
                let phase = (tick / 2 + k * 10) % 20
                let x0 = ox + 13 * p + CGFloat(phase) * 0.6
                let y0 = oy - 4 - CGFloat(phase) * 0.9
                for (r, line) in pattern.enumerated() {
                    for (c, ch) in line.enumerated() where ch == "x" {
                        cell(c, r, C.zzz, size: zs, x0: x0, y0: y0)
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
