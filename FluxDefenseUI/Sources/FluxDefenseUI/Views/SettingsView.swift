import SwiftUI
import LaunchAtLogin

struct SettingsView: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    @State private var showingAdvancedSettings = false
    @State private var showingWhitelistManager = false
    @State private var alertMessage = ""
    @State private var showingAlert = false
    @State private var launchAtLogin = LaunchAtLogin.isEnabled
    
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 16) {
                // Protection Settings
                ProtectionSettingsCard()
                
                // System Integration
                SystemIntegrationCard(launchAtLogin: $launchAtLogin)
                
                // Monitoring Settings
                MonitoringSettingsCard()
                
                // Whitelist Management
                WhitelistManagementCard(showingWhitelistManager: $showingWhitelistManager)
                
                // Advanced Settings
                AdvancedSettingsCard(showingAdvancedSettings: $showingAdvancedSettings)
                
                // About & Support
                AboutCard()
            }
            .padding(16)
        }
        .alert("Settings", isPresented: $showingAlert) {
            Button("OK") { }
        } message: {
            Text(alertMessage)
        }
        .sheet(isPresented: $showingWhitelistManager) {
            WhitelistManagerView()
        }
        .sheet(isPresented: $showingAdvancedSettings) {
            AdvancedSettingsView()
        }
    }
}

struct ProtectionSettingsCard: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Protection Settings")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                SettingToggle(
                    title: "File System Protection",
                    description: "Monitor and control file access and execution",
                    icon: "doc.badge.gearshape",
                    isOn: $fluxDefenseManager.fileProtectionEnabled
                )
                
                SettingToggle(
                    title: "Network Protection",
                    description: "Monitor and filter network connections",
                    icon: "network.badge.shield.half.filled",
                    isOn: $fluxDefenseManager.networkProtectionEnabled
                )
                
                SettingToggle(
                    title: "Real-time Scanning",
                    description: "Scan files as they are accessed or modified",
                    icon: "magnifyingglass.circle",
                    isOn: $fluxDefenseManager.realTimeScanningEnabled
                )
                
                SettingToggle(
                    title: "Block Unknown Files",
                    description: "Block execution of files not in whitelist",
                    icon: "exclamationmark.shield",
                    isOn: $fluxDefenseManager.blockUnknownFiles
                )
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct SystemIntegrationCard: View {
    @Binding var launchAtLogin: Bool
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("System Integration")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                SettingToggle(
                    title: "Launch at Login",
                    description: "Start FluxDefense automatically when you log in",
                    icon: "power",
                    isOn: $launchAtLogin
                )
                .onChange(of: launchAtLogin) { value in
                    LaunchAtLogin.isEnabled = value
                }
                
                SettingRow(
                    title: "System Extension Status",
                    description: "Check and manage system extension permissions",
                    icon: "gear.badge"
                ) {
                    Button("Check Status") {
                        // Check system extension status
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.accentColor)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
                
                SettingRow(
                    title: "Full Disk Access",
                    description: "Required for complete file system monitoring",
                    icon: "internaldrive"
                ) {
                    Button("Open Settings") {
                        NSWorkspace.shared.open(URL(string: "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")!)
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.orange)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct MonitoringSettingsCard: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Monitoring Settings")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                SettingRow(
                    title: "Monitoring Mode",
                    description: "Choose between passive monitoring or active protection",
                    icon: "eye"
                ) {
                    Picker("Mode", selection: $fluxDefenseManager.monitoringMode) {
                        Text("Passive").tag(MonitoringMode.passive)
                        Text("Active").tag(MonitoringMode.active)
                    }
                    .pickerStyle(SegmentedPickerStyle())
                    .frame(width: 120)
                }
                
                SettingRow(
                    title: "Log Retention",
                    description: "How long to keep security event logs",
                    icon: "clock.arrow.circlepath"
                ) {
                    Picker("Retention", selection: $fluxDefenseManager.logRetentionDays) {
                        Text("7 days").tag(7)
                        Text("30 days").tag(30)
                        Text("90 days").tag(90)
                        Text("1 year").tag(365)
                    }
                    .pickerStyle(MenuPickerStyle())
                    .frame(width: 80)
                }
                
                SettingRow(
                    title: "Event Severity Threshold",
                    description: "Minimum severity level for logged events",
                    icon: "slider.horizontal.3"
                ) {
                    Picker("Severity", selection: $fluxDefenseManager.minimumSeverity) {
                        Text("Low").tag(SecuritySeverity.low)
                        Text("Medium").tag(SecuritySeverity.medium)
                        Text("High").tag(SecuritySeverity.high)
                    }
                    .pickerStyle(MenuPickerStyle())
                    .frame(width: 80)
                }
                
                SettingToggle(
                    title: "Detailed Logging",
                    description: "Include additional metadata in event logs",
                    icon: "doc.text.magnifyingglass",
                    isOn: $fluxDefenseManager.detailedLoggingEnabled
                )
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct WhitelistManagementCard: View {
    @Binding var showingWhitelistManager: Bool
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Whitelist Management")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                SettingRow(
                    title: "Whitelisted Files",
                    description: "\(fluxDefenseManager.whitelistCount) files in whitelist",
                    icon: "checklist"
                ) {
                    Button("Manage") {
                        showingWhitelistManager = true
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.accentColor)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
                
                SettingRow(
                    title: "Auto-update Whitelist",
                    description: "Automatically add trusted system files",
                    icon: "arrow.triangle.2.circlepath"
                ) {
                    Button("Update Now") {
                        fluxDefenseManager.updateSystemWhitelist()
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.green)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
                
                SettingRow(
                    title: "Import/Export",
                    description: "Backup or restore whitelist data",
                    icon: "square.and.arrow.up.on.square"
                ) {
                    HStack(spacing: 4) {
                        Button("Import") {
                            fluxDefenseManager.importWhitelist()
                        }
                        .font(.caption)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 3)
                        .background(Color.blue)
                        .foregroundColor(.white)
                        .cornerRadius(3)
                        
                        Button("Export") {
                            fluxDefenseManager.exportWhitelist()
                        }
                        .font(.caption)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 3)
                        .background(Color.purple)
                        .foregroundColor(.white)
                        .cornerRadius(3)
                    }
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct AdvancedSettingsCard: View {
    @Binding var showingAdvancedSettings: Bool
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Advanced Settings")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                SettingRow(
                    title: "Performance Tuning",
                    description: "Adjust CPU and memory usage limits",
                    icon: "speedometer"
                ) {
                    Button("Configure") {
                        showingAdvancedSettings = true
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.orange)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
                
                SettingRow(
                    title: "Debug Mode",
                    description: "Enable detailed debug logging",
                    icon: "ant"
                ) {
                    Toggle("", isOn: $fluxDefenseManager.debugModeEnabled)
                        .toggleStyle(SwitchToggleStyle())
                }
                
                SettingRow(
                    title: "Reset All Settings",
                    description: "Restore default configuration",
                    icon: "arrow.counterclockwise"
                ) {
                    Button("Reset") {
                        fluxDefenseManager.resetToDefaults()
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.red)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct AboutCard: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("About FluxDefense")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                SettingRow(
                    title: "Version",
                    description: "FluxDefense 1.0.0 (Build 100)",
                    icon: "info.circle"
                ) {
                    Button("Check Updates") {
                        // Check for updates
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.green)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
                
                SettingRow(
                    title: "Support",
                    description: "Get help and report issues",
                    icon: "questionmark.circle"
                ) {
                    Button("Contact") {
                        NSWorkspace.shared.open(URL(string: "mailto:support@fluxdefense.com")!)
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.blue)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
                
                SettingRow(
                    title: "Open Source",
                    description: "View source code and contribute",
                    icon: "chevron.left.forwardslash.chevron.right"
                ) {
                    Button("GitHub") {
                        NSWorkspace.shared.open(URL(string: "https://github.com/fluxdefense/fluxdefense")!)
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.black)
                    .foregroundColor(.white)
                    .cornerRadius(4)
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct SettingToggle: View {
    let title: String
    let description: String
    let icon: String
    @Binding var isOn: Bool
    
    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .foregroundColor(.accentColor)
                .font(.title3)
                .frame(width: 24)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            
            Spacer()
            
            Toggle("", isOn: $isOn)
                .toggleStyle(SwitchToggleStyle())
        }
        .padding(.vertical, 4)
    }
}

struct SettingRow<Content: View>: View {
    let title: String
    let description: String
    let icon: String
    let content: Content
    
    init(title: String, description: String, icon: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.description = description
        self.icon = icon
        self.content = content()
    }
    
    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .foregroundColor(.accentColor)
                .font(.title3)
                .frame(width: 24)
            
            VStack(alignment: .leading, spacing: 2) {
                Text(title)
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            
            Spacer()
            
            content
        }
        .padding(.vertical, 4)
    }
}

struct WhitelistManagerView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    @State private var searchText = ""
    
    var filteredWhitelistEntries: [WhitelistEntry] {
        if searchText.isEmpty {
            return fluxDefenseManager.whitelistEntries
        } else {
            return fluxDefenseManager.whitelistEntries.filter { entry in
                entry.path.localizedCaseInsensitiveContains(searchText) ||
                entry.name.localizedCaseInsensitiveContains(searchText)
            }
        }
    }
    
    var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                // Search bar
                HStack {
                    Image(systemName: "magnifyingglass")
                        .foregroundColor(.secondary)
                    
                    TextField("Search whitelist...", text: $searchText)
                        .textFieldStyle(PlainTextFieldStyle())
                }
                .padding(12)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(6)
                .padding(16)
                
                Divider()
                
                // Whitelist entries
                if filteredWhitelistEntries.isEmpty {
                    VStack(spacing: 16) {
                        Image(systemName: "checklist")
                            .foregroundColor(.secondary)
                            .font(.largeTitle)
                        
                        Text(searchText.isEmpty ? "No whitelist entries" : "No matching entries")
                            .font(.headline)
                            .foregroundColor(.secondary)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    List {
                        ForEach(filteredWhitelistEntries, id: \.id) { entry in
                            WhitelistEntryRow(entry: entry)
                        }
                        .onDelete { indexSet in
                            fluxDefenseManager.removeWhitelistEntries(at: indexSet)
                        }
                    }
                }
            }
            .navigationTitle("Whitelist Manager")
            
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("Done") {
                        dismiss()
                    }
                }
                
                ToolbarItem(placement: .automatic) {
                    Button("Add") {
                        fluxDefenseManager.addFileToWhitelist()
                    }
                }
            }
        }
        .frame(width: 600, height: 500)
    }
}

struct WhitelistEntryRow: View {
    let entry: WhitelistEntry
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Image(systemName: "doc.badge.checkmark")
                    .foregroundColor(.green)
                
                Text(entry.name)
                    .font(.subheadline)
                    .fontWeight(.medium)
                
                Spacer()
                
                Text(entry.dateAdded.formatted(.relative(presentation: .numeric)))
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text(entry.path)
                .font(.caption)
                .foregroundColor(.secondary)
                .lineLimit(2)
            
            if !entry.hash.isEmpty {
                Text("SHA256: \(entry.hash.prefix(16))...")
                    .font(.caption2)
                    .foregroundColor(.secondary)
                    .fontDesign(.monospaced)
            }
        }
        .padding(.vertical, 4)
    }
}

struct AdvancedSettingsView: View {
    @Environment(\.dismiss) private var dismiss
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        NavigationView {
            Form {
                Section("Performance") {
                    VStack(alignment: .leading) {
                        Text("CPU Usage Limit: \(Int(fluxDefenseManager.cpuUsageLimit))%")
                        Slider(value: $fluxDefenseManager.cpuUsageLimit, in: 10...90, step: 5)
                    }
                    
                    VStack(alignment: .leading) {
                        Text("Memory Usage Limit: \(Int(fluxDefenseManager.memoryUsageLimit))MB")
                        Slider(value: $fluxDefenseManager.memoryUsageLimit, in: 100...2048, step: 100)
                    }
                }
                
                Section("Monitoring") {
                    VStack(alignment: .leading) {
                        Text("Scan Interval: \(Int(fluxDefenseManager.scanInterval)) seconds")
                        Slider(value: $fluxDefenseManager.scanInterval, in: 1...60, step: 1)
                    }
                    
                    Toggle("Enable Hash Verification", isOn: $fluxDefenseManager.hashVerificationEnabled)
                    Toggle("Monitor System Processes", isOn: $fluxDefenseManager.monitorSystemProcesses)
                }
                
                Section("Network") {
                    Toggle("Block Suspicious Connections", isOn: $fluxDefenseManager.blockSuspiciousConnections)
                    Toggle("Log All Network Activity", isOn: $fluxDefenseManager.logAllNetworkActivity)
                    
                    VStack(alignment: .leading) {
                        Text("Connection Timeout: \(Int(fluxDefenseManager.connectionTimeout)) seconds")
                        Slider(value: $fluxDefenseManager.connectionTimeout, in: 5...300, step: 5)
                    }
                }
            }
            .navigationTitle("Advanced Settings")
            
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
        .frame(width: 500, height: 400)
    }
}

