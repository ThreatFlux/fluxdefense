// swift-tools-version:5.7
import PackageDescription

let package = Package(
    name: "FluxDefenseExtension",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(
            name: "FluxDefenseExtension",
            targets: ["FluxDefenseExtension"]
        )
    ],
    dependencies: [],
    targets: [
        .executableTarget(
            name: "FluxDefenseExtension",
            dependencies: [],
            path: "Sources/FluxDefenseExtension",
            linkerSettings: [
                .linkedFramework("EndpointSecurity"),
                .linkedFramework("NetworkExtension"),
                .linkedFramework("SystemExtensions"),
                .linkedLibrary("fluxdefense", .when(platforms: [.macOS]))
            ]
        )
    ]
)