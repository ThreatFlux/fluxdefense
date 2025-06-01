#!/bin/bash
cd /Users/vtriple/fluxdefense/FluxDefenseUI
echo "Starting FluxDefenseUI..."
(.build/debug/FluxDefenseUI > output.log 2>&1 &)
PID=$!
echo "App started with PID: $PID"
sleep 3
echo "Checking if app is still running..."
if ps -p $PID > /dev/null; then
    echo "✅ App is running successfully!"
    echo "First 50 lines of output:"
    head -50 output.log
    kill $PID
else
    echo "❌ App crashed or exited"
    echo "Full output:"
    cat output.log
fi