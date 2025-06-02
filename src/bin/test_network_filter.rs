use anyhow::Result;
use clap::{Parser, Subcommand};
use fluxdefense::linux_security::{
    NetworkFilter, NetworkFilterRule, NetworkEvent, FilterAction, Direction, Protocol,
    IptablesManager, IptablesRule, Chain, RuleAction,
};
use fluxdefense::linux_security::network_filter::{IpMatcher, PortMatcher};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about = "Test FluxDefense Network Filtering", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start packet capture and filtering
    Capture {
        /// Network interface to capture on (default: any)
        #[arg(short, long)]
        interface: Option<String>,
        
        /// Enable filtering (not just monitoring)
        #[arg(short, long)]
        filter: bool,
        
        /// Enable DNS filtering
        #[arg(short, long)]
        dns_filter: bool,
        
        /// Duration to capture in seconds (0 for infinite)
        #[arg(short = 't', long, default_value = "0")]
        duration: u64,
    },
    
    /// Manage iptables rules
    Iptables {
        #[command(subcommand)]
        action: IptablesAction,
    },
    
    /// Add network filter rules
    AddRule {
        /// Rule name
        #[arg(short, long)]
        name: String,
        
        /// Action: allow, block, or log
        #[arg(short, long)]
        action: String,
        
        /// Protocol: tcp, udp, icmp, or any
        #[arg(short, long)]
        protocol: Option<String>,
        
        /// Source IP address
        #[arg(short, long)]
        source_ip: Option<String>,
        
        /// Destination IP address
        #[arg(short, long)]
        dest_ip: Option<String>,
        
        /// Source port
        #[arg(short = 'S', long)]
        source_port: Option<u16>,
        
        /// Destination port
        #[arg(short = 'D', long)]
        dest_port: Option<u16>,
        
        /// Rule priority (higher = more important)
        #[arg(short = 'P', long, default_value = "100")]
        priority: i32,
    },
    
    /// Block a domain name
    BlockDomain {
        /// Domain to block
        domain: String,
    },
}

#[derive(Subcommand, Debug)]
enum IptablesAction {
    /// Initialize FluxDefense iptables chains
    Init,
    
    /// Clean up FluxDefense iptables chains
    Cleanup,
    
    /// Block an IP address
    BlockIp {
        /// IP address to block
        ip: String,
    },
    
    /// Block a port
    BlockPort {
        /// Port number to block
        port: u16,
        
        /// Protocol (tcp or udp)
        #[arg(short, long, default_value = "tcp")]
        protocol: String,
    },
    
    /// List current rules
    List {
        /// Specific chain to list
        #[arg(short, long)]
        chain: Option<String>,
    },
    
    /// Save rules to file
    Save {
        /// File path to save rules
        #[arg(short, long)]
        file: Option<String>,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let args = Args::parse();
    
    match args.command {
        Commands::Capture { interface, filter, dns_filter, duration } => {
            capture_packets(interface, filter, dns_filter, duration)?;
        }
        Commands::Iptables { action } => {
            handle_iptables(action)?;
        }
        Commands::AddRule { name, action, protocol, source_ip, dest_ip, source_port, dest_port, priority } => {
            add_filter_rule(name, action, protocol, source_ip, dest_ip, source_port, dest_port, priority)?;
        }
        Commands::BlockDomain { domain } => {
            block_domain(domain)?;
        }
    }
    
    Ok(())
}

fn capture_packets(interface: Option<String>, filtering: bool, dns_filtering: bool, duration: u64) -> Result<()> {
    // Check if running as root for packet capture
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        error!("Packet capture requires root privileges");
        warn!("Please run with sudo");
        std::process::exit(1);
    }
    
    info!("Starting network packet capture");
    
    // Create statistics counters
    let packet_count = Arc::new(AtomicUsize::new(0));
    let dns_count = Arc::new(AtomicUsize::new(0));
    let blocked_count = Arc::new(AtomicUsize::new(0));
    
    let packet_count_clone = Arc::clone(&packet_count);
    let dns_count_clone = Arc::clone(&dns_count);
    let blocked_count_clone = Arc::clone(&blocked_count);
    
    // Create network filter
    let mut filter = NetworkFilter::new(move |event: NetworkEvent| {
        match event {
            NetworkEvent::PacketCaptured { timestamp, protocol, source, destination, size, action, rule_id } => {
                packet_count_clone.fetch_add(1, Ordering::SeqCst);
                
                let action_str = match action {
                    FilterAction::Allow => "ALLOW",
                    FilterAction::Block => {
                        blocked_count_clone.fetch_add(1, Ordering::SeqCst);
                        "BLOCK"
                    }
                    FilterAction::Log => "LOG",
                };
                
                info!(
                    "[{}] {:?} {}:{} -> {}:{} ({} bytes) [{}]{}",
                    packet_count_clone.load(Ordering::SeqCst),
                    protocol,
                    source.0, source.1,
                    destination.0, destination.1,
                    size,
                    action_str,
                    rule_id.map(|id| format!(" (rule: {})", id)).unwrap_or_default()
                );
            }
            NetworkEvent::DnsQuery { domain, query_type, source, action, .. } => {
                dns_count_clone.fetch_add(1, Ordering::SeqCst);
                
                let action_str = match action {
                    FilterAction::Block => {
                        blocked_count_clone.fetch_add(1, Ordering::SeqCst);
                        "BLOCKED"
                    }
                    _ => "ALLOWED",
                };
                
                info!(
                    "[DNS] {} ({}) from {} [{}]",
                    domain, query_type, source, action_str
                );
            }
            NetworkEvent::ConnectionNew { protocol, source, destination, .. } => {
                info!(
                    "[NEW] {:?} connection: {}:{} -> {}:{}",
                    protocol,
                    source.0, source.1,
                    destination.0, destination.1
                );
            }
            NetworkEvent::ConnectionClosed { protocol, source, destination, duration, packets, bytes, .. } => {
                info!(
                    "[CLOSED] {:?} connection: {}:{} -> {}:{} (duration: {:?}, packets: {}, bytes: {})",
                    protocol,
                    source.0, source.1,
                    destination.0, destination.1,
                    duration, packets, bytes
                );
            }
            NetworkEvent::RuleMatched { rule_name, action, packet_info, .. } => {
                info!("[RULE] {} - {:?}: {}", rule_name, action, packet_info);
            }
        }
    })?;
    
    // Configure filter
    filter.set_filtering_enabled(filtering);
    filter.set_dns_filtering_enabled(dns_filtering);
    
    // Add some example rules if filtering is enabled
    if filtering {
        info!("Adding example filtering rules");
        
        // Block connections to private IP ranges from outside
        filter.add_rule(NetworkFilterRule {
            id: Uuid::new_v4().to_string(),
            name: "Block private IPs".to_string(),
            direction: Direction::Inbound,
            action: FilterAction::Block,
            protocol: Some(Protocol::Any),
            source_ip: None,
            dest_ip: Some(IpMatcher::Subnet(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)), 16)),
            source_port: None,
            dest_port: None,
            priority: 100,
            enabled: true,
        })?;
        
        // Log all HTTP/HTTPS traffic
        filter.add_rule(NetworkFilterRule {
            id: Uuid::new_v4().to_string(),
            name: "Log HTTP".to_string(),
            direction: Direction::Both,
            action: FilterAction::Log,
            protocol: Some(Protocol::Tcp),
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: Some(PortMatcher::Single(80)),
            priority: 50,
            enabled: true,
        })?;
        
        filter.add_rule(NetworkFilterRule {
            id: Uuid::new_v4().to_string(),
            name: "Log HTTPS".to_string(),
            direction: Direction::Both,
            action: FilterAction::Log,
            protocol: Some(Protocol::Tcp),
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: Some(PortMatcher::Single(443)),
            priority: 50,
            enabled: true,
        })?;
    }
    
    // Add example DNS blacklist if DNS filtering is enabled
    if dns_filtering {
        info!("Adding example DNS blacklist");
        filter.add_dns_blacklist("malware.example.com".to_string())?;
        filter.add_dns_blacklist("phishing.example.com".to_string())?;
        filter.add_dns_blacklist("ads.example.com".to_string())?;
    }
    
    // Start filter
    filter.start()?;
    
    // Start packet capture
    filter.start_capture(interface.as_deref())?;
    
    // Show status
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║            FluxDefense Network Filter Active                  ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Interface: {:^50} ║", interface.as_deref().unwrap_or("any"));
    println!("║ Filtering: {:^50} ║", if filtering { "ENABLED" } else { "DISABLED (monitoring only)" });
    println!("║ DNS Filter: {:^49} ║", if dns_filtering { "ENABLED" } else { "DISABLED" });
    println!("║                                                               ║");
    println!("║ Capturing:                                                    ║");
    println!("║ • TCP/UDP/ICMP packets                                        ║");
    println!("║ • DNS queries (port 53)                                       ║");
    println!("║ • Connection tracking                                         ║");
    println!("║                                                               ║");
    println!("║ Press Ctrl+C to stop                                          ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");
    
    // Set up Ctrl+C handler
    let running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down...");
        r.store(false, Ordering::SeqCst);
    })?;
    
    // Run for specified duration or until Ctrl+C
    if duration > 0 {
        info!("Capturing for {} seconds", duration);
        std::thread::sleep(Duration::from_secs(duration));
    } else {
        info!("Capturing until Ctrl+C");
        while running.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(100));
        }
    }
    
    // Stop filter
    filter.stop()?;
    
    // Print statistics
    let stats = filter.get_stats()?;
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                    Capture Statistics                         ║");
    println!("╠═══════════════════════════════════════════════════════════════╣");
    println!("║ Total packets captured: {:>37} ║", stats.packets_captured);
    println!("║ Packets allowed: {:>44} ║", stats.packets_allowed);
    println!("║ Packets blocked: {:>44} ║", stats.packets_blocked);
    println!("║ Packets logged: {:>45} ║", stats.packets_logged);
    println!("║ DNS queries: {:>48} ║", stats.dns_queries);
    println!("║ DNS blocked: {:>48} ║", stats.dns_blocked);
    println!("║ Connections tracked: {:>40} ║", stats.connections_tracked);
    println!("╚═══════════════════════════════════════════════════════════════╝");
    
    Ok(())
}

fn handle_iptables(action: IptablesAction) -> Result<()> {
    // Check if running as root
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        error!("iptables management requires root privileges");
        warn!("Please run with sudo");
        std::process::exit(1);
    }
    
    let manager = IptablesManager::new()?;
    
    match action {
        IptablesAction::Init => {
            info!("Initializing FluxDefense iptables chains");
            manager.initialize_chains()?;
            manager.allow_established()?;
            info!("iptables chains initialized successfully");
        }
        IptablesAction::Cleanup => {
            info!("Cleaning up FluxDefense iptables chains");
            manager.cleanup_chains()?;
            info!("iptables chains cleaned up successfully");
        }
        IptablesAction::BlockIp { ip } => {
            info!("Blocking IP address: {}", ip);
            manager.block_ip(&ip, Some("FluxDefense blocked IP"))?;
            info!("IP {} blocked successfully", ip);
        }
        IptablesAction::BlockPort { port, protocol } => {
            info!("Blocking port {} ({})", port, protocol);
            manager.block_port(port, &protocol, Some("FluxDefense blocked port"))?;
            info!("Port {} ({}) blocked successfully", port, protocol);
        }
        IptablesAction::List { chain } => {
            let chain_enum = chain.as_ref().map(|c| match c.as_str() {
                "INPUT" => Chain::Input,
                "OUTPUT" => Chain::Output,
                "FORWARD" => Chain::Forward,
                "FLUXDEFENSE_INPUT" => Chain::FluxInput,
                "FLUXDEFENSE_OUTPUT" => Chain::FluxOutput,
                "FLUXDEFENSE_FORWARD" => Chain::FluxForward,
                _ => Chain::Input,
            });
            
            let rules = manager.list_rules(chain_enum)?;
            println!("Current iptables rules:");
            for rule in rules {
                println!("{}", rule);
            }
        }
        IptablesAction::Save { file } => {
            info!("Saving iptables rules");
            manager.save_rules(file.as_deref())?;
            if let Some(f) = file {
                info!("Rules saved to {}", f);
            }
        }
    }
    
    Ok(())
}

fn add_filter_rule(
    name: String,
    action_str: String,
    protocol: Option<String>,
    source_ip: Option<String>,
    dest_ip: Option<String>,
    source_port: Option<u16>,
    dest_port: Option<u16>,
    priority: i32,
) -> Result<()> {
    info!("Adding network filter rule: {}", name);
    
    let action = match action_str.to_lowercase().as_str() {
        "allow" => FilterAction::Allow,
        "block" => FilterAction::Block,
        "log" => FilterAction::Log,
        _ => {
            error!("Invalid action: {}. Use allow, block, or log", action_str);
            std::process::exit(1);
        }
    };
    
    let proto = protocol.map(|p| match p.to_lowercase().as_str() {
        "tcp" => Protocol::Tcp,
        "udp" => Protocol::Udp,
        "icmp" => Protocol::Icmp,
        "any" => Protocol::Any,
        _ => {
            error!("Invalid protocol: {}. Use tcp, udp, icmp, or any", p);
            std::process::exit(1);
        }
    });
    
    let rule = NetworkFilterRule {
        id: Uuid::new_v4().to_string(),
        name,
        direction: Direction::Both,
        action,
        protocol: proto,
        source_ip: source_ip.map(|ip| {
            IpMatcher::Single(ip.parse().expect("Invalid source IP"))
        }),
        dest_ip: dest_ip.map(|ip| {
            IpMatcher::Single(ip.parse().expect("Invalid destination IP"))
        }),
        source_port: source_port.map(PortMatcher::Single),
        dest_port: dest_port.map(PortMatcher::Single),
        priority,
        enabled: true,
    };
    
    info!("Rule created: {:?}", rule);
    info!("Note: This rule would be added to an active filter instance");
    
    Ok(())
}

fn block_domain(domain: String) -> Result<()> {
    info!("Blocking domain: {}", domain);
    info!("Note: This would be added to an active filter's DNS blacklist");
    Ok(())
}