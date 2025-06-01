use std::path::{Path, PathBuf};
use std::fs;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use anyhow::Result;
use tracing::{info, warn, debug};
use uuid::Uuid;

use crate::policy::{FilePolicy, NetworkPolicy};
use crate::scanner::FileRecord;
use crate::system_metrics::{SystemMetrics, SystemMetricsCollector};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub process_info: ProcessInfo,
    pub verdict: Verdict,
    pub policy_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityEventType {
    FileExecution {
        target_path: PathBuf,
        file_hash: Option<String>,
        code_signature: Option<String>,
    },
    FileAccess {
        target_path: PathBuf,
        access_type: FileAccessType,
    },
    NetworkConnection {
        remote_ip: String,
        remote_port: u16,
        domain: Option<String>,
        protocol: NetworkProtocol,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileAccessType {
    Read,
    Write,
    Execute,
    Create,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkProtocol {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub path: PathBuf,
    pub parent_pid: Option<u32>,
    pub user_id: u32,
    pub executable_hash: Option<String>,
    pub command_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Verdict {
    Allow,
    Deny,
    Log,  // Passive mode - just log the event
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLog {
    pub events: Vec<SecurityEvent>,
    pub statistics: EventStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStatistics {
    pub total_events: usize,
    pub events_by_type: HashMap<String, usize>,
    pub events_by_verdict: HashMap<String, usize>,
    pub unique_processes: usize,
    pub unique_files: usize,
    pub start_time: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

pub struct PassiveMonitor {
    events: Arc<Mutex<Vec<SecurityEvent>>>,
    statistics: Arc<Mutex<EventStatistics>>,
    file_policy: FilePolicy,
    network_policy: NetworkPolicy,
    log_file_path: PathBuf,
    passive_mode: bool,
    system_metrics_collector: SystemMetricsCollector,
    latest_system_metrics: Arc<Mutex<Option<SystemMetrics>>>,
}

impl PassiveMonitor {
    pub fn new(log_file_path: PathBuf, passive_mode: bool) -> Result<Self> {
        let now = Utc::now();
        let statistics = EventStatistics {
            total_events: 0,
            events_by_type: HashMap::new(),
            events_by_verdict: HashMap::new(),
            unique_processes: 0,
            unique_files: 0,
            start_time: now,
            last_updated: now,
        };

        Ok(Self {
            events: Arc::new(Mutex::new(Vec::new())),
            statistics: Arc::new(Mutex::new(statistics)),
            file_policy: FilePolicy::default(),
            network_policy: NetworkPolicy::default(),
            log_file_path,
            passive_mode,
            system_metrics_collector: SystemMetricsCollector::new(),
            latest_system_metrics: Arc::new(Mutex::new(None)),
        })
    }

    pub fn load_whitelist_data(&mut self, whitelist_dir: &Path) -> Result<()> {
        info!("Loading whitelist data from: {:?}", whitelist_dir);
        
        if !whitelist_dir.exists() {
            warn!("Whitelist directory does not exist: {:?}", whitelist_dir);
            return Ok(());
        }

        let manifest_path = whitelist_dir.join("scan_manifest.json");
        if !manifest_path.exists() {
            warn!("No scan manifest found in whitelist directory");
            return Ok(());
        }

        // Load scan manifest to get file records
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: crate::scanner::ScanManifest = serde_json::from_str(&manifest_content)?;
        
        info!("Loading {} file records from whitelist", manifest.file_records.len());
        
        let mut loaded_count = 0;
        for (uuid, filename) in &manifest.file_records {
            let file_path = whitelist_dir.join(filename);
            if let Ok(content) = fs::read_to_string(&file_path) {
                if let Ok(file_record) = serde_json::from_str::<FileRecord>(&content) {
                    // Add to file policy whitelist
                    self.file_policy.add_allowed_path(file_record.path.clone());
                    
                    if !file_record.sha256_hash.is_empty() {
                        self.file_policy.add_allowed_hash(file_record.sha256_hash.clone());
                    }
                    
                    if let Some(ref sig) = file_record.code_signature {
                        if !sig.authority.is_empty() {
                            self.file_policy.add_allowed_signer(sig.authority.clone());
                        }
                    }
                    
                    loaded_count += 1;
                }
            }
        }
        
        info!("Successfully loaded {} file records into whitelist", loaded_count);
        Ok(())
    }

    pub fn handle_file_execution_event(
        &self,
        process_info: ProcessInfo,
        target_path: PathBuf,
        file_hash: Option<String>,
        code_signature: Option<String>,
    ) -> Verdict {
        let event_type = SecurityEventType::FileExecution {
            target_path: target_path.clone(),
            file_hash: file_hash.clone(),
            code_signature: code_signature.clone(),
        };

        let (verdict, reason) = if self.passive_mode {
            (Verdict::Log, "Passive mode - logging only".to_string())
        } else {
            self.evaluate_file_policy(&target_path, file_hash.as_deref(), code_signature.as_deref())
        };

        let event = SecurityEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            process_info,
            verdict: verdict.clone(),
            policy_reason: reason,
        };

        self.log_event(event);
        verdict
    }

    pub fn handle_file_access_event(
        &self,
        process_info: ProcessInfo,
        target_path: PathBuf,
        access_type: FileAccessType,
    ) -> Verdict {
        let event_type = SecurityEventType::FileAccess {
            target_path: target_path.clone(),
            access_type,
        };

        let (verdict, reason) = if self.passive_mode {
            (Verdict::Log, "Passive mode - logging only".to_string())
        } else {
            // For file access, we're generally more permissive
            if self.file_policy.is_path_allowed(&target_path) {
                (Verdict::Allow, "Path in whitelist".to_string())
            } else {
                (Verdict::Deny, "Path not in whitelist".to_string())
            }
        };

        let event = SecurityEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            process_info,
            verdict: verdict.clone(),
            policy_reason: reason,
        };

        self.log_event(event);
        verdict
    }

    pub fn handle_network_connection_event(
        &self,
        process_info: ProcessInfo,
        remote_ip: String,
        remote_port: u16,
        domain: Option<String>,
        protocol: NetworkProtocol,
    ) -> Verdict {
        let event_type = SecurityEventType::NetworkConnection {
            remote_ip: remote_ip.clone(),
            remote_port,
            domain: domain.clone(),
            protocol,
        };

        let (verdict, reason) = if self.passive_mode {
            (Verdict::Log, "Passive mode - logging only".to_string())
        } else {
            let ip_addr = match remote_ip.parse() {
                Ok(ip) => ip,
                Err(_) => {
                    return self.log_and_return_verdict(
                        SecurityEvent {
                            id: Uuid::new_v4().to_string(),
                            timestamp: Utc::now(),
                            event_type,
                            process_info,
                            verdict: Verdict::Deny,
                            policy_reason: "Invalid IP address".to_string(),
                        }
                    );
                }
            };

            if self.network_policy.is_connection_allowed(ip_addr, remote_port, domain.as_deref()) {
                (Verdict::Allow, "Connection allowed by policy".to_string())
            } else {
                (Verdict::Deny, "Connection denied by policy".to_string())
            }
        };

        let event = SecurityEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            event_type,
            process_info,
            verdict: verdict.clone(),
            policy_reason: reason,
        };

        self.log_event(event);
        verdict
    }

    fn evaluate_file_policy(
        &self,
        path: &Path,
        hash: Option<&str>,
        signer: Option<&str>,
    ) -> (Verdict, String) {
        if self.file_policy.is_execution_allowed(path, hash, signer) {
            (Verdict::Allow, "File execution allowed by policy".to_string())
        } else {
            (Verdict::Deny, "File execution denied by policy".to_string())
        }
    }

    fn log_and_return_verdict(&self, event: SecurityEvent) -> Verdict {
        let verdict = event.verdict.clone();
        self.log_event(event);
        verdict
    }

    fn log_event(&self, event: SecurityEvent) {
        // Log to structured log
        match &event.event_type {
            SecurityEventType::FileExecution { target_path, .. } => {
                debug!("File execution event: {:?} -> {:?}", target_path, event.verdict);
            }
            SecurityEventType::FileAccess { target_path, access_type } => {
                debug!("File access event: {:?} ({:?}) -> {:?}", target_path, access_type, event.verdict);
            }
            SecurityEventType::NetworkConnection { remote_ip, remote_port, .. } => {
                debug!("Network connection event: {}:{} -> {:?}", remote_ip, remote_port, event.verdict);
            }
        }

        // Add to in-memory event store
        {
            let mut events = self.events.lock().unwrap();
            events.push(event.clone());
        }

        // Update statistics
        {
            let mut stats = self.statistics.lock().unwrap();
            stats.total_events += 1;
            stats.last_updated = Utc::now();
            
            let event_type_key = match &event.event_type {
                SecurityEventType::FileExecution { .. } => "file_execution",
                SecurityEventType::FileAccess { .. } => "file_access",
                SecurityEventType::NetworkConnection { .. } => "network_connection",
            };
            *stats.events_by_type.entry(event_type_key.to_string()).or_insert(0) += 1;
            
            let verdict_key = format!("{:?}", event.verdict).to_lowercase();
            *stats.events_by_verdict.entry(verdict_key).or_insert(0) += 1;
        }

        // Write to log file
        if let Err(e) = self.write_event_to_file(&event) {
            warn!("Failed to write event to log file: {}", e);
        }
    }

    fn write_event_to_file(&self, event: &SecurityEvent) -> Result<()> {
        let json = serde_json::to_string(event)?;
        
        // Ensure parent directory exists
        if let Some(parent) = self.log_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Append to log file
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;
        
        writeln!(file, "{}", json)?;
        Ok(())
    }

    pub fn get_event_log(&self) -> EventLog {
        let events = self.events.lock().unwrap().clone();
        let statistics = self.statistics.lock().unwrap().clone();
        
        EventLog {
            events,
            statistics,
        }
    }

    pub fn get_statistics(&self) -> EventStatistics {
        self.statistics.lock().unwrap().clone()
    }

    pub fn save_event_log(&self, path: &Path) -> Result<()> {
        let event_log = self.get_event_log();
        let json = serde_json::to_string_pretty(&event_log)?;
        fs::write(path, json)?;
        info!("Event log saved to: {:?}", path);
        Ok(())
    }

    pub fn set_passive_mode(&mut self, passive: bool) {
        self.passive_mode = passive;
        if passive {
            info!("Switched to passive mode - events will be logged only");
        } else {
            info!("Switched to active mode - events will be enforced");
        }
    }

    pub fn is_passive_mode(&self) -> bool {
        self.passive_mode
    }

    /// Collect current system metrics
    pub fn collect_system_metrics(&mut self) -> Result<SystemMetrics> {
        let metrics = self.system_metrics_collector.collect_metrics()?;
        
        // Store the latest metrics
        if let Ok(mut latest) = self.latest_system_metrics.lock() {
            *latest = Some(metrics.clone());
        }
        
        debug!("Collected system metrics: CPU {:.1}%, RAM {:.1}%, Disk I/O: R{}/W{}, Network I/O: R{}/W{}",
            metrics.cpu_usage,
            metrics.memory_usage,
            format_bytes(metrics.disk_read_rate),
            format_bytes(metrics.disk_write_rate),
            format_bytes(metrics.network_rx_rate),
            format_bytes(metrics.network_tx_rate)
        );
        
        Ok(metrics)
    }

    /// Get the latest collected system metrics
    pub fn get_latest_system_metrics(&self) -> Option<SystemMetrics> {
        self.latest_system_metrics.lock().ok()?.clone()
    }

    /// Start continuous system metrics collection in background
    pub fn start_system_metrics_collection(&self) -> std::thread::JoinHandle<()> {
        let latest_metrics = Arc::clone(&self.latest_system_metrics);
        let mut collector = SystemMetricsCollector::new();
        
        std::thread::spawn(move || {
            loop {
                match collector.collect_metrics() {
                    Ok(metrics) => {
                        if let Ok(mut latest) = latest_metrics.lock() {
                            *latest = Some(metrics);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to collect system metrics: {}", e);
                    }
                }
                
                // Collect metrics every 2 seconds
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        })
    }

    /// Enhanced event logging that includes system metrics
    pub fn log_event_with_metrics(&mut self, event: SecurityEvent) -> Result<()> {
        // Collect current system metrics
        let current_metrics = self.collect_system_metrics().ok();
        
        // Create enhanced event with system context
        let enhanced_event = EnhancedSecurityEvent {
            security_event: event.clone(),
            system_metrics: current_metrics,
            timestamp: Utc::now(),
        };

        // Log the enhanced event
        let log_entry = serde_json::to_string(&enhanced_event)?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file_path)?;
        
        use std::io::Write;
        writeln!(file, "{}", log_entry)?;
        
        // Also store in memory
        self.log_event(event);
        
        Ok(())
    }
}

/// Enhanced security event that includes system metrics context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedSecurityEvent {
    pub security_event: SecurityEvent,
    pub system_metrics: Option<SystemMetrics>,
    pub timestamp: DateTime<Utc>,
}

/// Format bytes for human-readable display
fn format_bytes(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{:.0}{}", size, UNITS[unit_index])
    } else {
        format!("{:.1}{}", size, UNITS[unit_index])
    }
}