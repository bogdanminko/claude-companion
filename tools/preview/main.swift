import AppKit

// Sprite preview: pixel-preview <out.png> — all states side by side.
let out = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "build/preview.png"

let frames: [(PixelSprite.Eyes, Bool, PixelSprite.Screen, Double, Int?)] = [
    (.open(lookUp: false), false, .prompt(cursor: true), 0, nil),
    (.open(lookUp: true), true, .prompt(cursor: false), 1.2, nil),
    (.open(lookUp: false), false, .thinking(active: 0), 2.4, 0),
    (.open(lookUp: false), false, .listening(tick: 3), 3.6, nil),
    (.closed, false, .off, 4.8, 1),
]

let s = 14, gap = 40
let cw = PixelSprite.w * s, ch = PixelSprite.h * s
let w = frames.count * (cw + gap) + gap, h = ch + 2 * gap
let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: w, pixelsHigh: h, bitsPerSample: 8,
                           samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                           colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
NSColor(srgbRed: 0.08, green: 0.08, blue: 0.08, alpha: 1).setFill()
NSRect(x: 0, y: 0, width: w, height: h).fill()
for (i, f) in frames.enumerated() {
    let g = PixelSprite.render(legPhase: f.3, eyes: f.0, blush: f.1, screen: f.2, fiesta: f.4)
    let ox = gap + i * (cw + gap)
    for (y, row) in g.enumerated() {
        for (x, c) in row.enumerated() {
            guard let c else { continue }
            c.setFill()
            NSRect(x: ox + x * s, y: h - gap - (y + 1) * s, width: s, height: s).fill()
        }
    }
}
NSGraphicsContext.restoreGraphicsState()
try! rep.representation(using: .png, properties: [:])!.write(to: URL(fileURLWithPath: out))
print("✓ \(out)")
