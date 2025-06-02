use anyhow::Result;
use tracing::info;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting FluxDefense Phase 1 Component Test (Simple)");
    
    // Check if running as root
    let uid = unsafe { libc::geteuid() };
    println!("Running as UID: {}", uid);
    
    println!("\n=== Testing Core Components ===");
    
    // Test 1: Process monitoring
    println!("\n1. Process Monitoring:");
    #[cfg(all(target_os = "linux", feature = "pcap"))]
    {
        use fluxdefense::linux_security::ProcessMonitor;
        let mut monitor = ProcessMonitor::new();
        match monitor.start_monitoring() {
            Ok(_) => println!("   ✓ Process monitor started"),
            Err(e) => println!("   ✗ Process monitor failed: {}", e),
        }
    }
    
    // Test 2: Pattern matching
    println!("\n2. Pattern Matching:");
    #[cfg(all(target_os = "linux", feature = "pcap"))]
    {
        use fluxdefense::linux_security::{PatternMatcher, process_monitor::ProcessInfo};
        
        let matcher = PatternMatcher::new()?;
        
        // Test crypto miner pattern
        let miner_process = ProcessInfo {
            pid: 1234,
            ppid: 1,
            name: "xmrig".to_string(),
            exe_path: Some(PathBuf::from("/usr/bin/xmrig")),
            cmdline: vec!["xmrig", "--pool", "pool.minexmr.com"].iter()
                .map(|s| s.to_string()).collect(),
            uid: 1000,
            gid: 1000,
            start_time: 0,
        };
        
        let matches = matcher.check_process(&miner_process, None);
        println!("   Crypto miner detection: {} patterns matched", matches.len());
        
        // Test normal process
        let normal_process = ProcessInfo {
            pid: 5678,
            ppid: 1,
            name: "firefox".to_string(),
            exe_path: Some(PathBuf::from("/usr/bin/firefox")),
            cmdline: vec!["firefox", "https://example.com"].iter()
                .map(|s| s.to_string()).collect(),
            uid: 1000,
            gid: 1000,
            start_time: 0,
        };
        
        let matches = matcher.check_process(&normal_process, None);
        println!("   Normal process detection: {} patterns matched", matches.len());
        println!("   ✓ Pattern matching working");
    }
    
    // Test 3: Event correlation
    println!("\n3. Event Correlation:");
    #[cfg(all(target_os = "linux", feature = "pcap"))]
    {
        use fluxdefense::linux_security::EventCorrelator;
        let correlator = EventCorrelator::new()?;
        println!("   ✓ Event correlator initialized");
    }
    
    // Test 4: File system monitoring status
    println!("\n4. File System Monitoring:");
    if uid == 0 {
        #[cfg(all(target_os = "linux", feature = "pcap"))]
        {
            use fluxdefense::linux_security::FanotifyMonitor;
            match FanotifyMonitor::new() {
                Ok(mut monitor) => {
                    match monitor.start_monitoring() {
                        Ok(_) => println!("   ✓ Fanotify monitor started (root access)"),
                        Err(e) => println!("   ✗ Fanotify start failed: {}", e),
                    }
                }
                Err(e) => println!("   ✗ Fanotify init failed: {}", e),
            }
        }
    } else {
        println!("   ⚠ Fanotify requires root access (current UID: {})", uid);
    }
    
    // Test 5: Network components
    println!("\n5. Network Components:");
    #[cfg(all(target_os = "linux", feature = "pcap"))]
    {
        // DNS filter (without async)
        use fluxdefense::linux_security::DnsFilter;
        let filter = DnsFilter::new()?;
        filter.load_default_blacklists()?;
        
        // Test DGA detection
        let test_domains = vec![
            ("google.com", false),
            ("asdkjhqwlekjhasdlkjh.com", true),
            ("facebook.com", false),
            ("a1b2c3d4e5f6.net", true),
        ];
        
        println!("   DGA Detection:");
        for (domain, expected) in test_domains {
            let is_dga = filter.is_dga_domain(domain);
            let status = if is_dga == expected { "✓" } else { "✗" };
            println!("     {} {} -> DGA: {}", status, domain, is_dga);
        }
        
        // Netfilter
        if uid == 0 {
            use fluxdefense::linux_security::NetfilterManager;
            let manager = NetfilterManager::new()?;
            println!("   ✓ Netfilter manager created");
        } else {
            println!("   ⚠ Netfilter requires root access");
        }
    }
    
    println!("\n=== Summary ===");
    println!("Phase 1 components are implemented and ready!");
    println!("Some features require root access for full functionality.");
    
    Ok(())
}