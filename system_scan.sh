#!/bin/bash

# FluxDefense Complete System Scan Script
# This will scan the entire macOS system to build a comprehensive whitelist

set -e

SCANNER="./target/release/file-scanner"
DATA_DIR="./system-whitelist-data"
LOG_FILE="./system_scan.log"

echo "Starting FluxDefense Complete System Scan at $(date)"
echo "Data directory: $DATA_DIR"
echo "Log file: $LOG_FILE"

# Ensure scanner is built
if [ ! -f "$SCANNER" ]; then
    echo "Building file-scanner..."
    cargo build --bin file-scanner --release --no-default-features
fi

# Create data directory
mkdir -p "$DATA_DIR"

# Function to run scanner with logging
run_scan() {
    local description="$1"
    shift
    echo "$(date): Starting $description..." | tee -a "$LOG_FILE"
    $SCANNER --data-dir "$DATA_DIR" "$@" 2>&1 | tee -a "$LOG_FILE"
    echo "$(date): Completed $description" | tee -a "$LOG_FILE"
    echo "----------------------------------------" | tee -a "$LOG_FILE"
}

# System Core Directories
echo "=== SCANNING SYSTEM CORE DIRECTORIES ===" | tee -a "$LOG_FILE"
run_scan "System root directories" --max-depth 3 \
    /System/Library/Frameworks \
    /System/Library/PrivateFrameworks \
    /System/Library/CoreServices \
    /System/Applications \
    /System/Cryptexes

# System Binaries
echo "=== SCANNING SYSTEM BINARIES ===" | tee -a "$LOG_FILE"
run_scan "System binaries" --max-depth 2 \
    /usr/bin \
    /usr/sbin \
    /usr/libexec \
    /bin \
    /sbin

# System Libraries
echo "=== SCANNING SYSTEM LIBRARIES ===" | tee -a "$LOG_FILE"
run_scan "System libraries" --max-depth 2 \
    /usr/lib \
    /usr/local/lib \
    /System/Library/Extensions

# Applications
echo "=== SCANNING APPLICATIONS ===" | tee -a "$LOG_FILE"
run_scan "Applications directory" \
    /Applications

# Library directories
echo "=== SCANNING LIBRARY DIRECTORIES ===" | tee -a "$LOG_FILE"
run_scan "Library directories" --max-depth 3 \
    /Library/Apple \
    /Library/Application\ Support \
    /Library/Frameworks \
    /Library/LaunchAgents \
    /Library/LaunchDaemons \
    /Library/PrivilegedHelperTools

# Developer tools (if present)
echo "=== SCANNING DEVELOPER TOOLS ===" | tee -a "$LOG_FILE"
if [ -d "/Applications/Xcode.app" ]; then
    run_scan "Xcode developer tools" --max-depth 4 \
        /Applications/Xcode.app/Contents/Developer/usr/bin \
        /Applications/Xcode.app/Contents/Developer/Platforms \
        /Applications/Xcode.app/Contents/Developer/Toolchains
fi

# Homebrew (if present)
echo "=== SCANNING HOMEBREW ===" | tee -a "$LOG_FILE"
if [ -d "/opt/homebrew" ]; then
    run_scan "Homebrew ARM64" --max-depth 2 \
        /opt/homebrew/bin \
        /opt/homebrew/sbin \
        /opt/homebrew/lib
fi

if [ -d "/usr/local/Homebrew" ] || [ -d "/usr/local/bin/brew" ]; then
    run_scan "Homebrew Intel" --max-depth 2 \
        /usr/local/bin \
        /usr/local/sbin \
        /usr/local/lib
fi

# User Applications (if accessible)
echo "=== SCANNING USER DIRECTORIES ===" | tee -a "$LOG_FILE"
if [ -d "$HOME/Applications" ]; then
    run_scan "User Applications" \
        "$HOME/Applications"
fi

# Common third-party locations
echo "=== SCANNING THIRD-PARTY LOCATIONS ===" | tee -a "$LOG_FILE"
for dir in "/opt" "/usr/local/share" "/Library/Internet Plug-Ins"; do
    if [ -d "$dir" ]; then
        run_scan "Third-party directory: $dir" --max-depth 3 "$dir"
    fi
done

# Generate final report
echo "=== GENERATING FINAL REPORT ===" | tee -a "$LOG_FILE"
echo "$(date): System scan completed!" | tee -a "$LOG_FILE"

if [ -f "$DATA_DIR/scan_manifest.json" ]; then
    echo "Scan statistics:" | tee -a "$LOG_FILE"
    grep -E "(total_files_scanned|files_by_type)" "$DATA_DIR/scan_manifest.json" | tee -a "$LOG_FILE"
    
    echo "" | tee -a "$LOG_FILE"
    echo "Data directory size:" | tee -a "$LOG_FILE"
    du -sh "$DATA_DIR" | tee -a "$LOG_FILE"
    
    echo "" | tee -a "$LOG_FILE"
    echo "Number of file records:" | tee -a "$LOG_FILE"
    ls -1 "$DATA_DIR"/*.json 2>/dev/null | wc -l | tee -a "$LOG_FILE"
fi

echo "$(date): Complete system scan finished!" | tee -a "$LOG_FILE"
echo "Results stored in: $DATA_DIR" | tee -a "$LOG_FILE"
echo "Log file: $LOG_FILE" | tee -a "$LOG_FILE"