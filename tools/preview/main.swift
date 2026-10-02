import AppKit

// Превью спрайта: несколько состояний в ряд, 8 физ. пикселей на пиксель спрайта.
// Запуск: make preview  →  build/preview.png
let states: [(OctoSprite.Eyes, Bool, OctoSprite.Screen, Double)] = [
    (.open(lookUp: false), false, .prompt(cursor: true), 0),
    (.open(lookUp: true), true, .prompt(cursor: false), 1.2),
    (.open(lookUp: false), false, .thinking(active: 1), 2.4),
    (.open(lookUp: false), false, .listening(tick: 3), 3.6),
    (.closed, false, .off, 4.8),
]
let s = 8, n = OctoSprite.n, gap = 16
let w = states.count * (n * s + gap) + gap, h = n * s + 2 * gap
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: w, pixelsHigh: h, bitsPerSample: 8,
                           samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                           colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
NSColor(srgbRed: 0.08, green: 0.08, blue: 0.08, alpha: 1).setFill()
NSRect(x: 0, y: 0, width: w, height: h).fill()
for (i, st) in states.enumerated() {
    let g = OctoSprite.render(tentaclePhase: st.3, eyes: st.0, blush: st.1, screen: st.2)
    let ox = gap + i * (n * s + gap)
    for y in 0..<n {
        for x in 0..<n {
            guard let c = g[y][x] else { continue }
            c.setFill()
            NSRect(x: ox + x * s, y: h - gap - (y + 1) * s, width: s, height: s).fill()
        }
    }
}
NSGraphicsContext.restoreGraphicsState()
let out = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "build/preview.png"
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: out))
print("✓ \(out)")
