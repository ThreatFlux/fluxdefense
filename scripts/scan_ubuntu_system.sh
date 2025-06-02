#!/bin/bash

# Scan Ubuntu system directories to generate whitelist data
# Run with sudo for complete system access

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCANNER="$PROJECT_ROOT/target/debug/file-scanner"
DATA_DIR="$PROJECT_ROOT/ubuntu-whitelist-data"

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "This script is designed for Linux systems only"
    exit 1
fi

# Build scanner if it doesn't exist
if [ ! -f "$SCANNER" ]; then
    echo "Building file-scanner..."
    cd "$PROJECT_ROOT"
    cargo build --bin file-scanner
fi

# Create data directory
mkdir -p "$DATA_DIR"

echo "Starting Ubuntu system scan..."
echo "Data will be saved to: $DATA_DIR"

# Important Ubuntu/Linux system directories to scan
SYSTEM_DIRS=(
    "/usr/bin"
    "/usr/sbin"
    "/bin"
    "/sbin"
    "/usr/lib"
    "/usr/lib64"
    "/lib"
    "/lib64"
    "/usr/libexec"
    "/usr/share/applications"
    "/opt"
)

# Check which directories exist
EXISTING_DIRS=()
for dir in "${SYSTEM_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        EXISTING_DIRS+=("$dir")
    fi
done

echo "Scanning directories: ${EXISTING_DIRS[*]}"

# Run the scanner
if [ "$EUID" -ne 0 ]; then
    echo "Note: Running without root privileges. Some files may not be accessible."
    echo "For complete system scan, run with: sudo $0"
fi

"$SCANNER" \
    --data-dir "$DATA_DIR" \
    --max-depth 5 \
    "${EXISTING_DIRS[@]}"

# Count results
if [ -d "$DATA_DIR" ]; then
    FILE_COUNT=$(find "$DATA_DIR" -name "*.json" | wc -l)
    echo "Scan complete! Generated $FILE_COUNT whitelist entries."
    
    # Generate summary
    echo "Generating scan summary..."
    cat > "$DATA_DIR/scan_summary.txt" << EOF
Ubuntu System Scan Summary
==========================
Date: $(date)
Host: $(hostname)
OS: $(lsb_release -d 2>/dev/null | cut -f2 || echo "Ubuntu")
Kernel: $(uname -r)
Files scanned: $FILE_COUNT

Directories scanned:
$(printf '%s\n' "${EXISTING_DIRS[@]}")
EOF
    
    echo "Summary saved to: $DATA_DIR/scan_summary.txt"
else
    echo "Error: No data directory found after scan"
    exit 1
fi