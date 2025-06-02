import { Bell, Search, User } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { ThemeToggle } from "@/components/theme-toggle"

interface HeaderProps {
  activeTab: string
}

export function Header({ activeTab }: HeaderProps) {
  const getTabTitle = (tab: string) => {
    const titles: Record<string, string> = {
      dashboard: "Dashboard",
      security: "Security Events",
      network: "Network Monitor", 
      activity: "Activity Monitor",
      threats: "Threat Detection",
      logs: "Event Logs",
      live: "Live Monitor",
      settings: "Settings"
    }
    return titles[tab] || "FluxDefense"
  }

  return (
    <header className="h-16 bg-card border-b border-border flex items-center justify-between px-6">
      {/* Title */}
      <div className="flex items-center space-x-4">
        <h2 className="text-2xl font-semibold">{getTabTitle(activeTab)}</h2>
        <Badge variant="outline" className="text-xs">
          ACTIVE
        </Badge>
      </div>

      {/* Actions */}
      <div className="flex items-center space-x-4">
        {/* Search */}
        <Button variant="outline" size="sm" className="w-64 justify-start text-muted-foreground">
          <Search className="h-4 w-4 mr-2" />
          Search events...
        </Button>

        {/* Notifications */}
        <Button variant="outline" size="icon" className="relative">
          <Bell className="h-4 w-4" />
          <Badge 
            variant="destructive" 
            className="absolute -top-2 -right-2 h-5 w-5 text-xs p-0 flex items-center justify-center"
          >
            3
          </Badge>
        </Button>

        {/* Theme Toggle */}
        <ThemeToggle />

        {/* User Menu */}
        <Button variant="outline" size="icon">
          <User className="h-4 w-4" />
        </Button>
      </div>
    </header>
  )
}