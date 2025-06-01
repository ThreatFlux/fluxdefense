#!/bin/bash

# Build configuration
APP_NAME="FluxDefenseUI"
BUNDLE_ID="com.fluxdefense.ui"
BUILD_DIR=".build/release"
APP_PATH="$BUILD_DIR/$APP_NAME.app"

echo "Building FluxDefenseUI..."

# Build in release mode
swift build -c release

# Create app bundle structure
echo "Creating app bundle..."
rm -rf "$APP_PATH"
mkdir -p "$APP_PATH/Contents/MacOS"
mkdir -p "$APP_PATH/Contents/Resources"

# Copy executable
cp "$BUILD_DIR/$APP_NAME" "$APP_PATH/Contents/MacOS/"

# Copy Info.plist
cp Info.plist "$APP_PATH/Contents/"

# Create PkgInfo file
echo "APPL????" > "$APP_PATH/Contents/PkgInfo"

# Find available signing identities
echo ""
echo "Available signing identities:"
security find-identity -v -p codesigning

echo ""
echo "To sign the app, run:"
echo "codesign --force --deep --sign \"Developer ID Application: YOUR_NAME\" --entitlements FluxDefenseUI.entitlements \"$APP_PATH\""
echo ""
echo "Replace YOUR_NAME with your Developer ID from the list above."
echo ""
echo "After signing, you can verify with:"
echo "codesign -vvv --deep --strict \"$APP_PATH\""
echo ""
echo "To run the signed app:"
echo "open \"$APP_PATH\""