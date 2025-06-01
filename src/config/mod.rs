use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::Result;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub file_policy_path: Option<PathBuf>,
    pub network_policy_path: Option<PathBuf>,
    pub log_level: String,
    pub log_file_path: Option<PathBuf>,
    pub enable_file_monitoring: bool,
    pub enable_network_monitoring: bool,
    pub quarantine_directory: PathBuf,
    pub alert_webhook_url: Option<String>,
    pub update_interval_seconds: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            file_policy_path: Some(PathBuf::from("/etc/fluxdefense/file_policy.json")),
            network_policy_path: Some(PathBuf::from("/etc/fluxdefense/network_policy.json")),
            log_level: "info".to_string(),
            log_file_path: Some(PathBuf::from("/var/log/fluxdefense.log")),
            enable_file_monitoring: true,
            enable_network_monitoring: true,
            quarantine_directory: PathBuf::from("/var/quarantine/fluxdefense"),
            alert_webhook_url: None,
            update_interval_seconds: 300, // 5 minutes
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path.exists() {
            warn!("Config file not found: {:?}, using defaults", path);
            return Ok(Self::default());
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        info!("Configuration loaded from: {:?}", path);
        Ok(config)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        info!("Configuration saved to: {:?}", path);
        Ok(())
    }
    
    pub fn get_default_config_path() -> PathBuf {
        // Try to use system config directory
        if cfg!(target_os = "macos") {
            PathBuf::from("/Library/Preferences/com.fluxdefense.config.json")
        } else {
            PathBuf::from("/etc/fluxdefense/config.json")
        }
    }
    
    pub fn get_user_config_path() -> Option<PathBuf> {
        if let Some(home) = dirs::home_dir() {
            Some(home.join(".config/fluxdefense/config.json"))
        } else {
            None
        }
    }
    
    pub fn load_config() -> Result<Self> {
        // Try user config first, then system config
        if let Some(user_config_path) = Self::get_user_config_path() {
            if user_config_path.exists() {
                info!("Loading user configuration");
                return Self::load_from_file(&user_config_path);
            }
        }
        
        let system_config_path = Self::get_default_config_path();
        if system_config_path.exists() {
            info!("Loading system configuration");
            return Self::load_from_file(&system_config_path);
        }
        
        info!("No configuration file found, using defaults");
        Ok(Self::default())
    }
    
    pub fn validate(&self) -> Result<()> {
        // Validate log level
        match self.log_level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => {
                return Err(anyhow::anyhow!("Invalid log level: {}", self.log_level));
            }
        }
        
        // Validate quarantine directory
        if let Some(parent) = self.quarantine_directory.parent() {
            if !parent.exists() {
                warn!("Quarantine directory parent does not exist: {:?}", parent);
            }
        }
        
        // Validate policy file paths
        if let Some(ref path) = self.file_policy_path {
            if !path.exists() {
                warn!("File policy path does not exist: {:?}", path);
            }
        }
        
        if let Some(ref path) = self.network_policy_path {
            if !path.exists() {
                warn!("Network policy path does not exist: {:?}", path);
            }
        }
        
        Ok(())
    }
    
    pub fn create_directories(&self) -> Result<()> {
        // Create quarantine directory
        std::fs::create_dir_all(&self.quarantine_directory)?;
        info!("Created quarantine directory: {:?}", self.quarantine_directory);
        
        // Create log file directory
        if let Some(ref log_path) = self.log_file_path {
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent)?;
                info!("Created log directory: {:?}", parent);
            }
        }
        
        // Create policy file directories
        if let Some(ref policy_path) = self.file_policy_path {
            if let Some(parent) = policy_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        if let Some(ref policy_path) = self.network_policy_path {
            if let Some(parent) = policy_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        Ok(())
    }
}