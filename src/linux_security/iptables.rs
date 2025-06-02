use std::process::Command;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use std::net::IpAddr;

// IPTables chain names for FluxDefense
const FLUX_INPUT_CHAIN: &str = "FLUXDEFENSE_INPUT";
const FLUX_OUTPUT_CHAIN: &str = "FLUXDEFENSE_OUTPUT";
const FLUX_FORWARD_CHAIN: &str = "FLUXDEFENSE_FORWARD";

#[derive(Debug, Clone)]
pub struct IptablesManager {
    dry_run: bool,
    use_nftables: bool,
}

#[derive(Debug, Clone)]
pub struct IptablesRule {
    pub chain: Chain,
    pub action: RuleAction,
    pub protocol: Option<String>,
    pub source_ip: Option<String>,
    pub dest_ip: Option<String>,
    pub source_port: Option<u16>,
    pub dest_port: Option<u16>,
    pub interface: Option<String>,
    pub comment: Option<String>,
    pub position: Option<u32>, // For inserting at specific position
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
    Input,
    Output,
    Forward,
    FluxInput,
    FluxOutput,
    FluxForward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuleAction {
    Accept,
    Drop,
    Reject,
    Log,
    Jump(Chain),
}

impl IptablesManager {
    pub fn new() -> Result<Self> {
        // Check if we have iptables or nftables
        let has_iptables = Command::new("which")
            .arg("iptables")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
            
        let has_nftables = Command::new("which")
            .arg("nft")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
            
        if !has_iptables && !has_nftables {
            return Err(anyhow!("Neither iptables nor nftables found"));
        }
        
        // Check if running as root
        let uid = unsafe { libc::geteuid() };
        if uid != 0 {
            warn!("Not running as root - iptables commands will fail");
        }
        
        Ok(Self {
            dry_run: false,
            use_nftables: !has_iptables && has_nftables,
        })
    }
    
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }
    
    pub fn initialize_chains(&self) -> Result<()> {
        info!("Initializing FluxDefense iptables chains");
        
        // Create custom chains
        self.create_chain(Chain::FluxInput)?;
        self.create_chain(Chain::FluxOutput)?;
        self.create_chain(Chain::FluxForward)?;
        
        // Add jump rules from main chains to our chains
        self.add_jump_rule(Chain::Input, Chain::FluxInput)?;
        self.add_jump_rule(Chain::Output, Chain::FluxOutput)?;
        self.add_jump_rule(Chain::Forward, Chain::FluxForward)?;
        
        info!("FluxDefense iptables chains initialized");
        Ok(())
    }
    
    pub fn cleanup_chains(&self) -> Result<()> {
        info!("Cleaning up FluxDefense iptables chains");
        
        // Remove jump rules
        self.remove_jump_rule(Chain::Input, Chain::FluxInput)?;
        self.remove_jump_rule(Chain::Output, Chain::FluxOutput)?;
        self.remove_jump_rule(Chain::Forward, Chain::FluxForward)?;
        
        // Flush and delete custom chains
        self.flush_chain(Chain::FluxInput)?;
        self.flush_chain(Chain::FluxOutput)?;
        self.flush_chain(Chain::FluxForward)?;
        
        self.delete_chain(Chain::FluxInput)?;
        self.delete_chain(Chain::FluxOutput)?;
        self.delete_chain(Chain::FluxForward)?;
        
        info!("FluxDefense iptables chains cleaned up");
        Ok(())
    }
    
    fn create_chain(&self, chain: Chain) -> Result<()> {
        let chain_name = self.chain_to_string(chain);
        let mut cmd = self.build_command();
        
        cmd.arg("-N").arg(&chain_name);
        
        self.execute_command(cmd, &format!("Create chain {}", chain_name))
    }
    
    fn delete_chain(&self, chain: Chain) -> Result<()> {
        let chain_name = self.chain_to_string(chain);
        let mut cmd = self.build_command();
        
        cmd.arg("-X").arg(&chain_name);
        
        self.execute_command(cmd, &format!("Delete chain {}", chain_name))
    }
    
    fn flush_chain(&self, chain: Chain) -> Result<()> {
        let chain_name = self.chain_to_string(chain);
        let mut cmd = self.build_command();
        
        cmd.arg("-F").arg(&chain_name);
        
        self.execute_command(cmd, &format!("Flush chain {}", chain_name))
    }
    
    fn add_jump_rule(&self, from_chain: Chain, to_chain: Chain) -> Result<()> {
        let from_name = self.chain_to_string(from_chain);
        let to_name = self.chain_to_string(to_chain);
        
        // Check if jump rule already exists
        if self.jump_rule_exists(&from_name, &to_name)? {
            debug!("Jump rule from {} to {} already exists", from_name, to_name);
            return Ok(());
        }
        
        let mut cmd = self.build_command();
        cmd.arg("-A").arg(&from_name)
           .arg("-j").arg(&to_name);
        
        self.execute_command(cmd, &format!("Add jump rule {} -> {}", from_name, to_name))
    }
    
    fn remove_jump_rule(&self, from_chain: Chain, to_chain: Chain) -> Result<()> {
        let from_name = self.chain_to_string(from_chain);
        let to_name = self.chain_to_string(to_chain);
        
        let mut cmd = self.build_command();
        cmd.arg("-D").arg(&from_name)
           .arg("-j").arg(&to_name);
        
        // Ignore errors for removing non-existent rules
        let _ = self.execute_command(cmd, &format!("Remove jump rule {} -> {}", from_name, to_name));
        Ok(())
    }
    
    fn jump_rule_exists(&self, from_chain: &str, to_chain: &str) -> Result<bool> {
        let mut cmd = self.build_command();
        cmd.arg("-L").arg(from_chain).arg("-n");
        
        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        Ok(stdout.contains(to_chain))
    }
    
    pub fn add_rule(&self, rule: &IptablesRule) -> Result<()> {
        let mut cmd = self.build_command();
        
        // Add or Insert
        if let Some(pos) = rule.position {
            cmd.arg("-I");
            cmd.arg(self.chain_to_string(rule.chain));
            cmd.arg(pos.to_string());
        } else {
            cmd.arg("-A");
            cmd.arg(self.chain_to_string(rule.chain));
        }
        
        // Protocol
        if let Some(ref proto) = rule.protocol {
            cmd.arg("-p").arg(proto);
        }
        
        // Source IP
        if let Some(ref src_ip) = rule.source_ip {
            cmd.arg("-s").arg(src_ip);
        }
        
        // Destination IP
        if let Some(ref dst_ip) = rule.dest_ip {
            cmd.arg("-d").arg(dst_ip);
        }
        
        // Source port (requires protocol)
        if let Some(src_port) = rule.source_port {
            if rule.protocol.is_some() {
                cmd.arg("--sport").arg(src_port.to_string());
            }
        }
        
        // Destination port (requires protocol)
        if let Some(dst_port) = rule.dest_port {
            if rule.protocol.is_some() {
                cmd.arg("--dport").arg(dst_port.to_string());
            }
        }
        
        // Interface
        if let Some(ref iface) = rule.interface {
            match rule.chain {
                Chain::Input | Chain::FluxInput => {
                    cmd.arg("-i").arg(iface);
                }
                Chain::Output | Chain::FluxOutput => {
                    cmd.arg("-o").arg(iface);
                }
                _ => {}
            }
        }
        
        // Comment
        if let Some(ref comment) = rule.comment {
            cmd.arg("-m").arg("comment")
               .arg("--comment").arg(comment);
        }
        
        // Action
        match rule.action {
            RuleAction::Accept => { cmd.arg("-j").arg("ACCEPT"); },
            RuleAction::Drop => { cmd.arg("-j").arg("DROP"); },
            RuleAction::Reject => { cmd.arg("-j").arg("REJECT"); },
            RuleAction::Log => {
                cmd.arg("-j").arg("LOG");
                if let Some(ref comment) = rule.comment {
                    cmd.arg("--log-prefix").arg(format!("[FLUX-{}] ", comment));
                }
            }
            RuleAction::Jump(chain) => {
                cmd.arg("-j").arg(self.chain_to_string(chain));
            }
        }
        
        self.execute_command(cmd, "Add rule")
    }
    
    pub fn remove_rule(&self, rule: &IptablesRule) -> Result<()> {
        let mut cmd = self.build_command();
        
        // Delete by specification
        cmd.arg("-D");
        cmd.arg(self.chain_to_string(rule.chain));
        
        // Build the same rule specification
        if let Some(ref proto) = rule.protocol {
            cmd.arg("-p").arg(proto);
        }
        
        if let Some(ref src_ip) = rule.source_ip {
            cmd.arg("-s").arg(src_ip);
        }
        
        if let Some(ref dst_ip) = rule.dest_ip {
            cmd.arg("-d").arg(dst_ip);
        }
        
        if let Some(src_port) = rule.source_port {
            if rule.protocol.is_some() {
                cmd.arg("--sport").arg(src_port.to_string());
            }
        }
        
        if let Some(dst_port) = rule.dest_port {
            if rule.protocol.is_some() {
                cmd.arg("--dport").arg(dst_port.to_string());
            }
        }
        
        if let Some(ref iface) = rule.interface {
            match rule.chain {
                Chain::Input | Chain::FluxInput => {
                    cmd.arg("-i").arg(iface);
                }
                Chain::Output | Chain::FluxOutput => {
                    cmd.arg("-o").arg(iface);
                }
                _ => {}
            }
        }
        
        if let Some(ref comment) = rule.comment {
            cmd.arg("-m").arg("comment")
               .arg("--comment").arg(comment);
        }
        
        match rule.action {
            RuleAction::Accept => { cmd.arg("-j").arg("ACCEPT"); },
            RuleAction::Drop => { cmd.arg("-j").arg("DROP"); },
            RuleAction::Reject => { cmd.arg("-j").arg("REJECT"); },
            RuleAction::Log => { cmd.arg("-j").arg("LOG"); },
            RuleAction::Jump(chain) => {
                cmd.arg("-j").arg(self.chain_to_string(chain));
            }
        }
        
        self.execute_command(cmd, "Remove rule")
    }
    
    pub fn list_rules(&self, chain: Option<Chain>) -> Result<Vec<String>> {
        let mut cmd = self.build_command();
        
        cmd.arg("-L");
        if let Some(c) = chain {
            cmd.arg(self.chain_to_string(c));
        }
        cmd.arg("-n").arg("-v");
        
        let output = cmd.output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to list rules: {}", stderr));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
    
    pub fn save_rules(&self, file_path: Option<&str>) -> Result<()> {
        let cmd_name = if self.use_nftables {
            "nft"
        } else {
            "iptables-save"
        };
        
        let output = Command::new(cmd_name).output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to save rules: {}", stderr));
        }
        
        if let Some(path) = file_path {
            std::fs::write(path, output.stdout)?;
            info!("Saved iptables rules to {}", path);
        } else {
            // Print to stdout
            print!("{}", String::from_utf8_lossy(&output.stdout));
        }
        
        Ok(())
    }
    
    pub fn restore_rules(&self, file_path: &str) -> Result<()> {
        let cmd_name = if self.use_nftables {
            "nft"
        } else {
            "iptables-restore"
        };
        
        let mut cmd = Command::new(cmd_name);
        cmd.stdin(std::process::Stdio::from(
            std::fs::File::open(file_path)?
        ));
        
        let output = cmd.output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Failed to restore rules: {}", stderr));
        }
        
        info!("Restored iptables rules from {}", file_path);
        Ok(())
    }
    
    // Helper methods for common rules
    pub fn block_ip(&self, ip: &str, comment: Option<&str>) -> Result<()> {
        // Block incoming
        self.add_rule(&IptablesRule {
            chain: Chain::FluxInput,
            action: RuleAction::Drop,
            protocol: None,
            source_ip: Some(ip.to_string()),
            dest_ip: None,
            source_port: None,
            dest_port: None,
            interface: None,
            comment: comment.map(|s| s.to_string()),
            position: Some(1), // Insert at beginning for immediate effect
        })?;
        
        // Block outgoing
        self.add_rule(&IptablesRule {
            chain: Chain::FluxOutput,
            action: RuleAction::Drop,
            protocol: None,
            source_ip: None,
            dest_ip: Some(ip.to_string()),
            source_port: None,
            dest_port: None,
            interface: None,
            comment: comment.map(|s| s.to_string()),
            position: Some(1),
        })?;
        
        info!("Blocked IP: {}", ip);
        Ok(())
    }
    
    pub fn block_port(&self, port: u16, protocol: &str, comment: Option<&str>) -> Result<()> {
        // Block incoming
        self.add_rule(&IptablesRule {
            chain: Chain::FluxInput,
            action: RuleAction::Drop,
            protocol: Some(protocol.to_string()),
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: Some(port),
            interface: None,
            comment: comment.map(|s| s.to_string()),
            position: Some(1),
        })?;
        
        info!("Blocked port: {} ({})", port, protocol);
        Ok(())
    }
    
    pub fn allow_established(&self) -> Result<()> {
        let mut cmd = self.build_command();
        cmd.arg("-A").arg(self.chain_to_string(Chain::FluxInput))
           .arg("-m").arg("state")
           .arg("--state").arg("ESTABLISHED,RELATED")
           .arg("-j").arg("ACCEPT");
        
        self.execute_command(cmd, "Allow established connections")
    }
    
    pub fn enable_logging(&self, chain: Chain, prefix: &str) -> Result<()> {
        self.add_rule(&IptablesRule {
            chain,
            action: RuleAction::Log,
            protocol: None,
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: None,
            interface: None,
            comment: Some(prefix.to_string()),
            position: None,
        })
    }
    
    fn build_command(&self) -> Command {
        if self.use_nftables {
            // For nftables compatibility layer
            Command::new("nft")
        } else {
            Command::new("iptables")
        }
    }
    
    fn execute_command(&self, mut cmd: Command, operation: &str) -> Result<()> {
        if self.dry_run {
            info!("DRY RUN: {} - {:?}", operation, cmd);
            return Ok(());
        }
        
        debug!("Executing: {} - {:?}", operation, cmd);
        
        let output = cmd.output()?;
        
        if output.status.success() {
            debug!("{} succeeded", operation);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(anyhow!("{} failed: {}", operation, stderr))
        }
    }
    
    fn chain_to_string(&self, chain: Chain) -> String {
        match chain {
            Chain::Input => "INPUT".to_string(),
            Chain::Output => "OUTPUT".to_string(),
            Chain::Forward => "FORWARD".to_string(),
            Chain::FluxInput => FLUX_INPUT_CHAIN.to_string(),
            Chain::FluxOutput => FLUX_OUTPUT_CHAIN.to_string(),
            Chain::FluxForward => FLUX_FORWARD_CHAIN.to_string(),
        }
    }
}

// Helper functions for creating common rules
impl IptablesRule {
    pub fn block_ip_rule(ip: &str) -> Self {
        Self {
            chain: Chain::FluxInput,
            action: RuleAction::Drop,
            protocol: None,
            source_ip: Some(ip.to_string()),
            dest_ip: None,
            source_port: None,
            dest_port: None,
            interface: None,
            comment: Some(format!("Block IP {}", ip)),
            position: None,
        }
    }
    
    pub fn allow_port_rule(port: u16, protocol: &str) -> Self {
        Self {
            chain: Chain::FluxInput,
            action: RuleAction::Accept,
            protocol: Some(protocol.to_string()),
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: Some(port),
            interface: None,
            comment: Some(format!("Allow {} port {}", protocol, port)),
            position: None,
        }
    }
    
    pub fn rate_limit_rule(port: u16, protocol: &str, rate: &str) -> Self {
        // Note: This is a simplified version. Real rate limiting requires
        // additional iptables modules like hashlimit
        Self {
            chain: Chain::FluxInput,
            action: RuleAction::Accept,
            protocol: Some(protocol.to_string()),
            source_ip: None,
            dest_ip: None,
            source_port: None,
            dest_port: Some(port),
            interface: None,
            comment: Some(format!("Rate limit {} port {} to {}", protocol, port, rate)),
            position: None,
        }
    }
}