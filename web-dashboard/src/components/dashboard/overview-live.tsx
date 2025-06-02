import { useEffect, useState } from 'react';
import { 
  Shield, 
  Activity, 
  Network, 
  AlertTriangle,
  CheckCircle,
  XCircle,
  TrendingUp,
  TrendingDown,
  Loader2
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { api } from '@/services/api';
import type { SystemStatus, ThreatMetrics, NetworkMetrics, SystemMetrics, SecurityEvent } from '@/services/api';

export function DashboardOverviewLive() {
  const [loading, setLoading] = useState(true);
  const [systemStatus, setSystemStatus] = useState<SystemStatus | null>(null);
  const [threatMetrics, setThreatMetrics] = useState<ThreatMetrics | null>(null);
  const [networkMetrics, setNetworkMetrics] = useState<NetworkMetrics | null>(null);
  const [systemMetrics, setSystemMetrics] = useState<SystemMetrics | null>(null);
  const [recentEvents, setRecentEvents] = useState<SecurityEvent[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchDashboardData();
    const interval = setInterval(fetchDashboardData, 5000); // Refresh every 5 seconds
    return () => clearInterval(interval);
  }, []);

  const fetchDashboardData = async () => {
    try {
      const [statusRes, threatsRes, networkRes, metricsRes, eventsRes] = await Promise.all([
        api.getSystemStatus(),
        api.getThreatMetrics(),
        api.getNetworkMetrics(),
        api.getSystemMetrics(),
        api.getSecurityEvents({ limit: 5 })
      ]);

      if (statusRes.success) setSystemStatus(statusRes.data!);
      if (threatsRes.success) setThreatMetrics(threatsRes.data!);
      if (networkRes.success) setNetworkMetrics(networkRes.data!);
      if (metricsRes.success) setSystemMetrics(metricsRes.data!);
      if (eventsRes.success) setRecentEvents(eventsRes.data!);
      
      setLoading(false);
    } catch (err) {
      setError('Failed to fetch dashboard data');
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  };

  const getSeverityColor = (severity: string): string => {
    switch (severity.toLowerCase()) {
      case 'critical': return 'text-red-600';
      case 'high': return 'text-red-500';
      case 'medium': return 'text-yellow-500';
      case 'low': return 'text-blue-500';
      default: return 'text-gray-500';
    }
  };

  const getSeverityBadgeVariant = (severity: string): "default" | "secondary" | "destructive" | "outline" => {
    switch (severity.toLowerCase()) {
      case 'critical':
      case 'high': return 'destructive';
      case 'medium': return 'outline';
      default: return 'secondary';
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <Loader2 className="h-8 w-8 animate-spin" />
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
      {/* System Status Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">System Status</CardTitle>
            <Shield className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center space-x-2">
              {systemStatus?.status === 'secure' ? (
                <CheckCircle className="h-4 w-4 text-green-500" />
              ) : (
                <XCircle className="h-4 w-4 text-red-500" />
              )}
              <div className="text-2xl font-bold capitalize">{systemStatus?.status || 'Unknown'}</div>
            </div>
            <p className="text-xs text-muted-foreground">
              Uptime: {systemStatus ? formatUptime(systemStatus.uptime) : 'N/A'}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Threats</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center space-x-2">
              {(threatMetrics?.active_threats || 0) > 0 ? (
                <XCircle className="h-4 w-4 text-red-500" />
              ) : (
                <CheckCircle className="h-4 w-4 text-green-500" />
              )}
              <div className="text-2xl font-bold">{threatMetrics?.active_threats || 0}</div>
            </div>
            <p className="text-xs text-muted-foreground">
              {threatMetrics?.threats_blocked || 0} blocked today
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Network Traffic</CardTitle>
            <Network className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center space-x-2">
              <TrendingUp className="h-4 w-4 text-blue-500" />
              <div className="text-2xl font-bold">
                {networkMetrics ? formatBytes(networkMetrics.bytes_in + networkMetrics.bytes_out) : '0 B'}
              </div>
            </div>
            <p className="text-xs text-muted-foreground">
              {networkMetrics?.active_connections || 0} active connections
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">CPU Usage</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="flex items-center space-x-2">
              {(systemMetrics?.cpu_usage || 0) > 80 ? (
                <TrendingUp className="h-4 w-4 text-red-500" />
              ) : (
                <TrendingDown className="h-4 w-4 text-green-500" />
              )}
              <div className="text-2xl font-bold">
                {systemMetrics?.cpu_usage?.toFixed(1) || 0}%
              </div>
            </div>
            <p className="text-xs text-muted-foreground">
              Memory: {systemMetrics?.memory_usage?.toFixed(1) || 0}%
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Recent Events and Network Activity */}
      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Recent Security Events</CardTitle>
            <CardDescription>
              Latest file system and process monitoring alerts
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {recentEvents.length === 0 ? (
              <p className="text-sm text-muted-foreground">No recent events</p>
            ) : (
              recentEvents.map((event) => (
                <div key={event.id} className="flex items-center justify-between">
                  <div className="flex items-center space-x-2">
                    <AlertTriangle className={`h-4 w-4 ${getSeverityColor(event.severity)}`} />
                    <span className="text-sm truncate max-w-[250px]">{event.description}</span>
                  </div>
                  <Badge variant={getSeverityBadgeVariant(event.severity)}>
                    {event.severity}
                  </Badge>
                </div>
              ))
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Network Activity</CardTitle>
            <CardDescription>
              Network traffic and connection monitoring
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Network className="h-4 w-4 text-blue-500" />
                <span className="text-sm">DNS queries</span>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-sm font-medium">{networkMetrics?.dns_queries || 0}</span>
                {(networkMetrics?.dns_blocked || 0) > 0 && (
                  <Badge variant="destructive" className="text-xs">
                    {networkMetrics?.dns_blocked} blocked
                  </Badge>
                )}
              </div>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <Activity className="h-4 w-4 text-green-500" />
                <span className="text-sm">Active connections</span>
              </div>
              <Badge variant="secondary">{networkMetrics?.active_connections || 0}</Badge>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center space-x-2">
                <AlertTriangle className="h-4 w-4 text-orange-500" />
                <span className="text-sm">Blocked connections</span>
              </div>
              <Badge variant="outline">{networkMetrics?.blocked_connections || 0}</Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* System Information */}
      <Card>
        <CardHeader>
          <CardTitle>System Information</CardTitle>
          <CardDescription>
            Current system status and configuration
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Monitoring Status</h4>
              <div className="space-y-1 text-sm text-muted-foreground">
                {systemStatus?.active_monitors?.map((monitor) => (
                  <div key={monitor} className="flex justify-between">
                    <span className="capitalize">{monitor.replace('_', ' ')}</span>
                    <Badge variant="default">Active</Badge>
                  </div>
                ))}
              </div>
            </div>
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Configuration</h4>
              <div className="space-y-1 text-sm text-muted-foreground">
                <div className="flex justify-between">
                  <span>Enforcement Mode</span>
                  <Badge variant="outline" className="capitalize">
                    {systemStatus?.enforcement_mode || 'Unknown'}
                  </Badge>
                </div>
                <div className="flex justify-between">
                  <span>Total Events</span>
                  <span>{threatMetrics?.total_events || 0}</span>
                </div>
                <div className="flex justify-between">
                  <span>Processes</span>
                  <span>{systemMetrics?.processes || 0}</span>
                </div>
              </div>
            </div>
            <div className="space-y-2">
              <h4 className="text-sm font-medium">Resource Usage</h4>
              <div className="space-y-1 text-sm text-muted-foreground">
                <div className="flex justify-between">
                  <span>CPU Usage</span>
                  <span>{systemMetrics?.cpu_usage?.toFixed(1) || 0}%</span>
                </div>
                <div className="flex justify-between">
                  <span>Memory Usage</span>
                  <span>{systemMetrics?.memory_usage?.toFixed(1) || 0}%</span>
                </div>
                <div className="flex justify-between">
                  <span>Disk Usage</span>
                  <span>{systemMetrics?.disk_usage?.toFixed(1) || 0}%</span>
                </div>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}