use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};

// For packet capture
use pcap::{Capture, Device, Packet, Active};

// For DNS parsing
use std::str;

// Network filtering rules
#[derive(Debug, Clone)]
pub struct NetworkFilterRule {
    pub id: String,
    pub name: String,
    pub direction: Direction,
    pub action: FilterAction,
    pub protocol: Option<Protocol>,
    pub source_ip: Option<IpMatcher>,
    pub dest_ip: Option<IpMatcher>,
    pub source_port: Option<PortMatcher>,
    pub dest_port: Option<PortMatcher>,
    pub priority: i32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Inbound,
    Outbound,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterAction {
    Allow,
    Block,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Any,
}

#[derive(Debug, Clone)]
pub enum IpMatcher {
    Single(IpAddr),
    Range(IpAddr, IpAddr),
    Subnet(IpAddr, u8), // CIDR notation
    Any,
}

#[derive(Debug, Clone)]
pub enum PortMatcher {
    Single(u16),
    Range(u16, u16),
    List(Vec<u16>),
    Any,
}

// DNS cache entry
#[derive(Debug, Clone)]
struct DnsCacheEntry {
    domain: String,
    ips: Vec<IpAddr>,
    cached_at: Instant,
    ttl: Duration,
}

// Packet statistics
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub packets_captured: u64,
    pub packets_allowed: u64,
    pub packets_blocked: u64,
    pub packets_logged: u64,
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub connections_tracked: usize,
    pub dns_queries: u64,
    pub dns_blocked: u64,
}

pub struct NetworkFilter {
    // Packet capture
    pcap_handle: Option<Arc<Mutex<Capture<Active>>>>,
    capture_interface: Option<String>,
    
    // Filtering rules
    rules: Arc<RwLock<Vec<NetworkFilterRule>>>,
    
    // DNS filtering
    dns_blacklist: Arc<RwLock<HashSet<String>>>,
    dns_whitelist: Arc<RwLock<HashSet<String>>>,
    dns_cache: Arc<Mutex<HashMap<String, DnsCacheEntry>>>,
    
    // Connection tracking
    active_connections: Arc<Mutex<HashMap<ConnectionKey, ConnectionInfo>>>,
    
    // Statistics
    stats: Arc<Mutex<NetworkStats>>,
    
    // Configuration
    capture_enabled: bool,
    filtering_enabled: bool,
    dns_filtering_enabled: bool,
    
    // Event callback
    event_handler: Arc<dyn Fn(NetworkEvent) + Send + Sync>,
    
    running: Arc<Mutex<bool>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ConnectionKey {
    protocol: u8,
    local_addr: IpAddr,
    local_port: u16,
    remote_addr: IpAddr,
    remote_port: u16,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    first_seen: Instant,
    last_seen: Instant,
    packets: u64,
    bytes: u64,
    state: ConnectionState,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    New,
    Established,
    Closing,
    Closed,
}

#[derive(Debug, Clone)]
pub enum NetworkEvent {
    PacketCaptured {
        timestamp: Instant,
        protocol: Protocol,
        source: (IpAddr, u16),
        destination: (IpAddr, u16),
        size: usize,
        action: FilterAction,
        rule_id: Option<String>,
    },
    DnsQuery {
        timestamp: Instant,
        domain: String,
        query_type: String,
        source: IpAddr,
        action: FilterAction,
    },
    ConnectionNew {
        timestamp: Instant,
        protocol: Protocol,
        source: (IpAddr, u16),
        destination: (IpAddr, u16),
    },
    ConnectionClosed {
        timestamp: Instant,
        protocol: Protocol,
        source: (IpAddr, u16),
        destination: (IpAddr, u16),
        duration: Duration,
        packets: u64,
        bytes: u64,
    },
    RuleMatched {
        timestamp: Instant,
        rule_id: String,
        rule_name: String,
        action: FilterAction,
        packet_info: String,
    },
}

impl NetworkFilter {
    pub fn new<F>(event_handler: F) -> Result<Self>
    where
        F: Fn(NetworkEvent) + Send + Sync + 'static
    {
        info!("Initializing network filter with pcap support");
        
        Ok(Self {
            pcap_handle: None,
            capture_interface: None,
            rules: Arc::new(RwLock::new(Vec::new())),
            dns_blacklist: Arc::new(RwLock::new(HashSet::new())),
            dns_whitelist: Arc::new(RwLock::new(HashSet::new())),
            dns_cache: Arc::new(Mutex::new(HashMap::new())),
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(NetworkStats::default())),
            capture_enabled: false,
            filtering_enabled: true,
            dns_filtering_enabled: true,
            event_handler: Arc::new(event_handler),
            running: Arc::new(Mutex::new(false)),
        })
    }
    
    pub fn start_capture(&mut self, interface: Option<&str>) -> Result<()> {
        if self.capture_enabled {
            return Ok(());
        }
        
        // Find capture device
        let device = if let Some(iface) = interface {
            Device::list()?
                .into_iter()
                .find(|d| d.name == iface)
                .ok_or_else(|| anyhow!("Interface {} not found", iface))?
        } else {
            // Use default device
            Device::lookup()
                .map_err(|e| anyhow!("Failed to find default device: {}", e))?
                .ok_or_else(|| anyhow!("No default device found"))?
        };
        
        info!("Starting packet capture on interface: {}", device.name);
        self.capture_interface = Some(device.name.clone());
        
        // Create capture handle
        let cap = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .timeout(100) // 100ms timeout for reads
            .open()?;
        
        // Set BPF filter to reduce overhead (optional)
        // cap.filter("tcp or udp or icmp", true)?;
        
        self.pcap_handle = Some(Arc::new(Mutex::new(cap)));
        self.capture_enabled = true;
        
        // Start capture thread
        self.start_capture_thread();
        
        Ok(())
    }
    
    fn start_capture_thread(&self) {
        let pcap_handle = match &self.pcap_handle {
            Some(handle) => Arc::clone(handle),
            None => return,
        };
        
        let rules = Arc::clone(&self.rules);
        let dns_blacklist = Arc::clone(&self.dns_blacklist);
        let dns_whitelist = Arc::clone(&self.dns_whitelist);
        let dns_cache = Arc::clone(&self.dns_cache);
        let active_connections = Arc::clone(&self.active_connections);
        let stats = Arc::clone(&self.stats);
        let event_handler = Arc::clone(&self.event_handler);
        let running = Arc::clone(&self.running);
        let filtering_enabled = self.filtering_enabled;
        let dns_filtering_enabled = self.dns_filtering_enabled;
        
        thread::spawn(move || {
            info!("Packet capture thread started");
            
            while *running.lock().unwrap() {
                let mut cap = match pcap_handle.lock() {
                    Ok(cap) => cap,
                    Err(_) => {
                        error!("Failed to lock pcap handle");
                        break;
                    }
                };
                
                match cap.next_packet() {
                    Ok(packet) => {
                        Self::process_packet(
                            &packet,
                            &rules,
                            &dns_blacklist,
                            &dns_whitelist,
                            &dns_cache,
                            &active_connections,
                            &stats,
                            &event_handler,
                            filtering_enabled,
                            dns_filtering_enabled,
                        );
                    }
                    Err(pcap::Error::TimeoutExpired) => {
                        // Normal timeout, continue
                        continue;
                    }
                    Err(e) => {
                        error!("Error capturing packet: {}", e);
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
            
            info!("Packet capture thread stopped");
        });
    }
    
    fn process_packet(
        packet: &Packet,
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        dns_blacklist: &Arc<RwLock<HashSet<String>>>,
        dns_whitelist: &Arc<RwLock<HashSet<String>>>,
        dns_cache: &Arc<Mutex<HashMap<String, DnsCacheEntry>>>,
        active_connections: &Arc<Mutex<HashMap<ConnectionKey, ConnectionInfo>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
        filtering_enabled: bool,
        dns_filtering_enabled: bool,
    ) {
        // Update stats
        if let Ok(mut stats) = stats.lock() {
            stats.packets_captured += 1;
        }
        
        // Parse packet headers
        let data = packet.data;
        if data.len() < 14 {
            return; // Too small for Ethernet header
        }
        
        // Skip Ethernet header (14 bytes)
        let ip_data = &data[14..];
        
        // Check IP version
        let ip_version = (ip_data[0] >> 4) & 0x0f;
        
        match ip_version {
            4 => Self::process_ipv4_packet(
                ip_data,
                rules,
                dns_blacklist,
                dns_whitelist,
                dns_cache,
                active_connections,
                stats,
                event_handler,
                filtering_enabled,
                dns_filtering_enabled,
            ),
            6 => Self::process_ipv6_packet(
                ip_data,
                rules,
                dns_blacklist,
                dns_whitelist,
                dns_cache,
                active_connections,
                stats,
                event_handler,
                filtering_enabled,
                dns_filtering_enabled,
            ),
            _ => {
                debug!("Unknown IP version: {}", ip_version);
            }
        }
    }
    
    fn process_ipv4_packet(
        data: &[u8],
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        dns_blacklist: &Arc<RwLock<HashSet<String>>>,
        dns_whitelist: &Arc<RwLock<HashSet<String>>>,
        dns_cache: &Arc<Mutex<HashMap<String, DnsCacheEntry>>>,
        active_connections: &Arc<Mutex<HashMap<ConnectionKey, ConnectionInfo>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
        filtering_enabled: bool,
        dns_filtering_enabled: bool,
    ) {
        if data.len() < 20 {
            return; // Too small for IPv4 header
        }
        
        // Parse IPv4 header
        let header_len = ((data[0] & 0x0f) * 4) as usize;
        if data.len() < header_len {
            return;
        }
        
        let protocol = data[9];
        let src_ip = IpAddr::V4(Ipv4Addr::new(data[12], data[13], data[14], data[15]));
        let dst_ip = IpAddr::V4(Ipv4Addr::new(data[16], data[17], data[18], data[19]));
        
        // Parse transport layer
        let transport_data = &data[header_len..];
        
        match protocol {
            6 => { // TCP
                if transport_data.len() >= 4 {
                    let src_port = u16::from_be_bytes([transport_data[0], transport_data[1]]);
                    let dst_port = u16::from_be_bytes([transport_data[2], transport_data[3]]);
                    
                    Self::process_tcp_packet(
                        src_ip, src_port, dst_ip, dst_port,
                        transport_data, data.len(),
                        rules, active_connections, stats, event_handler,
                        filtering_enabled,
                    );
                }
            }
            17 => { // UDP
                if transport_data.len() >= 8 {
                    let src_port = u16::from_be_bytes([transport_data[0], transport_data[1]]);
                    let dst_port = u16::from_be_bytes([transport_data[2], transport_data[3]]);
                    
                    // Check if it's DNS (port 53)
                    if (dst_port == 53 || src_port == 53) && dns_filtering_enabled {
                        Self::process_dns_packet(
                            &transport_data[8..], // Skip UDP header
                            src_ip,
                            dns_blacklist,
                            dns_whitelist,
                            dns_cache,
                            stats,
                            event_handler,
                        );
                    }
                    
                    Self::process_udp_packet(
                        src_ip, src_port, dst_ip, dst_port,
                        transport_data, data.len(),
                        rules, active_connections, stats, event_handler,
                        filtering_enabled,
                    );
                }
            }
            1 => { // ICMP
                Self::process_icmp_packet(
                    src_ip, dst_ip, data.len(),
                    rules, stats, event_handler,
                    filtering_enabled,
                );
            }
            _ => {
                debug!("Unknown protocol: {}", protocol);
            }
        }
    }
    
    fn process_ipv6_packet(
        data: &[u8],
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        dns_blacklist: &Arc<RwLock<HashSet<String>>>,
        dns_whitelist: &Arc<RwLock<HashSet<String>>>,
        dns_cache: &Arc<Mutex<HashMap<String, DnsCacheEntry>>>,
        active_connections: &Arc<Mutex<HashMap<ConnectionKey, ConnectionInfo>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
        filtering_enabled: bool,
        dns_filtering_enabled: bool,
    ) {
        // TODO: Implement IPv6 packet processing
        // Similar to IPv4 but with 40-byte fixed header
    }
    
    fn process_tcp_packet(
        src_ip: IpAddr,
        src_port: u16,
        dst_ip: IpAddr,
        dst_port: u16,
        tcp_data: &[u8],
        packet_size: usize,
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        active_connections: &Arc<Mutex<HashMap<ConnectionKey, ConnectionInfo>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
        filtering_enabled: bool,
    ) {
        let conn_key = ConnectionKey {
            protocol: 6, // TCP
            local_addr: src_ip,
            local_port: src_port,
            remote_addr: dst_ip,
            remote_port: dst_port,
        };
        
        // Update connection tracking
        let mut new_connection = false;
        if let Ok(mut connections) = active_connections.lock() {
            let now = Instant::now();
            
            if let Some(conn_info) = connections.get_mut(&conn_key) {
                conn_info.last_seen = now;
                conn_info.packets += 1;
                conn_info.bytes += packet_size as u64;
            } else {
                new_connection = true;
                connections.insert(conn_key.clone(), ConnectionInfo {
                    first_seen: now,
                    last_seen: now,
                    packets: 1,
                    bytes: packet_size as u64,
                    state: ConnectionState::New,
                });
            }
        }
        
        if new_connection {
            event_handler(NetworkEvent::ConnectionNew {
                timestamp: Instant::now(),
                protocol: Protocol::Tcp,
                source: (src_ip, src_port),
                destination: (dst_ip, dst_port),
            });
        }
        
        // Apply filtering rules
        if filtering_enabled {
            let action = Self::evaluate_rules(
                &rules,
                Protocol::Tcp,
                src_ip,
                src_port,
                dst_ip,
                dst_port,
            );
            
            match action {
                Some((FilterAction::Block, rule_id)) => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_blocked += 1;
                    }
                    
                    event_handler(NetworkEvent::PacketCaptured {
                        timestamp: Instant::now(),
                        protocol: Protocol::Tcp,
                        source: (src_ip, src_port),
                        destination: (dst_ip, dst_port),
                        size: packet_size,
                        action: FilterAction::Block,
                        rule_id: Some(rule_id),
                    });
                }
                Some((FilterAction::Log, rule_id)) => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_logged += 1;
                    }
                    
                    event_handler(NetworkEvent::PacketCaptured {
                        timestamp: Instant::now(),
                        protocol: Protocol::Tcp,
                        source: (src_ip, src_port),
                        destination: (dst_ip, dst_port),
                        size: packet_size,
                        action: FilterAction::Log,
                        rule_id: Some(rule_id),
                    });
                }
                _ => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_allowed += 1;
                    }
                }
            }
        }
    }
    
    fn process_udp_packet(
        src_ip: IpAddr,
        src_port: u16,
        dst_ip: IpAddr,
        dst_port: u16,
        udp_data: &[u8],
        packet_size: usize,
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        active_connections: &Arc<Mutex<HashMap<ConnectionKey, ConnectionInfo>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
        filtering_enabled: bool,
    ) {
        // Similar to TCP processing but for UDP
        let conn_key = ConnectionKey {
            protocol: 17, // UDP
            local_addr: src_ip,
            local_port: src_port,
            remote_addr: dst_ip,
            remote_port: dst_port,
        };
        
        // Update connection tracking
        let mut new_connection = false;
        if let Ok(mut connections) = active_connections.lock() {
            let now = Instant::now();
            
            if let Some(conn_info) = connections.get_mut(&conn_key) {
                conn_info.last_seen = now;
                conn_info.packets += 1;
                conn_info.bytes += packet_size as u64;
            } else {
                new_connection = true;
                connections.insert(conn_key.clone(), ConnectionInfo {
                    first_seen: now,
                    last_seen: now,
                    packets: 1,
                    bytes: packet_size as u64,
                    state: ConnectionState::New,
                });
            }
        }
        
        if new_connection {
            event_handler(NetworkEvent::ConnectionNew {
                timestamp: Instant::now(),
                protocol: Protocol::Udp,
                source: (src_ip, src_port),
                destination: (dst_ip, dst_port),
            });
        }
        
        // Apply filtering rules
        if filtering_enabled {
            let action = Self::evaluate_rules(
                &rules,
                Protocol::Udp,
                src_ip,
                src_port,
                dst_ip,
                dst_port,
            );
            
            match action {
                Some((FilterAction::Block, rule_id)) => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_blocked += 1;
                    }
                    
                    event_handler(NetworkEvent::PacketCaptured {
                        timestamp: Instant::now(),
                        protocol: Protocol::Udp,
                        source: (src_ip, src_port),
                        destination: (dst_ip, dst_port),
                        size: packet_size,
                        action: FilterAction::Block,
                        rule_id: Some(rule_id),
                    });
                }
                _ => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_allowed += 1;
                    }
                }
            }
        }
    }
    
    fn process_icmp_packet(
        src_ip: IpAddr,
        dst_ip: IpAddr,
        packet_size: usize,
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
        filtering_enabled: bool,
    ) {
        // Apply filtering rules for ICMP
        if filtering_enabled {
            let action = Self::evaluate_rules(
                &rules,
                Protocol::Icmp,
                src_ip,
                0,
                dst_ip,
                0,
            );
            
            match action {
                Some((FilterAction::Block, rule_id)) => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_blocked += 1;
                    }
                    
                    event_handler(NetworkEvent::PacketCaptured {
                        timestamp: Instant::now(),
                        protocol: Protocol::Icmp,
                        source: (src_ip, 0),
                        destination: (dst_ip, 0),
                        size: packet_size,
                        action: FilterAction::Block,
                        rule_id: Some(rule_id),
                    });
                }
                _ => {
                    if let Ok(mut stats) = stats.lock() {
                        stats.packets_allowed += 1;
                    }
                }
            }
        }
    }
    
    fn process_dns_packet(
        dns_data: &[u8],
        src_ip: IpAddr,
        dns_blacklist: &Arc<RwLock<HashSet<String>>>,
        dns_whitelist: &Arc<RwLock<HashSet<String>>>,
        dns_cache: &Arc<Mutex<HashMap<String, DnsCacheEntry>>>,
        stats: &Arc<Mutex<NetworkStats>>,
        event_handler: &Arc<dyn Fn(NetworkEvent) + Send + Sync>,
    ) {
        // Basic DNS parsing (simplified)
        if dns_data.len() < 12 {
            return; // Too small for DNS header
        }
        
        // Check if it's a query (QR bit = 0)
        let flags = u16::from_be_bytes([dns_data[2], dns_data[3]]);
        if (flags & 0x8000) != 0 {
            return; // It's a response, not a query
        }
        
        // Parse question section
        let mut offset = 12;
        let mut domain = String::new();
        
        // Read domain name
        while offset < dns_data.len() {
            let label_len = dns_data[offset] as usize;
            if label_len == 0 {
                offset += 1;
                break;
            }
            
            if !domain.is_empty() {
                domain.push('.');
            }
            
            offset += 1;
            if offset + label_len > dns_data.len() {
                return;
            }
            
            if let Ok(label) = str::from_utf8(&dns_data[offset..offset + label_len]) {
                domain.push_str(label);
            }
            
            offset += label_len;
        }
        
        if domain.is_empty() {
            return;
        }
        
        // Update stats
        if let Ok(mut stats) = stats.lock() {
            stats.dns_queries += 1;
        }
        
        // Check blacklist/whitelist
        let action = if let Ok(whitelist) = dns_whitelist.read() {
            if whitelist.contains(&domain) {
                FilterAction::Allow
            } else if let Ok(blacklist) = dns_blacklist.read() {
                if blacklist.contains(&domain) {
                    if let Ok(mut stats) = stats.lock() {
                        stats.dns_blocked += 1;
                    }
                    FilterAction::Block
                } else {
                    FilterAction::Log
                }
            } else {
                FilterAction::Log
            }
        } else {
            FilterAction::Log
        };
        
        event_handler(NetworkEvent::DnsQuery {
            timestamp: Instant::now(),
            domain: domain.clone(),
            query_type: "A".to_string(), // Simplified
            source: src_ip,
            action,
        });
        
        // Cache the domain for future reference
        if action != FilterAction::Block {
            if let Ok(mut cache) = dns_cache.lock() {
                cache.insert(domain.clone(), DnsCacheEntry {
                    domain,
                    ips: Vec::new(), // Will be populated when response is seen
                    cached_at: Instant::now(),
                    ttl: Duration::from_secs(300),
                });
            }
        }
    }
    
    fn evaluate_rules(
        rules: &Arc<RwLock<Vec<NetworkFilterRule>>>,
        protocol: Protocol,
        src_ip: IpAddr,
        src_port: u16,
        dst_ip: IpAddr,
        dst_port: u16,
    ) -> Option<(FilterAction, String)> {
        let rules = match rules.read() {
            Ok(r) => r,
            Err(_) => return None,
        };
        
        // Sort by priority and evaluate
        let mut matching_rules: Vec<_> = rules
            .iter()
            .filter(|rule| rule.enabled && Self::rule_matches(
                rule, protocol, src_ip, src_port, dst_ip, dst_port
            ))
            .collect();
        
        matching_rules.sort_by_key(|r| -r.priority); // Higher priority first
        
        if let Some(rule) = matching_rules.first() {
            Some((rule.action, rule.id.clone()))
        } else {
            None
        }
    }
    
    fn rule_matches(
        rule: &NetworkFilterRule,
        protocol: Protocol,
        src_ip: IpAddr,
        src_port: u16,
        dst_ip: IpAddr,
        dst_port: u16,
    ) -> bool {
        // Check protocol
        if let Some(rule_protocol) = rule.protocol {
            if rule_protocol != Protocol::Any && rule_protocol != protocol {
                return false;
            }
        }
        
        // Check source IP
        if let Some(ref matcher) = rule.source_ip {
            if !Self::ip_matches(matcher, src_ip) {
                return false;
            }
        }
        
        // Check destination IP
        if let Some(ref matcher) = rule.dest_ip {
            if !Self::ip_matches(matcher, dst_ip) {
                return false;
            }
        }
        
        // Check source port
        if let Some(ref matcher) = rule.source_port {
            if !Self::port_matches(matcher, src_port) {
                return false;
            }
        }
        
        // Check destination port
        if let Some(ref matcher) = rule.dest_port {
            if !Self::port_matches(matcher, dst_port) {
                return false;
            }
        }
        
        true
    }
    
    fn ip_matches(matcher: &IpMatcher, ip: IpAddr) -> bool {
        match matcher {
            IpMatcher::Single(match_ip) => ip == *match_ip,
            IpMatcher::Range(start, end) => {
                match (ip, start, end) {
                    (IpAddr::V4(ip), IpAddr::V4(start), IpAddr::V4(end)) => {
                        ip >= *start && ip <= *end
                    }
                    (IpAddr::V6(ip), IpAddr::V6(start), IpAddr::V6(end)) => {
                        ip >= *start && ip <= *end
                    }
                    _ => false,
                }
            }
            IpMatcher::Subnet(network, prefix_len) => {
                Self::ip_in_subnet(ip, *network, *prefix_len)
            }
            IpMatcher::Any => true,
        }
    }
    
    fn ip_in_subnet(ip: IpAddr, network: IpAddr, prefix_len: u8) -> bool {
        match (ip, network) {
            (IpAddr::V4(ip), IpAddr::V4(net)) => {
                let ip_bits = u32::from_be_bytes(ip.octets());
                let net_bits = u32::from_be_bytes(net.octets());
                let mask = !((1u32 << (32 - prefix_len)) - 1);
                (ip_bits & mask) == (net_bits & mask)
            }
            (IpAddr::V6(ip), IpAddr::V6(net)) => {
                let ip_bits = u128::from_be_bytes(ip.octets());
                let net_bits = u128::from_be_bytes(net.octets());
                let mask = !((1u128 << (128 - prefix_len)) - 1);
                (ip_bits & mask) == (net_bits & mask)
            }
            _ => false,
        }
    }
    
    fn port_matches(matcher: &PortMatcher, port: u16) -> bool {
        match matcher {
            PortMatcher::Single(p) => port == *p,
            PortMatcher::Range(start, end) => port >= *start && port <= *end,
            PortMatcher::List(ports) => ports.contains(&port),
            PortMatcher::Any => true,
        }
    }
    
    pub fn start(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        // Start connection cleanup thread
        self.start_cleanup_thread();
        
        info!("Network filter started");
        Ok(())
    }
    
    fn start_cleanup_thread(&self) {
        let active_connections = Arc::clone(&self.active_connections);
        let event_handler = Arc::clone(&self.event_handler);
        let running = Arc::clone(&self.running);
        
        thread::spawn(move || {
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_secs(30));
                
                // Clean up old connections
                if let Ok(mut connections) = active_connections.lock() {
                    let now = Instant::now();
                    let timeout = Duration::from_secs(300); // 5 minute timeout
                    
                    let expired: Vec<_> = connections
                        .iter()
                        .filter(|(_, info)| now.duration_since(info.last_seen) > timeout)
                        .map(|(key, info)| (key.clone(), info.clone()))
                        .collect();
                    
                    for (key, info) in expired {
                        connections.remove(&key);
                        
                        event_handler(NetworkEvent::ConnectionClosed {
                            timestamp: now,
                            protocol: match key.protocol {
                                6 => Protocol::Tcp,
                                17 => Protocol::Udp,
                                _ => Protocol::Any,
                            },
                            source: (key.local_addr, key.local_port),
                            destination: (key.remote_addr, key.remote_port),
                            duration: now.duration_since(info.first_seen),
                            packets: info.packets,
                            bytes: info.bytes,
                        });
                    }
                }
            }
        });
    }
    
    pub fn stop(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            if !*running {
                return Ok(());
            }
            *running = false;
        }
        
        self.capture_enabled = false;
        info!("Network filter stopped");
        Ok(())
    }
    
    // Rule management
    pub fn add_rule(&self, rule: NetworkFilterRule) -> Result<()> {
        let mut rules = self.rules.write()
            .map_err(|_| anyhow!("Failed to acquire rules write lock"))?;
        rules.push(rule);
        Ok(())
    }
    
    pub fn remove_rule(&self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write()
            .map_err(|_| anyhow!("Failed to acquire rules write lock"))?;
        rules.retain(|r| r.id != rule_id);
        Ok(())
    }
    
    pub fn update_rule(&self, rule_id: &str, updated_rule: NetworkFilterRule) -> Result<()> {
        let mut rules = self.rules.write()
            .map_err(|_| anyhow!("Failed to acquire rules write lock"))?;
        if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
            *rule = updated_rule;
        }
        Ok(())
    }
    
    pub fn get_rules(&self) -> Result<Vec<NetworkFilterRule>> {
        let rules = self.rules.read()
            .map_err(|_| anyhow!("Failed to acquire rules read lock"))?;
        Ok(rules.clone())
    }
    
    // DNS management
    pub fn add_dns_blacklist(&self, domain: String) -> Result<()> {
        let mut blacklist = self.dns_blacklist.write()
            .map_err(|_| anyhow!("Failed to acquire DNS blacklist write lock"))?;
        blacklist.insert(domain);
        Ok(())
    }
    
    pub fn remove_dns_blacklist(&self, domain: &str) -> Result<()> {
        let mut blacklist = self.dns_blacklist.write()
            .map_err(|_| anyhow!("Failed to acquire DNS blacklist write lock"))?;
        blacklist.remove(domain);
        Ok(())
    }
    
    pub fn add_dns_whitelist(&self, domain: String) -> Result<()> {
        let mut whitelist = self.dns_whitelist.write()
            .map_err(|_| anyhow!("Failed to acquire DNS whitelist write lock"))?;
        whitelist.insert(domain);
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<NetworkStats> {
        let stats = self.stats.lock()
            .map_err(|_| anyhow!("Failed to acquire stats lock"))?;
        Ok(stats.clone())
    }
    
    pub fn reset_stats(&self) -> Result<()> {
        let mut stats = self.stats.lock()
            .map_err(|_| anyhow!("Failed to acquire stats lock"))?;
        *stats = NetworkStats::default();
        Ok(())
    }
    
    pub fn set_filtering_enabled(&mut self, enabled: bool) {
        self.filtering_enabled = enabled;
    }
    
    pub fn set_dns_filtering_enabled(&mut self, enabled: bool) {
        self.dns_filtering_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ip_subnet_matching() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let network = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 0));
        
        assert!(NetworkFilter::ip_in_subnet(ip, network, 24));
        assert!(!NetworkFilter::ip_in_subnet(ip, network, 32));
    }
    
    #[test]
    fn test_port_matching() {
        let matcher = PortMatcher::Range(80, 443);
        assert!(NetworkFilter::port_matches(&matcher, 80));
        assert!(NetworkFilter::port_matches(&matcher, 443));
        assert!(NetworkFilter::port_matches(&matcher, 200));
        assert!(!NetworkFilter::port_matches(&matcher, 444));
    }
}