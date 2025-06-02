use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Dashboard Overview Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemStatus {
    pub status: String,
    pub uptime: u64,
    pub active_monitors: Vec<String>,
    pub enforcement_mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreatMetrics {
    pub active_threats: u32,
    pub threats_blocked: u32,
    pub total_events: u32,
    pub last_scan: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkMetrics {
    pub active_connections: u32,
    pub blocked_connections: u32,
    pub dns_queries: u32,
    pub dns_blocked: u32,
    pub bytes_in: u64,
    pub bytes_out: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub load_average: Vec<f64>,
    pub uptime: u64,
}

// Security Events Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub action: String,
    pub details: HashMap<String, serde_json::Value>,
    pub user: Option<String>,
    pub pid: Option<u32>,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
}

// Network Events Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConnection {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub protocol: String,
    pub source_ip: String,
    pub source_port: u16,
    pub dest_ip: String,
    pub dest_port: u16,
    pub status: String,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub packets: u32,
    pub duration: u64,
    pub process: String,
    pub pid: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DnsQuery {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub domain: String,
    pub query_type: String,
    pub source_ip: String,
    pub status: String,
    pub response: Option<String>,
}

// Activity Monitor Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub command: String,
    pub ppid: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemResources {
    pub cpu: CpuInfo,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
    pub network: NetworkInfo,
    pub processes: Vec<ProcessInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CpuInfo {
    pub usage: f32,
    pub cores: u32,
    pub load_average: Vec<f64>,
    pub frequency: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub percent: f32,
    pub swap_total: u64,
    pub swap_used: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiskInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub percent: f32,
    pub mount_point: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInfo {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub packets_in: u64,
    pub packets_out: u64,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub packets_received: u64,
    pub packets_sent: u64,
    pub is_up: bool,
}

// Threat Detection Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThreatDetection {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub threat_type: String,
    pub severity: String,
    pub status: String,
    pub file_path: String,
    pub file_hash: String,
    pub file_size: u64,
    pub confidence: f32,
    pub description: String,
    pub source: String,
    pub recommendations: Vec<String>,
    pub signatures: Vec<String>,
    pub behavior_analysis: BehaviorAnalysis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BehaviorAnalysis {
    pub network_connections: Vec<String>,
    pub file_modifications: Vec<String>,
    pub registry_changes: Vec<String>,
    pub process_spawning: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MalwareSignature {
    pub id: String,
    pub name: String,
    pub signature_type: String,
    pub pattern: String,
    pub severity: String,
    pub last_updated: DateTime<Utc>,
    pub detection_count: u32,
}

// Event Logs Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub category: String,
    pub source: String,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub user: Option<String>,
    pub pid: Option<u32>,
    pub tags: Vec<String>,
}

// Live Monitor Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LiveEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub source: String,
    pub details: HashMap<String, serde_json::Value>,
}

// Settings Models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecuritySettings {
    pub enforcement_mode: String,
    pub enable_file_monitoring: bool,
    pub enable_network_filtering: bool,
    pub enable_process_monitoring: bool,
    pub enable_threat_detection: bool,
    pub log_level: String,
    pub quarantine_enabled: bool,
    pub auto_update: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkSettings {
    pub enable_dns_filtering: bool,
    pub enable_packet_capture: bool,
    pub enable_iptables_integration: bool,
    pub default_interface: String,
    pub capture_buffer_size: u32,
    pub max_connections: u32,
    pub dns_blacklist: Vec<String>,
    pub trusted_networks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NotificationSettings {
    pub enable_email_notifications: bool,
    pub enable_webhooks: bool,
    pub critical_threshold: u32,
    pub email_recipients: Vec<String>,
    pub webhook_url: String,
    pub notification_frequency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSettings {
    pub max_log_retention: u32,
    pub enable_performance_monitoring: bool,
    pub cpu_threshold: f32,
    pub memory_threshold: f32,
    pub disk_threshold: f32,
    pub enable_auto_backup: bool,
    pub backup_location: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllSettings {
    pub security: SecuritySettings,
    pub network: NetworkSettings,
    pub notifications: NotificationSettings,
    pub system: SystemSettings,
}

// API Response Models
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }

    pub fn error(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub version: String,
    pub uptime: u64,
    pub services: HashMap<String, String>,
}

// WebSocket Message Types
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    LiveEvent { data: LiveEvent },
    SecurityEvent { data: SecurityEvent },
    NetworkEvent { data: NetworkConnection },
    SystemMetrics { data: SystemMetrics },
    ThreatDetection { data: ThreatDetection },
    LogEntry { data: LogEntry },
    Heartbeat { timestamp: DateTime<Utc> },
}

// Query Parameters
#[derive(Debug, Deserialize)]
pub struct EventQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub severity: Option<String>,
    pub event_type: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub level: Option<String>,
    pub category: Option<String>,
    pub search: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}