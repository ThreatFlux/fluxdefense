use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use anyhow::Result;
use tracing::{info, warn, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePolicy {
    pub allowed_paths: HashSet<PathBuf>,
    pub allowed_hashes: HashSet<String>,
    pub allowed_signers: HashSet<String>,
    pub trusted_directories: HashSet<PathBuf>,
    pub system_paths: HashSet<PathBuf>,
}

impl Default for FilePolicy {
    fn default() -> Self {
        let mut policy = Self {
            allowed_paths: HashSet::new(),
            allowed_hashes: HashSet::new(),
            allowed_signers: HashSet::new(),
            trusted_directories: HashSet::new(),
            system_paths: HashSet::new(),
        };
        
        // Add default macOS system paths
        policy.add_system_paths();
        policy
    }
}

impl FilePolicy {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn add_system_paths(&mut self) {
        // Core macOS system directories that should generally be trusted
        let system_paths = vec![
            "/System",
            "/usr/bin",
            "/usr/sbin",
            "/usr/libexec",
            "/Library/Apple",
            "/Applications/Utilities",
        ];
        
        for path in system_paths {
            self.system_paths.insert(PathBuf::from(path));
        }
        
        // Common trusted application directories
        let trusted_dirs = vec![
            "/Applications",
            "/Library/Application Support",
            "/usr/local/bin",
        ];
        
        for dir in trusted_dirs {
            self.trusted_directories.insert(PathBuf::from(dir));
        }
    }
    
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        // Check exact path matches
        if self.allowed_paths.contains(path) {
            debug!("Path allowed by exact match: {:?}", path);
            return true;
        }
        
        // Check if path is in a trusted directory
        for trusted_dir in &self.trusted_directories {
            if path.starts_with(trusted_dir) {
                debug!("Path allowed by trusted directory: {:?}", path);
                return true;
            }
        }
        
        // Check if path is in system directories
        for system_path in &self.system_paths {
            if path.starts_with(system_path) {
                debug!("Path allowed by system path: {:?}", path);
                return true;
            }
        }
        
        warn!("Path not allowed: {:?}", path);
        false
    }
    
    pub fn is_hash_allowed(&self, hash: &str) -> bool {
        if self.allowed_hashes.contains(hash) {
            debug!("Hash allowed: {}", hash);
            return true;
        }
        
        warn!("Hash not allowed: {}", hash);
        false
    }
    
    pub fn is_signer_allowed(&self, signer: &str) -> bool {
        if self.allowed_signers.contains(signer) {
            debug!("Signer allowed: {}", signer);
            return true;
        }
        
        // Check for Apple signatures (always trusted)
        if signer.contains("Apple") || signer.contains("Developer ID Application") {
            debug!("Apple/Developer ID signer allowed: {}", signer);
            return true;
        }
        
        warn!("Signer not allowed: {}", signer);
        false
    }
    
    pub fn is_execution_allowed(
        &self,
        path: &Path,
        hash: Option<&str>,
        signer: Option<&str>,
    ) -> bool {
        // If path is allowed, allow execution
        if self.is_path_allowed(path) {
            return true;
        }
        
        // If hash is provided and allowed, allow execution
        if let Some(hash) = hash {
            if self.is_hash_allowed(hash) {
                return true;
            }
        }
        
        // If signer is provided and allowed, allow execution
        if let Some(signer) = signer {
            if self.is_signer_allowed(signer) {
                return true;
            }
        }
        
        false
    }
    
    pub fn add_allowed_path(&mut self, path: PathBuf) {
        info!("Adding allowed path: {:?}", path);
        self.allowed_paths.insert(path);
    }
    
    pub fn add_allowed_hash(&mut self, hash: String) {
        info!("Adding allowed hash: {}", hash);
        self.allowed_hashes.insert(hash);
    }
    
    pub fn add_allowed_signer(&mut self, signer: String) {
        info!("Adding allowed signer: {}", signer);
        self.allowed_signers.insert(signer);
    }
    
    pub fn add_trusted_directory(&mut self, directory: PathBuf) {
        info!("Adding trusted directory: {:?}", directory);
        self.trusted_directories.insert(directory);
    }
    
    pub fn remove_allowed_path(&mut self, path: &Path) {
        self.allowed_paths.remove(path);
    }
    
    pub fn remove_allowed_hash(&mut self, hash: &str) {
        self.allowed_hashes.remove(hash);
    }
    
    pub fn remove_allowed_signer(&mut self, signer: &str) {
        self.allowed_signers.remove(signer);
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        info!("File policy saved to: {:?}", path);
        Ok(())
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let policy: Self = serde_json::from_str(&content)?;
        info!("File policy loaded from: {:?}", path);
        Ok(policy)
    }
}