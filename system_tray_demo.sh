#!/bin/bash

echo "🚀 FluxDefense System Tray Demo"
echo "================================="
echo ""
echo "This demo shows real-time system metrics in the macOS system tray!"
echo ""

# Check if app is already running
if pgrep -f "FluxDefenseUI" > /dev/null; then
    echo "✅ FluxDefense UI is already running in system tray"
else
    echo "📱 Starting FluxDefense UI..."
    cd FluxDefenseUI
    ./.build/debug/FluxDefenseUI &
    UI_PID=$!
    echo "✅ FluxDefense UI started with PID: $UI_PID"
fi

echo ""
echo "🔍 What you'll see in the system tray:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🟢12% 🟢45% 📶🟢● 💾🟢●"
echo ""
echo "Legend:"
echo "  🟢/🟡/🔴 12%  = CPU usage with color-coded status"
echo "  🟢/🟡/🔴 45%  = Memory usage with color-coded status"  
echo "  📶🟢/● or XM = Network activity (● = idle, XM = MB/s)"
echo "  💾🟢/● or XM = Disk activity (● = idle, XM = MB/s)"
echo ""
echo "Color coding:"
echo "  🟢 Green  = Normal (CPU <70%, Memory <80%)"
echo "  🟡 Yellow = Warning (CPU 70-90%, Memory 80-95%)"
echo "  🔴 Red    = Critical (CPU >90%, Memory >95%)"
echo ""
echo "📊 Test the JSON API that powers the UI:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Show current metrics
./target/release/flux-monitor metrics --json --once | jq -r '
  "🖥️  CPU: \(.cpu_usage | floor)% \("🟢🟡🔴"[if .cpu_usage >= 90 then 2 elif .cpu_usage >= 70 then 1 else 0 end:1])",
  "🧠 Memory: \(.memory_usage | floor)% \("🟢🟡🔴"[if .memory_usage >= 95 then 2 elif .memory_usage >= 80 then 1 else 0 end:1])",
  "📶 Network: ↓\(.network_rx_rate/1024/1024 | floor)MB/s ↑\(.network_tx_rate/1024/1024 | floor)MB/s",
  "💾 Disk: R\(.disk_read_rate/1024/1024 | floor)MB/s W\(.disk_write_rate/1024/1024 | floor)MB/s",
  "⚡ Load: \(.load_average[0] | . * 100 / 100)",
  "🔢 Processes: \(.process_count)"
'

echo ""
echo "🎯 Features implemented:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Real-time CPU percentage from macOS 'top' command"
echo "✅ Real-time memory usage from 'vm_stat'"  
echo "✅ Network I/O rates from 'netstat -ib'"
echo "✅ Disk I/O rates from 'iostat'"
echo "✅ Color-coded status indicators (green/yellow/red)"
echo "✅ Hover tooltip with detailed system information"
echo "✅ SwiftUI integration with Rust backend via JSON API"
echo "✅ Auto-updating every 3 seconds"
echo ""
echo "🔮 Future enhancements:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔄 Historical charts in popup dashboard"
echo "📈 Customizable update intervals"
echo "⚙️  User-configurable thresholds" 
echo "🔔 System alerts for critical usage"
echo "📊 Export metrics to CSV/JSON"
echo ""
echo "Look for FluxDefense in your macOS menu bar!"
echo "Click the metrics to open the full dashboard."
echo ""
echo "Press Ctrl+C to stop monitoring..."

# Keep script running so user can see the demo
trap 'echo ""; echo "🛑 Demo stopped."; exit 0' INT
while true; do
    sleep 10
    echo -n "."
done