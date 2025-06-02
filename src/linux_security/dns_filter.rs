use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
// use trust_dns_resolver::TokioAsyncResolver;
// use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};

// DNS filtering and monitoring module
// Provides DNS request interception, filtering, and caching

#[derive(Clone)]
pub struct DnsFilter {
    // Filtering lists
    blacklist_domains: Arc<RwLock<HashSet<String>>>,
    whitelist_domains: Arc<RwLock<HashSet<String>>>,
    blacklist_patterns: Arc<RwLock<Vec<regex::Regex>>>,
    
    // Caching
    cache: Arc<RwLock<DnsCache>>,
    
    // Configuration
    config: Arc<RwLock<DnsFilterConfig>>,
    
    // Statistics
    stats: Arc<RwLock<DnsStats>>,
}

#[derive(Debug, Clone)]
pub struct DnsFilterConfig {
    pub enabled: bool,
    pub block_mode: BlockMode,
    pub cache_ttl: Duration,
    pub upstream_servers: Vec<SocketAddr>,
    pub listen_port: u16,
    pub log_queries: bool,
    pub block_suspicious_tlds: bool,
    pub block_dga_domains: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockMode {
    Nxdomain,       // Return NXDOMAIN
    Refused,        // Return REFUSED
    SinkHole(IpAddr), // Return a sinkhole IP
}

#[derive(Debug, Clone)]
struct DnsCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    domain: String,
    result: DnsResult,
    created_at: Instant,
    ttl: Duration,
    hit_count: u64,
}

#[derive(Debug, Clone)]
enum DnsResult {
    Resolved(Vec<IpAddr>),
    Blocked(String), // Reason
    Error(String),
}

#[derive(Debug, Clone, Default)]
pub struct DnsStats {
    pub total_queries: u64,
    pub blocked_queries: u64,
    pub cached_responses: u64,
    pub upstream_queries: u64,
    pub malicious_domains_blocked: u64,
    pub dga_domains_blocked: u64,
}

#[derive(Debug, Clone)]
pub struct DnsEvent {
    pub timestamp: Instant,
    pub query_type: DnsQueryType,
    pub domain: String,
    pub source: SocketAddr,
    pub action: DnsAction,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DnsQueryType {
    A,      // IPv4
    AAAA,   // IPv6
    MX,     // Mail server
    TXT,    // Text record
    CNAME,  // Canonical name
    Other(String),
}

#[derive(Debug, Clone)]
pub enum DnsAction {
    Allowed,
    Blocked,
    Cached,
    Error,
}

impl DnsFilter {
    pub fn new() -> Result<Self> {
        let config = DnsFilterConfig {
            enabled: true,
            block_mode: BlockMode::Nxdomain,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            upstream_servers: vec![
                "8.8.8.8:53".parse()?,
                "8.8.4.4:53".parse()?,
                "1.1.1.1:53".parse()?,
            ],
            listen_port: 5353, // Alternative DNS port
            log_queries: true,
            block_suspicious_tlds: true,
            block_dga_domains: true,
        };
        
        Ok(Self {
            blacklist_domains: Arc::new(RwLock::new(HashSet::new())),
            whitelist_domains: Arc::new(RwLock::new(HashSet::new())),
            blacklist_patterns: Arc::new(RwLock::new(Vec::new())),
            cache: Arc::new(RwLock::new(DnsCache {
                entries: HashMap::new(),
                max_entries: 10000,
            })),
            config: Arc::new(RwLock::new(config)),
            stats: Arc::new(RwLock::new(DnsStats::default())),
        })
    }
    
    pub fn load_default_blacklists(&self) -> Result<()> {
        let mut blacklist = self.blacklist_domains.write()
            .map_err(|_| anyhow!("Failed to acquire blacklist write lock"))?;
        
        // Add known malicious domains
        let malicious_domains = vec![
            // Command & Control servers
            "evil.com",
            "malware-c2.com",
            "botnet-control.net",
            
            // Phishing domains
            "phishing-site.com",
            "fake-bank.com",
            
            // Cryptomining pools
            "pool.minexmr.com",
            "xmrpool.eu",
            "dwarfpool.com",
            
            // Known malware distribution
            "malware-download.com",
            "exploit-kit.net",
        ];
        
        for domain in malicious_domains {
            blacklist.insert(domain.to_string());
        }
        
        // Add pattern-based filtering
        let mut patterns = self.blacklist_patterns.write()
            .map_err(|_| anyhow!("Failed to acquire patterns write lock"))?;
        
        // DGA (Domain Generation Algorithm) patterns
        patterns.push(regex::Regex::new(r"^[a-z]{20,}\.com$")?); // Long random strings
        patterns.push(regex::Regex::new(r"^[0-9a-f]{16,}\.")?); // Hex strings
        
        // Suspicious TLD patterns
        if self.config.read().unwrap().block_suspicious_tlds {
            patterns.push(regex::Regex::new(r"\.tk$")?);
            patterns.push(regex::Regex::new(r"\.ml$")?);
            patterns.push(regex::Regex::new(r"\.ga$")?);
            patterns.push(regex::Regex::new(r"\.cc$")?);
        }
        
        info!("Loaded {} blacklisted domains and {} patterns", 
              blacklist.len(), patterns.len());
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub async fn start_dns_proxy(&self, event_handler: mpsc::Sender<DnsEvent>) -> Result<()> {
        let config = self.config.read()
            .map_err(|_| anyhow!("Failed to read config"))?;
        
        if !config.enabled {
            return Ok(());
        }
        
        let listen_addr = format!("127.0.0.1:{}", config.listen_port);
        let socket = Arc::new(UdpSocket::bind(&listen_addr).await?);
        
        info!("DNS filter proxy listening on {}", listen_addr);
        
        let mut buf = vec![0u8; 512]; // DNS max packet size
        
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    let packet = buf[..len].to_vec();
                    
                    // Process DNS query
                    // TODO: Fix Send + Sync issue with tokio::spawn
                    // let socket_clone = Arc::clone(&socket);
                    // let filter_clone = self.clone();
                    // let handler_clone = event_handler.clone();
                    // tokio::spawn(async move {
                    //     filter_clone.process_dns_query(
                    //         packet,
                    //         src,
                    //         socket_clone,
                    //         handler_clone
                    //     ).await;
                    // });
                }
                Err(e) => {
                    error!("Error receiving DNS query: {}", e);
                }
            }
        }
    }
    
    async fn process_dns_query(
        self,
        packet: Vec<u8>,
        src: SocketAddr,
        socket: Arc<UdpSocket>,
        event_handler: mpsc::Sender<DnsEvent>,
    ) {
        // Update stats
        if let Ok(mut stats) = self.stats.write() {
            stats.total_queries += 1;
        }
        
        // Parse DNS query (simplified - in production use trust-dns-proto)
        if let Some(domain) = self.parse_dns_query(&packet) {
            let query_type = self.get_query_type(&packet);
            
            // Check if domain should be blocked
            let (action, reason) = self.check_domain(&domain).await;
            
            match action {
                DnsAction::Blocked => {
                    // Send blocked response
                    if let Ok(response) = self.create_blocked_response(&packet).await {
                        let _ = socket.send_to(&response, src).await;
                    }
                    
                    if let Ok(mut stats) = self.stats.write() {
                        stats.blocked_queries += 1;
                        if reason.as_ref().map_or(false, |r| r.contains("malicious")) {
                            stats.malicious_domains_blocked += 1;
                        }
                        if reason.as_ref().map_or(false, |r| r.contains("DGA")) {
                            stats.dga_domains_blocked += 1;
                        }
                    }
                }
                DnsAction::Cached => {
                    // Return cached response
                    if let Some(response) = self.get_cached_response(&domain).await {
                        let _ = socket.send_to(&response, src).await;
                        
                        if let Ok(mut stats) = self.stats.write() {
                            stats.cached_responses += 1;
                        }
                    }
                }
                DnsAction::Allowed => {
                    // Forward to upstream
                    if let Ok(response) = self.forward_to_upstream(&packet).await {
                        let _ = socket.send_to(&response, src).await;
                        
                        // Cache the response
                        self.cache_response(&domain, &response).await;
                        
                        if let Ok(mut stats) = self.stats.write() {
                            stats.upstream_queries += 1;
                        }
                    }
                }
                _ => {}
            }
            
            // Send event
            let event = DnsEvent {
                timestamp: Instant::now(),
                query_type,
                domain: domain.clone(),
                source: src,
                action,
                reason,
            };
            
            let _ = event_handler.send(event).await;
        }
    }
    
    fn parse_dns_query(&self, packet: &[u8]) -> Option<String> {
        // Simplified DNS parsing - extract domain name
        // In production, use trust-dns-proto for proper parsing
        
        if packet.len() < 12 {
            return None;
        }
        
        let mut offset = 12; // Skip header
        let mut domain = String::new();
        
        while offset < packet.len() {
            let len = packet[offset] as usize;
            if len == 0 {
                break;
            }
            
            if !domain.is_empty() {
                domain.push('.');
            }
            
            offset += 1;
            if offset + len > packet.len() {
                return None;
            }
            
            if let Ok(label) = std::str::from_utf8(&packet[offset..offset + len]) {
                domain.push_str(label);
            }
            
            offset += len;
        }
        
        if domain.is_empty() {
            None
        } else {
            Some(domain.to_lowercase())
        }
    }
    
    fn get_query_type(&self, packet: &[u8]) -> DnsQueryType {
        // Extract query type from DNS packet
        // This is simplified - in production use proper DNS parsing
        DnsQueryType::A
    }
    
    async fn check_domain(&self, domain: &str) -> (DnsAction, Option<String>) {
        // Check whitelist first
        if let Ok(whitelist) = self.whitelist_domains.read() {
            if whitelist.contains(domain) {
                return (DnsAction::Allowed, None);
            }
        }
        
        // Check exact blacklist
        if let Ok(blacklist) = self.blacklist_domains.read() {
            if blacklist.contains(domain) {
                return (DnsAction::Blocked, Some("Blacklisted domain".to_string()));
            }
        }
        
        // Check patterns
        if let Ok(patterns) = self.blacklist_patterns.read() {
            for pattern in patterns.iter() {
                if pattern.is_match(domain) {
                    return (DnsAction::Blocked, Some("Matches blacklist pattern".to_string()));
                }
            }
        }
        
        // Check for DGA domains
        if self.config.read().unwrap().block_dga_domains {
            if self.is_dga_domain(domain) {
                return (DnsAction::Blocked, Some("Suspected DGA domain".to_string()));
            }
        }
        
        // Check cache
        if self.is_cached(domain).await {
            return (DnsAction::Cached, None);
        }
        
        (DnsAction::Allowed, None)
    }
    
    pub fn is_dga_domain(&self, domain: &str) -> bool {
        // Simple DGA detection heuristics
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return false;
        }
        
        let subdomain = parts[0];
        
        // Check for high entropy (randomness)
        let entropy = self.calculate_entropy(subdomain);
        if entropy > 4.0 && subdomain.len() > 10 {
            return true;
        }
        
        // Check for excessive consonants
        let consonant_ratio = self.calculate_consonant_ratio(subdomain);
        if consonant_ratio > 0.8 && subdomain.len() > 8 {
            return true;
        }
        
        // Check for numeric patterns
        let has_hex_pattern = subdomain.chars().all(|c| c.is_ascii_hexdigit());
        if has_hex_pattern && subdomain.len() > 12 {
            return true;
        }
        
        false
    }
    
    fn calculate_entropy(&self, s: &str) -> f64 {
        let mut char_counts = HashMap::new();
        let len = s.len() as f64;
        
        for c in s.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }
        
        let mut entropy = 0.0;
        for count in char_counts.values() {
            let p = *count as f64 / len;
            entropy -= p * p.log2();
        }
        
        entropy
    }
    
    fn calculate_consonant_ratio(&self, s: &str) -> f64 {
        let vowels = "aeiouAEIOU";
        let consonants = s.chars()
            .filter(|c| c.is_alphabetic() && !vowels.contains(*c))
            .count();
        let total = s.chars().filter(|c| c.is_alphabetic()).count();
        
        if total == 0 {
            0.0
        } else {
            consonants as f64 / total as f64
        }
    }
    
    async fn is_cached(&self, domain: &str) -> bool {
        if let Ok(cache) = self.cache.read() {
            if let Some(entry) = cache.entries.get(domain) {
                if entry.created_at.elapsed() < entry.ttl {
                    return true;
                }
            }
        }
        false
    }
    
    async fn get_cached_response(&self, domain: &str) -> Option<Vec<u8>> {
        // In a real implementation, we would construct a proper DNS response
        // from the cached data
        None
    }
    
    async fn create_blocked_response(&self, query: &[u8]) -> Result<Vec<u8>> {
        // Create a DNS response based on block mode
        let config = self.config.read()
            .map_err(|_| anyhow!("Failed to read config"))?;
        
        // In a real implementation, we would properly construct a DNS response
        // For now, return a simple NXDOMAIN response
        let mut response = query.to_vec();
        if response.len() >= 3 {
            response[2] |= 0x80; // Set response bit
            response[3] = 0x83; // Set NXDOMAIN
        }
        
        Ok(response)
    }
    
    async fn forward_to_upstream(&self, query: &[u8]) -> Result<Vec<u8>> {
        let config = self.config.read()
            .map_err(|_| anyhow!("Failed to read config"))?;
        
        // Try each upstream server
        for upstream in &config.upstream_servers {
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            socket.send_to(query, upstream).await?;
            
            let mut buf = vec![0u8; 512];
            match tokio::time::timeout(
                Duration::from_secs(2),
                socket.recv_from(&mut buf)
            ).await {
                Ok(Ok((len, _))) => {
                    return Ok(buf[..len].to_vec());
                }
                _ => continue,
            }
        }
        
        Err(anyhow!("All upstream servers failed"))
    }
    
    async fn cache_response(&self, domain: &str, response: &[u8]) {
        let config = self.config.read().unwrap();
        
        if let Ok(mut cache) = self.cache.write() {
            // Implement cache eviction if needed
            if cache.entries.len() >= cache.max_entries {
                // Remove oldest entry
                if let Some(oldest) = cache.entries.iter()
                    .min_by_key(|(_, entry)| entry.created_at)
                    .map(|(k, _)| k.clone()) {
                    cache.entries.remove(&oldest);
                }
            }
            
            cache.entries.insert(domain.to_string(), CacheEntry {
                domain: domain.to_string(),
                result: DnsResult::Resolved(Vec::new()), // Would parse from response
                created_at: Instant::now(),
                ttl: config.cache_ttl,
                hit_count: 0,
            });
        }
    }
    
    // Management methods
    pub fn add_blacklist_domain(&self, domain: String) -> Result<()> {
        let mut blacklist = self.blacklist_domains.write()
            .map_err(|_| anyhow!("Failed to acquire blacklist write lock"))?;
        blacklist.insert(domain);
        Ok(())
    }
    
    pub fn remove_blacklist_domain(&self, domain: &str) -> Result<()> {
        let mut blacklist = self.blacklist_domains.write()
            .map_err(|_| anyhow!("Failed to acquire blacklist write lock"))?;
        blacklist.remove(domain);
        Ok(())
    }
    
    pub fn add_whitelist_domain(&self, domain: String) -> Result<()> {
        let mut whitelist = self.whitelist_domains.write()
            .map_err(|_| anyhow!("Failed to acquire whitelist write lock"))?;
        whitelist.insert(domain);
        Ok(())
    }
    
    pub fn add_blacklist_pattern(&self, pattern: &str) -> Result<()> {
        let regex = regex::Regex::new(pattern)?;
        let mut patterns = self.blacklist_patterns.write()
            .map_err(|_| anyhow!("Failed to acquire patterns write lock"))?;
        patterns.push(regex);
        Ok(())
    }
    
    pub fn get_stats(&self) -> Result<DnsStats> {
        let stats = self.stats.read()
            .map_err(|_| anyhow!("Failed to read stats"))?;
        Ok(stats.clone())
    }
    
    pub fn clear_cache(&self) -> Result<()> {
        let mut cache = self.cache.write()
            .map_err(|_| anyhow!("Failed to acquire cache write lock"))?;
        cache.entries.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dga_detection() {
        let filter = DnsFilter::new().unwrap();
        
        // Test DGA-like domains
        assert!(filter.is_dga_domain("asdkjhqwlekjhasdlkjh.com"));
        assert!(filter.is_dga_domain("a1b2c3d4e5f6a7b8c9d0.net"));
        assert!(filter.is_dga_domain("zzxxccvvbbnnmm.org"));
        
        // Test normal domains
        assert!(!filter.is_dga_domain("google.com"));
        assert!(!filter.is_dga_domain("facebook.com"));
        assert!(!filter.is_dga_domain("example.org"));
    }
    
    #[test]
    fn test_entropy_calculation() {
        let filter = DnsFilter::new().unwrap();
        
        // High entropy (random)
        let high_entropy = filter.calculate_entropy("aB3xY9zQ");
        assert!(high_entropy > 2.5);
        
        // Low entropy (repetitive)
        let low_entropy = filter.calculate_entropy("aaaaaaa");
        assert!(low_entropy < 1.0);
    }
}