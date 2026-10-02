import AppKit

/// Clawd, the Claude Code mascot, block for block from the CLI banner (▐▛███▜▌ / ▝▜█████▛▘ / ▘▘ ▝▝).
/// One terminal quadrant is twice as tall as wide, so each banner row is two square pixels: 18×10.
enum PixelSprite {
    static let w = 18, h = 14
    static let top = 4  // rows above the head, for the fiesta sombrero

    enum Eyes { case open(lookUp: Bool), closed }
    enum Screen { case prompt(cursor: Bool), thinking(active: Int), listening(tick: Int), off }

    enum P {
        static let body  = NSColor(srgbRed: 0.851, green: 0.467, blue: 0.341, alpha: 1) // #D97757
        static let light = NSColor(srgbRed: 0.937, green: 0.635, blue: 0.533, alpha: 1) // blush
        static let eye   = NSColor(srgbRed: 0.169, green: 0.106, blue: 0.090, alpha: 1)
        static let straw = NSColor(srgbRed: 0.910, green: 0.757, blue: 0.353, alpha: 1)  // sombrero
        static let strawDark = NSColor(srgbRed: 0.722, green: 0.565, blue: 0.184, alpha: 1)
        static let band  = NSColor(srgbRed: 0.753, green: 0.224, blue: 0.169, alpha: 1)
        static let wood  = NSColor(srgbRed: 0.545, green: 0.290, blue: 0.157, alpha: 1)  // guitar
        static let neck  = NSColor(srgbRed: 0.290, green: 0.157, blue: 0.071, alpha: 1)
    }

    private static let legX = [4, 6, 11, 13]
    /// Leg phase with all four legs down (icons). Any real phase always has one leg lifted.
    static let standing = Double.nan

    /// `screen` drives the body language now: thinking — eyes look around, listening — arms flap.
    /// `fiesta`: sombrero + guitar; its value is the strum frame (odd = strumming hand down).
    static func render(legPhase phase: Double, eyes: Eyes, blush: Bool, screen: Screen, fiesta: Int? = nil) -> [[NSColor?]] {
        var g = [[NSColor?]](repeating: [NSColor?](repeating: nil, count: w), count: h)
        func put(_ x: Int, _ y: Int, _ c: NSColor?) { if x >= 0, y + top >= 0, x < w, y + top < h { g[y + top][x] = c } }

        for y in 0...7 { for x in 3...14 { put(x, y, P.body) } }          // body

        var armTop = 4
        if case .listening(let tick) = screen, (tick / 3) % 2 == 1 { armTop = 3 }
        for y in armTop...(armTop + 1) { put(1, y, P.body); put(2, y, P.body); put(15, y, P.body); put(16, y, P.body) }

        for (i, lx) in legX.enumerated() {
            let lifted = sin(phase + Double(i) * 1.7) > 0.6                // lift in turn
            put(lx, 8, P.body)
            if !lifted { put(lx, 9, P.body) }
        }

        var look = 0
        if case .thinking(let active) = screen { look = active - 1 }
        for ex in [5, 12] {
            switch eyes {
            case .open(let up):
                put(ex + look, up ? 1 : 2, P.eye); put(ex + look, up ? 2 : 3, P.eye)
            case .closed:
                put(ex, 3, P.eye)
            }
        }

        if blush { put(4, 5, P.light); put(13, 5, P.light) }

        guard let fiesta else { return g }

        // sombrero: crown, red band, wide brim with upturned tips
        for y in -4...(-3) { for x in 6...11 { put(x, y, P.straw) } }
        for x in 6...11 { put(x, -2, P.band) }
        for x in 1...16 { put(x, -1, P.straw) }
        put(0, -2, P.straw); put(17, -2, P.straw)
        for x in 3...14 { put(x, 0, P.strawDark) }                     // shadow on the forehead

        // guitar across the belly, neck in the left hand
        for y in 4...7 { for x in 9...13 where !((x == 9 || x == 13) && (y == 4 || y == 7)) { put(x, y, P.wood) } }
        put(11, 5, P.neck); put(11, 6, P.neck)                         // sound hole
        for x in 2...8 { put(x, 5, P.neck) }
        put(1, 4, P.neck)                                              // headstock
        if fiesta % 2 == 1 {                                           // strumming hand drops onto the strings
            for x in 15...16 { put(x, armTop, nil); put(x, armTop + 2, P.body) }
        }
        return g
    }
}
