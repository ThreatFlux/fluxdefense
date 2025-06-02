#!/bin/bash

echo "🚀 FluxDefense Linux System Tray Setup"
echo "======================================="
echo ""

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ Error: This script is for Linux only!"
    echo "   Detected OS: $OSTYPE"
    exit 1
fi

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for Python 3
if ! command_exists python3; then
    echo "❌ Python 3 is not installed"
    echo "   Please install: sudo apt-get install python3"
    exit 1
else
    echo "✅ Python 3 found: $(python3 --version)"
fi

# Check for required system packages
echo ""
echo "🔍 Checking system dependencies..."

MISSING_DEPS=""

# Check for GTK and AppIndicator
if ! python3 -c "import gi" 2>/dev/null; then
    MISSING_DEPS="$MISSING_DEPS python3-gi"
fi

# Check which tray implementation to use
TRAY_SCRIPT=""
if python3 -c "import gi; gi.require_version('AppIndicator3', '0.1')" 2>/dev/null; then
    echo "✅ AppIndicator3 found - using enhanced tray implementation"
    TRAY_SCRIPT="flux_tray_linux.py"
else
    echo "⚠️  AppIndicator3 not found - using GTK StatusIcon fallback"
    echo "   For better integration, install: gir1.2-appindicator3-0.1"
    TRAY_SCRIPT="flux_tray_linux_gtk.py"
fi

if [ -n "$MISSING_DEPS" ]; then
    echo "❌ Missing dependencies:$MISSING_DEPS"
    echo ""
    echo "📦 To install on Ubuntu/Debian:"
    echo "   sudo apt-get update"
    echo "   sudo apt-get install$MISSING_DEPS"
    echo ""
    echo "📦 To install on Fedora:"
    echo "   sudo dnf install python3-gobject gtk3 libappindicator-gtk3"
    echo ""
    echo "📦 To install on Arch:"
    echo "   sudo pacman -S python-gobject gtk3 libappindicator-gtk3"
    exit 1
else
    echo "✅ All Python dependencies are installed"
fi

# Check if flux-monitor is built
echo ""
echo "🔧 Checking FluxDefense build..."

FLUX_MONITOR="./target/release/flux-monitor"
if [ ! -f "$FLUX_MONITOR" ]; then
    echo "⚠️  flux-monitor not found, building..."
    cargo build --release --bin flux-monitor
    if [ $? -ne 0 ]; then
        echo "❌ Failed to build flux-monitor"
        exit 1
    fi
else
    echo "✅ flux-monitor found"
fi

# Create desktop entry for autostart (optional)
echo ""
read -p "Would you like to add FluxDefense to startup applications? [y/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    DESKTOP_FILE="$HOME/.config/autostart/fluxdefense-tray.desktop"
    mkdir -p "$HOME/.config/autostart"
    
    cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Type=Application
Name=FluxDefense System Tray
Comment=System monitoring and security
Exec=$PWD/scripts/$TRAY_SCRIPT
Hidden=false
NoDisplay=false
X-GNOME-Autostart-enabled=true
Icon=security-high
EOF
    
    echo "✅ Added to startup applications"
    echo "   Location: $DESKTOP_FILE"
fi

echo ""
echo "🎉 Setup complete!"
echo ""
echo "📊 To run FluxDefense System Tray:"
echo "   ./scripts/$TRAY_SCRIPT"
echo ""
echo "🖥️  The tray icon will appear in your system tray showing:"
echo "   - Real-time CPU and memory usage"
echo "   - Network and disk activity"
echo "   - Color-coded status indicators"
echo ""
echo "💡 Tips:"
echo "   - Click the tray icon to see the menu"
echo "   - The metrics update every 3 seconds"
echo "   - Hover over items to see detailed information"