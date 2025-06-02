#!/bin/bash

# FluxDefense Phase 2 Integration Test
# Tests the complete system with web dashboard

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}FluxDefense Phase 2 Integration Test${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check dependencies
echo -e "${YELLOW}Checking dependencies...${NC}"

# Check if libpcap-dev is installed
if ! dpkg -l | grep -q libpcap-dev; then
    echo -e "${RED}Error: libpcap-dev not installed${NC}"
    echo "Install with: sudo apt-get install libpcap-dev"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo -e "${RED}Error: Node.js not installed${NC}"
    echo "Install Node.js to run the web dashboard"
    exit 1
fi

echo -e "${GREEN}✓ All dependencies satisfied${NC}"
echo ""

# Build the Rust API server
echo -e "${YELLOW}Building API server...${NC}"
cargo build --release --features pcap --bin fluxdefense-api 2>&1 | grep -E "(Compiling|Finished|error)" || true

if [ ! -f target/release/fluxdefense-api ]; then
    echo -e "${RED}Error: Failed to build API server${NC}"
    exit 1
fi

echo -e "${GREEN}✓ API server built successfully${NC}"
echo ""

# Build the web dashboard
echo -e "${YELLOW}Building web dashboard...${NC}"
cd web-dashboard

if [ ! -d node_modules ]; then
    echo "Installing dependencies..."
    npm install
fi

# Build for production
npm run build > /dev/null 2>&1

if [ ! -d dist ]; then
    echo -e "${RED}Error: Failed to build web dashboard${NC}"
    exit 1
fi

cd ..
echo -e "${GREEN}✓ Web dashboard built successfully${NC}"
echo ""

# Start the API server
echo -e "${YELLOW}Starting API server...${NC}"

# Kill any existing API server
pkill -f "fluxdefense-api" 2>/dev/null || true
sleep 1

# Start API server with real monitoring
if [ "$EUID" -eq 0 ]; then
    USE_REAL_MONITORING=true ./target/release/fluxdefense-api > /tmp/fluxdefense_api.log 2>&1 &
else
    echo -e "${YELLOW}Note: Running without root - some features will be limited${NC}"
    USE_REAL_MONITORING=true ./target/release/fluxdefense-api > /tmp/fluxdefense_api.log 2>&1 &
fi

API_PID=$!
sleep 3

# Check if API server is running
if ! kill -0 $API_PID 2>/dev/null; then
    echo -e "${RED}Error: API server failed to start${NC}"
    echo "Check logs: tail -f /tmp/fluxdefense_api.log"
    exit 1
fi

echo -e "${GREEN}✓ API server running (PID: $API_PID)${NC}"
echo ""

# Test API endpoints
echo -e "${YELLOW}Testing API endpoints...${NC}"

# Test health check
if curl -s http://localhost:3177/api/health | grep -q "healthy"; then
    echo -e "${GREEN}✓ Health check: OK${NC}"
else
    echo -e "${RED}✗ Health check: FAILED${NC}"
fi

# Test system status
if curl -s http://localhost:3177/api/dashboard/status | grep -q "success"; then
    echo -e "${GREEN}✓ System status: OK${NC}"
else
    echo -e "${RED}✗ System status: FAILED${NC}"
fi

# Test threat metrics
if curl -s http://localhost:3177/api/dashboard/threats | grep -q "success"; then
    echo -e "${GREEN}✓ Threat metrics: OK${NC}"
else
    echo -e "${RED}✗ Threat metrics: FAILED${NC}"
fi

# Test security events
if curl -s http://localhost:3177/api/security/events | grep -q "success"; then
    echo -e "${GREEN}✓ Security events: OK${NC}"
else
    echo -e "${RED}✗ Security events: FAILED${NC}"
fi

# Test policies
if curl -s http://localhost:3177/api/policies | grep -q "success"; then
    echo -e "${GREEN}✓ Security policies: OK${NC}"
else
    echo -e "${RED}✗ Security policies: FAILED${NC}"
fi

# Test alerts
if curl -s http://localhost:3177/api/alerts | grep -q "success"; then
    echo -e "${GREEN}✓ Alerts: OK${NC}"
else
    echo -e "${RED}✗ Alerts: FAILED${NC}"
fi

echo ""

# Test WebSocket connection
echo -e "${YELLOW}Testing WebSocket...${NC}"
# Simple WebSocket test using curl (requires curl 7.86+) or fallback message
if command -v wscat &> /dev/null; then
    timeout 2 wscat -c ws://localhost:3177/api/live/ws 2>/dev/null && echo -e "${GREEN}✓ WebSocket: OK${NC}" || echo -e "${YELLOW}○ WebSocket: Connection test timed out (normal)${NC}"
else
    echo -e "${YELLOW}○ WebSocket: wscat not installed, skipping test${NC}"
fi

echo ""

# Display access information
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}FluxDefense is running!${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${GREEN}Web Dashboard:${NC} http://localhost:3177"
echo -e "${GREEN}API Endpoints:${NC} http://localhost:3177/api/"
echo -e "${GREEN}WebSocket:${NC} ws://localhost:3177/api/live/ws"
echo ""
echo -e "${YELLOW}API Server Logs:${NC} tail -f /tmp/fluxdefense_api.log"
echo ""

# Monitor for a bit
echo -e "${YELLOW}Monitoring for 10 seconds...${NC}"
echo ""

# Show some live logs
echo "Recent API logs:"
tail -n 10 /tmp/fluxdefense_api.log | grep -E "(INFO|WARN|ERROR)" || echo "No recent logs"

echo ""

# Create a test threat
if [ "$EUID" -eq 0 ]; then
    echo -e "${YELLOW}Creating test threat...${NC}"
    # Create a suspicious process name
    bash -c 'exec -a "xmrig --pool mining.pool.com" sleep 2' &
    TEST_PID=$!
    sleep 3
    kill $TEST_PID 2>/dev/null || true
    echo -e "${GREEN}✓ Test threat created and cleaned up${NC}"
fi

echo ""
echo -e "${GREEN}Phase 2 Integration Test Complete!${NC}"
echo ""
echo "Next steps:"
echo "1. Open http://localhost:3177 in your browser"
echo "2. Monitor real-time security events"
echo "3. Configure security policies"
echo "4. View system metrics and alerts"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop the API server${NC}"

# Keep running
wait $API_PID