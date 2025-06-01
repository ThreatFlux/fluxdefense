pub mod esf;
pub mod network;
pub mod policy;
pub mod config;
pub mod ffi;
pub mod scanner;
pub mod monitor;
pub mod system_metrics;

use anyhow::Result;
use tracing::{info, warn};
use std::path::PathBuf;

pub struct FluxDefense {
    esf_client: Option<esf::EsfClient>,
    network_filter: Option<network::NetworkFilter>,
    pub file_policy: policy::FilePolicy,
    pub network_policy: policy::NetworkPolicy,
    monitor: Option<monitor::PassiveMonitor>,
    config: config::Config,
}

impl FluxDefense {
    pub fn new() -> Result<Self> {
        info!("Initializing FluxDefense EDR system");
        
        let config = config::Config::load_config()?;
        
        Ok(Self {
            esf_client: None,
            network_filter: None,
            file_policy: policy::FilePolicy::default(),
            network_policy: policy::NetworkPolicy::default(),
            monitor: None,
            config,
        })
    }
    
    pub fn new_with_config(config: config::Config) -> Result<Self> {
        info!("Initializing FluxDefense EDR system with provided config");
        
        Ok(Self {
            esf_client: None,
            network_filter: None,
            file_policy: policy::FilePolicy::default(),
            network_policy: policy::NetworkPolicy::default(),
            monitor: None,
            config,
        })
    }
    
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting FluxDefense protection");
        
        // Create passive monitor
        let log_path = self.config.log_file_path.clone()
            .unwrap_or_else(|| PathBuf::from("./fluxdefense-events.log"));
        
        #[cfg(feature = "passive-mode")]
        let passive_mode = true;
        #[cfg(not(feature = "passive-mode"))]
        let passive_mode = false;
        
        let mut monitor = monitor::PassiveMonitor::new(log_path, passive_mode)?;
        
        // Load whitelist data if available
        if let Some(ref policy_path) = self.config.file_policy_path {
            if let Some(parent) = policy_path.parent() {
                if let Err(e) = monitor.load_whitelist_data(parent) {
                    warn!("Failed to load whitelist data: {}", e);
                }
            }
        }
        
        self.monitor = Some(monitor);
        
        // Initialize ESF client only if not in passive mode
        if !passive_mode {
            self.esf_client = Some(esf::EsfClient::new()?);
            self.network_filter = Some(network::NetworkFilter::new()?);
        } else {
            info!("Running in passive mode - ESF and network filtering disabled");
        }
        
        info!("FluxDefense protection started successfully (passive_mode: {})", passive_mode);
        Ok(())
    }
    
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping FluxDefense protection");
        
        if let Some(ref mut client) = self.esf_client {
            client.stop()?;
        }
        
        if let Some(ref mut filter) = self.network_filter {
            filter.stop()?;
        }
        
        info!("FluxDefense protection stopped");
        Ok(())
    }
    
    pub fn get_monitor(&self) -> Option<&monitor::PassiveMonitor> {
        self.monitor.as_ref()
    }
    
    pub fn get_monitor_mut(&mut self) -> Option<&mut monitor::PassiveMonitor> {
        self.monitor.as_mut()
    }
    
    pub fn simulate_file_execution(&self, process_path: PathBuf, target_path: PathBuf) -> monitor::Verdict {
        if let Some(ref monitor) = self.monitor {
            let process_info = monitor::ProcessInfo {
                pid: std::process::id(),
                path: process_path,
                parent_pid: None,
                user_id: unsafe { libc::getuid() },
                executable_hash: None,
                command_line: None,
            };
            
            monitor.handle_file_execution_event(process_info, target_path, None, None)
        } else {
            monitor::Verdict::Deny
        }
    }
    
    pub fn simulate_network_connection(&self, remote_ip: String, remote_port: u16, domain: Option<String>) -> monitor::Verdict {
        if let Some(ref monitor) = self.monitor {
            let process_info = monitor::ProcessInfo {
                pid: std::process::id(),
                path: PathBuf::from("/usr/bin/test"),
                parent_pid: None,
                user_id: unsafe { libc::getuid() },
                executable_hash: None,
                command_line: None,
            };
            
            monitor.handle_network_connection_event(
                process_info, 
                remote_ip, 
                remote_port, 
                domain, 
                monitor::NetworkProtocol::Tcp
            )
        } else {
            monitor::Verdict::Deny
        }
    }
}