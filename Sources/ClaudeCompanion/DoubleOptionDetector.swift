import AppKit

/// Detects a system-wide double press of Option (left or right).
/// A press counts only if Option is pressed and released alone, with no other keys.
@MainActor
final class DoubleOptionDetector {
    var onDoubleTap: (() -> Void)?

    private static let optionKeyCodes: Set<UInt16> = [58, 61] // left and right Option
    private static let maxTapDuration: TimeInterval = 0.35
    private static let doubleTapWindow: TimeInterval = 0.4

    private var monitors: [Any] = []
    private var optionDownAt: Date?
    private var lastTapAt = Date.distantPast

    func start() {
        guard monitors.isEmpty else { return }
        let mask: NSEvent.EventTypeMask = [.flagsChanged, .keyDown, .leftMouseDown, .rightMouseDown]
        if let g = NSEvent.addGlobalMonitorForEvents(matching: mask, handler: { [weak self] e in
            MainActor.assumeIsolated { self?.handle(e) }
        }) { monitors.append(g) }
        // the global monitor gets no events while Pixel itself is active
        if let l = NSEvent.addLocalMonitorForEvents(matching: mask, handler: { [weak self] e in
            MainActor.assumeIsolated { self?.handle(e) }
            return e
        }) { monitors.append(l) }
    }

    func stop() {
        monitors.forEach { NSEvent.removeMonitor($0) }
        monitors.removeAll()
        reset()
    }

    private func reset() {
        optionDownAt = nil
        lastTapAt = .distantPast
    }

    private func handle(_ e: NSEvent) {
        guard e.type == .flagsChanged else { reset(); return } // any key or click resets the sequence

        let mods = e.modifierFlags
            .intersection(.deviceIndependentFlagsMask)
            .subtracting([.capsLock, .function, .numericPad])
        let isOptionKey = Self.optionKeyCodes.contains(e.keyCode)

        if isOptionKey, mods == .option {
            optionDownAt = Date()                          // Option pressed alone
        } else if isOptionKey, mods.isEmpty, let down = optionDownAt {
            optionDownAt = nil                             // Option released
            let now = Date()
            guard now.timeIntervalSince(down) < Self.maxTapDuration else { reset(); return }
            if now.timeIntervalSince(lastTapAt) < Self.doubleTapWindow {
                lastTapAt = .distantPast
                onDoubleTap?()
            } else {
                lastTapAt = now
            }
        } else {
            reset()                                        // another modifier or Caps Lock
        }
    }
}
