import AppKit

/// Процедурный спрайт осьминога 32×32: форма из эллипса, скруглённого прямоугольника и пяти щупалец,
/// поверх — контур, трёхтоновая светотень, глаза с бликами и экран-терминал на пузе.
enum OctoSprite {
    static let n = 32

    enum Eyes { case open(lookUp: Bool), closed }
    enum Screen { case prompt(cursor: Bool), thinking(active: Int), listening(tick: Int), off }

    enum P {
        static let outline   = NSColor(srgbRed: 0.290, green: 0.133, blue: 0.086, alpha: 1) // #4A2216
        static let shadow    = NSColor(srgbRed: 0.710, green: 0.345, blue: 0.231, alpha: 1) // #B5583B
        static let base      = NSColor(srgbRed: 0.851, green: 0.467, blue: 0.341, alpha: 1) // #D97757
        static let highlight = NSColor(srgbRed: 0.914, green: 0.604, blue: 0.494, alpha: 1) // #E99A7E
        static let spec      = NSColor(srgbRed: 0.965, green: 0.804, blue: 0.745, alpha: 1) // #F6CDBE
        static let rim       = NSColor(srgbRed: 0.557, green: 0.247, blue: 0.157, alpha: 1) // #8E3F28
        static let screen    = NSColor(srgbRed: 0.090, green: 0.090, blue: 0.102, alpha: 1) // #17171A
        static let glare     = NSColor(srgbRed: 0.165, green: 0.165, blue: 0.188, alpha: 1) // #2A2A30
        static let glyph     = NSColor(srgbRed: 0.486, green: 0.890, blue: 0.545, alpha: 1) // #7CE38B
        static let glyphDim  = NSColor(srgbRed: 0.486, green: 0.890, blue: 0.545, alpha: 0.28)
        static let eye       = NSColor(srgbRed: 0.169, green: 0.106, blue: 0.090, alpha: 1)
        static let shine     = NSColor.white
        static let blush     = NSColor(srgbRed: 0.949, green: 0.545, blue: 0.604, alpha: 1) // #F28B9A
    }

    // MARK: Геометрия (координаты — центры пикселей, y растёт вниз)

    private static let tentacleX: [Double] = [6.5, 11, 16, 21, 25.5]
    private static let tentacleCurl: [Double] = [-2.6, -1.0, 0, 1.0, 2.6]

    private static func inHead(_ x: Double, _ y: Double) -> Bool {
        let dx = (x - 16) / 12, dy = (y - 10) / 9.5
        if dx * dx + dy * dy <= 1 { return true }
        // нижняя часть тела — прямоугольник со скруглёнными нижними углами
        guard x >= 4, x <= 28, y >= 10, y <= 21.5 else { return false }
        if y > 18.5 {
            for cx in [7.0, 25.0] where (cx == 7 ? x < 7 : x > 25) {
                return (x - cx) * (x - cx) + (y - 18.5) * (y - 18.5) <= 9
            }
        }
        return true
    }

    /// Номер щупальца и его центр по x в этой строке, если точка внутри.
    private static func tentacle(_ x: Double, _ y: Double, phase: Double) -> (center: Double, width: Double)? {
        guard y >= 19, y <= 30 else { return nil }
        let t = (y - 19) / 11
        for i in 0..<5 {
            let tipCut = (i == 2) ? 30.0 : 29.5
            guard y <= tipCut else { continue }
            let width = 3.6 * (1 - t * 0.55)
            let sway = sin(t * 3.2 + phase + Double(i) * 1.3) * 1.4 * t
            let center = tentacleX[i] + tentacleCurl[i] * t * t + sway
            if abs(x - center) <= width / 2 { return (center, width) }
        }
        return nil
    }

    private static func inScreen(_ x: Double, _ y: Double, pad: Double = 0) -> Bool {
        let x0 = 8 - pad, x1 = 24 + pad, y0 = 12 - pad, y1 = 20 + pad
        guard x >= x0, x <= x1, y >= y0, y <= y1 else { return false }
        let r = 1.6 + pad
        let cx = min(max(x, x0 + r), x1 - r), cy = min(max(y, y0 + r), y1 - r)
        return (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r
    }

    // MARK: Рендер

    static func render(tentaclePhase phase: Double, eyes: Eyes, blush: Bool, screen: Screen) -> [[NSColor?]] {
        var g = [[NSColor?]](repeating: [NSColor?](repeating: nil, count: n), count: n)

        func body(_ x: Int, _ y: Int) -> Bool {
            guard x >= 0, y >= 0, x < n, y < n else { return false }
            let cx = Double(x) + 0.5, cy = Double(y) + 0.5
            return inHead(cx, cy) || tentacle(cx, cy, phase: phase) != nil
        }

        for y in 0..<n {
            for x in 0..<n {
                let cx = Double(x) + 0.5, cy = Double(y) + 0.5
                if inHead(cx, cy) {
                    // свет падает слева сверху
                    let l = -((cx - 16) / 14 * 0.55 + (cy - 7) / 13 * 0.85)
                    var c = l > 0.32 ? P.highlight : (l < -0.55 ? P.shadow : P.base)
                    let sx = (cx - 10) / 2.6, sy = (cy - 4.2) / 1.5
                    if sx * sx + sy * sy <= 1 { c = P.spec }
                    g[y][x] = c
                } else if let t = tentacle(cx, cy, phase: phase) {
                    g[y][x] = cx > t.center + t.width * 0.12 ? P.shadow : P.base
                } else if body(x - 1, y) || body(x + 1, y) || body(x, y - 1) || body(x, y + 1) {
                    g[y][x] = P.outline
                }
            }
        }

        // экран с тёмной рамкой и бликом
        for y in 10..<23 {
            for x in 6..<26 {
                let cx = Double(x) + 0.5, cy = Double(y) + 0.5
                if inScreen(cx, cy) {
                    g[y][x] = (y == 12 && x >= 9 && x <= 12) || (y == 13 && x == 9) ? P.glare : P.screen
                } else if inScreen(cx, cy, pad: 1) {
                    g[y][x] = P.rim
                }
            }
        }

        // глаза
        for ex in [11.5, 20.5] {
            switch eyes {
            case .open(let lookUp):
                let ey = lookUp ? 7.0 : 8.0
                for y in 4..<11 {
                    for x in Int(ex) - 2...Int(ex) + 2 {
                        let dx = (Double(x) + 0.5 - ex) / 1.7, dy = (Double(y) + 0.5 - ey) / 2.3
                        if dx * dx + dy * dy <= 1 { g[y][x] = P.eye }
                    }
                }
                let top = Int(ey - 2)
                g[top + 1][Int(ex) - 1] = P.shine
            case .closed:
                let xi = Int(ex)
                g[8][xi - 1] = P.eye; g[9][xi] = P.eye; g[8][xi + 1] = P.eye   // довольная дуга
            }
        }

        if blush {
            for (x, y) in [(7, 10), (8, 10), (23, 10), (24, 10)] { g[y][x] = P.blush }
        }

        // содержимое экрана
        switch screen {
        case .prompt(let cursor):
            for (x, y) in [(10, 13), (11, 14), (12, 15), (11, 16), (10, 17)] { g[y][x] = P.glyph }
            if cursor { for x in 14...17 { g[17][x] = P.glyph } }
        case .thinking(let active):
            for i in 0..<3 {
                let c = i == active ? P.glyph : P.glyphDim
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] { g[15 + dy][11 + i * 4 + dx] = c }
            }
        case .listening(let tick):
            for x in 10...21 {
                let wave = abs(sin(Double(tick) * 0.6 + Double(x) * 0.9)) * abs(cos(Double(tick) * 0.23 + Double(x) * 0.4))
                let h = 1 + Int(wave * 5.5)
                for k in 0..<h { g[18 - k][x] = P.glyph }
            }
        case .off:
            break
        }
        return g
    }
}
