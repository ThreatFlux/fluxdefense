use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::collections::HashSet;
use anyhow::Result;
use tracing::{info, warn, debug};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub allowed_ips: HashSet<IpAddr>,
    pub allowed_domains: HashSet<String>,
    pub allowed_ports: HashSet<u16>,
    pub blocked_ips: HashSet<IpAddr>,
    pub blocked_domains: HashSet<String>,
    pub blocked_ports: HashSet<u16>,
    pub allow_local_network: bool,
    pub allow_system_processes: bool,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        let mut policy = Self {
            allowed_ips: HashSet::new(),
            allowed_domains: HashSet::new(),
            allowed_ports: HashSet::new(),
            blocked_ips: HashSet::new(),
            blocked_domains: HashSet::new(),
            blocked_ports: HashSet::new(),
            allow_local_network: true,
            allow_system_processes: true,
        };
        
        policy.add_default_rules();
        policy
    }
}

impl NetworkPolicy {
    pub fn new() -> Self {
        Self::default()
    }
    
    fn add_default_rules(&mut self) {
        // Allow common safe ports
        let safe_ports = vec![80, 443, 53, 22, 993, 995, 587, 465];
        for port in safe_ports {
            self.allowed_ports.insert(port);
        }
        
        // Allow major cloud providers and services
        let trusted_domains = vec![
            "apple.com",
            "icloud.com",
            "github.com",
            "microsoft.com",
            "google.com",
            "amazonaws.com",
            "cloudflare.com",
        ];
        
        for domain in trusted_domains {
            self.allowed_domains.insert(domain.to_string());
        }
    }
    
    pub fn is_ip_allowed(&self, ip: IpAddr) -> bool {
        // Check if explicitly blocked
        if self.blocked_ips.contains(&ip) {
            warn!("IP explicitly blocked: {}", ip);
            return false;
        }
        
        // Check if explicitly allowed
        if self.allowed_ips.contains(&ip) {
            debug!("IP explicitly allowed: {}", ip);
            return true;
        }
        
        // Check if local network traffic is allowed
        if self.allow_local_network && self.is_local_ip(ip) {
            debug!("Local IP allowed: {}", ip);
            return true;
        }
        
        warn!("IP not allowed: {}", ip);
        false
    }
    
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        // Check if explicitly blocked
        for blocked_domain in &self.blocked_domains {
            if domain.ends_with(blocked_domain) {
                warn!("Domain blocked: {} (matches {})", domain, blocked_domain);
                return false;
            }
        }
        
        // Check if explicitly allowed
        for allowed_domain in &self.allowed_domains {
            if domain.ends_with(allowed_domain) {
                debug!("Domain allowed: {} (matches {})", domain, allowed_domain);
                return true;
            }
        }
        
        warn!("Domain not allowed: {}", domain);
        false
    }
    
    pub fn is_port_allowed(&self, port: u16) -> bool {
        // Check if explicitly blocked
        if self.blocked_ports.contains(&port) {
            warn!("Port explicitly blocked: {}", port);
            return false;
        }
        
        // Check if explicitly allowed
        if self.allowed_ports.contains(&port) {
            debug!("Port explicitly allowed: {}", port);
            return true;
        }
        
        warn!("Port not allowed: {}", port);
        false
    }
    
    pub fn is_connection_allowed(
        &self,
        remote_ip: IpAddr,
        remote_port: u16,
        domain: Option<&str>,
    ) -> bool {
        // Check port first
        if !self.is_port_allowed(remote_port) {
            return false;
        }
        
        // Check domain if provided
        if let Some(domain) = domain {
            if !self.is_domain_allowed(domain) {
                return false;
            }
        }
        
        // Check IP
        self.is_ip_allowed(remote_ip)
    }
    
    fn is_local_ip(&self, ip: IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                // Private IPv4 ranges
                ipv4.is_private() || 
                ipv4.is_loopback() ||
                ipv4.is_link_local()
            }
            IpAddr::V6(ipv6) => {
                // Private IPv6 ranges
                ipv6.is_loopback() ||
                ipv6.is_unspecified() ||
                (ipv6.segments()[0] & 0xfe00) == 0xfc00 || // Unique local
                (ipv6.segments()[0] & 0xffc0) == 0xfe80    // Link local
            }
        }
    }
    
    pub fn add_allowed_ip(&mut self, ip: IpAddr) {
        info!("Adding allowed IP: {}", ip);
        self.allowed_ips.insert(ip);
    }
    
    pub fn add_allowed_domain(&mut self, domain: String) {
        info!("Adding allowed domain: {}", domain);
        self.allowed_domains.insert(domain);
    }
    
    pub fn add_allowed_port(&mut self, port: u16) {
        info!("Adding allowed port: {}", port);
        self.allowed_ports.insert(port);
    }
    
    pub fn add_blocked_ip(&mut self, ip: IpAddr) {
        info!("Adding blocked IP: {}", ip);
        self.blocked_ips.insert(ip);
    }
    
    pub fn add_blocked_domain(&mut self, domain: String) {
        info!("Adding blocked domain: {}", domain);
        self.blocked_domains.insert(domain);
    }
    
    pub fn add_blocked_port(&mut self, port: u16) {
        info!("Adding blocked port: {}", port);
        self.blocked_ports.insert(port);
    }
    
    pub fn remove_allowed_ip(&mut self, ip: &IpAddr) {
        self.allowed_ips.remove(ip);
    }
    
    pub fn remove_allowed_domain(&mut self, domain: &str) {
        self.allowed_domains.remove(domain);
    }
    
    pub fn remove_allowed_port(&mut self, port: &u16) {
        self.allowed_ports.remove(port);
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        info!("Network policy saved to: {:?}", path);
        Ok(())
    }
    
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let policy: Self = serde_json::from_str(&content)?;
        info!("Network policy loaded from: {:?}", path);
        Ok(policy)
    }
}