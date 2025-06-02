import { useState } from 'react'
import { Sidebar } from '@/components/layout/sidebar'
import { Header } from '@/components/layout/header'
import { DashboardOverview } from '@/components/dashboard/overview'
import { SecurityEvents } from '@/components/security/security-events'
import { NetworkMonitor } from '@/components/network/network-monitor'
import { ActivityMonitor } from '@/components/activity/activity-monitor'
import { ThreatDetection } from '@/components/threats/threat-detection'
import { EventLogs } from '@/components/logs/event-logs'
import { LiveMonitor } from '@/components/live/live-monitor'
import { Settings } from '@/components/settings/settings'

function App() {
  const [activeTab, setActiveTab] = useState('dashboard')

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return <DashboardOverview />
      case 'security':
        return <SecurityEvents />
      case 'network':
        return <NetworkMonitor />
      case 'activity':
        return <ActivityMonitor />
      case 'threats':
        return <ThreatDetection />
      case 'logs':
        return <EventLogs />
      case 'live':
        return <LiveMonitor />
      case 'settings':
        return <Settings />
      default:
        return <DashboardOverview />
    }
  }

  return (
    <div className="h-screen flex bg-background">
      {/* Sidebar */}
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} />
      
      {/* Main Content */}
      <div className="flex-1 flex flex-col">
        {/* Header */}
        <Header activeTab={activeTab} />
        
        {/* Content */}
        <main className="flex-1 overflow-auto p-6">
          {renderContent()}
        </main>
      </div>
    </div>
  )
}

export default App