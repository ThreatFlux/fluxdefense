import SwiftUI

struct SecurityLogsView: View {
    @StateObject private var fluxDefenseManager = FluxDefenseManager.shared
    @State private var selectedFilter: EventFilter = .all
    @State private var searchText = ""
    @State private var selectedEvent: SecurityEvent?
    
    enum EventFilter: String, CaseIterable {
        case all = "All"
        case threats = "Threats"
        case file = "File Events"
        case network = "Network"
        
        var icon: String {
            switch self {
            case .all: return "list.bullet"
            case .threats: return "exclamationmark.triangle"
            case .file: return "doc"
            case .network: return "network"
            }
        }
    }
    
    var filteredEvents: [SecurityEvent] {
        let filtered = fluxDefenseManager.allEvents.filter { event in
            switch selectedFilter {
            case .all:
                return true
            case .threats:
                return event.severity == .high
            case .file:
                return event.type == .fileExecution || event.type == .fileAccess
            case .network:
                return event.type == .networkConnection
            }
        }
        
        if searchText.isEmpty {
            return filtered
        } else {
            return filtered.filter { event in
                event.description.localizedCaseInsensitiveContains(searchText) ||
                event.processPath.localizedCaseInsensitiveContains(searchText)
            }
        }
    }
    
    var body: some View {
        VStack(spacing: 0) {
            // Search and filters
            VStack(spacing: 12) {
                HStack {
                    Image(systemName: "magnifyingglass")
                        .foregroundColor(.secondary)
                    
                    TextField("Search events...", text: $searchText)
                        .textFieldStyle(PlainTextFieldStyle())
                }
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(Color(NSColor.controlBackgroundColor))
                .cornerRadius(6)
                
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(EventFilter.allCases, id: \.self) { filter in
                            FilterChip(
                                title: filter.rawValue,
                                icon: filter.icon,
                                isSelected: selectedFilter == filter
                            ) {
                                selectedFilter = filter
                            }
                        }
                    }
                    .padding(.horizontal, 16)
                }
            }
            .padding(16)
            
            Divider()
            
            // Events list
            if filteredEvents.isEmpty {
                VStack(spacing: 16) {
                    Image(systemName: selectedFilter == .threats ? "checkmark.shield" : "doc.text")
                        .foregroundColor(.secondary)
                        .font(.largeTitle)
                    
                    Text(selectedFilter == .threats ? "No threats detected" : "No events found")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    
                    if !searchText.isEmpty {
                        Text("Try adjusting your search terms")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(spacing: 1) {
                        ForEach(filteredEvents, id: \.id) { event in
                            EventRowView(event: event)
                                .background(Color(NSColor.controlBackgroundColor))
                                .onTapGesture {
                                    selectedEvent = event
                                }
                        }
                    }
                }
            }
        }
        .sheet(item: $selectedEvent) { event in
            EventDetailView(event: event)
        }
    }
}

struct FilterChip: View {
    let title: String
    let icon: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.caption)
                
                Text(title)
                    .font(.caption)
                    .fontWeight(.medium)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 6)
            .background(
                isSelected 
                    ? Color.accentColor
                    : Color(NSColor.controlBackgroundColor)
            )
            .foregroundColor(
                isSelected 
                    ? .white
                    : .primary
            )
            .cornerRadius(16)
        }
        .buttonStyle(PlainButtonStyle())
    }
}

struct EventRowView: View {
    let event: SecurityEvent
    
    var body: some View {
        HStack(spacing: 12) {
            // Icon and severity
            VStack(spacing: 4) {
                Image(systemName: event.type.icon)
                    .foregroundColor(event.severity.color)
                    .font(.title3)
                
                Circle()
                    .fill(event.severity.color)
                    .frame(width: 4, height: 4)
            }
            
            // Content
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(event.description)
                        .font(.system(.body, design: .default))
                        .fontWeight(.medium)
                        .lineLimit(1)
                    
                    Spacer()
                    
                    Text(event.timestamp.formatted(.relative(presentation: .numeric)))
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                HStack {
                    Label(event.processName, systemImage: "app")
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                    
                    Spacer()
                    
                    Text(event.verdict.rawValue.uppercased())
                        .font(.caption2)
                        .fontWeight(.bold)
                        .foregroundColor(.white)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(event.verdict.color)
                        .cornerRadius(3)
                }
            }
            
            Image(systemName: "chevron.right")
                .foregroundColor(.secondary)
                .font(.caption)
        }
        .padding(12)
        .background(Color(NSColor.controlBackgroundColor))
    }
}

struct EventDetailView: View {
    let event: SecurityEvent
    @Environment(\.dismiss) private var dismiss
    
    var body: some View {
        NavigationView {
            ScrollView {
                VStack(alignment: .leading, spacing: 20) {
                    // Header
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Image(systemName: event.type.icon)
                                .foregroundColor(event.severity.color)
                                .font(.title2)
                            
                            Text(event.description)
                                .font(.headline)
                                .fontWeight(.semibold)
                            
                            Spacer()
                        }
                        
                        HStack {
                            Text(event.verdict.rawValue.uppercased())
                                .font(.caption)
                                .fontWeight(.bold)
                                .foregroundColor(.white)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(event.verdict.color)
                                .cornerRadius(4)
                            
                            Text(event.timestamp.formatted(.dateTime))
                                .font(.caption)
                                .foregroundColor(.secondary)
                            
                            Spacer()
                        }
                    }
                    
                    Divider()
                    
                    // Details
                    VStack(alignment: .leading, spacing: 16) {
                        DetailSection(title: "Process Information") {
                            DetailRow(label: "Process", value: event.processName)
                            DetailRow(label: "Path", value: event.processPath)
                            DetailRow(label: "PID", value: "\(event.processId)")
                        }
                        
                        if event.type == .networkConnection {
                            DetailSection(title: "Network Details") {
                                DetailRow(label: "Remote IP", value: event.networkInfo?.remoteIP ?? "N/A")
                                DetailRow(label: "Port", value: "\(event.networkInfo?.port ?? 0)")
                                DetailRow(label: "Protocol", value: event.networkInfo?.networkProtocol ?? "N/A")
                            }
                        }
                        
                        if event.type == .fileExecution || event.type == .fileAccess {
                            DetailSection(title: "File Details") {
                                DetailRow(label: "Target Path", value: event.fileInfo?.path ?? "N/A")
                                DetailRow(label: "File Size", value: formatFileSize(event.fileInfo?.size ?? 0))
                                DetailRow(label: "Hash", value: event.fileInfo?.hash ?? "N/A")
                            }
                        }
                        
                        DetailSection(title: "Security Analysis") {
                            DetailRow(label: "Severity", value: event.severity.rawValue.capitalized)
                            DetailRow(label: "Reason", value: event.reason)
                            DetailRow(label: "Risk Score", value: "\(Int(event.riskScore * 100))%")
                        }
                    }
                }
                .padding(20)
            }
            .navigationTitle("Event Details")
            
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
        .frame(width: 500, height: 600)
    }
    
    private func formatFileSize(_ bytes: Int64) -> String {
        let formatter = ByteCountFormatter()
        formatter.allowedUnits = [.useBytes, .useKB, .useMB, .useGB]
        formatter.countStyle = .file
        return formatter.string(fromByteCount: bytes)
    }
}

struct DetailSection<Content: View>: View {
    let title: String
    let content: Content
    
    init(title: String, @ViewBuilder content: () -> Content) {
        self.title = title
        self.content = content()
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.headline)
                .fontWeight(.semibold)
            
            VStack(alignment: .leading, spacing: 6) {
                content
            }
            .padding(12)
            .background(Color(NSColor.controlBackgroundColor))
            .cornerRadius(8)
        }
    }
}

struct DetailRow: View {
    let label: String
    let value: String
    
    var body: some View {
        HStack {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
                .frame(width: 80, alignment: .leading)
            
            Text(value)
                .font(.caption)
                .fontWeight(.medium)
                .textSelection(.enabled)
            
            Spacer()
        }
    }
}

