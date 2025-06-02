use std::process::{Command, Stdio};
use std::io::Write;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};

// Netfilter/nftables integration for packet filtering
// This module provides a Rust interface to nftables for advanced packet filtering

#[derive(Debug, Clone)]
pub struct NetfilterRule {
    pub id: String,
    pub table: String,
    pub chain: String,
    pub priority: i32,
    pub rule: NftRule,
    pub comment: String,
}

#[derive(Debug, Clone)]
pub enum NftRule {
    // Basic filtering
    Accept,
    Drop,
    Reject { with_type: Option<String> },
    
    // Connection tracking
    ConnTrack { state: Vec<ConnState> },
    
    // Protocol matching
    Protocol { 
        proto: String,
        sport: Option<PortMatch>,
        dport: Option<PortMatch>,
        action: Box<NftRule>,
    },
    
    // IP matching
    IpMatch {
        direction: Direction,
        addr: IpMatch,
        action: Box<NftRule>,
    },
    
    // Rate limiting
    RateLimit {
        rate: u32,
        per: RatePer,
        burst: Option<u32>,
        action: Box<NftRule>,
    },
    
    // Logging
    Log {
        prefix: String,
        level: LogLevel,
        continue_rule: Option<Box<NftRule>>,
    },
    
    // Complex rules
    Compound(Vec<NftRule>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnState {
    New,
    Established,
    Related,
    Invalid,
}

#[derive(Debug, Clone)]
pub enum Direction {
    Source,
    Destination,
}

#[derive(Debug, Clone)]
pub enum IpMatch {
    Single(String),
    Range(String, String),
    Subnet(String),
    Set(String), // Reference to a named set
}

#[derive(Debug, Clone)]
pub enum PortMatch {
    Single(u16),
    Range(u16, u16),
    Set(String), // Reference to a named set
}

#[derive(Debug, Clone)]
pub enum RatePer {
    Second,
    Minute,
    Hour,
    Day,
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

pub struct NetfilterManager {
    rules: Arc<RwLock<HashMap<String, NetfilterRule>>>,
    tables: Arc<RwLock<HashMap<String, NftTable>>>,
    sets: Arc<RwLock<HashMap<String, NftSet>>>,
    enabled: bool,
}

#[derive(Debug, Clone)]
struct NftTable {
    name: String,
    family: String,
    chains: HashMap<String, NftChain>,
}

#[derive(Debug, Clone)]
struct NftChain {
    name: String,
    chain_type: String,
    hook: String,
    priority: i32,
    policy: String,
}

#[derive(Debug, Clone)]
struct NftSet {
    name: String,
    table: String,
    set_type: String,
    elements: Vec<String>,
}

impl NetfilterManager {
    pub fn new() -> Result<Self> {
        let manager = Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            tables: Arc::new(RwLock::new(HashMap::new())),
            sets: Arc::new(RwLock::new(HashMap::new())),
            enabled: false,
        };
        
        Ok(manager)
    }
    
    pub fn initialize(&mut self) -> Result<()> {
        // Check if nftables is available
        if !self.check_nftables_available()? {
            return Err(anyhow!("nftables is not available on this system"));
        }
        
        // Check if we have necessary permissions
        if !self.check_permissions()? {
            return Err(anyhow!("Insufficient permissions for netfilter operations (need CAP_NET_ADMIN)"));
        }
        
        // Create our main table and chains
        self.create_fluxdefense_table()?;
        
        self.enabled = true;
        info!("Netfilter manager initialized successfully");
        
        Ok(())
    }
    
    fn check_nftables_available(&self) -> Result<bool> {
        match Command::new("nft").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version = String::from_utf8_lossy(&output.stdout);
                    debug!("nftables version: {}", version.trim());
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(_) => Ok(false),
        }
    }
    
    fn check_permissions(&self) -> Result<bool> {
        // Try to list tables (requires permissions)
        match Command::new("nft").arg("list").arg("tables").output() {
            Ok(output) => Ok(output.status.success()),
            Err(_) => Ok(false),
        }
    }
    
    fn create_fluxdefense_table(&mut self) -> Result<()> {
        // Create the main FluxDefense table
        let table_cmd = "add table inet fluxdefense";
        self.execute_nft_command(table_cmd)?;
        
        // Create input chain for incoming packets
        let input_chain = r#"add chain inet fluxdefense input { 
            type filter hook input priority 0; 
            policy accept; 
        }"#;
        self.execute_nft_command(input_chain)?;
        
        // Create output chain for outgoing packets
        let output_chain = r#"add chain inet fluxdefense output { 
            type filter hook output priority 0; 
            policy accept; 
        }"#;
        self.execute_nft_command(output_chain)?;
        
        // Create forward chain
        let forward_chain = r#"add chain inet fluxdefense forward { 
            type filter hook forward priority 0; 
            policy accept; 
        }"#;
        self.execute_nft_command(forward_chain)?;
        
        // Store table info
        let mut tables = self.tables.write()
            .map_err(|_| anyhow!("Failed to acquire tables write lock"))?;
        
        let mut chains = HashMap::new();
        chains.insert("input".to_string(), NftChain {
            name: "input".to_string(),
            chain_type: "filter".to_string(),
            hook: "input".to_string(),
            priority: 0,
            policy: "accept".to_string(),
        });
        chains.insert("output".to_string(), NftChain {
            name: "output".to_string(),
            chain_type: "filter".to_string(),
            hook: "output".to_string(),
            priority: 0,
            policy: "accept".to_string(),
        });
        chains.insert("forward".to_string(), NftChain {
            name: "forward".to_string(),
            chain_type: "filter".to_string(),
            hook: "forward".to_string(),
            priority: 0,
            policy: "accept".to_string(),
        });
        
        tables.insert("fluxdefense".to_string(), NftTable {
            name: "fluxdefense".to_string(),
            family: "inet".to_string(),
            chains,
        });
        
        info!("Created FluxDefense nftables table and chains");
        Ok(())
    }
    
    fn execute_nft_command(&self, command: &str) -> Result<String> {
        debug!("Executing nft command: {}", command);
        
        let output = Command::new("nft")
            .arg("-f")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(command.as_bytes())?;
                }
                child.wait_with_output()
            })?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("nft command failed: {}", error))
        }
    }
    
    pub fn add_rule(&mut self, rule: NetfilterRule) -> Result<()> {
        if !self.enabled {
            return Err(anyhow!("Netfilter manager is not initialized"));
        }
        
        // Generate nftables command from rule
        let nft_cmd = self.generate_nft_rule_command(&rule)?;
        
        // Execute the command
        self.execute_nft_command(&nft_cmd)?;
        
        // Store the rule
        let mut rules = self.rules.write()
            .map_err(|_| anyhow!("Failed to acquire rules write lock"))?;
        rules.insert(rule.id.clone(), rule.clone());
        
        info!("Added netfilter rule: {}", rule.id);
        Ok(())
    }
    
    fn generate_nft_rule_command(&self, rule: &NetfilterRule) -> Result<String> {
        let mut cmd = format!("add rule inet {} {} ", rule.table, rule.chain);
        
        // Add rule expression
        let expr = self.generate_rule_expression(&rule.rule)?;
        cmd.push_str(&expr);
        
        // Add comment if provided
        if !rule.comment.is_empty() {
            cmd.push_str(&format!(" comment \"{}\"", rule.comment));
        }
        
        Ok(cmd)
    }
    
    fn generate_rule_expression(&self, rule: &NftRule) -> Result<String> {
        match rule {
            NftRule::Accept => Ok("accept".to_string()),
            NftRule::Drop => Ok("drop".to_string()),
            NftRule::Reject { with_type } => {
                if let Some(reject_type) = with_type {
                    Ok(format!("reject with {}", reject_type))
                } else {
                    Ok("reject".to_string())
                }
            }
            
            NftRule::ConnTrack { state } => {
                let states: Vec<String> = state.iter().map(|s| match s {
                    ConnState::New => "new",
                    ConnState::Established => "established",
                    ConnState::Related => "related",
                    ConnState::Invalid => "invalid",
                }.to_string()).collect();
                
                Ok(format!("ct state {{ {} }}", states.join(", ")))
            }
            
            NftRule::Protocol { proto, sport, dport, action } => {
                let mut expr = format!("meta l4proto {}", proto);
                
                if let Some(sport_match) = sport {
                    expr.push_str(&format!(" {} sport {}", proto, self.format_port_match(sport_match)?));
                }
                
                if let Some(dport_match) = dport {
                    expr.push_str(&format!(" {} dport {}", proto, self.format_port_match(dport_match)?));
                }
                
                let action_expr = self.generate_rule_expression(action)?;
                expr.push_str(&format!(" {}", action_expr));
                
                Ok(expr)
            }
            
            NftRule::IpMatch { direction, addr, action } => {
                let dir = match direction {
                    Direction::Source => "saddr",
                    Direction::Destination => "daddr",
                };
                
                let addr_expr = match addr {
                    IpMatch::Single(ip) => ip.clone(),
                    IpMatch::Range(start, end) => format!("{}-{}", start, end),
                    IpMatch::Subnet(subnet) => subnet.clone(),
                    IpMatch::Set(set_name) => format!("@{}", set_name),
                };
                
                let action_expr = self.generate_rule_expression(action)?;
                Ok(format!("ip {} {} {}", dir, addr_expr, action_expr))
            }
            
            NftRule::RateLimit { rate, per, burst, action } => {
                let per_str = match per {
                    RatePer::Second => "second",
                    RatePer::Minute => "minute",
                    RatePer::Hour => "hour",
                    RatePer::Day => "day",
                };
                
                let mut limit_expr = format!("limit rate {}/{}", rate, per_str);
                if let Some(burst_val) = burst {
                    limit_expr.push_str(&format!(" burst {}", burst_val));
                }
                
                let action_expr = self.generate_rule_expression(action)?;
                Ok(format!("{} {}", limit_expr, action_expr))
            }
            
            NftRule::Log { prefix, level, continue_rule } => {
                let level_str = match level {
                    LogLevel::Emergency => "emerg",
                    LogLevel::Alert => "alert",
                    LogLevel::Critical => "crit",
                    LogLevel::Error => "err",
                    LogLevel::Warning => "warn",
                    LogLevel::Notice => "notice",
                    LogLevel::Info => "info",
                    LogLevel::Debug => "debug",
                };
                
                let mut expr = format!("log prefix \"{}\" level {}", prefix, level_str);
                
                if let Some(cont) = continue_rule {
                    let cont_expr = self.generate_rule_expression(cont)?;
                    expr.push_str(&format!(" {}", cont_expr));
                }
                
                Ok(expr)
            }
            
            NftRule::Compound(rules) => {
                let exprs: Result<Vec<String>> = rules.iter()
                    .map(|r| self.generate_rule_expression(r))
                    .collect();
                Ok(exprs?.join(" "))
            }
        }
    }
    
    fn format_port_match(&self, port_match: &PortMatch) -> Result<String> {
        match port_match {
            PortMatch::Single(port) => Ok(port.to_string()),
            PortMatch::Range(start, end) => Ok(format!("{}-{}", start, end)),
            PortMatch::Set(set_name) => Ok(format!("@{}", set_name)),
        }
    }
    
    pub fn remove_rule(&mut self, rule_id: &str) -> Result<()> {
        let mut rules = self.rules.write()
            .map_err(|_| anyhow!("Failed to acquire rules write lock"))?;
        
        if let Some(rule) = rules.remove(rule_id) {
            // In production, we would need to find and delete the specific rule
            // This is complex with nftables, so we might need to track rule handles
            warn!("Rule removal from nftables not fully implemented");
            info!("Removed rule from tracking: {}", rule_id);
        }
        
        Ok(())
    }
    
    pub fn create_ip_set(&mut self, name: &str, set_type: &str) -> Result<()> {
        let cmd = format!("add set inet fluxdefense {} {{ type {}; }}", name, set_type);
        self.execute_nft_command(&cmd)?;
        
        let mut sets = self.sets.write()
            .map_err(|_| anyhow!("Failed to acquire sets write lock"))?;
        
        sets.insert(name.to_string(), NftSet {
            name: name.to_string(),
            table: "fluxdefense".to_string(),
            set_type: set_type.to_string(),
            elements: Vec::new(),
        });
        
        info!("Created IP set: {}", name);
        Ok(())
    }
    
    pub fn add_to_set(&mut self, set_name: &str, element: &str) -> Result<()> {
        let cmd = format!("add element inet fluxdefense {} {{ {} }}", set_name, element);
        self.execute_nft_command(&cmd)?;
        
        let mut sets = self.sets.write()
            .map_err(|_| anyhow!("Failed to acquire sets write lock"))?;
        
        if let Some(set) = sets.get_mut(set_name) {
            set.elements.push(element.to_string());
        }
        
        Ok(())
    }
    
    // Helper methods for common rule patterns
    pub fn block_ip(&mut self, ip: &str, comment: &str) -> Result<()> {
        let rule = NetfilterRule {
            id: format!("block_ip_{}", ip.replace('.', "_")),
            table: "fluxdefense".to_string(),
            chain: "input".to_string(),
            priority: 100,
            rule: NftRule::IpMatch {
                direction: Direction::Source,
                addr: IpMatch::Single(ip.to_string()),
                action: Box::new(NftRule::Drop),
            },
            comment: comment.to_string(),
        };
        
        self.add_rule(rule)
    }
    
    pub fn rate_limit_port(&mut self, port: u16, rate: u32, per: RatePer) -> Result<()> {
        let rule = NetfilterRule {
            id: format!("rate_limit_port_{}", port),
            table: "fluxdefense".to_string(),
            chain: "input".to_string(),
            priority: 50,
            rule: NftRule::Protocol {
                proto: "tcp".to_string(),
                sport: None,
                dport: Some(PortMatch::Single(port)),
                action: Box::new(NftRule::RateLimit {
                    rate,
                    per,
                    burst: Some(rate * 2),
                    action: Box::new(NftRule::Accept),
                }),
            },
            comment: format!("Rate limit port {}", port),
        };
        
        self.add_rule(rule)
    }
    
    pub fn block_port_range(&mut self, start: u16, end: u16, proto: &str) -> Result<()> {
        let rule = NetfilterRule {
            id: format!("block_ports_{}_{}", start, end),
            table: "fluxdefense".to_string(),
            chain: "input".to_string(),
            priority: 100,
            rule: NftRule::Protocol {
                proto: proto.to_string(),
                sport: None,
                dport: Some(PortMatch::Range(start, end)),
                action: Box::new(NftRule::Drop),
            },
            comment: format!("Block {} ports {}-{}", proto, start, end),
        };
        
        self.add_rule(rule)
    }
    
    pub fn allow_established(&mut self) -> Result<()> {
        let rule = NetfilterRule {
            id: "allow_established".to_string(),
            table: "fluxdefense".to_string(),
            chain: "input".to_string(),
            priority: 10,
            rule: NftRule::ConnTrack {
                state: vec![ConnState::Established, ConnState::Related],
            },
            comment: "Allow established connections".to_string(),
        };
        
        self.add_rule(rule)
    }
    
    pub fn log_and_drop_invalid(&mut self) -> Result<()> {
        let rule = NetfilterRule {
            id: "log_drop_invalid".to_string(),
            table: "fluxdefense".to_string(),
            chain: "input".to_string(),
            priority: 20,
            rule: NftRule::Compound(vec![
                NftRule::ConnTrack {
                    state: vec![ConnState::Invalid],
                },
                NftRule::Log {
                    prefix: "[FluxDefense] Invalid: ".to_string(),
                    level: LogLevel::Warning,
                    continue_rule: Some(Box::new(NftRule::Drop)),
                },
            ]),
            comment: "Log and drop invalid connections".to_string(),
        };
        
        self.add_rule(rule)
    }
    
    pub fn cleanup(&mut self) -> Result<()> {
        if self.enabled {
            // Remove our table and all rules
            let cmd = "delete table inet fluxdefense";
            match self.execute_nft_command(cmd) {
                Ok(_) => info!("Cleaned up FluxDefense nftables rules"),
                Err(e) => warn!("Failed to cleanup nftables: {}", e),
            }
            
            self.enabled = false;
        }
        
        Ok(())
    }
}

impl Drop for NetfilterManager {
    fn drop(&mut self) {
        if self.enabled {
            if let Err(e) = self.cleanup() {
                error!("Failed to cleanup netfilter rules: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rule_generation() {
        let manager = NetfilterManager::new().unwrap();
        
        // Test simple drop rule
        let expr = manager.generate_rule_expression(&NftRule::Drop).unwrap();
        assert_eq!(expr, "drop");
        
        // Test protocol rule
        let proto_rule = NftRule::Protocol {
            proto: "tcp".to_string(),
            sport: None,
            dport: Some(PortMatch::Single(80)),
            action: Box::new(NftRule::Accept),
        };
        let expr = manager.generate_rule_expression(&proto_rule).unwrap();
        assert!(expr.contains("tcp dport 80 accept"));
    }
}