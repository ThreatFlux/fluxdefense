use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{channel, Sender, Receiver};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};

use crate::monitor::{PassiveMonitor, ProcessInfo as MonitorProcessInfo, FileAccessType, NetworkProtocol, Verdict};
use super::fanotify::{FanotifyMonitor, FanotifyEvent};
use super::netlink::{NetlinkMonitor, NetworkConnection};
use super::process_monitor::{ProcessMonitor, ProcessInfo};

#[derive(Debug)]
pub enum SecurityEvent {
    FileExecution {
        process_info: MonitorProcessInfo,
        target_path: PathBuf,
        file_hash: Option<String>,
        code_signature: Option<String>,
    },
    FileAccess {
        process_info: MonitorProcessInfo,
        target_path: PathBuf,
        access_type: FileAccessType,
    },
    NetworkConnection {
        process_info: MonitorProcessInfo,
        remote_ip: String,
        remote_port: u16,
        domain: Option<String>,
        protocol: NetworkProtocol,
    },
}

pub struct LinuxSecurityMonitor {
    fanotify: Option<FanotifyMonitor>,
    netlink: Option<NetlinkMonitor>,
    process_monitor: Arc<Mutex<ProcessMonitor>>,
    running: Arc<Mutex<bool>>,
    event_sender: Option<Sender<SecurityEvent>>,
    event_receiver: Option<Receiver<SecurityEvent>>,
}

impl LinuxSecurityMonitor {
    pub fn new() -> Result<Self> {
        info!("Initializing Linux security monitor");
        
        // Try to initialize fanotify (requires root or CAP_SYS_ADMIN)
        let fanotify = match FanotifyMonitor::new() {
            Ok(mut monitor) => {
                if let Err(e) = monitor.start_monitoring() {
                    warn!("Failed to start fanotify monitoring: {}", e);
                    None
                } else {
                    Some(monitor)
                }
            }
            Err(e) => {
                warn!("Failed to initialize fanotify (requires root): {}", e);
                None
            }
        };
        
        // Initialize netlink monitor
        let netlink = match NetlinkMonitor::new() {
            Ok(mut monitor) => {
                if let Err(e) = monitor.start_monitoring() {
                    warn!("Failed to start netlink monitoring: {}", e);
                    None
                } else {
                    Some(monitor)
                }
            }
            Err(e) => {
                warn!("Failed to initialize netlink monitor: {}", e);
                None
            }
        };
        
        // Initialize process monitor
        let mut process_monitor = ProcessMonitor::new();
        if let Err(e) = process_monitor.start_monitoring() {
            warn!("Failed to start process monitoring: {}", e);
        }
        
        // Create event channel
        let (sender, receiver) = channel();
        
        Ok(Self {
            fanotify,
            netlink,
            process_monitor: Arc::new(Mutex::new(process_monitor)),
            running: Arc::new(Mutex::new(false)),
            event_sender: Some(sender),
            event_receiver: Some(receiver),
        })
    }
    
    pub fn start(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            if *running {
                return Ok(());
            }
            *running = true;
        }
        
        info!("Starting Linux security monitoring");
        
        // Start fanotify event processing thread if available
        if self.fanotify.is_some() {
            let process_monitor = Arc::clone(&self.process_monitor);
            let running = Arc::clone(&self.running);
            let sender = self.event_sender.as_ref().unwrap().clone();
            
            thread::spawn(move || {
                Self::process_fanotify_events(process_monitor, running, sender);
            });
        }
        
        // Start network monitoring thread if available
        if self.netlink.is_some() {
            let process_monitor = Arc::clone(&self.process_monitor);
            let running = Arc::clone(&self.running);
            let sender = self.event_sender.as_ref().unwrap().clone();
            
            thread::spawn(move || {
                Self::process_network_events(process_monitor, running, sender);
            });
        }
        
        Ok(())
    }
    
    fn process_fanotify_events(
        process_monitor: Arc<Mutex<ProcessMonitor>>,
        running: Arc<Mutex<bool>>,
        sender: Sender<SecurityEvent>,
    ) {
        info!("Starting fanotify event processing thread");
        
        while *running.lock().unwrap() {
            // For now, just sleep - actual implementation would read events
            thread::sleep(Duration::from_secs(1));
        }
        
        info!("Stopped fanotify event processing");
    }
    
    fn handle_fanotify_event(
        &self,
        event: &FanotifyEvent,
        passive_monitor: &PassiveMonitor,
        process_monitor: &Arc<Mutex<ProcessMonitor>>,
    ) {
        debug!("Fanotify event: {:?}", event);
        
        // Get process info
        let process_info = {
            let pm = process_monitor.lock().unwrap();
            pm.get_process_by_pid(event.pid as u32).cloned()
        };
        
        let monitor_process_info = if let Some(info) = process_info {
            MonitorProcessInfo {
                pid: info.pid,
                path: info.exe_path.unwrap_or_else(|| PathBuf::from(&info.name)),
                parent_pid: Some(info.ppid),
                user_id: info.uid,
                executable_hash: None,
                command_line: Some(info.cmdline.join(" ")),
            }
        } else {
            MonitorProcessInfo {
                pid: event.pid as u32,
                path: PathBuf::from(format!("pid:{}", event.pid)),
                parent_pid: None,
                user_id: 0,
                executable_hash: None,
                command_line: None,
            }
        };
        
        if let Some(path) = &event.path {
            if event.is_exec() {
                // File execution event
                passive_monitor.handle_file_execution_event(
                    monitor_process_info,
                    path.clone(),
                    None, // TODO: Calculate file hash
                    None, // TODO: Get code signature if available
                );
            } else if event.is_open() || event.is_access() {
                // File access event
                let access_type = if event.is_modify() {
                    FileAccessType::Write
                } else {
                    FileAccessType::Read
                };
                
                passive_monitor.handle_file_access_event(
                    monitor_process_info,
                    path.clone(),
                    access_type,
                );
            }
        }
    }
    
    fn process_network_events(
        process_monitor: Arc<Mutex<ProcessMonitor>>,
        running: Arc<Mutex<bool>>,
        sender: Sender<SecurityEvent>,
    ) {
        info!("Starting network event processing thread");
        
        while *running.lock().unwrap() {
            // For now, just sleep - actual implementation would monitor connections
            thread::sleep(Duration::from_secs(1));
        }
        
        info!("Stopped network event processing");
    }
    
    fn handle_network_connection(
        &self,
        conn: &NetworkConnection,
        passive_monitor: &PassiveMonitor,
        process_monitor: &Arc<Mutex<ProcessMonitor>>,
    ) {
        debug!("New network connection: {:?}", conn);
        
        // Find process by inode
        let process_info = {
            let pm = process_monitor.lock().unwrap();
            pm.find_process_by_inode(conn.inode).cloned()
        };
        
        let monitor_process_info = if let Some(info) = process_info {
            MonitorProcessInfo {
                pid: info.pid,
                path: info.exe_path.unwrap_or_else(|| PathBuf::from(&info.name)),
                parent_pid: Some(info.ppid),
                user_id: info.uid,
                executable_hash: None,
                command_line: Some(info.cmdline.join(" ")),
            }
        } else {
            MonitorProcessInfo {
                pid: 0,
                path: PathBuf::from("unknown"),
                parent_pid: None,
                user_id: conn.uid,
                executable_hash: None,
                command_line: None,
            }
        };
        
        passive_monitor.handle_network_connection_event(
            monitor_process_info,
            conn.remote_addr.to_string(),
            conn.remote_port,
            None, // TODO: Resolve domain name
            NetworkProtocol::Tcp,
        );
    }
    
    pub fn stop(&mut self) -> Result<()> {
        {
            let mut running = self.running.lock().unwrap();
            if !*running {
                return Ok(());
            }
            *running = false;
        }
        
        info!("Stopping Linux security monitoring");
        
        if let Some(mut fanotify) = self.fanotify.take() {
            fanotify.stop()?;
        }
        
        if let Some(mut netlink) = self.netlink.take() {
            netlink.stop()?;
        }
        
        if let Ok(mut pm) = self.process_monitor.lock() {
            pm.stop()?;
        }
        
        Ok(())
    }
}