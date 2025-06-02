#!/usr/bin/env python3
"""
FluxDefense Linux System Tray
A system tray application for Linux that displays real-time system metrics
"""

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('AppIndicator3', '0.1')
from gi.repository import Gtk, AppIndicator3, GLib
import json
import subprocess
import threading
import time
import os
import sys

class SystemMetrics:
    def __init__(self):
        self.cpu_usage = 0.0
        self.memory_usage = 0.0
        self.memory_used_gb = 0.0
        self.network_rx_rate = 0.0
        self.network_tx_rate = 0.0
        self.disk_read_rate = 0.0
        self.disk_write_rate = 0.0
        self.load_average = [0.0, 0.0, 0.0]
        self.process_count = 0
        self.uptime_seconds = 0
        self.last_net_rx = 0
        self.last_net_tx = 0
        self.last_disk_read = 0
        self.last_disk_write = 0
        self.last_update = time.time()

    def update(self):
        """Update metrics from flux-monitor or /proc"""
        try:
            # Try flux-monitor first
            flux_path = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), 
                                   'target', 'release', 'flux-monitor')
            if os.path.exists(flux_path):
                result = subprocess.run([flux_path, 'metrics', '--json', '--once'], 
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    data = json.loads(result.stdout)
                    self.cpu_usage = data.get('cpu_usage', 0.0)
                    self.memory_usage = data.get('memory_usage', 0.0)
                    self.memory_used_gb = data.get('memory_used_gb', 0.0)
                    self.network_rx_rate = data.get('network_rx_rate', 0.0)
                    self.network_tx_rate = data.get('network_tx_rate', 0.0)
                    self.disk_read_rate = data.get('disk_read_rate', 0.0)
                    self.disk_write_rate = data.get('disk_write_rate', 0.0)
                    self.load_average = data.get('load_average', [0.0, 0.0, 0.0])
                    self.process_count = data.get('process_count', 0)
                    self.uptime_seconds = data.get('uptime_seconds', 0)
                    return
        except Exception as e:
            print(f"Error getting metrics from flux-monitor: {e}")

        # Fallback to manual parsing
        self._update_from_proc()

    def _update_from_proc(self):
        """Update metrics by reading /proc files directly"""
        current_time = time.time()
        time_delta = current_time - self.last_update
        
        # CPU usage
        try:
            with open('/proc/stat', 'r') as f:
                line = f.readline()
                cpu_times = list(map(int, line.split()[1:8]))
                total = sum(cpu_times)
                idle = cpu_times[3] + cpu_times[4]  # idle + iowait
                self.cpu_usage = ((total - idle) / total) * 100 if total > 0 else 0
        except:
            pass

        # Memory usage
        try:
            with open('/proc/meminfo', 'r') as f:
                meminfo = {}
                for line in f:
                    parts = line.split()
                    if len(parts) >= 2:
                        meminfo[parts[0].rstrip(':')] = int(parts[1])
                
                total = meminfo.get('MemTotal', 1)
                available = meminfo.get('MemAvailable', 0)
                used = total - available
                self.memory_usage = (used / total) * 100 if total > 0 else 0
                self.memory_used_gb = used / 1024 / 1024
        except:
            pass

        # Network stats
        try:
            with open('/proc/net/dev', 'r') as f:
                lines = f.readlines()[2:]  # Skip headers
                total_rx = 0
                total_tx = 0
                for line in lines:
                    if ':' in line:
                        iface, stats = line.split(':', 1)
                        iface = iface.strip()
                        # Skip loopback and virtual interfaces
                        if iface not in ['lo', 'docker0', 'virbr0']:
                            values = stats.split()
                            if len(values) >= 9:
                                total_rx += int(values[0])
                                total_tx += int(values[8])
                
                if time_delta > 0:
                    self.network_rx_rate = (total_rx - self.last_net_rx) / time_delta if self.last_net_rx > 0 else 0
                    self.network_tx_rate = (total_tx - self.last_net_tx) / time_delta if self.last_net_tx > 0 else 0
                self.last_net_rx = total_rx
                self.last_net_tx = total_tx
        except:
            pass

        # Disk stats
        try:
            with open('/proc/diskstats', 'r') as f:
                total_read = 0
                total_write = 0
                for line in f:
                    parts = line.split()
                    if len(parts) >= 10:
                        device = parts[2]
                        # Only count physical disks
                        if device.startswith('sd') or device.startswith('nvme'):
                            if not any(c.isdigit() for c in device[-1:]):  # Skip partitions
                                total_read += int(parts[5]) * 512  # sectors to bytes
                                total_write += int(parts[9]) * 512
                
                if time_delta > 0:
                    self.disk_read_rate = (total_read - self.last_disk_read) / time_delta if self.last_disk_read > 0 else 0
                    self.disk_write_rate = (total_write - self.last_disk_write) / time_delta if self.last_disk_write > 0 else 0
                self.last_disk_read = total_read
                self.last_disk_write = total_write
        except:
            pass

        # Load average
        try:
            with open('/proc/loadavg', 'r') as f:
                parts = f.read().split()
                self.load_average = [float(parts[i]) for i in range(3)]
        except:
            pass

        # Process count
        try:
            proc_count = sum(1 for name in os.listdir('/proc') if name.isdigit())
            self.process_count = proc_count
        except:
            pass

        # Uptime
        try:
            with open('/proc/uptime', 'r') as f:
                self.uptime_seconds = int(float(f.read().split()[0]))
        except:
            pass

        self.last_update = current_time


class FluxDefenseTray:
    def __init__(self):
        self.metrics = SystemMetrics()
        self.indicator = AppIndicator3.Indicator.new(
            "fluxdefense-tray",
            "security-high",
            AppIndicator3.IndicatorCategory.SYSTEM_SERVICES
        )
        self.indicator.set_status(AppIndicator3.IndicatorStatus.ACTIVE)
        
        # Create menu
        self.menu = Gtk.Menu()
        
        # Status item
        self.status_item = Gtk.MenuItem(label="FluxDefense System Monitor")
        self.status_item.set_sensitive(False)
        self.menu.append(self.status_item)
        
        self.menu.append(Gtk.SeparatorMenuItem())
        
        # Dashboard item
        dashboard_item = Gtk.MenuItem(label="Open Dashboard")
        dashboard_item.connect("activate", self.open_dashboard)
        self.menu.append(dashboard_item)
        
        # Refresh item
        refresh_item = Gtk.MenuItem(label="Refresh Now")
        refresh_item.connect("activate", self.refresh_metrics)
        self.menu.append(refresh_item)
        
        self.menu.append(Gtk.SeparatorMenuItem())
        
        # About item
        about_item = Gtk.MenuItem(label="About")
        about_item.connect("activate", self.show_about)
        self.menu.append(about_item)
        
        # Quit item
        quit_item = Gtk.MenuItem(label="Quit")
        quit_item.connect("activate", self.quit)
        self.menu.append(quit_item)
        
        self.menu.show_all()
        self.indicator.set_menu(self.menu)
        
        # Start update thread
        self.running = True
        self.update_thread = threading.Thread(target=self.update_loop, daemon=True)
        self.update_thread.start()
        
        # Schedule UI updates
        GLib.timeout_add_seconds(3, self.update_ui)

    def get_status_color(self, value, warning, critical):
        """Get color emoji based on threshold"""
        if value >= critical:
            return "🔴"
        elif value >= warning:
            return "🟡"
        else:
            return "🟢"

    def format_rate(self, bytes_per_sec):
        """Format network/disk rate"""
        if bytes_per_sec < 1024:
            return "●"
        elif bytes_per_sec < 1024 * 1024:
            return f"{int(bytes_per_sec / 1024)}K"
        else:
            return f"{int(bytes_per_sec / 1024 / 1024)}M"

    def update_ui(self):
        """Update the UI with current metrics"""
        m = self.metrics
        
        # Create status text
        cpu_color = self.get_status_color(m.cpu_usage, 70, 90)
        mem_color = self.get_status_color(m.memory_usage, 80, 95)
        
        net_status = f"{self.format_rate(max(m.network_rx_rate, m.network_tx_rate))}🟢"
        disk_status = f"{self.format_rate(max(m.disk_read_rate, m.disk_write_rate))}🟢"
        
        status_text = f"CPU{cpu_color}{int(m.cpu_usage)}% RAM{mem_color}{int(m.memory_usage)}% ({m.memory_used_gb:.1f}GB) NET{net_status} DISK{disk_status}"
        
        # Update label
        self.indicator.set_label(status_text, "FluxDefense")
        
        # Create tooltip
        hours = m.uptime_seconds // 3600
        tooltip = f"""FluxDefense System Monitor
────────────────────────
CPU Usage: {m.cpu_usage:.1f}%
Memory: {m.memory_usage:.1f}% ({m.memory_used_gb:.2f} GB used)
Network: ↓{m.network_rx_rate/1024/1024:.1f} MB/s ↑{m.network_tx_rate/1024/1024:.1f} MB/s
Disk: R{m.disk_read_rate/1024/1024:.1f} MB/s W{m.disk_write_rate/1024/1024:.1f} MB/s
Load: {m.load_average[0]:.2f} {m.load_average[1]:.2f} {m.load_average[2]:.2f}
Processes: {m.process_count}
Uptime: {hours} hours"""
        
        self.status_item.set_label(tooltip)
        
        return True  # Continue timeout

    def update_loop(self):
        """Background thread to update metrics"""
        while self.running:
            self.metrics.update()
            time.sleep(3)

    def open_dashboard(self, widget):
        """Open the dashboard in a browser"""
        try:
            subprocess.run(["xdg-open", "http://localhost:8080"], check=False)
        except Exception as e:
            print(f"Failed to open dashboard: {e}")

    def refresh_metrics(self, widget):
        """Force refresh metrics"""
        self.metrics.update()
        self.update_ui()

    def show_about(self, widget):
        """Show about dialog"""
        dialog = Gtk.MessageDialog(
            parent=None,
            flags=0,
            message_type=Gtk.MessageType.INFO,
            buttons=Gtk.ButtonsType.OK,
            text="FluxDefense System Tray"
        )
        dialog.format_secondary_text(
            "Real-time system monitoring for Linux\n\n"
            "Part of the FluxDefense security suite\n"
            "https://github.com/fluxdefense/fluxdefense"
        )
        dialog.run()
        dialog.destroy()

    def quit(self, widget):
        """Quit the application"""
        self.running = False
        Gtk.main_quit()


def main():
    # Check if we're on Linux
    if sys.platform != 'linux':
        print("This system tray implementation is Linux-specific.")
        print("For macOS, use the FluxDefenseUI Swift application.")
        sys.exit(1)
    
    # Check for required dependencies
    try:
        import gi
        gi.require_version('Gtk', '3.0')
        gi.require_version('AppIndicator3', '0.1')
    except Exception as e:
        print("Error: Missing required dependencies")
        print("Please install: sudo apt-get install python3-gi gir1.2-appindicator3-0.1")
        sys.exit(1)
    
    # Create and run the tray application
    app = FluxDefenseTray()
    Gtk.main()


if __name__ == "__main__":
    main()