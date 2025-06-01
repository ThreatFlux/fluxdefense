import SwiftUI

struct DashboardView: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 16) {
                // Security Status Card
                SecurityStatusCard()
                
                // Quick Stats
                HStack(spacing: 12) {
                    StatCard(
                        title: "Events Today",
                        value: "\(fluxDefenseManager.todayEventCount)",
                        icon: "calendar",
                        color: .blue
                    )
                    
                    StatCard(
                        title: "Threats Blocked",
                        value: "\(fluxDefenseManager.threatsBlocked)",
                        icon: "shield.slash",
                        color: .red
                    )
                }
                
                // System Performance
                SystemPerformanceCard()
                
                // Recent Events
                RecentEventsCard()
                
                // Quick Actions
                QuickActionsCard()
            }
            .padding(16)
        }
    }
}

struct SecurityStatusCard: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Image(systemName: "checkmark.shield")
                    .foregroundColor(.green)
                    .font(.title2)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text("System Protected")
                        .font(.headline)
                        .fontWeight(.semibold)
                    
                    Text("All security modules active")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                Text("ACTIVE")
                    .font(.caption)
                    .fontWeight(.bold)
                    .foregroundColor(.white)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.green)
                    .cornerRadius(4)
            }
            
            Divider()
            
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text("File Protection")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    HStack(spacing: 4) {
                        Circle()
                            .fill(fluxDefenseManager.fileProtectionEnabled ? .green : .red)
                            .frame(width: 6, height: 6)
                        
                        Text(fluxDefenseManager.fileProtectionEnabled ? "Active" : "Inactive")
                            .font(.caption)
                            .fontWeight(.medium)
                    }
                }
                
                Spacer()
                
                VStack(alignment: .leading, spacing: 4) {
                    Text("Network Protection")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    HStack(spacing: 4) {
                        Circle()
                            .fill(fluxDefenseManager.networkProtectionEnabled ? .green : .red)
                            .frame(width: 6, height: 6)
                        
                        Text(fluxDefenseManager.networkProtectionEnabled ? "Active" : "Inactive")
                            .font(.caption)
                            .fontWeight(.medium)
                    }
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.green.opacity(0.3), lineWidth: 1)
        )
        .cornerRadius(8)
    }
}

struct StatCard: View {
    let title: String
    let value: String
    let icon: String
    let color: Color
    
    var body: some View {
        VStack(spacing: 8) {
            Image(systemName: icon)
                .foregroundColor(color)
                .font(.title2)
            
            Text(value)
                .font(.title2)
                .fontWeight(.bold)
            
            Text(title)
                .font(.caption)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
        }
        .frame(maxWidth: .infinity)
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct SystemPerformanceCard: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("System Performance")
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(spacing: 12) {
                PerformanceBar(
                    title: "CPU",
                    percentage: systemMonitor.cpuUsage,
                    color: .blue
                )
                
                PerformanceBar(
                    title: "Memory",
                    percentage: systemMonitor.memoryUsage,
                    color: .orange
                )
                
                PerformanceBar(
                    title: "Disk",
                    percentage: systemMonitor.diskUsage,
                    color: .purple
                )
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct PerformanceBar: View {
    let title: String
    let percentage: Double
    let color: Color
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                
                Spacer()
                
                Text("\(Int(percentage))%")
                    .font(.caption)
                    .fontWeight(.medium)
                    .foregroundColor(color)
            }
            
            GeometryReader { geometry in
                ZStack(alignment: .leading) {
                    Rectangle()
                        .fill(Color.secondary.opacity(0.2))
                        .frame(height: 4)
                    
                    Rectangle()
                        .fill(color)
                        .frame(width: geometry.size.width * (percentage / 100), height: 4)
                }
            }
            .frame(height: 4)
            .cornerRadius(2)
        }
    }
}

struct RecentEventsCard: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Recent Events")
                    .font(.headline)
                    .fontWeight(.semibold)
                
                Spacer()
                
                Button("View All") {
                    // Switch to logs tab
                }
                .font(.caption)
                .foregroundColor(.accentColor)
            }
            
            if fluxDefenseManager.recentEvents.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "checkmark.circle")
                        .foregroundColor(.green)
                        .font(.title)
                    
                    Text("No recent security events")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 20)
            } else {
                LazyVStack(spacing: 8) {
                    ForEach(fluxDefenseManager.recentEvents.prefix(3), id: \.id) { event in
                        EventRowView(event: event)
                    }
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct QuickActionsCard: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Quick Actions")
                .font(.headline)
                .fontWeight(.semibold)
            
            HStack(spacing: 12) {
                ActionButton(
                    title: "Scan System",
                    icon: "magnifyingglass.circle",
                    color: .blue
                ) {
                    fluxDefenseManager.startSystemScan()
                }
                
                ActionButton(
                    title: "Update Rules",
                    icon: "arrow.down.circle",
                    color: .green
                ) {
                    fluxDefenseManager.updateSecurityRules()
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct ActionButton: View {
    let title: String
    let icon: String
    let color: Color
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .foregroundColor(color)
                    .font(.title2)
                
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                    .multilineTextAlignment(.center)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 16)
            .background(Color(NSColor.controlBackgroundColor))
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .stroke(color.opacity(0.3), lineWidth: 1)
            )
            .cornerRadius(6)
        }
        .buttonStyle(PlainButtonStyle())
    }
}

