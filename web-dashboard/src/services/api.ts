// API configuration
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:3177/api';

// API response types
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// System types
export interface SystemStatus {
  status: string;
  uptime: number;
  active_monitors: string[];
  enforcement_mode: string;
}

export interface ThreatMetrics {
  active_threats: number;
  threats_blocked: number;
  total_events: number;
  last_scan: string;
}

export interface NetworkMetrics {
  active_connections: number;
  blocked_connections: number;
  dns_queries: number;
  dns_blocked: number;
  bytes_in: number;
  bytes_out: number;
}

export interface SystemMetrics {
  cpu_usage: number;
  memory_usage: number;
  disk_usage: number;
  processes: number;
  temperature?: number;
  uptime: number;
}

export interface SecurityEvent {
  id: string;
  timestamp: string;
  event_type: string;
  severity: string;
  source: string;
  description: string;
  process?: string;
  user?: string;
  file_path?: string;
  network_info?: string;
  action_taken: string;
  metadata: Record<string, any>;
}

export interface NetworkConnection {
  id: string;
  timestamp: string;
  protocol: string;
  source_ip: string;
  source_port: number;
  dest_ip: string;
  dest_port: number;
  status: string;
  bytes_in: number;
  bytes_out: number;
  packets_in: number;
  packets_out: number;
  process?: string;
  risk_score: number;
}

export interface DnsQuery {
  id: string;
  timestamp: string;
  domain: string;
  query_type: string;
  source_ip: string;
  status: string;
  response_time: number;
  answer?: string;
  blocked_reason?: string;
}

export interface ThreatDetection {
  id: string;
  timestamp: string;
  threat_type: string;
  severity: string;
  source: string;
  target: string;
  description: string;
  status: string;
  action_taken: string;
  metadata: Record<string, any>;
}

export interface LiveEvent {
  id: string;
  timestamp: string;
  event_type: string;
  category: string;
  message: string;
  severity: string;
  source: string;
  data: Record<string, any>;
}

export interface SecurityPolicy {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  policy_type: string;
  rules: PolicyRule[];
  actions: string[];
  created_at: string;
  updated_at: string;
}

export interface PolicyRule {
  id: string;
  condition: string;
  parameters: Record<string, any>;
}

export interface Alert {
  id: string;
  timestamp: string;
  severity: string;
  title: string;
  description: string;
  source: string;
  policy_id?: string;
  event_id?: string;
  status: string;
  assigned_to?: string;
  resolved_at?: string;
  notes: AlertNote[];
}

export interface AlertNote {
  id: string;
  timestamp: string;
  author: string;
  content: string;
}

// API client class
class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<ApiResponse<T>> {
    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('API request failed:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  // Dashboard endpoints
  async getSystemStatus(): Promise<ApiResponse<SystemStatus>> {
    return this.request<SystemStatus>('/dashboard/status');
  }

  async getThreatMetrics(): Promise<ApiResponse<ThreatMetrics>> {
    return this.request<ThreatMetrics>('/dashboard/threats');
  }

  async getNetworkMetrics(): Promise<ApiResponse<NetworkMetrics>> {
    return this.request<NetworkMetrics>('/dashboard/network');
  }

  // System monitoring
  async getSystemMetrics(): Promise<ApiResponse<SystemMetrics>> {
    return this.request<SystemMetrics>('/system/metrics');
  }

  // Security events
  async getSecurityEvents(params?: {
    severity?: string;
    event_type?: string;
    limit?: number;
    offset?: number;
  }): Promise<ApiResponse<SecurityEvent[]>> {
    const queryParams = new URLSearchParams();
    if (params) {
      Object.entries(params).forEach(([key, value]) => {
        if (value !== undefined) {
          queryParams.append(key, value.toString());
        }
      });
    }
    const query = queryParams.toString() ? `?${queryParams.toString()}` : '';
    return this.request<SecurityEvent[]>(`/security/events${query}`);
  }

  async getSecurityEvent(id: string): Promise<ApiResponse<SecurityEvent>> {
    return this.request<SecurityEvent>(`/security/events/${id}`);
  }

  // Network monitoring
  async getNetworkConnections(): Promise<ApiResponse<NetworkConnection[]>> {
    return this.request<NetworkConnection[]>('/network/connections');
  }

  async getDnsQueries(): Promise<ApiResponse<DnsQuery[]>> {
    return this.request<DnsQuery[]>('/network/dns');
  }

  // Threat detection
  async getThreatDetections(): Promise<ApiResponse<ThreatDetection[]>> {
    return this.request<ThreatDetection[]>('/threats/detections');
  }

  // Live events
  async getLiveEvents(): Promise<ApiResponse<LiveEvent[]>> {
    return this.request<LiveEvent[]>('/live/events');
  }

  // Policies
  async getPolicies(): Promise<ApiResponse<SecurityPolicy[]>> {
    return this.request<SecurityPolicy[]>('/policies');
  }

  async getPolicy(id: string): Promise<ApiResponse<SecurityPolicy>> {
    return this.request<SecurityPolicy>(`/policies/${id}`);
  }

  async createPolicy(policy: Omit<SecurityPolicy, 'id' | 'created_at' | 'updated_at'>): Promise<ApiResponse<SecurityPolicy>> {
    return this.request<SecurityPolicy>('/policies', {
      method: 'POST',
      body: JSON.stringify(policy),
    });
  }

  async updatePolicy(id: string, policy: Partial<SecurityPolicy>): Promise<ApiResponse<SecurityPolicy>> {
    return this.request<SecurityPolicy>(`/policies/${id}`, {
      method: 'PUT',
      body: JSON.stringify(policy),
    });
  }

  async deletePolicy(id: string): Promise<ApiResponse<void>> {
    return this.request<void>(`/policies/${id}`, {
      method: 'DELETE',
    });
  }

  // Alerts
  async getAlerts(): Promise<ApiResponse<Alert[]>> {
    return this.request<Alert[]>('/alerts');
  }

  async getAlert(id: string): Promise<ApiResponse<Alert>> {
    return this.request<Alert>(`/alerts/${id}`);
  }

  async updateAlertStatus(id: string, status: string, assigned_to?: string): Promise<ApiResponse<Alert>> {
    return this.request<Alert>(`/alerts/${id}/status`, {
      method: 'PUT',
      body: JSON.stringify({ status, assigned_to }),
    });
  }

  async addAlertNote(id: string, author: string, content: string): Promise<ApiResponse<AlertNote>> {
    return this.request<AlertNote>(`/alerts/${id}/notes`, {
      method: 'POST',
      body: JSON.stringify({ author, content }),
    });
  }

  // WebSocket connection for live events
  connectWebSocket(onMessage: (event: LiveEvent) => void): WebSocket {
    const wsUrl = this.baseUrl.replace('http', 'ws').replace('/api', '/api/live/ws');
    const ws = new WebSocket(wsUrl);

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.type === 'LiveEvent' && message.data) {
          onMessage(message.data);
        }
      } catch (error) {
        console.error('WebSocket message parse error:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket connection closed');
      // Implement reconnection logic if needed
    };

    return ws;
  }
}

// Export singleton instance
export const api = new ApiClient(API_BASE_URL);