import { useState, useEffect } from 'react'
import { 
  Settings as SettingsIcon, 
  Shield,
  Network,
  Bell,
  Database,
  Key,
  Monitor,
  Save,
  RefreshCw,
  AlertTriangle,
  CheckCircle,
  Upload,
  Download,
  Trash2,
  Plus
} from "lucide-react"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { api } from '@/services/api'

interface SecuritySettings {
  enforcementMode: 'passive' | 'permissive' | 'enforcing'
  enableFileMonitoring: boolean
  enableNetworkFiltering: boolean
  enableProcessMonitoring: boolean
  enableThreatDetection: boolean
  logLevel: 'debug' | 'info' | 'warning' | 'error'
  quarantineEnabled: boolean
  autoUpdate: boolean
}

interface NetworkSettings {
  enableDnsFiltering: boolean
  enablePacketCapture: boolean
  enableIptablesIntegration: boolean
  defaultInterface: string
  captureBufferSize: number
  maxConnections: number
  dnsBlacklist: string[]
  trustedNetworks: string[]
}

interface NotificationSettings {
  enableEmailNotifications: boolean
  enableWebhooks: boolean
  criticalThreshold: number
  emailRecipients: string[]
  webhookUrl: string
  notificationFrequency: 'immediate' | 'hourly' | 'daily'
}

interface SystemSettings {
  maxLogRetention: number
  enablePerformanceMonitoring: boolean
  cpuThreshold: number
  memoryThreshold: number
  diskThreshold: number
  enableAutoBackup: boolean
  backupLocation: string
}

export function Settings() {
  const [activeTab, setActiveTab] = useState<'security' | 'network' | 'notifications' | 'system'>('security')
  const [hasChanges, setHasChanges] = useState(false)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)

  const [securitySettings, setSecuritySettings] = useState<SecuritySettings>({
    enforcementMode: 'enforcing',
    enableFileMonitoring: true,
    enableNetworkFiltering: true,
    enableProcessMonitoring: true,
    enableThreatDetection: true,
    logLevel: 'info',
    quarantineEnabled: true,
    autoUpdate: true
  })

  const [networkSettings, setNetworkSettings] = useState<NetworkSettings>({
    enableDnsFiltering: true,
    enablePacketCapture: true,
    enableIptablesIntegration: true,
    defaultInterface: 'eth0',
    captureBufferSize: 1024,
    maxConnections: 10000,
    dnsBlacklist: [],
    trustedNetworks: []
  })

  const [notificationSettings, setNotificationSettings] = useState<NotificationSettings>({
    enableEmailNotifications: false,
    enableWebhooks: false,
    criticalThreshold: 5,
    emailRecipients: [],
    webhookUrl: '',
    notificationFrequency: 'immediate'
  })

  const [systemSettings, setSystemSettings] = useState<SystemSettings>({
    maxLogRetention: 30,
    enablePerformanceMonitoring: true,
    cpuThreshold: 80,
    memoryThreshold: 85,
    diskThreshold: 90,
    enableAutoBackup: true,
    backupLocation: '/var/backups/fluxdefense'
  })
  
  // Fetch settings from API
  useEffect(() => {
    fetchSettings()
  }, [])
  
  const fetchSettings = async () => {
    setLoading(true)
    try {
      const response = await api.getSettings()
      if (response.success && response.data) {
        setSecuritySettings(response.data.security)
        setNetworkSettings(response.data.network)
        setNotificationSettings(response.data.notifications)
        setSystemSettings(response.data.system)
      }
    } catch (error) {
      console.error('Failed to fetch settings:', error)
    } finally {
      setLoading(false)
    }
  }

  const handleSave = async () => {
    setSaving(true)
    try {
      const response = await api.updateSettings({
        security: securitySettings,
        network: networkSettings,
        notifications: notificationSettings,
        system: systemSettings
      })
      if (response.success) {
        setHasChanges(false)
      }
    } catch (error) {
      console.error('Failed to save settings:', error)
    } finally {
      setSaving(false)
    }
  }

  const handleReset = () => {
    // Reset to defaults
    setHasChanges(false)
  }

  const addDnsBlacklist = () => {
    const domain = prompt('Enter domain to blacklist:')
    if (domain && !networkSettings.dnsBlacklist.includes(domain)) {
      setNetworkSettings(prev => ({
        ...prev,
        dnsBlacklist: [...prev.dnsBlacklist, domain]
      }))
      setHasChanges(true)
    }
  }

  const removeDnsBlacklist = (domain: string) => {
    setNetworkSettings(prev => ({
      ...prev,
      dnsBlacklist: prev.dnsBlacklist.filter(d => d !== domain)
    }))
    setHasChanges(true)
  }

  const addEmailRecipient = () => {
    const email = prompt('Enter email address:')
    if (email && !notificationSettings.emailRecipients.includes(email)) {
      setNotificationSettings(prev => ({
        ...prev,
        emailRecipients: [...prev.emailRecipients, email]
      }))
      setHasChanges(true)
    }
  }

  const removeEmailRecipient = (email: string) => {
    setNotificationSettings(prev => ({
      ...prev,
      emailRecipients: prev.emailRecipients.filter(e => e !== email)
    }))
    setHasChanges(true)
  }

  const tabs = [
    { id: 'security', label: 'Security', icon: Shield },
    { id: 'network', label: 'Network', icon: Network },
    { id: 'notifications', label: 'Notifications', icon: Bell },
    { id: 'system', label: 'System', icon: Database }
  ]

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight">Settings</h2>
          <p className="text-muted-foreground">
            Configure FluxDefense security monitoring and system preferences
          </p>
        </div>
        <div className="flex items-center space-x-2">
          {hasChanges && (
            <Badge variant="outline" className="border-orange-500 text-orange-700">
              Unsaved Changes
            </Badge>
          )}
          <Button variant="outline" size="sm" onClick={handleReset}>
            <RefreshCw className="h-4 w-4 mr-2" />
            Reset
          </Button>
          <Button size="sm" onClick={handleSave} disabled={!hasChanges || saving || loading}>
            <Save className="h-4 w-4 mr-2" />
            {saving ? 'Saving...' : 'Save Changes'}
          </Button>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex space-x-2">
        {tabs.map((tab) => {
          const Icon = tab.icon
          return (
            <Button
              key={tab.id}
              variant={activeTab === tab.id ? 'default' : 'outline'}
              onClick={() => setActiveTab(tab.id as any)}
            >
              <Icon className="h-4 w-4 mr-2" />
              {tab.label}
            </Button>
          )
        })}
      </div>

      {/* Security Settings */}
      {activeTab === 'security' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Security Policy</CardTitle>
              <CardDescription>
                Configure core security monitoring and enforcement settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Enforcement Mode</label>
                  <select
                    value={securitySettings.enforcementMode}
                    onChange={(e) => {
                      setSecuritySettings(prev => ({ ...prev, enforcementMode: e.target.value as any }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  >
                    <option value="passive">Passive (Log Only)</option>
                    <option value="permissive">Permissive (Log + Allow)</option>
                    <option value="enforcing">Enforcing (Log + Block)</option>
                  </select>
                  <p className="text-xs text-muted-foreground">
                    How the system responds to policy violations
                  </p>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Log Level</label>
                  <select
                    value={securitySettings.logLevel}
                    onChange={(e) => {
                      setSecuritySettings(prev => ({ ...prev, logLevel: e.target.value as any }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  >
                    <option value="debug">Debug</option>
                    <option value="info">Info</option>
                    <option value="warning">Warning</option>
                    <option value="error">Error</option>
                  </select>
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="text-sm font-medium">Monitoring Components</h4>
                <div className="grid gap-4 md:grid-cols-2">
                  {[
                    { key: 'enableFileMonitoring', label: 'File System Monitoring', desc: 'Monitor file access and modifications' },
                    { key: 'enableNetworkFiltering', label: 'Network Filtering', desc: 'Filter network traffic and connections' },
                    { key: 'enableProcessMonitoring', label: 'Process Monitoring', desc: 'Monitor process execution and behavior' },
                    { key: 'enableThreatDetection', label: 'Threat Detection', desc: 'Advanced malware and threat analysis' },
                    { key: 'quarantineEnabled', label: 'Quarantine', desc: 'Automatically quarantine detected threats' },
                    { key: 'autoUpdate', label: 'Auto Updates', desc: 'Automatically update threat signatures' }
                  ].map((setting) => (
                    <div key={setting.key} className="flex items-start space-x-3">
                      <input
                        type="checkbox"
                        checked={securitySettings[setting.key as keyof SecuritySettings] as boolean}
                        onChange={(e) => {
                          setSecuritySettings(prev => ({ ...prev, [setting.key]: e.target.checked }))
                          setHasChanges(true)
                        }}
                        className="mt-1"
                      />
                      <div>
                        <div className="text-sm font-medium">{setting.label}</div>
                        <div className="text-xs text-muted-foreground">{setting.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Network Settings */}
      {activeTab === 'network' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Network Configuration</CardTitle>
              <CardDescription>
                Configure network monitoring and filtering settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Default Interface</label>
                  <input
                    type="text"
                    value={networkSettings.defaultInterface}
                    onChange={(e) => {
                      setNetworkSettings(prev => ({ ...prev, defaultInterface: e.target.value }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                    placeholder="eth0"
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Capture Buffer Size (KB)</label>
                  <input
                    type="number"
                    value={networkSettings.captureBufferSize}
                    onChange={(e) => {
                      setNetworkSettings(prev => ({ ...prev, captureBufferSize: parseInt(e.target.value) }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  />
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="text-sm font-medium">Network Features</h4>
                <div className="space-y-3">
                  {[
                    { key: 'enableDnsFiltering', label: 'DNS Filtering', desc: 'Filter DNS queries and block malicious domains' },
                    { key: 'enablePacketCapture', label: 'Packet Capture', desc: 'Capture and analyze network packets' },
                    { key: 'enableIptablesIntegration', label: 'IPTables Integration', desc: 'Integrate with Linux firewall rules' }
                  ].map((setting) => (
                    <div key={setting.key} className="flex items-start space-x-3">
                      <input
                        type="checkbox"
                        checked={networkSettings[setting.key as keyof NetworkSettings] as boolean}
                        onChange={(e) => {
                          setNetworkSettings(prev => ({ ...prev, [setting.key]: e.target.checked }))
                          setHasChanges(true)
                        }}
                        className="mt-1"
                      />
                      <div>
                        <div className="text-sm font-medium">{setting.label}</div>
                        <div className="text-xs text-muted-foreground">{setting.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h4 className="text-sm font-medium">DNS Blacklist</h4>
                  <Button size="sm" variant="outline" onClick={addDnsBlacklist}>
                    <Plus className="h-4 w-4 mr-2" />
                    Add Domain
                  </Button>
                </div>
                <div className="space-y-2">
                  {networkSettings.dnsBlacklist.map((domain, index) => (
                    <div key={index} className="flex items-center justify-between p-2 border rounded">
                      <span className="text-sm font-mono">{domain}</span>
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => removeDnsBlacklist(domain)}
                      >
                        <Trash2 className="h-4 w-4" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Notification Settings */}
      {activeTab === 'notifications' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Notification Settings</CardTitle>
              <CardDescription>
                Configure alerts and notification delivery methods
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Critical Event Threshold</label>
                  <input
                    type="number"
                    value={notificationSettings.criticalThreshold}
                    onChange={(e) => {
                      setNotificationSettings(prev => ({ ...prev, criticalThreshold: parseInt(e.target.value) }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  />
                  <p className="text-xs text-muted-foreground">
                    Send notifications when this many critical events occur
                  </p>
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Notification Frequency</label>
                  <select
                    value={notificationSettings.notificationFrequency}
                    onChange={(e) => {
                      setNotificationSettings(prev => ({ ...prev, notificationFrequency: e.target.value as any }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  >
                    <option value="immediate">Immediate</option>
                    <option value="hourly">Hourly Digest</option>
                    <option value="daily">Daily Summary</option>
                  </select>
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="text-sm font-medium">Notification Methods</h4>
                <div className="space-y-3">
                  <div className="flex items-start space-x-3">
                    <input
                      type="checkbox"
                      checked={notificationSettings.enableEmailNotifications}
                      onChange={(e) => {
                        setNotificationSettings(prev => ({ ...prev, enableEmailNotifications: e.target.checked }))
                        setHasChanges(true)
                      }}
                      className="mt-1"
                    />
                    <div>
                      <div className="text-sm font-medium">Email Notifications</div>
                      <div className="text-xs text-muted-foreground">Send alerts via email</div>
                    </div>
                  </div>

                  <div className="flex items-start space-x-3">
                    <input
                      type="checkbox"
                      checked={notificationSettings.enableWebhooks}
                      onChange={(e) => {
                        setNotificationSettings(prev => ({ ...prev, enableWebhooks: e.target.checked }))
                        setHasChanges(true)
                      }}
                      className="mt-1"
                    />
                    <div>
                      <div className="text-sm font-medium">Webhook Notifications</div>
                      <div className="text-xs text-muted-foreground">Send alerts to external systems</div>
                    </div>
                  </div>
                </div>
              </div>

              {notificationSettings.enableEmailNotifications && (
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <h4 className="text-sm font-medium">Email Recipients</h4>
                    <Button size="sm" variant="outline" onClick={addEmailRecipient}>
                      <Plus className="h-4 w-4 mr-2" />
                      Add Email
                    </Button>
                  </div>
                  <div className="space-y-2">
                    {notificationSettings.emailRecipients.map((email, index) => (
                      <div key={index} className="flex items-center justify-between p-2 border rounded">
                        <span className="text-sm">{email}</span>
                        <Button
                          size="sm"
                          variant="ghost"
                          onClick={() => removeEmailRecipient(email)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {notificationSettings.enableWebhooks && (
                <div className="space-y-2">
                  <label className="text-sm font-medium">Webhook URL</label>
                  <input
                    type="url"
                    value={notificationSettings.webhookUrl}
                    onChange={(e) => {
                      setNotificationSettings(prev => ({ ...prev, webhookUrl: e.target.value }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                    placeholder="https://your-webhook-endpoint.com"
                  />
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      )}

      {/* System Settings */}
      {activeTab === 'system' && (
        <div className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>System Configuration</CardTitle>
              <CardDescription>
                Configure system resources, logging, and maintenance settings
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              <div className="grid gap-6 md:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">Log Retention (days)</label>
                  <input
                    type="number"
                    value={systemSettings.maxLogRetention}
                    onChange={(e) => {
                      setSystemSettings(prev => ({ ...prev, maxLogRetention: parseInt(e.target.value) }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">Backup Location</label>
                  <input
                    type="text"
                    value={systemSettings.backupLocation}
                    onChange={(e) => {
                      setSystemSettings(prev => ({ ...prev, backupLocation: e.target.value }))
                      setHasChanges(true)
                    }}
                    className="w-full px-3 py-2 border border-input rounded-md bg-background"
                  />
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="text-sm font-medium">Performance Thresholds</h4>
                <div className="grid gap-4 md:grid-cols-3">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">CPU Threshold (%)</label>
                    <input
                      type="number"
                      min="0"
                      max="100"
                      value={systemSettings.cpuThreshold}
                      onChange={(e) => {
                        setSystemSettings(prev => ({ ...prev, cpuThreshold: parseInt(e.target.value) }))
                        setHasChanges(true)
                      }}
                      className="w-full px-3 py-2 border border-input rounded-md bg-background"
                    />
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium">Memory Threshold (%)</label>
                    <input
                      type="number"
                      min="0"
                      max="100"
                      value={systemSettings.memoryThreshold}
                      onChange={(e) => {
                        setSystemSettings(prev => ({ ...prev, memoryThreshold: parseInt(e.target.value) }))
                        setHasChanges(true)
                      }}
                      className="w-full px-3 py-2 border border-input rounded-md bg-background"
                    />
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium">Disk Threshold (%)</label>
                    <input
                      type="number"
                      min="0"
                      max="100"
                      value={systemSettings.diskThreshold}
                      onChange={(e) => {
                        setSystemSettings(prev => ({ ...prev, diskThreshold: parseInt(e.target.value) }))
                        setHasChanges(true)
                      }}
                      className="w-full px-3 py-2 border border-input rounded-md bg-background"
                    />
                  </div>
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="text-sm font-medium">System Features</h4>
                <div className="space-y-3">
                  {[
                    { key: 'enablePerformanceMonitoring', label: 'Performance Monitoring', desc: 'Monitor system resource usage' },
                    { key: 'enableAutoBackup', label: 'Automatic Backups', desc: 'Automatically backup configuration and logs' }
                  ].map((setting) => (
                    <div key={setting.key} className="flex items-start space-x-3">
                      <input
                        type="checkbox"
                        checked={systemSettings[setting.key as keyof SystemSettings] as boolean}
                        onChange={(e) => {
                          setSystemSettings(prev => ({ ...prev, [setting.key]: e.target.checked }))
                          setHasChanges(true)
                        }}
                        className="mt-1"
                      />
                      <div>
                        <div className="text-sm font-medium">{setting.label}</div>
                        <div className="text-xs text-muted-foreground">{setting.desc}</div>
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              <div className="space-y-4">
                <h4 className="text-sm font-medium">Data Management</h4>
                <div className="flex space-x-2">
                  <Button variant="outline" size="sm">
                    <Upload className="h-4 w-4 mr-2" />
                    Import Config
                  </Button>
                  <Button variant="outline" size="sm">
                    <Download className="h-4 w-4 mr-2" />
                    Export Config
                  </Button>
                  <Button variant="outline" size="sm">
                    <RefreshCw className="h-4 w-4 mr-2" />
                    Reset to Defaults
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}