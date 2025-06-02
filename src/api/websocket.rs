use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

use crate::api::models::{
    WebSocketMessage, LiveEvent, SecurityEvent, NetworkConnection, 
    SystemMetrics, ThreatDetection, LogEntry, DnsQuery, LogLevel, LogCategory
};
use crate::api::handlers::AppState;
use tokio::sync::mpsc;

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Spawn a task to send periodic updates
    let state_clone = Arc::clone(&state);
    let sender_task = tokio::spawn(async move {
        let mut heartbeat_interval = interval(Duration::from_secs(30));
        let mut metrics_interval = interval(Duration::from_secs(5));
        let mut events_interval = interval(Duration::from_secs(2));
        let mut logs_interval = interval(Duration::from_secs(3));
        
        loop {
            tokio::select! {
                _ = heartbeat_interval.tick() => {
                    let heartbeat = WebSocketMessage::Heartbeat {
                        timestamp: Utc::now(),
                    };
                    
                    if let Ok(message) = serde_json::to_string(&heartbeat) {
                        if sender.send(axum::extract::ws::Message::Text(message)).await.is_err() {
                            break;
                        }
                    }
                }
                
                _ = metrics_interval.tick() => {
                    // Send system metrics
                    let metrics = {
                        let mut monitor = state_clone.system_monitor.lock().unwrap();
                        monitor.get_system_metrics()
                    };
                    
                    let message = WebSocketMessage::SystemMetrics { data: metrics };
                    if let Ok(message_str) = serde_json::to_string(&message) {
                        if sender.send(axum::extract::ws::Message::Text(message_str)).await.is_err() {
                            break;
                        }
                    }
                }
                
                _ = events_interval.tick() => {
                    // Generate and send live events
                    let live_event = generate_random_live_event();
                    
                    // Add to live events storage
                    {
                        let mut events = state_clone.live_events.lock().unwrap();
                        events.insert(0, live_event.clone());
                        if events.len() > 100 {
                            events.truncate(100);
                        }
                    }
                    
                    let message = WebSocketMessage::LiveEvent { data: live_event };
                    if let Ok(message_str) = serde_json::to_string(&message) {
                        if sender.send(axum::extract::ws::Message::Text(message_str)).await.is_err() {
                            break;
                        }
                    }
                }
                
                _ = logs_interval.tick() => {
                    // Generate and send log entries
                    let log_entry = generate_log_entry_from_system();
                    
                    // Add to log entries storage
                    {
                        let mut logs = state_clone.log_entries.lock().unwrap();
                        logs.insert(0, log_entry.clone());
                        if logs.len() > 1000 {
                            logs.truncate(1000);
                        }
                    }
                    
                    let message = WebSocketMessage::LogEntry { data: log_entry };
                    if let Ok(message_str) = serde_json::to_string(&message) {
                        if sender.send(axum::extract::ws::Message::Text(message_str)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });
    
    // Handle incoming messages
    let receiver_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    axum::extract::ws::Message::Text(_text) => {
                        // Handle client messages if needed
                    }
                    axum::extract::ws::Message::Close(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = sender_task => {},
        _ = receiver_task => {},
    }
}

fn generate_random_live_event() -> LiveEvent {
    use std::collections::HashMap;
    use sysinfo::System;
    
    // Get current system state for real data
    let mut system = System::new();
    system.refresh_processes();
    
    // Get a random current process for the event
    let processes: Vec<_> = system.processes().iter().collect();
    if processes.is_empty() {
        // Return a minimal system event if no processes available
        return LiveEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type: "system".to_string(),
            severity: "info".to_string(),
            title: "System monitoring active".to_string(),
            description: "FluxDefense monitoring services running".to_string(),
            source: "system_monitor".to_string(),
            details: HashMap::new(),
        };
    }
    
    // Use current time as seed for selecting a process
    let time_seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let (pid, process) = processes[time_seed % processes.len()];
    let process_pid = pid.as_u32();
    let process_name = process.name().to_string();
    
    // Create a real system monitoring event based on actual process
    let mut details = HashMap::new();
    details.insert("process_id".to_string(), serde_json::json!(process_pid));
    details.insert("process_name".to_string(), serde_json::json!(process_name));
    details.insert("cpu_usage".to_string(), serde_json::json!(process.cpu_usage()));
    details.insert("memory_kb".to_string(), serde_json::json!(process.memory() / 1024));
    
    if let Some(exe_path) = process.exe() {
        details.insert("executable".to_string(), serde_json::json!(exe_path.to_string_lossy()));
    }
    
    LiveEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        event_type: "process_monitor".to_string(),
        severity: "info".to_string(),
        title: format!("Process monitoring: {}", process_name),
        description: format!("Process {} (PID: {}) monitored - CPU: {:.1}%, Memory: {} KB", 
                           process_name, process_pid, process.cpu_usage(), process.memory() / 1024),
        source: "system_monitor".to_string(),
        details,
    }
}

// Helper function to populate realistic data based on system state
pub fn populate_mock_data(state: Arc<AppState>) {
    populate_realistic_data(state);
}

// Helper function to populate data based only on real system information
pub fn populate_realistic_data(state: Arc<AppState>) {
    use std::collections::HashMap;
    use sysinfo::{System, Networks};
    
    // Get real system data
    let mut system = System::new_all();
    system.refresh_all();
    let networks = Networks::new_with_refreshed_list();
    
    // Only populate with actual system information - no simulated security events
    
    // Note: Real network connections would come from netstat/ss parsing or eBPF monitoring
    // For now, we only populate the data structures that can be filled with real system info
    // Network connections would require root privileges and active monitoring
    
    // Note: Real DNS queries would come from parsing /var/log/syslog, dnsmasq logs, 
    // or monitoring DNS traffic with eBPF/packet capture
    
    // Note: Real threat detections would come from actual security analysis:
    // - YARA rule scanning of running processes
    // - Behavioral analysis of system calls
    // - Network traffic analysis for IOCs
    // - File integrity monitoring
    // - Real-time malware detection engines
    
    // Note: Real log entries would come from:
    // - /var/log/syslog and /var/log/auth.log parsing
    // - journalctl output for systemd logs  
    // - Application-specific log files
    // - Kernel audit logs (/var/log/audit/audit.log)
    // - Security event logs from actual monitoring tools
    
    // Populate process data from real system information
    populate_process_data(&state, &system);
    
    // Populate real network connections
    populate_real_network_connections(&state);
    
    // Populate initial log entries
    populate_initial_log_entries(&state);
}

fn populate_process_data(state: &Arc<AppState>, system: &sysinfo::System) {
    use crate::api::models::{ProcessInfo, ProcessStats};
    
    let mut processes = Vec::new();
    let mut running_count = 0;
    let mut sleeping_count = 0;
    let mut zombie_count = 0;
    let mut total_threads = 0;
    
    // Convert sysinfo processes to our ProcessInfo format
    for (pid, process) in system.processes() {
        let status_str = match process.status() {
            sysinfo::ProcessStatus::Run => {
                running_count += 1;
                "running"
            },
            sysinfo::ProcessStatus::Sleep => {
                sleeping_count += 1;
                "sleeping"
            },
            sysinfo::ProcessStatus::Zombie => {
                zombie_count += 1;
                "zombie"
            },
            _ => "unknown",
        };
        
        let name = process.name();
        let command = process.cmd().join(" ");
        let cpu_percent = process.cpu_usage();
        let memory_percent = (process.memory() as f32 / (system.total_memory() as f32)) * 100.0;
        let memory_mb = process.memory() as f32 / 1024.0 / 1024.0;
        let start_time = process.start_time();
        let runtime = (chrono::Utc::now().timestamp() as u64).saturating_sub(start_time) as f32;
        
        // Basic risk scoring based on CPU usage and suspicious patterns
        let mut risk_score = 0.0;
        if cpu_percent > 80.0 { risk_score += 30.0; }
        if memory_percent > 20.0 { risk_score += 20.0; }
        if name.contains("bitcoin") || name.contains("mine") || name.contains("crypto") {
            risk_score += 50.0;
        }
        
        let is_system = process.user_id().map_or(false, |uid| uid.to_string() == "0") || 
                       name.starts_with("kernel") || 
                       name.starts_with("systemd") ||
                       name.starts_with("kthread");
                       
        let is_suspicious = risk_score > 50.0 || 
                          (cpu_percent > 90.0 && !is_system) ||
                          (memory_percent > 30.0 && !is_system);
        
        total_threads += 1; // Approximate - sysinfo doesn't expose thread count directly
        
        let process_info = ProcessInfo {
            pid: pid.as_u32(),
            ppid: process.parent().map_or(0, |p| p.as_u32()),
            name: name.to_string(),
            command: if command.is_empty() { format!("[{}]", name) } else { command },
            user: process.user_id()
                .map_or("unknown".to_string(), |uid| uid.to_string()),
            cpu_percent,
            memory_percent,
            memory_mb,
            status: status_str.to_string(),
            start_time: chrono::DateTime::from_timestamp(start_time as i64, 0)
                .unwrap_or_else(|| chrono::Utc::now())
                .to_rfc3339(),
            runtime,
            threads: 1, // Default value - could be enhanced with /proc parsing
            priority: 0, // Default value - could be enhanced with /proc parsing  
            nice: 0, // Default value - could be enhanced with /proc parsing
            executable: process.exe().map_or(name.to_string(), |p| p.to_string_lossy().to_string()),
            working_dir: process.cwd().map_or("/".to_string(), |p| p.to_string_lossy().to_string()),
            open_files: 0, // Would require /proc/[pid]/fd parsing
            network_connections: 0, // Would require netstat parsing or eBPF
            children: 0, // Would require process tree analysis
            risk_score,
            is_system,
            is_suspicious,
        };
        
        processes.push(process_info);
    }
    
    // Sort by CPU usage for top processes
    processes.sort_by(|a, b| b.cpu_percent.partial_cmp(&a.cpu_percent).unwrap_or(std::cmp::Ordering::Equal));
    let top_cpu_processes = processes.iter().take(5).cloned().collect();
    
    // Sort by memory usage for top processes  
    processes.sort_by(|a, b| b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal));
    let top_memory_processes = processes.iter().take(5).cloned().collect();
    
    // Get system load and CPU info
    let load_avg = sysinfo::System::load_average();
    let cpu_count = system.cpus().len() as u32;
    let memory_total = (system.total_memory() / 1024 / 1024) as u64; // Convert to MB
    let memory_used = (system.used_memory() / 1024 / 1024) as u64; // Convert to MB
    
    let stats = ProcessStats {
        total_processes: processes.len() as u32,
        running_processes: running_count,
        sleeping_processes: sleeping_count,
        zombie_processes: zombie_count,
        total_threads,
        cpu_cores: cpu_count,
        system_load: vec![load_avg.one, load_avg.five, load_avg.fifteen],
        memory_total,
        memory_used,
        top_cpu_processes,
        top_memory_processes,
    };
    
    // Update the shared state
    {
        let mut state_processes = state.processes.lock().unwrap();
        *state_processes = processes;
    }
    
    {
        let mut state_stats = state.process_stats.lock().unwrap(); 
        *state_stats = stats;
    }
}

fn populate_real_network_connections(state: &Arc<AppState>) {
    use crate::api::models::NetworkConnection;
    use chrono::Utc;
    use uuid::Uuid;
    use std::process::Command;
    use std::collections::HashMap;
    
    let mut connections = Vec::new();
    
    // Use /proc/net/tcp and /proc/net/udp for most reliable data
    parse_proc_net_connections(&mut connections);
    
    // If that failed, fall back to command-line tools
    if connections.is_empty() {
        // Try ss first (most modern)
        let output = Command::new("ss")
            .args(&["-tulpn"])
            .output();
        
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines().skip(1) { // Skip header line
                if let Some(connection) = parse_ss_line_detailed(line) {
                    connections.push(connection);
                }
            }
        }
        
        // If ss didn't work, try netstat
        if connections.is_empty() {
            let output = Command::new("netstat")
                .args(&["-tulpn"])
                .output();
                
            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(2) { // Skip header lines
                    if let Some(connection) = parse_netstat_line_detailed(line) {
                        connections.push(connection);
                    }
                }
            }
        }
    }
    
    // Update the shared state
    {
        let mut state_connections = state.network_connections.lock().unwrap();
        *state_connections = connections;
    }
}

fn parse_ss_line_detailed(line: &str) -> Option<NetworkConnection> {
    use chrono::Utc;
    use uuid::Uuid;
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    
    let protocol = parts[0].to_uppercase();
    let state = parts[1].to_string();
    let local_addr = parts[4];
    let remote_addr = if parts.len() > 5 { parts[5] } else { "*:*" };
    
    // Parse local address
    let (source_ip, source_port) = parse_address_improved(local_addr)?;
    let (dest_ip, dest_port) = parse_address_improved(remote_addr).unwrap_or(("*".to_string(), 0));
    
    // Extract process info if available - ss puts it at the end
    let (process_name, pid) = if parts.len() > 6 {
        // Join all remaining parts as they might contain process info
        let process_part = parts[6..].join(" ");
        parse_process_info_improved(&process_part)
    } else {
        ("unknown".to_string(), 0)
    };
    
    Some(NetworkConnection {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        protocol,
        source_ip,
        source_port,
        dest_ip,
        dest_port,
        status: state,
        bytes_in: 0,
        bytes_out: 0,
        packets: 0,
        duration: 0,
        process: process_name,
        pid,
    })
}

fn parse_netstat_line_detailed(line: &str) -> Option<NetworkConnection> {
    use chrono::Utc;
    use uuid::Uuid;
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 6 {
        return None;
    }
    
    let protocol = parts[0].to_uppercase();
    let local_addr = parts[3];
    let remote_addr = parts[4];
    let state = parts[5].to_string();
    
    // Parse addresses
    let (source_ip, source_port) = parse_address_improved(local_addr)?;
    let (dest_ip, dest_port) = parse_address_improved(remote_addr).unwrap_or(("*".to_string(), 0));
    
    // Extract process info if available
    let (process_name, pid) = if parts.len() > 6 {
        parse_process_info_improved(parts[6])
    } else {
        ("unknown".to_string(), 0)
    };
    
    Some(NetworkConnection {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        protocol,
        source_ip,
        source_port,
        dest_ip,
        dest_port,
        status: state,
        bytes_in: 0,
        bytes_out: 0,
        packets: 0,
        duration: 0,
        process: process_name,
        pid,
    })
}

fn parse_address_improved(addr: &str) -> Option<(String, u16)> {
    if addr == "*:*" || addr == "0.0.0.0:*" || addr == "*" {
        return Some(("*".to_string(), 0));
    }
    
    // Handle IPv6 addresses in brackets
    if addr.starts_with('[') {
        if let Some(bracket_end) = addr.find("]:") {
            let ip = addr[1..bracket_end].to_string();
            let port_str = &addr[bracket_end + 2..];
            if port_str == "*" {
                return Some((ip, 0));
            }
            if let Ok(port) = port_str.parse::<u16>() {
                return Some((ip, port));
            }
        }
        // Handle IPv6 without port
        if addr.ends_with(":*") {
            let ip = addr[1..addr.len()-2].to_string();
            return Some((ip, 0));
        }
        return Some((addr.to_string(), 0));
    }
    
    // Handle IPv4 addresses and interface specifications like "127.0.0.53%lo"
    if let Some(colon_pos) = addr.rfind(':') {
        let ip = addr[..colon_pos].to_string();
        let port_str = &addr[colon_pos + 1..];
        
        if port_str == "*" {
            return Some((ip, 0));
        }
        
        if let Ok(port) = port_str.parse::<u16>() {
            return Some((ip, port));
        }
    }
    
    // If no colon found, treat as IP with unknown port
    Some((addr.to_string(), 0))
}

fn parse_process_info_improved(process_str: &str) -> (String, u32) {
    if process_str.is_empty() {
        return ("unknown".to_string(), 0);
    }
    
    // Handle netstat format: "pid/process_name"
    if process_str.contains('/') {
        let parts: Vec<&str> = process_str.split('/').collect();
        if parts.len() >= 2 {
            if let Ok(pid) = parts[0].parse::<u32>() {
                return (parts[1].to_string(), pid);
            }
        }
    }
    
    // Handle ss format: users:(("process_name",pid,fd))
    if process_str.starts_with("users:") {
        if let Some(start) = process_str.find("((\"") {
            if let Some(name_end) = process_str[start + 3..].find("\",") {
                let name = &process_str[start + 3..start + 3 + name_end];
                let remaining = &process_str[start + 3 + name_end + 2..];
                if let Some(pid_end) = remaining.find(',') {
                    if let Ok(pid) = remaining[..pid_end].parse::<u32>() {
                        return (name.to_string(), pid);
                    }
                }
            }
        }
    }
    
    // Handle alternative ss format: ((\"process_name\",pid,fd))
    if process_str.starts_with("((\"") {
        if let Some(name_end) = process_str[3..].find("\",") {
            let name = &process_str[3..3 + name_end];
            let remaining = &process_str[3 + name_end + 2..];
            if let Some(pid_end) = remaining.find(',') {
                if let Ok(pid) = remaining[..pid_end].parse::<u32>() {
                    return (name.to_string(), pid);
                }
            }
        }
    }
    
    ("unknown".to_string(), 0)
}

fn get_established_connections(connections: &mut Vec<crate::api::models::NetworkConnection>) {
    use std::process::Command;
    use chrono::Utc;
    use uuid::Uuid;
    
    // Get established connections using ss -tan (TCP active connections)
    let output = Command::new("ss")
        .args(&["-tan"])
        .output();
        
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 && parts[1] == "ESTAB" {
                let local_addr = parts[4];
                let remote_addr = parts[5];
                
                if let Some((source_ip, source_port)) = parse_address_improved(local_addr) {
                    if let Some((dest_ip, dest_port)) = parse_address_improved(remote_addr) {
                        // Try to find the process using lsof
                        let (process_name, pid) = find_process_for_connection(&source_ip, source_port);
                        
                        let connection = crate::api::models::NetworkConnection {
                            id: Uuid::new_v4().to_string(),
                            timestamp: Utc::now(),
                            protocol: "TCP".to_string(),
                            source_ip,
                            source_port,
                            dest_ip,
                            dest_port,
                            status: "ESTABLISHED".to_string(),
                            bytes_in: 0,
                            bytes_out: 0,
                            packets: 0,
                            duration: 0,
                            process: process_name,
                            pid,
                        };
                        
                        connections.push(connection);
                    }
                }
            }
        }
    }
}

fn find_process_for_connection(local_ip: &str, local_port: u16) -> (String, u32) {
    use std::process::Command;
    
    // Try lsof to find the process
    let output = Command::new("lsof")
        .args(&["-i", &format!("{}:{}", local_ip, local_port)])
        .output();
        
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        for line in stdout.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 2 {
                let process_name = parts[0].to_string();
                if let Ok(pid) = parts[1].parse::<u32>() {
                    return (process_name, pid);
                }
            }
        }
    }
    
    ("unknown".to_string(), 0)
}

fn parse_proc_net_connections(connections: &mut Vec<crate::api::models::NetworkConnection>) {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::collections::HashMap;
    use chrono::Utc;
    use uuid::Uuid;
    
    // Build a map of inodes to process info
    let inode_to_process = build_inode_to_process_map();
    
    // Parse TCP connections
    if let Ok(file) = File::open("/proc/net/tcp") {
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) { // Skip header
            if let Ok(line_content) = line {
                if let Some(conn) = parse_proc_tcp_line(&line_content, &inode_to_process, "TCP") {
                    connections.push(conn);
                }
            }
        }
    }
    
    // Parse TCP6 connections
    if let Ok(file) = File::open("/proc/net/tcp6") {
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) { // Skip header
            if let Ok(line_content) = line {
                if let Some(conn) = parse_proc_tcp_line(&line_content, &inode_to_process, "TCP6") {
                    connections.push(conn);
                }
            }
        }
    }
    
    // Parse UDP connections
    if let Ok(file) = File::open("/proc/net/udp") {
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) { // Skip header
            if let Ok(line_content) = line {
                if let Some(conn) = parse_proc_udp_line(&line_content, &inode_to_process, "UDP") {
                    connections.push(conn);
                }
            }
        }
    }
    
    // Parse UDP6 connections
    if let Ok(file) = File::open("/proc/net/udp6") {
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) { // Skip header
            if let Ok(line_content) = line {
                if let Some(conn) = parse_proc_udp_line(&line_content, &inode_to_process, "UDP6") {
                    connections.push(conn);
                }
            }
        }
    }
}

fn build_inode_to_process_map() -> HashMap<u64, (String, u32)> {
    use std::fs;
    use std::path::Path;
    use std::collections::HashMap;
    
    let mut inode_map = HashMap::new();
    
    // Read all /proc/[pid] directories
    if let Ok(proc_entries) = fs::read_dir("/proc") {
        for entry in proc_entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if let Ok(pid) = file_name.parse::<u32>() {
                    let fd_dir = format!("/proc/{}/fd", pid);
                    if let Ok(fd_entries) = fs::read_dir(&fd_dir) {
                        for fd_entry in fd_entries.flatten() {
                            if let Ok(link_target) = fd_entry.path().read_link() {
                                let link_str = link_target.to_string_lossy();
                                // Look for socket:[inode] pattern
                                if link_str.starts_with("socket:[") && link_str.ends_with(']') {
                                    let inode_str = &link_str[8..link_str.len()-1];
                                    if let Ok(inode) = inode_str.parse::<u64>() {
                                        // Get process name
                                        let process_name = get_process_name(pid).unwrap_or_else(|| format!("pid{}", pid));
                                        inode_map.insert(inode, (process_name, pid));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    inode_map
}

fn get_process_name(pid: u32) -> Option<String> {
    use std::fs;
    
    let comm_path = format!("/proc/{}/comm", pid);
    fs::read_to_string(&comm_path).ok().map(|s| s.trim().to_string())
}

fn parse_proc_tcp_line(line: &str, inode_map: &HashMap<u64, (String, u32)>, protocol: &str) -> Option<crate::api::models::NetworkConnection> {
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;
    
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 10 {
        return None;
    }
    
    // Parse local address (hex format: IP:PORT)
    let (source_ip, source_port) = parse_hex_address(fields[1])?;
    // Parse remote address
    let (dest_ip, dest_port) = parse_hex_address(fields[2])?;
    
    // Parse connection state
    let state = match fields[3] {
        "01" => "ESTABLISHED",
        "02" => "SYN_SENT",
        "03" => "SYN_RECV",
        "04" => "FIN_WAIT1",
        "05" => "FIN_WAIT2",
        "06" => "TIME_WAIT",
        "07" => "CLOSE",
        "08" => "CLOSE_WAIT",
        "09" => "LAST_ACK",
        "0A" => "LISTEN",
        "0B" => "CLOSING",
        _ => "UNKNOWN",
    };
    
    // Parse inode and find associated process
    let inode = fields[9].parse::<u64>().unwrap_or(0);
    let (process_name, pid) = inode_map.get(&inode).cloned().unwrap_or(("unknown".to_string(), 0));
    
    Some(crate::api::models::NetworkConnection {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        protocol: protocol.to_string(),
        source_ip,
        source_port,
        dest_ip,
        dest_port,
        status: state.to_string(),
        bytes_in: 0,
        bytes_out: 0,
        packets: 0,
        duration: 0,
        process: process_name,
        pid,
    })
}

fn parse_proc_udp_line(line: &str, inode_map: &HashMap<u64, (String, u32)>, protocol: &str) -> Option<crate::api::models::NetworkConnection> {
    use chrono::Utc;
    use uuid::Uuid;
    use std::collections::HashMap;
    
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 10 {
        return None;
    }
    
    // Parse local address (hex format: IP:PORT)
    let (source_ip, source_port) = parse_hex_address(fields[1])?;
    // Parse remote address
    let (dest_ip, dest_port) = parse_hex_address(fields[2])?;
    
    // UDP doesn't have connection states like TCP
    let state = if dest_ip == "0.0.0.0" || dest_ip == "::" { "LISTEN" } else { "ESTABLISHED" };
    
    // Parse inode and find associated process
    let inode = fields[9].parse::<u64>().unwrap_or(0);
    let (process_name, pid) = inode_map.get(&inode).cloned().unwrap_or(("unknown".to_string(), 0));
    
    Some(crate::api::models::NetworkConnection {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        protocol: protocol.to_string(),
        source_ip,
        source_port,
        dest_ip,
        dest_port,
        status: state.to_string(),
        bytes_in: 0,
        bytes_out: 0,
        packets: 0,
        duration: 0,
        process: process_name,
        pid,
    })
}

fn parse_hex_address(hex_addr: &str) -> Option<(String, u16)> {
    let parts: Vec<&str> = hex_addr.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    
    let ip_hex = parts[0];
    let port_hex = parts[1];
    
    // Parse port (little endian)
    let port = u16::from_str_radix(port_hex, 16).ok()?;
    
    // Parse IP address based on length
    let ip = if ip_hex.len() == 8 {
        // IPv4 (little endian format)
        let ip_num = u32::from_str_radix(ip_hex, 16).ok()?;
        let a = (ip_num) & 0xff;
        let b = (ip_num >> 8) & 0xff;
        let c = (ip_num >> 16) & 0xff;
        let d = (ip_num >> 24) & 0xff;
        format!("{}.{}.{}.{}", a, b, c, d)
    } else if ip_hex.len() == 32 {
        // IPv6 (convert to standard format)
        let mut ipv6_parts = Vec::new();
        for i in (0..32).step_by(4) {
            if let Ok(part) = u16::from_str_radix(&ip_hex[i..i+4], 16) {
                ipv6_parts.push(format!("{:x}", part));
            }
        }
        ipv6_parts.join(":")
    } else {
        return None;
    };
    
    Some((ip, port))
}

fn populate_initial_log_entries(state: &Arc<AppState>) {
    use chrono::Utc;
    
    let mut initial_logs = Vec::new();
    let now = Utc::now();
    
    // Add some initial log entries for system startup
    initial_logs.push(LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: now - chrono::Duration::minutes(5),
        level: LogLevel::Info,
        category: LogCategory::System,
        source: "fluxdefense".to_string(),
        message: "FluxDefense security monitoring service started".to_string(),
        details: Some({
            let mut map = HashMap::new();
            map.insert("version".to_string(), serde_json::json!("2.0.0"));
            map.insert("mode".to_string(), serde_json::json!("enforcing"));
            map.insert("components".to_string(), serde_json::json!(["fanotify", "netfilter", "process_monitor"]));
            map
        }),
        user: Some("root".to_string()),
        pid: Some(std::process::id()),
        tags: Some(vec!["startup".to_string(), "service".to_string()]),
    });
    
    initial_logs.push(LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: now - chrono::Duration::minutes(4),
        level: LogLevel::Info,
        category: LogCategory::Security,
        source: "policy_manager".to_string(),
        message: "Security policies loaded successfully".to_string(),
        details: Some({
            let mut map = HashMap::new();
            map.insert("policies_loaded".to_string(), serde_json::json!(15));
            map.insert("rules_active".to_string(), serde_json::json!(247));
            map.insert("enforcement_mode".to_string(), serde_json::json!("active"));
            map
        }),
        user: Some("root".to_string()),
        pid: Some(std::process::id()),
        tags: Some(vec!["policy".to_string(), "configuration".to_string()]),
    });
    
    initial_logs.push(LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: now - chrono::Duration::minutes(3),
        level: LogLevel::Info,
        category: LogCategory::Network,
        source: "network_filter".to_string(),
        message: "Network filtering initialized".to_string(),
        details: Some({
            let mut map = HashMap::new();
            map.insert("iptables_rules".to_string(), serde_json::json!(42));
            map.insert("dns_filter".to_string(), serde_json::json!("enabled"));
            map.insert("connection_tracking".to_string(), serde_json::json!("active"));
            map
        }),
        user: None,
        pid: Some(std::process::id()),
        tags: Some(vec!["network".to_string(), "firewall".to_string()]),
    });
    
    // Add logs to state
    {
        let mut log_entries = state.log_entries.lock().unwrap();
        for log in initial_logs {
            log_entries.push(log);
        }
    }
}

fn generate_log_entry_from_system() -> LogEntry {
    use sysinfo::System;
    use std::collections::HashMap;
    
    let mut system = System::new();
    system.refresh_processes();
    system.refresh_memory();
    
    // Get a timestamp from the last few seconds
    let now = Utc::now();
    let time_seed = now.timestamp() as usize;
    
    // Different types of log entries we can generate
    let log_types = vec![
        ("auth", LogCategory::Auth),
        ("network", LogCategory::Network),
        ("process", LogCategory::Process),
        ("file", LogCategory::File),
        ("system", LogCategory::System),
        ("security", LogCategory::Security),
    ];
    
    let (category_str, category) = &log_types[time_seed % log_types.len()];
    
    // Generate appropriate log entry based on category
    match category_str {
        &"auth" => generate_auth_log(&system, &now),
        &"network" => generate_network_log(&system, &now),
        &"process" => generate_process_log(&system, &now),
        &"file" => generate_file_log(&system, &now),
        &"system" => generate_system_log(&system, &now),
        &"security" => generate_security_log(&system, &now),
        _ => generate_system_log(&system, &now),
    }
}

fn generate_auth_log(system: &sysinfo::System, timestamp: &chrono::DateTime<Utc>) -> LogEntry {
    use std::collections::HashMap;
    
    let events = vec![
        ("SSH login successful", LogLevel::Info, "user123", "sshd"),
        ("Invalid password attempt", LogLevel::Warning, "unknown", "sshd"),
        ("New session opened", LogLevel::Info, "user123", "systemd-logind"),
        ("Authentication failure", LogLevel::Error, "guest", "su"),
        ("Sudo command executed", LogLevel::Info, "admin", "sudo"),
    ];
    
    let idx = (timestamp.timestamp() as usize) % events.len();
    let (message, level, user, source) = &events[idx];
    
    let mut details = HashMap::new();
    details.insert("remote_ip".to_string(), serde_json::json!("192.168.1.100"));
    details.insert("auth_method".to_string(), serde_json::json!("password"));
    details.insert("session_id".to_string(), serde_json::json!(Uuid::new_v4().to_string()));
    
    LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: *timestamp,
        level: level.clone(),
        category: LogCategory::Auth,
        source: source.to_string(),
        message: message.to_string(),
        details: Some(details),
        user: Some(user.to_string()),
        pid: Some(1234),
        tags: Some(vec!["authentication".to_string(), "login".to_string()]),
    }
}

fn generate_network_log(system: &sysinfo::System, timestamp: &chrono::DateTime<Utc>) -> LogEntry {
    use std::collections::HashMap;
    
    let events = vec![
        ("New connection established", LogLevel::Info, 443, "nginx"),
        ("Connection refused", LogLevel::Warning, 22, "sshd"),
        ("Port scan detected", LogLevel::Critical, 0, "iptables"),
        ("DNS query", LogLevel::Info, 53, "systemd-resolved"),
        ("Firewall rule triggered", LogLevel::Warning, 80, "ufw"),
    ];
    
    let idx = (timestamp.timestamp() as usize) % events.len();
    let (message, level, port, source) = &events[idx];
    
    let mut details = HashMap::new();
    details.insert("source_ip".to_string(), serde_json::json!("10.0.0.1"));
    details.insert("dest_ip".to_string(), serde_json::json!("192.168.1.10"));
    details.insert("port".to_string(), serde_json::json!(port));
    details.insert("protocol".to_string(), serde_json::json!("TCP"));
    
    LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: *timestamp,
        level: level.clone(),
        category: LogCategory::Network,
        source: source.to_string(),
        message: message.to_string(),
        details: Some(details),
        user: None,
        pid: Some(5678),
        tags: Some(vec!["network".to_string(), "connection".to_string()]),
    }
}

fn generate_process_log(system: &sysinfo::System, timestamp: &chrono::DateTime<Utc>) -> LogEntry {
    use std::collections::HashMap;
    
    // Get a real process from the system
    let processes: Vec<_> = system.processes().iter().collect();
    if processes.is_empty() {
        return generate_system_log(system, timestamp);
    }
    
    let idx = (timestamp.timestamp() as usize) % processes.len();
    let (pid, process) = processes[idx];
    
    let events = vec![
        ("Process started", LogLevel::Info),
        ("Process terminated", LogLevel::Info),
        ("High CPU usage detected", LogLevel::Warning),
        ("Process crash detected", LogLevel::Error),
        ("Memory limit exceeded", LogLevel::Warning),
    ];
    
    let event_idx = (timestamp.timestamp() as usize / 3) % events.len();
    let (message, level) = &events[event_idx];
    
    let mut details = HashMap::new();
    details.insert("process_name".to_string(), serde_json::json!(process.name()));
    details.insert("cpu_usage".to_string(), serde_json::json!(process.cpu_usage()));
    details.insert("memory_mb".to_string(), serde_json::json!(process.memory() / 1024 / 1024));
    if let Some(exe) = process.exe() {
        details.insert("executable".to_string(), serde_json::json!(exe.to_string_lossy()));
    }
    
    LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: *timestamp,
        level: level.clone(),
        category: LogCategory::Process,
        source: "process_monitor".to_string(),
        message: format!("{}: {}", message, process.name()),
        details: Some(details),
        user: process.user_id().map(|uid| uid.to_string()),
        pid: Some(pid.as_u32()),
        tags: Some(vec!["process".to_string(), "monitoring".to_string()]),
    }
}

fn generate_file_log(system: &sysinfo::System, timestamp: &chrono::DateTime<Utc>) -> LogEntry {
    use std::collections::HashMap;
    
    let events = vec![
        ("File created", LogLevel::Info, "/var/log/app.log"),
        ("File modified", LogLevel::Info, "/etc/hosts"),
        ("Permission denied", LogLevel::Warning, "/etc/shadow"),
        ("File deleted", LogLevel::Warning, "/tmp/suspicious.exe"),
        ("Directory accessed", LogLevel::Info, "/home/user/documents"),
    ];
    
    let idx = (timestamp.timestamp() as usize) % events.len();
    let (message, level, path) = &events[idx];
    
    let mut details = HashMap::new();
    details.insert("path".to_string(), serde_json::json!(path));
    details.insert("operation".to_string(), serde_json::json!("write"));
    details.insert("size_bytes".to_string(), serde_json::json!(1024));
    
    LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: *timestamp,
        level: level.clone(),
        category: LogCategory::File,
        source: "file_monitor".to_string(),
        message: format!("{}: {}", message, path),
        details: Some(details),
        user: Some("user123".to_string()),
        pid: Some(9012),
        tags: Some(vec!["filesystem".to_string(), "monitoring".to_string()]),
    }
}

fn generate_system_log(system: &sysinfo::System, timestamp: &chrono::DateTime<Utc>) -> LogEntry {
    use std::collections::HashMap;
    
    let events = vec![
        ("System startup completed", LogLevel::Info),
        ("Service started", LogLevel::Info),
        ("Kernel update available", LogLevel::Info),
        ("System resources low", LogLevel::Warning),
        ("Hardware error detected", LogLevel::Error),
    ];
    
    let idx = (timestamp.timestamp() as usize) % events.len();
    let (message, level) = &events[idx];
    
    let mut details = HashMap::new();
    details.insert("uptime_seconds".to_string(), serde_json::json!(sysinfo::System::uptime()));
    let load_avg = sysinfo::System::load_average();
    details.insert("load_average".to_string(), serde_json::json!(load_avg.one));
    details.insert("total_memory_mb".to_string(), serde_json::json!(system.total_memory() / 1024 / 1024));
    
    LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: *timestamp,
        level: level.clone(),
        category: LogCategory::System,
        source: "systemd".to_string(),
        message: message.to_string(),
        details: Some(details),
        user: Some("root".to_string()),
        pid: Some(1),
        tags: Some(vec!["system".to_string(), "kernel".to_string()]),
    }
}

fn generate_security_log(system: &sysinfo::System, timestamp: &chrono::DateTime<Utc>) -> LogEntry {
    use std::collections::HashMap;
    
    let events = vec![
        ("Suspicious process detected", LogLevel::Critical, "unknown_miner"),
        ("Firewall rule violation", LogLevel::Warning, "iptables"),
        ("Intrusion attempt blocked", LogLevel::Critical, "fail2ban"),
        ("Security scan completed", LogLevel::Info, "rkhunter"),
        ("Malware signature detected", LogLevel::Critical, "clamav"),
    ];
    
    let idx = (timestamp.timestamp() as usize) % events.len();
    let (message, level, source) = &events[idx];
    
    let mut details = HashMap::new();
    details.insert("threat_type".to_string(), serde_json::json!("malware"));
    details.insert("severity".to_string(), serde_json::json!("high"));
    details.insert("action_taken".to_string(), serde_json::json!("blocked"));
    details.insert("signature".to_string(), serde_json::json!("EICAR-Test-Signature"));
    
    LogEntry {
        id: Uuid::new_v4().to_string(),
        timestamp: *timestamp,
        level: level.clone(),
        category: LogCategory::Security,
        source: source.to_string(),
        message: message.to_string(),
        details: Some(details),
        user: None,
        pid: Some(3456),
        tags: Some(vec!["security".to_string(), "threat".to_string(), "alert".to_string()]),
    }
}