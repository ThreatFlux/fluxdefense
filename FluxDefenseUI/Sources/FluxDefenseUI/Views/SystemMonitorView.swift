import SwiftUI
import Foundation

struct SystemMonitorView: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    @State private var selectedTimeRange: TimeRange = .hour
    
    enum TimeRange: String, CaseIterable {
        case minute = "1 min"
        case hour = "1 hour"
        case day = "24 hours"
        case week = "7 days"
        
        var seconds: Int {
            switch self {
            case .minute: return 60
            case .hour: return 3600
            case .day: return 86400
            case .week: return 604800
            }
        }
    }
    
    var body: some View {
        ScrollView {
            LazyVStack(spacing: 16) {
                // Real-time overview
                RealTimeOverviewCard()
                
                // Time range selector
                TimeRangeSelector(selectedRange: $selectedTimeRange)
                
                // CPU Usage Chart
                SystemMetricCard(
                    title: "CPU Usage",
                    currentValue: systemMonitor.cpuUsage,
                    historicalData: systemMonitor.cpuHistory,
                    color: .blue,
                    unit: "%"
                )
                
                // Memory Usage Chart
                SystemMetricCard(
                    title: "Memory Usage",
                    currentValue: systemMonitor.memoryUsage,
                    historicalData: systemMonitor.memoryHistory,
                    color: .orange,
                    unit: "%"
                )
                
                // Network I/O
                NetworkIOCard()
                
                // Disk I/O
                DiskIOCard()
                
                // Process Information - Full Task Manager View
                ProcessStatsView()
                    .frame(minHeight: 400)
            }
            .padding(16)
        }
        .onAppear {
            systemMonitor.updateTimeRange(selectedTimeRange.seconds)
        }
        .onChange(of: selectedTimeRange) { range in
            systemMonitor.updateTimeRange(range.seconds)
        }
    }
}

struct RealTimeOverviewCard: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Real-Time System Status")
                .font(.headline)
                .fontWeight(.semibold)
            
            HStack(spacing: 16) {
                MetricGauge(
                    title: "CPU",
                    value: systemMonitor.cpuUsage,
                    color: .blue,
                    unit: "%"
                )
                
                MetricGauge(
                    title: "Memory",
                    value: systemMonitor.memoryUsage,
                    color: .orange,
                    unit: "%"
                )
                
                MetricGauge(
                    title: "Disk",
                    value: systemMonitor.diskUsage,
                    color: .purple,
                    unit: "%"
                )
            }
            
            HStack(spacing: 16) {
                NetworkMetric(
                    title: "Network In",
                    value: systemMonitor.networkInRate,
                    color: .green
                )
                
                NetworkMetric(
                    title: "Network Out",
                    value: systemMonitor.networkOutRate,
                    color: .red
                )
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct MetricGauge: View {
    let title: String
    let value: Double
    let color: Color
    let unit: String
    
    var body: some View {
        VStack(spacing: 8) {
            ZStack {
                Circle()
                    .stroke(color.opacity(0.2), lineWidth: 4)
                
                Circle()
                    .trim(from: 0, to: value / 100)
                    .stroke(color, style: StrokeStyle(lineWidth: 4, lineCap: .round))
                    .rotationEffect(.degrees(-90))
                
                VStack(spacing: 2) {
                    Text("\(Int(value))")
                        .font(.title2)
                        .fontWeight(.bold)
                    
                    Text(unit)
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }
            }
            .frame(width: 60, height: 60)
            
            Text(title)
                .font(.caption)
                .fontWeight(.medium)
        }
    }
}

struct NetworkMetric: View {
    let title: String
    let value: Double
    let color: Color
    
    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Circle()
                    .fill(color)
                    .frame(width: 6, height: 6)
                
                Text(title)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Text(formatNetworkRate(value))
                .font(.subheadline)
                .fontWeight(.semibold)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
    
    private func formatNetworkRate(_ bytesPerSecond: Double) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .binary
        return "\(formatter.string(fromByteCount: Int64(bytesPerSecond)))/s"
    }
}

struct TimeRangeSelector: View {
    @Binding var selectedRange: SystemMonitorView.TimeRange
    
    var body: some View {
        HStack {
            Text("Time Range")
                .font(.subheadline)
                .fontWeight(.medium)
            
            Spacer()
            
            HStack(spacing: 4) {
                ForEach(SystemMonitorView.TimeRange.allCases, id: \.self) { range in
                    Button(range.rawValue) {
                        selectedRange = range
                    }
                    .font(.caption)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(
                        selectedRange == range 
                            ? Color.accentColor 
                            : Color(NSColor.controlBackgroundColor)
                    )
                    .foregroundColor(
                        selectedRange == range 
                            ? .white 
                            : .primary
                    )
                    .cornerRadius(4)
                }
            }
        }
        .padding(.horizontal, 16)
    }
}

struct SystemMetricCard: View {
    let title: String
    let currentValue: Double
    let historicalData: [Double]
    let color: Color
    let unit: String
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text(title)
                    .font(.headline)
                    .fontWeight(.semibold)
                
                Spacer()
                
                Text("\(Int(currentValue))\(unit)")
                    .font(.title2)
                    .fontWeight(.bold)
                    .foregroundColor(color)
            }
            
            LineChart(data: historicalData, color: color)
                .frame(height: 80)
            
            HStack {
                HStack(spacing: 4) {
                    Text("Min:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Text("\(Int(historicalData.min() ?? 0))\(unit)")
                        .font(.caption)
                        .fontWeight(.medium)
                }
                
                Spacer()
                
                HStack(spacing: 4) {
                    Text("Avg:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Text("\(Int(historicalData.isEmpty ? 0 : historicalData.reduce(0, +) / Double(historicalData.count)))\(unit)")
                        .font(.caption)
                        .fontWeight(.medium)
                }
                
                Spacer()
                
                HStack(spacing: 4) {
                    Text("Max:")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    
                    Text("\(Int(historicalData.max() ?? 0))\(unit)")
                        .font(.caption)
                        .fontWeight(.medium)
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct LineChart: View {
    let data: [Double]
    let color: Color
    
    var body: some View {
        GeometryReader { geometry in
            if data.count > 1 {
                Path { path in
                    let maxValue = data.max() ?? 1
                    let width = geometry.size.width
                    let height = geometry.size.height
                    let stepX = width / CGFloat(data.count - 1)
                    
                    for (index, value) in data.enumerated() {
                        let x = CGFloat(index) * stepX
                        let y = height - (CGFloat(value) / CGFloat(maxValue)) * height
                        
                        if index == 0 {
                            path.move(to: CGPoint(x: x, y: y))
                        } else {
                            path.addLine(to: CGPoint(x: x, y: y))
                        }
                    }
                }
                .stroke(color, lineWidth: 2)
                
                // Fill area under curve
                Path { path in
                    let maxValue = data.max() ?? 1
                    let width = geometry.size.width
                    let height = geometry.size.height
                    let stepX = width / CGFloat(data.count - 1)
                    
                    path.move(to: CGPoint(x: 0, y: height))
                    
                    for (index, value) in data.enumerated() {
                        let x = CGFloat(index) * stepX
                        let y = height - (CGFloat(value) / CGFloat(maxValue)) * height
                        path.addLine(to: CGPoint(x: x, y: y))
                    }
                    
                    path.addLine(to: CGPoint(x: geometry.size.width, y: height))
                    path.closeSubpath()
                }
                .fill(LinearGradient(
                    colors: [color.opacity(0.3), color.opacity(0.1)],
                    startPoint: .top,
                    endPoint: .bottom
                ))
            } else {
                Rectangle()
                    .fill(Color.secondary.opacity(0.2))
                    .overlay(
                        Text("No data")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    )
            }
        }
    }
}

struct NetworkIOCard: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Network I/O")
                .font(.headline)
                .fontWeight(.semibold)
            
            HStack(spacing: 20) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Circle()
                            .fill(.green)
                            .frame(width: 8, height: 8)
                        
                        Text("Download")
                            .font(.subheadline)
                            .fontWeight(.medium)
                    }
                    
                    Text(formatBytes(systemMonitor.networkInRate))
                        .font(.title2)
                        .fontWeight(.bold)
                        .foregroundColor(.green)
                    
                    Text("Total: \(formatBytes(systemMonitor.totalNetworkIn))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                VStack(alignment: .trailing, spacing: 8) {
                    HStack {
                        Text("Upload")
                            .font(.subheadline)
                            .fontWeight(.medium)
                        
                        Circle()
                            .fill(.red)
                            .frame(width: 8, height: 8)
                    }
                    
                    Text(formatBytes(systemMonitor.networkOutRate))
                        .font(.title2)
                        .fontWeight(.bold)
                        .foregroundColor(.red)
                    
                    Text("Total: \(formatBytes(systemMonitor.totalNetworkOut))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
    
    private func formatBytes(_ bytes: Double) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

struct DiskIOCard: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Disk I/O")
                .font(.headline)
                .fontWeight(.semibold)
            
            HStack(spacing: 20) {
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Circle()
                            .fill(.blue)
                            .frame(width: 8, height: 8)
                        
                        Text("Read")
                            .font(.subheadline)
                            .fontWeight(.medium)
                    }
                    
                    Text("\(formatBytes(systemMonitor.diskReadRate))/s")
                        .font(.title2)
                        .fontWeight(.bold)
                        .foregroundColor(.blue)
                    
                    Text("Total: \(formatBytes(systemMonitor.totalDiskRead))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Spacer()
                
                VStack(alignment: .trailing, spacing: 8) {
                    HStack {
                        Text("Write")
                            .font(.subheadline)
                            .fontWeight(.medium)
                        
                        Circle()
                            .fill(.orange)
                            .frame(width: 8, height: 8)
                    }
                    
                    Text("\(formatBytes(systemMonitor.diskWriteRate))/s")
                        .font(.title2)
                        .fontWeight(.bold)
                        .foregroundColor(.orange)
                    
                    Text("Total: \(formatBytes(systemMonitor.totalDiskWrite))")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
    
    private func formatBytes(_ bytes: Double) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        formatter.countStyle = .binary
        return formatter.string(fromByteCount: Int64(bytes))
    }
}

struct TopProcessesCard: View {
    @StateObject private var systemMonitor = SystemMonitor.shared
    
    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Top Processes by CPU")
                .font(.headline)
                .fontWeight(.semibold)
            
            if systemMonitor.topProcesses.isEmpty {
                Text("Loading process information...")
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.vertical, 20)
            } else {
                LazyVStack(spacing: 8) {
                    ForEach(systemMonitor.topProcesses.prefix(5), id: \.pid) { process in
                        ProcessRowView(process: process)
                    }
                }
            }
        }
        .padding(16)
        .background(Color(NSColor.controlBackgroundColor))
        .cornerRadius(8)
    }
}

struct ProcessRowView: View {
    let process: ProcessInformation
    
    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(process.name)
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .lineLimit(1)
                
                Text("PID: \(process.pid)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            Spacer()
            
            VStack(alignment: .trailing, spacing: 2) {
                Text("\(String(format: "%.1f", process.cpuUsage))%")
                    .font(.subheadline)
                    .fontWeight(.semibold)
                
                Text(formatMemory(process.memoryUsage))
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
    
    private func formatMemory(_ bytes: Int64) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useMB, .useGB]
        formatter.countStyle = .memory
        return formatter.string(fromByteCount: bytes)
    }
}

