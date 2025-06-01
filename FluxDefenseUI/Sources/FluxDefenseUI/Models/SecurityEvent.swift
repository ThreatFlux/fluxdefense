import Foundation
import SwiftUI

struct SecurityEvent: Identifiable, Codable {
    let id: UUID
    let timestamp: Date
    let type: EventType
    let severity: SecuritySeverity
    let description: String
    let processName: String
    let processPath: String
    let processId: Int32
    let verdict: Verdict
    let reason: String
    let riskScore: Double
    let fileInfo: FileInfo?
    let networkInfo: NetworkInfo?
    
    init(
        id: UUID = UUID(),
        timestamp: Date = Date(),
        type: EventType,
        severity: SecuritySeverity,
        description: String,
        processName: String,
        processPath: String,
        processId: Int32,
        verdict: Verdict,
        reason: String,
        riskScore: Double = 0.0,
        fileInfo: FileInfo? = nil,
        networkInfo: NetworkInfo? = nil
    ) {
        self.id = id
        self.timestamp = timestamp
        self.type = type
        self.severity = severity
        self.description = description
        self.processName = processName
        self.processPath = processPath
        self.processId = processId
        self.verdict = verdict
        self.reason = reason
        self.riskScore = riskScore
        self.fileInfo = fileInfo
        self.networkInfo = networkInfo
    }
}

enum EventType: String, Codable, CaseIterable {
    case fileExecution = "file_execution"
    case fileAccess = "file_access"
    case networkConnection = "network_connection"
    case processStart = "process_start"
    case systemCall = "system_call"
    
    var icon: String {
        switch self {
        case .fileExecution: return "play.circle"
        case .fileAccess: return "doc"
        case .networkConnection: return "network"
        case .processStart: return "gear"
        case .systemCall: return "terminal"
        }
    }
    
    var displayName: String {
        switch self {
        case .fileExecution: return "File Execution"
        case .fileAccess: return "File Access"
        case .networkConnection: return "Network Connection"
        case .processStart: return "Process Start"
        case .systemCall: return "System Call"
        }
    }
}

enum SecuritySeverity: String, Codable, CaseIterable {
    case low = "low"
    case medium = "medium"
    case high = "high"
    case critical = "critical"
    
    var color: Color {
        switch self {
        case .low: return .green
        case .medium: return .yellow
        case .high: return .orange
        case .critical: return .red
        }
    }
    
    var displayName: String {
        return rawValue.capitalized
    }
}

enum Verdict: String, Codable, CaseIterable {
    case allow = "allow"
    case deny = "deny"
    case log = "log"
    case quarantine = "quarantine"
    
    var color: Color {
        switch self {
        case .allow: return .green
        case .deny: return .red
        case .log: return .blue
        case .quarantine: return .orange
        }
    }
    
    var displayName: String {
        return rawValue.capitalized
    }
}

struct FileInfo: Codable {
    let path: String
    let size: Int64
    let hash: String
    let signature: String?
    let bundleIdentifier: String?
    let version: String?
    
    init(
        path: String,
        size: Int64 = 0,
        hash: String = "",
        signature: String? = nil,
        bundleIdentifier: String? = nil,
        version: String? = nil
    ) {
        self.path = path
        self.size = size
        self.hash = hash
        self.signature = signature
        self.bundleIdentifier = bundleIdentifier
        self.version = version
    }
}

struct NetworkInfo: Codable {
    let remoteIP: String
    let port: Int
    let networkProtocol: String
    let domain: String?
    let direction: NetworkDirection
    let bytesTransferred: Int64
    
    init(
        remoteIP: String,
        port: Int,
        networkProtocol: String,
        domain: String? = nil,
        direction: NetworkDirection = .outbound,
        bytesTransferred: Int64 = 0
    ) {
        self.remoteIP = remoteIP
        self.port = port
        self.networkProtocol = networkProtocol
        self.domain = domain
        self.direction = direction
        self.bytesTransferred = bytesTransferred
    }
}

enum NetworkDirection: String, Codable {
    case inbound = "inbound"
    case outbound = "outbound"
    case bidirectional = "bidirectional"
    
    var displayName: String {
        return rawValue.capitalized
    }
}

enum MonitoringMode: String, Codable, CaseIterable {
    case passive = "passive"
    case active = "active"
    
    var displayName: String {
        return rawValue.capitalized
    }
}

enum FluxDefenseStatus: String, Codable {
    case active = "Active"
    case passive = "Passive"
    case disabled = "Disabled"
    case error = "Error"
    
    var color: Color {
        switch self {
        case .active: return .green
        case .passive: return .yellow
        case .disabled: return .gray
        case .error: return .red
        }
    }
}

struct WhitelistEntry: Identifiable, Codable {
    let id: UUID
    let name: String
    let path: String
    let hash: String
    let dateAdded: Date
    let addedBy: String
    let verified: Bool
    
    init(
        id: UUID = UUID(),
        name: String,
        path: String,
        hash: String,
        dateAdded: Date = Date(),
        addedBy: String = "user",
        verified: Bool = false
    ) {
        self.id = id
        self.name = name
        self.path = path
        self.hash = hash
        self.dateAdded = dateAdded
        self.addedBy = addedBy
        self.verified = verified
    }
}

struct ProcessInformation: Identifiable, Codable {
    let id: UUID
    let pid: Int32
    let name: String
    let path: String
    let cpuUsage: Double
    let memoryUsage: Int64
    let parentPid: Int32
    let startTime: Date
    let user: String?
    
    init(
        id: UUID = UUID(),
        pid: Int32,
        name: String,
        path: String = "",
        cpuUsage: Double = 0.0,
        memoryUsage: Int64 = 0,
        parentPid: Int32 = 0,
        startTime: Date = Date(),
        user: String? = nil
    ) {
        self.id = id
        self.pid = pid
        self.name = name
        self.path = path
        self.cpuUsage = cpuUsage
        self.memoryUsage = memoryUsage
        self.parentPid = parentPid
        self.startTime = startTime
        self.user = user
    }
}

// MARK: - Sample Data Extensions
extension SecurityEvent {
    static var sampleEvents: [SecurityEvent] {
        [
            SecurityEvent(
                type: .fileExecution,
                severity: .medium,
                description: "Unknown binary executed",
                processName: "suspicious_app",
                processPath: "/tmp/suspicious_app",
                processId: 1234,
                verdict: .deny,
                reason: "File not in whitelist",
                riskScore: 0.75,
                fileInfo: FileInfo(
                    path: "/tmp/suspicious_app",
                    size: 2048576,
                    hash: "a1b2c3d4e5f6...",
                    signature: nil
                )
            ),
            SecurityEvent(
                type: .networkConnection,
                severity: .high,
                description: "Connection to suspicious IP",
                processName: "malware.exe",
                processPath: "/Applications/malware.exe",
                processId: 5678,
                verdict: .deny,
                reason: "IP address on blocklist",
                riskScore: 0.90,
                networkInfo: NetworkInfo(
                    remoteIP: "192.168.1.100",
                    port: 8080,
                    networkProtocol: "TCP",
                    domain: "suspicious.example.com",
                    direction: .outbound
                )
            ),
            SecurityEvent(
                type: .fileAccess,
                severity: .low,
                description: "System file accessed",
                processName: "System Preferences",
                processPath: "/System/Applications/System Preferences.app",
                processId: 9012,
                verdict: .allow,
                reason: "Trusted system application",
                riskScore: 0.10,
                fileInfo: FileInfo(
                    path: "/System/Library/CoreServices/SystemUIServer.app",
                    size: 1024000,
                    hash: "f1e2d3c4b5a6...",
                    signature: "Apple Inc."
                )
            )
        ]
    }
}