import { 
  Activity, 
  Network, 
  FileText, 
  Settings, 
  BarChart3,
  Globe,
  Shield
} from "lucide-react"
import { cn } from "@/lib/utils"
import { Button } from "@/components/ui/button"

interface SidebarProps {
  activeTab: string
  onTabChange: (tab: string) => void
}

const navigation = [
  {
    name: "Dashboard",
    id: "dashboard",
    icon: BarChart3,
    description: "System overview and metrics"
  },
  {
    name: "Network Monitor",
    id: "network", 
    icon: Network,
    description: "Network traffic and filtering"
  },
  {
    name: "Network Connections",
    id: "connections",
    icon: Globe,
    description: "Detailed network connection analysis"
  },
  {
    name: "Process Manager",
    id: "processes",
    icon: Activity,
    description: "System process monitoring"
  },
  {
    name: "Event Logs",
    id: "logs",
    icon: FileText,
    description: "Real-time system events"
  },
  {
    name: "Settings",
    id: "settings",
    icon: Settings,
    description: "Configuration and preferences"
  }
]

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  return (
    <div className="w-64 bg-card border-r border-border flex flex-col">
      {/* Header */}
      <div className="p-6 border-b border-border">
        <div className="flex items-center space-x-2">
          <Shield className="h-8 w-8 text-primary" />
          <div>
            <h1 className="text-xl font-bold">FluxDefense</h1>
            <p className="text-sm text-muted-foreground">Security Dashboard</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-2">
        {navigation.map((item) => {
          const Icon = item.icon
          const isActive = activeTab === item.id
          
          return (
            <Button
              key={item.id}
              variant={isActive ? "secondary" : "ghost"}
              className={cn(
                "w-full justify-start h-auto p-3 text-left",
                isActive && "bg-secondary text-secondary-foreground"
              )}
              onClick={() => onTabChange(item.id)}
            >
              <div className="flex items-start space-x-3">
                <Icon className="h-5 w-5 mt-0.5 shrink-0" />
                <div className="flex-1 min-w-0">
                  <div className="font-medium">{item.name}</div>
                  <div className="text-xs text-muted-foreground mt-0.5">
                    {item.description}
                  </div>
                </div>
              </div>
            </Button>
          )
        })}
      </nav>

      {/* Footer */}
      <div className="p-4 border-t border-border">
        <div className="text-xs text-muted-foreground">
          <div>FluxDefense v2.0.0</div>
          <div>Linux Security Monitor</div>
        </div>
      </div>
    </div>
  )
}