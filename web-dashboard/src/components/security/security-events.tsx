import { useState } from 'react'
import { 
  Shield, 
  AlertTriangle, 
  FileText, 
  Eye,
  CheckCircle,
  XCircle,
  Filter,
  Download
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

type EventSeverity = 'low' | 'medium' | 'high' | 'critical'
type EventType = 'file_access' | 'process_start' | 'privilege_escalation' | 'suspicious_binary' | 'crypto_mining' | 'network_connection'

interface SecurityEvent {
  id: string
  timestamp: string
  type: EventType
  severity: EventSeverity
  title: string
  description: string
  source: string
  action: 'allowed' | 'blocked' | 'logged'
  details: Record<string, any>
}

const mockEvents: SecurityEvent[] = [
  {
    id: '1',
    timestamp: '2024-06-01 15:30:45',
    type: 'suspicious_binary',
    severity: 'high',
    title: 'Suspicious binary execution detected',
    description: 'Unknown binary attempted to execute with elevated privileges',
    source: '/tmp/malware_sample',
    action: 'blocked',
    details: {
      pid: 12345,
      user: 'testuser',
      hash: 'a1b2c3d4e5f6...',
      parent_process: '/bin/bash'
    }
  },
  {
    id: '2',
    timestamp: '2024-06-01 15:28:12',
    type: 'privilege_escalation',
    severity: 'medium',
    title: 'Privilege escalation attempt',
    description: 'Process attempted to gain root privileges',
    source: '/usr/bin/sudo',
    action: 'logged',
    details: {
      pid: 11234,
      user: 'user1',
      target_user: 'root',
      command: 'passwd root'
    }
  },
  {
    id: '3',
    timestamp: '2024-06-01 15:25:33',
    type: 'file_access',
    severity: 'low',
    title: 'Sensitive file access',
    description: 'Process accessed system configuration file',
    source: '/etc/shadow',
    action: 'allowed',
    details: {
      pid: 9876,
      user: 'admin',
      access_type: 'read',
      process: '/usr/bin/cat'
    }
  },
  {
    id: '4',
    timestamp: '2024-06-01 15:22:18',
    type: 'crypto_mining',
    severity: 'critical',
    title: 'Cryptocurrency mining detected',
    description: 'High CPU usage pattern consistent with crypto mining',
    source: '/tmp/xmrig',
    action: 'blocked',
    details: {
      pid: 15678,
      user: 'nobody',
      cpu_usage: '95%',
      network_connections: ['pool.supportxmr.com:3333']
    }
  },
  {
    id: '5',
    timestamp: '2024-06-01 15:20:05',
    type: 'network_connection',
    severity: 'medium',
    title: 'Suspicious network connection',
    description: 'Connection to known malicious IP address',
    source: '192.168.1.100',
    action: 'blocked',
    details: {
      destination: '45.123.456.789',
      port: 4444,
      protocol: 'TCP',
      process: '/tmp/backdoor'
    }
  }
]

const getSeverityColor = (severity: EventSeverity) => {
  switch (severity) {
    case 'low': return 'text-blue-500'
    case 'medium': return 'text-yellow-500'
    case 'high': return 'text-orange-500'
    case 'critical': return 'text-red-500'
    default: return 'text-gray-500'
  }
}

const getSeverityBadge = (severity: EventSeverity) => {
  switch (severity) {
    case 'low': return <Badge variant="secondary">Low</Badge>
    case 'medium': return <Badge variant="outline">Medium</Badge>
    case 'high': return <Badge variant="destructive">High</Badge>
    case 'critical': return <Badge variant="destructive" className="bg-red-600">Critical</Badge>
    default: return <Badge variant="secondary">Unknown</Badge>
  }
}

const getActionIcon = (action: string) => {
  switch (action) {
    case 'allowed': return <CheckCircle className="h-4 w-4 text-green-500" />
    case 'blocked': return <XCircle className="h-4 w-4 text-red-500" />
    case 'logged': return <Eye className="h-4 w-4 text-blue-500" />
    default: return <FileText className="h-4 w-4 text-gray-500" />
  }
}

const getTypeIcon = (type: EventType) => {
  switch (type) {
    case 'file_access': return <FileText className="h-4 w-4" />
    case 'process_start': return <Shield className="h-4 w-4" />
    case 'privilege_escalation': return <AlertTriangle className="h-4 w-4" />
    case 'suspicious_binary': return <Shield className="h-4 w-4" />
    case 'crypto_mining': return <AlertTriangle className="h-4 w-4" />
    case 'network_connection': return <Shield className="h-4 w-4" />
    default: return <FileText className="h-4 w-4" />
  }
}

export function SecurityEvents() {
  const [selectedEvent, setSelectedEvent] = useState<SecurityEvent | null>(null)
  const [filter, setFilter] = useState<EventSeverity | 'all'>('all')

  const filteredEvents = filter === 'all' 
    ? mockEvents 
    : mockEvents.filter(event => event.severity === filter)

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Security Events</h2>
          <p className="text-muted-foreground">
            Monitor file system access, process execution, and security policy violations
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
            <CardTitle className="text-sm font-medium">Total Events</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">1,247</div>
            <p className="text-xs text-muted-foreground">
              +12% from yesterday
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Blocked Events</CardTitle>
            <XCircle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">23</div>
            <p className="text-xs text-muted-foreground">
              Security violations
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Critical Alerts</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-orange-500">3</div>
            <p className="text-xs text-muted-foreground">
              Require immediate attention
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Policy Violations</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">15</div>
            <p className="text-xs text-muted-foreground">
              Policy enforcement active
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Event List */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Recent Events</CardTitle>
            <CardDescription>
              Latest security events from file system and process monitoring
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {filteredEvents.map((event) => (
              <div
                key={event.id}
                className="flex items-start space-x-3 p-3 rounded-lg border cursor-pointer hover:bg-accent/50 transition-colors"
                onClick={() => setSelectedEvent(event)}
              >
                <div className={`${getSeverityColor(event.severity)} mt-1`}>
                  {getTypeIcon(event.type)}
                </div>
                <div className="flex-1 space-y-1">
                  <div className="flex items-center justify-between">
                    <p className="text-sm font-medium leading-none">
                      {event.title}
                    </p>
                    <div className="flex items-center space-x-2">
                      {getActionIcon(event.action)}
                      {getSeverityBadge(event.severity)}
                    </div>
                  </div>
                  <p className="text-sm text-muted-foreground">
                    {event.description}
                  </p>
                  <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                    <span>{event.timestamp}</span>
                    <span>•</span>
                    <span>{event.source}</span>
                  </div>
                </div>
              </div>
            ))}
          </CardContent>
        </Card>

        {/* Event Details */}
        <Card>
          <CardHeader>
            <CardTitle>Event Details</CardTitle>
            <CardDescription>
              {selectedEvent ? 'Detailed information about the selected event' : 'Select an event to view details'}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {selectedEvent ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">{selectedEvent.title}</h3>
                  {getSeverityBadge(selectedEvent.severity)}
                </div>
                
                <div className="space-y-2">
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>
                      <span className="text-muted-foreground">Timestamp:</span>
                      <div>{selectedEvent.timestamp}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Action:</span>
                      <div className="flex items-center space-x-2">
                        {getActionIcon(selectedEvent.action)}
                        <span className="capitalize">{selectedEvent.action}</span>
                      </div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Type:</span>
                      <div className="capitalize">{selectedEvent.type.replace('_', ' ')}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Source:</span>
                      <div className="font-mono text-xs">{selectedEvent.source}</div>
                    </div>
                  </div>
                </div>

                <div>
                  <span className="text-muted-foreground text-sm">Description:</span>
                  <p className="mt-1">{selectedEvent.description}</p>
                </div>

                <div>
                  <span className="text-muted-foreground text-sm">Additional Details:</span>
                  <div className="mt-2 p-3 bg-muted rounded-lg">
                    <pre className="text-xs font-mono whitespace-pre-wrap">
                      {JSON.stringify(selectedEvent.details, null, 2)}
                    </pre>
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-center text-muted-foreground py-8">
                <Shield className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>Select a security event from the list to view detailed information</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}