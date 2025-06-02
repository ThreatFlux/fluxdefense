use anyhow::Result;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
use gtk::prelude::*;
#[cfg(target_os = "linux")]
use libappindicator::{AppIndicator, AppIndicatorStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemMetrics {
    cpu_usage: f64,
    memory_usage: f64,
    memory_used_gb: f64,
    network_rx_rate: f64,
    network_tx_rate: f64,
    disk_read_rate: f64,
    disk_write_rate: f64,
    load_average: Vec<f64>,
    process_count: u32,
    uptime_seconds: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_used_gb: 0.0,
            network_rx_rate: 0.0,
            network_tx_rate: 0.0,
            disk_read_rate: 0.0,
            disk_write_rate: 0.0,
            load_average: vec![0.0; 3],
            process_count: 0,
            uptime_seconds: 0,
        }
    }
}

fn get_system_metrics() -> Result<SystemMetrics> {
    // Try to get metrics from flux-monitor first
    let output = Command::new("./target/release/flux-monitor")
        .args(&["metrics", "--json", "--once"])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let json_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(metrics) = serde_json::from_str::<SystemMetrics>(&json_str) {
                return Ok(metrics);
            }
        }
    }

    // Fallback to reading from /proc
    let mut metrics = SystemMetrics::default();

    // CPU usage from /proc/stat
    if let Ok(stat) = std::fs::read_to_string("/proc/stat") {
        let first_line = stat.lines().next().unwrap_or("");
        let values: Vec<u64> = first_line
            .split_whitespace()
            .skip(1)
            .filter_map(|v| v.parse().ok())
            .collect();
        
        if values.len() >= 4 {
            let total = values.iter().sum::<u64>();
            let idle = values[3];
            if total > 0 {
                metrics.cpu_usage = ((total - idle) as f64 / total as f64) * 100.0;
            }
        }
    }

    // Memory from /proc/meminfo
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
            }
        }
        
        if total_kb > 0 {
            let used_kb = total_kb - available_kb;
            metrics.memory_usage = (used_kb as f64 / total_kb as f64) * 100.0;
            metrics.memory_used_gb = used_kb as f64 / 1024.0 / 1024.0;
        }
    }

    // Load average from /proc/loadavg
    if let Ok(loadavg) = std::fs::read_to_string("/proc/loadavg") {
        let parts: Vec<f64> = loadavg
            .split_whitespace()
            .take(3)
            .filter_map(|v| v.parse().ok())
            .collect();
        metrics.load_average = parts;
    }

    // Process count
    if let Ok(entries) = std::fs::read_dir("/proc") {
        metrics.process_count = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_str().map(|s| s.chars().all(|c| c.is_numeric())).unwrap_or(false))
            .count() as u32;
    }

    Ok(metrics)
}

fn get_status_color(value: f64, warning_threshold: f64, critical_threshold: f64) -> &'static str {
    if value >= critical_threshold {
        "🔴"
    } else if value >= warning_threshold {
        "🟡"
    } else {
        "🟢"
    }
}

fn format_rate(bytes_per_sec: f64) -> String {
    if bytes_per_sec < 1024.0 {
        "●".to_string()
    } else if bytes_per_sec < 1024.0 * 1024.0 {
        format!("{}K", (bytes_per_sec / 1024.0) as u32)
    } else {
        format!("{}M", (bytes_per_sec / 1024.0 / 1024.0) as u32)
    }
}

fn create_status_text(metrics: &SystemMetrics) -> String {
    let cpu_color = get_status_color(metrics.cpu_usage, 70.0, 90.0);
    let mem_color = get_status_color(metrics.memory_usage, 80.0, 95.0);
    
    let net_status = if metrics.network_rx_rate > 1024.0 || metrics.network_tx_rate > 1024.0 {
        format!("{}🟢", format_rate(metrics.network_rx_rate.max(metrics.network_tx_rate)))
    } else {
        "●🟢".to_string()
    };
    
    let disk_status = if metrics.disk_read_rate > 1024.0 || metrics.disk_write_rate > 1024.0 {
        format!("{}🟢", format_rate(metrics.disk_read_rate.max(metrics.disk_write_rate)))
    } else {
        "●🟢".to_string()
    };

    format!(
        "CPU{}{:.0}% RAM{}{:.0}% ({:.1}GB) NET{} DISK{}",
        cpu_color, metrics.cpu_usage,
        mem_color, metrics.memory_usage, metrics.memory_used_gb,
        net_status,
        disk_status
    )
}

fn create_tooltip_text(metrics: &SystemMetrics) -> String {
    format!(
        "FluxDefense System Monitor\n\
         ────────────────────────\n\
         CPU Usage: {:.1}%\n\
         Memory: {:.1}% ({:.2} GB used)\n\
         Network: ↓{:.1} MB/s ↑{:.1} MB/s\n\
         Disk: R{:.1} MB/s W{:.1} MB/s\n\
         Load: {:.2} {:.2} {:.2}\n\
         Processes: {}\n\
         Uptime: {} hours",
        metrics.cpu_usage,
        metrics.memory_usage, metrics.memory_used_gb,
        metrics.network_rx_rate / 1024.0 / 1024.0,
        metrics.network_tx_rate / 1024.0 / 1024.0,
        metrics.disk_read_rate / 1024.0 / 1024.0,
        metrics.disk_write_rate / 1024.0 / 1024.0,
        metrics.load_average.get(0).unwrap_or(&0.0),
        metrics.load_average.get(1).unwrap_or(&0.0),
        metrics.load_average.get(2).unwrap_or(&0.0),
        metrics.process_count,
        metrics.uptime_seconds / 3600
    )
}

#[cfg(target_os = "linux")]
fn main() -> Result<()> {
    // Initialize GTK
    gtk::init()?;

    // Create the app indicator
    let mut indicator = AppIndicator::new("fluxdefense-tray", "");
    indicator.set_status(AppIndicatorStatus::Active);
    indicator.set_icon("security-high");

    // Create menu
    let menu = gtk::Menu::new();
    
    // Status item (will be updated with metrics)
    let status_item = gtk::MenuItem::with_label("FluxDefense System Monitor");
    status_item.set_sensitive(false);
    menu.append(&status_item);
    
    menu.append(&gtk::SeparatorMenuItem::new());
    
    // Show Dashboard item
    let dashboard_item = gtk::MenuItem::with_label("Show Dashboard");
    let dashboard_item_clone = dashboard_item.clone();
    dashboard_item.connect_activate(move |_| {
        // Launch the dashboard in a browser
        if let Err(e) = Command::new("xdg-open")
            .arg("http://localhost:8080")
            .spawn()
        {
            eprintln!("Failed to open dashboard: {}", e);
        }
    });
    menu.append(&dashboard_item);
    
    menu.append(&gtk::SeparatorMenuItem::new());
    
    // Quit item
    let quit_item = gtk::MenuItem::with_label("Quit");
    quit_item.connect_activate(|_| {
        gtk::main_quit();
    });
    menu.append(&quit_item);
    
    menu.show_all();
    indicator.set_menu(&mut menu);

    // Shared metrics state
    let metrics = Arc::new(Mutex::new(SystemMetrics::default()));
    let metrics_clone = metrics.clone();

    // Start metrics update thread
    thread::spawn(move || {
        loop {
            if let Ok(new_metrics) = get_system_metrics() {
                if let Ok(mut m) = metrics_clone.lock() {
                    *m = new_metrics;
                }
            }
            thread::sleep(Duration::from_secs(3));
        }
    });

    // Update the display periodically
    let status_item_clone = status_item.clone();
    gtk::timeout_add(3000, move || {
        if let Ok(m) = metrics.lock() {
            let status_text = create_status_text(&m);
            let tooltip_text = create_tooltip_text(&m);
            
            status_item_clone.set_label(&status_text);
            
            // Set tooltip on the menu item
            if let Some(widget) = status_item_clone.get_child() {
                widget.set_tooltip_text(Some(&tooltip_text));
            }
        }
        gtk::Continue(true)
    });

    // Run GTK main loop
    gtk::main();
    
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("This system tray implementation is Linux-specific.");
    eprintln!("For macOS, use the FluxDefenseUI Swift application.");
    std::process::exit(1);
}