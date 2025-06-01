use std::fs::File;
use std::os::unix::io::{AsRawFd, RawFd};
use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error, debug};
use libc::{self, c_int};
use std::mem;

// Fanotify constants
const FAN_CLOEXEC: c_int = 0x00000001;
const FAN_CLASS_CONTENT: c_int = 0x00000004;
const FAN_CLASS_PRE_CONTENT: c_int = 0x00000008;
const FAN_UNLIMITED_QUEUE: c_int = 0x00000010;
const FAN_UNLIMITED_MARKS: c_int = 0x00000020;

const FAN_MARK_ADD: c_int = 0x00000001;
const FAN_MARK_MOUNT: c_int = 0x00000010;
const FAN_MARK_FILESYSTEM: c_int = 0x00000100;

// Fanotify events
const FAN_ACCESS: u64 = 0x00000001;
const FAN_MODIFY: u64 = 0x00000002;
const FAN_CLOSE_WRITE: u64 = 0x00000008;
const FAN_CLOSE_NOWRITE: u64 = 0x00000010;
const FAN_OPEN: u64 = 0x00000020;
const FAN_OPEN_EXEC: u64 = 0x00001000;
const FAN_ACCESS_PERM: u64 = 0x00020000;
const FAN_OPEN_PERM: u64 = 0x00010000;
const FAN_OPEN_EXEC_PERM: u64 = 0x00040000;

const FAN_ALLOW: u32 = 0x01;
const FAN_DENY: u32 = 0x02;

const FAN_EVENT_INFO_TYPE_FID: u8 = 1;
const FAN_EVENT_INFO_TYPE_DFID_NAME: u8 = 2;
const FAN_EVENT_INFO_TYPE_DFID: u8 = 3;
const FAN_EVENT_INFO_TYPE_PIDFD: u8 = 4;

#[repr(C)]
struct FanotifyEventMetadata {
    event_len: u32,
    vers: u8,
    reserved: u8,
    metadata_len: u16,
    mask: u64,
    fd: i32,
    pid: i32,
}

#[repr(C)]
struct FanotifyResponse {
    fd: i32,
    response: u32,
}

pub struct FanotifyMonitor {
    fd: RawFd,
    running: bool,
}

impl FanotifyMonitor {
    pub fn new() -> Result<Self> {
        info!("Initializing fanotify monitor");
        
        // Check if we have CAP_SYS_ADMIN capability
        let uid = unsafe { libc::geteuid() };
        if uid != 0 {
            warn!("Fanotify requires root privileges or CAP_SYS_ADMIN capability");
            return Err(anyhow!("Insufficient privileges for fanotify"));
        }
        
        // Initialize fanotify
        let fd = unsafe {
            libc::syscall(
                libc::SYS_fanotify_init,
                FAN_CLOEXEC | FAN_CLASS_PRE_CONTENT | FAN_UNLIMITED_QUEUE | FAN_UNLIMITED_MARKS,
                libc::O_RDONLY | libc::O_LARGEFILE
            )
        };
        
        if fd < 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to initialize fanotify: {}", err));
        }
        
        let fd = fd as RawFd;
        info!("Fanotify initialized successfully with fd: {}", fd);
        
        Ok(Self {
            fd,
            running: false,
        })
    }
    
    pub fn add_mount_mark(&self, mount_path: &str, mask: u64) -> Result<()> {
        let path_cstr = std::ffi::CString::new(mount_path)?;
        
        let ret = unsafe {
            libc::syscall(
                libc::SYS_fanotify_mark,
                self.fd,
                FAN_MARK_ADD | FAN_MARK_MOUNT,
                mask,
                libc::AT_FDCWD,
                path_cstr.as_ptr()
            )
        };
        
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to add fanotify mark on {}: {}", mount_path, err));
        }
        
        info!("Added fanotify mark on mount: {} with mask: 0x{:x}", mount_path, mask);
        Ok(())
    }
    
    pub fn add_filesystem_mark(&self, mask: u64) -> Result<()> {
        // Monitor entire filesystem
        let ret = unsafe {
            libc::syscall(
                libc::SYS_fanotify_mark,
                self.fd,
                FAN_MARK_ADD | FAN_MARK_FILESYSTEM,
                mask,
                libc::AT_FDCWD,
                std::ptr::null::<libc::c_char>()
            )
        };
        
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to add filesystem-wide fanotify mark: {}", err));
        }
        
        info!("Added filesystem-wide fanotify mark with mask: 0x{:x}", mask);
        Ok(())
    }
    
    pub fn start_monitoring(&mut self) -> Result<()> {
        if self.running {
            return Ok(());
        }
        
        // Add marks for monitoring
        // Monitor file execution with permission checks
        let exec_mask = FAN_OPEN_EXEC_PERM | FAN_OPEN_EXEC;
        
        // Monitor file access and modifications
        let access_mask = FAN_OPEN | FAN_ACCESS | FAN_MODIFY | FAN_CLOSE_WRITE;
        
        // Try to monitor the root filesystem
        if let Err(e) = self.add_mount_mark("/", exec_mask | access_mask) {
            warn!("Failed to monitor root filesystem: {}", e);
            // Try specific directories instead
            for path in &["/usr", "/bin", "/sbin", "/opt", "/home"] {
                if let Err(e) = self.add_mount_mark(path, exec_mask | access_mask) {
                    warn!("Failed to monitor {}: {}", path, e);
                }
            }
        }
        
        self.running = true;
        info!("Fanotify monitoring started");
        Ok(())
    }
    
    pub fn read_events(&self) -> Result<Vec<FanotifyEvent>> {
        let mut buffer = vec![0u8; 4096];
        let mut events = Vec::new();
        
        let bytes_read = unsafe {
            libc::read(self.fd, buffer.as_mut_ptr() as *mut libc::c_void, buffer.len())
        };
        
        if bytes_read < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::WouldBlock {
                return Ok(events);
            }
            return Err(anyhow!("Failed to read fanotify events: {}", err));
        }
        
        let mut offset = 0;
        while offset < bytes_read as usize {
            if offset + mem::size_of::<FanotifyEventMetadata>() > bytes_read as usize {
                break;
            }
            
            let metadata = unsafe {
                &*(buffer.as_ptr().add(offset) as *const FanotifyEventMetadata)
            };
            
            if metadata.vers != libc::FANOTIFY_METADATA_VERSION as u8 {
                warn!("Unsupported fanotify metadata version: {}", metadata.vers);
                break;
            }
            
            let event = FanotifyEvent {
                mask: metadata.mask,
                fd: metadata.fd,
                pid: metadata.pid,
                path: self.get_path_from_fd(metadata.fd),
            };
            
            events.push(event);
            
            // Respond to permission events
            if metadata.mask & (FAN_OPEN_PERM | FAN_ACCESS_PERM | FAN_OPEN_EXEC_PERM) != 0 {
                self.respond_to_event(metadata.fd, FAN_ALLOW)?;
            }
            
            // Close the file descriptor
            if metadata.fd >= 0 {
                unsafe { libc::close(metadata.fd) };
            }
            
            offset += metadata.event_len as usize;
        }
        
        Ok(events)
    }
    
    fn get_path_from_fd(&self, fd: RawFd) -> Option<PathBuf> {
        if fd < 0 {
            return None;
        }
        
        let proc_path = format!("/proc/self/fd/{}", fd);
        std::fs::read_link(&proc_path).ok()
    }
    
    fn respond_to_event(&self, fd: RawFd, response: u32) -> Result<()> {
        let resp = FanotifyResponse { fd, response };
        
        let ret = unsafe {
            libc::write(
                self.fd,
                &resp as *const _ as *const libc::c_void,
                mem::size_of::<FanotifyResponse>()
            )
        };
        
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            return Err(anyhow!("Failed to respond to fanotify event: {}", err));
        }
        
        Ok(())
    }
    
    pub fn stop(&mut self) -> Result<()> {
        if self.running {
            info!("Stopping fanotify monitoring");
            self.running = false;
        }
        Ok(())
    }
}

impl Drop for FanotifyMonitor {
    fn drop(&mut self) {
        if self.fd >= 0 {
            unsafe { libc::close(self.fd) };
        }
    }
}

#[derive(Debug, Clone)]
pub struct FanotifyEvent {
    pub mask: u64,
    pub fd: RawFd,
    pub pid: i32,
    pub path: Option<PathBuf>,
}

impl FanotifyEvent {
    pub fn is_exec(&self) -> bool {
        self.mask & (FAN_OPEN_EXEC | FAN_OPEN_EXEC_PERM) != 0
    }
    
    pub fn is_open(&self) -> bool {
        self.mask & (FAN_OPEN | FAN_OPEN_PERM) != 0
    }
    
    pub fn is_access(&self) -> bool {
        self.mask & (FAN_ACCESS | FAN_ACCESS_PERM) != 0
    }
    
    pub fn is_modify(&self) -> bool {
        self.mask & FAN_MODIFY != 0
    }
    
    pub fn is_permission_event(&self) -> bool {
        self.mask & (FAN_OPEN_PERM | FAN_ACCESS_PERM | FAN_OPEN_EXEC_PERM) != 0
    }
}