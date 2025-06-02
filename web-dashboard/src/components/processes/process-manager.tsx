import { useState, useEffect } from 'react';
import {
  Activity,
  Cpu,
  MemoryStick,
  HardDrive,
  Users,
  Clock,
  AlertTriangle,
  Shield,
  Terminal,
  Settings,
  Search,
  RefreshCw,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  X,
  Play,
  Square,
  Trash2,
  Eye,
  Filter,
  Download
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { api } from '@/services/api';

interface ProcessInfo {
  pid: number;
  ppid: number;
  name: string;
  command: string;
  user: string;
  cpu_percent: number;
  memory_percent: number;
  memory_mb: number;
  status: string;
  start_time: string;
  runtime: number;
  threads: number;
  priority: number;
  nice: number;
  executable: string;
  working_dir: string;
  open_files: number;
  network_connections: number;
  children: number;
  risk_score: number;
  is_system: boolean;
  is_suspicious: boolean;
}

interface ProcessStats {
  total_processes: number;
  running_processes: number;
  sleeping_processes: number;
  zombie_processes: number;
  total_threads: number;
  cpu_cores: number;
  system_load: number[];
  memory_total: number;
  memory_used: number;
  top_cpu_processes: ProcessInfo[];
  top_memory_processes: ProcessInfo[];
}

type SortField = 'pid' | 'name' | 'cpu_percent' | 'memory_percent' | 'memory_mb' | 'user' | 'status' | 'runtime';
type SortDirection = 'asc' | 'desc';

export function ProcessManager() {
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [stats, setStats] = useState<ProcessStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedProcess, setSelectedProcess] = useState<ProcessInfo | null>(null);
  const [sortField, setSortField] = useState<SortField>('cpu_percent');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [showSystemProcesses, setShowSystemProcesses] = useState(true);
  const [refreshInterval, setRefreshInterval] = useState(2000);
  const [autoRefresh, setAutoRefresh] = useState(true);

  useEffect(() => {
    fetchProcessData();
    let interval: NodeJS.Timeout;
    if (autoRefresh) {
      interval = setInterval(fetchProcessData, refreshInterval);
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [autoRefresh, refreshInterval]);

  const fetchProcessData = async () => {
    try {
      const [processesRes, statsRes] = await Promise.all([
        api.getProcesses(),
        api.getProcessStats()
      ]);
      
      if (processesRes.success && statsRes.success && processesRes.data && statsRes.data) {
        setProcesses(processesRes.data);
        setStats(statsRes.data);
        setLoading(false);
      } else {
        setError('Failed to load process data from API');
        setLoading(false);
      }
    } catch (err) {
      setError('Failed to fetch process data from API');
      setLoading(false);
    }
  };


  const filteredAndSortedProcesses = () => {
    let filtered = processes.filter(process => {
      const matchesSearch = process.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           process.command.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           process.user.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           process.pid.toString().includes(searchTerm);
      
      const matchesStatus = statusFilter === 'all' || process.status === statusFilter;
      const matchesSystemFilter = showSystemProcesses || !process.is_system;
      
      return matchesSearch && matchesStatus && matchesSystemFilter;
    });
    
    filtered.sort((a, b) => {
      const aValue = a[sortField];
      const bValue = b[sortField];
      
      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return sortDirection === 'asc' 
          ? aValue.localeCompare(bValue)
          : bValue.localeCompare(aValue);
      }
      
      return sortDirection === 'asc' 
        ? (aValue as number) - (bValue as number)
        : (bValue as number) - (aValue as number);
    });
    
    return filtered;
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('desc');
    }
  };

  const getSortIcon = (field: SortField) => {
    if (sortField !== field) return <ArrowUpDown className="h-4 w-4 opacity-50" />;
    return sortDirection === 'asc' 
      ? <ArrowUp className="h-4 w-4" />
      : <ArrowDown className="h-4 w-4" />;
  };

  const formatUptime = (seconds: number) => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return 'bg-green-500';
      case 'sleeping': return 'bg-blue-500';
      case 'waiting': return 'bg-yellow-500';
      case 'zombie': return 'bg-red-500';
      case 'stopped': return 'bg-gray-500';
      default: return 'bg-gray-400';
    }
  };

  const getRiskColor = (score: number) => {
    if (score > 70) return 'text-red-500 bg-red-50';
    if (score > 40) return 'text-yellow-600 bg-yellow-50';
    return 'text-green-600 bg-green-50';
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-8 w-8 animate-spin" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="text-center text-red-500 p-4">
        {error}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Process Manager</h1>
          <p className="text-muted-foreground">Advanced system process monitoring and management</p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            {autoRefresh ? <Square className="h-4 w-4" /> : <Play className="h-4 w-4" />}
            {autoRefresh ? 'Pause' : 'Resume'}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchProcessData}
          >
            <RefreshCw className="h-4 w-4" />
            Refresh
          </Button>
        </div>
      </div>

      {/* Statistics Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Processes</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.total_processes}</div>
            <p className="text-xs text-muted-foreground">
              {stats?.running_processes} running, {stats?.sleeping_processes} sleeping
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">System Load</CardTitle>
            <Cpu className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.system_load[0].toFixed(2)}</div>
            <p className="text-xs text-muted-foreground">
              {stats?.cpu_cores} cores, {stats?.total_threads} threads
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Memory Usage</CardTitle>
            <MemoryStick className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {((stats?.memory_used || 0) / (stats?.memory_total || 1) * 100).toFixed(1)}%
            </div>
            <p className="text-xs text-muted-foreground">
              {stats?.memory_used}MB / {stats?.memory_total}MB
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Suspicious</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-500">
              {processes.filter(p => p.is_suspicious).length}
            </div>
            <p className="text-xs text-muted-foreground">
              High risk processes detected
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Filters and Search */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Filters</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Search</label>
              <div className="relative">
                <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search processes..."
                  value={searchTerm}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearchTerm(e.target.value)}
                  className="pl-8"
                />
              </div>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">Status</label>
              <select
                value={statusFilter}
                onChange={(e) => setStatusFilter(e.target.value)}
                className="w-full px-3 py-2 border border-input bg-background rounded-md"
              >
                <option value="all">All Status</option>
                <option value="running">Running</option>
                <option value="sleeping">Sleeping</option>
                <option value="waiting">Waiting</option>
                <option value="zombie">Zombie</option>
                <option value="stopped">Stopped</option>
              </select>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">Refresh Rate</label>
              <select
                value={refreshInterval}
                onChange={(e) => setRefreshInterval(Number(e.target.value))}
                className="w-full px-3 py-2 border border-input bg-background rounded-md"
              >
                <option value="1000">1 second</option>
                <option value="2000">2 seconds</option>
                <option value="5000">5 seconds</option>
                <option value="10000">10 seconds</option>
              </select>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">Options</label>
              <div className="space-y-2">
                <label className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    checked={showSystemProcesses}
                    onChange={(e) => setShowSystemProcesses(e.target.checked)}
                  />
                  <span className="text-sm">Show system processes</span>
                </label>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Process Table */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">
            Processes ({filteredAndSortedProcesses().length})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b">
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('pid')}>
                    <div className="flex items-center space-x-1">
                      <span>PID</span>
                      {getSortIcon('pid')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('name')}>
                    <div className="flex items-center space-x-1">
                      <span>Name</span>
                      {getSortIcon('name')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('user')}>
                    <div className="flex items-center space-x-1">
                      <span>User</span>
                      {getSortIcon('user')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('cpu_percent')}>
                    <div className="flex items-center space-x-1">
                      <span>CPU%</span>
                      {getSortIcon('cpu_percent')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('memory_percent')}>
                    <div className="flex items-center space-x-1">
                      <span>Memory%</span>
                      {getSortIcon('memory_percent')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('memory_mb')}>
                    <div className="flex items-center space-x-1">
                      <span>Memory</span>
                      {getSortIcon('memory_mb')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('status')}>
                    <div className="flex items-center space-x-1">
                      <span>Status</span>
                      {getSortIcon('status')}
                    </div>
                  </th>
                  <th className="text-left p-2">Risk</th>
                  <th className="text-left p-2">Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredAndSortedProcesses().slice(0, 50).map((process) => (
                  <tr 
                    key={process.pid}
                    className={`border-b hover:bg-muted/50 cursor-pointer ${
                      process.is_suspicious ? 'bg-red-50 hover:bg-red-100' : ''
                    }`}
                    onClick={() => setSelectedProcess(process)}
                  >
                    <td className="p-2 font-mono">{process.pid}</td>
                    <td className="p-2">
                      <div className="flex items-center space-x-2">
                        <span className="font-medium">{process.name}</span>
                        {process.is_suspicious && <AlertTriangle className="h-4 w-4 text-red-500" />}
                        {process.is_system && <Shield className="h-4 w-4 text-blue-500" />}
                      </div>
                    </td>
                    <td className="p-2">{process.user}</td>
                    <td className="p-2">
                      <div className="flex items-center space-x-2">
                        <div className="w-12 bg-gray-200 rounded-full h-2">
                          <div 
                            className="bg-blue-600 h-2 rounded-full" 
                            style={{ width: `${Math.min(process.cpu_percent, 100)}%` }}
                          ></div>
                        </div>
                        <span className="text-xs font-mono">{process.cpu_percent.toFixed(1)}%</span>
                      </div>
                    </td>
                    <td className="p-2">
                      <div className="flex items-center space-x-2">
                        <div className="w-12 bg-gray-200 rounded-full h-2">
                          <div 
                            className="bg-green-600 h-2 rounded-full" 
                            style={{ width: `${Math.min(process.memory_percent, 100)}%` }}
                          ></div>
                        </div>
                        <span className="text-xs font-mono">{process.memory_percent.toFixed(1)}%</span>
                      </div>
                    </td>
                    <td className="p-2 font-mono">{process.memory_mb.toFixed(1)}MB</td>
                    <td className="p-2">
                      <Badge variant="outline" className={getStatusColor(process.status) + ' text-white border-0'}>
                        {process.status}
                      </Badge>
                    </td>
                    <td className="p-2">
                      <Badge variant="outline" className={getRiskColor(process.risk_score)}>
                        {process.risk_score.toFixed(0)}
                      </Badge>
                    </td>
                    <td className="p-2">
                      <div className="flex items-center space-x-1">
                        <Button size="sm" variant="ghost" onClick={(e) => { e.stopPropagation(); setSelectedProcess(process); }}>
                          <Eye className="h-3 w-3" />
                        </Button>
                        <Button size="sm" variant="ghost" className="text-red-500">
                          <Trash2 className="h-3 w-3" />
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Process Detail Modal */}
      {selectedProcess && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <Card className="w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center space-x-2">
                  <Terminal className="h-5 w-5" />
                  <span>Process Details - {selectedProcess.name} (PID: {selectedProcess.pid})</span>
                </CardTitle>
                <Button variant="ghost" size="sm" onClick={() => setSelectedProcess(null)}>
                  <X className="h-4 w-4" />
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Basic Information</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Process ID:</span>
                        <span className="font-mono">{selectedProcess.pid}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Parent PID:</span>
                        <span className="font-mono">{selectedProcess.ppid}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>User:</span>
                        <span>{selectedProcess.user}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Status:</span>
                        <Badge variant="outline" className={getStatusColor(selectedProcess.status) + ' text-white border-0'}>
                          {selectedProcess.status}
                        </Badge>
                      </div>
                      <div className="flex justify-between">
                        <span>Priority:</span>
                        <span>{selectedProcess.priority}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Nice:</span>
                        <span>{selectedProcess.nice}</span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Resource Usage</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>CPU Usage:</span>
                        <span className="font-mono">{selectedProcess.cpu_percent.toFixed(2)}%</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Memory Usage:</span>
                        <span className="font-mono">{selectedProcess.memory_percent.toFixed(2)}%</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Memory (MB):</span>
                        <span className="font-mono">{selectedProcess.memory_mb.toFixed(1)}MB</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Threads:</span>
                        <span>{selectedProcess.threads}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Open Files:</span>
                        <span>{selectedProcess.open_files}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Network Connections:</span>
                        <span>{selectedProcess.network_connections}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Process Details</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Executable:</span>
                        <span className="font-mono text-xs">{selectedProcess.executable}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Working Directory:</span>
                        <span className="font-mono text-xs">{selectedProcess.working_dir}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Start Time:</span>
                        <span>{new Date(selectedProcess.start_time).toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Runtime:</span>
                        <span>{formatUptime(selectedProcess.runtime)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Children:</span>
                        <span>{selectedProcess.children}</span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Security</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Risk Score:</span>
                        <Badge variant="outline" className={getRiskColor(selectedProcess.risk_score)}>
                          {selectedProcess.risk_score.toFixed(0)}/100
                        </Badge>
                      </div>
                      <div className="flex justify-between">
                        <span>System Process:</span>
                        <span>{selectedProcess.is_system ? 'Yes' : 'No'}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Suspicious:</span>
                        <span className={selectedProcess.is_suspicious ? 'text-red-500' : 'text-green-500'}>
                          {selectedProcess.is_suspicious ? 'Yes' : 'No'}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Command Line</h3>
                    <div className="bg-gray-100 p-3 rounded text-xs font-mono break-all">
                      {selectedProcess.command}
                    </div>
                  </div>
                </div>
              </div>

              <div className="mt-6 flex justify-end space-x-2">
                <Button variant="outline" size="sm">
                  <Download className="h-4 w-4 mr-2" />
                  Export Details
                </Button>
                <Button variant="destructive" size="sm">
                  <Trash2 className="h-4 w-4 mr-2" />
                  Terminate Process
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}