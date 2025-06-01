# Linux Support for FluxDefense

FluxDefense now supports both macOS and Linux platforms. This document describes the Linux-specific features and implementation.

## System Monitoring (Fully Implemented)

The system metrics module now provides complete Linux support for:

### CPU Monitoring
- Reads from `/proc/stat` to calculate CPU usage percentage
- Supports multi-core systems
- Tracks user, system, idle, iowait, and other CPU states

### Memory Monitoring  
- Reads from `/proc/meminfo` for memory statistics
- Calculates used memory considering buffers, cache, and reclaimable memory
- Reports total memory, used memory, and usage percentage

### Disk I/O Monitoring
- Reads from `/proc/diskstats` for disk statistics
- Filters physical disks (SCSI/SATA and NVMe)
- Calculates read/write rates in bytes per second
- Supports multiple disk devices

### Network I/O Monitoring
- Reads from `/proc/net/dev` for network interface statistics
- Filters out loopback and virtual interfaces
- Tracks received and transmitted bytes
- Calculates network rates per interface

### Process Monitoring
- Compatible `ps` command usage for process listing
- Load average from `/proc/loadavg`
- Uptime from `/proc/uptime`
- Process count tracking

## Security Features (Foundation Implemented)

### Linux Security Modules Created:
1. **Fanotify Monitor** - File system monitoring using fanotify API
   - Requires root or CAP_SYS_ADMIN capability
   - Can monitor file execution, access, and modifications
   - Permission-based blocking capabilities

2. **Netlink Monitor** - Network connection monitoring
   - Uses NETLINK_INET_DIAG for TCP/UDP connection tracking
   - Monitors established connections and state changes
   - Maps connections to processes via inodes

3. **Process Monitor** - Process tracking and information
   - Reads from `/proc/[pid]/*` for process details
   - Tracks process hierarchy (parent-child relationships)
   - Maps processes to network connections

## Testing

A comprehensive test script (`scripts/test_linux.sh`) is provided that:
- Verifies the OS is Linux (tested on Ubuntu 24.04)
- Builds FluxDefense with Linux support
- Tests all system metrics collection
- Verifies passive monitoring mode
- Checks for required permissions and kernel features

## Usage

On Linux systems, FluxDefense automatically:
- Uses passive monitoring mode (no kernel extensions required)
- Collects system metrics from `/proc` and `/sys` filesystems
- Logs security events without blocking

### Running on Linux:
```bash
# Regular user (limited features)
./target/release/fluxdefense

# With full capabilities (requires root)
sudo ./target/release/fluxdefense
```

### Testing:
```bash
# Run the Linux test suite
./scripts/test_linux.sh

# With root for full feature testing
sudo ./scripts/test_linux.sh
```

## Implementation Details

### Conditional Compilation
- Platform-specific code uses `#[cfg(target_os = "linux")]` and `#[cfg(target_os = "macos")]`
- Shared interfaces ensure cross-platform compatibility
- Graceful degradation when features are unavailable

### File System Structure
- `/proc/stat` - CPU statistics
- `/proc/meminfo` - Memory information
- `/proc/diskstats` - Disk I/O statistics
- `/proc/net/dev` - Network interface statistics
- `/proc/loadavg` - System load averages
- `/proc/uptime` - System uptime
- `/proc/[pid]/*` - Process information

### Security Considerations
- Fanotify requires elevated privileges (root or CAP_SYS_ADMIN)
- Netlink socket access may require specific permissions
- Process information visibility depends on user permissions
- Passive mode operates without special privileges

## Future Enhancements

Potential areas for expansion:
1. eBPF-based monitoring for more detailed security events
2. SELinux/AppArmor integration
3. Audit subsystem integration
4. Container (Docker/Podman) awareness
5. systemd service integration
6. Distribution-specific packages (deb, rpm)