use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub disk_read_rate: f64,
    pub disk_write_rate: f64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub network_rx_rate: f64,
    pub network_tx_rate: f64,
    pub load_average: [f64; 3],
    pub process_count: u32,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f64,
    pub memory_bytes: u64,
    pub memory_percent: f64,
    pub command: String,
}

#[derive(Debug)]
pub struct SystemMetricsCollector {
    previous_metrics: Option<SystemMetrics>,
    previous_timestamp: Option<u64>,
}

impl SystemMetricsCollector {
    pub fn new() -> Self {
        Self {
            previous_metrics: None,
            previous_timestamp: None,
        }
    }

    pub fn collect_metrics(&mut self) -> anyhow::Result<SystemMetrics> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let cpu_usage = self.get_cpu_usage()?;
        let (memory_usage, memory_total, memory_used) = self.get_memory_usage()?;
        let (disk_read_bytes, disk_write_bytes, disk_read_rate, disk_write_rate) = 
            self.get_disk_io_metrics()?;
        let (network_rx_bytes, network_tx_bytes, network_rx_rate, network_tx_rate) = 
            self.get_network_metrics()?;
        let load_average = self.get_load_average()?;
        let process_count = self.get_process_count()?;
        let uptime_seconds = self.get_uptime()?;

        let metrics = SystemMetrics {
            timestamp,
            cpu_usage,
            memory_usage,
            memory_total,
            memory_used,
            disk_read_bytes,
            disk_write_bytes,
            disk_read_rate,
            disk_write_rate,
            network_rx_bytes,
            network_tx_bytes,
            network_rx_rate,
            network_tx_rate,
            load_average,
            process_count,
            uptime_seconds,
        };

        // Store for next calculation
        self.previous_metrics = Some(metrics.clone());
        self.previous_timestamp = Some(timestamp);

        Ok(metrics)
    }

    fn get_cpu_usage(&self) -> anyhow::Result<f64> {
        #[cfg(target_os = "macos")]
        {
            use std::mem;
            use std::ptr;

            // Get CPU info using host_processor_info
            let mut processor_count: u32 = 0;
            let mut cpu_info: *mut i32 = ptr::null_mut();
            let mut cpu_info_count: u32 = 0;

            // This is a simplified approach - in a real implementation,
            // you'd use mach system calls to get actual CPU usage
            let output = Command::new("top")
                .args(&["-l", "1", "-n", "0", "-s", "0"])
                .output()?;

            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse CPU usage from top output
            for line in output_str.lines() {
                if line.contains("CPU usage:") {
                    // Example: "CPU usage: 5.12% user, 2.34% sys, 92.54% idle"
                    if let Some(idle_start) = line.find("% idle") {
                        if let Some(idle_percent_start) = line[..idle_start].rfind(' ') {
                            if let Ok(idle_percent) = line[idle_percent_start + 1..idle_start].parse::<f64>() {
                                return Ok(100.0 - idle_percent);
                            }
                        }
                    }
                }
            }
            
            // Fallback to load average approximation
            let load = self.get_load_average()?;
            let cpu_count = num_cpus::get() as f64;
            Ok((load[0] / cpu_count * 100.0).min(100.0))
        }

        #[cfg(target_os = "linux")]
        {
            // Read from /proc/stat for CPU usage calculation
            let stat_content = fs::read_to_string("/proc/stat")?;
            let first_line = stat_content.lines().next()
                .ok_or_else(|| anyhow::anyhow!("Failed to read CPU stats"))?;
            
            // Parse CPU stats: cpu user nice system idle iowait irq softirq steal guest guest_nice
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 5 || parts[0] != "cpu" {
                return Err(anyhow::anyhow!("Invalid CPU stat format"));
            }
            
            let user: u64 = parts[1].parse()?;
            let nice: u64 = parts[2].parse()?;
            let system: u64 = parts[3].parse()?;
            let idle: u64 = parts[4].parse()?;
            let iowait: u64 = parts.get(5).and_then(|s| s.parse().ok()).unwrap_or(0);
            let irq: u64 = parts.get(6).and_then(|s| s.parse().ok()).unwrap_or(0);
            let softirq: u64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0);
            let steal: u64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0);
            
            let total = user + nice + system + idle + iowait + irq + softirq + steal;
            let busy = total - idle - iowait;
            
            if total > 0 {
                Ok((busy as f64 / total as f64) * 100.0)
            } else {
                Ok(0.0)
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            // Fallback for other systems
            Ok(0.0)
        }
    }

    fn get_memory_usage(&self) -> anyhow::Result<(f64, u64, u64)> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("vm_stat").output()?;
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            let mut page_size = 4096u64; // Default page size
            let mut pages_free = 0u64;
            let mut pages_active = 0u64;
            let mut pages_inactive = 0u64;
            let mut pages_speculative = 0u64;
            let mut pages_wired = 0u64;
            let mut pages_compressed = 0u64;

            for line in output_str.lines() {
                if line.starts_with("page size of ") {
                    if let Some(size_str) = line.split_whitespace().nth(3) {
                        page_size = size_str.parse().unwrap_or(4096);
                    }
                } else if line.contains("Pages free:") {
                    pages_free = Self::extract_pages(line);
                } else if line.contains("Pages active:") {
                    pages_active = Self::extract_pages(line);
                } else if line.contains("Pages inactive:") {
                    pages_inactive = Self::extract_pages(line);
                } else if line.contains("Pages speculative:") {
                    pages_speculative = Self::extract_pages(line);
                } else if line.contains("Pages wired down:") {
                    pages_wired = Self::extract_pages(line);
                } else if line.contains("Pages stored in compressor:") {
                    pages_compressed = Self::extract_pages(line);
                }
            }

            let total_pages = pages_free + pages_active + pages_inactive + 
                            pages_speculative + pages_wired + pages_compressed;
            let used_pages = total_pages - pages_free - pages_speculative;
            
            let total_memory = total_pages * page_size;
            let used_memory = used_pages * page_size;
            let usage_percent = if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };

            Ok((usage_percent, total_memory, used_memory))
        }

        #[cfg(target_os = "linux")]
        {
            // Read from /proc/meminfo
            let meminfo = fs::read_to_string("/proc/meminfo")?;
            let mut total_memory = 0u64;
            let mut free_memory = 0u64;
            let mut buffers = 0u64;
            let mut cached = 0u64;
            let mut s_reclaimable = 0u64;
            
            for line in meminfo.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    match parts[0] {
                        "MemTotal:" => total_memory = parts[1].parse::<u64>()? * 1024,
                        "MemFree:" => free_memory = parts[1].parse::<u64>()? * 1024,
                        "Buffers:" => buffers = parts[1].parse::<u64>()? * 1024,
                        "Cached:" => cached = parts[1].parse::<u64>()? * 1024,
                        "SReclaimable:" => s_reclaimable = parts[1].parse::<u64>()? * 1024,
                        _ => {}
                    }
                }
            }
            
            // Calculate used memory (total - free - buffers - cached - reclaimable)
            let available_memory = free_memory + buffers + cached + s_reclaimable;
            let used_memory = total_memory.saturating_sub(available_memory);
            let usage_percent = if total_memory > 0 {
                (used_memory as f64 / total_memory as f64) * 100.0
            } else {
                0.0
            };
            
            Ok((usage_percent, total_memory, used_memory))
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok((0.0, 0, 0))
        }
    }

    fn extract_pages(line: &str) -> u64 {
        line.split_whitespace()
            .nth(2)
            .and_then(|s| s.trim_end_matches('.').parse().ok())
            .unwrap_or(0)
    }

    fn get_disk_io_metrics(&mut self) -> anyhow::Result<(u64, u64, f64, f64)> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("iostat")
                .args(&["-d", "-c", "1"])
                .output()?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut total_read_bytes = 0u64;
            let mut total_write_bytes = 0u64;

            // Parse iostat output to get disk I/O statistics
            let lines: Vec<&str> = output_str.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                if line.contains("disk") && i + 1 < lines.len() {
                    let data_line = lines[i + 1];
                    let parts: Vec<&str> = data_line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        // iostat typically shows KB read/write per second
                        if let (Ok(read_kb), Ok(write_kb)) = (
                            parts[1].parse::<f64>(),
                            parts[2].parse::<f64>()
                        ) {
                            total_read_bytes = (read_kb * 1024.0) as u64;
                            total_write_bytes = (write_kb * 1024.0) as u64;
                        }
                    }
                    break;
                }
            }

            // Calculate rates if we have previous data
            let (read_rate, write_rate) = if let (Some(prev), Some(prev_time)) = 
                (&self.previous_metrics, self.previous_timestamp) {
                let time_diff = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - prev_time;
                
                if time_diff > 0 {
                    let read_diff = total_read_bytes.saturating_sub(prev.disk_read_bytes);
                    let write_diff = total_write_bytes.saturating_sub(prev.disk_write_bytes);
                    
                    (
                        read_diff as f64 / time_diff as f64,
                        write_diff as f64 / time_diff as f64
                    )
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };

            Ok((total_read_bytes, total_write_bytes, read_rate, write_rate))
        }

        #[cfg(target_os = "linux")]
        {
            // Read from /proc/diskstats or /sys/block/*/stat
            let mut total_read_bytes = 0u64;
            let mut total_write_bytes = 0u64;
            
            // Read diskstats for all block devices
            let diskstats = fs::read_to_string("/proc/diskstats")?;
            
            for line in diskstats.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 14 {
                    // Only count physical disks (major device number 8 for SCSI/SATA, 259 for NVMe)
                    let major: u32 = parts[0].parse().unwrap_or(0);
                    let device_name = parts[2];
                    
                    // Filter out partitions and virtual devices
                    if (major == 8 || major == 259) && !device_name.chars().any(|c| c.is_digit(10)) {
                        // Column 5: sectors read, Column 9: sectors written
                        // Sectors are typically 512 bytes
                        if let (Ok(sectors_read), Ok(sectors_written)) = (
                            parts[5].parse::<u64>(),
                            parts[9].parse::<u64>()
                        ) {
                            total_read_bytes += sectors_read * 512;
                            total_write_bytes += sectors_written * 512;
                        }
                    }
                }
            }
            
            // Calculate rates
            let (read_rate, write_rate) = if let (Some(prev), Some(prev_time)) = 
                (&self.previous_metrics, self.previous_timestamp) {
                let time_diff = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - prev_time;
                
                if time_diff > 0 {
                    let read_diff = total_read_bytes.saturating_sub(prev.disk_read_bytes);
                    let write_diff = total_write_bytes.saturating_sub(prev.disk_write_bytes);
                    
                    (
                        read_diff as f64 / time_diff as f64,
                        write_diff as f64 / time_diff as f64
                    )
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };
            
            Ok((total_read_bytes, total_write_bytes, read_rate, write_rate))
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok((0, 0, 0.0, 0.0))
        }
    }

    fn get_network_metrics(&mut self) -> anyhow::Result<(u64, u64, f64, f64)> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("netstat")
                .args(&["-ib"])
                .output()?;
            
            let output_str = String::from_utf8_lossy(&output.stdout);
            let mut total_rx_bytes = 0u64;
            let mut total_tx_bytes = 0u64;

            for line in output_str.lines() {
                if line.contains("en") && !line.contains("Name") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 10 {
                        // netstat -ib format: Name Mtu Network Address Ipkts Ierrs Ibytes Opkts Oerrs Obytes Coll
                        if let (Ok(rx_bytes), Ok(tx_bytes)) = (
                            parts[6].parse::<u64>(),
                            parts[9].parse::<u64>()
                        ) {
                            total_rx_bytes += rx_bytes;
                            total_tx_bytes += tx_bytes;
                        }
                    }
                }
            }

            // Calculate rates if we have previous data
            let (rx_rate, tx_rate) = if let (Some(prev), Some(prev_time)) = 
                (&self.previous_metrics, self.previous_timestamp) {
                let time_diff = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - prev_time;
                
                if time_diff > 0 {
                    let rx_diff = total_rx_bytes.saturating_sub(prev.network_rx_bytes);
                    let tx_diff = total_tx_bytes.saturating_sub(prev.network_tx_bytes);
                    
                    (
                        rx_diff as f64 / time_diff as f64,
                        tx_diff as f64 / time_diff as f64
                    )
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };

            Ok((total_rx_bytes, total_tx_bytes, rx_rate, tx_rate))
        }

        #[cfg(target_os = "linux")]
        {
            let mut total_rx_bytes = 0u64;
            let mut total_tx_bytes = 0u64;
            
            // Read from /proc/net/dev
            let net_dev = fs::read_to_string("/proc/net/dev")?;
            
            for line in net_dev.lines().skip(2) { // Skip header lines
                if let Some(colon_pos) = line.find(':') {
                    let interface = line[..colon_pos].trim();
                    let stats = &line[colon_pos + 1..];
                    let parts: Vec<&str> = stats.split_whitespace().collect();
                    
                    // Skip loopback and virtual interfaces
                    if interface != "lo" && !interface.starts_with("vir") && !interface.starts_with("docker") {
                        if parts.len() >= 16 {
                            // Column 0: rx_bytes, Column 8: tx_bytes
                            if let (Ok(rx_bytes), Ok(tx_bytes)) = (
                                parts[0].parse::<u64>(),
                                parts[8].parse::<u64>()
                            ) {
                                total_rx_bytes += rx_bytes;
                                total_tx_bytes += tx_bytes;
                            }
                        }
                    }
                }
            }
            
            // Calculate rates
            let (rx_rate, tx_rate) = if let (Some(prev), Some(prev_time)) = 
                (&self.previous_metrics, self.previous_timestamp) {
                let time_diff = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() - prev_time;
                
                if time_diff > 0 {
                    let rx_diff = total_rx_bytes.saturating_sub(prev.network_rx_bytes);
                    let tx_diff = total_tx_bytes.saturating_sub(prev.network_tx_bytes);
                    
                    (
                        rx_diff as f64 / time_diff as f64,
                        tx_diff as f64 / time_diff as f64
                    )
                } else {
                    (0.0, 0.0)
                }
            } else {
                (0.0, 0.0)
            };
            
            Ok((total_rx_bytes, total_tx_bytes, rx_rate, tx_rate))
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Ok((0, 0, 0.0, 0.0))
        }
    }

    fn get_load_average(&self) -> anyhow::Result<[f64; 3]> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("uptime").output()?;
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse load averages from uptime output
            if let Some(load_start) = output_str.find("load averages:") {
                let load_part = &output_str[load_start + 14..];
                let loads: Vec<&str> = load_part.trim().split_whitespace().collect();
                
                if loads.len() >= 3 {
                    let load1 = loads[0].parse().unwrap_or(0.0);
                    let load5 = loads[1].parse().unwrap_or(0.0);
                    let load15 = loads[2].parse().unwrap_or(0.0);
                    return Ok([load1, load5, load15]);
                }
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/loadavg
            let loadavg = fs::read_to_string("/proc/loadavg")?;
            let parts: Vec<&str> = loadavg.split_whitespace().collect();
            
            if parts.len() >= 3 {
                let load1 = parts[0].parse().unwrap_or(0.0);
                let load5 = parts[1].parse().unwrap_or(0.0);
                let load15 = parts[2].parse().unwrap_or(0.0);
                return Ok([load1, load5, load15]);
            }
        }
        
        Ok([0.0, 0.0, 0.0])
    }

    fn get_process_count(&self) -> anyhow::Result<u32> {
        let output = Command::new("ps")
            .args(&["-ax"])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let count = output_str.lines().count().saturating_sub(1) as u32; // Subtract header
        Ok(count)
    }

    fn get_uptime(&self) -> anyhow::Result<u64> {
        #[cfg(target_os = "macos")]
        {
            let output = Command::new("uptime").output()?;
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // Parse uptime - this is a simplified parser
            if let Some(up_pos) = output_str.find("up ") {
                let uptime_part = &output_str[up_pos + 3..];
                
                // Look for patterns like "1 day", "2:30", etc.
                let mut total_seconds = 0u64;
                
                if uptime_part.contains("day") {
                    if let Some(day_match) = uptime_part.split_whitespace().next() {
                        if let Ok(days) = day_match.parse::<u64>() {
                            total_seconds += days * 24 * 3600;
                        }
                    }
                }
                
                // This is a simplified implementation
                // In practice, you'd want more robust uptime parsing
                return Ok(total_seconds);
            }
        }
        
        #[cfg(target_os = "linux")]
        {
            // Read from /proc/uptime - first value is uptime in seconds
            let uptime_content = fs::read_to_string("/proc/uptime")?;
            let parts: Vec<&str> = uptime_content.split_whitespace().collect();
            
            if let Some(first) = parts.first() {
                if let Ok(uptime_float) = first.parse::<f64>() {
                    return Ok(uptime_float as u64);
                }
            }
        }
        
        Ok(0)
    }

    pub fn get_top_processes(&self, limit: usize) -> anyhow::Result<Vec<ProcessMetrics>> {
        let output = Command::new("ps")
            .args(&["-axo", "pid,pcpu,pmem,comm", "-r"])
            .output()?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        let mut processes = Vec::new();
        
        for (i, line) in output_str.lines().enumerate() {
            if i == 0 || processes.len() >= limit {
                continue; // Skip header or if we have enough
            }
            
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let (Ok(pid), Ok(cpu), Ok(mem)) = (
                    parts[0].parse::<u32>(),
                    parts[1].parse::<f64>(),
                    parts[2].parse::<f64>()
                ) {
                    processes.push(ProcessMetrics {
                        pid,
                        name: parts[3].to_string(),
                        cpu_percent: cpu,
                        memory_bytes: 0, // Would need additional calculation
                        memory_percent: mem,
                        command: parts[3..].join(" "),
                    });
                }
            }
        }
        
        Ok(processes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut collector = SystemMetricsCollector::new();
        let metrics = collector.collect_metrics();
        assert!(metrics.is_ok());
    }

    #[test]
    fn test_process_metrics() {
        let collector = SystemMetricsCollector::new();
        let processes = collector.get_top_processes(5);
        assert!(processes.is_ok());
    }
}