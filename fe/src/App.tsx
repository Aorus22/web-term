import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PanelLeft, Plus } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { ScrollArea } from '@/components/ui/scroll-area'
import { TooltipProvider } from '@/components/ui/tooltip'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'

import { TagFilter } from '@/features/connections/components/TagFilter'
import { ConnectionList } from '@/features/connections/components/ConnectionList'
import { ConnectionForm } from '@/features/connections/components/ConnectionForm'
import { TabBar } from '@/components/TabBar'
import { ThemeToggle } from '@/components/ThemeToggle'
import { TerminalPane } from '@/features/terminal/TerminalPane'
import { NewTabView } from '@/components/NewTabView'
import { useTheme } from '@/hooks/use-theme'
import { useKeyboardShortcuts } from '@/hooks/use-keyboard-shortcuts'

const queryClient = new QueryClient()

function AppContent() {
  const { 
    sidebarOpen, 
    toggleSidebar, 
    setCreatingConnection, 
    sessions, 
    activeSessionId,
    setActiveSession,
    removeSession
  } = useAppStore()
  const { resolvedTheme } = useTheme()

  useKeyboardShortcuts({
    onNewTab: () => setActiveSession(null),
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
    <div className="flex h-screen w-full bg-background text-foreground overflow-hidden">
      {/* Sidebar */}
      <aside
        className={cn(
          "flex flex-col border-r bg-muted/30 transition-all duration-300 ease-in-out",
          sidebarOpen ? "w-[280px]" : "w-0 overflow-hidden border-none"
        )}
      >
        <div className="p-4 flex items-center justify-between">
          <h2 className="font-semibold text-lg tracking-tight">WebTerm</h2>
          <Button variant="ghost" size="icon" onClick={() => setCreatingConnection(true)}>
            <Plus className="h-4 w-4" />
          </Button>
        </div>
        
        <Separator />
        
        <TagFilter />
        
        <ScrollArea className="flex-1">
          <ConnectionList />
        </ScrollArea>
      </aside>

      {/* Main Area */}
      <main className="flex-1 flex flex-col min-w-0 bg-background">
        {/* Tab Bar / Header */}
        <header className="h-12 border-b flex items-center px-4 gap-4 bg-muted/10">
          {!sidebarOpen && (
            <Button variant="ghost" size="icon" onClick={toggleSidebar} className="h-8 w-8">
              <PanelLeft className="h-4 w-4" />
            </Button>
          )}
          {sidebarOpen && (
             <Button variant="ghost" size="icon" onClick={toggleSidebar} className="h-8 w-8">
                <PanelLeft className="h-4 w-4" />
             </Button>
          )}
          
          <div className="flex-1 flex items-center gap-2 overflow-x-auto no-scrollbar">
            <TabBar />
          </div>
          <ThemeToggle />
        </header>

        {/* Content Area — all sessions rendered, only active one visible */}
        <div className="flex-1 relative">
          {sessions.map((session) => (
            <div
              key={session.id}
              className={cn("absolute inset-0", session.id !== activeSessionId && "invisible pointer-events-none")}
            >
              <TerminalPane
                sessionId={session.id}
                isActive={session.id === activeSessionId}
                initialConnect={session.connectionId ? { connectionId: session.connectionId, host: session.host, port: session.port, username: session.username } : undefined}
                theme={resolvedTheme}
              />
            </div>
          ))}
          {!activeSessionId && (
            <NewTabView />
          )}
        </div>
      </main>

      {/* Overlays */}
      <ConnectionForm />
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
