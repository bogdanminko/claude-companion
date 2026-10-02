import AppKit
import ImageIO
import UniformTypeIdentifiers

// Demo GIFs for the README: drives the real CompanionView offscreen, one GIF per scene.
//   make demo  →  docs/*.gif
let out = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "docs"
try? FileManager.default.createDirectory(atPath: out, withIntermediateDirectories: true)
_ = NSApplication.shared

let scale = 2
let bg = NSColor(srgbRed: 0.08, green: 0.08, blue: 0.09, alpha: 1)

@MainActor
func record(_ name: String, frames: Int, setup: (CompanionView) -> Void) {
    let view = CompanionView(frame: NSRect(origin: .zero, size: CompanionView.size))
    setup(view)
    let w = Int(view.bounds.width) * scale, h = Int(view.bounds.height) * scale
    let url = URL(fileURLWithPath: "\(out)/\(name).gif") as CFURL
    let dest = CGImageDestinationCreateWithURL(url, UTType.gif.identifier as CFString, frames, nil)!
    CGImageDestinationSetProperties(dest, [kCGImagePropertyGIFDictionary: [kCGImagePropertyGIFLoopCount: 0]] as CFDictionary)
    let frameProps = [kCGImagePropertyGIFDictionary: [kCGImagePropertyGIFDelayTime: 0.1]] as CFDictionary
    for _ in 0..<frames {
        view.step()
        let rep = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: w, pixelsHigh: h, bitsPerSample: 8,
                                   samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
                                   colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
        rep.size = view.bounds.size
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
        bg.setFill()
        view.bounds.fill()
        NSGraphicsContext.restoreGraphicsState()
        view.cacheDisplay(in: view.bounds, to: rep)
        CGImageDestinationAddImage(dest, rep.cgImage!, frameProps)
    }
    CGImageDestinationFinalize(dest)
    print("✓ \(out)/\(name).gif")
}

MainActor.assumeIsolated {
    record("idle", frames: 50) { _ in }
    record("fiesta", frames: 60) { $0.startTrick(.fiesta) }
    record("jump", frames: 24) { v in v.startTrick(.jump) }
    record("glitch", frames: 20) { $0.startTrick(.glitch) }
    record("wave", frames: 24) { $0.startTrick(.wave) }
    record("look", frames: 24) { $0.startTrick(.look) }
    record("shake", frames: 20) { $0.startTrick(.shake) }
    record("sleep", frames: 40) { $0.goToSleep() }
}
