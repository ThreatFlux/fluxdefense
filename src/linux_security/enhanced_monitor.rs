use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use sha2::{Sha256, Digest};
use std::io::Read;

use super::fanotify::{FanotifyMonitor, FanotifyEvent};
use super::netlink::{NetlinkMonitor, NetworkConnection};
use super::process_monitor::{ProcessMonitor, ProcessInfo};
use crate::monitor::{SecurityEvent, SecurityEventType, ProcessInfo as MonitorProcessInfo, 
                      FileAccessType, NetworkProtocol, Verdict};

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    // Whitelist/blacklist for file paths
    allowed_paths: HashSet<PathBuf>,
    denied_paths: HashSet<PathBuf>,
    
    // Whitelist/blacklist for file hashes
    allowed_hashes: HashSet<String>,
    denied_hashes: HashSet<String>,
    
    // Process executable whitelist
    allowed_executables: HashSet<PathBuf>,
    denied_executables: HashSet<PathBuf>,
    
    // Network rules
    allowed_ips: HashSet<String>,
    denied_ips: HashSet<String>,
    allowed_ports: HashSet<u16>,
    denied_ports: HashSet<u16>,
    
    // Behavior patterns
    suspicious_patterns: Vec<SuspiciousPattern>,
    
    // Mode settings
    enforcement_mode: EnforcementMode,
    log_allowed: bool,
    log_denied: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnforcementMode {
    Passive,    // Log only
    Permissive, // Log and allow
    Enforcing,  // Log and block
}

#[derive(Debug, Clone)]
pub struct SuspiciousPattern {
    name: String,
    description: String,
    check_fn: fn(&ProcessInfo, &FanotifyEvent) -> bool,
}

pub struct EnhancedSecurityMonitor {
    fanotify: Arc<Mutex<FanotifyMonitor>>,
    netlink: Arc<Mutex<NetlinkMonitor>>,
    process_monitor: Arc<Mutex<ProcessMonitor>>,
    policy: Arc<RwLock<SecurityPolicy>>,
    running: Arc<Mutex<bool>>,
    event_handler: Arc<dyn Fn(SecurityEvent) + Send + Sync>,
    hash_cache: Arc<Mutex<HashMap<PathBuf, String>>>,
}

impl EnhancedSecurityMonitor {
    pub fn new<F>(event_handler: F) -> Result<Self>
    where
        F: Fn(SecurityEvent) + Send + Sync + 'static
    {
        info!("Initializing enhanced Linux security monitor");
        
        // Initialize components
        let fanotify = Arc::new(Mutex::new(FanotifyMonitor::new()?));
        let netlink = Arc::new(Mutex::new(NetlinkMonitor::new()?));
        let process_monitor = Arc::new(Mutex::new(ProcessMonitor::new()));
        
        // Default policy
        let policy = SecurityPolicy {
            allowed_paths: HashSet::new(),
            denied_paths: HashSet::new(),
            allowed_hashes: HashSet::new(),
            denied_hashes: HashSet::new(),
            allowed_executables: HashSet::new(),
            denied_executables: HashSet::new(),
            allowed_ips: HashSet::new(),
            denied_ips: HashSet::new(),
            allowed_ports: HashSet::new(),
            denied_ports: HashSet::new(),
            suspicious_patterns: Self::default_suspicious_patterns(),
            enforcement_mode: EnforcementMode::Passive,
            log_allowed: false,
            log_denied: true,
        };
        
        Ok(Self {
            fanotify,
            netlink,
            process_monitor,
            policy: Arc::new(RwLock::new(policy)),
            running: Arc::new(Mutex::new(false)),
            event_handler: Arc::new(event_handler),
            hash_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    fn default_suspicious_patterns() -> Vec<SuspiciousPattern> {
        vec![
            SuspiciousPattern {
                name: "Crypto Miner".to_string(),
                description: "Potential cryptocurrency miner detected".to_string(),
                check_fn: |proc, event| {
                    if let Some(path) = &event.path {
                        let path_str = path.to_string_lossy().to_lowercase();
                        path_str.contains("xmrig") || 
                        path_str.contains("minerd") ||
                        path_str.contains("ethminer") ||
                        proc.cmdline.iter().any(|arg| {
                            let arg_lower = arg.to_lowercase();
                            arg_lower.contains("pool.") || 
                            arg_lower.contains("stratum") ||
                            arg_lower.contains("--donate-level")
                        })
                    } else {
                        false
                    }
                },
            },
            SuspiciousPattern {
                name: "Reverse Shell".to_string(),
                description: "Potential reverse shell detected".to_string(),
                check_fn: |proc, _event| {
                    proc.cmdline.iter().any(|arg| {
                        arg.contains("/dev/tcp/") || 
                        arg.contains("nc -e") ||
                        arg.contains("bash -i") ||
                        (arg.contains("sh") && arg.contains(">&"))
                    })
                },
            },
            SuspiciousPattern {
                name: "Privilege Escalation".to_string(),
                description: "Potential privilege escalation attempt".to_string(),
                check_fn: |proc, event| {
                    if let Some(path) = &event.path {
                        let path_str = path.to_string_lossy();
                        (path_str.contains("/etc/passwd") || 
                         path_str.contains("/etc/shadow") ||
                         path_str.contains("/etc/sudoers")) &&
                        proc.uid != 0
                    } else {
                        false
                    }
                },
            },
        ]
    }
    
    pub fn start(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting enhanced security monitoring");
        
        // Start process monitoring
        {
            let mut pm = self.process_monitor.lock().unwrap();
            pm.start_monitoring()?;
        }
        
        // Start fanotify monitoring
        {
            let mut fm = self.fanotify.lock().unwrap();
            fm.start_monitoring()?;
        }
        
        // Start netlink monitoring
        {
            let mut nm = self.netlink.lock().unwrap();
            nm.start_monitoring()?;
        }
        
        // Start monitoring threads
        self.start_fanotify_thread();
        self.start_netlink_thread();
        self.start_process_scanning_thread();
        
        Ok(())
    }
    
    fn start_fanotify_thread(&self) {
        let fanotify = Arc::clone(&self.fanotify);
        let process_monitor = Arc::clone(&self.process_monitor);
        let policy = Arc::clone(&self.policy);
        let running = Arc::clone(&self.running);
        let event_handler = Arc::clone(&self.event_handler);
        let hash_cache = Arc::clone(&self.hash_cache);
        
        thread::spawn(move || {
            info!("Fanotify monitoring thread started");
            
            while *running.lock().unwrap() {
                let mut fm = match fanotify.lock() {
                    Ok(fm) => fm,
                    Err(_) => {
                        error!("Failed to lock fanotify monitor");
                        break;
                    }
                };
                
                // Read events with decision callback
                let policy_clone = Arc::clone(&policy);
                let process_monitor_clone = Arc::clone(&process_monitor);
                let hash_cache_clone = Arc::clone(&hash_cache);
                
                match fm.read_events(|event| {
                    Self::make_decision(event, &policy_clone, &process_monitor_clone, &hash_cache_clone)
                }) {
                    Ok(events) => {
                        drop(fm); // Release lock before processing
                        
                        for event in events {
                            Self::handle_fanotify_event(
                                &event,
                                &process_monitor,
                                &policy,
                                &event_handler,
                                &hash_cache,
                            );
                        }
                    }
                    Err(e) => {
                        error!("Error reading fanotify events: {}", e);
                    }
                }
                
                thread::sleep(Duration::from_millis(10));
            }
            
            info!("Fanotify monitoring thread stopped");
        });
    }
    
    fn make_decision(
        event: &FanotifyEvent,
        policy: &Arc<RwLock<SecurityPolicy>>,
        process_monitor: &Arc<Mutex<ProcessMonitor>>,
        hash_cache: &Arc<Mutex<HashMap<PathBuf, String>>>,
    ) -> bool {
        let policy = match policy.read() {
            Ok(p) => p,
            Err(_) => return true, // Allow on error
        };
        
        // In passive mode, always allow
        if policy.enforcement_mode == EnforcementMode::Passive {
            return true;
        }
        
        // Get process info
        let process_info = process_monitor
            .lock()
            .ok()
            .and_then(|pm| pm.get_process_by_pid(event.pid as u32).cloned());
        
        if let Some(path) = &event.path {
            // Check denied paths first
            if policy.denied_paths.contains(path) {
                debug!("Path denied by policy: {:?}", path);
                return false;
            }
            
            // Check allowed paths
            if policy.allowed_paths.contains(path) {
                return true;
            }
            
            // For execution events, check executable whitelist
            if event.is_exec() {
                if let Some(ref proc_info) = process_info {
                    if let Some(ref exe_path) = proc_info.exe_path {
                        if policy.denied_executables.contains(exe_path) {
                            debug!("Executable denied by policy: {:?}", exe_path);
                            return false;
                        }
                        if policy.allowed_executables.contains(exe_path) {
                            return true;
                        }
                    }
                }
                
                // Check file hash if available
                if let Ok(mut cache) = hash_cache.lock() {
                    if let Some(hash) = cache.get(path).cloned() {
                        if policy.denied_hashes.contains(&hash) {
                            debug!("File hash denied by policy: {}", hash);
                            return false;
                        }
                        if policy.allowed_hashes.contains(&hash) {
                            return true;
                        }
                    }
                }
            }
            
            // Check suspicious patterns
            if let Some(ref proc_info) = process_info {
                for pattern in &policy.suspicious_patterns {
                    if (pattern.check_fn)(proc_info, event) {
                        warn!("Suspicious pattern detected: {} - {}", pattern.name, pattern.description);
                        if policy.enforcement_mode == EnforcementMode::Enforcing {
                            return false;
                        }
                    }
                }
            }
        }
        
        // Default: allow in permissive mode, deny in enforcing mode
        policy.enforcement_mode != EnforcementMode::Enforcing
    }
    
    fn handle_fanotify_event(
        event: &FanotifyEvent,
        process_monitor: &Arc<Mutex<ProcessMonitor>>,
        policy: &Arc<RwLock<SecurityPolicy>>,
        event_handler: &Arc<dyn Fn(SecurityEvent) + Send + Sync>,
        hash_cache: &Arc<Mutex<HashMap<PathBuf, String>>>,
    ) {
        let process_info = process_monitor
            .lock()
            .ok()
            .and_then(|pm| pm.get_process_by_pid(event.pid as u32).cloned());
        
        let monitor_process_info = if let Some(info) = process_info {
            MonitorProcessInfo {
                pid: info.pid,
                path: info.exe_path.unwrap_or_else(|| PathBuf::from(&info.name)),
                parent_pid: Some(info.ppid),
                user_id: info.uid,
                executable_hash: None,
                command_line: Some(info.cmdline.join(" ")),
            }
        } else {
            MonitorProcessInfo {
                pid: event.pid as u32,
                path: PathBuf::from(format!("pid:{}", event.pid)),
                parent_pid: None,
                user_id: 0,
                executable_hash: None,
                command_line: None,
            }
        };
        
        if let Some(path) = &event.path {
            let file_hash = if event.is_exec() {
                // Calculate hash for executables
                Self::calculate_file_hash(path, hash_cache)
            } else {
                None
            };
            
            let event_type = if event.is_exec() {
                SecurityEventType::FileExecution {
                    target_path: path.clone(),
                    file_hash: file_hash.clone(),
                    code_signature: None, // TODO: Implement code signature checking
                }
            } else {
                let access_type = if event.is_modify() {
                    FileAccessType::Write
                } else {
                    FileAccessType::Read
                };
                
                SecurityEventType::FileAccess {
                    target_path: path.clone(),
                    access_type,
                }
            };
            
            let security_event = SecurityEvent {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                event_type,
                process_info: monitor_process_info,
                verdict: Verdict::Log, // Already decided in make_decision
                policy_reason: "Fanotify event".to_string(),
            };
            
            event_handler(security_event);
        }
    }
    
    fn calculate_file_hash(path: &Path, cache: &Arc<Mutex<HashMap<PathBuf, String>>>) -> Option<String> {
        // Check cache first
        if let Ok(cache) = cache.lock() {
            if let Some(hash) = cache.get(path) {
                return Some(hash.clone());
            }
        }
        
        // Calculate hash
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut hasher = Sha256::new();
            let mut buffer = vec![0; 8192];
            
            loop {
                match file.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => hasher.update(&buffer[..n]),
                    Err(_) => return None,
                }
            }
            
            let hash = format!("{:x}", hasher.finalize());
            
            // Cache the result
            if let Ok(mut cache) = cache.lock() {
                cache.insert(path.to_path_buf(), hash.clone());
            }
            
            Some(hash)
        } else {
            None
        }
    }
    
    fn start_netlink_thread(&self) {
        let netlink = Arc::clone(&self.netlink);
        let process_monitor = Arc::clone(&self.process_monitor);
        let policy = Arc::clone(&self.policy);
        let running = Arc::clone(&self.running);
        let event_handler = Arc::clone(&self.event_handler);
        
        thread::spawn(move || {
            info!("Network monitoring thread started");
            
            while *running.lock().unwrap() {
                let nm = match netlink.lock() {
                    Ok(nm) => nm,
                    Err(_) => {
                        error!("Failed to lock netlink monitor");
                        break;
                    }
                };
                
                match nm.get_tcp_connections() {
                    Ok(connections) => {
                        drop(nm); // Release lock
                        
                        for conn in connections {
                            Self::handle_network_connection(
                                &conn,
                                &process_monitor,
                                &policy,
                                &event_handler,
                            );
                        }
                    }
                    Err(e) => {
                        error!("Error getting network connections: {}", e);
                    }
                }
                
                thread::sleep(Duration::from_secs(1));
            }
            
            info!("Network monitoring thread stopped");
        });
    }
    
    fn handle_network_connection(
        conn: &NetworkConnection,
        process_monitor: &Arc<Mutex<ProcessMonitor>>,
        policy: &Arc<RwLock<SecurityPolicy>>,
        event_handler: &Arc<dyn Fn(SecurityEvent) + Send + Sync>,
    ) {
        let process_info = process_monitor
            .lock()
            .ok()
            .and_then(|pm| pm.find_process_by_inode(conn.inode).cloned());
        
        let monitor_process_info = if let Some(info) = process_info {
            MonitorProcessInfo {
                pid: info.pid,
                path: info.exe_path.unwrap_or_else(|| PathBuf::from(&info.name)),
                parent_pid: Some(info.ppid),
                user_id: info.uid,
                executable_hash: None,
                command_line: Some(info.cmdline.join(" ")),
            }
        } else {
            MonitorProcessInfo {
                pid: 0,
                path: PathBuf::from("unknown"),
                parent_pid: None,
                user_id: conn.uid,
                executable_hash: None,
                command_line: None,
            }
        };
        
        // Check policy
        let policy = match policy.read() {
            Ok(p) => p,
            Err(_) => return,
        };
        
        let remote_ip = conn.remote_addr.to_string();
        let verdict = if policy.denied_ips.contains(&remote_ip) {
            Verdict::Deny
        } else if policy.denied_ports.contains(&conn.remote_port) {
            Verdict::Deny
        } else if policy.allowed_ips.contains(&remote_ip) || 
                  policy.allowed_ports.contains(&conn.remote_port) {
            Verdict::Allow
        } else {
            Verdict::Log
        };
        
        let security_event = SecurityEvent {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: SecurityEventType::NetworkConnection {
                remote_ip,
                remote_port: conn.remote_port,
                domain: None, // TODO: DNS resolution
                protocol: NetworkProtocol::Tcp,
            },
            process_info: monitor_process_info,
            verdict,
            policy_reason: "Network policy".to_string(),
        };
        
        event_handler(security_event);
    }
    
    fn start_process_scanning_thread(&self) {
        let process_monitor = Arc::clone(&self.process_monitor);
        let policy = Arc::clone(&self.policy);
        let running = Arc::clone(&self.running);
        let event_handler = Arc::clone(&self.event_handler);
        
        thread::spawn(move || {
            info!("Process scanning thread started");
            
            while *running.lock().unwrap() {
                if let Ok(mut pm) = process_monitor.lock() {
                    if let Err(e) = pm.refresh_processes() {
                        error!("Error refreshing process list: {}", e);
                    }
                }
                
                thread::sleep(Duration::from_secs(5));
            }
            
            info!("Process scanning thread stopped");
        });
    }
    
    pub fn stop(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            if !*running {
                return Ok(());
            }
            *running = false;
        }
        
        info!("Stopping enhanced security monitoring");
        
        // Stop all monitors
        if let Ok(mut fm) = self.fanotify.lock() {
            fm.stop()?;
        }
        
        if let Ok(mut nm) = self.netlink.lock() {
            nm.stop()?;
        }
        
        if let Ok(mut pm) = self.process_monitor.lock() {
            pm.stop()?;
        }
        
        Ok(())
    }
    
    // Policy management methods
    pub fn update_policy<F>(&self, update_fn: F) -> Result<()>
    where
        F: FnOnce(&mut SecurityPolicy)
    {
        let mut policy = self.policy.write()
            .map_err(|_| anyhow!("Failed to acquire policy write lock"))?;
        update_fn(&mut policy);
        Ok(())
    }
    
    pub fn set_enforcement_mode(&self, mode: EnforcementMode) -> Result<()> {
        self.update_policy(|p| p.enforcement_mode = mode)
    }
    
    pub fn add_allowed_path(&self, path: PathBuf) -> Result<()> {
        self.update_policy(|p| { p.allowed_paths.insert(path); })
    }
    
    pub fn add_denied_path(&self, path: PathBuf) -> Result<()> {
        self.update_policy(|p| { p.denied_paths.insert(path); })
    }
    
    pub fn add_allowed_hash(&self, hash: String) -> Result<()> {
        self.update_policy(|p| { p.allowed_hashes.insert(hash); })
    }
    
    pub fn add_denied_hash(&self, hash: String) -> Result<()> {
        self.update_policy(|p| { p.denied_hashes.insert(hash); })
    }
    
    pub fn add_allowed_executable(&self, path: PathBuf) -> Result<()> {
        self.update_policy(|p| { p.allowed_executables.insert(path); })
    }
    
    pub fn add_denied_executable(&self, path: PathBuf) -> Result<()> {
        self.update_policy(|p| { p.denied_executables.insert(path); })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_policy_creation() {
        let policy = SecurityPolicy {
            allowed_paths: HashSet::new(),
            denied_paths: HashSet::new(),
            allowed_hashes: HashSet::new(),
            denied_hashes: HashSet::new(),
            allowed_executables: HashSet::new(),
            denied_executables: HashSet::new(),
            allowed_ips: HashSet::new(),
            denied_ips: HashSet::new(),
            allowed_ports: HashSet::new(),
            denied_ports: HashSet::new(),
            suspicious_patterns: Vec::new(),
            enforcement_mode: EnforcementMode::Passive,
            log_allowed: false,
            log_denied: true,
        };
        
        assert_eq!(policy.enforcement_mode, EnforcementMode::Passive);
    }
}