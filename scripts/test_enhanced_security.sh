#!/bin/bash

set -e

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║        FluxDefense Enhanced Security Framework Test           ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ This test requires Linux (detected: $OSTYPE)"
    exit 1
fi

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ This test requires root privileges for fanotify"
    echo "Please run with: sudo $0"
    exit 1
fi

echo "✅ Running on Linux with root privileges"
echo

# Build the project
echo "🔨 Building FluxDefense with enhanced monitor..."
cargo build --release --bin test-enhanced-monitor 2>&1 | grep -E "(Compiling|Finished)" || true

if [ ! -f "target/release/test-enhanced-monitor" ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo

# Function to run a test
run_test() {
    local test_name="$1"
    local mode="$2"
    local args="$3"
    local duration="${4:-10}"
    
    echo "═══════════════════════════════════════════════════════════════"
    echo "🧪 Test: $test_name"
    echo "   Mode: $mode"
    echo "   Duration: ${duration}s"
    echo "═══════════════════════════════════════════════════════════════"
    
    # Run the monitor in background
    ./target/release/test-enhanced-monitor --mode "$mode" $args --duration "$duration" &
    MONITOR_PID=$!
    
    # Wait a bit for monitor to start
    sleep 2
    
    # Perform test actions
    echo "📋 Performing test actions..."
    
    # Test file operations
    echo "test" > /tmp/fluxdefense_test.txt
    cat /tmp/fluxdefense_test.txt > /dev/null
    rm -f /tmp/fluxdefense_test.txt
    
    # Test command execution
    ls /tmp > /dev/null
    ps aux | head -n 5 > /dev/null
    
    # Test network operations (if available)
    if command -v curl &> /dev/null; then
        curl -s -m 2 http://example.com > /dev/null 2>&1 || true
    fi
    
    if command -v ping &> /dev/null; then
        ping -c 1 -W 1 8.8.8.8 > /dev/null 2>&1 || true
    fi
    
    # Wait for monitor to finish
    wait $MONITOR_PID
    
    echo
}

# Test 1: Passive mode (logging only)
run_test "Passive Mode - Log Only" "passive" "" 5

# Test 2: Permissive mode with some rules
run_test "Permissive Mode with Rules" "permissive" \
    "--allow-exe /bin/ls --allow-exe /bin/cat --deny-path /etc/shadow" 5

# Test 3: Test suspicious pattern detection
echo
echo "═══════════════════════════════════════════════════════════════"
echo "🧪 Test: Suspicious Pattern Detection"
echo "═══════════════════════════════════════════════════════════════"

# Create a test script that looks like a crypto miner
cat > /tmp/fake_miner.sh << 'EOF'
#!/bin/bash
# This is a fake miner for testing
echo "Starting xmrig..."
echo "Connecting to pool.example.com:3333"
echo "--donate-level 1"
sleep 5
EOF

chmod +x /tmp/fake_miner.sh

# Run monitor and execute the fake miner
./target/release/test-enhanced-monitor --mode "permissive" --duration 10 &
MONITOR_PID=$!

sleep 2
echo "📋 Executing suspicious script..."
/tmp/fake_miner.sh &
MINER_PID=$!

wait $MONITOR_PID
kill $MINER_PID 2>/dev/null || true
rm -f /tmp/fake_miner.sh

echo
echo "✅ Enhanced security framework tests completed!"
echo
echo "📊 Summary:"
echo "   • Fanotify file monitoring with permission events"
echo "   • File hash calculation and caching"
echo "   • Process behavior analysis"
echo "   • Suspicious pattern detection"
echo "   • Policy-based decision making"
echo
echo "🎯 Next steps:"
echo "   1. Implement network filtering with netfilter"
echo "   2. Add eBPF integration for deeper monitoring"
echo "   3. Build web dashboard for real-time visualization"
echo "   4. Create comprehensive whitelist management"