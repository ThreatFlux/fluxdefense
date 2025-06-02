import { useState } from 'react'
import { Sidebar } from '@/components/layout/sidebar'
import { Header } from '@/components/layout/header'
import { DashboardOverview } from '@/components/dashboard/overview'
import { DashboardOverviewLive } from '@/components/dashboard/overview-live'
import { NetworkMonitor } from '@/components/network/network-monitor'
import { ProcessManager } from '@/components/processes/process-manager'
import { NetworkConnections } from '@/components/network/network-connections'
import { EventLogs } from '@/components/logs/event-logs'
import { Settings } from '@/components/settings/settings'

function App() {
  const [activeTab, setActiveTab] = useState('dashboard')
  const [useLiveData, setUseLiveData] = useState(true) // Toggle for live vs static data

  const renderContent = () => {
    switch (activeTab) {
      case 'dashboard':
        return useLiveData ? <DashboardOverviewLive /> : <DashboardOverview />
      case 'network':
        return <NetworkMonitor />
      case 'processes':
        return <ProcessManager />
      case 'connections':
        return <NetworkConnections />
      case 'logs':
        return <EventLogs />
      case 'settings':
        return <Settings />
      default:
        return useLiveData ? <DashboardOverviewLive /> : <DashboardOverview />
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