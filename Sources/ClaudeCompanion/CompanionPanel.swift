import AppKit

/// Transparent borderless window, floats above all windows on every Space.
@MainActor
final class CompanionPanel: NSPanel {
    let companionView: CompanionView
    private static let originKey = "pixelOrigin"

    init() {
        companionView = CompanionView(frame: NSRect(origin: .zero, size: CompanionView.size))
        super.init(
            contentRect: companionView.frame,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        isOpaque = false
        backgroundColor = .clear
        hasShadow = false
        level = .floating
        collectionBehavior = [.canJoinAllSpaces, .stationary, .fullScreenAuxiliary, .ignoresCycle]
        hidesOnDeactivate = false
        isReleasedWhenClosed = false
        contentView = companionView
        restorePosition()
    }

    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }

    func savePosition() {
        UserDefaults.standard.set(NSStringFromPoint(frame.origin), forKey: Self.originKey)
    }

    private func restorePosition() {
        if let saved = UserDefaults.standard.string(forKey: Self.originKey) {
            let p = NSPointFromString(saved)
            let center = NSPoint(x: p.x + frame.width / 2, y: p.y + frame.height / 2)
            if NSScreen.screens.contains(where: { $0.frame.contains(center) }) {
                setFrameOrigin(p)
                return
            }
        }
        resetPosition()
    }

    func resetPosition() {
        guard let vf = NSScreen.main?.visibleFrame else { return }
        setFrameOrigin(NSPoint(x: vf.maxX - frame.width - 24, y: vf.minY + 24))
        savePosition()
    }
}
