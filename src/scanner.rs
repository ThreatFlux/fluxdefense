use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, Context};
use tracing::{info, warn, debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub uuid: String,
    pub path: PathBuf,
    pub size: u64,
    pub modified: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub sha256_hash: String,
    pub file_type: FileType,
    pub permissions: u32,
    pub is_executable: bool,
    pub is_signed: bool,
    pub code_signature: Option<CodeSignature>,
    pub bundle_info: Option<BundleInfo>,
    pub scan_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Executable,
    Library,
    Application,
    Bundle,
    Script,
    Document,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSignature {
    pub authority: String,
    pub team_identifier: Option<String>,
    pub bundle_identifier: Option<String>,
    pub is_valid: bool,
    pub is_apple_signed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleInfo {
    pub bundle_identifier: Option<String>,
    pub version: Option<String>,
    pub display_name: Option<String>,
    pub executable_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanManifest {
    pub scan_id: String,
    pub scan_timestamp: DateTime<Utc>,
    pub total_files_scanned: usize,
    pub files_by_type: HashMap<String, usize>,
    pub scan_paths: Vec<PathBuf>,
    pub file_records: HashMap<String, String>, // UUID -> relative path to JSON file
}

pub struct FileScanner {
    data_dir: PathBuf,
    manifest: ScanManifest,
}

impl FileScanner {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;
        
        let scan_id = Uuid::new_v4().to_string();
        let manifest = ScanManifest {
            scan_id,
            scan_timestamp: Utc::now(),
            total_files_scanned: 0,
            files_by_type: HashMap::new(),
            scan_paths: Vec::new(),
            file_records: HashMap::new(),
        };
        
        Ok(Self { data_dir, manifest })
    }
    
    pub fn scan_directory(&mut self, path: &Path, max_depth: Option<usize>) -> Result<()> {
        info!("Starting scan of directory: {:?}", path);
        self.manifest.scan_paths.push(path.to_path_buf());
        
        let walker = if let Some(depth) = max_depth {
            WalkDir::new(path).max_depth(depth)
        } else {
            WalkDir::new(path)
        };
        
        for entry in walker {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        if let Err(e) = self.scan_file(entry.path()) {
                            warn!("Failed to scan file {:?}: {}", entry.path(), e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error walking directory: {}", e);
                }
            }
        }
        
        info!("Completed scan of directory: {:?}", path);
        Ok(())
    }
    
    pub fn scan_file(&mut self, path: &Path) -> Result<()> {
        debug!("Scanning file: {:?}", path);
        
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to get metadata for {:?}", path))?;
        
        // Skip symbolic links and very large files
        if metadata.is_symlink() || metadata.len() > 1_000_000_000 {
            debug!("Skipping file: {:?} (symlink or too large)", path);
            return Ok(());
        }
        
        let file_record = self.create_file_record(path, &metadata)?;
        self.save_file_record(&file_record)?;
        
        // Update manifest
        self.manifest.total_files_scanned += 1;
        let file_type_str = format!("{:?}", file_record.file_type);
        *self.manifest.files_by_type.entry(file_type_str).or_insert(0) += 1;
        
        if self.manifest.total_files_scanned % 1000 == 0 {
            info!("Scanned {} files...", self.manifest.total_files_scanned);
        }
        
        Ok(())
    }
    
    fn create_file_record(&self, path: &Path, metadata: &fs::Metadata) -> Result<FileRecord> {
        let uuid = Uuid::new_v4().to_string();
        
        // Calculate file hash
        let hash = self.calculate_file_hash(path)?;
        
        // Determine file type
        let file_type = self.determine_file_type(path);
        
        // Check if executable
        let is_executable = self.is_executable(path, metadata);
        
        // Get code signature info (macOS specific)
        let (is_signed, code_signature) = self.get_code_signature_info(path);
        
        // Get bundle info if applicable
        let bundle_info = self.get_bundle_info(path);
        
        let record = FileRecord {
            uuid,
            path: path.to_path_buf(),
            size: metadata.len(),
            modified: DateTime::from(metadata.modified()?),
            created: DateTime::from(metadata.created().unwrap_or_else(|_| metadata.modified().unwrap())),
            sha256_hash: hash,
            file_type,
            permissions: self.get_permissions(metadata),
            is_executable,
            is_signed,
            code_signature,
            bundle_info,
            scan_timestamp: Utc::now(),
        };
        
        Ok(record)
    }
    
    fn calculate_file_hash(&self, path: &Path) -> Result<String> {
        let contents = fs::read(path)
            .with_context(|| format!("Failed to read file for hashing: {:?}", path))?;
        
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        
        Ok(hex::encode(result))
    }
    
    fn determine_file_type(&self, path: &Path) -> FileType {
        let extension = path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "app" => FileType::Application,
            "bundle" | "framework" | "plugin" => FileType::Bundle,
            "dylib" | "so" | "a" => FileType::Library,
            "sh" | "zsh" | "bash" | "py" | "rb" | "pl" => FileType::Script,
            "pdf" | "doc" | "txt" | "md" => FileType::Document,
            "" => {
                // Check if it's an executable binary
                if self.is_likely_executable(path) {
                    FileType::Executable
                } else {
                    FileType::Other("unknown".to_string())
                }
            }
            _ => FileType::Other(extension),
        }
    }
    
    fn is_likely_executable(&self, path: &Path) -> bool {
        // Check for common executable locations
        let path_str = path.to_string_lossy().to_lowercase();
        path_str.contains("/bin/") || 
        path_str.contains("/sbin/") ||
        path_str.contains("/usr/") ||
        path_str.contains(".app/contents/macos/")
    }
    
    fn is_executable(&self, path: &Path, metadata: &fs::Metadata) -> bool {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        (mode & 0o111) != 0 || self.is_likely_executable(path)
    }
    
    fn get_permissions(&self, metadata: &fs::Metadata) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }
    
    fn get_code_signature_info(&self, path: &Path) -> (bool, Option<CodeSignature>) {
        // Use macOS codesign command to check signature
        let output = std::process::Command::new("codesign")
            .args(&["-dv", "--verbose=4"])
            .arg(path)
            .output();
        
        match output {
            Ok(output) if output.status.success() => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                self.parse_codesign_output(&stderr)
            }
            _ => (false, None)
        }
    }
    
    fn parse_codesign_output(&self, output: &str) -> (bool, Option<CodeSignature>) {
        if output.contains("code object is not signed") {
            return (false, None);
        }
        
        let mut authority = String::new();
        let mut team_identifier = None;
        let mut bundle_identifier = None;
        let is_apple_signed = output.contains("Apple") || output.contains("Software Signing");
        
        for line in output.lines() {
            if line.contains("Authority=") {
                authority = line.split("Authority=").nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string();
            } else if line.contains("TeamIdentifier=") {
                team_identifier = Some(line.split("TeamIdentifier=").nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string());
            } else if line.contains("Identifier=") {
                bundle_identifier = Some(line.split("Identifier=").nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string());
            }
        }
        
        let signature = CodeSignature {
            authority,
            team_identifier,
            bundle_identifier,
            is_valid: true,
            is_apple_signed,
        };
        
        (true, Some(signature))
    }
    
    fn get_bundle_info(&self, path: &Path) -> Option<BundleInfo> {
        // Check if this is part of an app bundle
        let path_str = path.to_string_lossy();
        if let Some(app_start) = path_str.find(".app/") {
            let app_path = &path_str[..app_start + 4];
            let info_plist_path = format!("{}/Contents/Info.plist", app_path);
            
            if let Ok(plist_content) = fs::read_to_string(&info_plist_path) {
                return self.parse_info_plist(&plist_content);
            }
        }
        
        None
    }
    
    fn parse_info_plist(&self, content: &str) -> Option<BundleInfo> {
        // Simple plist parsing (in production, use a proper plist parser)
        let mut bundle_identifier = None;
        let mut version = None;
        let mut display_name = None;
        let mut executable_name = None;
        
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains("CFBundleIdentifier") && i + 1 < lines.len() {
                bundle_identifier = self.extract_plist_string_value(lines[i + 1]);
            } else if line.contains("CFBundleShortVersionString") && i + 1 < lines.len() {
                version = self.extract_plist_string_value(lines[i + 1]);
            } else if line.contains("CFBundleDisplayName") && i + 1 < lines.len() {
                display_name = self.extract_plist_string_value(lines[i + 1]);
            } else if line.contains("CFBundleExecutable") && i + 1 < lines.len() {
                executable_name = self.extract_plist_string_value(lines[i + 1]);
            }
        }
        
        if bundle_identifier.is_some() || version.is_some() || display_name.is_some() {
            Some(BundleInfo {
                bundle_identifier,
                version,
                display_name,
                executable_name,
            })
        } else {
            None
        }
    }
    
    fn extract_plist_string_value(&self, line: &str) -> Option<String> {
        if let (Some(start), Some(end)) = (line.find("<string>"), line.find("</string>")) {
            let value = &line[start + 8..end];
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
        None
    }
    
    fn save_file_record(&mut self, record: &FileRecord) -> Result<()> {
        let filename = format!("{}.json", record.uuid);
        let filepath = self.data_dir.join(&filename);
        
        let json = serde_json::to_string_pretty(record)?;
        fs::write(&filepath, json)?;
        
        // Add to manifest
        self.manifest.file_records.insert(
            record.uuid.clone(),
            filename,
        );
        
        Ok(())
    }
    
    pub fn save_manifest(&self) -> Result<()> {
        let manifest_path = self.data_dir.join("scan_manifest.json");
        let json = serde_json::to_string_pretty(&self.manifest)?;
        fs::write(manifest_path, json)?;
        info!("Scan manifest saved with {} files", self.manifest.total_files_scanned);
        Ok(())
    }
    
    pub fn load_manifest(data_dir: &Path) -> Result<ScanManifest> {
        let manifest_path = data_dir.join("scan_manifest.json");
        let content = fs::read_to_string(manifest_path)?;
        let manifest: ScanManifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }
    
    pub fn get_scan_stats(&self) -> String {
        format!(
            "Scan Statistics:\n\
             Total Files: {}\n\
             File Types: {:#?}\n\
             Scan Paths: {:#?}",
            self.manifest.total_files_scanned,
            self.manifest.files_by_type,
            self.manifest.scan_paths
        )
    }
}