import { useState, useEffect } from 'react';
import {
  Network,
  Globe,
  Shield,
  AlertTriangle,
  Eye,
  Search,
  RefreshCw,
  ArrowUpDown,
  ArrowUp,
  ArrowDown,
  Filter,
  Download,
  X,
  Activity,
  Clock,
  MapPin,
  Wifi,
  WifiOff,
  Lock,
  Unlock,
  Zap,
  TrendingUp,
  BarChart3
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { api } from '@/services/api';

interface NetworkConnection {
  id: string;
  local_address: string;
  local_port: number;
  remote_address: string;
  remote_port: number;
  protocol: string;
  state: string;
  process_name: string;
  process_id: number;
  user: string;
  bytes_sent: number;
  bytes_received: number;
  packets_sent: number;
  packets_received: number;
  duration: number;
  established_time: string;
  country: string;
  organization: string;
  is_encrypted: boolean;
  is_suspicious: boolean;
  risk_score: number;
  direction: 'inbound' | 'outbound' | 'bidirectional';
  service_name?: string;
  geolocation?: {
    city: string;
    region: string;
    country: string;
    latitude: number;
    longitude: number;
  };
}

interface NetworkStats {
  total_connections: number;
  active_connections: number;
  listening_ports: number;
  established_connections: number;
  total_bytes_sent: number;
  total_bytes_received: number;
  total_packets_sent: number;
  total_packets_received: number;
  suspicious_connections: number;
  encrypted_connections: number;
  unique_remote_hosts: number;
  bandwidth_usage: number;
  top_processes: Array<{
    name: string;
    connections: number;
    bytes_transferred: number;
  }>;
  top_destinations: Array<{
    address: string;
    connections: number;
    country: string;
  }>;
  protocol_distribution: Array<{
    protocol: string;
    count: number;
    percentage: number;
  }>;
}

type SortField = 'local_port' | 'remote_address' | 'process_name' | 'bytes_sent' | 'bytes_received' | 'duration' | 'risk_score';
type SortDirection = 'asc' | 'desc';

export function NetworkConnections() {
  const [connections, setConnections] = useState<NetworkConnection[]>([]);
  const [stats, setStats] = useState<NetworkStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedConnection, setSelectedConnection] = useState<NetworkConnection | null>(null);
  const [sortField, setSortField] = useState<SortField>('bytes_sent');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [protocolFilter, setProtocolFilter] = useState<string>('all');
  const [stateFilter, setStateFilter] = useState<string>('all');
  const [showSuspiciousOnly, setShowSuspiciousOnly] = useState(false);
  const [refreshInterval, setRefreshInterval] = useState(3000);
  const [autoRefresh, setAutoRefresh] = useState(true);

  useEffect(() => {
    fetchNetworkData();
    let interval: NodeJS.Timeout;
    if (autoRefresh) {
      interval = setInterval(fetchNetworkData, refreshInterval);
    }
    return () => {
      if (interval) clearInterval(interval);
    };
  }, [autoRefresh, refreshInterval]);

  const fetchNetworkData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const [connectionsRes, statsRes] = await Promise.all([
        api.getNetworkConnections(),
        fetch('/api/network/stats').then(res => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`);
          return res.json();
        })
      ]);
      
      if (connectionsRes.success && connectionsRes.data) {
        // Transform API data to match our interface
        const transformedConnections = connectionsRes.data.map((conn: any) => ({
          id: `${conn.local_address}:${conn.local_port}-${conn.remote_address}:${conn.remote_port}`,
          local_address: conn.local_address || '0.0.0.0',
          local_port: conn.local_port || 0,
          remote_address: conn.remote_address || '0.0.0.0',
          remote_port: conn.remote_port || 0,
          protocol: conn.protocol || 'TCP',
          state: conn.state || 'UNKNOWN',
          process_name: conn.process_name || 'unknown',
          process_id: conn.process_id || 0,
          user: conn.user || 'unknown',
          bytes_sent: conn.bytes_sent || 0,
          bytes_received: conn.bytes_received || 0,
          packets_sent: conn.packets_sent || 0,
          packets_received: conn.packets_received || 0,
          duration: conn.duration || 0,
          established_time: conn.established_time || new Date().toISOString(),
          country: conn.country || 'Unknown',
          organization: conn.organization || 'Unknown',
          is_encrypted: conn.is_encrypted || false,
          is_suspicious: conn.is_suspicious || false,
          risk_score: conn.risk_score || 0,
          direction: conn.direction || 'outbound' as 'inbound' | 'outbound',
          service_name: getServiceName(conn.remote_port || 0),
        }));
        
        setConnections(transformedConnections);
        
        // Use stats from API or generate from connections
        if (statsRes && typeof statsRes === 'object') {
          setStats(statsRes);
        } else {
          setStats(generateStatsFromConnections(transformedConnections));
        }
        
        setLoading(false);
      } else {
        throw new Error(connectionsRes.error || 'Failed to fetch network connections');
      }
    } catch (err) {
      console.error('Failed to fetch network data:', err);
      setError(err instanceof Error ? err.message : 'Failed to fetch network data');
      setConnections([]);
      setStats(null);
      setLoading(false);
    }
  };


  const getServiceName = (port: number): string => {
    const commonPorts: { [key: number]: string } = {
      21: 'FTP', 22: 'SSH', 23: 'Telnet', 25: 'SMTP', 53: 'DNS', 80: 'HTTP',
      110: 'POP3', 143: 'IMAP', 443: 'HTTPS', 993: 'IMAPS', 995: 'POP3S',
      3389: 'RDP', 5432: 'PostgreSQL', 3306: 'MySQL', 6379: 'Redis',
      27017: 'MongoDB', 8080: 'HTTP-Alt', 8443: 'HTTPS-Alt'
    };
    return commonPorts[port] || 'Unknown';
  };

  const generateStatsFromConnections = (connections: NetworkConnection[]): NetworkStats => {
    const totalBytesSent = connections.reduce((sum, conn) => sum + conn.bytes_sent, 0);
    const totalBytesReceived = connections.reduce((sum, conn) => sum + conn.bytes_received, 0);
    const totalPacketsSent = connections.reduce((sum, conn) => sum + conn.packets_sent, 0);
    const totalPacketsReceived = connections.reduce((sum, conn) => sum + conn.packets_received, 0);

    const processMap = new Map<string, { connections: number; bytes: number }>();
    const destinationMap = new Map<string, { connections: number; country: string }>();
    const protocolMap = new Map<string, number>();

    connections.forEach(conn => {
      // Process stats
      const processKey = conn.process_name;
      const processData = processMap.get(processKey) || { connections: 0, bytes: 0 };
      processData.connections++;
      processData.bytes += conn.bytes_sent + conn.bytes_received;
      processMap.set(processKey, processData);

      // Destination stats
      const destKey = conn.remote_address;
      const destData = destinationMap.get(destKey) || { connections: 0, country: conn.country };
      destData.connections++;
      destinationMap.set(destKey, destData);

      // Protocol stats
      const protocolCount = protocolMap.get(conn.protocol) || 0;
      protocolMap.set(conn.protocol, protocolCount + 1);
    });

    return {
      total_connections: connections.length,
      active_connections: connections.filter(c => c.state === 'ESTABLISHED').length,
      listening_ports: connections.filter(c => c.state === 'LISTENING').length,
      established_connections: connections.filter(c => c.state === 'ESTABLISHED').length,
      total_bytes_sent: totalBytesSent,
      total_bytes_received: totalBytesReceived,
      total_packets_sent: totalPacketsSent,
      total_packets_received: totalPacketsReceived,
      suspicious_connections: connections.filter(c => c.is_suspicious).length,
      encrypted_connections: connections.filter(c => c.is_encrypted).length,
      unique_remote_hosts: new Set(connections.map(c => c.remote_address)).size,
      bandwidth_usage: (totalBytesSent + totalBytesReceived) / (1024 * 1024), // MB
      top_processes: Array.from(processMap.entries())
        .map(([name, data]) => ({ name, connections: data.connections, bytes_transferred: data.bytes }))
        .sort((a, b) => b.bytes_transferred - a.bytes_transferred)
        .slice(0, 5),
      top_destinations: Array.from(destinationMap.entries())
        .map(([address, data]) => ({ address, connections: data.connections, country: data.country }))
        .sort((a, b) => b.connections - a.connections)
        .slice(0, 5),
      protocol_distribution: Array.from(protocolMap.entries())
        .map(([protocol, count]) => ({
          protocol,
          count,
          percentage: (count / connections.length) * 100
        }))
        .sort((a, b) => b.count - a.count)
    };
  };

  const filteredAndSortedConnections = () => {
    let filtered = connections.filter(connection => {
      const matchesSearch = 
        connection.remote_address.toLowerCase().includes(searchTerm.toLowerCase()) ||
        connection.process_name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        connection.local_port.toString().includes(searchTerm) ||
        connection.remote_port.toString().includes(searchTerm) ||
        (connection.service_name && connection.service_name.toLowerCase().includes(searchTerm.toLowerCase()));
      
      const matchesProtocol = protocolFilter === 'all' || connection.protocol === protocolFilter;
      const matchesState = stateFilter === 'all' || connection.state === stateFilter;
      const matchesSuspicious = !showSuspiciousOnly || connection.is_suspicious;
      
      return matchesSearch && matchesProtocol && matchesState && matchesSuspicious;
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

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return Math.round((bytes / Math.pow(1024, i)) * 100) / 100 + ' ' + sizes[i];
  };

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    
    if (hours > 0) return `${hours}h ${minutes}m`;
    if (minutes > 0) return `${minutes}m ${secs}s`;
    return `${secs}s`;
  };

  const getStateColor = (state: string) => {
    switch (state) {
      case 'ESTABLISHED': return 'bg-green-500';
      case 'LISTENING': return 'bg-blue-500';
      case 'TIME_WAIT': return 'bg-yellow-500';
      case 'CLOSE_WAIT': return 'bg-orange-500';
      case 'SYN_SENT': return 'bg-purple-500';
      case 'SYN_RECV': return 'bg-indigo-500';
      default: return 'bg-gray-500';
    }
  };

  const getRiskColor = (score: number) => {
    if (score > 70) return 'text-red-500 bg-red-50';
    if (score > 40) return 'text-yellow-600 bg-yellow-50';
    return 'text-green-600 bg-green-50';
  };

  const getDirectionIcon = (direction: string) => {
    switch (direction) {
      case 'inbound': return <ArrowDown className="h-3 w-3 text-blue-500" />;
      case 'outbound': return <ArrowUp className="h-3 w-3 text-green-500" />;
      default: return <ArrowUpDown className="h-3 w-3 text-gray-500" />;
    }
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
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">Network Connections</h1>
            <p className="text-muted-foreground">Real-time network activity monitoring and analysis</p>
          </div>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchNetworkData}
          >
            <RefreshCw className="h-4 w-4 mr-2" />
            Retry
          </Button>
        </div>
        <Card>
          <CardContent className="text-center py-8">
            <AlertTriangle className="h-12 w-12 mx-auto mb-4 text-red-500" />
            <h3 className="text-lg font-semibold mb-2">Failed to Load Network Data</h3>
            <p className="text-muted-foreground mb-4">{error}</p>
            <p className="text-sm text-muted-foreground">Please ensure the FluxDefense API server is running and accessible.</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">Network Connections</h1>
          <p className="text-muted-foreground">Real-time network activity monitoring and analysis</p>
        </div>
        <div className="flex items-center space-x-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setAutoRefresh(!autoRefresh)}
          >
            {autoRefresh ? <RefreshCw className="h-4 w-4 animate-spin" /> : <RefreshCw className="h-4 w-4" />}
            {autoRefresh ? 'Auto Refresh' : 'Refresh'}
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchNetworkData}
          >
            <RefreshCw className="h-4 w-4" />
            Manual Refresh
          </Button>
        </div>
      </div>

      {/* Statistics Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Connections</CardTitle>
            <Network className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.total_connections}</div>
            <p className="text-xs text-muted-foreground">
              {stats?.active_connections} active, {stats?.listening_ports} listening
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Data Transfer</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{formatBytes(stats?.total_bytes_sent || 0)}</div>
            <p className="text-xs text-muted-foreground">
              Sent: {formatBytes(stats?.total_bytes_sent || 0)}<br/>
              Received: {formatBytes(stats?.total_bytes_received || 0)}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Security Status</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">{stats?.encrypted_connections}</div>
            <p className="text-xs text-muted-foreground">
              Encrypted connections<br/>
              <span className="text-red-500">{stats?.suspicious_connections} suspicious</span>
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Remote Hosts</CardTitle>
            <Globe className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats?.unique_remote_hosts}</div>
            <p className="text-xs text-muted-foreground">
              Unique destinations
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
          <div className="grid gap-4 md:grid-cols-5">
            <div className="space-y-2">
              <label className="text-sm font-medium">Search</label>
              <div className="relative">
                <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search connections..."
                  value={searchTerm}
                  onChange={(e: React.ChangeEvent<HTMLInputElement>) => setSearchTerm(e.target.value)}
                  className="pl-8"
                />
              </div>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">Protocol</label>
              <select
                value={protocolFilter}
                onChange={(e) => setProtocolFilter(e.target.value)}
                className="w-full px-3 py-2 border border-input bg-background rounded-md"
              >
                <option value="all">All Protocols</option>
                <option value="TCP">TCP</option>
                <option value="UDP">UDP</option>
                <option value="ICMP">ICMP</option>
              </select>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">State</label>
              <select
                value={stateFilter}
                onChange={(e) => setStateFilter(e.target.value)}
                className="w-full px-3 py-2 border border-input bg-background rounded-md"
              >
                <option value="all">All States</option>
                <option value="ESTABLISHED">Established</option>
                <option value="LISTENING">Listening</option>
                <option value="TIME_WAIT">Time Wait</option>
                <option value="CLOSE_WAIT">Close Wait</option>
              </select>
            </div>
            
            <div className="space-y-2">
              <label className="text-sm font-medium">Refresh Rate</label>
              <select
                value={refreshInterval}
                onChange={(e) => setRefreshInterval(Number(e.target.value))}
                className="w-full px-3 py-2 border border-input bg-background rounded-md"
              >
                <option value="2000">2 seconds</option>
                <option value="3000">3 seconds</option>
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
                    checked={showSuspiciousOnly}
                    onChange={(e) => setShowSuspiciousOnly(e.target.checked)}
                  />
                  <span className="text-sm">Show suspicious only</span>
                </label>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Connections Table */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">
            Network Connections ({filteredAndSortedConnections().length})
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b">
                  <th className="text-left p-2">Dir</th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('process_name')}>
                    <div className="flex items-center space-x-1">
                      <span>Process</span>
                      {getSortIcon('process_name')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('local_port')}>
                    <div className="flex items-center space-x-1">
                      <span>Local Port</span>
                      {getSortIcon('local_port')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('remote_address')}>
                    <div className="flex items-center space-x-1">
                      <span>Remote Address</span>
                      {getSortIcon('remote_address')}
                    </div>
                  </th>
                  <th className="text-left p-2">Protocol</th>
                  <th className="text-left p-2">State</th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('bytes_sent')}>
                    <div className="flex items-center space-x-1">
                      <span>Data Sent</span>
                      {getSortIcon('bytes_sent')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('bytes_received')}>
                    <div className="flex items-center space-x-1">
                      <span>Data Received</span>
                      {getSortIcon('bytes_received')}
                    </div>
                  </th>
                  <th className="text-left p-2 cursor-pointer hover:bg-muted" onClick={() => handleSort('duration')}>
                    <div className="flex items-center space-x-1">
                      <span>Duration</span>
                      {getSortIcon('duration')}
                    </div>
                  </th>
                  <th className="text-left p-2">Security</th>
                  <th className="text-left p-2">Actions</th>
                </tr>
              </thead>
              <tbody>
                {connections.length === 0 ? (
                  <tr>
                    <td colSpan={11} className="text-center py-8 text-muted-foreground">
                      No network connections available. Data will appear here when connections are detected.
                    </td>
                  </tr>
                ) : (
                  filteredAndSortedConnections().slice(0, 100).map((connection) => (
                    <tr 
                      key={connection.id}
                      className={`border-b hover:bg-muted/50 cursor-pointer ${
                        connection.is_suspicious ? 'bg-red-50 hover:bg-red-100' : ''
                      }`}
                      onClick={() => setSelectedConnection(connection)}
                    >
                      <td className="p-2">
                        {getDirectionIcon(connection.direction)}
                      </td>
                      <td className="p-2">
                        <div className="flex items-center space-x-2">
                          <span className="font-medium">{connection.process_name}</span>
                          <span className="text-xs text-muted-foreground">({connection.process_id})</span>
                          {connection.is_suspicious && <AlertTriangle className="h-4 w-4 text-red-500" />}
                        </div>
                      </td>
                      <td className="p-2 font-mono">{connection.local_port}</td>
                      <td className="p-2">
                        <div>
                          <div className="font-mono">{connection.remote_address}:{connection.remote_port}</div>
                          {connection.service_name && (
                            <div className="text-xs text-muted-foreground">{connection.service_name}</div>
                          )}
                        </div>
                      </td>
                      <td className="p-2">
                        <Badge variant="outline">{connection.protocol}</Badge>
                      </td>
                      <td className="p-2">
                        <Badge variant="outline" className={getStateColor(connection.state) + ' text-white border-0'}>
                          {connection.state}
                        </Badge>
                      </td>
                      <td className="p-2 font-mono">{formatBytes(connection.bytes_sent)}</td>
                      <td className="p-2 font-mono">{formatBytes(connection.bytes_received)}</td>
                      <td className="p-2">{formatDuration(connection.duration)}</td>
                      <td className="p-2">
                        <div className="flex items-center space-x-1">
                          {connection.is_encrypted ? <Lock className="h-3 w-3 text-green-500" /> : <Unlock className="h-3 w-3 text-red-500" />}
                          <Badge variant="outline" className={getRiskColor(connection.risk_score)}>
                            {connection.risk_score.toFixed(0)}
                          </Badge>
                        </div>
                      </td>
                      <td className="p-2">
                        <Button size="sm" variant="ghost" onClick={(e) => { e.stopPropagation(); setSelectedConnection(connection); }}>
                          <Eye className="h-3 w-3" />
                        </Button>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Connection Detail Modal */}
      {selectedConnection && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <Card className="w-full max-w-4xl max-h-[90vh] overflow-y-auto">
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="flex items-center space-x-2">
                  <Network className="h-5 w-5" />
                  <span>Connection Details - {selectedConnection.process_name}</span>
                </CardTitle>
                <Button variant="ghost" size="sm" onClick={() => setSelectedConnection(null)}>
                  <X className="h-4 w-4" />
                </Button>
              </div>
            </CardHeader>
            <CardContent>
              <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Connection Information</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Local Address:</span>
                        <span className="font-mono">{selectedConnection.local_address}:{selectedConnection.local_port}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Remote Address:</span>
                        <span className="font-mono">{selectedConnection.remote_address}:{selectedConnection.remote_port}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Protocol:</span>
                        <Badge variant="outline">{selectedConnection.protocol}</Badge>
                      </div>
                      <div className="flex justify-between">
                        <span>State:</span>
                        <Badge variant="outline" className={getStateColor(selectedConnection.state) + ' text-white border-0'}>
                          {selectedConnection.state}
                        </Badge>
                      </div>
                      <div className="flex justify-between">
                        <span>Direction:</span>
                        <div className="flex items-center space-x-1">
                          {getDirectionIcon(selectedConnection.direction)}
                          <span className="capitalize">{selectedConnection.direction}</span>
                        </div>
                      </div>
                      <div className="flex justify-between">
                        <span>Service:</span>
                        <span>{selectedConnection.service_name || 'Unknown'}</span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Process Information</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Process Name:</span>
                        <span className="font-medium">{selectedConnection.process_name}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Process ID:</span>
                        <span className="font-mono">{selectedConnection.process_id}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>User:</span>
                        <span>{selectedConnection.user}</span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Data Transfer</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Bytes Sent:</span>
                        <span className="font-mono">{formatBytes(selectedConnection.bytes_sent)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Bytes Received:</span>
                        <span className="font-mono">{formatBytes(selectedConnection.bytes_received)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Packets Sent:</span>
                        <span className="font-mono">{selectedConnection.packets_sent.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Packets Received:</span>
                        <span className="font-mono">{selectedConnection.packets_received.toLocaleString()}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="space-y-4">
                  <div>
                    <h3 className="font-semibold mb-2">Timing Information</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Established:</span>
                        <span>{new Date(selectedConnection.established_time).toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Duration:</span>
                        <span>{formatDuration(selectedConnection.duration)}</span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Geographic Information</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Country:</span>
                        <span>{selectedConnection.country}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Organization:</span>
                        <span>{selectedConnection.organization}</span>
                      </div>
                      {selectedConnection.geolocation && (
                        <>
                          <div className="flex justify-between">
                            <span>City:</span>
                            <span>{selectedConnection.geolocation.city}</span>
                          </div>
                          <div className="flex justify-between">
                            <span>Coordinates:</span>
                            <span className="font-mono">
                              {selectedConnection.geolocation.latitude.toFixed(4)}, {selectedConnection.geolocation.longitude.toFixed(4)}
                            </span>
                          </div>
                        </>
                      )}
                    </div>
                  </div>

                  <div>
                    <h3 className="font-semibold mb-2">Security Analysis</h3>
                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span>Encrypted:</span>
                        <div className="flex items-center space-x-1">
                          {selectedConnection.is_encrypted ? <Lock className="h-3 w-3 text-green-500" /> : <Unlock className="h-3 w-3 text-red-500" />}
                          <span className={selectedConnection.is_encrypted ? 'text-green-500' : 'text-red-500'}>
                            {selectedConnection.is_encrypted ? 'Yes' : 'No'}
                          </span>
                        </div>
                      </div>
                      <div className="flex justify-between">
                        <span>Risk Score:</span>
                        <Badge variant="outline" className={getRiskColor(selectedConnection.risk_score)}>
                          {selectedConnection.risk_score.toFixed(0)}/100
                        </Badge>
                      </div>
                      <div className="flex justify-between">
                        <span>Suspicious:</span>
                        <span className={selectedConnection.is_suspicious ? 'text-red-500' : 'text-green-500'}>
                          {selectedConnection.is_suspicious ? 'Yes' : 'No'}
                        </span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div className="mt-6 flex justify-end space-x-2">
                <Button variant="outline" size="sm">
                  <Download className="h-4 w-4 mr-2" />
                  Export Details
                </Button>
                <Button variant="destructive" size="sm" disabled={selectedConnection.state === 'LISTENING'}>
                  <X className="h-4 w-4 mr-2" />
                  Terminate Connection
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
}