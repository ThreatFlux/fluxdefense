use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EsfEventType {
    AuthExec,
    AuthOpen,
    AuthCreate,
    AuthRename,
    AuthUnlink,
    NotifyExec,
    NotifyOpen,
    NotifyCreate,
    NotifyRename,
    NotifyUnlink,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsfEvent {
    pub event_type: EsfEventType,
    pub process_path: PathBuf,
    pub target_path: Option<PathBuf>,
    pub process_id: u32,
    pub user_id: u32,
    pub timestamp: u64,
    pub process_hash: Option<String>,
    pub code_signature: Option<String>,
}

impl EsfEvent {
    pub fn new(
        event_type: EsfEventType,
        process_path: PathBuf,
        process_id: u32,
        user_id: u32,
    ) -> Self {
        Self {
            event_type,
            process_path,
            target_path: None,
            process_id,
            user_id,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            process_hash: None,
            code_signature: None,
        }
    }
    
    pub fn with_target_path(mut self, target_path: PathBuf) -> Self {
        self.target_path = Some(target_path);
        self
    }
    
    pub fn with_hash(mut self, hash: String) -> Self {
        self.process_hash = Some(hash);
        self
    }
    
    pub fn with_signature(mut self, signature: String) -> Self {
        self.code_signature = Some(signature);
        self
    }
}