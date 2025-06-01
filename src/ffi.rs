use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Once;
use tracing::{info, warn, error};
use crate::FluxDefense;
use crate::policy::{FilePolicy, NetworkPolicy};
use std::path::PathBuf;

static INIT: Once = Once::new();
static mut FLUX_DEFENSE: Option<FluxDefense> = None;

// Initialize the FluxDefense system
#[no_mangle]
pub extern "C" fn flux_defense_init() {
    INIT.call_once(|| {
        // Initialize logging
        tracing_subscriber::fmt::init();
        
        info!("Initializing FluxDefense from Swift extension");
        
        match FluxDefense::new() {
            Ok(defense) => {
                unsafe {
                    FLUX_DEFENSE = Some(defense);
                }
                info!("FluxDefense initialized successfully");
            }
            Err(e) => {
                error!("Failed to initialize FluxDefense: {}", e);
            }
        }
    });
}

// Cleanup the FluxDefense system
#[no_mangle]
pub extern "C" fn flux_defense_cleanup() {
    info!("Cleaning up FluxDefense");
    unsafe {
        FLUX_DEFENSE = None;
    }
}

// Evaluate a file system event and return whether it should be allowed
#[no_mangle]
pub extern "C" fn flux_defense_evaluate_file_event(
    event_type: c_int,
    pid: u32,
    path_ptr: *const c_char,
) -> bool {
    if path_ptr.is_null() {
        warn!("Received null path pointer");
        return false;
    }
    
    let path_cstr = unsafe { CStr::from_ptr(path_ptr) };
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            warn!("Invalid UTF-8 in path");
            return false;
        }
    };
    
    let path = PathBuf::from(path_str);
    
    info!("Evaluating file event: type={}, pid={}, path={:?}", event_type, pid, path);
    
    unsafe {
        if let Some(ref defense) = FLUX_DEFENSE {
            // Use the monitor to handle the event if available
            if let Some(_monitor) = defense.get_monitor() {
                let verdict = defense.simulate_file_execution(
                    PathBuf::from(format!("PID:{}", pid)),
                    path.clone()
                );
                match verdict {
                    crate::monitor::Verdict::Allow | crate::monitor::Verdict::Log => true,
                    crate::monitor::Verdict::Deny => false,
                }
            } else {
                // Fallback to simple file policy check
                defense.file_policy.is_path_allowed(&path)
            }
        } else {
            warn!("FluxDefense not initialized, denying by default");
            false
        }
    }
}

// Evaluate a network connection and return whether it should be allowed
#[no_mangle]
pub extern "C" fn flux_defense_evaluate_network_connection(
    remote_ip_ptr: *const c_char,
    remote_port: u16,
    domain_ptr: *const c_char,
) -> bool {
    if remote_ip_ptr.is_null() {
        warn!("Received null remote IP pointer");
        return false;
    }
    
    let ip_cstr = unsafe { CStr::from_ptr(remote_ip_ptr) };
    let ip_str = match ip_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            warn!("Invalid UTF-8 in IP address");
            return false;
        }
    };
    
    let remote_ip = match ip_str.parse() {
        Ok(ip) => ip,
        Err(_) => {
            warn!("Invalid IP address format: {}", ip_str);
            return false;
        }
    };
    
    let domain = if domain_ptr.is_null() {
        None
    } else {
        let domain_cstr = unsafe { CStr::from_ptr(domain_ptr) };
        match domain_cstr.to_str() {
            Ok(s) => Some(s),
            Err(_) => {
                warn!("Invalid UTF-8 in domain");
                None
            }
        }
    };
    
    info!("Evaluating network connection: ip={}, port={}, domain={:?}", 
          remote_ip, remote_port, domain);
    
    unsafe {
        if let Some(ref defense) = FLUX_DEFENSE {
            defense.network_policy.is_connection_allowed(remote_ip, remote_port, domain)
        } else {
            warn!("FluxDefense not initialized, denying by default");
            false
        }
    }
}

// Get the current file policy as JSON
#[no_mangle]
pub extern "C" fn flux_defense_get_file_policy() -> *mut c_char {
    unsafe {
        if let Some(ref defense) = FLUX_DEFENSE {
            match serde_json::to_string(&defense.file_policy) {
                Ok(json) => {
                    match CString::new(json) {
                        Ok(cstring) => cstring.into_raw(),
                        Err(_) => std::ptr::null_mut(),
                    }
                }
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

// Get the current network policy as JSON
#[no_mangle]
pub extern "C" fn flux_defense_get_network_policy() -> *mut c_char {
    unsafe {
        if let Some(ref defense) = FLUX_DEFENSE {
            match serde_json::to_string(&defense.network_policy) {
                Ok(json) => {
                    match CString::new(json) {
                        Ok(cstring) => cstring.into_raw(),
                        Err(_) => std::ptr::null_mut(),
                    }
                }
                Err(_) => std::ptr::null_mut(),
            }
        } else {
            std::ptr::null_mut()
        }
    }
}

// Free a string allocated by Rust
#[no_mangle]
pub extern "C" fn flux_defense_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}