// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "Octo",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "Octo",
            path: "Sources/Octo"
        )
    ]
)
