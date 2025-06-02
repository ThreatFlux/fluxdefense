#!/bin/bash

echo "🚀 FluxDefense Linux System Tray Demo"
echo "======================================"
echo ""

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ Error: This demo is for Linux only!"
    echo "   For macOS, use: ./scripts/system_tray_demo.sh"
    exit 1
fi

# Check if the tray app is already running
if pgrep -f "flux_tray_linux" > /dev/null; then
    echo "✅ FluxDefense System Tray is already running"
    echo "   Look for the icon in your system tray!"
else
    echo "📱 Starting FluxDefense System Tray..."
    
    # Check dependencies and choose script
    if python3 -c "import gi; gi.require_version('AppIndicator3', '0.1')" 2>/dev/null; then
        TRAY_SCRIPT="./scripts/flux_tray_linux.py"
        echo "   Using AppIndicator3 implementation"
    elif python3 -c "import gi; gi.require_version('Gtk', '3.0')" 2>/dev/null; then
        TRAY_SCRIPT="./scripts/flux_tray_linux_gtk.py"
        echo "   Using GTK StatusIcon implementation"
    else
        echo "❌ Missing dependencies! Please run:"
        echo "   ./scripts/setup_linux_tray.sh"
        exit 1
    fi
    
    # Start the tray application
    $TRAY_SCRIPT &
    TRAY_PID=$!
    echo "✅ System tray started with PID: $TRAY_PID"
fi

echo ""
echo "🔍 What you'll see in the system tray:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "CPU🟢12% RAM🟢45% (2.1GB) NET●🟢 DISK●🟢"
echo ""
echo "Legend:"
echo "  🟢/🟡/🔴 XX%  = Status with color-coded indicators"
echo "  CPU          = CPU usage percentage"
echo "  RAM          = Memory usage percentage and GB used"
echo "  NET ●/XM     = Network activity (● = idle, XM = MB/s)"
echo "  DISK ●/XM    = Disk activity (● = idle, XM = MB/s)"
echo ""
echo "Color coding:"
echo "  🟢 Green  = Normal (CPU <70%, Memory <80%)"
echo "  🟡 Yellow = Warning (CPU 70-90%, Memory 80-95%)"
echo "  🔴 Red    = Critical (CPU >90%, Memory >95%)"
echo ""

# Show current metrics
echo "📊 Current System Metrics:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Try to get metrics from flux-monitor
if [ -f "./target/release/flux-monitor" ]; then
    ./target/release/flux-monitor metrics --json --once 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    print(f\"🖥️  CPU: {data['cpu_usage']:.0f}% {'🟢' if data['cpu_usage'] < 70 else '🟡' if data['cpu_usage'] < 90 else '🔴'}\")
    print(f\"🧠 Memory: {data['memory_usage']:.0f}% ({data.get('memory_used_gb', 0):.1f}GB) {'🟢' if data['memory_usage'] < 80 else '🟡' if data['memory_usage'] < 95 else '🔴'}\")
    print(f\"📶 Network: ↓{data['network_rx_rate']/1024/1024:.1f}MB/s ↑{data['network_tx_rate']/1024/1024:.1f}MB/s\")
    print(f\"💾 Disk: R{data['disk_read_rate']/1024/1024:.1f}MB/s W{data['disk_write_rate']/1024/1024:.1f}MB/s\")
    print(f\"⚡ Load: {data['load_average'][0]:.2f} {data['load_average'][1]:.2f} {data['load_average'][2]:.2f}\")
    print(f\"🔢 Processes: {data['process_count']}\")
except:
    print('Failed to parse metrics')
"
else
    # Fallback to basic system info
    echo "🖥️  CPU: $(grep 'cpu ' /proc/stat | awk '{usage=($2+$3+$4+$6+$7+$8)*100/($2+$3+$4+$5+$6+$7+$8)} END {printf "%.0f%%", usage}')"
    echo "🧠 Memory: $(free | grep Mem | awk '{printf "%.0f%% (%.1fGB)", $3/$2 * 100.0, $3/1024/1024}')"
    echo "⚡ Load: $(cat /proc/loadavg | cut -d' ' -f1-3)"
    echo "🔢 Processes: $(ls -1 /proc | grep -E '^[0-9]+$' | wc -l)"
fi

echo ""
echo "🎯 Features:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Native Linux system tray using AppIndicator3"
echo "✅ Real-time metrics from /proc filesystem"
echo "✅ Integration with flux-monitor backend"
echo "✅ Color-coded status indicators"
echo "✅ Detailed tooltip on hover"
echo "✅ Menu with dashboard access"
echo "✅ Auto-updates every 3 seconds"
echo "✅ Lightweight Python implementation"
echo ""
echo "🔧 Menu Options:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 System metrics display (updated live)"
echo "🌐 Open Dashboard - Launch web interface"
echo "🔄 Refresh Now - Force metric update"
echo "ℹ️  About - Application information"
echo "❌ Quit - Exit the tray application"
echo ""
echo "💡 Tips:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• The tray icon updates automatically"
echo "• Click the icon to access the menu"
echo "• Works with GNOME, KDE, XFCE, and other desktops"
echo "• Requires python3-gi and libappindicator"
echo ""
echo "Press Ctrl+C to stop the demo..."

# Keep script running
trap 'echo ""; echo "🛑 Demo stopped."; exit 0' INT
while true; do
    sleep 10
    echo -n "."
done