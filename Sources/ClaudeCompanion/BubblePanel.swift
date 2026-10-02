import AppKit

/// Dark capsule with two icon buttons: Claude and Claude Code. Shows on hover over Pixel.
@MainActor
final class BubblePanel: NSPanel {
    var onClaude: (() -> Void)?
    var onCode: (() -> Void)?

    private static let size = NSSize(width: 112, height: 50)
    private var anchor: NSRect = .zero
    private var outsideClickMonitor: Any?
    private var hideCheck: DispatchWorkItem?

    init() {
        super.init(
            contentRect: NSRect(origin: .zero, size: Self.size),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        level = .floating
        collectionBehavior = [.canJoinAllSpaces, .stationary, .fullScreenAuxiliary, .ignoresCycle]
        hidesOnDeactivate = false
        isReleasedWhenClosed = false

        let capsule = NSView(frame: NSRect(origin: .zero, size: Self.size))
        capsule.wantsLayer = true
        capsule.layer?.backgroundColor = NSColor(srgbRed: 0.16, green: 0.16, blue: 0.17, alpha: 0.96).cgColor
        capsule.layer?.cornerRadius = Self.size.height / 2
        capsule.layer?.borderWidth = 1
        capsule.layer?.borderColor = NSColor(white: 1, alpha: 0.12).cgColor

        let claudeIcon = NSWorkspace.shared.urlForApplication(withBundleIdentifier: ClaudeApp.claudeBundleID)
            .map { NSWorkspace.shared.icon(forFile: $0.path) } ?? NSImage()
        let claudeButton = Self.makeButton(image: claudeIcon, label: "Open Claude", action: #selector(claudeTapped))
        let codeButton = Self.makeButton(image: Self.clawdIcon(), label: "Open Claude Code", action: #selector(codeTapped))
        claudeButton.target = self
        codeButton.target = self

        let half = Self.size.width / 2
        claudeButton.frame = NSRect(x: 6, y: 5, width: half - 8, height: 40)
        codeButton.frame = NSRect(x: half + 2, y: 5, width: half - 8, height: 40)

        let divider = NSView(frame: NSRect(x: half - 0.5, y: 13, width: 1, height: 24))
        divider.wantsLayer = true
        divider.layer?.backgroundColor = NSColor(white: 1, alpha: 0.18).cgColor

        capsule.addSubview(claudeButton)
        capsule.addSubview(divider)
        capsule.addSubview(codeButton)
        contentView = capsule
        capsule.addTrackingArea(NSTrackingArea(rect: .zero, options: [.mouseEnteredAndExited, .activeAlways, .inVisibleRect], owner: self))
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    private static func makeButton(image: NSImage, label: String, action: Selector) -> NSButton {
        if image.size.width > 40 { image.size = NSSize(width: 30, height: 30) } // app icon; pixel art keeps its own size
        let b = NSButton(image: image, target: nil, action: action)
        b.isBordered = false
        b.imageScaling = .scaleNone
        b.toolTip = label
        b.setAccessibilityLabel(label)
        return b
    }

    /// Claude Code icon: the same Clawd sprite at 2 pt per pixel.
    private static func clawdIcon() -> NSImage {
        let grid = PixelSprite.render(legPhase: PixelSprite.standing, eyes: .open(lookUp: false), blush: false, screen: .off).dropFirst(PixelSprite.top)
        let p: CGFloat = 2
        return NSImage(size: NSSize(width: CGFloat(PixelSprite.w) * p, height: CGFloat(PixelSprite.h - PixelSprite.top) * p), flipped: true) { _ in
            NSGraphicsContext.current?.shouldAntialias = false
            for (y, row) in grid.enumerated() {
                for (x, c) in row.enumerated() {
                    guard let c else { continue }
                    c.setFill()
                    NSRect(x: CGFloat(x) * p, y: CGFloat(y) * p, width: p, height: p).fill()
                }
            }
            return true
        }
    }

    // MARK: Show / hide

    func toggle(below anchor: NSRect) {
        isVisible ? hide() : show(below: anchor)
    }

    func show(below anchor: NSRect) {
        self.anchor = anchor
        hideCheck?.cancel()
        guard !isVisible else { return }
        let w = Self.size.width, h = Self.size.height
        var origin = NSPoint(x: anchor.midX - w / 2, y: anchor.minY - h + 14)
        let screen = NSScreen.screens.first { $0.frame.intersects(anchor) } ?? NSScreen.main
        if let vf = screen?.visibleFrame {
            if origin.y < vf.minY { origin.y = anchor.maxY - 10 }        // no room below — show above the head
            origin.x = min(max(origin.x, vf.minX + 4), vf.maxX - w - 4)
        }
        setFrameOrigin(origin)
        alphaValue = 0
        orderFrontRegardless()
        NSAnimationContext.runAnimationGroup { $0.duration = 0.12; animator().alphaValue = 1 }

        outsideClickMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { _ in
            MainActor.assumeIsolated { self.hide() }
        }
    }

    /// Mouse left Pixel or the capsule: hide shortly unless it moved onto the other one.
    func scheduleHide() {
        hideCheck?.cancel()
        let work = DispatchWorkItem { [weak self] in
            guard let self else { return }
            let m = NSEvent.mouseLocation
            if !self.frame.contains(m), !self.anchor.contains(m) { self.hide() }
        }
        hideCheck = work
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5, execute: work)
    }

    func hide() {
        hideCheck?.cancel()
        if let m = outsideClickMonitor { NSEvent.removeMonitor(m); outsideClickMonitor = nil }
        orderOut(nil)
    }

    override func mouseEntered(with event: NSEvent) { hideCheck?.cancel() }
    override func mouseExited(with event: NSEvent) { scheduleHide() }

    @objc private func claudeTapped() { hide(); onClaude?() }
    @objc private func codeTapped() { hide(); onCode?() }
}
