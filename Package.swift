// swift-tools-version: 6.2
import PackageDescription

let package = Package(
    name: "md2cb",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(name: "md2cb"),
    ]
)
