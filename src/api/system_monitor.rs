use crate::api::models::{
    SystemResources, CpuInfo, MemoryInfo, DiskInfo, NetworkInfo, 
    NetworkInterface, ProcessInfo, SystemMetrics
};
use sysinfo::{System, Disks, Networks};
use chrono::{DateTime, Utc};
use std::time::SystemTime;

pub struct SystemMonitor {
    system: System,
    disks: Disks,
    networks: Networks,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();
        Self { system, disks, networks }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
        self.disks.refresh();
        self.networks.refresh();
    }

    pub fn get_system_metrics(&mut self) -> SystemMetrics {
        self.refresh();
        
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        let memory_used = self.system.used_memory();
        let memory_total = self.system.total_memory();
        let memory_usage = if memory_total > 0 {
            (memory_used as f32 / memory_total as f32) * 100.0
        } else {
            0.0
        };

        // Get real disk usage for root partition
        let disk_usage = self.disks
            .iter()
            .find(|disk| {
                disk.mount_point().to_str().map_or(false, |mount| mount == "/" || mount.starts_with("/"))
            })
            .map(|disk| {
                let total = disk.total_space();
                let available = disk.available_space();
                let used = total - available;
                if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        let load_average = System::load_average();
        let uptime = System::uptime();

        SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            load_average: vec![load_average.one, load_average.five, load_average.fifteen],
            uptime,
        }
    }

    pub fn get_system_resources(&mut self) -> SystemResources {
        self.refresh();

        let cpu_info = self.get_cpu_info();
        let memory_info = self.get_memory_info();
        let disk_info = self.get_disk_info();
        let network_info = self.get_network_info();
        let processes = self.get_processes();

        SystemResources {
            cpu: cpu_info,
            memory: memory_info,
            disk: disk_info,
            network: network_info,
            processes,
        }
    }

    fn get_cpu_info(&self) -> CpuInfo {
        let cpu_usage = self.system.global_cpu_info().cpu_usage();
        let cores = self.system.cpus().len() as u32;
        let load_average = System::load_average();
        let frequency = self.system.global_cpu_info().frequency();

        CpuInfo {
            usage: cpu_usage,
            cores,
            load_average: vec![load_average.one, load_average.five, load_average.fifteen],
            frequency,
        }
    }

    fn get_memory_info(&self) -> MemoryInfo {
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        let available = self.system.available_memory();
        let percent = if total > 0 {
            (used as f32 / total as f32) * 100.0
        } else {
            0.0
        };
        let swap_total = self.system.total_swap();
        let swap_used = self.system.used_swap();

        MemoryInfo {
            total,
            used,
            available,
            percent,
            swap_total,
            swap_used,
        }
    }

    fn get_disk_info(&self) -> DiskInfo {
        // Get real disk info for root partition
        if let Some(disk) = self.disks.iter().find(|d| {
            d.mount_point().to_str().map_or(false, |mount| mount == "/" || mount.starts_with("/"))
        }) {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let percent = if total > 0 {
                (used as f32 / total as f32) * 100.0
            } else {
                0.0
            };

            DiskInfo {
                total,
                used,
                available,
                percent,
                mount_point: disk.mount_point().to_string_lossy().to_string(),
            }
        } else {
            DiskInfo {
                total: 0,
                used: 0,
                available: 0,
                percent: 0.0,
                mount_point: "/".to_string(),
            }
        }
    }

    fn get_network_info(&self) -> NetworkInfo {
        // Get real network info
        let mut total_bytes_in = 0;
        let mut total_bytes_out = 0;
        let mut total_packets_in = 0;
        let mut total_packets_out = 0;
        let mut interfaces = Vec::new();

        for (interface_name, network) in &self.networks {
            let bytes_received = network.total_received();
            let bytes_sent = network.total_transmitted();
            let packets_received = network.total_packets_received();
            let packets_sent = network.total_packets_transmitted();

            total_bytes_in += bytes_received;
            total_bytes_out += bytes_sent;
            total_packets_in += packets_received;
            total_packets_out += packets_sent;

            interfaces.push(NetworkInterface {
                name: interface_name.clone(),
                bytes_received,
                bytes_sent,
                packets_received,
                packets_sent,
                is_up: bytes_received > 0 || bytes_sent > 0,
            });
        }

        NetworkInfo {
            bytes_in: total_bytes_in,
            bytes_out: total_bytes_out,
            packets_in: total_packets_in,
            packets_out: total_packets_out,
            interfaces,
        }
    }

    fn get_processes(&self) -> Vec<ProcessInfo> {
        let mut processes: Vec<ProcessInfo> = self.system.processes()
            .iter()
            .map(|(pid, process)| {
                let start_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time());
                let start_time: DateTime<Utc> = start_time.into();

                ProcessInfo {
                    pid: pid.as_u32(),
                    ppid: process.parent().map_or(0, |p| p.as_u32()),
                    name: process.name().to_string(),
                    command: process.cmd().join(" "),
                    user: process.user_id().map(|uid| uid.to_string()).unwrap_or_else(|| "unknown".to_string()),
                    cpu_percent: process.cpu_usage(),
                    memory_percent: (process.memory() as f32 / (1024.0 * 1024.0 * 1024.0)) * 100.0, // Convert to percentage
                    memory_mb: process.memory() as f32 / 1024.0 / 1024.0,
                    status: format!("{:?}", process.status()),
                    start_time: start_time.to_rfc3339(),
                    runtime: (chrono::Utc::now().timestamp() as u64).saturating_sub(process.start_time()) as f32,
                    threads: 1, // Default
                    priority: 0, // Default
                    nice: 0, // Default
                    executable: process.exe().map_or(process.name().to_string(), |p| p.to_string_lossy().to_string()),
                    working_dir: process.cwd().map_or("/".to_string(), |p| p.to_string_lossy().to_string()),
                    open_files: 0, // Default
                    network_connections: 0, // Default
                    children: 0, // Default
                    risk_score: if process.cpu_usage() > 80.0 { 50.0 } else { 10.0 },
                    is_system: process.name().starts_with("kernel") || process.name().starts_with("systemd"),
                    is_suspicious: process.cpu_usage() > 90.0,
                }
            })
            .collect();

        // Sort by CPU usage (descending)
        processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to top 50 processes
        processes.truncate(50);
        processes
    }

    pub fn get_process_by_pid(&mut self, pid: u32) -> Option<ProcessInfo> {
        self.refresh();
        
        if let Some(process) = self.system.process(sysinfo::Pid::from_u32(pid)) {
            let start_time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time());
            let start_time: DateTime<Utc> = start_time.into();

            Some(ProcessInfo {
                pid,
                ppid: process.parent().map_or(0, |p| p.as_u32()),
                name: process.name().to_string(),
                command: process.cmd().join(" "),
                user: process.user_id().map(|uid| uid.to_string()).unwrap_or_else(|| "unknown".to_string()),
                cpu_percent: process.cpu_usage(),
                memory_percent: (process.memory() as f32 / (1024.0 * 1024.0 * 1024.0)) * 100.0,
                memory_mb: process.memory() as f32 / 1024.0 / 1024.0,
                status: format!("{:?}", process.status()),
                start_time: start_time.to_rfc3339(),
                runtime: (chrono::Utc::now().timestamp() as u64).saturating_sub(process.start_time()) as f32,
                threads: 1,
                priority: 0,
                nice: 0,
                executable: process.exe().map_or(process.name().to_string(), |p| p.to_string_lossy().to_string()),
                working_dir: process.cwd().map_or("/".to_string(), |p| p.to_string_lossy().to_string()),
                open_files: 0,
                network_connections: 0,
                children: 0,
                risk_score: if process.cpu_usage() > 80.0 { 50.0 } else { 10.0 },
                is_system: process.name().starts_with("kernel") || process.name().starts_with("systemd"),
                is_suspicious: process.cpu_usage() > 90.0,
            })
        } else {
            None
        }
    }
}