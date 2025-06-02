import { useState, useEffect, useRef } from 'react'
import { 
  Eye, 
  Play,
  Pause,
  RotateCcw,
  Download,
  Filter,
  AlertTriangle,
  Shield,
  Network,
  Activity,
  FileText,
  Clock,
  Zap
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

type EventType = 'security' | 'network' | 'system' | 'file_access' | 'process' | 'threat'
type EventSeverity = 'low' | 'medium' | 'high' | 'critical'

interface LiveEvent {
  id: string
  timestamp: string
  type: EventType
  severity: EventSeverity
  title: string
  description: string
  source: string
  details: Record<string, any>
}

const mockEventSources = [
  'fanotify_monitor',
  'network_filter', 
  'enhanced_monitor',
  'threat_detector',
  'process_monitor',
  'iptables_manager'
]

const eventTemplates = [
  {
    type: 'security' as EventType,
    severity: 'high' as EventSeverity,
    titles: [
      'Suspicious binary execution detected',
      'Privilege escalation attempt blocked',
      'Unauthorized file access prevented',
      'Malicious process terminated'
    ],
    descriptions: [
      'Unknown binary with suspicious behavior patterns',
      'Process attempted to gain elevated privileges',
      'Access to sensitive system file blocked',
      'Process exhibiting malware characteristics'
    ]
  },
  {
    type: 'network' as EventType,
    severity: 'medium' as EventSeverity,
    titles: [
      'DNS query to suspicious domain',
      'Outbound connection blocked',
      'High traffic volume detected',
      'Network anomaly identified'
    ],
    descriptions: [
      'Query to domain in threat intelligence blacklist',
      'Connection attempt to known malicious IP',
      'Unusual network traffic pattern observed',
      'Abnormal network behavior detected'
    ]
  },
  {
    type: 'file_access' as EventType,
    severity: 'low' as EventSeverity,
    titles: [
      'System file accessed',
      'Configuration file modified',
      'Log file rotation completed',
      'Temporary file created'
    ],
    descriptions: [
      'Normal system file access logged',
      'Configuration change detected',
      'Routine log maintenance operation',
      'Temporary file created by application'
    ]
  },
  {
    type: 'threat' as EventType,
    severity: 'critical' as EventSeverity,
    titles: [
      'Cryptocurrency miner detected',
      'Ransomware activity blocked',
      'Trojan communication intercepted',
      'Rootkit installation attempt'
    ],
    descriptions: [
      'High CPU usage pattern consistent with mining',
      'File encryption attempt prevented',
      'Command and control communication blocked',
      'System-level persistence mechanism detected'
    ]
  }
]

const generateRandomEvent = (): LiveEvent => {
  const template = eventTemplates[Math.floor(Math.random() * eventTemplates.length)]
  const title = template.titles[Math.floor(Math.random() * template.titles.length)]
  const description = template.descriptions[Math.floor(Math.random() * template.descriptions.length)]
  const source = mockEventSources[Math.floor(Math.random() * mockEventSources.length)]
  
  return {
    id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
    timestamp: new Date().toISOString().replace('T', ' ').substr(0, 23),
    type: template.type,
    severity: template.severity,
    title,
    description,
    source,
    details: {
      process_id: Math.floor(Math.random() * 65536),
      user: ['root', 'admin', 'user', 'www-data'][Math.floor(Math.random() * 4)],
      file_path: ['/tmp/suspicious', '/usr/bin/malware', '/etc/passwd', '/var/log/auth.log'][Math.floor(Math.random() * 4)],
      ip_address: `192.168.1.${Math.floor(Math.random() * 255)}`,
      port: Math.floor(Math.random() * 65536)
    }
  }
}

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

const getTypeIcon = (type: EventType) => {
  switch (type) {
    case 'security': return <Shield className="h-4 w-4" />
    case 'network': return <Network className="h-4 w-4" />
    case 'system': return <Activity className="h-4 w-4" />
    case 'file_access': return <FileText className="h-4 w-4" />
    case 'process': return <Zap className="h-4 w-4" />
    case 'threat': return <AlertTriangle className="h-4 w-4" />
    default: return <Eye className="h-4 w-4" />
  }
}

const getTypeBadge = (type: EventType) => {
  const colors = {
    security: 'border-red-500 text-red-700',
    network: 'border-blue-500 text-blue-700', 
    system: 'border-green-500 text-green-700',
    file_access: 'border-purple-500 text-purple-700',
    process: 'border-orange-500 text-orange-700',
    threat: 'border-red-600 text-red-800'
  }
  
  return (
    <Badge variant="outline" className={colors[type]}>
      {type.replace('_', ' ').toUpperCase()}
    </Badge>
  )
}

export function LiveMonitor() {
  const [events, setEvents] = useState<LiveEvent[]>([])
  const [isMonitoring, setIsMonitoring] = useState(true)
  const [filterType, setFilterType] = useState<EventType | 'all'>('all')
  const [filterSeverity, setFilterSeverity] = useState<EventSeverity | 'all'>('all')
  const [selectedEvent, setSelectedEvent] = useState<LiveEvent | null>(null)
  const [autoScroll, setAutoScroll] = useState(true)
  const eventsEndRef = useRef<HTMLDivElement>(null)

  // Generate new events periodically
  useEffect(() => {
    if (!isMonitoring) return

    const interval = setInterval(() => {
      const newEvent = generateRandomEvent()
      setEvents(prev => [newEvent, ...prev.slice(0, 99)]) // Keep only last 100 events
    }, Math.random() * 3000 + 1000) // Random interval between 1-4 seconds

    return () => clearInterval(interval)
  }, [isMonitoring])

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (autoScroll && eventsEndRef.current) {
      eventsEndRef.current.scrollIntoView({ behavior: 'smooth' })
    }
  }, [events, autoScroll])

  const filteredEvents = events.filter(event => {
    const matchesType = filterType === 'all' || event.type === filterType
    const matchesSeverity = filterSeverity === 'all' || event.severity === filterSeverity
    return matchesType && matchesSeverity
  })

  const clearEvents = () => {
    setEvents([])
    setSelectedEvent(null)
  }

  const exportEvents = () => {
    const dataStr = JSON.stringify(events, null, 2)
    const dataBlob = new Blob([dataStr], { type: 'application/json' })
    const url = URL.createObjectURL(dataBlob)
    const link = document.createElement('a')
    link.href = url
    link.download = `fluxdefense-live-events-${new Date().toISOString().split('T')[0]}.json`
    link.click()
  }

  const eventCounts = {
    total: events.length,
    critical: events.filter(e => e.severity === 'critical').length,
    high: events.filter(e => e.severity === 'high').length,
    security: events.filter(e => e.type === 'security').length,
    threats: events.filter(e => e.type === 'threat').length
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Live Monitor</h2>
          <p className="text-muted-foreground">
            Real-time security event streaming and analysis
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant={isMonitoring ? "destructive" : "default"}
            size="sm"
            onClick={() => setIsMonitoring(!isMonitoring)}
          >
            {isMonitoring ? <Pause className="h-4 w-4 mr-2" /> : <Play className="h-4 w-4 mr-2" />}
            {isMonitoring ? 'Pause' : 'Resume'}
          </Button>
          <Button variant="outline" size="sm" onClick={clearEvents}>
            <RotateCcw className="h-4 w-4 mr-2" />
            Clear
          </Button>
          <Button variant="outline" size="sm" onClick={exportEvents}>
            <Download className="h-4 w-4 mr-2" />
            Export
          </Button>
        </div>
      </div>

      {/* Status Indicators */}
      <div className="grid gap-4 md:grid-cols-5">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Stream Status</CardTitle>
            <Eye className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${isMonitoring ? 'text-green-500' : 'text-red-500'}`}>
              {isMonitoring ? 'LIVE' : 'PAUSED'}
            </div>
            <p className="text-xs text-muted-foreground">
              Event monitoring
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Events Captured</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{eventCounts.total}</div>
            <p className="text-xs text-muted-foreground">
              In current session
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Critical Events</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">{eventCounts.critical}</div>
            <p className="text-xs text-muted-foreground">
              Immediate attention
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Security Events</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-orange-500">{eventCounts.security}</div>
            <p className="text-xs text-muted-foreground">
              Security violations
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Threats Detected</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">{eventCounts.threats}</div>
            <p className="text-xs text-muted-foreground">
              Active threats
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <Card>
        <CardHeader>
          <CardTitle>Live Stream Filters</CardTitle>
          <CardDescription>
            Filter real-time events by type and severity
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-4">
            <div className="flex items-center space-x-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <select
                value={filterType}
                onChange={(e) => setFilterType(e.target.value as EventType | 'all')}
                className="px-3 py-2 border border-input rounded-md bg-background"
              >
                <option value="all">All Types</option>
                <option value="security">Security</option>
                <option value="network">Network</option>
                <option value="system">System</option>
                <option value="file_access">File Access</option>
                <option value="process">Process</option>
                <option value="threat">Threat</option>
              </select>
            </div>

            <select
              value={filterSeverity}
              onChange={(e) => setFilterSeverity(e.target.value as EventSeverity | 'all')}
              className="px-3 py-2 border border-input rounded-md bg-background"
            >
              <option value="all">All Severities</option>
              <option value="critical">Critical</option>
              <option value="high">High</option>
              <option value="medium">Medium</option>
              <option value="low">Low</option>
            </select>

            <Button
              variant={autoScroll ? "default" : "outline"}
              size="sm"
              onClick={() => setAutoScroll(!autoScroll)}
            >
              Auto Scroll
            </Button>

            <div className="text-sm text-muted-foreground flex items-center">
              Showing {filteredEvents.length} of {events.length} events
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Event Stream */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card className="h-[600px] flex flex-col">
          <CardHeader className="pb-3">
            <CardTitle>Live Event Stream</CardTitle>
            <CardDescription>
              Real-time security events as they occur
            </CardDescription>
          </CardHeader>
          <CardContent className="flex-1 overflow-auto space-y-2">
            {filteredEvents.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                <Eye className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>No events to display</p>
                <p className="text-xs">Events will appear here as they are detected</p>
              </div>
            ) : (
              <>
                {filteredEvents.map((event) => (
                  <div
                    key={event.id}
                    className="flex items-start space-x-3 p-3 rounded-lg border cursor-pointer hover:bg-accent/50 transition-colors animate-in slide-in-from-top duration-300"
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
                          {getSeverityBadge(event.severity)}
                        </div>
                      </div>
                      <p className="text-sm text-muted-foreground">
                        {event.description}
                      </p>
                      <div className="flex items-center space-x-2 text-xs text-muted-foreground">
                        <Clock className="h-3 w-3" />
                        <span>{event.timestamp}</span>
                        <span>•</span>
                        <span>{event.source}</span>
                      </div>
                    </div>
                  </div>
                ))}
                <div ref={eventsEndRef} />
              </>
            )}
          </CardContent>
        </Card>

        {/* Event Details */}
        <Card className="h-[600px] flex flex-col">
          <CardHeader className="pb-3">
            <CardTitle>Event Details</CardTitle>
            <CardDescription>
              {selectedEvent ? 'Detailed information about the selected event' : 'Select an event to view details'}
            </CardDescription>
          </CardHeader>
          <CardContent className="flex-1 overflow-auto">
            {selectedEvent ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">{selectedEvent.title}</h3>
                  <div className="flex items-center space-x-2">
                    {getTypeBadge(selectedEvent.type)}
                    {getSeverityBadge(selectedEvent.severity)}
                  </div>
                </div>
                
                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-2 text-sm">
                    <div>
                      <span className="text-muted-foreground">Timestamp:</span>
                      <div>{selectedEvent.timestamp}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Source:</span>
                      <div>{selectedEvent.source}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Event Type:</span>
                      <div className="capitalize">{selectedEvent.type.replace('_', ' ')}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Severity:</span>
                      <div className="capitalize">{selectedEvent.severity}</div>
                    </div>
                  </div>
                </div>

                <div>
                  <span className="text-muted-foreground text-sm">Description:</span>
                  <p className="mt-1">{selectedEvent.description}</p>
                </div>

                <div>
                  <span className="text-muted-foreground text-sm">Event Details:</span>
                  <div className="mt-2 p-3 bg-muted rounded-lg">
                    <pre className="text-xs font-mono whitespace-pre-wrap overflow-x-auto">
                      {JSON.stringify(selectedEvent.details, null, 2)}
                    </pre>
                  </div>
                </div>
              </div>
            ) : (
              <div className="text-center text-muted-foreground py-8">
                <Eye className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>Select an event from the stream to view detailed information</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  )
}