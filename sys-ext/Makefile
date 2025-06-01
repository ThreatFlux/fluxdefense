# FluxDefense System Extension Build Makefile

PRODUCT_NAME = FluxDefenseExtension
BUNDLE_ID = com.fluxdefense.extension
TEAM_ID = YOUR_TEAM_ID_HERE
SIGNING_IDENTITY = "Developer ID Application: Your Name ($(TEAM_ID))"

# Paths
BUILD_DIR = build
RUST_LIB_DIR = ../target/release
EXTENSION_DIR = $(BUILD_DIR)/$(PRODUCT_NAME).systemextension
CONTENTS_DIR = $(EXTENSION_DIR)/Contents
MACOS_DIR = $(CONTENTS_DIR)/MacOS
RESOURCES_DIR = $(CONTENTS_DIR)/Resources

# Swift build configuration
SWIFT_BUILD_CONFIG = --configuration release
SWIFT_EXEC = $(BUILD_DIR)/release/$(PRODUCT_NAME)

.PHONY: all clean rust-lib swift-build create-bundle sign notarize

all: create-bundle

# Build Rust library
rust-lib:
	@echo "Building Rust library..."
	cd .. && cargo build --release --lib
	@echo "Rust library built successfully"

# Build Swift executable
swift-build:
	@echo "Building Swift extension..."
	swift build $(SWIFT_BUILD_CONFIG)
	@echo "Swift extension built successfully"

# Create system extension bundle
create-bundle: rust-lib swift-build
	@echo "Creating system extension bundle..."
	
	# Create directory structure
	mkdir -p $(MACOS_DIR)
	mkdir -p $(RESOURCES_DIR)
	
	# Copy Swift executable
	cp $(SWIFT_EXEC) $(MACOS_DIR)/
	
	# Copy Rust library
	cp $(RUST_LIB_DIR)/libfluxdefense.dylib $(MACOS_DIR)/ || \
	cp $(RUST_LIB_DIR)/libfluxdefense.a $(MACOS_DIR)/ || \
	echo "Warning: Rust library not found"
	
	# Copy Info.plist
	cp Info.plist $(CONTENTS_DIR)/
	
	# Update bundle identifier and version in Info.plist
	/usr/libexec/PlistBuddy -c "Set :CFBundleIdentifier $(BUNDLE_ID)" $(CONTENTS_DIR)/Info.plist
	
	@echo "System extension bundle created at: $(EXTENSION_DIR)"

# Code signing
sign: create-bundle
	@echo "Signing system extension..."
	codesign --force --sign $(SIGNING_IDENTITY) \
		--entitlements entitlements.plist \
		--options runtime \
		$(EXTENSION_DIR)
	@echo "System extension signed successfully"

# Verify code signature
verify: sign
	@echo "Verifying code signature..."
	codesign --verify --verbose $(EXTENSION_DIR)
	spctl --assess --verbose $(EXTENSION_DIR)

# Install system extension (requires user approval)
install: sign
	@echo "Installing system extension..."
	@echo "Note: This requires user approval in System Preferences > Security & Privacy"
	systemextensionsctl install $(EXTENSION_DIR)

# Uninstall system extension
uninstall:
	@echo "Uninstalling system extension..."
	systemextensionsctl uninstall $(TEAM_ID) $(BUNDLE_ID)

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	rm -rf $(BUILD_DIR)
	swift package clean
	cd .. && cargo clean

# Development helpers
dev-setup:
	@echo "Setting up development environment..."
	@echo "1. Update TEAM_ID in Makefile with your Apple Developer Team ID"
	@echo "2. Update SIGNING_IDENTITY with your Developer ID Application certificate"
	@echo "3. Request Endpoint Security entitlement from Apple"
	@echo "4. Install Xcode command line tools if not already installed"

# Build for distribution
dist: clean sign verify
	@echo "Creating distribution package..."
	mkdir -p dist
	cp -R $(EXTENSION_DIR) dist/
	tar -czf dist/FluxDefenseExtension.tar.gz -C dist $(PRODUCT_NAME).systemextension
	@echo "Distribution package created: dist/FluxDefenseExtension.tar.gz"