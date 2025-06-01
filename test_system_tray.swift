#!/usr/bin/env swift

import Foundation
import AppKit

// Simple test to verify system tray functionality
class SystemTrayTest: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var updateTimer: Timer?
    
    func applicationDidFinishLaunching(_ notification: Notification) {
        print("🚀 Starting system tray test...")
        
        // Hide dock icon
        NSApp.setActivationPolicy(.accessory)
        
        // Create status bar item
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        
        if let button = statusItem?.button {
            button.title = "🔄 Loading..."
            button.toolTip = "FluxDefense System Monitor Test"
        }
        
        // Update every 3 seconds
        updateTimer = Timer.scheduledTimer(withTimeInterval: 3.0, repeats: true) { _ in
            self.updateStatusBar()
        }
        
        // Initial update
        updateStatusBar()
        
        print("✅ System tray test started. Look for status in menu bar.")
    }
    
    func updateStatusBar() {
        guard let button = statusItem?.button else { return }
        
        print("🔄 Updating status bar...")
        
        // Test direct command execution
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/Users/vtriple/fluxdefense/target/release/flux-monitor")
        task.arguments = ["metrics", "--json", "--once"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            
            print("📊 Raw output: \(output.prefix(100))...")
            
            // Try to parse JSON
            if let jsonData = output.data(using: .utf8),
               let json = try? JSONSerialization.jsonObject(with: jsonData) as? [String: Any],
               let cpuUsage = json["cpu_usage"] as? Double,
               let memoryUsage = json["memory_usage"] as? Double {
                
                print("✅ Parsed metrics: CPU=\(cpuUsage)%, Memory=\(memoryUsage)%")
                
                // Update button with real data
                let cpuColor = cpuUsage > 80 ? "🔴" : (cpuUsage > 60 ? "🟡" : "🟢")
                let memColor = memoryUsage > 80 ? "🔴" : (memoryUsage > 60 ? "🟡" : "🟢")
                
                button.title = "\(cpuColor)\(Int(cpuUsage))% \(memColor)\(Int(memoryUsage))%"
                button.toolTip = """
                FluxDefense System Monitor
                ━━━━━━━━━━━━━━━━━━━━━━━━━━━
                🖥️  CPU: \(String(format: "%.1f", cpuUsage))%
                🧠 Memory: \(String(format: "%.1f", memoryUsage))%
                📊 Data: Real-time from Rust backend
                """
                
            } else {
                print("❌ Failed to parse JSON metrics")
                button.title = "❌ Parse Error"
            }
            
        } catch {
            print("❌ Failed to run command: \(error)")
            button.title = "❌ Exec Error"
        }
    }
}

// Main entry point
let app = NSApplication.shared
let delegate = SystemTrayTest()
app.delegate = delegate

print("🎯 Running system tray test...")
app.run()