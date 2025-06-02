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

const mockMetrics: SystemMetrics = {
  cpu: {
    usage: 23.4,
    cores: 8,
    loadAverage: [0.8, 1.2, 1.5]
  },
  memory: {
    total: 16 * 1024 * 1024 * 1024, // 16GB
    used: 8.2 * 1024 * 1024 * 1024, // 8.2GB
    available: 7.8 * 1024 * 1024 * 1024, // 7.8GB
    percent: 51.3
  },
  disk: {
    total: 512 * 1024 * 1024 * 1024, // 512GB
    used: 256 * 1024 * 1024 * 1024, // 256GB
    available: 256 * 1024 * 1024 * 1024, // 256GB
    percent: 50.0
  },
  network: {
    bytesIn: 1024 * 1024 * 1024, // 1GB
    bytesOut: 512 * 1024 * 1024, // 512MB
    packetsIn: 1500000,
    packetsOut: 950000
  }
}

const mockProcesses: Process[] = [
  {
    pid: 1234,
    name: 'firefox',
    user: 'user',
    cpu: 15.2,
    memory: 12.5,
    status: 'running',
    startTime: '09:30:15',
    command: '/usr/bin/firefox --no-sandbox'
  },
  {
    pid: 5678,
    name: 'code',
    user: 'user',
    cpu: 8.7,
    memory: 8.9,
    status: 'running',
    startTime: '08:45:32',
    command: '/usr/bin/code --unity-launch'
  },
  {
    pid: 9012,
    name: 'systemd',
    user: 'root',
    cpu: 0.1,
    memory: 0.2,
    status: 'sleeping',
    startTime: '00:00:01',
    command: '/sbin/systemd --switched-root'
  },
  {
    pid: 3456,
    name: 'gnome-shell',
    user: 'user',
    cpu: 3.2,
    memory: 4.1,
    status: 'running',
    startTime: '08:30:00',
    command: '/usr/bin/gnome-shell'
  },
  {
    pid: 7890,
    name: 'docker',
    user: 'root',
    cpu: 1.8,
    memory: 2.3,
    status: 'running',
    startTime: '08:00:12',
    command: '/usr/bin/dockerd'
  }
]

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
  const [metrics, setMetrics] = useState<SystemMetrics>(mockMetrics)
  const [processes, setProcesses] = useState<Process[]>(mockProcesses)
  const [selectedProcess, setSelectedProcess] = useState<Process | null>(null)
  const [autoRefresh, setAutoRefresh] = useState(true)

  // Simulate real-time updates
  useEffect(() => {
    if (!autoRefresh) return

    const interval = setInterval(() => {
      setMetrics(prev => ({
        ...prev,
        cpu: {
          ...prev.cpu,
          usage: Math.max(0, Math.min(100, prev.cpu.usage + (Math.random() - 0.5) * 10)),
          loadAverage: prev.cpu.loadAverage.map(load => Math.max(0, load + (Math.random() - 0.5) * 0.2))
        },
        memory: {
          ...prev.memory,
          percent: Math.max(0, Math.min(100, prev.memory.percent + (Math.random() - 0.5) * 5))
        }
      }))

      setProcesses(prev => prev.map(proc => ({
        ...proc,
        cpu: Math.max(0, Math.min(100, proc.cpu + (Math.random() - 0.5) * 5)),
        memory: Math.max(0, Math.min(100, proc.memory + (Math.random() - 0.5) * 2))
      })))
    }, 2000)

    return () => clearInterval(interval)
  }, [autoRefresh])

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
            <div className={`text-2xl font-bold ${getCpuColor(metrics.cpu.usage)}`}>
              {metrics.cpu.usage.toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              {metrics.cpu.cores} cores • Load: {metrics.cpu.loadAverage[0].toFixed(2)}
            </p>
            <div className="mt-2 w-full bg-secondary rounded-full h-2">
              <div 
                className={`h-2 rounded-full transition-all duration-300 ${
                  metrics.cpu.usage > 80 ? 'bg-red-500' : 
                  metrics.cpu.usage > 60 ? 'bg-orange-500' : 
                  metrics.cpu.usage > 40 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${metrics.cpu.usage}%` }}
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
            <div className={`text-2xl font-bold ${getMemoryColor(metrics.memory.percent)}`}>
              {metrics.memory.percent.toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              {formatBytes(metrics.memory.used)} of {formatBytes(metrics.memory.total)}
            </p>
            <div className="mt-2 w-full bg-secondary rounded-full h-2">
              <div 
                className={`h-2 rounded-full transition-all duration-300 ${
                  metrics.memory.percent > 85 ? 'bg-red-500' : 
                  metrics.memory.percent > 70 ? 'bg-orange-500' : 
                  metrics.memory.percent > 50 ? 'bg-yellow-500' : 'bg-green-500'
                }`}
                style={{ width: `${metrics.memory.percent}%` }}
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
              {metrics.disk.percent.toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              {formatBytes(metrics.disk.used)} of {formatBytes(metrics.disk.total)}
            </p>
            <div className="mt-2 w-full bg-secondary rounded-full h-2">
              <div 
                className="h-2 rounded-full bg-blue-500 transition-all duration-300"
                style={{ width: `${metrics.disk.percent}%` }}
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
                <span>{formatBytes(metrics.network.bytesIn)}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Out:</span>
                <span>{formatBytes(metrics.network.bytesOut)}</span>
              </div>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              {metrics.network.packetsIn.toLocaleString()} packets in
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
            {processes
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
              ))}
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
                  {metrics.cpu.loadAverage[index]?.toFixed(2) || '0.00'}
                </div>
                <div className="text-xs text-muted-foreground">
                  {metrics.cpu.loadAverage[index] > metrics.cpu.cores ? 'High' : 'Normal'}
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}