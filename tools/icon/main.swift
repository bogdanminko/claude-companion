import AppKit

// App icon: Clawd on a dark rounded square, written as an .iconset for iconutil.
let dir = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "build/AppIcon.iconset"
try? FileManager.default.createDirectory(atPath: dir, withIntermediateDirectories: true)

let grid = PixelSprite.render(legPhase: PixelSprite.standing, eyes: .open(lookUp: false), blush: false, screen: .off).dropFirst(PixelSprite.top)

func png(_ size: Int) -> Data {
    let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: size, pixelsHigh: size, bitsPerSample: 8,
                               samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                               colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    let s = CGFloat(size)
    // macOS icon grid: 824/1024 rounded square, centred
    let inset = s * 100 / 1024, side = s - 2 * inset
    NSColor(srgbRed: 0.10, green: 0.10, blue: 0.11, alpha: 1).setFill()
    NSBezierPath(roundedRect: NSRect(x: inset, y: inset, width: side, height: side), xRadius: side * 0.225, yRadius: side * 0.225).fill()

    NSGraphicsContext.current?.shouldAntialias = false
    let p = (side * 0.78 / CGFloat(PixelSprite.w)).rounded(.down).clamped(min: 1)
    let ox = ((s - p * CGFloat(PixelSprite.w)) / 2).rounded()
    let oy = ((s - p * CGFloat(PixelSprite.h - PixelSprite.top)) / 2).rounded()
    for (y, row) in grid.enumerated() {
        for (x, c) in row.enumerated() {
            guard let c else { continue }
            c.setFill()
            NSRect(x: ox + CGFloat(x) * p, y: s - oy - CGFloat(y + 1) * p, width: p, height: p).fill()
        }
    }
    NSGraphicsContext.restoreGraphicsState()
    return rep.representation(using: .png, properties: [:])!
}

extension CGFloat { func clamped(min m: CGFloat) -> CGFloat { Swift.max(self, m) } }

for base in [16, 32, 128, 256, 512] {
    try! png(base).write(to: URL(fileURLWithPath: "\(dir)/icon_\(base)x\(base).png"))
    try! png(base * 2).write(to: URL(fileURLWithPath: "\(dir)/icon_\(base)x\(base)@2x.png"))
}
print("✓ \(dir)")
