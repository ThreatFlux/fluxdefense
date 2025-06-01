use std::path::PathBuf;
use clap::{Arg, Command};
use tracing::{info, error};
use anyhow::Result;
use fluxdefense::{FluxDefense, config::Config};
use fluxdefense::monitor::{Verdict, ProcessInfo, NetworkProtocol};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let matches = Command::new("flux-monitor")
        .version("1.0.0")
        .author("FluxDefense Team")
        .about("FluxDefense passive monitoring and testing tool")
        .subcommand(
            Command::new("start")
                .about("Start passive monitoring mode")
                .arg(
                    Arg::new("whitelist-dir")
                        .long("whitelist-dir")
                        .help("Directory containing whitelist data")
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("log-file")
                        .long("log-file")
                        .help("Path to event log file")
                        .default_value("./fluxdefense-events.log")
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("test")
                .about("Test the monitoring system with simulated events")
                .arg(
                    Arg::new("whitelist-dir")
                        .long("whitelist-dir")
                        .help("Directory containing whitelist data")
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("stats")
                .about("Show monitoring statistics")
                .arg(
                    Arg::new("log-file")
                        .long("log-file")
                        .help("Path to event log file")
                        .default_value("./fluxdefense-events.log")
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("interactive")
                .about("Interactive testing mode")
                .arg(
                    Arg::new("whitelist-dir")
                        .long("whitelist-dir")
                        .help("Directory containing whitelist data")
                        .value_parser(clap::value_parser!(PathBuf))
                )
        )
        .subcommand(
            Command::new("metrics")
                .about("Monitor system metrics in real-time")
                .arg(
                    Arg::new("interval")
                        .long("interval")
                        .short('i')
                        .help("Update interval in seconds")
                        .default_value("2")
                        .value_parser(clap::value_parser!(u64))
                )
                .arg(
                    Arg::new("duration")
                        .long("duration")
                        .short('d')
                        .help("Duration to monitor in seconds (0 = infinite)")
                        .default_value("0")
                        .value_parser(clap::value_parser!(u64))
                )
                .arg(
                    Arg::new("json")
                        .long("json")
                        .short('j')
                        .help("Output metrics in JSON format")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("once")
                        .long("once")
                        .help("Collect metrics once and exit")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .get_matches();
    
    match matches.subcommand() {
        Some(("start", sub_matches)) => {
            start_monitoring(sub_matches).await?;
        }
        Some(("test", sub_matches)) => {
            run_tests(sub_matches).await?;
        }
        Some(("stats", sub_matches)) => {
            show_statistics(sub_matches).await?;
        }
        Some(("interactive", sub_matches)) => {
            run_interactive_mode(sub_matches).await?;
        }
        Some(("metrics", sub_matches)) => {
            monitor_system_metrics(sub_matches).await?;
        }
        _ => {
            println!("No subcommand provided. Use --help for usage information.");
        }
    }
    
    Ok(())
}

async fn start_monitoring(matches: &clap::ArgMatches) -> Result<()> {
    info!("Starting FluxDefense passive monitoring...");
    
    let log_file = matches.get_one::<PathBuf>("log-file").unwrap().clone();
    
    let mut config = Config::default();
    config.log_file_path = Some(log_file);
    
    if let Some(whitelist_dir) = matches.get_one::<PathBuf>("whitelist-dir") {
        config.file_policy_path = Some(whitelist_dir.join("file_policy.json"));
        config.network_policy_path = Some(whitelist_dir.join("network_policy.json"));
    }
    
    let mut defense = FluxDefense::new_with_config(config)?;
    defense.start().await?;
    
    // Start system metrics collection in background
    let _metrics_handle = if let Some(monitor) = defense.get_monitor() {
        Some(monitor.start_system_metrics_collection())
    } else {
        None
    };
    
    info!("FluxDefense monitoring started with system metrics collection. Press Ctrl+C to stop...");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    info!("Received shutdown signal");
    defense.stop().await?;
    
    // Show final statistics
    if let Some(monitor) = defense.get_monitor() {
        let stats = monitor.get_statistics();
        println!("\nFinal Statistics:");
        println!("Total Events: {}", stats.total_events);
        println!("Events by Type: {:#?}", stats.events_by_type);
        println!("Events by Verdict: {:#?}", stats.events_by_verdict);
    }
    
    Ok(())
}

async fn run_tests(matches: &clap::ArgMatches) -> Result<()> {
    info!("Running FluxDefense monitoring tests...");
    
    let mut config = Config::default();
    config.log_file_path = Some(PathBuf::from("./test-events.log"));
    
    if let Some(whitelist_dir) = matches.get_one::<PathBuf>("whitelist-dir") {
        config.file_policy_path = Some(whitelist_dir.join("file_policy.json"));
        config.network_policy_path = Some(whitelist_dir.join("network_policy.json"));
    }
    
    let mut defense = FluxDefense::new_with_config(config)?;
    defense.start().await?;
    
    println!("Running test scenarios...\n");
    
    // Test 1: System binaries (should be allowed)
    println!("Test 1: System binaries");
    let verdict = defense.simulate_file_execution(
        PathBuf::from("/bin/zsh"),
        PathBuf::from("/usr/bin/ls")
    );
    println!("  /usr/bin/ls execution: {:?}", verdict);
    
    let verdict = defense.simulate_file_execution(
        PathBuf::from("/bin/zsh"),
        PathBuf::from("/bin/cat")
    );
    println!("  /bin/cat execution: {:?}", verdict);
    
    // Test 2: Applications (should be allowed if in whitelist)
    println!("\nTest 2: Applications");
    let verdict = defense.simulate_file_execution(
        PathBuf::from("/bin/zsh"),
        PathBuf::from("/Applications/Safari.app/Contents/MacOS/Safari")
    );
    println!("  Safari execution: {:?}", verdict);
    
    // Test 3: Unknown/suspicious files (should be logged in passive mode)
    println!("\nTest 3: Unknown files");
    let verdict = defense.simulate_file_execution(
        PathBuf::from("/bin/zsh"),
        PathBuf::from("/tmp/suspicious_binary")
    );
    println!("  /tmp/suspicious_binary execution: {:?}", verdict);
    
    // Test 4: Network connections
    println!("\nTest 4: Network connections");
    let verdict = defense.simulate_network_connection(
        "8.8.8.8".to_string(),
        443,
        Some("google.com".to_string())
    );
    println!("  Connection to google.com:443: {:?}", verdict);
    
    let verdict = defense.simulate_network_connection(
        "192.168.1.1".to_string(),
        80,
        None
    );
    println!("  Connection to 192.168.1.1:80: {:?}", verdict);
    
    let verdict = defense.simulate_network_connection(
        "10.0.0.1".to_string(),
        22,
        None
    );
    println!("  Connection to 10.0.0.1:22: {:?}", verdict);
    
    // Show statistics
    if let Some(monitor) = defense.get_monitor() {
        let stats = monitor.get_statistics();
        println!("\nTest Statistics:");
        println!("Total Events: {}", stats.total_events);
        println!("Events by Type: {:#?}", stats.events_by_type);
        println!("Events by Verdict: {:#?}", stats.events_by_verdict);
        
        // Save event log
        monitor.save_event_log(&PathBuf::from("./test-event-log.json"))?;
        println!("\nDetailed event log saved to: test-event-log.json");
    }
    
    defense.stop().await?;
    Ok(())
}

async fn show_statistics(_matches: &clap::ArgMatches) -> Result<()> {
    println!("Statistics functionality not yet implemented");
    println!("Use the 'test' command to generate events and see live statistics");
    Ok(())
}

async fn run_interactive_mode(matches: &clap::ArgMatches) -> Result<()> {
    info!("Starting FluxDefense interactive mode...");
    
    let mut config = Config::default();
    config.log_file_path = Some(PathBuf::from("./interactive-events.log"));
    
    if let Some(whitelist_dir) = matches.get_one::<PathBuf>("whitelist-dir") {
        config.file_policy_path = Some(whitelist_dir.join("file_policy.json"));
        config.network_policy_path = Some(whitelist_dir.join("network_policy.json"));
    }
    
    let mut defense = FluxDefense::new_with_config(config)?;
    defense.start().await?;
    
    println!("FluxDefense Interactive Mode");
    println!("Commands:");
    println!("  exec <path>                    - Simulate file execution");
    println!("  net <ip> <port> [domain]       - Simulate network connection");
    println!("  stats                          - Show statistics");
    println!("  quit                           - Exit");
    println!();
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts.get(0) {
            Some(&"exec") => {
                if let Some(path) = parts.get(1) {
                    let verdict = defense.simulate_file_execution(
                        PathBuf::from("/bin/zsh"),
                        PathBuf::from(path)
                    );
                    println!("Execution of {}: {:?}", path, verdict);
                } else {
                    println!("Usage: exec <path>");
                }
            }
            Some(&"net") => {
                if let (Some(ip), Some(port)) = (parts.get(1), parts.get(2)) {
                    if let Ok(port_num) = port.parse::<u16>() {
                        let domain = parts.get(3).map(|s| s.to_string());
                        let verdict = defense.simulate_network_connection(
                            ip.to_string(),
                            port_num,
                            domain.clone()
                        );
                        match domain {
                            Some(d) => println!("Connection to {}:{} ({}): {:?}", ip, port_num, d, verdict),
                            None => println!("Connection to {}:{}: {:?}", ip, port_num, verdict),
                        }
                    } else {
                        println!("Invalid port number");
                    }
                } else {
                    println!("Usage: net <ip> <port> [domain]");
                }
            }
            Some(&"stats") => {
                if let Some(monitor) = defense.get_monitor() {
                    let stats = monitor.get_statistics();
                    println!("Statistics:");
                    println!("  Total Events: {}", stats.total_events);
                    println!("  Events by Type: {:#?}", stats.events_by_type);
                    println!("  Events by Verdict: {:#?}", stats.events_by_verdict);
                    println!("  Uptime: {} seconds", 
                        (chrono::Utc::now() - stats.start_time).num_seconds());
                } else {
                    println!("Monitor not available");
                }
            }
            Some(&"quit") | Some(&"exit") => {
                break;
            }
            Some(cmd) => {
                println!("Unknown command: {}", cmd);
            }
            None => {}
        }
    }
    
    println!("Shutting down...");
    defense.stop().await?;
    
    Ok(())
}

async fn monitor_system_metrics(matches: &clap::ArgMatches) -> Result<()> {
    use fluxdefense::system_metrics::SystemMetricsCollector;
    use std::time::{Duration, Instant};
    
    let interval = *matches.get_one::<u64>("interval").unwrap();
    let duration = *matches.get_one::<u64>("duration").unwrap();
    let json_output = matches.get_flag("json");
    let once_mode = matches.get_flag("once");
    
    let mut collector = SystemMetricsCollector::new();
    let start_time = Instant::now();
    
    if json_output && once_mode {
        // JSON one-shot mode for UI integration
        match collector.collect_metrics() {
            Ok(metrics) => {
                let json = serde_json::to_string_pretty(&metrics)?;
                println!("{}", json);
                return Ok(());
            }
            Err(e) => {
                error!("Failed to collect metrics: {}", e);
                return Err(e);
            }
        }
    }
    
    if !json_output {
        info!("Starting system metrics monitoring (interval: {}s, duration: {}s)", 
              interval, if duration == 0 { "infinite".to_string() } else { duration.to_string() });
        
        println!("FluxDefense System Metrics Monitor");
        println!("=================================");
        println!("Press Ctrl+C to stop...\n");
    }
    
    loop {
        // Check if we should exit based on duration
        if duration > 0 && start_time.elapsed().as_secs() >= duration {
            println!("\nMonitoring duration completed.");
            break;
        }
        
        // Collect metrics
        match collector.collect_metrics() {
            Ok(metrics) => {
                if json_output {
                    // JSON mode - just output the metrics as JSON
                    let json = serde_json::to_string_pretty(&metrics)?;
                    println!("{}", json);
                    if once_mode {
                        break;
                    }
                } else {
                    // Clear screen and print header
                    print!("\x1B[2J\x1B[H"); // ANSI clear screen and move to top
                    
                    println!("FluxDefense System Metrics - {}", 
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
                    println!("================================{}", "=".repeat(20));
                
                // CPU Information
                println!("💻 CPU Usage:");
                println!("   Current: {:.1}%", metrics.cpu_usage);
                println!("   Load Avg: {:.2}, {:.2}, {:.2} (1m, 5m, 15m)", 
                    metrics.load_average[0], metrics.load_average[1], metrics.load_average[2]);
                
                // Memory Information
                println!("\n🧠 Memory Usage:");
                println!("   Used: {:.1}% ({} / {})", 
                    metrics.memory_usage,
                    format_bytes(metrics.memory_used as f64),
                    format_bytes(metrics.memory_total as f64));
                
                // Disk I/O
                println!("\n💾 Disk I/O:");
                println!("   Read Rate:  {}/s", format_bytes(metrics.disk_read_rate));
                println!("   Write Rate: {}/s", format_bytes(metrics.disk_write_rate));
                println!("   Total Read:  {}", format_bytes(metrics.disk_read_bytes as f64));
                println!("   Total Write: {}", format_bytes(metrics.disk_write_bytes as f64));
                
                // Network I/O
                println!("\n🌐 Network I/O:");
                println!("   RX Rate: {}/s", format_bytes(metrics.network_rx_rate));
                println!("   TX Rate: {}/s", format_bytes(metrics.network_tx_rate));
                println!("   Total RX: {}", format_bytes(metrics.network_rx_bytes as f64));
                println!("   Total TX: {}", format_bytes(metrics.network_tx_bytes as f64));
                
                // System Information
                println!("\n📊 System Info:");
                println!("   Processes: {}", metrics.process_count);
                println!("   Uptime: {}s", metrics.uptime_seconds);
                
                // Get top processes
                if let Ok(processes) = collector.get_top_processes(5) {
                    println!("\n🔝 Top Processes by CPU:");
                    for (i, proc) in processes.iter().enumerate() {
                        println!("   {}. {} (PID: {}) - CPU: {:.1}%, MEM: {:.1}%", 
                            i + 1, proc.name, proc.pid, proc.cpu_percent, proc.memory_percent);
                    }
                }
                
                    println!("\n📈 Metrics collected at: {}", 
                        chrono::DateTime::from_timestamp(metrics.timestamp as i64, 0)
                            .unwrap_or_default()
                            .format("%H:%M:%S"));
                    
                    if duration > 0 {
                        let remaining = duration.saturating_sub(start_time.elapsed().as_secs());
                        println!("⏱️  Time remaining: {}s", remaining);
                    }
                    
                    println!("\nPress Ctrl+C to exit");
                }
            }
            Err(e) => {
                error!("Failed to collect metrics: {}", e);
                println!("❌ Error collecting metrics: {}", e);
            }
        }
        
        // Wait for next interval or check for Ctrl+C
        let sleep_duration = Duration::from_secs(interval);
        tokio::select! {
            _ = tokio::time::sleep(sleep_duration) => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\n\nReceived interrupt signal. Exiting...");
                break;
            }
        }
    }
    
    println!("System metrics monitoring stopped.");
    Ok(())
}

fn format_bytes(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{:.0} {}", size, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}