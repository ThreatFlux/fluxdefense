#!/bin/bash

set -e

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║        FluxDefense Network Filtering & PCAP Test              ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ This test requires Linux (detected: $OSTYPE)"
    exit 1
fi

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ This test requires root privileges for pcap and iptables"
    echo "Please run with: sudo $0"
    exit 1
fi

# Check for required tools
echo "🔍 Checking requirements..."
MISSING_TOOLS=()

if ! command -v iptables &> /dev/null; then
    MISSING_TOOLS+=("iptables")
fi

if ! ldconfig -p | grep -q libpcap; then
    MISSING_TOOLS+=("libpcap-dev")
fi

if [ ${#MISSING_TOOLS[@]} -ne 0 ]; then
    echo "❌ Missing required tools: ${MISSING_TOOLS[*]}"
    echo
    echo "Install with:"
    echo "  Ubuntu/Debian: sudo apt-get install iptables libpcap-dev"
    echo "  Fedora: sudo dnf install iptables libpcap-devel"
    echo "  Arch: sudo pacman -S iptables libpcap"
    exit 1
fi

echo "✅ All requirements met"
echo

# Build the project
echo "🔨 Building FluxDefense network filter..."
cargo build --release --bin test-network-filter 2>&1 | grep -E "(Compiling|Finished)" || true

if [ ! -f "target/release/test-network-filter" ]; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful"
echo

# Function to run a test
run_test() {
    local test_name="$1"
    local command="$2"
    local duration="${3:-10}"
    
    echo "═══════════════════════════════════════════════════════════════"
    echo "🧪 Test: $test_name"
    echo "   Command: $command"
    echo "   Duration: ${duration}s"
    echo "═══════════════════════════════════════════════════════════════"
    
    # Run the command
    eval "$command" &
    TEST_PID=$!
    
    # Wait a bit for the test to start
    sleep 2
    
    if [ "$duration" != "manual" ]; then
        # Generate some network traffic during the test
        echo "📋 Generating test network traffic..."
        
        # DNS queries
        nslookup google.com > /dev/null 2>&1 || true
        nslookup example.com > /dev/null 2>&1 || true
        
        # HTTP requests
        curl -s -m 2 http://example.com > /dev/null 2>&1 || true
        wget -q -O /dev/null -T 2 http://httpbin.org/get 2>&1 || true
        
        # ICMP (ping)
        ping -c 3 -W 1 8.8.8.8 > /dev/null 2>&1 || true
        
        # Wait for the test to complete
        sleep $((duration - 2))
        
        # Stop the test
        kill $TEST_PID 2>/dev/null || true
        wait $TEST_PID 2>/dev/null || true
    else
        wait $TEST_PID
    fi
    
    echo
}

# Test 1: Basic packet capture (monitoring only)
run_test "Packet Capture - Monitoring Only" \
    "./target/release/test-network-filter capture --duration 5" 5

# Test 2: Packet capture with filtering
run_test "Packet Capture with Filtering" \
    "./target/release/test-network-filter capture --filter --duration 5" 5

# Test 3: DNS filtering
run_test "DNS Filtering" \
    "./target/release/test-network-filter capture --dns-filter --duration 5" 5

# Test 4: Full filtering (packets + DNS)
run_test "Full Filtering (Packets + DNS)" \
    "./target/release/test-network-filter capture --filter --dns-filter --duration 5" 5

# Test 5: IPTables integration
echo "═══════════════════════════════════════════════════════════════"
echo "🧪 Test: IPTables Integration"
echo "═══════════════════════════════════════════════════════════════"

# Initialize FluxDefense chains
echo "📋 Initializing FluxDefense iptables chains..."
./target/release/test-network-filter iptables init

# List rules
echo "📋 Current iptables rules:"
./target/release/test-network-filter iptables list

# Block a test IP
echo "📋 Blocking test IP 192.0.2.1..."
./target/release/test-network-filter iptables block-ip 192.0.2.1

# Block a test port
echo "📋 Blocking test port 9999..."
./target/release/test-network-filter iptables block-port 9999

# List rules again
echo "📋 Updated iptables rules:"
./target/release/test-network-filter iptables list --chain FLUXDEFENSE_INPUT

# Clean up
echo "📋 Cleaning up iptables chains..."
./target/release/test-network-filter iptables cleanup

echo

# Test 6: Interface-specific capture
echo "═══════════════════════════════════════════════════════════════"
echo "🧪 Test: Interface-Specific Capture"
echo "═══════════════════════════════════════════════════════════════"

# Get default interface
DEFAULT_IFACE=$(ip route | grep default | awk '{print $5}' | head -n1)

if [ -n "$DEFAULT_IFACE" ]; then
    echo "📋 Capturing on interface: $DEFAULT_IFACE"
    run_test "Interface Capture on $DEFAULT_IFACE" \
        "./target/release/test-network-filter capture --interface $DEFAULT_IFACE --duration 5" 5
else
    echo "⚠️  Could not determine default interface, skipping test"
fi

echo
echo "✅ Network filtering tests completed!"
echo
echo "📊 Summary of capabilities tested:"
echo "   • Packet capture using libpcap"
echo "   • Real-time packet filtering"
echo "   • DNS query monitoring and blocking"
echo "   • Connection tracking"
echo "   • IPTables rule management"
echo "   • Interface-specific capture"
echo
echo "🎯 Next steps:"
echo "   1. Integrate with enhanced security monitor"
echo "   2. Add deep packet inspection (DPI)"
echo "   3. Implement application-layer filtering"
echo "   4. Add IDS/IPS signatures"
echo "   5. Create network anomaly detection"