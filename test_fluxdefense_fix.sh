#!/bin/bash

echo "Testing FluxDefenseUI startup..."
cd /Users/vtriple/fluxdefense/FluxDefenseUI

# Build the app
echo "Building..."
swift build

# Run the app for 5 seconds and capture output
echo "Running app for 5 seconds..."
timeout 5 .build/debug/FluxDefenseUI 2>&1 | tee app_output.log

echo "App output:"
cat app_output.log

# Check if any crash messages appear
if grep -q "NSInternalInconsistencyException\|bundleProxyForCurrentProcess is nil" app_output.log; then
    echo "❌ FAILED: App crashed with notification center error"
    exit 1
else
    echo "✅ SUCCESS: App started without notification center crash"
    exit 0
fi