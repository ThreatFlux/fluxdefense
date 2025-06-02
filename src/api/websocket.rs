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
    SystemMetrics, ThreatDetection, LogEntry, DnsQuery
};
use crate::api::handlers::AppState;

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
}