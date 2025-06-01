// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FluxDefenseUI",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(
            name: "FluxDefenseUI",
            targets: ["FluxDefenseUI"]
        )
    ],
    dependencies: [
        .package(url: "https://github.com/sindresorhus/LaunchAtLogin", from: "5.0.0")
    ],
    targets: [
        .executableTarget(
            name: "FluxDefenseUI",
            dependencies: [
                "LaunchAtLogin"
            ]
        )
    ]
)