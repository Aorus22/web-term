import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PanelLeft, Server, Key, ArrowLeftRight, Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'

import { ConnectionForm } from '@/features/connections/components/ConnectionForm'
import { TabBar } from '@/components/TabBar'
import { Toaster } from '@/components/ui/sonner'
import { TerminalPane } from '@/features/terminal/TerminalPane'
import { HostsPage } from '@/features/hosts/components/HostsPage'
import { SSHKeysPage } from '@/features/ssh-keys/components/SSHKeysPage'
import { PortForwardsPage } from '@/features/forwards/components/PortForwardsPage'
import { SettingsPage } from '@/features/settings/components/SettingsPage'
import { NewTabView } from '@/components/NewTabView'
import { useKeyboardShortcuts } from '@/hooks/use-keyboard-shortcuts'
import { CustomCursor } from '@/components/CustomCursor'

const queryClient = new QueryClient()

function AppContent() {
  const { 
    sidebarOpen, 
    toggleSidebar, 
    sessions, 
    activeSessionId,
    setActiveSession,
    removeSession,
    sidebarPage,
    setSidebarPage
  } = useAppStore()

  const isElectron = !!window.electron

  useKeyboardShortcuts({
    onNewTab: () => {
      setActiveSession(null)
      setSidebarPage('new-tab')
    },
    onCloseTab: () => {
      if (activeSessionId) removeSession(activeSessionId)
    },
    onNextTab: () => {
      if (sessions.length <= 1) return
      const idx = sessions.findIndex(s => s.id === activeSessionId)
      const next = sessions[(idx + 1) % sessions.length]
      setActiveSession(next.id)
    },
    onPrevTab: () => {
      if (sessions.length <= 1) return
      const idx = sessions.findIndex(s => s.id === activeSessionId)
      const prev = sessions[(idx - 1 + sessions.length) % sessions.length]
      setActiveSession(prev.id)
    },
  })

  return (
    <div className={cn(
      "flex h-screen w-full bg-background text-foreground overflow-hidden",
      isElectron && "rounded-xl border shadow-2xl"
    )}>
      {/* Sidebar */}
      <aside
        className={cn(
          "flex flex-col border-r bg-muted/30 transition-all duration-300 ease-in-out",
          sidebarOpen ? "w-[200px]" : "w-0 overflow-hidden border-none"
        )}
      >
        <div className="p-4 flex items-center justify-between">
          <h2 className="font-semibold text-lg tracking-tight">WebTerm</h2>
        </div>
        
        <Separator />
        
        <nav className="flex flex-col gap-1 p-2">
          <button 
            onClick={() => {
              setSidebarPage('hosts')
              setActiveSession(null)
            }} 
            className={cn(
              "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors", 
              sidebarPage === 'hosts' ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <Server className="h-4 w-4" /> Hosts
          </button>
          <button 
            onClick={() => {
              setSidebarPage('keys')
              setActiveSession(null)
            }} 
            className={cn(
              "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors", 
              sidebarPage === 'keys' ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <Key className="h-4 w-4" /> SSH Keys
          </button>
          <button 
            onClick={() => {
              setSidebarPage('forwards')
              setActiveSession(null)
            }} 
            className={cn(
              "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors", 
              sidebarPage === 'forwards' ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <ArrowLeftRight className="h-4 w-4" /> Port Forwards
          </button>
        </nav>

        <div className="flex-1" />
        <Separator />
        <div className="p-2">
          <button 
            onClick={() => {
              setSidebarPage('settings')
              setActiveSession(null)
            }} 
            className={cn(
              "w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors", 
              sidebarPage === 'settings' ? "bg-accent text-accent-foreground" : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <Settings className="h-4 w-4" /> Settings
          </button>
        </div>
      </aside>

      {/* Main Area */}
      <main className="flex-1 flex flex-col min-w-0 bg-background overflow-hidden">
        {/* Tab Bar / Header */}
        <header 
          className={cn(
            "h-12 border-b flex items-center px-4 gap-4 bg-muted/10 shrink-0",
            isElectron && "drag-region"
          )}
          style={isElectron ? { WebkitAppRegion: 'drag' } as any : {}}
        >
          <div className="no-drag contents" style={{ WebkitAppRegion: 'no-drag' } as any}>
            <Button variant="ghost" size="icon" onClick={toggleSidebar} className="h-8 w-8 shrink-0">
              <PanelLeft className="h-4 w-4" />
            </Button>
            
            <div className="flex-1 flex items-center gap-2 overflow-x-auto no-scrollbar">
              <TabBar />
            </div>
          </div>
        </header>

        {/* Content Area — all sessions rendered, only active one visible */}
        <div className="flex-1 relative flex flex-col min-h-0 overflow-hidden">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={cn("absolute inset-0", session.id !== activeSessionId && "invisible pointer-events-none")}
            >
              <TerminalPane
                sessionId={session.id}
                isActive={session.id === activeSessionId}
                initialConnect={session.connectionId ? { connectionId: session.connectionId, host: session.host, port: session.port, username: session.username } : undefined}
              />
            </div>
          ))}
          {!activeSessionId && sidebarPage === 'new-tab' && (
            <NewTabView />
          )}
          {!activeSessionId && sidebarPage === 'hosts' && (
            <HostsPage />
          )}
          {!activeSessionId && sidebarPage === 'keys' && (
            <SSHKeysPage />
          )}
          {!activeSessionId && sidebarPage === 'forwards' && (
            <PortForwardsPage />
          )}
          {!activeSessionId && sidebarPage === 'settings' && (
            <SettingsPage />
          )}
        </div>
      </main>


      {/* Overlays */}
      <ConnectionForm />
      <Toaster />
      <CustomCursor />
    </div>
  )
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <TooltipProvider>
        <AppContent />
      </TooltipProvider>
    </QueryClientProvider>
  )
}

export default App
