import { useState } from 'react'
import { 
  Network, 
  Shield, 
  Activity,
  Globe,
  Download,
  Upload,
  AlertTriangle,
  CheckCircle,
  XCircle,
  Filter
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

type ConnectionStatus = 'active' | 'blocked' | 'closed'
type Protocol = 'TCP' | 'UDP' | 'ICMP'

interface NetworkConnection {
  id: string
  timestamp: string
  protocol: Protocol
  sourceIp: string
  sourcePort: number
  destIp: string
  destPort: number
  status: ConnectionStatus
  bytesIn: number
  bytesOut: number
  packets: number
  duration: string
  process: string
  pid: number
}

interface DnsQuery {
  id: string
  timestamp: string
  domain: string
  queryType: string
  sourceIp: string
  status: 'allowed' | 'blocked'
  response?: string
}

const mockConnections: NetworkConnection[] = [
  {
    id: '1',
    timestamp: '2024-06-01 15:35:22',
    protocol: 'TCP',
    sourceIp: '192.168.1.100',
    sourcePort: 45678,
    destIp: '93.184.216.34',
    destPort: 443,
    status: 'active',
    bytesIn: 2048,
    bytesOut: 1024,
    packets: 15,
    duration: '2m 15s',
    process: 'firefox',
    pid: 12345
  },
  {
    id: '2',
    timestamp: '2024-06-01 15:34:18',
    protocol: 'TCP',
    sourceIp: '192.168.1.100',
    sourcePort: 54321,
    destIp: '45.123.456.789',
    destPort: 4444,
    status: 'blocked',
    bytesIn: 0,
    bytesOut: 128,
    packets: 3,
    duration: '0s',
    process: 'malware',
    pid: 98765
  },
  {
    id: '3',
    timestamp: '2024-06-01 15:33:45',
    protocol: 'UDP',
    sourceIp: '192.168.1.100',
    sourcePort: 53,
    destIp: '8.8.8.8',
    destPort: 53,
    status: 'closed',
    bytesIn: 512,
    bytesOut: 256,
    packets: 4,
    duration: '1s',
    process: 'systemd-resolved',
    pid: 567
  }
]

const mockDnsQueries: DnsQuery[] = [
  {
    id: '1',
    timestamp: '2024-06-01 15:35:30',
    domain: 'google.com',
    queryType: 'A',
    sourceIp: '192.168.1.100',
    status: 'allowed',
    response: '142.250.191.14'
  },
  {
    id: '2',
    timestamp: '2024-06-01 15:35:15',
    domain: 'malware.example.com',
    queryType: 'A',
    sourceIp: '192.168.1.100',
    status: 'blocked'
  },
  {
    id: '3',
    timestamp: '2024-06-01 15:34:58',
    domain: 'github.com',
    queryType: 'AAAA',
    sourceIp: '192.168.1.100',
    status: 'allowed',
    response: '2606:50c0:8000::154'
  }
]

const getStatusIcon = (status: ConnectionStatus | 'allowed' | 'blocked') => {
  switch (status) {
    case 'active': return <Activity className="h-4 w-4 text-green-500" />
    case 'allowed': return <CheckCircle className="h-4 w-4 text-green-500" />
    case 'blocked': return <XCircle className="h-4 w-4 text-red-500" />
    case 'closed': return <CheckCircle className="h-4 w-4 text-gray-500" />
    default: return <Activity className="h-4 w-4 text-gray-500" />
  }
}

const getStatusBadge = (status: ConnectionStatus | 'allowed' | 'blocked') => {
  switch (status) {
    case 'active': return <Badge variant="default">Active</Badge>
    case 'allowed': return <Badge variant="secondary">Allowed</Badge>
    case 'blocked': return <Badge variant="destructive">Blocked</Badge>
    case 'closed': return <Badge variant="outline">Closed</Badge>
    default: return <Badge variant="outline">Unknown</Badge>
  }
}

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

export function NetworkMonitor() {
  const [selectedConnection, setSelectedConnection] = useState<NetworkConnection | null>(null)
  const [activeTab, setActiveTab] = useState<'connections' | 'dns'>('connections')

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Network Monitor</h2>
          <p className="text-muted-foreground">
            Monitor network connections, DNS queries, and traffic filtering
          </p>
        </div>
        <div className="flex items-center space-x-2">
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
            <CardTitle className="text-sm font-medium">Active Connections</CardTitle>
            <Network className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">47</div>
            <p className="text-xs text-muted-foreground">
              Currently active
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Blocked Connections</CardTitle>
            <XCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">12</div>
            <p className="text-xs text-muted-foreground">
              Security violations
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Data Transfer</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center space-x-2">
              <Download className="h-4 w-4 text-blue-500" />
              <span className="text-sm font-medium">2.3 GB</span>
            </div>
            <div className="flex items-center space-x-2">
              <Upload className="h-4 w-4 text-green-500" />
              <span className="text-sm font-medium">892 MB</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">DNS Queries</CardTitle>
            <Globe className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">1,523</div>
            <p className="text-xs text-muted-foreground">
              23 blocked
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Tabs */}
      <div className="flex space-x-2">
        <Button
          variant={activeTab === 'connections' ? 'default' : 'outline'}
          onClick={() => setActiveTab('connections')}
        >
          <Network className="h-4 w-4 mr-2" />
          Connections
        </Button>
        <Button
          variant={activeTab === 'dns' ? 'default' : 'outline'}
          onClick={() => setActiveTab('dns')}
        >
          <Globe className="h-4 w-4 mr-2" />
          DNS Queries
        </Button>
      </div>

      {/* Content */}
      {activeTab === 'connections' ? (
        <div className="grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Network Connections</CardTitle>
              <CardDescription>
                Real-time network connection monitoring and filtering
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {mockConnections.map((connection) => (
                <div
                  key={connection.id}
                  className="flex items-start space-x-3 p-3 rounded-lg border cursor-pointer hover:bg-accent/50 transition-colors"
                  onClick={() => setSelectedConnection(connection)}
                >
                  <div className="mt-1">
                    {getStatusIcon(connection.status)}
                  </div>
                  <div className="flex-1 space-y-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium leading-none">
                        {connection.sourceIp}:{connection.sourcePort} → {connection.destIp}:{connection.destPort}
                      </p>
                      {getStatusBadge(connection.status)}
                    </div>
                    <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                      <span>{connection.protocol}</span>
                      <span>{connection.process} ({connection.pid})</span>
                      <span>{formatBytes(connection.bytesIn + connection.bytesOut)}</span>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {connection.timestamp} • Duration: {connection.duration}
                    </div>
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>

          {/* Connection Details */}
          <Card>
            <CardHeader>
              <CardTitle>Connection Details</CardTitle>
              <CardDescription>
                {selectedConnection ? 'Detailed information about the selected connection' : 'Select a connection to view details'}
              </CardDescription>
            </CardHeader>
            <CardContent>
              {selectedConnection ? (
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h3 className="text-lg font-semibold">
                      {selectedConnection.sourceIp}:{selectedConnection.sourcePort}
                    </h3>
                    {getStatusBadge(selectedConnection.status)}
                  </div>
                  
                  <div className="space-y-3">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <span className="text-muted-foreground">Protocol:</span>
                        <div>{selectedConnection.protocol}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Duration:</span>
                        <div>{selectedConnection.duration}</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Process:</span>
                        <div>{selectedConnection.process} (PID: {selectedConnection.pid})</div>
                      </div>
                      <div>
                        <span className="text-muted-foreground">Packets:</span>
                        <div>{selectedConnection.packets}</div>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-3 bg-muted rounded-lg">
                        <div className="text-sm text-muted-foreground">Source</div>
                        <div className="font-mono">{selectedConnection.sourceIp}:{selectedConnection.sourcePort}</div>
                      </div>
                      <div className="p-3 bg-muted rounded-lg">
                        <div className="text-sm text-muted-foreground">Destination</div>
                        <div className="font-mono">{selectedConnection.destIp}:{selectedConnection.destPort}</div>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 gap-4">
                      <div className="p-3 bg-muted rounded-lg">
                        <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                          <Download className="h-4 w-4" />
                          <span>Bytes In</span>
                        </div>
                        <div className="text-lg font-semibold">{formatBytes(selectedConnection.bytesIn)}</div>
                      </div>
                      <div className="p-3 bg-muted rounded-lg">
                        <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                          <Upload className="h-4 w-4" />
                          <span>Bytes Out</span>
                        </div>
                        <div className="text-lg font-semibold">{formatBytes(selectedConnection.bytesOut)}</div>
                      </div>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="text-center text-muted-foreground py-8">
                  <Network className="h-12 w-12 mx-auto mb-4 opacity-50" />
                  <p>Select a network connection to view detailed information</p>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      ) : (
        <Card>
          <CardHeader>
            <CardTitle>DNS Queries</CardTitle>
            <CardDescription>
              DNS query monitoring and domain filtering
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {mockDnsQueries.map((query) => (
              <div
                key={query.id}
                className="flex items-start space-x-3 p-3 rounded-lg border"
              >
                <div className="mt-1">
                  {getStatusIcon(query.status)}
                </div>
                <div className="flex-1 space-y-1">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium leading-none">
                      {query.domain}
                    </p>
                    {getStatusBadge(query.status)}
                  </div>
                  <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                    <span>Type: {query.queryType}</span>
                    <span>From: {query.sourceIp}</span>
                    {query.response && <span>Response: {query.response}</span>}
                  </div>
                  <div className="text-xs text-muted-foreground">
                    {query.timestamp}
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