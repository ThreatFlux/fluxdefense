#!/bin/bash

# Comprehensive Ubuntu system scan using file-scanner
# This collects detailed binary analysis data for all system files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SCANNER="$PROJECT_ROOT/file-scanner/target/release/file-scanner"
DATA_DIR="$PROJECT_ROOT/ubuntu-detailed-scan"

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "This script is designed for Linux systems only"
    exit 1
fi

# Build scanner if it doesn't exist
if [ ! -f "$SCANNER" ]; then
    echo "Building file-scanner..."
    cd "$PROJECT_ROOT/file-scanner"
    cargo build --release
fi

# Create data directory
mkdir -p "$DATA_DIR"
mkdir -p "$DATA_DIR/metadata"
mkdir -p "$DATA_DIR/binaries"
mkdir -p "$DATA_DIR/scripts"
mkdir -p "$DATA_DIR/libraries"
mkdir -p "$DATA_DIR/other"

echo "Starting comprehensive Ubuntu system scan..."
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

# Check if running with appropriate permissions
if [ "$EUID" -ne 0 ]; then
    echo "Note: Running without root privileges. Some files may not be accessible."
    echo "For complete system scan, run with: sudo $0"
fi

# Function to classify file type based on scanner output
classify_file() {
    local json_file="$1"
    local mime_type=$(jq -r '.mime_type' "$json_file" 2>/dev/null || echo "")
    local is_exec=$(jq -r '.is_executable' "$json_file" 2>/dev/null || echo "false")
    local file_path=$(jq -r '.file_path' "$json_file" 2>/dev/null || echo "")
    
    # Check binary info
    local binary_format=$(jq -r '.binary_info.format // empty' "$json_file" 2>/dev/null || echo "")
    
    if [[ -n "$binary_format" ]]; then
        echo "binaries"
    elif [[ "$file_path" =~ \.(sh|py|pl|rb|lua|tcl|awk|sed)$ ]]; then
        echo "scripts"
    elif [[ "$file_path" =~ \.(so|dylib|a)$ ]]; then
        echo "libraries"
    elif [[ "$is_exec" == "true" ]]; then
        echo "binaries"
    else
        echo "other"
    fi
}

# Scan function for individual files
scan_file() {
    local file_path="$1"
    local base_name=$(basename "$file_path")
    local safe_name=$(echo "$base_name" | sed 's/[^a-zA-Z0-9._-]/_/g')
    local temp_file="/tmp/scan_${safe_name}_$$.json"
    
    # Run scanner with all features
    if "$SCANNER" "$file_path" \
        --format json \
        --strings \
        --min-string-len 6 \
        --hex-dump \
        --hex-dump-size 256 \
        --verify-signatures \
        > "$temp_file" 2>/dev/null; then
        
        # Determine category
        local category=$(classify_file "$temp_file")
        
        # Generate hash-based filename to avoid conflicts
        local file_hash=$(jq -r '.hashes.sha256 // empty' "$temp_file" 2>/dev/null || echo "")
        if [[ -z "$file_hash" ]]; then
            file_hash=$(sha256sum "$file_path" | cut -d' ' -f1)
        fi
        
        local final_name="${file_hash:0:16}_${safe_name}.json"
        local final_path="$DATA_DIR/$category/$final_name"
        
        # Move to appropriate directory
        mv "$temp_file" "$final_path"
        echo "  ✓ $file_path -> $category/$final_name"
        return 0
    else
        rm -f "$temp_file"
        return 1
    fi
}

# Initialize counters
TOTAL_FILES=0
SCANNED_FILES=0
FAILED_FILES=0

# Create progress tracking file
PROGRESS_FILE="$DATA_DIR/scan_progress.log"
echo "Ubuntu Detailed System Scan - $(date)" > "$PROGRESS_FILE"
echo "======================================" >> "$PROGRESS_FILE"

# Scan each directory
for dir in "${EXISTING_DIRS[@]}"; do
    echo ""
    echo "Scanning directory: $dir"
    echo "Processing $dir..." >> "$PROGRESS_FILE"
    
    # Count files first
    FILE_COUNT=$(find "$dir" -type f 2>/dev/null | wc -l || echo 0)
    echo "  Found $FILE_COUNT files to scan"
    
    # Process files
    while IFS= read -r -d '' file; do
        TOTAL_FILES=$((TOTAL_FILES + 1))
        
        if scan_file "$file"; then
            SCANNED_FILES=$((SCANNED_FILES + 1))
        else
            FAILED_FILES=$((FAILED_FILES + 1))
            echo "  ✗ Failed: $file" >> "$PROGRESS_FILE"
        fi
        
        # Progress indicator every 100 files
        if [ $((TOTAL_FILES % 100)) -eq 0 ]; then
            echo "  Progress: $TOTAL_FILES files processed..."
        fi
        
    done < <(find "$dir" -type f -print0 2>/dev/null)
    
    echo "  Completed $dir: $SCANNED_FILES successful, $FAILED_FILES failed" >> "$PROGRESS_FILE"
done

# Generate summary report
SUMMARY_FILE="$DATA_DIR/scan_summary.json"
cat > "$SUMMARY_FILE" << EOF
{
  "scan_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "hostname": "$(hostname)",
  "os_info": {
    "distribution": "$(lsb_release -d 2>/dev/null | cut -f2 || echo 'Ubuntu')",
    "kernel": "$(uname -r)",
    "architecture": "$(uname -m)"
  },
  "scan_statistics": {
    "total_files_found": $TOTAL_FILES,
    "files_scanned": $SCANNED_FILES,
    "files_failed": $FAILED_FILES,
    "success_rate": $(awk "BEGIN {if ($TOTAL_FILES > 0) printf \"%.2f\", ($SCANNED_FILES/$TOTAL_FILES)*100; else print 0}")
  },
  "directories_scanned": $(echo "${EXISTING_DIRS[@]}" | jq -R -s -c 'split(" ")'),
  "categories": {
    "binaries": $(find "$DATA_DIR/binaries" -name "*.json" 2>/dev/null | wc -l || echo 0),
    "scripts": $(find "$DATA_DIR/scripts" -name "*.json" 2>/dev/null | wc -l || echo 0),
    "libraries": $(find "$DATA_DIR/libraries" -name "*.json" 2>/dev/null | wc -l || echo 0),
    "other": $(find "$DATA_DIR/other" -name "*.json" 2>/dev/null | wc -l || echo 0)
  }
}
EOF

# Generate category summaries
echo ""
echo "Generating category summaries..."

# Binary analysis summary
if [ -d "$DATA_DIR/binaries" ] && [ "$(ls -A "$DATA_DIR/binaries" 2>/dev/null)" ]; then
    echo "Analyzing binaries..."
    jq -s '[.[] | select(.binary_info != null) | {
        path: .file_path,
        format: .binary_info.format,
        arch: .binary_info.architecture,
        compiler: .binary_info.compiler,
        stripped: .binary_info.is_stripped,
        imports_count: (.binary_info.imports // [] | length),
        exports_count: (.binary_info.exports // [] | length)
    }]' "$DATA_DIR/binaries"/*.json > "$DATA_DIR/metadata/binaries_analysis.json"
fi

# String analysis summary
echo "Analyzing extracted strings..."
jq -s '[.[] | select(.extracted_strings != null) | {
    path: .file_path,
    total_strings: (.extracted_strings.total // 0),
    interesting_strings: (.extracted_strings.interesting // [])
}] | map(select(.total_strings > 0))' "$DATA_DIR"/*/*.json > "$DATA_DIR/metadata/strings_analysis.json" 2>/dev/null || true

# Security-relevant summary
echo "Creating security summary..."
cat > "$DATA_DIR/metadata/security_summary.json" << 'EOF'
{
  "scan_complete": true,
  "purpose": "Ubuntu system baseline for security monitoring",
  "data_categories": {
    "file_hashes": "All files have MD5, SHA256, SHA512, and BLAKE3 hashes",
    "binary_analysis": "ELF binaries analyzed for imports, exports, and compiler info",
    "string_extraction": "ASCII and Unicode strings extracted from all files",
    "permissions": "File permissions and ownership recorded",
    "timestamps": "Creation, modification, and access times captured"
  }
}
EOF

echo ""
echo "Scan complete!"
echo "========================================"
echo "Total files found:    $TOTAL_FILES"
echo "Successfully scanned: $SCANNED_FILES"
echo "Failed to scan:       $FAILED_FILES"
echo "Success rate:         $(awk "BEGIN {if ($TOTAL_FILES > 0) printf \"%.2f%%\", ($SCANNED_FILES/$TOTAL_FILES)*100; else print 0}")"
echo ""
echo "Results saved to: $DATA_DIR"
echo "Summary file: $SUMMARY_FILE"
echo "Progress log: $PROGRESS_FILE"
echo ""
echo "Category breakdown:"
echo "  Binaries:  $(find "$DATA_DIR/binaries" -name "*.json" 2>/dev/null | wc -l || echo 0)"
echo "  Scripts:   $(find "$DATA_DIR/scripts" -name "*.json" 2>/dev/null | wc -l || echo 0)"
echo "  Libraries: $(find "$DATA_DIR/libraries" -name "*.json" 2>/dev/null | wc -l || echo 0)"
echo "  Other:     $(find "$DATA_DIR/other" -name "*.json" 2>/dev/null | wc -l || echo 0)"