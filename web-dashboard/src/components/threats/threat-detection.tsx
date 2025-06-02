import { useState } from 'react'
import { 
  AlertTriangle, 
  Shield, 
  Skull,
  Bug,
  Zap,
  FileText,
  Download,
  Search,
  Filter,
  Clock,
  CheckCircle,
  XCircle,
  Eye,
  Trash2
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

type ThreatSeverity = 'low' | 'medium' | 'high' | 'critical'
type ThreatType = 'malware' | 'virus' | 'trojan' | 'ransomware' | 'spyware' | 'rootkit' | 'suspicious_activity' | 'crypto_miner'
type ThreatStatus = 'detected' | 'quarantined' | 'removed' | 'ignored' | 'investigating'

interface ThreatDetection {
  id: string
  timestamp: string
  name: string
  type: ThreatType
  severity: ThreatSeverity
  status: ThreatStatus
  filePath: string
  fileHash: string
  fileSize: number
  confidence: number
  description: string
  source: string
  recommendations: string[]
  signatures: string[]
  behaviorAnalysis: {
    networkConnections: string[]
    fileModifications: string[]
    registryChanges: string[]
    processSpawning: string[]
  }
}

interface MalwareSignature {
  id: string
  name: string
  type: string
  pattern: string
  severity: ThreatSeverity
  lastUpdated: string
  detectionCount: number
}

const mockThreats: ThreatDetection[] = [
  {
    id: '1',
    timestamp: '2024-06-01 15:42:33',
    name: 'Trojan.Generic.KD.12345',
    type: 'trojan',
    severity: 'critical',
    status: 'quarantined',
    filePath: '/tmp/malicious_binary',
    fileHash: 'a1b2c3d4e5f6789012345678901234567890abcd',
    fileSize: 2048576,
    confidence: 95,
    description: 'Generic trojan detected attempting to establish backdoor connection',
    source: 'Signature Detection',
    recommendations: [
      'Quarantine the file immediately',
      'Scan system for additional infections',
      'Update antivirus signatures',
      'Review network logs for suspicious activity'
    ],
    signatures: ['Trojan.Generic.KD', 'Backdoor.Connection.Pattern'],
    behaviorAnalysis: {
      networkConnections: ['45.123.456.789:4444', 'malicious.domain.com:8080'],
      fileModifications: ['/etc/hosts', '/home/user/.bashrc'],
      registryChanges: [],
      processSpawning: ['/bin/bash', '/usr/bin/curl']
    }
  },
  {
    id: '2',
    timestamp: '2024-06-01 15:38:15',
    name: 'CryptoMiner.XMRig.Variant',
    type: 'crypto_miner',
    severity: 'high',
    status: 'removed',
    filePath: '/tmp/xmrig',
    fileHash: 'b2c3d4e5f6789012345678901234567890abcde1',
    fileSize: 4194304,
    confidence: 98,
    description: 'Cryptocurrency mining malware using XMRig mining pool',
    source: 'Behavior Analysis',
    recommendations: [
      'Remove mining executable',
      'Check for persistence mechanisms',
      'Monitor CPU usage patterns',
      'Block mining pool domains'
    ],
    signatures: ['CryptoMiner.XMRig', 'Mining.Pool.Connection'],
    behaviorAnalysis: {
      networkConnections: ['pool.supportxmr.com:3333', 'pool.minexmr.com:4444'],
      fileModifications: ['/etc/crontab'],
      registryChanges: [],
      processSpawning: ['xmrig', 'wget']
    }
  },
  {
    id: '3',
    timestamp: '2024-06-01 15:25:07',
    name: 'Suspicious.Binary.Execution',
    type: 'suspicious_activity',
    severity: 'medium',
    status: 'investigating',
    filePath: '/tmp/unknown_binary',
    fileHash: 'c3d4e5f6789012345678901234567890abcdef12',
    fileSize: 1024000,
    confidence: 75,
    description: 'Unknown binary with suspicious execution patterns',
    source: 'Heuristic Analysis',
    recommendations: [
      'Analyze binary in sandbox environment',
      'Check file reputation databases',
      'Monitor for additional suspicious behavior',
      'Consider quarantining if risk increases'
    ],
    signatures: ['Suspicious.Execution.Pattern'],
    behaviorAnalysis: {
      networkConnections: ['unknown.server.com:9999'],
      fileModifications: ['/tmp/config.dat'],
      registryChanges: [],
      processSpawning: ['/bin/sh']
    }
  }
]

const mockSignatures: MalwareSignature[] = [
  {
    id: '1',
    name: 'Trojan.Generic.KD',
    type: 'Signature',
    pattern: 'hex:4D5A90000300000004000000FFFF0000',
    severity: 'critical',
    lastUpdated: '2024-06-01 12:00:00',
    detectionCount: 23
  },
  {
    id: '2',
    name: 'CryptoMiner.XMRig',
    type: 'Hash',
    pattern: 'sha256:a1b2c3d4e5f6...',
    severity: 'high',
    lastUpdated: '2024-06-01 10:30:00',
    detectionCount: 15
  },
  {
    id: '3',
    name: 'Backdoor.Connection.Pattern',
    type: 'Network',
    pattern: 'tcp:*:4444,*:8080',
    severity: 'high',
    lastUpdated: '2024-06-01 08:15:00',
    detectionCount: 8
  }
]

const getSeverityColor = (severity: ThreatSeverity) => {
  switch (severity) {
    case 'low': return 'text-blue-500'
    case 'medium': return 'text-yellow-500'
    case 'high': return 'text-orange-500'
    case 'critical': return 'text-red-500'
    default: return 'text-gray-500'
  }
}

const getSeverityBadge = (severity: ThreatSeverity) => {
  switch (severity) {
    case 'low': return <Badge variant="secondary">Low</Badge>
    case 'medium': return <Badge variant="outline">Medium</Badge>
    case 'high': return <Badge variant="destructive">High</Badge>
    case 'critical': return <Badge variant="destructive" className="bg-red-600">Critical</Badge>
    default: return <Badge variant="secondary">Unknown</Badge>
  }
}

const getStatusIcon = (status: ThreatStatus) => {
  switch (status) {
    case 'detected': return <AlertTriangle className="h-4 w-4 text-orange-500" />
    case 'quarantined': return <Shield className="h-4 w-4 text-blue-500" />
    case 'removed': return <CheckCircle className="h-4 w-4 text-green-500" />
    case 'ignored': return <XCircle className="h-4 w-4 text-gray-500" />
    case 'investigating': return <Eye className="h-4 w-4 text-yellow-500" />
    default: return <AlertTriangle className="h-4 w-4 text-gray-500" />
  }
}

const getStatusBadge = (status: ThreatStatus) => {
  switch (status) {
    case 'detected': return <Badge variant="destructive">Detected</Badge>
    case 'quarantined': return <Badge variant="outline">Quarantined</Badge>
    case 'removed': return <Badge variant="secondary">Removed</Badge>
    case 'ignored': return <Badge variant="outline" className="opacity-60">Ignored</Badge>
    case 'investigating': return <Badge variant="default">Investigating</Badge>
    default: return <Badge variant="outline">Unknown</Badge>
  }
}

const getThreatIcon = (type: ThreatType) => {
  switch (type) {
    case 'malware': return <Bug className="h-4 w-4" />
    case 'virus': return <Skull className="h-4 w-4" />
    case 'trojan': return <AlertTriangle className="h-4 w-4" />
    case 'ransomware': return <Zap className="h-4 w-4" />
    case 'spyware': return <Eye className="h-4 w-4" />
    case 'rootkit': return <Shield className="h-4 w-4" />
    case 'suspicious_activity': return <AlertTriangle className="h-4 w-4" />
    case 'crypto_miner': return <Zap className="h-4 w-4" />
    default: return <Bug className="h-4 w-4" />
  }
}

const formatBytes = (bytes: number): string => {
  const sizes = ['B', 'KB', 'MB', 'GB']
  if (bytes === 0) return '0 B'
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round((bytes / Math.pow(1024, i)) * 100) / 100 + ' ' + sizes[i]
}

export function ThreatDetection() {
  const [selectedThreat, setSelectedThreat] = useState<ThreatDetection | null>(null)
  const [activeTab, setActiveTab] = useState<'threats' | 'signatures'>('threats')

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Threat Detection</h2>
          <p className="text-muted-foreground">
            Advanced malware detection, analysis, and threat intelligence
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button variant="outline" size="sm">
            <Search className="h-4 w-4 mr-2" />
            Search
          </Button>
          <Button variant="outline" size="sm">
            <Filter className="h-4 w-4 mr-2" />
            Filter
          </Button>
          <Button variant="outline" size="sm">
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Statistics */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Threats</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">3</div>
            <p className="text-xs text-muted-foreground">
              Require immediate attention
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Quarantined</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-500">15</div>
            <p className="text-xs text-muted-foreground">
              Safely contained
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Signatures</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">12,847</div>
            <p className="text-xs text-muted-foreground">
              Updated 2h ago
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Detection Rate</CardTitle>
            <CheckCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">99.7%</div>
            <p className="text-xs text-muted-foreground">
              Last 30 days
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <div className="flex space-x-2">
        <Button
          variant={activeTab === 'threats' ? 'default' : 'outline'}
          onClick={() => setActiveTab('threats')}
        >
          <AlertTriangle className="h-4 w-4 mr-2" />
          Threat Detections
        </Button>
        <Button
          variant={activeTab === 'signatures' ? 'default' : 'outline'}
          onClick={() => setActiveTab('signatures')}
        >
          <FileText className="h-4 w-4 mr-2" />
          Malware Signatures
        </Button>
      </div>

      {/* Content */}
      {activeTab === 'threats' ? (
        <div className="grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Detected Threats</CardTitle>
              <CardDescription>
                Malware and suspicious activity detected by the security engine
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {mockThreats.map((threat) => (
                <div
                  key={threat.id}
                  className="flex items-start space-x-3 p-3 rounded-lg border cursor-pointer hover:bg-accent/50 transition-colors"
                  onClick={() => setSelectedThreat(threat)}
                >
                  <div className={`${getSeverityColor(threat.severity)} mt-1`}>
                    {getThreatIcon(threat.type)}
                  </div>
                  <div className="flex-1 space-y-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium leading-none">
                        {threat.name}
                      </p>
                      <div className="flex items-center space-x-2">
                        {getStatusIcon(threat.status)}
                        {getSeverityBadge(threat.severity)}
                      </div>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {threat.description}
                    </p>
                    <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                      <span>Confidence: {threat.confidence}%</span>
                      <span>•</span>
                      <span>{formatBytes(threat.fileSize)}</span>
                      <span>•</span>
                      <span>{threat.timestamp}</span>
                    </div>
                    <div className="text-xs text-muted-foreground font-mono">
                      {threat.filePath}
                    </div>
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>

          {/* Threat Details */}
          <Card>
            <CardHeader>
              <CardTitle>Threat Analysis</CardTitle>
              <CardDescription>
                {selectedThreat ? 'Detailed analysis of the selected threat' : 'Select a threat to view detailed analysis'}
              </CardDescription>
            </CardHeader>
            <CardContent>
              {selectedThreat ? (
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h3 className="text-lg font-semibold">{selectedThreat.name}</h3>
                    <div className="flex items-center space-x-2">
                      {getStatusBadge(selectedThreat.status)}
                      {getSeverityBadge(selectedThreat.severity)}
                    </div>
                  </div>
                  
                  <div className="space-y-3">
                    <div className="grid grid-cols-2 gap-2 text-sm">
                      <div>
                        <span className="text-muted-foreground">Detection Time:</span>
                        <div>{selectedThreat.timestamp}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Confidence:</span>
                        <div>{selectedThreat.confidence}%</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Type:</span>
                        <div className="capitalize">{selectedThreat.type.replace('_', ' ')}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Source:</span>
                        <div>{selectedThreat.source}</div>
                      </div>
                    </div>

                    <div>
                      <span className="text-muted-foreground text-sm">File Information:</span>
                      <div className="mt-2 p-3 bg-muted rounded-lg space-y-1">
                        <div className="text-sm font-mono">{selectedThreat.filePath}</div>
                        <div className="text-xs text-muted-foreground">
                          Hash: {selectedThreat.fileHash}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          Size: {formatBytes(selectedThreat.fileSize)}
                        </div>
                      </div>
                    </div>

                    <div>
                      <span className="text-muted-foreground text-sm">Matched Signatures:</span>
                      <div className="mt-2 flex flex-wrap gap-2">
                        {selectedThreat.signatures.map((sig, index) => (
                          <Badge key={index} variant="outline" className="text-xs">
                            {sig}
                          </Badge>
                        ))}
                      </div>
                    </div>

                    <div>
                      <span className="text-muted-foreground text-sm">Behavior Analysis:</span>
                      <div className="mt-2 space-y-2">
                        {selectedThreat.behaviorAnalysis.networkConnections.length > 0 && (
                          <div className="p-2 bg-muted rounded text-xs">
                            <div className="font-medium">Network Connections:</div>
                            {selectedThreat.behaviorAnalysis.networkConnections.map((conn, i) => (
                              <div key={i} className="font-mono">{conn}</div>
                            ))}
                          </div>
                        )}
                        {selectedThreat.behaviorAnalysis.fileModifications.length > 0 && (
                          <div className="p-2 bg-muted rounded text-xs">
                            <div className="font-medium">File Modifications:</div>
                            {selectedThreat.behaviorAnalysis.fileModifications.map((file, i) => (
                              <div key={i} className="font-mono">{file}</div>
                            ))}
                          </div>
                        )}
                      </div>
                    </div>

                    <div>
                      <span className="text-muted-foreground text-sm">Recommendations:</span>
                      <ul className="mt-2 space-y-1">
                        {selectedThreat.recommendations.map((rec, index) => (
                          <li key={index} className="text-sm flex items-start space-x-2">
                            <CheckCircle className="h-3 w-3 text-green-500 mt-0.5 shrink-0" />
                            <span>{rec}</span>
                          </li>
                        ))}
                      </ul>
                    </div>

                    <div className="flex space-x-2">
                      <Button size="sm" variant="destructive">
                        <Trash2 className="h-4 w-4 mr-2" />
                        Remove
                      </Button>
                      <Button size="sm" variant="outline">
                        <Shield className="h-4 w-4 mr-2" />
                        Quarantine
                      </Button>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-center text-muted-foreground py-8">
                  <AlertTriangle className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>Select a threat detection to view detailed analysis</p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>Malware Signatures</CardTitle>
            <CardDescription>
              Signature database for threat detection and pattern matching
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {mockSignatures.map((signature) => (
              <div
                key={signature.id}
                className="flex items-start space-x-3 p-3 rounded-lg border"
              >
                <div className={`${getSeverityColor(signature.severity)} mt-1`}>
                  <FileText className="h-4 w-4" />
                </div>
                <div className="flex-1 space-y-1">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium leading-none">
                      {signature.name}
                    </p>
                    <div className="flex items-center space-x-2">
                      {getSeverityBadge(signature.severity)}
                      <Badge variant="outline">{signature.type}</Badge>
                    </div>
                  </div>
                  <div className="text-sm text-muted-foreground font-mono">
                    {signature.pattern}
                  </div>
                  <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                    <span>Detections: {signature.detectionCount}</span>
                    <span>•</span>
                    <div className="flex items-center space-x-1">
                      <Clock className="h-3 w-3" />
                      <span>Updated: {signature.lastUpdated}</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  )
}