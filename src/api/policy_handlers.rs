use axum::{
    extract::{Query, State, Path},
    http::StatusCode,
    response::Json,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::api::models::ApiResponse;
use crate::api::handlers::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub policy_type: PolicyType,
    pub rules: Vec<PolicyRule>,
    pub actions: Vec<PolicyAction>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyType {
    FileAccess,
    ProcessExecution,
    NetworkConnection,
    DnsFiltering,
    BehaviorAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub condition: RuleCondition,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleCondition {
    PathMatches,
    ProcessNameMatches,
    CommandLineContains,
    NetworkDestination,
    DomainMatches,
    BehaviorPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyAction {
    Allow,
    Block,
    Alert,
    Quarantine,
    Terminate,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub source: String,
    pub policy_id: Option<String>,
    pub event_id: Option<String>,
    pub status: AlertStatus,
    pub assigned_to: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub notes: Vec<AlertNote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    New,
    Acknowledged,
    InProgress,
    Resolved,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNote {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStats {
    pub total_policies: u32,
    pub active_policies: u32,
    pub total_alerts: u32,
    pub unresolved_alerts: u32,
    pub critical_alerts: u32,
    pub recent_violations: Vec<PolicyViolation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub timestamp: DateTime<Utc>,
    pub policy_name: String,
    pub violation_type: String,
    pub details: String,
    pub action_taken: String,
}

// Policy management endpoints

pub async fn get_policies(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<SecurityPolicy>>> {
    // In a real implementation, policies would be stored in a database
    let policies = vec![
        SecurityPolicy {
            id: "pol_1".to_string(),
            name: "Block Crypto Miners".to_string(),
            description: "Prevent cryptocurrency mining software from running".to_string(),
            enabled: true,
            policy_type: PolicyType::ProcessExecution,
            rules: vec![
                PolicyRule {
                    id: "rule_1".to_string(),
                    condition: RuleCondition::ProcessNameMatches,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("pattern".to_string(), serde_json::json!("xmrig|minerd|cgminer"));
                        params
                    },
                },
                PolicyRule {
                    id: "rule_2".to_string(),
                    condition: RuleCondition::CommandLineContains,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("keywords".to_string(), serde_json::json!(["pool", "stratum", "mining"]));
                        params
                    },
                },
            ],
            actions: vec![PolicyAction::Block, PolicyAction::Alert, PolicyAction::Terminate],
            created_at: Utc::now() - chrono::Duration::days(30),
            updated_at: Utc::now() - chrono::Duration::days(5),
        },
        SecurityPolicy {
            id: "pol_2".to_string(),
            name: "Protect Sensitive Files".to_string(),
            description: "Monitor and control access to sensitive system files".to_string(),
            enabled: true,
            policy_type: PolicyType::FileAccess,
            rules: vec![
                PolicyRule {
                    id: "rule_3".to_string(),
                    condition: RuleCondition::PathMatches,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("paths".to_string(), serde_json::json!([
                            "/etc/passwd",
                            "/etc/shadow",
                            "/etc/sudoers",
                            "/root/.ssh/*"
                        ]));
                        params
                    },
                },
            ],
            actions: vec![PolicyAction::Alert, PolicyAction::Log],
            created_at: Utc::now() - chrono::Duration::days(60),
            updated_at: Utc::now() - chrono::Duration::days(10),
        },
        SecurityPolicy {
            id: "pol_3".to_string(),
            name: "Block Malicious Domains".to_string(),
            description: "Prevent connections to known malicious domains".to_string(),
            enabled: true,
            policy_type: PolicyType::DnsFiltering,
            rules: vec![
                PolicyRule {
                    id: "rule_4".to_string(),
                    condition: RuleCondition::DomainMatches,
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("blacklist".to_string(), serde_json::json!([
                            "malware.com",
                            "phishing-site.net",
                            "*.malicious.org"
                        ]));
                        params.insert("use_threat_intel".to_string(), serde_json::json!(true));
                        params
                    },
                },
            ],
            actions: vec![PolicyAction::Block, PolicyAction::Alert],
            created_at: Utc::now() - chrono::Duration::days(45),
            updated_at: Utc::now() - chrono::Duration::days(1),
        },
    ];
    
    Json(ApiResponse::success(policies))
}

pub async fn get_policy(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<SecurityPolicy>>, StatusCode> {
    // Mock implementation
    if id == "pol_1" {
        let policy = SecurityPolicy {
            id: "pol_1".to_string(),
            name: "Block Crypto Miners".to_string(),
            description: "Prevent cryptocurrency mining software from running".to_string(),
            enabled: true,
            policy_type: PolicyType::ProcessExecution,
            rules: vec![],
            actions: vec![PolicyAction::Block, PolicyAction::Alert],
            created_at: Utc::now() - chrono::Duration::days(30),
            updated_at: Utc::now() - chrono::Duration::days(5),
        };
        Ok(Json(ApiResponse::success(policy)))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn create_policy(
    State(state): State<Arc<AppState>>,
    Json(policy): Json<SecurityPolicy>,
) -> Json<ApiResponse<SecurityPolicy>> {
    let mut new_policy = policy;
    new_policy.id = format!("pol_{}", Uuid::new_v4());
    new_policy.created_at = Utc::now();
    new_policy.updated_at = Utc::now();
    
    Json(ApiResponse::success(new_policy))
}

pub async fn update_policy(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(policy): Json<SecurityPolicy>,
) -> Json<ApiResponse<SecurityPolicy>> {
    let mut updated_policy = policy;
    updated_policy.id = id;
    updated_policy.updated_at = Utc::now();
    
    Json(ApiResponse::success(updated_policy))
}

pub async fn delete_policy(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<()>> {
    Json(ApiResponse::success(()))
}

// Alert management endpoints

pub async fn get_alerts(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<Alert>>> {
    let alerts = vec![
        Alert {
            id: "alert_1".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(2),
            severity: AlertSeverity::High,
            title: "Crypto Miner Detected".to_string(),
            description: "XMRig cryptocurrency miner detected and blocked".to_string(),
            source: "process_monitor".to_string(),
            policy_id: Some("pol_1".to_string()),
            event_id: Some("evt_123".to_string()),
            status: AlertStatus::New,
            assigned_to: None,
            resolved_at: None,
            notes: vec![],
        },
        Alert {
            id: "alert_2".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(5),
            severity: AlertSeverity::Medium,
            title: "Suspicious DNS Query".to_string(),
            description: "Blocked connection to known malware C2 server".to_string(),
            source: "dns_filter".to_string(),
            policy_id: Some("pol_3".to_string()),
            event_id: Some("evt_456".to_string()),
            status: AlertStatus::Acknowledged,
            assigned_to: Some("admin".to_string()),
            resolved_at: None,
            notes: vec![
                AlertNote {
                    id: "note_1".to_string(),
                    timestamp: Utc::now() - chrono::Duration::hours(4),
                    author: "admin".to_string(),
                    content: "Investigating the source process".to_string(),
                },
            ],
        },
    ];
    
    Json(ApiResponse::success(alerts))
}

pub async fn get_alert(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Alert>>, StatusCode> {
    // Mock implementation
    if id == "alert_1" {
        let alert = Alert {
            id: "alert_1".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(2),
            severity: AlertSeverity::High,
            title: "Crypto Miner Detected".to_string(),
            description: "XMRig cryptocurrency miner detected and blocked".to_string(),
            source: "process_monitor".to_string(),
            policy_id: Some("pol_1".to_string()),
            event_id: Some("evt_123".to_string()),
            status: AlertStatus::New,
            assigned_to: None,
            resolved_at: None,
            notes: vec![],
        };
        Ok(Json(ApiResponse::success(alert)))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

pub async fn update_alert_status(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(status_update): Json<HashMap<String, String>>,
) -> Json<ApiResponse<Alert>> {
    // Mock implementation
    let mut alert = Alert {
        id: id.clone(),
        timestamp: Utc::now() - chrono::Duration::hours(2),
        severity: AlertSeverity::High,
        title: "Crypto Miner Detected".to_string(),
        description: "XMRig cryptocurrency miner detected and blocked".to_string(),
        source: "process_monitor".to_string(),
        policy_id: Some("pol_1".to_string()),
        event_id: Some("evt_123".to_string()),
        status: AlertStatus::New,
        assigned_to: None,
        resolved_at: None,
        notes: vec![],
    };
    
    if let Some(new_status) = status_update.get("status") {
        alert.status = match new_status.as_str() {
            "acknowledged" => AlertStatus::Acknowledged,
            "in_progress" => AlertStatus::InProgress,
            "resolved" => {
                alert.resolved_at = Some(Utc::now());
                AlertStatus::Resolved
            }
            "dismissed" => AlertStatus::Dismissed,
            _ => AlertStatus::New,
        };
    }
    
    if let Some(assigned_to) = status_update.get("assigned_to") {
        alert.assigned_to = Some(assigned_to.clone());
    }
    
    Json(ApiResponse::success(alert))
}

pub async fn add_alert_note(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(note_content): Json<HashMap<String, String>>,
) -> Json<ApiResponse<AlertNote>> {
    let note = AlertNote {
        id: Uuid::new_v4().to_string(),
        timestamp: Utc::now(),
        author: note_content.get("author").unwrap_or(&"system".to_string()).clone(),
        content: note_content.get("content").unwrap_or(&"".to_string()).clone(),
    };
    
    Json(ApiResponse::success(note))
}

pub async fn get_policy_stats(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<PolicyStats>> {
    let stats = PolicyStats {
        total_policies: 5,
        active_policies: 4,
        total_alerts: 47,
        unresolved_alerts: 12,
        critical_alerts: 2,
        recent_violations: vec![
            PolicyViolation {
                timestamp: Utc::now() - chrono::Duration::minutes(30),
                policy_name: "Block Crypto Miners".to_string(),
                violation_type: "Process Execution".to_string(),
                details: "Attempted to run xmrig".to_string(),
                action_taken: "Blocked and terminated".to_string(),
            },
            PolicyViolation {
                timestamp: Utc::now() - chrono::Duration::hours(2),
                policy_name: "Block Malicious Domains".to_string(),
                violation_type: "DNS Query".to_string(),
                details: "Query to malware-c2.com".to_string(),
                action_taken: "Blocked".to_string(),
            },
        ],
    };
    
    Json(ApiResponse::success(stats))
}