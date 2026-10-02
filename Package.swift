// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "ClaudeCompanion",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "ClaudeCompanion",
            path: "Sources/ClaudeCompanion"
        )
    ]
)
