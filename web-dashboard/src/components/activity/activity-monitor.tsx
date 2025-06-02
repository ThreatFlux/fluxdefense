import { useState, useEffect } from 'react'
import { 
  Activity, 
  Cpu, 
  HardDrive,
  MemoryStick,
  Zap,
  Users,
  Clock,
  TrendingUp,
  AlertTriangle,
  RefreshCw
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"

interface SystemMetrics {
  cpu: {
    usage: number
    cores: number
    loadAverage: number[]
  }
  memory: {
    total: number
    used: number
    available: number
    percent: number
  }
  disk: {
    total: number
    used: number
    available: number
    percent: number
  }
  network: {
    bytesIn: number
    bytesOut: number
    packetsIn: number
    packetsOut: number
  }
}

interface Process {
  pid: number
  name: string
  user: string
  cpu: number
  memory: number
  status: 'running' | 'sleeping' | 'stopped' | 'zombie'
  startTime: string
  command: string
}



const formatBytes = (bytes: number): string => {
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  if (bytes === 0) return '0 B'
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return Math.round((bytes / Math.pow(1024, i)) * 100) / 100 + ' ' + sizes[i]
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'running': return 'text-green-500'
    case 'sleeping': return 'text-blue-500'
    case 'stopped': return 'text-red-500'
    case 'zombie': return 'text-orange-500'
    default: return 'text-gray-500'
  }
}

const getStatusBadge = (status: string) => {
  switch (status) {
    case 'running': return <Badge variant="default">Running</Badge>
    case 'sleeping': return <Badge variant="secondary">Sleeping</Badge>
    case 'stopped': return <Badge variant="destructive">Stopped</Badge>
    case 'zombie': return <Badge variant="outline" className="border-orange-500 text-orange-500">Zombie</Badge>
    default: return <Badge variant="outline">Unknown</Badge>
  }
}

const getCpuColor = (usage: number) => {
  if (usage > 80) return 'text-red-500'
  if (usage > 60) return 'text-orange-500'
  if (usage > 40) return 'text-yellow-500'
  return 'text-green-500'
}

const getMemoryColor = (percent: number) => {
  if (percent > 85) return 'text-red-500'
  if (percent > 70) return 'text-orange-500'
  if (percent > 50) return 'text-yellow-500'
  return 'text-green-500'
}

export function ActivityMonitor() {
  const [metrics, setMetrics] = useState<SystemMetrics | null>(null)
  const [processes, setProcesses] = useState<Process[]>([])
  const [selectedProcess, setSelectedProcess] = useState<Process | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(true)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Fetch system metrics and processes from API
  const fetchSystemData = async () => {
    try {
      setLoading(true)
      setError(null)
      
      const [metricsRes, processesRes] = await Promise.all([
        fetch('/api/system/metrics').then(res => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`);
          return res.json();
        }),
        fetch('/api/system/processes').then(res => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`);
          return res.json();
        })
      ])
      
      setMetrics(metricsRes)
      setProcesses(processesRes)
      setLoading(false)
    } catch (err) {
      console.error('Failed to fetch system data:', err)
      setError(err instanceof Error ? err.message : 'Failed to fetch system data')
      setLoading(false)
    }
  }

  // Initial load and auto-refresh
  useEffect(() => {
    fetchSystemData()
    
    if (!autoRefresh) return

    const interval = setInterval(fetchSystemData, 5000)
    return () => clearInterval(interval)
  }, [autoRefresh])

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-8 w-8 animate-spin" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-3xl font-bold tracking-tight">Activity Monitor</h2>
            <p className="text-muted-foreground">
              Real-time system resource monitoring and process management
            </p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchSystemData}
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            Retry
          </Button>
        </div>
        <Card>
          <CardContent className="text-center py-8">
            <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-red-500" />
            <h3 className="text-lg font-semibold mb-2">Failed to Load System Data</h3>
            <p className="text-muted-foreground mb-4">{error}</p>
            <p className="text-sm text-muted-foreground">Please ensure the FluxDefense API server is running and accessible.</p>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Activity Monitor</h2>
          <p className="text-muted-foreground">
            Real-time system resource monitoring and process management
          </p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant={autoRefresh ? "default" : "outline"}
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            <RefreshCw className={`h-4 w-4 mr-2 ${autoRefresh ? 'animate-spin' : ''}`} />
            Auto Refresh
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchSystemData}
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh Now
          </Button>
        </div>
      </div>

      {/* System Metrics */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getCpuColor(metrics?.cpu.usage || 0)}`}>
              {metrics?.cpu.usage.toFixed(1) || 0}%
            </div>
            <p className="text-xs text-muted-foreground">
              {metrics?.cpu.cores || 0} cores • Load: {metrics?.cpu.loadAverage[0]?.toFixed(2) || '0.00'}
            </p>
            <div className="mt-2 w-full bg-secondary rounded-full h-2">
              <div 
                className={`h-2 rounded-full transition-all duration-300 ${
                  (metrics?.cpu.usage || 0) > 80 ? 'bg-red-500' : 
                  (metrics?.cpu.usage || 0) > 60 ? 'bg-orange-500' : 
                  (metrics?.cpu.usage || 0) > 40 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${metrics?.cpu.usage || 0}%` }}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory</CardTitle>
            <MemoryStick className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className={`text-2xl font-bold ${getMemoryColor(metrics?.memory.percent || 0)}`}>
              {metrics?.memory.percent.toFixed(1) || 0}%
            </div>
            <p className="text-xs text-muted-foreground">
              {formatBytes(metrics?.memory.used || 0)} of {formatBytes(metrics?.memory.total || 0)}
            </p>
            <div className="mt-2 w-full bg-secondary rounded-full h-2">
              <div 
                className={`h-2 rounded-full transition-all duration-300 ${
                  (metrics?.memory.percent || 0) > 85 ? 'bg-red-500' : 
                  (metrics?.memory.percent || 0) > 70 ? 'bg-orange-500' : 
                  (metrics?.memory.percent || 0) > 50 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${metrics?.memory.percent || 0}%` }}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Disk Usage</CardTitle>
            <HardDrive className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {metrics?.disk.percent.toFixed(1) || 0}%
            </div>
            <p className="text-xs text-muted-foreground">
              {formatBytes(metrics?.disk.used || 0)} of {formatBytes(metrics?.disk.total || 0)}
            </p>
            <div className="mt-2 w-full bg-secondary rounded-full h-2">
              <div 
                className="h-2 rounded-full bg-blue-500 transition-all duration-300"
                style={{ width: `${metrics?.disk.percent || 0}%` }}
              />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Network I/O</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="space-y-1">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">In:</span>
                <span>{formatBytes(metrics?.network.bytesIn || 0)}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Out:</span>
                <span>{formatBytes(metrics?.network.bytesOut || 0)}</span>
              </div>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              {(metrics?.network.packetsIn || 0).toLocaleString()} packets in
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Process List and Details */}
      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Running Processes</CardTitle>
            <CardDescription>
              Active system processes sorted by resource usage
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {processes.length === 0 ? (
              <div className="text-center text-muted-foreground py-8">
                <Users className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>No process data available</p>
                <p className="text-xs">Process information will appear when data is available</p>
              </div>
            ) : (
              processes
                .sort((a, b) => b.cpu - a.cpu)
                .map((process) => (
                <div
                  key={process.pid}
                  className="flex items-center space-x-3 p-3 rounded-lg border cursor-pointer hover:bg-accent/50 transition-colors"
                  onClick={() => setSelectedProcess(process)}
                >
                  <div className={`${getStatusColor(process.status)}`}>
                    <Activity className="h-4 w-4" />
                  </div>
                  <div className="flex-1 space-y-1">
                    <div className="flex items-center justify-between">
                      <p className="text-sm font-medium leading-none">
                        {process.name} ({process.pid})
                      </p>
                      {getStatusBadge(process.status)}
                    </div>
                    <div className="flex items-center space-x-4 text-xs text-muted-foreground">
                      <span>CPU: {process.cpu.toFixed(1)}%</span>
                      <span>MEM: {process.memory.toFixed(1)}%</span>
                      <span>User: {process.user}</span>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      Started: {process.startTime}
                    </div>
                  </div>
                </div>
                ))
            )}
          </CardContent>
        </Card>

        {/* Process Details */}
        <Card>
          <CardHeader>
            <CardTitle>Process Details</CardTitle>
            <CardDescription>
              {selectedProcess ? 'Detailed information about the selected process' : 'Select a process to view details'}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {selectedProcess ? (
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">
                    {selectedProcess.name}
                  </h3>
                  {getStatusBadge(selectedProcess.status)}
                </div>
                
                <div className="space-y-3">
                  <div className="grid grid-cols-2 gap-4 text-sm">
                    <div>
                      <span className="text-muted-foreground">Process ID:</span>
                      <div className="font-mono">{selectedProcess.pid}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">User:</span>
                      <div>{selectedProcess.user}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Start Time:</span>
                      <div>{selectedProcess.startTime}</div>
                    </div>
                    <div>
                      <span className="text-muted-foreground">Status:</span>
                      <div className="capitalize">{selectedProcess.status}</div>
                    </div>
                  </div>

                  <div className="grid grid-cols-2 gap-4">
                    <div className="p-3 bg-muted rounded-lg">
                      <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                        <Cpu className="h-4 w-4" />
                        <span>CPU Usage</span>
                      </div>
                      <div className={`text-lg font-semibold ${getCpuColor(selectedProcess.cpu)}`}>
                        {selectedProcess.cpu.toFixed(1)}%
                      </div>
                    </div>
                    <div className="p-3 bg-muted rounded-lg">
                      <div className="flex items-center space-x-2 text-sm text-muted-foreground">
                        <MemoryStick className="h-4 w-4" />
                        <span>Memory Usage</span>
                      </div>
                      <div className="text-lg font-semibold">
                        {selectedProcess.memory.toFixed(1)}%
                      </div>
                    </div>
                  </div>

                  <div>
                    <span className="text-muted-foreground text-sm">Command Line:</span>
                    <div className="mt-2 p-3 bg-muted rounded-lg">
                      <code className="text-xs font-mono break-all">
                        {selectedProcess.command}
                      </code>
                    </div>
                  </div>

                  {selectedProcess.cpu > 80 && (
                    <div className="flex items-center space-x-2 p-3 bg-red-50 dark:bg-red-950/20 rounded-lg border border-red-200 dark:border-red-800">
                      <AlertTriangle className="h-4 w-4 text-red-500" />
                      <span className="text-sm text-red-700 dark:text-red-300">
                        High CPU usage detected
                      </span>
                    </div>
                  )}
                </div>
              </div>
            ) : (
              <div className="text-center text-muted-foreground py-8">
                <Users className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>Select a process to view detailed information</p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      {/* System Load Average */}
      <Card>
        <CardHeader>
          <CardTitle>System Load Average</CardTitle>
          <CardDescription>
            1-minute, 5-minute, and 15-minute load averages
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-4">
            {['1 min', '5 min', '15 min'].map((label, index) => (
              <div key={label} className="text-center p-4 bg-muted rounded-lg">
                <div className="text-sm text-muted-foreground">{label}</div>
                <div className="text-2xl font-bold">
                  {metrics?.cpu.loadAverage[index]?.toFixed(2) || '0.00'}
                </div>
                <div className="text-xs text-muted-foreground">
                  {(metrics?.cpu.loadAverage[index] || 0) > (metrics?.cpu.cores || 1) ? 'High' : 'Normal'}
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}