import Foundation
import SwiftUI
import IOKit
import IOKit.ps

// MARK: - System Metrics Data Structures

struct SystemMetrics: Codable {
    let timestamp: UInt64
    let cpuUsage: Double
    let memoryUsage: Double
    let memoryTotal: UInt64
    let memoryUsed: UInt64
    let diskReadBytes: UInt64
    let diskWriteBytes: UInt64
    let diskReadRate: Double
    let diskWriteRate: Double
    let networkRxBytes: UInt64
    let networkTxBytes: UInt64
    let networkRxRate: Double
    let networkTxRate: Double
    let loadAverage: [Double]
    let processCount: UInt32
    let uptimeSeconds: UInt64
    
    enum CodingKeys: String, CodingKey {
        case timestamp, cpuUsage = "cpu_usage", memoryUsage = "memory_usage"
        case memoryTotal = "memory_total", memoryUsed = "memory_used"
        case diskReadBytes = "disk_read_bytes", diskWriteBytes = "disk_write_bytes"
        case diskReadRate = "disk_read_rate", diskWriteRate = "disk_write_rate"
        case networkRxBytes = "network_rx_bytes", networkTxBytes = "network_tx_bytes"
        case networkRxRate = "network_rx_rate", networkTxRate = "network_tx_rate"
        case loadAverage = "load_average", processCount = "process_count"
        case uptimeSeconds = "uptime_seconds"
    }
}

@MainActor
class SystemMonitor: ObservableObject {
    static let shared = SystemMonitor()
    
    // MARK: - Published Properties
    @Published var cpuUsage: Double = 0.0
    @Published var memoryUsage: Double = 0.0
    @Published var memoryTotal: Double = 0.0  // Total RAM in bytes
    @Published var memoryUsed: Double = 0.0   // Used RAM in bytes
    @Published var diskUsage: Double = 0.0
    @Published var networkInRate: Double = 0.0
    @Published var networkOutRate: Double = 0.0
    @Published var totalNetworkIn: Double = 0.0
    @Published var totalNetworkOut: Double = 0.0
    @Published var diskReadRate: Double = 0.0
    @Published var diskWriteRate: Double = 0.0
    @Published var totalDiskRead: Double = 0.0
    @Published var totalDiskWrite: Double = 0.0
    
    // Status indicators for system tray
    @Published var cpuStatusColor: String = "green"
    @Published var memoryStatusColor: String = "green"
    @Published var diskStatusColor: String = "green"
    @Published var networkStatusColor: String = "green"
    
    // Historical data for charts
    @Published var cpuHistory: [Double] = []
    @Published var memoryHistory: [Double] = []
    @Published var networkHistory: [Double] = []
    @Published var diskHistory: [Double] = []
    
    // Process information
    @Published var topProcesses: [ProcessInformation] = []
    @Published var allProcesses: [ProcessInformation] = []
    @Published var activeProcessCount: Int = 0
    
    // Process user information
    var user: String? {
        return ProcessInfo.processInfo.userName
    }
    
    private var updateTimer: Timer?
    private var isMonitoring = false
    private var timeRange: Int = 3600 // Default 1 hour
    private let maxHistoryPoints = 100
    
    // Previous values for rate calculations
    private var previousNetworkIn: Double = 0.0
    private var previousNetworkOut: Double = 0.0
    private var previousDiskRead: Double = 0.0
    private var previousDiskWrite: Double = 0.0
    private var lastUpdateTime = Date()
    
    init() {
        // Initialize with some default values
        cpuUsage = Double.random(in: 5...25)
        memoryUsage = Double.random(in: 30...60)
        diskUsage = Double.random(in: 40...80)
        
        // Initialize memory values (assuming 16GB total for demo)
        memoryTotal = Double(ProcessInfo.processInfo.physicalMemory)
        memoryUsed = memoryTotal * (memoryUsage / 100.0)
        
        // Initialize history with some sample data
        for _ in 0..<20 {
            cpuHistory.append(Double.random(in: 5...25))
            memoryHistory.append(Double.random(in: 30...60))
            networkHistory.append(Double.random(in: 0...100))
            diskHistory.append(Double.random(in: 0...50))
        }
        
        loadSampleProcesses()
    }
    
    // MARK: - Public Methods
    
    func startMonitoring() {
        guard !isMonitoring else { return }
        isMonitoring = true
        
        // Update every 2 seconds
        updateTimer = Timer.scheduledTimer(withTimeInterval: 2.0, repeats: true) { _ in
            Task { @MainActor in
                self.updateSystemMetrics()
            }
        }
        
        print("System monitoring started")
    }
    
    func stopMonitoring() {
        isMonitoring = false
        updateTimer?.invalidate()
        updateTimer = nil
        
        print("System monitoring stopped")
    }
    
    func updateTimeRange(_ seconds: Int) {
        timeRange = seconds
        // Adjust history based on time range
        let targetPoints = min(maxHistoryPoints, timeRange / 30) // One point every 30 seconds
        
        if cpuHistory.count > targetPoints {
            cpuHistory = Array(cpuHistory.suffix(targetPoints))
            memoryHistory = Array(memoryHistory.suffix(targetPoints))
            networkHistory = Array(networkHistory.suffix(targetPoints))
            diskHistory = Array(diskHistory.suffix(targetPoints))
        }
    }
    
    func refreshProcessList() {
        updateProcessList()
    }
    
    // MARK: - Private Methods
    
    private func updateSystemMetrics() {
        // Try to fetch real metrics from Rust backend first
        if let realMetrics = fetchRustBackendMetrics() {
            updateFromRealMetrics(realMetrics)
        } else {
            // Fallback to simulated metrics
            updateCPUUsage()
            updateMemoryUsage()
            updateDiskUsage()
            updateNetworkMetrics()
            updateDiskIOMetrics()
        }
        
        updateProcessList()
        updateHistoricalData()
        updateStatusColors()
    }
    
    // MARK: - Real Metrics from Rust Backend
    
    private func fetchRustBackendMetrics() -> SystemMetrics? {
        do {
            // Execute flux-monitor to get current metrics in JSON format
            let task = Process()
            task.executableURL = URL(fileURLWithPath: "/Users/vtriple/fluxdefense/target/release/flux-monitor")
            task.arguments = ["metrics", "--json", "--once"]
            
            let pipe = Pipe()
            task.standardOutput = pipe
            task.standardError = Pipe() // Suppress error output
            
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            
            // Try to parse as JSON first
            if let metrics = try? JSONDecoder().decode(SystemMetrics.self, from: data) {
                print("✅ Successfully parsed real metrics from Rust backend")
                return metrics
            }
            
            // Fallback to text parsing if JSON fails
            let output = String(data: data, encoding: .utf8) ?? ""
            print("⚠️ JSON parsing failed, trying text parsing. Output: \(output.prefix(200))")
            return parseMetricsFromOutput(output)
            
        } catch {
            print("Failed to fetch real metrics: \(error)")
            return nil
        }
    }
    
    private func parseMetricsFromOutput(_ output: String) -> SystemMetrics? {
        // This is a simplified parser for the current text output
        // In production, you'd modify the Rust backend to output JSON
        
        var cpuUsage = 0.0
        var memoryUsage = 0.0
        var networkRx = 0.0
        var networkTx = 0.0
        var diskRead = 0.0
        var diskWrite = 0.0
        
        let lines = output.components(separatedBy: .newlines)
        
        for line in lines {
            if line.contains("Current:") && line.contains("%") {
                if let match = line.range(of: #"(\d+\.?\d*)%"#, options: .regularExpression) {
                    let percentString = String(line[match]).replacingOccurrences(of: "%", with: "")
                    cpuUsage = Double(percentString) ?? 0.0
                }
            } else if line.contains("Used:") && line.contains("%") {
                if let match = line.range(of: #"(\d+\.?\d*)%"#, options: .regularExpression) {
                    let percentString = String(line[match]).replacingOccurrences(of: "%", with: "")
                    memoryUsage = Double(percentString) ?? 0.0
                }
            } else if line.contains("RX Rate:") {
                // Parse network RX rate
                let components = line.components(separatedBy: " ")
                for (i, component) in components.enumerated() {
                    if component.contains("Rate:") && i + 1 < components.count {
                        let rateString = components[i + 1].replacingOccurrences(of: "MB/s", with: "")
                            .replacingOccurrences(of: "KB/s", with: "")
                            .replacingOccurrences(of: "GB/s", with: "")
                        if let rate = Double(rateString) {
                            if components[i + 1].contains("MB/s") {
                                networkRx = rate * 1024 * 1024
                            } else if components[i + 1].contains("KB/s") {
                                networkRx = rate * 1024
                            } else if components[i + 1].contains("GB/s") {
                                networkRx = rate * 1024 * 1024 * 1024
                            }
                        }
                    }
                }
            } else if line.contains("TX Rate:") {
                // Parse network TX rate
                let components = line.components(separatedBy: " ")
                for (i, component) in components.enumerated() {
                    if component.contains("Rate:") && i + 1 < components.count {
                        let rateString = components[i + 1].replacingOccurrences(of: "MB/s", with: "")
                            .replacingOccurrences(of: "KB/s", with: "")
                            .replacingOccurrences(of: "GB/s", with: "")
                        if let rate = Double(rateString) {
                            if components[i + 1].contains("MB/s") {
                                networkTx = rate * 1024 * 1024
                            } else if components[i + 1].contains("KB/s") {
                                networkTx = rate * 1024
                            } else if components[i + 1].contains("GB/s") {
                                networkTx = rate * 1024 * 1024 * 1024
                            }
                        }
                    }
                }
            }
        }
        
        // Create simplified metrics object
        return SystemMetrics(
            timestamp: UInt64(Date().timeIntervalSince1970),
            cpuUsage: cpuUsage,
            memoryUsage: memoryUsage,
            memoryTotal: 32 * 1024 * 1024 * 1024, // 32GB approximation
            memoryUsed: UInt64(Double(32 * 1024 * 1024 * 1024) * (memoryUsage / 100.0)),
            diskReadBytes: 0,
            diskWriteBytes: 0,
            diskReadRate: diskRead,
            diskWriteRate: diskWrite,
            networkRxBytes: 0,
            networkTxBytes: 0,
            networkRxRate: networkRx,
            networkTxRate: networkTx,
            loadAverage: [0.0, 0.0, 0.0],
            processCount: 500,
            uptimeSeconds: 3600
        )
    }
    
    private func updateFromRealMetrics(_ metrics: SystemMetrics) {
        print("📊 Updating UI with real metrics: CPU=\(metrics.cpuUsage)%, Memory=\(metrics.memoryUsage)%")
        
        cpuUsage = metrics.cpuUsage
        memoryUsage = metrics.memoryUsage
        memoryTotal = Double(metrics.memoryTotal)
        memoryUsed = Double(metrics.memoryUsed)
        networkInRate = metrics.networkRxRate
        networkOutRate = metrics.networkTxRate
        diskReadRate = metrics.diskReadRate
        diskWriteRate = metrics.diskWriteRate
        totalNetworkIn = Double(metrics.networkRxBytes)
        totalNetworkOut = Double(metrics.networkTxBytes)
        totalDiskRead = Double(metrics.diskReadBytes)
        totalDiskWrite = Double(metrics.diskWriteBytes)
        
        // Update disk usage from filesystem
        updateDiskUsage()
    }
    
    private func updateStatusColors() {
        // Update status colors based on thresholds
        cpuStatusColor = getStatusColor(for: cpuUsage, thresholds: (warning: 70, critical: 90))
        memoryStatusColor = getStatusColor(for: memoryUsage, thresholds: (warning: 80, critical: 95))
        
        let diskActivity = (diskReadRate + diskWriteRate) / (1024 * 1024) // MB/s
        diskStatusColor = getStatusColor(for: diskActivity, thresholds: (warning: 50, critical: 100))
        
        let networkActivity = (networkInRate + networkOutRate) / (1024 * 1024) // MB/s  
        networkStatusColor = getStatusColor(for: networkActivity, thresholds: (warning: 100, critical: 500))
    }
    
    private func getStatusColor(for value: Double, thresholds: (warning: Double, critical: Double)) -> String {
        if value >= thresholds.critical {
            return "red"
        } else if value >= thresholds.warning {
            return "yellow" 
        } else {
            return "green"
        }
    }
    
    private func updateCPUUsage() {
        #if os(macOS)
        // Get real CPU usage using top command
        let task = Process()
        task.launchPath = "/usr/bin/top"
        task.arguments = ["-l", "1", "-n", "0", "-s", "0"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                // Parse CPU usage from top output
                let lines = output.components(separatedBy: .newlines)
                for line in lines {
                    if line.contains("CPU usage:") {
                        // Extract user and sys percentages
                        var userPercent: Double = 0
                        var sysPercent: Double = 0
                        
                        if let userMatch = line.range(of: #"(\d+\.?\d*)% user"#, options: .regularExpression) {
                            let userString = String(line[userMatch]).replacingOccurrences(of: "% user", with: "")
                            userPercent = Double(userString) ?? 0
                        }
                        
                        if let sysMatch = line.range(of: #"(\d+\.?\d*)% sys"#, options: .regularExpression) {
                            let sysString = String(line[sysMatch]).replacingOccurrences(of: "% sys", with: "")
                            sysPercent = Double(sysString) ?? 0
                        }
                        
                        cpuUsage = userPercent + sysPercent
                        break
                    }
                }
            }
        } catch {
            // Fallback to load average
            var loadavg = [Double](repeating: 0, count: 3)
            let result = getloadavg(&loadavg, 3)
            if result > 0 {
                let cores = Double(Foundation.ProcessInfo.processInfo.processorCount)
                cpuUsage = min(100, (loadavg[0] / cores) * 100)
            }
        }
        #else
        // Non-macOS fallback
        cpuUsage = Double.random(in: 5...25)
        #endif
    }
    
    private func updateMemoryUsage() {
        #if os(macOS)
        // Get total physical memory
        memoryTotal = Double(Foundation.ProcessInfo.processInfo.physicalMemory)
        
        // Get actual memory usage using vm_stat
        let task = Process()
        task.launchPath = "/usr/bin/vm_stat"
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                // Parse vm_stat output
                let lines = output.components(separatedBy: .newlines)
                var pageSize: Int = 4096
                var freePages: Int = 0
                var activePages: Int = 0
                var inactivePages: Int = 0
                var wiredPages: Int = 0
                var compressedPages: Int = 0
                
                for line in lines {
                    if line.contains("page size of") {
                        let components = line.components(separatedBy: " ")
                        for (i, component) in components.enumerated() {
                            if component == "of" && i + 1 < components.count {
                                pageSize = Int(components[i + 1]) ?? 4096
                                break
                            }
                        }
                    } else if line.contains("Pages free:") {
                        freePages = extractPageCount(from: line)
                    } else if line.contains("Pages active:") {
                        activePages = extractPageCount(from: line)
                    } else if line.contains("Pages inactive:") {
                        inactivePages = extractPageCount(from: line)
                    } else if line.contains("Pages wired down:") {
                        wiredPages = extractPageCount(from: line)
                    } else if line.contains("Pages occupied by compressor:") {
                        compressedPages = extractPageCount(from: line)
                    }
                }
                
                // Calculate used memory
                let usedPages = activePages + inactivePages + wiredPages + compressedPages
                let totalPages = usedPages + freePages
                
                memoryUsed = Double(usedPages * pageSize)
                memoryUsage = (memoryUsed / memoryTotal) * 100.0
            }
        } catch {
            // Fallback to previous value
            print("Error getting memory stats: \(error)")
        }
        #else
        // Fallback for non-macOS platforms
        memoryUsed = memoryTotal * (memoryUsage / 100.0)
        #endif
    }
    
    private func extractPageCount(from line: String) -> Int {
        let components = line.components(separatedBy: .whitespaces)
        for component in components {
            if let value = Int(component.replacingOccurrences(of: ".", with: "")) {
                return value
            }
        }
        return 0
    }
    
    private func updateDiskUsage() {
        #if os(macOS)
        let fileManager = FileManager.default
        
        do {
            let attributes = try fileManager.attributesOfFileSystem(forPath: "/")
            if let totalSpace = attributes[.systemSize] as? NSNumber,
               let freeSpace = attributes[.systemFreeSize] as? NSNumber {
                let total = totalSpace.doubleValue
                let free = freeSpace.doubleValue
                let used = total - free
                diskUsage = (used / total) * 100
            }
        } catch {
            // Fallback to simulated data
            let baseUsage = diskUsage
            let variation = Double.random(in: -1...1)
            diskUsage = max(0, min(100, baseUsage + variation))
        }
        #endif
    }
    
    private func updateNetworkMetrics() {
        #if os(macOS)
        // Get real network statistics using nettop
        let task = Process()
        task.launchPath = "/usr/bin/nettop"
        task.arguments = ["-x", "-l", "1", "-t", "external"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            
            // Wait briefly for nettop to gather data
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                task.terminate()
            }
            
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                // Parse network bytes from nettop output
                var totalIn: Double = 0
                var totalOut: Double = 0
                
                let lines = output.components(separatedBy: .newlines)
                for line in lines {
                    // Look for lines with network data
                    if line.contains("bytes") {
                        let components = line.components(separatedBy: .whitespaces).filter { !$0.isEmpty }
                        
                        // Try to find bytes in/out values
                        for (index, component) in components.enumerated() {
                            if let value = Double(component.replacingOccurrences(of: ",", with: "")) {
                                if index > 0 && components[index-1].contains("in") {
                                    totalIn += value
                                } else if index > 0 && components[index-1].contains("out") {
                                    totalOut += value
                                }
                            }
                        }
                    }
                }
                
                let currentTime = Date()
                let timeDelta = currentTime.timeIntervalSince(lastUpdateTime)
                
                if timeDelta > 0 && previousNetworkIn > 0 && previousNetworkOut > 0 {
                    networkInRate = max(0, (totalIn - previousNetworkIn) / timeDelta)
                    networkOutRate = max(0, (totalOut - previousNetworkOut) / timeDelta)
                }
                
                previousNetworkIn = totalIn
                previousNetworkOut = totalOut
                totalNetworkIn = totalIn
                totalNetworkOut = totalOut
                lastUpdateTime = currentTime
            }
        } catch {
            print("Error getting network stats with nettop, trying netstat fallback: \(error)")
            updateNetworkMetricsUsingNetstat()
        }
        #else
        // Non-macOS fallback
        let currentTime = Date()
        let timeDelta = currentTime.timeIntervalSince(lastUpdateTime)
        
        let newInBytes = totalNetworkIn + Double.random(in: 1000...50000)
        let newOutBytes = totalNetworkOut + Double.random(in: 500...25000)
        
        if timeDelta > 0 {
            networkInRate = (newInBytes - totalNetworkIn) / timeDelta
            networkOutRate = (newOutBytes - totalNetworkOut) / timeDelta
        }
        
        totalNetworkIn = newInBytes
        totalNetworkOut = newOutBytes
        lastUpdateTime = currentTime
        #endif
    }
    
    private func updateNetworkMetricsUsingNetstat() {
        #if os(macOS)
        // Alternative method using netstat
        let task = Process()
        task.launchPath = "/usr/sbin/netstat"
        task.arguments = ["-ib"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                let lines = output.components(separatedBy: .newlines)
                
                var totalIn: Double = 0
                var totalOut: Double = 0
                
                for line in lines {
                    let components = line.components(separatedBy: .whitespaces).filter { !$0.isEmpty }
                    
                    // Look for network interfaces (skip loopback)
                    if components.count >= 11 && components[0] != "Name" && !components[0].starts(with: "lo") {
                        if let inBytes = Double(components[6]), let outBytes = Double(components[9]) {
                            totalIn += inBytes
                            totalOut += outBytes
                        }
                    }
                }
                
                let currentTime = Date()
                let timeDelta = currentTime.timeIntervalSince(lastUpdateTime)
                
                if timeDelta > 0 && previousNetworkIn > 0 && previousNetworkOut > 0 {
                    networkInRate = max(0, (totalIn - previousNetworkIn) / timeDelta)
                    networkOutRate = max(0, (totalOut - previousNetworkOut) / timeDelta)
                }
                
                previousNetworkIn = totalIn
                previousNetworkOut = totalOut
                totalNetworkIn = totalIn
                totalNetworkOut = totalOut
                lastUpdateTime = currentTime
            }
        } catch {
            print("Error getting network stats: \(error)")
        }
        #endif
    }
    
    private func updateDiskIOMetrics() {
        #if os(macOS)
        // Get real disk I/O statistics using iostat
        let task = Process()
        task.launchPath = "/usr/sbin/iostat"
        task.arguments = ["-c", "1", "-w", "1"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                let lines = output.components(separatedBy: .newlines)
                
                // iostat output typically has headers, then data
                // We want the last non-empty line which contains the latest stats
                for line in lines.reversed() {
                    let components = line.trimmingCharacters(in: .whitespaces)
                        .components(separatedBy: .whitespaces)
                        .filter { !$0.isEmpty }
                    
                    // Look for lines with numeric data (KB/t, tps, MB/s)
                    if components.count >= 6 {
                        // Try to parse MB/s values (usually last two columns)
                        if let mbRead = Double(components[components.count - 2]),
                           let mbWrite = Double(components[components.count - 1]) {
                            
                            // Convert MB/s to bytes/s
                            diskReadRate = mbRead * 1024 * 1024
                            diskWriteRate = mbWrite * 1024 * 1024
                            
                            // Update totals
                            let currentTime = Date()
                            let timeDelta = currentTime.timeIntervalSince(lastUpdateTime)
                            
                            if timeDelta > 0 {
                                totalDiskRead += diskReadRate * timeDelta
                                totalDiskWrite += diskWriteRate * timeDelta
                            }
                            
                            break
                        }
                    }
                }
            }
        } catch {
            print("Error getting disk I/O stats with iostat: \(error)")
            // Try alternative method
            updateDiskIOMetricsUsingDf()
        }
        #else
        // Non-macOS fallback
        let currentTime = Date()
        let timeDelta = currentTime.timeIntervalSince(lastUpdateTime)
        
        let newReadBytes = totalDiskRead + Double.random(in: 0...10000)
        let newWriteBytes = totalDiskWrite + Double.random(in: 0...5000)
        
        if timeDelta > 0 {
            diskReadRate = (newReadBytes - totalDiskRead) / timeDelta
            diskWriteRate = (newWriteBytes - totalDiskWrite) / timeDelta
        }
        
        totalDiskRead = newReadBytes
        totalDiskWrite = newWriteBytes
        #endif
    }
    
    private func updateDiskIOMetricsUsingDf() {
        #if os(macOS)
        // Alternative: monitor disk space changes as a proxy for I/O
        // This is less accurate but works without sudo
        let task = Process()
        task.launchPath = "/bin/df"
        task.arguments = ["-k", "/"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                let lines = output.components(separatedBy: .newlines)
                
                if lines.count > 1 {
                    let components = lines[1].components(separatedBy: .whitespaces).filter { !$0.isEmpty }
                    
                    if components.count >= 3 {
                        if let used = Double(components[2]) {
                            let usedBytes = used * 1024 // Convert from KB to bytes
                            
                            let currentTime = Date()
                            let timeDelta = currentTime.timeIntervalSince(lastUpdateTime)
                            
                            if timeDelta > 0 && previousDiskWrite > 0 {
                                let writeRate = max(0, (usedBytes - previousDiskWrite) / timeDelta)
                                diskWriteRate = writeRate
                                
                                // Estimate read rate as a fraction of write rate
                                diskReadRate = writeRate * 0.3
                            }
                            
                            previousDiskWrite = usedBytes
                            totalDiskWrite = usedBytes
                            totalDiskRead = usedBytes * 0.3
                        }
                    }
                }
            }
        } catch {
            print("Error getting disk stats: \(error)")
        }
        #endif
    }
    
    private func updateProcessList() {
        #if os(macOS)
        // Get real process list using ps command
        let task = Process()
        task.launchPath = "/bin/ps"
        task.arguments = ["aux"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                var processes: [ProcessInformation] = []
                let lines = output.components(separatedBy: .newlines)
                
                // Skip header line
                for (index, line) in lines.enumerated() where index > 0 && !line.isEmpty {
                    let components = line.components(separatedBy: .whitespaces).filter { !$0.isEmpty }
                    
                    if components.count >= 11 {
                        let user = components[0]
                        let pid = Int32(components[1]) ?? 0
                        let cpu = Double(components[2]) ?? 0.0
                        let mem = Double(components[3]) ?? 0.0
                        let vsz = Int64(components[4]) ?? 0 // Virtual memory size in KB
                        let rss = Int64(components[5]) ?? 0 // Resident set size in KB
                        
                        // Command is everything from column 10 onwards
                        let command = components[10...].joined(separator: " ")
                        
                        // Extract process name from command
                        let processName = extractProcessName(from: command)
                        
                        // Convert RSS to bytes for memory usage
                        let memoryUsage = rss * 1024
                        
                        processes.append(ProcessInformation(
                            pid: pid,
                            name: processName,
                            path: command,
                            cpuUsage: cpu,
                            memoryUsage: memoryUsage,
                            parentPid: 1, // ps doesn't provide parent PID easily
                            user: user
                        ))
                    }
                }
                
                // Update our properties
                allProcesses = processes
                topProcesses = processes.sorted { $0.cpuUsage > $1.cpuUsage }.prefix(10).map { $0 }
                activeProcessCount = processes.filter { $0.cpuUsage > 0.1 }.count
                
                // Also get more detailed info for top processes using top command
                updateTopProcessesWithDetailedInfo()
            }
        } catch {
            print("Error getting process list: \(error)")
            // Fallback to minimal process list
            loadSampleProcesses()
        }
        #else
        // Non-macOS fallback
        loadSampleProcesses()
        #endif
    }
    
    private func extractProcessName(from command: String) -> String {
        // Extract the actual process name from the command path
        let components = command.components(separatedBy: "/")
        var processName = components.last ?? command
        
        // Remove arguments if present
        if let spaceIndex = processName.firstIndex(of: " ") {
            processName = String(processName[..<spaceIndex])
        }
        
        // Handle special cases
        if processName.contains(".app/Contents/MacOS/") {
            // Extract app name
            if let appRange = command.range(of: #"([^/]+)\.app"#, options: .regularExpression) {
                processName = String(command[appRange]).replacingOccurrences(of: ".app", with: "")
            }
        }
        
        return processName
    }
    
    private func updateTopProcessesWithDetailedInfo() {
        #if os(macOS)
        // Get more accurate CPU usage for top processes using top command
        let task = Process()
        task.launchPath = "/usr/bin/top"
        task.arguments = ["-l", "1", "-n", "20", "-stats", "pid,command,cpu,rsize,vsize,user"]
        
        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = Pipe()
        
        do {
            try task.run()
            task.waitUntilExit()
            
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            if let output = String(data: data, encoding: .utf8) {
                // Parse top output to update CPU usage for our top processes
                let lines = output.components(separatedBy: .newlines)
                var inProcessSection = false
                
                for line in lines {
                    if line.contains("PID") && line.contains("COMMAND") {
                        inProcessSection = true
                        continue
                    }
                    
                    if inProcessSection && !line.isEmpty {
                        let components = line.components(separatedBy: .whitespaces).filter { !$0.isEmpty }
                        
                        if components.count >= 3 {
                            if let pid = Int32(components[0]),
                               let processIndex = topProcesses.firstIndex(where: { $0.pid == pid }) {
                                
                                // Update CPU usage from top output
                                if components.count > 2, let cpu = Double(components[2].replacingOccurrences(of: "%", with: "")) {
                                    var updatedProcess = topProcesses[processIndex]
                                    updatedProcess = ProcessInformation(
                                        id: updatedProcess.id,
                                        pid: updatedProcess.pid,
                                        name: updatedProcess.name,
                                        path: updatedProcess.path,
                                        cpuUsage: cpu,
                                        memoryUsage: updatedProcess.memoryUsage,
                                        parentPid: updatedProcess.parentPid,
                                        user: updatedProcess.user
                                    )
                                    topProcesses[processIndex] = updatedProcess
                                }
                            }
                        }
                    }
                }
            }
        } catch {
            print("Error getting detailed process info: \(error)")
        }
        #endif
    }
    
    private func updateHistoricalData() {
        // Add current values to history
        cpuHistory.append(cpuUsage)
        memoryHistory.append(memoryUsage)
        networkHistory.append((networkInRate + networkOutRate) / 1024 / 1024) // Convert to MB/s
        diskHistory.append((diskReadRate + diskWriteRate) / 1024 / 1024) // Convert to MB/s
        
        // Limit history size
        let maxPoints = min(maxHistoryPoints, timeRange / 30)
        if cpuHistory.count > maxPoints {
            cpuHistory.removeFirst()
            memoryHistory.removeFirst()
            networkHistory.removeFirst()
            diskHistory.removeFirst()
        }
    }
    
    private func loadSampleProcesses() {
        let sampleProcesses = [
            ProcessInformation(
                pid: 1,
                name: "kernel_task",
                path: "/kernel",
                cpuUsage: 2.5,
                memoryUsage: 100_000_000,
                parentPid: 0,
                user: "root"
            ),
            ProcessInformation(
                pid: 156,
                name: "Safari",
                path: "/Applications/Safari.app",
                cpuUsage: 15.2,
                memoryUsage: 450_000_000,
                parentPid: 1,
                user: ProcessInfo.processInfo.userName
            ),
            ProcessInformation(
                pid: 234,
                name: "Xcode",
                path: "/Applications/Xcode.app",
                cpuUsage: 8.7,
                memoryUsage: 1_200_000_000,
                parentPid: 1,
                user: ProcessInfo.processInfo.userName
            ),
            ProcessInformation(
                pid: 567,
                name: "Terminal",
                path: "/System/Applications/Utilities/Terminal.app",
                cpuUsage: 3.1,
                memoryUsage: 85_000_000,
                parentPid: 1,
                user: ProcessInfo.processInfo.userName
            ),
            ProcessInformation(
                pid: 890,
                name: "Mail",
                path: "/System/Applications/Mail.app",
                cpuUsage: 2.8,
                memoryUsage: 200_000_000,
                parentPid: 1,
                user: ProcessInfo.processInfo.userName
            )
        ]
        
        topProcesses = sampleProcesses.sorted { $0.cpuUsage > $1.cpuUsage }
        allProcesses = sampleProcesses
        activeProcessCount = sampleProcesses.filter { $0.cpuUsage > 0.1 }.count
    }
}

// MARK: - System Information Helpers

extension SystemMonitor {
    static func formatMemorySize(_ bytes: Int64) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useMB, .useGB]
        formatter.countStyle = .memory
        return formatter.string(fromByteCount: bytes)
    }
    
    static func formatNetworkSpeed(_ bytesPerSecond: Double) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .binary
        return "\(formatter.string(fromByteCount: Int64(bytesPerSecond)))/s"
    }
    
    static func getSystemUptime() -> TimeInterval {
        var uptime = timespec()
        if clock_gettime(CLOCK_UPTIME_RAW, &uptime) == 0 {
            return Double(uptime.tv_sec) + Double(uptime.tv_nsec) / 1_000_000_000
        }
        return 0
    }
    
    static func getCPUTemperature() -> Double? {
        // This would require IOKit and specific sensor access
        // Returning simulated temperature for demo
        return Double.random(in: 35...65)
    }
}
