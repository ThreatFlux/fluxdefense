#!/bin/bash

echo "🎯 FluxDefense System Tray - Enhanced Memory Display"
echo "===================================================="
echo ""
echo "✅ **NEW FEATURE: Real Memory Usage Display!** 🎉"
echo ""
echo "The system tray now shows actual RAM usage in GB alongside the percentage!"
echo ""
echo "📊 **What's New:**"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• **Previous format**: RAM🟢56%"
echo "• **New format**: RAM🟢56% (8.9GB)"
echo ""
echo "This shows:"
echo "  - Percentage of total RAM used (56%)"
echo "  - Actual GB of RAM being used (8.9GB)"
echo ""
echo "🔧 **Real System Memory Monitoring:**"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "• Uses macOS vm_stat command for accurate memory statistics"
echo "• Updates every 3 seconds with real-time data"
echo "• Shows physical memory usage (not virtual)"
echo ""
echo "📱 **Enhanced Tooltip Information:**"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Hover over the system tray to see:"
echo "• Memory: XX.X% (Y.Y GB / Z.Z GB)"
echo "• Shows used memory and total available memory"
echo ""
echo "🏃‍♂️ **Current System Memory Status:**"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Get actual memory info using vm_stat
echo "Fetching real memory statistics..."
echo ""

# Get total memory
TOTAL_MEM=$(sysctl -n hw.memsize)
TOTAL_GB=$(echo "scale=1; $TOTAL_MEM / 1024 / 1024 / 1024" | bc)

# Get memory usage from vm_stat
VM_STAT=$(vm_stat)
PAGE_SIZE=$(echo "$VM_STAT" | grep "page size" | awk '{print $8}')
FREE_PAGES=$(echo "$VM_STAT" | grep "Pages free" | awk '{print $3}' | tr -d '.')
ACTIVE_PAGES=$(echo "$VM_STAT" | grep "Pages active" | awk '{print $3}' | tr -d '.')
INACTIVE_PAGES=$(echo "$VM_STAT" | grep "Pages inactive" | awk '{print $3}' | tr -d '.')
WIRED_PAGES=$(echo "$VM_STAT" | grep "Pages wired" | awk '{print $4}' | tr -d '.')
COMPRESSED_PAGES=$(echo "$VM_STAT" | grep "Pages occupied by compressor" | awk '{print $5}' | tr -d '.')

# Calculate used pages
USED_PAGES=$((ACTIVE_PAGES + INACTIVE_PAGES + WIRED_PAGES + COMPRESSED_PAGES))
USED_BYTES=$((USED_PAGES * PAGE_SIZE))
USED_GB=$(echo "scale=1; $USED_BYTES / 1024 / 1024 / 1024" | bc)
USED_PERCENT=$(echo "scale=0; ($USED_BYTES * 100) / $TOTAL_MEM" | bc)

echo "• Total Physical Memory: ${TOTAL_GB} GB"
echo "• Memory Used: ${USED_GB} GB (${USED_PERCENT}%)"
echo "• Memory Free: $(echo "scale=1; $TOTAL_GB - $USED_GB" | bc) GB"
echo ""
echo "📊 **Memory Breakdown:**"
echo "  - Active: $(echo "scale=1; $ACTIVE_PAGES * $PAGE_SIZE / 1024 / 1024 / 1024" | bc) GB"
echo "  - Wired: $(echo "scale=1; $WIRED_PAGES * $PAGE_SIZE / 1024 / 1024 / 1024" | bc) GB"
echo "  - Compressed: $(echo "scale=1; $COMPRESSED_PAGES * $PAGE_SIZE / 1024 / 1024 / 1024" | bc) GB"
echo ""
echo "✨ **The system tray now provides instant visibility into**"
echo "✨ **your actual RAM usage, not just a percentage!**"