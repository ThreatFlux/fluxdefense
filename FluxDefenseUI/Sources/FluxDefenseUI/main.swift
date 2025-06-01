import SwiftUI
import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var popover: NSPopover?
    var hostingView: NSHostingView<ContentView>?
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        // Hide dock icon
        NSApp.setActivationPolicy(.accessory)
        
        // Create status bar item
        setupStatusBarItem()
        
        // Create popover
        setupPopover()
        
        // Start system monitoring
        SystemMonitor.shared.startMonitoring()
        FluxDefenseManager.shared.startMonitoring()
    }
    
    private func setupStatusBarItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        
        if let button = statusItem?.button {
            // Start with default shield icon
            Task { @MainActor in
                updateStatusBarContent()
            }
            button.action = #selector(togglePopover)
            button.target = self
            
            // Set up timer to update status bar every 3 seconds
            Timer.scheduledTimer(withTimeInterval: 3.0, repeats: true) { _ in
                Task { @MainActor in
                    self.updateStatusBarContent()
                }
            }
        }
    }
    
    @MainActor private func updateStatusBarContent() {
        guard let button = statusItem?.button else { return }
        
        let monitor = SystemMonitor.shared
        print("🔄 Updating status bar - CPU: \(monitor.cpuUsage)%, Memory: \(monitor.memoryUsage)%")
        
        // Create attributed string with icons and percentages
        let attributedString = NSMutableAttributedString()
        
        // CPU icon and percentage - make it more visible
        let cpuIcon = getColoredIcon("cpu", color: monitor.cpuStatusColor)
        attributedString.append(NSAttributedString(string: cpuIcon))
        attributedString.append(NSAttributedString(string: String(format: "%.0f%% ", monitor.cpuUsage)))
        
        // Memory icon and percentage with total usage
        let memIcon = getColoredIcon("memorychip", color: monitor.memoryStatusColor)
        attributedString.append(NSAttributedString(string: memIcon))
        
        // Format memory usage as "XX% (Y.YGB)"
        let memoryGB = monitor.memoryUsed / (1024 * 1024 * 1024)
        let memoryString = String(format: "%.0f%% (%.1fGB) ", monitor.memoryUsage, memoryGB)
        attributedString.append(NSAttributedString(string: memoryString))
        
        // Network icon and activity indicator
        let netIcon = getColoredIcon("network", color: monitor.networkStatusColor)
        attributedString.append(NSAttributedString(string: netIcon))
        let networkActivity = (monitor.networkInRate + monitor.networkOutRate) / (1024 * 1024) // MB/s
        if networkActivity > 1.0 {
            attributedString.append(NSAttributedString(string: String(format: "%.0fM ", networkActivity)))
        } else {
            attributedString.append(NSAttributedString(string: "● "))
        }
        
        // Disk icon and activity indicator
        let diskIcon = getColoredIcon("externaldrive", color: monitor.diskStatusColor)
        attributedString.append(NSAttributedString(string: diskIcon))
        let diskActivity = (monitor.diskReadRate + monitor.diskWriteRate) / (1024 * 1024) // MB/s
        if diskActivity > 1.0 {
            attributedString.append(NSAttributedString(string: String(format: "%.0fM", diskActivity)))
        } else {
            attributedString.append(NSAttributedString(string: "●"))
        }
        
        // Set the attributed string as button title
        button.attributedTitle = attributedString
        button.image = nil // Remove image to show text
        
        // Set tooltip with detailed info
        button.toolTip = createDetailedTooltip()
    }
    
    private func getColoredIcon(_ systemName: String, color: String) -> String {
        // Use more visible Unicode symbols and text labels
        switch systemName {
        case "cpu":
            let colorIcon = color == "red" ? "🔴" : (color == "yellow" ? "🟡" : "🟢")
            return "CPU\(colorIcon)"
        case "memorychip":
            let colorIcon = color == "red" ? "🔴" : (color == "yellow" ? "🟡" : "🟢")  
            return "RAM\(colorIcon)"
        case "network":
            let colorIcon = color == "red" ? "🔴" : (color == "yellow" ? "🟡" : "🟢")
            return "NET\(colorIcon)"
        case "externaldrive":
            let colorIcon = color == "red" ? "🔴" : (color == "yellow" ? "🟡" : "🟢")
            return "DISK\(colorIcon)"
        default:
            return "●"
        }
    }
    
    @MainActor private func createDetailedTooltip() -> String {
        let monitor = SystemMonitor.shared
        
        var tooltip = "FluxDefense System Monitor\n"
        tooltip += "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
        tooltip += String(format: "🖥️  CPU: %.1f%%\n", monitor.cpuUsage)
        
        // Detailed memory information
        let memoryUsedGB = monitor.memoryUsed / (1024 * 1024 * 1024)
        let memoryTotalGB = monitor.memoryTotal / (1024 * 1024 * 1024)
        tooltip += String(format: "🧠 Memory: %.1f%% (%.1f GB / %.1f GB)\n", 
                         monitor.memoryUsage, memoryUsedGB, memoryTotalGB)
        
        tooltip += String(format: "💾 Disk: %.1f%% used\n", monitor.diskUsage)
        
        let networkIn = monitor.networkInRate / (1024 * 1024)
        let networkOut = monitor.networkOutRate / (1024 * 1024)
        tooltip += String(format: "📶 Network: ↓%.1fMB/s ↑%.1fMB/s\n", networkIn, networkOut)
        
        let diskRead = monitor.diskReadRate / (1024 * 1024)
        let diskWrite = monitor.diskWriteRate / (1024 * 1024)
        tooltip += String(format: "💿 Disk I/O: R%.1fMB/s W%.1fMB/s\n", diskRead, diskWrite)
        
        tooltip += "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n"
        tooltip += "Click to open FluxDefense dashboard"
        
        return tooltip
    }
    
    private func setupPopover() {
        popover = NSPopover()
        popover?.contentSize = NSSize(width: 400, height: 500)
        popover?.behavior = .transient
        
        let contentView = ContentView()
        hostingView = NSHostingView(rootView: contentView)
        popover?.contentViewController = NSViewController()
        popover?.contentViewController?.view = hostingView!
    }
    
    @objc func togglePopover() {
        guard let popover = popover else { return }
        
        if popover.isShown {
            popover.performClose(nil)
        } else {
            if let button = statusItem?.button {
                popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
            }
        }
    }
}

// Main entry point
let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.run()