use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub exe_path: Option<PathBuf>,
    pub cmdline: Vec<String>,
    pub uid: u32,
    pub gid: u32,
    pub start_time: u64,
}

pub struct ProcessMonitor {
    processes: HashMap<u32, ProcessInfo>,
    running: bool,
}

impl ProcessMonitor {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            running: false,
        }
    }
    
    pub fn start_monitoring(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        self.running = true;
        info!("Process monitoring started");
        
        // Initial scan of all processes
        self.scan_all_processes()?;
        
        Ok(())
    }
    
    pub fn scan_all_processes(&mut self) -> Result<()> {
        let proc_dir = fs::read_dir("/proc")?;
        
        for entry in proc_dir {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            // Check if it's a PID directory
            if let Ok(pid) = file_name_str.parse::<u32>() {
                match self.get_process_info(pid) {
                    Ok(info) => {
                        self.processes.insert(pid, info);
                    }
                    Err(e) => {
                        // Process might have exited, this is normal
                        debug!("Failed to get info for PID {}: {}", pid, e);
                    }
                }
            }
        }
        
        info!("Scanned {} processes", self.processes.len());
        Ok(())
    }
    
    pub fn get_process_info(&self, pid: u32) -> Result<ProcessInfo> {
        let proc_path = format!("/proc/{}", pid);
        
        // Read /proc/[pid]/stat for basic info
        let stat_content = fs::read_to_string(format!("{}/stat", proc_path))?;
        let stat_parts = self.parse_stat(&stat_content)?;
        
        // Read /proc/[pid]/status for additional info
        let status_content = fs::read_to_string(format!("{}/status", proc_path))?;
        let status_info = self.parse_status(&status_content)?;
        
        // Read /proc/[pid]/cmdline
        let cmdline = self.read_cmdline(pid)?;
        
        // Read /proc/[pid]/exe symlink
        let exe_path = fs::read_link(format!("{}/exe", proc_path)).ok();
        
        Ok(ProcessInfo {
            pid,
            ppid: stat_parts.ppid,
            name: status_info.name.unwrap_or_else(|| stat_parts.comm.clone()),
            exe_path,
            cmdline,
            uid: status_info.uid,
            gid: status_info.gid,
            start_time: stat_parts.start_time,
        })
    }
    
    fn parse_stat(&self, content: &str) -> Result<StatInfo> {
        // Format: pid (comm) state ppid pgrp session tty_nr tpgid flags minflt cminflt majflt cmajflt utime stime cutime cstime priority nice num_threads itrealvalue starttime ...
        
        // Find the last ) to handle process names with parentheses
        let comm_end = content.rfind(')').ok_or_else(|| anyhow!("Invalid stat format"))?;
        let comm_start = content.find('(').ok_or_else(|| anyhow!("Invalid stat format"))?;
        
        let pid_str = &content[..comm_start].trim();
        let comm = &content[comm_start + 1..comm_end];
        let rest = &content[comm_end + 1..].trim();
        
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() < 20 {
            return Err(anyhow!("Insufficient stat fields"));
        }
        
        Ok(StatInfo {
            pid: pid_str.parse()?,
            comm: comm.to_string(),
            ppid: parts[1].parse()?,
            start_time: parts[19].parse()?,
        })
    }
    
    fn parse_status(&self, content: &str) -> Result<StatusInfo> {
        let mut info = StatusInfo::default();
        
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() < 2 {
                continue;
            }
            
            let key = parts[0].trim();
            let value = parts[1].trim();
            
            match key {
                "Name" => info.name = Some(value.to_string()),
                "PPid" => info.ppid = value.parse().ok(),
                "Uid" => {
                    let uid_parts: Vec<&str> = value.split_whitespace().collect();
                    if !uid_parts.is_empty() {
                        info.uid = uid_parts[0].parse().unwrap_or(0);
                    }
                }
                "Gid" => {
                    let gid_parts: Vec<&str> = value.split_whitespace().collect();
                    if !gid_parts.is_empty() {
                        info.gid = gid_parts[0].parse().unwrap_or(0);
                    }
                }
                _ => {}
            }
        }
        
        Ok(info)
    }
    
    fn read_cmdline(&self, pid: u32) -> Result<Vec<String>> {
        let cmdline_path = format!("/proc/{}/cmdline", pid);
        let content = fs::read(&cmdline_path)?;
        
        // cmdline is null-separated
        let args: Vec<String> = content
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).to_string())
            .collect();
        
        Ok(args)
    }
    
    pub fn find_process_by_inode(&self, inode: u32) -> Option<&ProcessInfo> {
        // Search through all processes to find which one has a socket with this inode
        for (pid, process) in &self.processes {
            let fd_path = format!("/proc/{}/fd", pid);
            if let Ok(entries) = fs::read_dir(&fd_path) {
                for entry in entries.flatten() {
                    if let Ok(link) = fs::read_link(entry.path()) {
                        let link_str = link.to_string_lossy();
                        if link_str.contains("socket:") && link_str.contains(&inode.to_string()) {
                            return Some(process);
                        }
                    }
                }
            }
        }
        None
    }
    
    pub fn get_process_by_pid(&self, pid: u32) -> Option<&ProcessInfo> {
        self.processes.get(&pid)
    }
    
    pub fn refresh_process(&mut self, pid: u32) -> Result<()> {
        match self.get_process_info(pid) {
            Ok(info) => {
                self.processes.insert(pid, info);
                Ok(())
            }
            Err(e) => {
                // Process might have exited
                self.processes.remove(&pid);
                Err(e)
            }
        }
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if self.running {
            info!("Stopping process monitoring");
            self.running = false;
        }
        Ok(())
    }
    
    pub fn refresh_processes(&mut self) -> Result<()> {
        // Re-scan all processes to update our tracking
        let proc_dir = fs::read_dir("/proc")?;
        let mut new_processes = HashMap::new();
        
        for entry in proc_dir {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            
            if let Ok(pid) = file_name_str.parse::<u32>() {
                match self.get_process_info(pid) {
                    Ok(info) => {
                        new_processes.insert(pid, info);
                    }
                    Err(_) => {
                        // Process might have exited
                    }
                }
            }
        }
        
        self.processes = new_processes;
        Ok(())
    }
    
    pub fn get_process_children(&self, parent_pid: u32) -> Vec<&ProcessInfo> {
        self.processes.values()
            .filter(|p| p.ppid == parent_pid)
            .collect()
    }
    
    pub fn get_process_tree(&self, root_pid: u32) -> Vec<&ProcessInfo> {
        let mut tree = Vec::new();
        let mut to_visit = vec![root_pid];
        let mut visited = std::collections::HashSet::new();
        
        while let Some(pid) = to_visit.pop() {
            if visited.contains(&pid) {
                continue;
            }
            visited.insert(pid);
            
            if let Some(process) = self.processes.get(&pid) {
                tree.push(process);
                
                // Add children to visit
                for child in self.get_process_children(pid) {
                    to_visit.push(child.pid);
                }
            }
        }
        
        tree
    }
    
    pub fn find_process_by_name(&self, name: &str) -> Vec<&ProcessInfo> {
        self.processes.values()
            .filter(|p| p.name.contains(name))
            .collect()
    }
    
    pub fn find_process_by_cmdline(&self, pattern: &str) -> Vec<&ProcessInfo> {
        self.processes.values()
            .filter(|p| p.cmdline.join(" ").contains(pattern))
            .collect()
    }
}

#[derive(Debug)]
struct StatInfo {
    pid: u32,
    comm: String,
    ppid: u32,
    start_time: u64,
}

#[derive(Debug, Default)]
struct StatusInfo {
    name: Option<String>,
    ppid: Option<u32>,
    uid: u32,
    gid: u32,
}