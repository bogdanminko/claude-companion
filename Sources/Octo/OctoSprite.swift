import AppKit

/// Осьминог 32×32 в стиле первой версии: плоская заливка без контура, блик на макушке,
/// полоса тени под экраном, короткие ножки. Двойное разрешение даёт плавные скругления.
enum OctoSprite {
    static let n = 32

    enum Eyes { case open(lookUp: Bool), closed }
    enum Screen { case prompt(cursor: Bool), thinking(active: Int), listening(tick: Int), off }

    enum P {
        static let body     = NSColor(srgbRed: 0.851, green: 0.467, blue: 0.341, alpha: 1) // #D97757
        static let shade    = NSColor(srgbRed: 0.722, green: 0.361, blue: 0.251, alpha: 1) // #B85C40
        static let light    = NSColor(srgbRed: 0.937, green: 0.635, blue: 0.533, alpha: 1) // блик
        static let eye      = NSColor(srgbRed: 0.169, green: 0.106, blue: 0.090, alpha: 1)
        static let shine    = NSColor.white
        static let screen   = NSColor(srgbRed: 0.118, green: 0.118, blue: 0.125, alpha: 1)
        static let glyph    = NSColor(srgbRed: 0.486, green: 0.890, blue: 0.545, alpha: 1)
        static let glyphDim = NSColor(srgbRed: 0.486, green: 0.890, blue: 0.545, alpha: 0.3)
    }

    // MARK: Силуэт

    private static func inHead(_ x: Double, _ y: Double) -> Bool {
        if y < 8.5 {                                   // купол
            let dx = (x - 16) / 12, dy = (y - 8.5) / 8.5
            return dx * dx + dy * dy <= 1
        }
        return x >= 4 && x <= 28 && y <= 24            // прямые бока до низа
    }

    /// Четыре ножки, как в первой версии: крайние расходятся наружу, все слегка шевелятся.
    private static let legX: [Double] = [6, 12, 20, 26]
    private static let legDrift: [Double] = [-3, -1, 1, 3]

    private static func inLeg(_ x: Double, _ y: Double, phase: Double) -> Bool {
        guard y >= 24, y <= 30 else { return false }
        let t = (y - 24) / 6
        for i in 0..<4 {
            let width = 4.4 - t * 0.9
            let center = legX[i] + legDrift[i] * t + sin(phase + Double(i) * 1.7) * 1.2 * t
            let tip = y > 29 ? 1.0 : 0                  // скруглённый кончик
            if abs(x - center) <= width / 2 - tip { return true }
        }
        return false
    }

    // MARK: Рендер

    static func render(tentaclePhase phase: Double, eyes: Eyes, blush: Bool, screen: Screen) -> [[NSColor?]] {
        var g = [[NSColor?]](repeating: [NSColor?](repeating: nil, count: n), count: n)
        func put(_ x: Int, _ y: Int, _ c: NSColor) {
            if x >= 0, y >= 0, x < n, y < n { g[y][x] = c }
        }

        for y in 0..<n {
            for x in 0..<n {
                let cx = Double(x) + 0.5, cy = Double(y) + 0.5
                if inHead(cx, cy) || inLeg(cx, cy, phase: phase) { g[y][x] = P.body }
            }
        }

        // блик на макушке — диагональный мазок, как в первой версии
        for (x, y) in [(11, 2), (12, 2), (13, 2), (9, 3), (10, 3), (11, 3), (12, 3),
                       (8, 4), (9, 4), (10, 4), (7, 5), (8, 5), (9, 5), (7, 6), (8, 6), (7, 7)] {
            put(x, y, P.light)
        }

        // полоса тени под экраном
        for y in 22...23 { for x in 4...27 { put(x, y, P.shade) } }

        // экран со скруглёнными углами
        for y in 13...21 {
            for x in 6...25 {
                let corner = (x == 6 || x == 25) && (y == 13 || y == 21)
                if !corner { put(x, y, P.screen) }
            }
        }

        // глаза: скруглённые квадраты 4×5 с бликом справа сверху
        for ex in [8, 20] {
            switch eyes {
            case .open(let lookUp):
                let top = lookUp ? 5 : 7
                for y in top..<(top + 5) {
                    for x in ex..<(ex + 4) {
                        let corner = (x == ex || x == ex + 3) && (y == top || y == top + 4)
                        if !corner { put(x, y, P.eye) }
                    }
                }
                put(ex + 2, top + 1, P.shine)
                put(ex + 2, top + 2, P.shine)
            case .closed:
                for x in ex..<(ex + 4) { put(x, 10, P.eye) }
                put(ex, 9, P.eye); put(ex + 3, 9, P.eye)        // мягкая дуга
            }
        }

        if blush {
            for (x, y) in [(5, 11), (6, 11), (25, 11), (26, 11)] { put(x, y, P.light) }
        }

        // содержимое экрана
        switch screen {
        case .prompt(let cursor):
            let chevron = ["xx...", ".xx..", "..xx.", ".xx..", "xx..."]
            for (r, line) in chevron.enumerated() {
                for (c, ch) in line.enumerated() where ch == "x" { put(9 + c, 15 + r, P.glyph) }
            }
            if cursor { for x in 15...19 { put(x, 18, P.glyph); put(x, 19, P.glyph) } }
        case .thinking(let active):
            for i in 0..<3 {
                let c = i == active ? P.glyph : P.glyphDim
                for dx in 0...1 { for dy in 0...1 { put(11 + i * 4 + dx, 16 + dy, c) } }
            }
        case .listening(let tick):
            for x in 8...23 {
                let wave = abs(sin(Double(tick) * 0.6 + Double(x) * 0.8)) * abs(cos(Double(tick) * 0.23 + Double(x) * 0.35))
                let h = 1 + Int(wave * 5.5)
                for k in 0..<h { put(x, 19 - k, P.glyph) }
            }
        case .off:
            break
        }
        return g
    }
}
