#!/bin/bash

echo "Testing FluxDefense on Linux..."
echo "================================"

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "This script is designed for Linux systems only."
    exit 1
fi

# Check Ubuntu version
if [ -f /etc/os-release ]; then
    . /etc/os-release
    echo "OS: $NAME $VERSION"
fi

echo ""
echo "Building FluxDefense for Linux..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo ""
echo "Testing system metrics collection..."
echo "===================================="

# Create a simple test program to check metrics
cat > test_metrics.rs << 'EOF'
use fluxdefense::system_metrics::SystemMetricsCollector;

fn main() {
    println!("Testing Linux system metrics...\n");
    
    let mut collector = SystemMetricsCollector::new();
    
    match collector.collect_metrics() {
        Ok(metrics) => {
            println!("System Metrics:");
            println!("  CPU Usage: {:.1}%", metrics.cpu_usage);
            println!("  Memory Usage: {:.1}% ({} / {} bytes)", 
                metrics.memory_usage, 
                metrics.memory_used, 
                metrics.memory_total
            );
            println!("  Load Average: {:.2}, {:.2}, {:.2}", 
                metrics.load_average[0], 
                metrics.load_average[1], 
                metrics.load_average[2]
            );
            println!("  Uptime: {} seconds", metrics.uptime_seconds);
            println!("  Process Count: {}", metrics.process_count);
            println!("  Disk I/O: Read: {} bytes/s, Write: {} bytes/s", 
                metrics.disk_read_rate as u64, 
                metrics.disk_write_rate as u64
            );
            println!("  Network I/O: RX: {} bytes/s, TX: {} bytes/s", 
                metrics.network_rx_rate as u64, 
                metrics.network_tx_rate as u64
            );
            
            // Collect metrics again after a delay to see rates
            println!("\nWaiting 2 seconds to calculate rates...");
            std::thread::sleep(std::time::Duration::from_secs(2));
            
            match collector.collect_metrics() {
                Ok(metrics2) => {
                    println!("\nUpdated Metrics:");
                    println!("  Disk I/O Rate: Read: {:.1} KB/s, Write: {:.1} KB/s", 
                        metrics2.disk_read_rate / 1024.0, 
                        metrics2.disk_write_rate / 1024.0
                    );
                    println!("  Network I/O Rate: RX: {:.1} KB/s, TX: {:.1} KB/s", 
                        metrics2.network_rx_rate / 1024.0, 
                        metrics2.network_tx_rate / 1024.0
                    );
                }
                Err(e) => eprintln!("Failed to collect second metrics: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to collect metrics: {}", e),
    }
    
    println!("\nTesting process metrics...");
    match collector.get_top_processes(5) {
        Ok(processes) => {
            println!("Top 5 processes by CPU:");
            for (i, proc) in processes.iter().enumerate() {
                println!("  {}. {} (PID: {}) - CPU: {:.1}%, MEM: {:.1}%", 
                    i + 1, proc.name, proc.pid, proc.cpu_percent, proc.memory_percent
                );
            }
        }
        Err(e) => eprintln!("Failed to get process list: {}", e),
    }
}
EOF

echo "Compiling test program..."
rustc --edition 2021 -L target/release/deps test_metrics.rs -o test_metrics --extern fluxdefense=target/release/libfluxdefense.rlib

if [ $? -eq 0 ]; then
    echo "Running metrics test..."
    ./test_metrics
    rm -f test_metrics test_metrics.rs
else
    echo "Failed to compile test program"
    rm -f test_metrics.rs
fi

echo ""
echo "Testing FluxDefense passive monitoring..."
echo "========================================"

# Test the main binary
if [ -f target/release/fluxdefense ]; then
    echo "Starting FluxDefense in passive mode (press Ctrl+C to stop)..."
    echo ""
    
    # Run with timeout for testing
    timeout 5s target/release/fluxdefense 2>&1 | head -20
    
    echo ""
    echo "FluxDefense test completed."
else
    echo "FluxDefense binary not found!"
fi

echo ""
echo "Checking for required permissions..."
echo "===================================="

# Check if we have root or CAP_SYS_ADMIN
if [ "$EUID" -eq 0 ]; then
    echo "✓ Running as root - all security features available"
else
    echo "✗ Not running as root - some features may be limited"
    echo "  For full functionality, run with: sudo $0"
fi

# Check for required kernel features
echo ""
echo "Checking kernel features..."
if [ -d /proc/sys/kernel ]; then
    echo "✓ /proc filesystem available"
fi

if [ -d /sys/class/net ]; then
    echo "✓ /sys filesystem available"
fi

# Check for fanotify support
if [ -e /usr/include/linux/fanotify.h ] || [ -e /usr/include/x86_64-linux-gnu/sys/fanotify.h ]; then
    echo "✓ fanotify headers found"
else
    echo "✗ fanotify headers not found - file monitoring may be limited"
fi

echo ""
echo "Linux system monitoring test complete!"