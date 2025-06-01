# FluxDefense System Extension

This directory contains the macOS System Extension that provides the kernel-level integration for FluxDefense.

## Overview

The system extension is written in Swift and acts as a bridge between macOS system frameworks (Endpoint Security and Network Extension) and the Rust-based FluxDefense core logic.

## Architecture

```
┌─────────────────────────────────────┐
│          macOS Kernel               │
│  ┌─────────────┐ ┌─────────────────┐│
│  │ EndpointSec │ │ NetworkExtension││
│  └─────────────┘ └─────────────────┘│
└─────────────┬─────────────┬─────────┘
              │             │
┌─────────────▼─────────────▼─────────┐
│        Swift System Extension       │
│  ┌─────────────┐ ┌─────────────────┐│
│  │ ESF Client  │ │ Network Filter  ││
│  └─────────────┘ └─────────────────┘│
└─────────────┬───────────────────────┘
              │ FFI Calls
┌─────────────▼───────────────────────┐
│         Rust FluxDefense Core       │
│  ┌─────────────┐ ┌─────────────────┐│
│  │File Policies│ │Network Policies ││
│  └─────────────┘ └─────────────────┘│
└─────────────────────────────────────┘
```

## Prerequisites

### Apple Developer Account
- Paid Apple Developer Program membership ($99/year)
- Developer ID Application certificate
- Endpoint Security entitlement (requires approval from Apple)

### Development Tools
- Xcode 14+ or Xcode Command Line Tools
- Swift 5.7+
- macOS 13.0+ (target system)

## Entitlements Required

The system extension requires special entitlements from Apple:

1. **Endpoint Security Client** (`com.apple.developer.endpoint-security.client`)
   - Required for file system monitoring
   - Must be requested from Apple via developer portal

2. **Network Extension** (`com.apple.developer.networking.networkextension`)
   - Required for network traffic filtering
   - Includes packet-tunnel-provider and content-filter-provider

## Building

### 1. Set Up Signing
Edit the `Makefile` and update:
```make
TEAM_ID = YOUR_TEAM_ID_HERE
SIGNING_IDENTITY = "Developer ID Application: Your Name (TEAM_ID)"
```

### 2. Build Rust Library
```bash
cd /path/to/fluxdefense
cargo build --release --lib
```

### 3. Build System Extension
```bash
cd sys-ext
make all
```

### 4. Sign the Extension
```bash
make sign
```

### 5. Verify Signature
```bash
make verify
```

## Installation

### Development Installation
```bash
make install
```

This will:
1. Copy the extension to the system
2. Prompt user for approval in System Preferences
3. Load the extension if approved

### User Approval Process
1. System Preferences → Security & Privacy
2. Allow system extension from your developer team
3. Extension will start automatically after approval

## Testing

### Check Extension Status
```bash
systemextensionsctl list
```

### View Extension Logs
```bash
log stream --predicate 'subsystem == "com.fluxdefense.extension"'
```

### Uninstall Extension
```bash
make uninstall
```

## File Structure

```
sys-ext/
├── Package.swift              # Swift Package Manager configuration
├── Sources/
│   └── FluxDefenseExtension/
│       └── main.swift         # Main extension implementation
├── Info.plist                 # Bundle configuration
├── entitlements.plist         # Required entitlements
├── Makefile                   # Build and signing automation
└── README.md                  # This file
```

## Key Components

### ESFClient
- Handles Endpoint Security Framework integration
- Monitors file system events (exec, open, create, etc.)
- Calls Rust backend for policy decisions
- Responds with allow/deny verdicts

### NetworkExtensionFilter
- Integrates with Network Extension framework
- Monitors network connections
- Filters traffic based on Rust policy engine

### FFI Bridge
- Provides C-compatible interface to Rust
- Handles data marshaling between Swift and Rust
- Manages lifecycle of Rust components

## Troubleshooting

### Common Issues

1. **Entitlement Missing**
   ```
   Error: Endpoint Security entitlement not found
   ```
   Solution: Request entitlement from Apple Developer portal

2. **Code Signing Failed**
   ```
   Error: No valid signing identity found
   ```
   Solution: Install Developer ID Application certificate

3. **Extension Not Loading**
   ```
   Error: System extension approval required
   ```
   Solution: Approve in System Preferences → Security & Privacy

### Debug Mode
Enable verbose logging:
```bash
sudo log config --mode "level:debug" --subsystem com.fluxdefense.extension
```

### Reset Extension System
If extensions are stuck:
```bash
sudo systemextensionsctl reset
```

## Security Considerations

- Extension runs with high privileges
- All policy decisions flow through Rust backend
- Logs may contain sensitive file paths
- Network filtering can impact system performance

## Distribution

For production deployment:
1. Build with release configuration
2. Sign with Distribution certificate
3. Notarize with Apple
4. Distribute via signed installer package

```bash
make dist
```