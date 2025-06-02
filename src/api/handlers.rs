use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api::models::{
    ApiResponse, HealthCheck, SystemStatus, ThreatMetrics, NetworkMetrics,
    SecurityEvent, NetworkConnection, DnsQuery, ThreatDetection, MalwareSignature,
    LogEntry, LiveEvent, AllSettings, SecuritySettings, NetworkSettings,
    NotificationSettings, SystemSettings, EventQuery, LogQuery,
};
use crate::api::system_monitor::SystemMonitor;

pub struct AppState {
    pub system_monitor: Arc<Mutex<SystemMonitor>>,
    pub security_events: Arc<Mutex<Vec<SecurityEvent>>>,
    pub network_connections: Arc<Mutex<Vec<NetworkConnection>>>,
    pub dns_queries: Arc<Mutex<Vec<DnsQuery>>>,
    pub threat_detections: Arc<Mutex<Vec<ThreatDetection>>>,
    pub malware_signatures: Arc<Mutex<Vec<MalwareSignature>>>,
    pub log_entries: Arc<Mutex<Vec<LogEntry>>>,
    pub live_events: Arc<Mutex<Vec<LiveEvent>>>,
    pub settings: Arc<Mutex<AllSettings>>,
    pub start_time: DateTime<Utc>,
}

impl AppState {
    pub fn new() -> Self {
        let settings = AllSettings {
            security: SecuritySettings {
                enforcement_mode: "enforcing".to_string(),
                enable_file_monitoring: true,
                enable_network_filtering: true,
                enable_process_monitoring: true,
                enable_threat_detection: true,
                log_level: "info".to_string(),
                quarantine_enabled: true,
                auto_update: true,
            },
            network: NetworkSettings {
                enable_dns_filtering: true,
                enable_packet_capture: true,
                enable_iptables_integration: true,
                default_interface: "eth0".to_string(),
                capture_buffer_size: 1024,
                max_connections: 10000,
                dns_blacklist: vec!["malware.example.com".to_string(), "phishing.test.org".to_string()],
                trusted_networks: vec!["192.168.1.0/24".to_string(), "10.0.0.0/8".to_string()],
            },
            notifications: NotificationSettings {
                enable_email_notifications: false,
                enable_webhooks: false,
                critical_threshold: 5,
                email_recipients: vec![],
                webhook_url: "".to_string(),
                notification_frequency: "immediate".to_string(),
            },
            system: SystemSettings {
                max_log_retention: 30,
                enable_performance_monitoring: true,
                cpu_threshold: 80.0,
                memory_threshold: 85.0,
                disk_threshold: 90.0,
                enable_auto_backup: true,
                backup_location: "/var/backups/fluxdefense".to_string(),
            },
        };

        Self {
            system_monitor: Arc::new(Mutex::new(SystemMonitor::new())),
            security_events: Arc::new(Mutex::new(Vec::new())),
            network_connections: Arc::new(Mutex::new(Vec::new())),
            dns_queries: Arc::new(Mutex::new(Vec::new())),
            threat_detections: Arc::new(Mutex::new(Vec::new())),
            malware_signatures: Arc::new(Mutex::new(Vec::new())),
            log_entries: Arc::new(Mutex::new(Vec::new())),
            live_events: Arc::new(Mutex::new(Vec::new())),
            settings: Arc::new(Mutex::new(settings)),
            start_time: Utc::now(),
        }
    }
}

// Health Check
pub async fn health_check(State(state): State<Arc<AppState>>) -> Json<ApiResponse<HealthCheck>> {
    let uptime = (Utc::now() - state.start_time).num_seconds() as u64;
    
    let mut services = HashMap::new();
    services.insert("system_monitor".to_string(), "active".to_string());
    services.insert("security_monitor".to_string(), "active".to_string());
    services.insert("network_monitor".to_string(), "active".to_string());
    services.insert("threat_detector".to_string(), "active".to_string());

    let health = HealthCheck {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime,
        services,
    };

    Json(ApiResponse::success(health))
}

// Dashboard Overview
pub async fn get_system_status(State(state): State<Arc<AppState>>) -> Json<ApiResponse<SystemStatus>> {
    let uptime = (Utc::now() - state.start_time).num_seconds() as u64;
    let settings = state.settings.lock().unwrap();
    
    let status = SystemStatus {
        status: "secure".to_string(),
        uptime,
        active_monitors: vec![
            "file_system".to_string(),
            "network".to_string(),
            "process".to_string(),
            "threat_detection".to_string(),
        ],
        enforcement_mode: settings.security.enforcement_mode.clone(),
    };

    Json(ApiResponse::success(status))
}

pub async fn get_threat_metrics(State(state): State<Arc<AppState>>) -> Json<ApiResponse<ThreatMetrics>> {
    let threats = state.threat_detections.lock().unwrap();
    let events = state.security_events.lock().unwrap();
    
    let active_threats = threats.iter().filter(|t| t.status == "detected").count() as u32;
    let threats_blocked = threats.iter().filter(|t| t.status == "quarantined" || t.status == "removed").count() as u32;
    
    let metrics = ThreatMetrics {
        active_threats,
        threats_blocked,
        total_events: events.len() as u32,
        last_scan: Utc::now(),
    };

    Json(ApiResponse::success(metrics))
}

pub async fn get_network_metrics(State(state): State<Arc<AppState>>) -> Json<ApiResponse<NetworkMetrics>> {
    let connections = state.network_connections.lock().unwrap();
    let dns_queries = state.dns_queries.lock().unwrap();
    
    let active_connections = connections.iter().filter(|c| c.status == "active").count() as u32;
    let blocked_connections = connections.iter().filter(|c| c.status == "blocked").count() as u32;
    let dns_blocked = dns_queries.iter().filter(|q| q.status == "blocked").count() as u32;
    
    let total_bytes_in: u64 = connections.iter().map(|c| c.bytes_in).sum();
    let total_bytes_out: u64 = connections.iter().map(|c| c.bytes_out).sum();
    
    let metrics = NetworkMetrics {
        active_connections,
        blocked_connections,
        dns_queries: dns_queries.len() as u32,
        dns_blocked,
        bytes_in: total_bytes_in,
        bytes_out: total_bytes_out,
    };

    Json(ApiResponse::success(metrics))
}

// System Metrics
pub async fn get_system_metrics(State(state): State<Arc<AppState>>) -> Json<ApiResponse<crate::api::models::SystemMetrics>> {
    let mut monitor = state.system_monitor.lock().unwrap();
    let metrics = monitor.get_system_metrics();
    Json(ApiResponse::success(metrics))
}

pub async fn get_system_resources(State(state): State<Arc<AppState>>) -> Json<ApiResponse<crate::api::models::SystemResources>> {
    let mut monitor = state.system_monitor.lock().unwrap();
    let resources = monitor.get_system_resources();
    Json(ApiResponse::success(resources))
}

// Security Events
pub async fn get_security_events(
    Query(query): Query<EventQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<SecurityEvent>>> {
    let events = state.security_events.lock().unwrap();
    let mut filtered_events: Vec<SecurityEvent> = events.iter().cloned().collect();
    
    // Apply filters
    if let Some(severity) = &query.severity {
        filtered_events.retain(|e| e.severity == *severity);
    }
    if let Some(event_type) = &query.event_type {
        filtered_events.retain(|e| e.event_type == *event_type);
    }
    if let Some(since) = query.since {
        filtered_events.retain(|e| e.timestamp >= since);
    }
    if let Some(until) = query.until {
        filtered_events.retain(|e| e.timestamp <= until);
    }
    
    // Sort by timestamp (newest first)
    filtered_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Apply pagination
    let offset = query.offset.unwrap_or(0) as usize;
    let limit = query.limit.unwrap_or(50) as usize;
    
    let paginated: Vec<SecurityEvent> = filtered_events
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    Json(ApiResponse::success(paginated))
}

pub async fn get_security_event(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SecurityEvent>>, StatusCode> {
    let events = state.security_events.lock().unwrap();
    
    if let Some(event) = events.iter().find(|e| e.id == id) {
        Ok(Json(ApiResponse::success(event.clone())))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

// Network Monitoring
pub async fn get_network_connections(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<NetworkConnection>>> {
    let connections = state.network_connections.lock().unwrap();
    let mut sorted: Vec<NetworkConnection> = connections.iter().cloned().collect();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sorted.truncate(100); // Limit to recent connections
    
    Json(ApiResponse::success(sorted))
}

pub async fn get_dns_queries(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<DnsQuery>>> {
    let queries = state.dns_queries.lock().unwrap();
    let mut sorted: Vec<DnsQuery> = queries.iter().cloned().collect();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sorted.truncate(100);
    
    Json(ApiResponse::success(sorted))
}

// Threat Detection
pub async fn get_threat_detections(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<ThreatDetection>>> {
    let threats = state.threat_detections.lock().unwrap();
    let mut sorted: Vec<ThreatDetection> = threats.iter().cloned().collect();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    Json(ApiResponse::success(sorted))
}

pub async fn get_malware_signatures(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<MalwareSignature>>> {
    let signatures = state.malware_signatures.lock().unwrap();
    let sorted: Vec<MalwareSignature> = signatures.iter().cloned().collect();
    
    Json(ApiResponse::success(sorted))
}

// Event Logs
pub async fn get_event_logs(
    Query(query): Query<LogQuery>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<LogEntry>>> {
    let logs = state.log_entries.lock().unwrap();
    let mut filtered_logs: Vec<LogEntry> = logs.iter().cloned().collect();
    
    // Apply filters
    if let Some(level) = &query.level {
        filtered_logs.retain(|l| l.level == *level);
    }
    if let Some(category) = &query.category {
        filtered_logs.retain(|l| l.category == *category);
    }
    if let Some(search) = &query.search {
        let search_lower = search.to_lowercase();
        filtered_logs.retain(|l| {
            l.message.to_lowercase().contains(&search_lower) ||
            l.source.to_lowercase().contains(&search_lower) ||
            l.tags.iter().any(|tag| tag.to_lowercase().contains(&search_lower))
        });
    }
    if let Some(since) = query.since {
        filtered_logs.retain(|l| l.timestamp >= since);
    }
    if let Some(until) = query.until {
        filtered_logs.retain(|l| l.timestamp <= until);
    }
    
    // Sort by timestamp (newest first)
    filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    
    // Apply pagination
    let offset = query.offset.unwrap_or(0) as usize;
    let limit = query.limit.unwrap_or(100) as usize;
    
    let paginated: Vec<LogEntry> = filtered_logs
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();
    
    Json(ApiResponse::success(paginated))
}

// Live Events
pub async fn get_live_events(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<LiveEvent>>> {
    let events = state.live_events.lock().unwrap();
    let mut sorted: Vec<LiveEvent> = events.iter().cloned().collect();
    sorted.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    sorted.truncate(50);
    
    Json(ApiResponse::success(sorted))
}

// Settings
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<AllSettings>> {
    let settings = state.settings.lock().unwrap();
    Json(ApiResponse::success(settings.clone()))
}

pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<AllSettings>,
) -> Json<ApiResponse<AllSettings>> {
    let mut settings = state.settings.lock().unwrap();
    *settings = new_settings.clone();
    Json(ApiResponse::success(new_settings))
}

pub async fn get_security_settings(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<SecuritySettings>> {
    let settings = state.settings.lock().unwrap();
    Json(ApiResponse::success(settings.security.clone()))
}

pub async fn update_security_settings(
    State(state): State<Arc<AppState>>,
    Json(new_settings): Json<SecuritySettings>,
) -> Json<ApiResponse<SecuritySettings>> {
    let mut settings = state.settings.lock().unwrap();
    settings.security = new_settings.clone();
    Json(ApiResponse::success(new_settings))
}