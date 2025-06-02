use anyhow::Result;
use clap::Parser;
use fluxdefense::linux_security::{EnhancedSecurityMonitor, EnforcementMode};
use fluxdefense::monitor::SecurityEvent;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};

#[derive(Parser, Debug)]
#[command(author, version, about = "Test FluxDefense Enhanced Security Monitor", long_about = None)]
struct Args {
    /// Set enforcement mode: passive, permissive, or enforcing
    #[arg(short, long, default_value = "passive")]
    mode: String,
    
    /// Add allowed executable paths (can be specified multiple times)
    #[arg(long)]
    allow_exe: Vec<PathBuf>,
    
    /// Add denied executable paths (can be specified multiple times)
    #[arg(long)]
    deny_exe: Vec<PathBuf>,
    
    /// Add allowed file paths (can be specified multiple times)
    #[arg(long)]
    allow_path: Vec<PathBuf>,
    
    /// Add denied file paths (can be specified multiple times)
    #[arg(long)]
    deny_path: Vec<PathBuf>,
    
    /// Run duration in seconds (0 for infinite)
    #[arg(short, long, default_value = "0")]
    duration: u64,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let args = Args::parse();
    
    // Check if running as root
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        error!("This program requires root privileges for fanotify monitoring");
        warn!("Please run with sudo");
        std::process::exit(1);
    }
    
    info!("Starting FluxDefense Enhanced Security Monitor Test");
    
    // Create event counter
    let event_counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&event_counter);
    
    // Create monitor with event handler
    let mut monitor = EnhancedSecurityMonitor::new(move |event: SecurityEvent| {
        let count = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
        
        match &event.event_type {
            fluxdefense::monitor::SecurityEventType::FileExecution { target_path, file_hash, .. } => {
                info!(
                    "[{}] FILE EXEC: {} -> {} (hash: {}) [{}]",
                    count,
                    event.process_info.path.display(),
                    target_path.display(),
                    file_hash.as_deref().unwrap_or("N/A"),
                    event.verdict.to_string()
                );
            }
            fluxdefense::monitor::SecurityEventType::FileAccess { target_path, access_type } => {
                info!(
                    "[{}] FILE ACCESS: {} -> {} ({:?}) [{}]",
                    count,
                    event.process_info.path.display(),
                    target_path.display(),
                    access_type,
                    event.verdict.to_string()
                );
            }
            fluxdefense::monitor::SecurityEventType::NetworkConnection { remote_ip, remote_port, .. } => {
                info!(
                    "[{}] NETWORK: {} -> {}:{} [{}]",
                    count,
                    event.process_info.path.display(),
                    remote_ip,
                    remote_port,
                    event.verdict.to_string()
                );
            }
        }
    })?;
    
    // Set enforcement mode
    let mode = match args.mode.as_str() {
        "passive" => EnforcementMode::Passive,
        "permissive" => EnforcementMode::Permissive,
        "enforcing" => EnforcementMode::Enforcing,
        _ => {
            error!("Invalid mode: {}. Use passive, permissive, or enforcing", args.mode);
            std::process::exit(1);
        }
    };
    
    monitor.set_enforcement_mode(mode)?;
    info!("Enforcement mode set to: {:?}", mode);
    
    // Add allowed executables
    for exe in args.allow_exe {
        monitor.add_allowed_executable(exe.clone())?;
        info!("Added allowed executable: {}", exe.display());
    }
    
    // Add denied executables
    for exe in args.deny_exe {
        monitor.add_denied_executable(exe.clone())?;
        info!("Added denied executable: {}", exe.display());
    }
    
    // Add allowed paths
    for path in args.allow_path {
        monitor.add_allowed_path(path.clone())?;
        info!("Added allowed path: {}", path.display());
    }
    
    // Add denied paths
    for path in args.deny_path {
        monitor.add_denied_path(path.clone())?;
        info!("Added denied path: {}", path.display());
    }
    
    // Start monitoring
    monitor.start()?;
    info!("Enhanced security monitoring started");
    
    // Show instructions
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║           FluxDefense Enhanced Security Monitor               ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Mode: {:^55} ║", format!("{:?}", mode));
    println!("║                                                               ║");
    println!("║ The monitor is now tracking:                                 ║");
    println!("║ • File executions (with hash calculation)                     ║");
    println!("║ • File access (read/write)                                    ║");
    println!("║ • Network connections                                         ║");
    println!("║ • Suspicious patterns (crypto miners, reverse shells, etc.)   ║");
    println!("║                                                               ║");
    println!("║ Try running commands like:                                    ║");
    println!("║ • ls, cat, wget, curl                                         ║");
    println!("║ • Open files in editors                                       ║");
    println!("║ • Make network connections                                    ║");
    println!("║                                                               ║");
    println!("║ Press Ctrl+C to stop monitoring                               ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    
    // Set up Ctrl+C handler
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;
    
    // Run for specified duration or until Ctrl+C
    if args.duration > 0 {
        info!("Running for {} seconds", args.duration);
        std::thread::sleep(Duration::from_secs(args.duration));
    } else {
        info!("Running until Ctrl+C");
        while running.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    
    // Stop monitoring
    monitor.stop()?;
    
    let total_events = event_counter.load(Ordering::SeqCst);
    info!("Monitoring stopped. Total events captured: {}", total_events);
    
    Ok(())
}

// Helper trait to display verdict
trait VerdictDisplay {
    fn to_string(&self) -> &'static str;
}

impl VerdictDisplay for fluxdefense::monitor::Verdict {
    fn to_string(&self) -> &'static str {
        match self {
            fluxdefense::monitor::Verdict::Allow => "ALLOW",
            fluxdefense::monitor::Verdict::Deny => "DENY",
            fluxdefense::monitor::Verdict::Log => "LOG",
        }
    }
}