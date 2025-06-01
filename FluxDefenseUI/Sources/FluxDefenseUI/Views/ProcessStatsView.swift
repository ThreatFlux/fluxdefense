import SwiftUI

struct ProcessStatsView: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    @State private var sortBy: SortField = .cpu
    @State private var sortAscending = false
    @State private var searchText = ""
    @State private var selectedProcess: ProcessInformation?
    
    enum SortField: String, CaseIterable {
        case name = "Name"
        case cpu = "CPU"
        case memory = "Memory"
        case pid = "PID"
        
        var icon: String {
            switch self {
            case .name: return "textformat"
            case .cpu: return "cpu"
            case .memory: return "memorychip"
            case .pid: return "number"
            }
        }
    }
    
    var sortedProcesses: [ProcessInformation] {
        let filtered = searchText.isEmpty 
            ? systemMonitor.allProcesses 
            : systemMonitor.allProcesses.filter { 
                $0.name.localizedCaseInsensitiveContains(searchText) 
            }
        
        return filtered.sorted { p1, p2 in
            switch sortBy {
            case .name:
                return sortAscending ? p1.name < p2.name : p1.name > p2.name
            case .cpu:
                return sortAscending ? p1.cpuUsage < p2.cpuUsage : p1.cpuUsage > p2.cpuUsage
            case .memory:
                return sortAscending ? p1.memoryUsage < p2.memoryUsage : p1.memoryUsage > p2.memoryUsage
            case .pid:
                return sortAscending ? p1.pid < p2.pid : p1.pid > p2.pid
            }
        }
    }
    
    var body: some View {
        VStack(spacing: 0) {
            // Header with search and stats
            ProcessStatsHeader(
                searchText: $searchText,
                processCount: systemMonitor.allProcesses.count
            )
            
            Divider()
            
            // Column headers
            ProcessTableHeader(
                sortBy: $sortBy,
                sortAscending: $sortAscending
            )
            
            Divider()
            
            // Process list
            ScrollView {
                LazyVStack(spacing: 0) {
                    ForEach(sortedProcesses, id: \.pid) { process in
                        ProcessRowDetailView(
                            process: process,
                            isSelected: selectedProcess?.pid == process.pid
                        )
                        .onTapGesture {
                            selectedProcess = process
                        }
                        
                        Divider()
                            .opacity(0.5)
                    }
                }
            }
            .frame(maxHeight: .infinity)
            
            // Bottom stats bar
            ProcessStatsBar()
        }
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct ProcessStatsHeader: View {
    @Binding var searchText: String
    let processCount: Int
    
    var body: some View {
        HStack(spacing: 12) {
            // Search field
            HStack {
                Image(systemName: "magnifyingglass")
                    .foregroundColor(.secondary)
                
                TextField("Search processes...", text: $searchText)
                    .textFieldStyle(PlainTextFieldStyle())
                
                if !searchText.isEmpty {
                    Button(action: { searchText = "" }) {
                        Image(systemName: "xmark.circle.fill")
                            .foregroundColor(.secondary)
                    }
                    .buttonStyle(PlainButtonStyle())
                }
            }
            .padding(6)
            .background(Color(NSColor.textBackgroundColor))
            .cornerRadius(6)
            .frame(maxWidth: 200)
            
            Spacer()
            
            // Process count
            Text("\(processCount) processes")
                .font(.caption)
                .foregroundColor(.secondary)
            
            // Refresh button
            Button(action: {
                SystemMonitor.shared.refreshProcessList()
            }) {
                Image(systemName: "arrow.clockwise")
                    .font(.caption)
            }
            .buttonStyle(PlainButtonStyle())
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}

struct ProcessTableHeader: View {
    @Binding var sortBy: ProcessStatsView.SortField
    @Binding var sortAscending: Bool
    
    var body: some View {
        HStack(spacing: 0) {
            // Process name
            HeaderButton(
                title: "Process Name",
                icon: "textformat",
                isActive: sortBy == .name,
                isAscending: sortAscending,
                width: nil
            ) {
                if sortBy == .name {
                    sortAscending.toggle()
                } else {
                    sortBy = .name
                    sortAscending = false
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            // PID
            HeaderButton(
                title: "PID",
                icon: "number",
                isActive: sortBy == .pid,
                isAscending: sortAscending,
                width: 60
            ) {
                if sortBy == .pid {
                    sortAscending.toggle()
                } else {
                    sortBy = .pid
                    sortAscending = false
                }
            }
            
            // CPU
            HeaderButton(
                title: "CPU %",
                icon: "cpu",
                isActive: sortBy == .cpu,
                isAscending: sortAscending,
                width: 80
            ) {
                if sortBy == .cpu {
                    sortAscending.toggle()
                } else {
                    sortBy = .cpu
                    sortAscending = false
                }
            }
            
            // Memory
            HeaderButton(
                title: "Memory",
                icon: "memorychip",
                isActive: sortBy == .memory,
                isAscending: sortAscending,
                width: 80
            ) {
                if sortBy == .memory {
                    sortAscending.toggle()
                } else {
                    sortBy = .memory
                    sortAscending = false
                }
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color(NSColor.textBackgroundColor).opacity(0.5))
    }
}

struct HeaderButton: View {
    let title: String
    let icon: String
    let isActive: Bool
    let isAscending: Bool
    let width: CGFloat?
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.caption2)
                
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
                
                if isActive {
                    Image(systemName: isAscending ? "chevron.up" : "chevron.down")
                        .font(.caption2)
                }
            }
            .foregroundColor(isActive ? .accentColor : .primary)
            .frame(width: width, alignment: width == nil ? .leading : .trailing)
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct ProcessRowDetailView: View {
    let process: ProcessInformation
    let isSelected: Bool
    
    var body: some View {
        HStack(spacing: 0) {
            // Process name
            HStack(spacing: 8) {
                // Status indicator
                Circle()
                    .fill(cpuColor(for: process.cpuUsage))
                    .frame(width: 6, height: 6)
                
                VStack(alignment: .leading, spacing: 2) {
                    Text(process.name)
                        .font(.system(size: 11))
                        .fontWeight(isSelected ? .medium : .regular)
                        .lineLimit(1)
                    
                    if let user = process.user {
                        Text(user)
                            .font(.system(size: 9))
                            .foregroundColor(.secondary)
                    }
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            
            // PID
            Text("\(process.pid)")
                .font(.system(size: 11, design: .monospaced))
                .foregroundColor(.secondary)
                .frame(width: 60, alignment: .trailing)
            
            // CPU Usage with bar
            HStack(spacing: 4) {
                // CPU bar
                GeometryReader { geometry in
                    ZStack(alignment: .leading) {
                        Rectangle()
                            .fill(Color.gray.opacity(0.2))
                            .frame(height: 4)
                        
                        Rectangle()
                            .fill(cpuColor(for: process.cpuUsage))
                            .frame(width: geometry.size.width * min(process.cpuUsage / 100.0, 1.0), height: 4)
                    }
                }
                .frame(width: 30, height: 4)
                
                Text(String(format: "%.1f", process.cpuUsage))
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundColor(cpuColor(for: process.cpuUsage))
            }
            .frame(width: 80, alignment: .trailing)
            
            // Memory Usage
            Text(formatMemory(process.memoryUsage))
                .font(.system(size: 11, design: .monospaced))
                .frame(width: 80, alignment: .trailing)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(
            isSelected 
                ? Color.accentColor.opacity(0.1) 
                : Color.clear
        )
    }
    
    private func cpuColor(for usage: Double) -> Color {
        if usage > 50 {
            return .red
        } else if usage > 25 {
            return .orange
        } else if usage > 10 {
            return .yellow
        } else {
            return .green
        }
    }
    
    private func formatMemory(_ bytes: Int64) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .memory
        formatter.allowsNonnumericFormatting = false
        return formatter.string(fromByteCount: bytes)
    }
}

struct ProcessStatsBar: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        HStack(spacing: 20) {
            // Total CPU
            StatItem(
                label: "Total CPU",
                value: String(format: "%.1f%%", systemMonitor.cpuUsage),
                color: .blue
            )
            
            // Total Memory
            StatItem(
                label: "Memory Used",
                value: String(format: "%.1f%%", systemMonitor.memoryUsage),
                color: .orange
            )
            
            // Active processes
            StatItem(
                label: "Active",
                value: "\(systemMonitor.activeProcessCount)",
                color: .green
            )
            
            Spacer()
            
            // Update time
            Text("Updated: \(formatUpdateTime())")
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color(NSColor.textBackgroundColor).opacity(0.5))
    }
    
    private func formatUpdateTime() -> String {
        let formatter = DateFormatter()
        formatter.timeStyle = .medium
        return formatter.string(from: Date())
    }
}

struct StatItem: View {
    let label: String
    let value: String
    let color: Color
    
    var body: some View {
        HStack(spacing: 4) {
            Circle()
                .fill(color)
                .frame(width: 6, height: 6)
            
            Text(label)
                .font(.caption2)
                .foregroundColor(.secondary)
            
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
        }
    }
}