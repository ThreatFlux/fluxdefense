import SwiftUI

struct ContentView: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    @StateObject private var systemMonitor = SystemMonitor.shared
    @State private var selectedTab: Tab = .dashboard
    
    enum Tab: String, CaseIterable {
        case dashboard = "Dashboard"
        case logs = "Security Logs"
        case system = "System Monitor"
        case settings = "Settings"
        
        var icon: String {
            switch self {
            case .dashboard: return "gauge.high"
            case .logs: return "doc.text.magnifyingglass"
            case .system: return "chart.line.uptrend.xyaxis"
            case .settings: return "gear"
            }
        }
    }
    
    var body: some View {
        VStack(spacing: 0) {
            // Header
            HeaderView(selectedTab: $selectedTab)
            
            Divider()
            
            // Content
            Group {
                switch selectedTab {
                case .dashboard:
                    DashboardView()
                case .logs:
                    SecurityLogsView()
                case .system:
                    SystemMonitorView()
                case .settings:
                    SettingsView()
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .background(Color(NSColor.controlBackgroundColor))
        .frame(width: 400, height: 500)
    }
}

struct HeaderView: View {
    @Binding var selectedTab: ContentView.Tab
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    
    var body: some View {
        VStack(spacing: 8) {
            // Logo and title
            HStack {
                Image(systemName: "shield.checkered")
                    .foregroundColor(.blue)
                    .font(.title2)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text("FluxDefense")
                        .font(.headline)
                        .fontWeight(.semibold)
                    
                    Text(fluxDefenseManager.status.rawValue)
                        .font(.caption)
                        .foregroundColor(fluxDefenseManager.status.color)
                }
                
                Spacer()
                
                // Status indicator
                Circle()
                    .fill(fluxDefenseManager.status.color)
                    .frame(width: 8, height: 8)
            }
            .padding(.horizontal, 16)
            .padding(.top, 12)
            
            // Tab selector
            HStack(spacing: 4) {
                ForEach(ContentView.Tab.allCases, id: \.self) { tab in
                    Button(action: { selectedTab = tab }) {
                        HStack(spacing: 4) {
                            Image(systemName: tab.icon)
                                .font(.caption)
                            
                            if selectedTab == tab {
                                Text(tab.rawValue)
                                    .font(.caption)
                                    .fontWeight(.medium)
                            }
                        }
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(
                            selectedTab == tab 
                                ? Color.accentColor.opacity(0.2)
                                : Color.clear
                        )
                        .foregroundColor(
                            selectedTab == tab 
                                ? .accentColor
                                : .secondary
                        )
                        .cornerRadius(6)
                    }
                    .buttonStyle(PlainButtonStyle())
                }
            }
            .padding(.horizontal, 12)
            .padding(.bottom, 8)
        }
    }
}

