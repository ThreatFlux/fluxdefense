use std::path::PathBuf;
use clap::{Arg, Command};
use tracing::{info, error};
use anyhow::Result;
use fluxdefense::scanner::FileScanner;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("file-scanner")
        .version("1.0.0")
        .author("FluxDefense Team")
        .about("Scans filesystem to build initial whitelist database")
        .arg(
            Arg::new("paths")
                .help("Paths to scan (can specify multiple)")
                .required(true)
                .num_args(1..)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("data-dir")
                .long("data-dir")
                .short('d')
                .help("Directory to store scan results")
                .default_value("./whitelist-data")
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("max-depth")
                .long("max-depth")
                .help("Maximum directory depth to scan")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("system-scan")
                .long("system-scan")
                .help("Scan common macOS system directories")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("applications-scan")
                .long("applications-scan")
                .help("Scan /Applications directory")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("homebrew-scan")
                .long("homebrew-scan")
                .help("Scan Homebrew installation directories")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();
    
    let data_dir = matches.get_one::<PathBuf>("data-dir").unwrap().clone();
    let max_depth = matches.get_one::<usize>("max-depth").copied();
    
    info!("Starting FluxDefense File Scanner");
    info!("Data directory: {:?}", data_dir);
    
    let mut scanner = FileScanner::new(data_dir)?;
    
    // Scan specified paths
    if let Some(paths) = matches.get_many::<PathBuf>("paths") {
        for path in paths {
            if path.exists() {
                info!("Scanning path: {:?}", path);
                if let Err(e) = scanner.scan_directory(path, max_depth) {
                    error!("Failed to scan {:?}: {}", path, e);
                }
            } else {
                error!("Path does not exist: {:?}", path);
            }
        }
    }
    
    // System scan
    if matches.get_flag("system-scan") {
        info!("Performing system scan...");
        let system_paths = get_system_scan_paths();
        for path in system_paths {
            if path.exists() {
                info!("Scanning system path: {:?}", path);
                if let Err(e) = scanner.scan_directory(&path, Some(3)) {
                    error!("Failed to scan system path {:?}: {}", path, e);
                }
            }
        }
    }
    
    // Applications scan
    if matches.get_flag("applications-scan") {
        info!("Scanning Applications directory...");
        let apps_path = PathBuf::from("/Applications");
        if apps_path.exists() {
            if let Err(e) = scanner.scan_directory(&apps_path, None) {
                error!("Failed to scan Applications: {}", e);
            }
        }
    }
    
    // Homebrew scan
    if matches.get_flag("homebrew-scan") {
        info!("Scanning Homebrew directories...");
        let homebrew_paths = get_homebrew_scan_paths();
        for path in homebrew_paths {
            if path.exists() {
                info!("Scanning Homebrew path: {:?}", path);
                if let Err(e) = scanner.scan_directory(&path, Some(2)) {
                    error!("Failed to scan Homebrew path {:?}: {}", path, e);
                }
            }
        }
    }
    
    // Save the manifest
    scanner.save_manifest()?;
    
    // Print scan statistics
    info!("Scan completed!");
    println!("\n{}", scanner.get_scan_stats());
    
    Ok(())
}

fn get_system_scan_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/System/Library/Frameworks"),
        PathBuf::from("/System/Library/PrivateFrameworks"),
        PathBuf::from("/System/Library/CoreServices"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/usr/libexec"),
        PathBuf::from("/Library/Apple"),
        PathBuf::from("/Library/Application Support"),
        PathBuf::from("/Library/LaunchAgents"),
        PathBuf::from("/Library/LaunchDaemons"),
        PathBuf::from("/System/Applications"),
    ]
}

fn get_homebrew_scan_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/opt/homebrew/sbin"), 
        PathBuf::from("/opt/homebrew/lib"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/local/sbin"),
        PathBuf::from("/usr/local/lib"),
    ]
}