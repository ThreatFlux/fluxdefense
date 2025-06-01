import Foundation
import SwiftUI
import AppKit
import UserNotifications

@MainActor
class FluxDefenseManager: ObservableObject {
    static let shared = FluxDefenseManager()
    
    // MARK: - Published Properties
    @Published var status: FluxDefenseStatus = .passive
    @Published var fileProtectionEnabled: Bool = true
    @Published var networkProtectionEnabled: Bool = true
    @Published var realTimeScanningEnabled: Bool = true
    @Published var blockUnknownFiles: Bool = false
    @Published var monitoringMode: MonitoringMode = .passive
    @Published var detailedLoggingEnabled: Bool = true
    @Published var debugModeEnabled: Bool = false
    
    // Event data
    @Published var allEvents: [SecurityEvent] = []
    @Published var recentEvents: [SecurityEvent] = []
    @Published var todayEventCount: Int = 0
    @Published var threatsBlocked: Int = 0
    
    // Whitelist management
    @Published var whitelistEntries: [WhitelistEntry] = []
    @Published var whitelistCount: Int = 0
    
    // Settings
    @Published var logRetentionDays: Int = 30
    @Published var minimumSeverity: SecuritySeverity = .low
    @Published var cpuUsageLimit: Double = 50
    @Published var memoryUsageLimit: Double = 512
    @Published var scanInterval: Double = 5
    @Published var hashVerificationEnabled: Bool = true
    @Published var monitorSystemProcesses: Bool = false
    @Published var blockSuspiciousConnections: Bool = true
    @Published var logAllNetworkActivity: Bool = false
    @Published var connectionTimeout: Double = 30
    
    private let eventLogPath = NSHomeDirectory() + "/Library/Application Support/FluxDefense/events.json"
    private let whitelistPath = NSHomeDirectory() + "/Library/Application Support/FluxDefense/whitelist.json"
    private let settingsPath = NSHomeDirectory() + "/Library/Application Support/FluxDefense/settings.json"
    
    private var updateTimer: Timer?
    private var isMonitoring = false
    private var fileSystemEventStream: FSEventStreamRef?
    private var networkMonitorTask: Process?
    private var lastEventStreamPaths: [String] = []
    
    init() {
        createDirectoriesIfNeeded()
        loadSettings()
        loadWhitelist()
        loadEvents()
        
        // Request notification permissions
        // Disabled for development builds to avoid crashes
        // requestNotificationPermissions()
        
        // Load sample data for demo
        if allEvents.isEmpty {
            loadSampleData()
        }
    }
    
    // MARK: - Public Methods
    
    func startMonitoring() {
        guard !isMonitoring else { return }
        isMonitoring = true
        status = monitoringMode == .active ? .active : .passive
        
        // Start file system monitoring
        startFileSystemMonitoring()
        
        // Start network monitoring
        startNetworkMonitoring()
        
        // Start periodic updates
        updateTimer = Timer.scheduledTimer(withTimeInterval: 5.0, repeats: true) { _ in
            Task { @MainActor in
                self.updateStatistics()
                self.checkForNewEvents()
                self.checkRunningProcesses()
            }
        }
        
        print("FluxDefense monitoring started in \(monitoringMode.rawValue) mode")
    }
    
    func stopMonitoring() {
        isMonitoring = false
        status = .disabled
        updateTimer?.invalidate()
        updateTimer = nil
        
        // Stop file system monitoring
        stopFileSystemMonitoring()
        
        // Stop network monitoring
        stopNetworkMonitoring()
        
        print("FluxDefense monitoring stopped")
    }
    
    func addEvent(_ event: SecurityEvent) {
        allEvents.insert(event, at: 0)
        updateRecentEvents()
        updateStatistics()
        saveEvents()
        
        // Send notification for high severity events
        // Disabled for development builds
        // if event.severity == .high || event.severity == .critical {
        //     sendNotification(for: event)
        // }
    }
    
    func clearEvents() {
        allEvents.removeAll()
        recentEvents.removeAll()
        todayEventCount = 0
        threatsBlocked = 0
        saveEvents()
    }
    
    func addToWhitelist(_ entry: WhitelistEntry) {
        whitelistEntries.append(entry)
        whitelistCount = whitelistEntries.count
        saveWhitelist()
    }
    
    func removeWhitelistEntries(at indexSet: IndexSet) {
        whitelistEntries.remove(atOffsets: indexSet)
        whitelistCount = whitelistEntries.count
        saveWhitelist()
    }
    
    func addFileToWhitelist() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        
        if panel.runModal() == .OK, let url = panel.url {
            let entry = WhitelistEntry(
                name: url.lastPathComponent,
                path: url.path,
                hash: calculateFileHash(at: url.path) ?? "",
                addedBy: "user",
                verified: true
            )
            addToWhitelist(entry)
        }
    }
    
    func updateSystemWhitelist() {
        // Simulate updating system whitelist
        let systemPaths = [
            "/System/Applications/",
            "/Applications/",
            "/usr/bin/",
            "/bin/"
        ]
        
        var newEntries: [WhitelistEntry] = []
        
        for path in systemPaths {
            // This would normally scan the directory and add legitimate files
            let entry = WhitelistEntry(
                name: "System Update \(Date().formatted(.dateTime))",
                path: path,
                hash: UUID().uuidString,
                addedBy: "system",
                verified: true
            )
            newEntries.append(entry)
        }
        
        whitelistEntries.append(contentsOf: newEntries)
        whitelistCount = whitelistEntries.count
        saveWhitelist()
    }
    
    func importWhitelist() {
        let panel = NSOpenPanel()
        panel.allowsMultipleSelection = false
        panel.canChooseDirectories = false
        panel.canChooseFiles = true
        panel.allowedContentTypes = [.json]
        
        if panel.runModal() == .OK, let url = panel.url {
            do {
                let data = try Data(contentsOf: url)
                let importedEntries = try JSONDecoder().decode([WhitelistEntry].self, from: data)
                whitelistEntries.append(contentsOf: importedEntries)
                whitelistCount = whitelistEntries.count
                saveWhitelist()
            } catch {
                print("Failed to import whitelist: \(error)")
            }
        }
    }
    
    func exportWhitelist() {
        let panel = NSSavePanel()
        panel.allowedContentTypes = [.json]
        panel.nameFieldStringValue = "fluxdefense_whitelist.json"
        
        if panel.runModal() == .OK, let url = panel.url {
            do {
                let data = try JSONEncoder().encode(whitelistEntries)
                try data.write(to: url)
            } catch {
                print("Failed to export whitelist: \(error)")
            }
        }
    }
    
    func startSystemScan() {
        status = .active
        
        // Simulate system scan
        DispatchQueue.global(qos: .background).async {
            Thread.sleep(forTimeInterval: 2.0)
            
            DispatchQueue.main.async {
                let scanEvent = SecurityEvent(
                    type: .systemCall,
                    severity: .low,
                    description: "System scan completed",
                    processName: "FluxDefense",
                    processPath: "/Applications/FluxDefense.app",
                    processId: Int32(Foundation.ProcessInfo.processInfo.processIdentifier),
                    verdict: .log,
                    reason: "Scheduled system scan"
                )
                
                self.addEvent(scanEvent)
                self.status = self.monitoringMode == .active ? .active : .passive
            }
        }
    }
    
    func updateSecurityRules() {
        // Simulate updating security rules
        let updateEvent = SecurityEvent(
            type: .systemCall,
            severity: .low,
            description: "Security rules updated",
            processName: "FluxDefense",
            processPath: "/Applications/FluxDefense.app",
            processId: Int32(Foundation.ProcessInfo.processInfo.processIdentifier),
            verdict: .log,
            reason: "Manual security rules update"
        )
        
        addEvent(updateEvent)
    }
    
    func resetToDefaults() {
        fileProtectionEnabled = true
        networkProtectionEnabled = true
        realTimeScanningEnabled = true
        blockUnknownFiles = false
        monitoringMode = .passive
        detailedLoggingEnabled = true
        debugModeEnabled = false
        logRetentionDays = 30
        minimumSeverity = .low
        cpuUsageLimit = 50
        memoryUsageLimit = 512
        scanInterval = 5
        hashVerificationEnabled = true
        monitorSystemProcesses = false
        blockSuspiciousConnections = true
        logAllNetworkActivity = false
        connectionTimeout = 30
        
        saveSettings()
    }
    
    // MARK: - Private Methods
    
    private func createDirectoriesIfNeeded() {
        let appSupportDir = NSHomeDirectory() + "/Library/Application Support/FluxDefense"
        try? FileManager.default.createDirectory(atPath: appSupportDir, withIntermediateDirectories: true)
    }
    
    private func loadSampleData() {
        allEvents = SecurityEvent.sampleEvents
        updateRecentEvents()
        updateStatistics()
        
        // Add some sample whitelist entries
        whitelistEntries = [
            WhitelistEntry(
                name: "System Preferences",
                path: "/System/Applications/System Preferences.app",
                hash: "a1b2c3d4e5f6789abcdef",
                addedBy: "system",
                verified: true
            ),
            WhitelistEntry(
                name: "Safari",
                path: "/Applications/Safari.app",
                hash: "f6e5d4c3b2a1098765432",
                addedBy: "user",
                verified: true
            ),
            WhitelistEntry(
                name: "Terminal",
                path: "/System/Applications/Utilities/Terminal.app",
                hash: "123456789abcdef012345",
                addedBy: "system",
                verified: true
            )
        ]
        whitelistCount = whitelistEntries.count
        saveWhitelist()
    }
    
    private func updateRecentEvents() {
        recentEvents = Array(allEvents.prefix(10))
    }
    
    private func updateStatistics() {
        let calendar = Calendar.current
        let today = calendar.startOfDay(for: Date())
        
        todayEventCount = allEvents.filter { event in
            calendar.isDate(event.timestamp, inSameDayAs: today)
        }.count
        
        threatsBlocked = allEvents.filter { event in
            event.verdict == .deny || event.verdict == .quarantine
        }.count
    }
    
    private func checkForNewEvents() {
        // Simulate random security events for demo
        if Int.random(in: 1...20) == 1 {
            let randomEvent = generateRandomEvent()
            addEvent(randomEvent)
        }
    }
    
    private func generateRandomEvent() -> SecurityEvent {
        let eventTypes: [EventType] = [.fileExecution, .fileAccess, .networkConnection, .processStart]
        let severities: [SecuritySeverity] = [.low, .low, .low, .medium, .high] // Weight towards lower severity
        let verdicts: [Verdict] = [.allow, .allow, .allow, .log, .deny] // Weight towards allow
        
        let type = eventTypes.randomElement()!
        let severity = severities.randomElement()!
        let verdict = verdicts.randomElement()!
        
        let processNames = ["Safari", "Terminal", "TextEdit", "Mail", "Notes", "Calculator"]
        let processName = processNames.randomElement()!
        
        return SecurityEvent(
            type: type,
            severity: severity,
            description: "\(type.displayName) detected",
            processName: processName,
            processPath: "/Applications/\(processName).app",
            processId: Int32.random(in: 1000...9999),
            verdict: verdict,
            reason: verdict == .allow ? "Trusted application" : "Security policy violation",
            riskScore: Double.random(in: 0.1...0.9)
        )
    }
    
    private func sendNotification(for event: SecurityEvent) {
        // Check if we're running in a proper app bundle
        guard Bundle.main.bundleIdentifier != nil else {
            print("Warning: Cannot send notification - no bundle identifier")
            return
        }
        
        // Additional check for development environment
        let bundleURL = Bundle.main.bundleURL
        if bundleURL.path.contains("DerivedData") || bundleURL.path.contains("Build/Products") {
            print("Warning: Cannot send notification from development build")
            return
        }
        
        let content = UNMutableNotificationContent()
        content.title = "FluxDefense Security Alert"
        content.body = "\(event.severity.displayName) threat detected: \(event.description)"
        content.sound = .default
        
        let request = UNNotificationRequest(
            identifier: UUID().uuidString,
            content: content,
            trigger: nil
        )
        
        UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
                print("Error sending notification: \(error)")
            }
        }
    }
    
    private func calculateFileHash(at path: String) -> String? {
        // This would normally calculate SHA256 hash of the file
        return UUID().uuidString.replacingOccurrences(of: "-", with: "").lowercased()
    }
    
    private func requestNotificationPermissions() {
        // Check if we're running in a proper app bundle
        guard Bundle.main.bundleIdentifier != nil else {
            print("Warning: No bundle identifier found. Notifications disabled.")
            return
        }
        
        // Additional check for development environment
        let bundleURL = Bundle.main.bundleURL
        if bundleURL.path.contains("DerivedData") || bundleURL.path.contains("Build/Products") {
            print("Warning: Running from development build. Notifications may not work properly.")
            // Don't attempt to access UNUserNotificationCenter in development builds
            return
        }
        
        UNUserNotificationCenter.current().requestAuthorization(options: [.alert, .sound]) { granted, error in
            if let error = error {
                print("Error requesting notification permissions: \(error)")
            } else if granted {
                print("Notification permissions granted")
            }
        }
    }
    
    // MARK: - Persistence
    
    private func saveEvents() {
        do {
            let data = try JSONEncoder().encode(allEvents)
            try data.write(to: URL(fileURLWithPath: eventLogPath))
        } catch {
            print("Failed to save events: \(error)")
        }
    }
    
    private func loadEvents() {
        guard FileManager.default.fileExists(atPath: eventLogPath) else { return }
        
        do {
            let data = try Data(contentsOf: URL(fileURLWithPath: eventLogPath))
            allEvents = try JSONDecoder().decode([SecurityEvent].self, from: data)
            updateRecentEvents()
            updateStatistics()
        } catch {
            print("Failed to load events: \(error)")
        }
    }
    
    private func saveWhitelist() {
        do {
            let data = try JSONEncoder().encode(whitelistEntries)
            try data.write(to: URL(fileURLWithPath: whitelistPath))
        } catch {
            print("Failed to save whitelist: \(error)")
        }
    }
    
    private func loadWhitelist() {
        guard FileManager.default.fileExists(atPath: whitelistPath) else { return }
        
        do {
            let data = try Data(contentsOf: URL(fileURLWithPath: whitelistPath))
            whitelistEntries = try JSONDecoder().decode([WhitelistEntry].self, from: data)
            whitelistCount = whitelistEntries.count
        } catch {
            print("Failed to load whitelist: \(error)")
        }
    }
    
    private func saveSettings() {
        let settings: [String: Any] = [
            "fileProtectionEnabled": fileProtectionEnabled,
            "networkProtectionEnabled": networkProtectionEnabled,
            "realTimeScanningEnabled": realTimeScanningEnabled,
            "blockUnknownFiles": blockUnknownFiles,
            "monitoringMode": monitoringMode.rawValue,
            "detailedLoggingEnabled": detailedLoggingEnabled,
            "debugModeEnabled": debugModeEnabled,
            "logRetentionDays": logRetentionDays,
            "minimumSeverity": minimumSeverity.rawValue,
            "cpuUsageLimit": cpuUsageLimit,
            "memoryUsageLimit": memoryUsageLimit,
            "scanInterval": scanInterval,
            "hashVerificationEnabled": hashVerificationEnabled,
            "monitorSystemProcesses": monitorSystemProcesses,
            "blockSuspiciousConnections": blockSuspiciousConnections,
            "logAllNetworkActivity": logAllNetworkActivity,
            "connectionTimeout": connectionTimeout
        ]
        
        do {
            let data = try JSONSerialization.data(withJSONObject: settings)
            try data.write(to: URL(fileURLWithPath: settingsPath))
        } catch {
            print("Failed to save settings: \(error)")
        }
    }
    
    private func loadSettings() {
        guard FileManager.default.fileExists(atPath: settingsPath) else { return }
        
        do {
            let data = try Data(contentsOf: URL(fileURLWithPath: settingsPath))
            let settings = try JSONSerialization.jsonObject(with: data) as? [String: Any] ?? [:]
            
            fileProtectionEnabled = settings["fileProtectionEnabled"] as? Bool ?? true
            networkProtectionEnabled = settings["networkProtectionEnabled"] as? Bool ?? true
            realTimeScanningEnabled = settings["realTimeScanningEnabled"] as? Bool ?? true
            blockUnknownFiles = settings["blockUnknownFiles"] as? Bool ?? false
            detailedLoggingEnabled = settings["detailedLoggingEnabled"] as? Bool ?? true
            debugModeEnabled = settings["debugModeEnabled"] as? Bool ?? false
            logRetentionDays = settings["logRetentionDays"] as? Int ?? 30
            cpuUsageLimit = settings["cpuUsageLimit"] as? Double ?? 50
            memoryUsageLimit = settings["memoryUsageLimit"] as? Double ?? 512
            scanInterval = settings["scanInterval"] as? Double ?? 5
            hashVerificationEnabled = settings["hashVerificationEnabled"] as? Bool ?? true
            monitorSystemProcesses = settings["monitorSystemProcesses"] as? Bool ?? false
            blockSuspiciousConnections = settings["blockSuspiciousConnections"] as? Bool ?? true
            logAllNetworkActivity = settings["logAllNetworkActivity"] as? Bool ?? false
            connectionTimeout = settings["connectionTimeout"] as? Double ?? 30
            
            if let modeString = settings["monitoringMode"] as? String {
                monitoringMode = MonitoringMode(rawValue: modeString) ?? .passive
            }
            
            if let severityString = settings["minimumSeverity"] as? String {
                minimumSeverity = SecuritySeverity(rawValue: severityString) ?? .low
            }
        } catch {
            print("Failed to load settings: \(error)")
        }
    }
    
    // MARK: - Real File System Monitoring
    
    private func startFileSystemMonitoring() {
        #if os(macOS)
        // Monitor important directories
        let pathsToWatch = [
            "/Applications",
            "/System/Library/LaunchDaemons",
            "/Library/LaunchDaemons",
            "/Library/LaunchAgents",
            NSHomeDirectory() + "/Library/LaunchAgents",
            NSHomeDirectory() + "/Downloads"
        ]
        
        lastEventStreamPaths = pathsToWatch
        
        let callback: FSEventStreamCallback = { streamRef, clientCallBackInfo, numEvents, eventPaths, eventFlags, eventIds in
            let fileSystemWatcher = Unmanaged<FluxDefenseManager>.fromOpaque(clientCallBackInfo!).takeUnretainedValue()
            let paths = Unmanaged<CFArray>.fromOpaque(eventPaths).takeUnretainedValue() as NSArray
            
            for i in 0..<numEvents {
                if let path = paths[i] as? String {
                    Task { @MainActor in
                        fileSystemWatcher.handleFileSystemEvent(at: path, flags: eventFlags[i])
                    }
                }
            }
        }
        
        var context = FSEventStreamContext(
            version: 0,
            info: Unmanaged.passUnretained(self).toOpaque(),
            retain: nil,
            release: nil,
            copyDescription: nil
        )
        
        fileSystemEventStream = FSEventStreamCreate(
            nil,
            callback,
            &context,
            pathsToWatch as CFArray,
            FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
            1.0,
            FSEventStreamCreateFlags(kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagUseCFTypes)
        )
        
        if let stream = fileSystemEventStream {
            FSEventStreamSetDispatchQueue(stream, DispatchQueue.global(qos: .background))
            FSEventStreamStart(stream)
        }
        #endif
    }
    
    private func stopFileSystemMonitoring() {
        #if os(macOS)
        if let stream = fileSystemEventStream {
            FSEventStreamStop(stream)
            FSEventStreamInvalidate(stream)
            FSEventStreamRelease(stream)
            fileSystemEventStream = nil
        }
        #endif
    }
    
    @MainActor
    private func handleFileSystemEvent(at path: String, flags: FSEventStreamEventFlags) {
        // Check if this is a file creation or modification
        if flags & FSEventStreamEventFlags(kFSEventStreamEventFlagItemCreated) != 0 ||
           flags & FSEventStreamEventFlags(kFSEventStreamEventFlagItemModified) != 0 {
            
            // Check if it's an executable or app
            let isExecutable = path.hasSuffix(".app") || 
                              path.contains("/LaunchDaemons/") || 
                              path.contains("/LaunchAgents/")
            
            if isExecutable || fileProtectionEnabled {
                // Create security event
                let event = SecurityEvent(
                    type: .fileExecution,
                    severity: isExecutable ? .medium : .low,
                    description: "File system change detected",
                    processName: URL(fileURLWithPath: path).lastPathComponent,
                    processPath: path,
                    processId: 0,
                    verdict: checkFileAgainstWhitelist(path) ? .allow : (blockUnknownFiles ? .deny : .log),
                    reason: "File system event",
                    fileInfo: FileInfo(
                        path: path,
                        size: getFileSize(path),
                        hash: calculateFileHash(path)
                    )
                )
                
                addEvent(event)
            }
        }
    }
    
    // MARK: - Network Monitoring
    
    private func startNetworkMonitoring() {
        #if os(macOS)
        // Use lsof to monitor network connections
        networkMonitorTask = Process()
        networkMonitorTask?.launchPath = "/usr/sbin/lsof"
        networkMonitorTask?.arguments = ["-i", "-n", "-P"]
        
        // Note: For real implementation, you'd use Network Extension framework
        // This is a simplified version for demonstration
        #endif
    }
    
    private func stopNetworkMonitoring() {
        networkMonitorTask?.terminate()
        networkMonitorTask = nil
    }
    
    // MARK: - Process Monitoring
    
    private func checkRunningProcesses() {
        #if os(macOS)
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
                let lines = output.components(separatedBy: .newlines)
                
                for line in lines {
                    // Check for suspicious processes
                    if line.contains("nc ") || // netcat
                       line.contains("nmap") ||
                       line.contains("tcpdump") ||
                       (line.contains("python") && line.contains("socket")) {
                        
                        let components = line.components(separatedBy: .whitespaces).filter { !$0.isEmpty }
                        if components.count > 10 {
                            let processName = extractProcessName(from: components[10...].joined(separator: " "))
                            
                            let event = SecurityEvent(
                                type: .processStart,
                                severity: .medium,
                                description: "Potentially suspicious process detected",
                                processName: processName,
                                processPath: components[10...].joined(separator: " "),
                                processId: Int32(components[1]) ?? 0,
                                verdict: blockUnknownFiles ? .deny : .log,
                                reason: "Process matches suspicious patterns"
                            )
                            
                            // Only add if we haven't seen this recently
                            if !recentEvents.contains(where: { 
                                $0.processName == processName && 
                                $0.timestamp.timeIntervalSinceNow > -60 
                            }) {
                                addEvent(event)
                            }
                        }
                    }
                }
            }
        } catch {
            print("Error checking processes: \(error)")
        }
        #endif
    }
    
    private func extractProcessName(from command: String) -> String {
        let components = command.components(separatedBy: "/")
        var name = components.last ?? command
        
        if let spaceIndex = name.firstIndex(of: " ") {
            name = String(name[..<spaceIndex])
        }
        
        return name
    }
    
    // MARK: - File Checking Utilities
    
    private func checkFileAgainstWhitelist(_ path: String) -> Bool {
        let fileHash = calculateFileHash(path)
        return whitelistEntries.contains { entry in
            entry.path == path || entry.hash == fileHash
        }
    }
    
    private func calculateFileHash(_ path: String) -> String {
        // Simplified hash calculation - in production use proper cryptographic hashing
        let url = URL(fileURLWithPath: path)
        if let data = try? Data(contentsOf: url) {
            return String(data.hashValue)
        }
        return ""
    }
    
    private func getFileSize(_ path: String) -> Int64 {
        let url = URL(fileURLWithPath: path)
        if let attributes = try? FileManager.default.attributesOfItem(atPath: url.path),
           let size = attributes[.size] as? Int64 {
            return size
        }
        return 0
    }
}