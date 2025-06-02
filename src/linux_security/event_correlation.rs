use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};

use crate::monitor::{SecurityEvent, SecurityEventType, ProcessInfo, Verdict};

// Event correlation engine for detecting complex attack patterns
// by analyzing relationships between multiple security events

#[derive(Debug, Clone)]
pub struct CorrelationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pattern: CorrelationPattern,
    pub time_window: Duration,
    pub severity: Severity,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum CorrelationPattern {
    // Sequential events from same process
    ProcessSequence {
        events: Vec<EventMatcher>,
        max_time_between: Duration,
    },
    
    // Multiple events within time window
    EventCluster {
        event_type: EventMatcher,
        min_count: usize,
        unique_sources: bool,
    },
    
    // Events across process tree
    ProcessTreePattern {
        root_event: EventMatcher,
        child_events: Vec<EventMatcher>,
    },
    
    // Network patterns
    NetworkSweep {
        min_targets: usize,
        port_range: Option<(u16, u16)>,
    },
    
    // File access patterns
    MassFileAccess {
        path_pattern: String,
        min_files: usize,
        access_types: Vec<FileAccessType>,
    },
    
    // Combined patterns
    KillChain {
        stages: Vec<KillChainStage>,
    },
}

#[derive(Debug, Clone)]
pub struct EventMatcher {
    pub event_type: EventTypePattern,
    pub process_name: Option<String>,
    pub path_pattern: Option<String>,
    pub network_pattern: Option<NetworkPattern>,
}

#[derive(Debug, Clone)]
pub enum EventTypePattern {
    FileExecution,
    FileAccess,
    NetworkConnection,
    ProcessSpawn,
    PrivilegeEscalation,
    Any,
}

#[derive(Debug, Clone)]
pub struct NetworkPattern {
    pub port: Option<u16>,
    pub ip_pattern: Option<String>,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KillChainStage {
    pub name: String,
    pub events: Vec<EventMatcher>,
    pub time_limit: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy)]
pub enum FileAccessType {
    Read,
    Write,
    Execute,
}

pub struct EventCorrelator {
    rules: Arc<RwLock<Vec<CorrelationRule>>>,
    event_buffer: Arc<RwLock<EventBuffer>>,
    correlations: Arc<RwLock<HashMap<String, ActiveCorrelation>>>,
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

struct EventBuffer {
    events: VecDeque<(Instant, SecurityEvent)>,
    max_age: Duration,
    max_size: usize,
}

struct ActiveCorrelation {
    rule_id: String,
    matched_events: Vec<SecurityEvent>,
    started_at: Instant,
    stage: usize,
}

pub struct RateLimiter {
    buckets: HashMap<String, TokenBucket>,
    config: RateLimiterConfig,
}

pub struct RateLimiterConfig {
    pub default_rate: u32,
    pub default_burst: u32,
    pub cleanup_interval: Duration,
}

struct TokenBucket {
    tokens: f64,
    last_update: Instant,
    rate: f64,
    capacity: f64,
}

#[derive(Debug, Clone)]
pub struct CorrelatedEvent {
    pub id: String,
    pub rule: CorrelationRule,
    pub events: Vec<SecurityEvent>,
    pub detected_at: Instant,
    pub severity: Severity,
    pub description: String,
}

impl EventCorrelator {
    pub fn new() -> Result<Self> {
        let mut correlator = Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            event_buffer: Arc::new(RwLock::new(EventBuffer {
                events: VecDeque::new(),
                max_age: Duration::from_secs(600), // 10 minutes
                max_size: 10000,
            })),
            correlations: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(RateLimiterConfig {
                default_rate: 100,
                default_burst: 200,
                cleanup_interval: Duration::from_secs(60),
            }))),
        };
        
        correlator.load_default_rules()?;
        Ok(correlator)
    }
    
    fn load_default_rules(&mut self) -> Result<()> {
        let rules = vec![
            // Reconnaissance followed by exploitation
            CorrelationRule {
                id: "recon_exploit".to_string(),
                name: "Reconnaissance to Exploitation".to_string(),
                description: "Detects reconnaissance followed by exploitation attempts".to_string(),
                pattern: CorrelationPattern::KillChain {
                    stages: vec![
                        KillChainStage {
                            name: "Reconnaissance".to_string(),
                            events: vec![
                                EventMatcher {
                                    event_type: EventTypePattern::FileExecution,
                                    process_name: Some("nmap".to_string()),
                                    path_pattern: None,
                                    network_pattern: None,
                                },
                                EventMatcher {
                                    event_type: EventTypePattern::FileAccess,
                                    process_name: None,
                                    path_pattern: Some("/etc/passwd".to_string()),
                                    network_pattern: None,
                                },
                            ],
                            time_limit: Duration::from_secs(300),
                        },
                        KillChainStage {
                            name: "Exploitation".to_string(),
                            events: vec![
                                EventMatcher {
                                    event_type: EventTypePattern::NetworkConnection,
                                    process_name: None,
                                    path_pattern: None,
                                    network_pattern: Some(NetworkPattern {
                                        port: Some(4444),
                                        ip_pattern: None,
                                        protocol: Some("tcp".to_string()),
                                    }),
                                },
                            ],
                            time_limit: Duration::from_secs(600),
                        },
                    ],
                },
                time_window: Duration::from_secs(900),
                severity: Severity::Critical,
                enabled: true,
            },
            
            // Rapid file access (ransomware pattern)
            CorrelationRule {
                id: "ransomware_pattern".to_string(),
                name: "Ransomware File Access Pattern".to_string(),
                description: "Detects rapid file access patterns typical of ransomware".to_string(),
                pattern: CorrelationPattern::MassFileAccess {
                    path_pattern: "/home/".to_string(),
                    min_files: 100,
                    access_types: vec![FileAccessType::Read, FileAccessType::Write],
                },
                time_window: Duration::from_secs(60),
                severity: Severity::Critical,
                enabled: true,
            },
            
            // Port scanning
            CorrelationRule {
                id: "port_scan".to_string(),
                name: "Port Scanning Activity".to_string(),
                description: "Detects port scanning behavior".to_string(),
                pattern: CorrelationPattern::NetworkSweep {
                    min_targets: 10,
                    port_range: Some((1, 65535)),
                },
                time_window: Duration::from_secs(30),
                severity: Severity::High,
                enabled: true,
            },
            
            // Process injection chain
            CorrelationRule {
                id: "process_injection".to_string(),
                name: "Process Injection Chain".to_string(),
                description: "Detects process injection attempts".to_string(),
                pattern: CorrelationPattern::ProcessSequence {
                    events: vec![
                        EventMatcher {
                            event_type: EventTypePattern::FileAccess,
                            process_name: None,
                            path_pattern: Some("/proc/*/mem".to_string()),
                            network_pattern: None,
                        },
                        EventMatcher {
                            event_type: EventTypePattern::ProcessSpawn,
                            process_name: None,
                            path_pattern: None,
                            network_pattern: None,
                        },
                    ],
                    max_time_between: Duration::from_secs(5),
                },
                time_window: Duration::from_secs(60),
                severity: Severity::High,
                enabled: true,
            },
            
            // Brute force detection
            CorrelationRule {
                id: "brute_force".to_string(),
                name: "Brute Force Attack".to_string(),
                description: "Detects repeated failed authentication attempts".to_string(),
                pattern: CorrelationPattern::EventCluster {
                    event_type: EventMatcher {
                        event_type: EventTypePattern::NetworkConnection,
                        process_name: Some("sshd".to_string()),
                        path_pattern: None,
                        network_pattern: Some(NetworkPattern {
                            port: Some(22),
                            ip_pattern: None,
                            protocol: Some("tcp".to_string()),
                        }),
                    },
                    min_count: 10,
                    unique_sources: false,
                },
                time_window: Duration::from_secs(60),
                severity: Severity::High,
                enabled: true,
            },
        ];
        
        let mut rules_guard = self.rules.write()
            .map_err(|_| anyhow!("Failed to acquire rules write lock"))?;
        
        for rule in rules {
            rules_guard.push(rule);
        }
        
        info!("Loaded {} correlation rules", rules_guard.len());
        Ok(())
    }
    
    pub fn process_event(&self, event: SecurityEvent) -> Option<CorrelatedEvent> {
        let now = Instant::now();
        
        // Add to buffer
        if let Ok(mut buffer) = self.event_buffer.write() {
            buffer.add_event(now, event.clone());
        }
        
        // Check rate limit
        let key = format!("{}:{}", event.process_info.pid, event.event_type.type_name());
        if !self.check_rate_limit(&key) {
            debug!("Event rate limited: {}", key);
            return None;
        }
        
        // Check correlation rules
        let rules = match self.rules.read() {
            Ok(r) => r,
            Err(_) => return None,
        };
        
        for rule in rules.iter() {
            if !rule.enabled {
                continue;
            }
            
            if let Some(correlated) = self.check_correlation(rule, &event, now) {
                return Some(correlated);
            }
        }
        
        None
    }
    
    fn check_correlation(&self, rule: &CorrelationRule, event: &SecurityEvent, now: Instant) -> Option<CorrelatedEvent> {
        match &rule.pattern {
            CorrelationPattern::ProcessSequence { events, max_time_between } => {
                self.check_process_sequence(rule, event, events, max_time_between, now)
            }
            CorrelationPattern::EventCluster { event_type, min_count, unique_sources } => {
                self.check_event_cluster(rule, event, event_type, *min_count, *unique_sources, now)
            }
            CorrelationPattern::NetworkSweep { min_targets, port_range } => {
                self.check_network_sweep(rule, event, *min_targets, port_range, now)
            }
            CorrelationPattern::MassFileAccess { path_pattern, min_files, access_types } => {
                self.check_mass_file_access(rule, event, path_pattern, *min_files, access_types, now)
            }
            CorrelationPattern::KillChain { stages } => {
                self.check_kill_chain(rule, event, stages, now)
            }
            _ => None,
        }
    }
    
    fn check_process_sequence(
        &self,
        rule: &CorrelationRule,
        event: &SecurityEvent,
        expected_events: &[EventMatcher],
        max_time_between: &Duration,
        now: Instant,
    ) -> Option<CorrelatedEvent> {
        let mut correlations = match self.correlations.write() {
            Ok(c) => c,
            Err(_) => return None,
        };
        
        let correlation_key = format!("{}:{}", rule.id, event.process_info.pid);
        
        if let Some(active) = correlations.get_mut(&correlation_key) {
            // Check if current event matches next expected
            if active.stage < expected_events.len() {
                let expected = &expected_events[active.stage];
                if self.event_matches(event, expected) {
                    active.matched_events.push(event.clone());
                    active.stage += 1;
                    
                    // Check if sequence complete
                    if active.stage >= expected_events.len() {
                        let correlated = CorrelatedEvent {
                            id: uuid::Uuid::new_v4().to_string(),
                            rule: rule.clone(),
                            events: active.matched_events.clone(),
                            detected_at: now,
                            severity: rule.severity,
                            description: format!("Process sequence detected: {}", rule.name),
                        };
                        
                        correlations.remove(&correlation_key);
                        return Some(correlated);
                    }
                }
            }
        } else if self.event_matches(event, &expected_events[0]) {
            // Start new correlation
            correlations.insert(correlation_key, ActiveCorrelation {
                rule_id: rule.id.clone(),
                matched_events: vec![event.clone()],
                started_at: now,
                stage: 1,
            });
        }
        
        None
    }
    
    fn check_event_cluster(
        &self,
        rule: &CorrelationRule,
        event: &SecurityEvent,
        event_matcher: &EventMatcher,
        min_count: usize,
        unique_sources: bool,
        now: Instant,
    ) -> Option<CorrelatedEvent> {
        if !self.event_matches(event, event_matcher) {
            return None;
        }
        
        let buffer = match self.event_buffer.read() {
            Ok(b) => b,
            Err(_) => return None,
        };
        
        let cutoff = now - rule.time_window;
        let mut matching_events = Vec::new();
        let mut sources = std::collections::HashSet::new();
        
        for (timestamp, buffered_event) in buffer.events.iter().rev() {
            if *timestamp < cutoff {
                break;
            }
            
            if self.event_matches(buffered_event, event_matcher) {
                if unique_sources {
                    sources.insert(buffered_event.process_info.pid);
                }
                matching_events.push(buffered_event.clone());
            }
        }
        
        let count = if unique_sources { sources.len() } else { matching_events.len() };
        
        if count >= min_count {
            Some(CorrelatedEvent {
                id: uuid::Uuid::new_v4().to_string(),
                rule: rule.clone(),
                events: matching_events,
                detected_at: now,
                severity: rule.severity,
                description: format!("Event cluster detected: {} (count: {})", rule.name, count),
            })
        } else {
            None
        }
    }
    
    fn check_network_sweep(
        &self,
        rule: &CorrelationRule,
        event: &SecurityEvent,
        min_targets: usize,
        port_range: &Option<(u16, u16)>,
        now: Instant,
    ) -> Option<CorrelatedEvent> {
        // Only process network connection events
        let (remote_ip, remote_port) = match &event.event_type {
            SecurityEventType::NetworkConnection { remote_ip, remote_port, .. } => {
                (remote_ip, *remote_port)
            }
            _ => return None,
        };
        
        // Check port range if specified
        if let Some((min_port, max_port)) = port_range {
            if remote_port < *min_port || remote_port > *max_port {
                return None;
            }
        }
        
        let buffer = match self.event_buffer.read() {
            Ok(b) => b,
            Err(_) => return None,
        };
        
        let cutoff = now - rule.time_window;
        let mut targets = std::collections::HashSet::new();
        let mut matching_events = Vec::new();
        
        for (timestamp, buffered_event) in buffer.events.iter().rev() {
            if *timestamp < cutoff {
                break;
            }
            
            if buffered_event.process_info.pid == event.process_info.pid {
                if let SecurityEventType::NetworkConnection { remote_ip, .. } = &buffered_event.event_type {
                    targets.insert(remote_ip.clone());
                    matching_events.push(buffered_event.clone());
                }
            }
        }
        
        if targets.len() >= min_targets {
            Some(CorrelatedEvent {
                id: uuid::Uuid::new_v4().to_string(),
                rule: rule.clone(),
                events: matching_events,
                detected_at: now,
                severity: rule.severity,
                description: format!("Network sweep detected: {} targets scanned", targets.len()),
            })
        } else {
            None
        }
    }
    
    fn check_mass_file_access(
        &self,
        rule: &CorrelationRule,
        event: &SecurityEvent,
        path_pattern: &str,
        min_files: usize,
        access_types: &[FileAccessType],
        now: Instant,
    ) -> Option<CorrelatedEvent> {
        // Only process file access events
        let (target_path, access_type) = match &event.event_type {
            SecurityEventType::FileAccess { target_path, access_type } => {
                (target_path, access_type)
            }
            _ => return None,
        };
        
        // Check if path matches pattern
        if !target_path.to_string_lossy().contains(path_pattern) {
            return None;
        }
        
        // Check access type
        let matches_type = access_types.iter().any(|t| match (t, access_type) {
            (FileAccessType::Read, crate::monitor::FileAccessType::Read) => true,
            (FileAccessType::Write, crate::monitor::FileAccessType::Write) => true,
            (FileAccessType::Execute, crate::monitor::FileAccessType::Execute) => true,
            _ => false,
        });
        
        if !matches_type {
            return None;
        }
        
        let buffer = match self.event_buffer.read() {
            Ok(b) => b,
            Err(_) => return None,
        };
        
        let cutoff = now - rule.time_window;
        let mut files_accessed = std::collections::HashSet::new();
        let mut matching_events = Vec::new();
        
        for (timestamp, buffered_event) in buffer.events.iter().rev() {
            if *timestamp < cutoff {
                break;
            }
            
            if buffered_event.process_info.pid == event.process_info.pid {
                if let SecurityEventType::FileAccess { target_path, .. } = &buffered_event.event_type {
                    if target_path.to_string_lossy().contains(path_pattern) {
                        files_accessed.insert(target_path.clone());
                        matching_events.push(buffered_event.clone());
                    }
                }
            }
        }
        
        if files_accessed.len() >= min_files {
            Some(CorrelatedEvent {
                id: uuid::Uuid::new_v4().to_string(),
                rule: rule.clone(),
                events: matching_events,
                detected_at: now,
                severity: rule.severity,
                description: format!("Mass file access detected: {} files accessed", files_accessed.len()),
            })
        } else {
            None
        }
    }
    
    fn check_kill_chain(
        &self,
        rule: &CorrelationRule,
        event: &SecurityEvent,
        stages: &[KillChainStage],
        now: Instant,
    ) -> Option<CorrelatedEvent> {
        // Complex kill chain detection would be implemented here
        // For now, return None
        None
    }
    
    fn event_matches(&self, event: &SecurityEvent, matcher: &EventMatcher) -> bool {
        // Check event type
        let type_matches = match (&matcher.event_type, &event.event_type) {
            (EventTypePattern::FileExecution, SecurityEventType::FileExecution { .. }) => true,
            (EventTypePattern::FileAccess, SecurityEventType::FileAccess { .. }) => true,
            (EventTypePattern::NetworkConnection, SecurityEventType::NetworkConnection { .. }) => true,
            (EventTypePattern::Any, _) => true,
            _ => false,
        };
        
        if !type_matches {
            return false;
        }
        
        // Check process name
        if let Some(expected_name) = &matcher.process_name {
            let process_name = event.process_info.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            if !process_name.contains(expected_name) {
                return false;
            }
        }
        
        // Check path pattern
        if let Some(path_pattern) = &matcher.path_pattern {
            let matches = match &event.event_type {
                SecurityEventType::FileExecution { target_path, .. } |
                SecurityEventType::FileAccess { target_path, .. } => {
                    target_path.to_string_lossy().contains(path_pattern)
                }
                _ => false,
            };
            
            if !matches {
                return false;
            }
        }
        
        true
    }
    
    fn check_rate_limit(&self, key: &str) -> bool {
        let mut limiter = match self.rate_limiter.write() {
            Ok(l) => l,
            Err(_) => return true, // Allow on error
        };
        
        limiter.check_and_update(key)
    }
}

impl EventBuffer {
    fn add_event(&mut self, timestamp: Instant, event: SecurityEvent) {
        // Remove old events
        let cutoff = timestamp - self.max_age;
        while let Some((ts, _)) = self.events.front() {
            if *ts < cutoff {
                self.events.pop_front();
            } else {
                break;
            }
        }
        
        // Add new event
        self.events.push_back((timestamp, event));
        
        // Enforce size limit
        while self.events.len() > self.max_size {
            self.events.pop_front();
        }
    }
}

impl RateLimiter {
    fn new(config: RateLimiterConfig) -> Self {
        Self {
            buckets: HashMap::new(),
            config,
        }
    }
    
    fn check_and_update(&mut self, key: &str) -> bool {
        let now = Instant::now();
        
        let bucket = self.buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket {
                tokens: self.config.default_burst as f64,
                last_update: now,
                rate: self.config.default_rate as f64,
                capacity: self.config.default_burst as f64,
            }
        });
        
        // Update tokens
        let elapsed = now.duration_since(bucket.last_update).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * bucket.rate).min(bucket.capacity);
        bucket.last_update = now;
        
        // Check if we can consume a token
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
    
    fn cleanup_old_buckets(&mut self) {
        let now = Instant::now();
        let cutoff = self.config.cleanup_interval;
        
        self.buckets.retain(|_, bucket| {
            now.duration_since(bucket.last_update) < cutoff
        });
    }
}

impl SecurityEventType {
    fn type_name(&self) -> &'static str {
        match self {
            SecurityEventType::FileExecution { .. } => "file_execution",
            SecurityEventType::FileAccess { .. } => "file_access",
            SecurityEventType::NetworkConnection { .. } => "network_connection",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(RateLimiterConfig {
            default_rate: 10,
            default_burst: 20,
            cleanup_interval: Duration::from_secs(60),
        });
        
        // Should allow burst
        for _ in 0..20 {
            assert!(limiter.check_and_update("test_key"));
        }
        
        // Should be rate limited
        assert!(!limiter.check_and_update("test_key"));
    }
}