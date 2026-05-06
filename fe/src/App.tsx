import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { PanelLeft, Server, Key, ArrowLeftRight, Settings, Files, Clipboard } from 'lucide-react'
import { useState, useEffect } from 'react'
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
import { SFTPView } from '@/features/sftp/components/SFTPView'
import { ClipboardView } from '@/features/clipboard/components/ClipboardView'
import { NewTabView } from '@/components/NewTabView'
import { useKeyboardShortcuts } from '@/hooks/use-keyboard-shortcuts'
import { AppThemeProvider } from '@/components/AppThemeProvider'

import { 
  isDesktop, 
  isTauri,
  getWindowState, 
  onWindowStateChange,
  getBackendPort,
  onBackendReady,
  startDragging,
  startResizing
} from '@/lib/desktop-ipc'

function ResizeHandles({ windowState }: { windowState: 'maximized' | 'restored' }) {
  if (!isTauri || windowState === 'maximized') return null;

  const handles = [
    { id: 'top', dir: 'North', className: 'absolute top-0 left-0 right-0 h-1 cursor-ns-resize z-50 border-none' },
    { id: 'bottom', dir: 'South', className: 'absolute bottom-0 left-0 right-0 h-1 cursor-ns-resize z-50 border-none' },
    { id: 'left', dir: 'West', className: 'absolute top-0 bottom-0 left-0 w-1 cursor-ew-resize z-50 border-none' },
    { id: 'right', dir: 'East', className: 'absolute top-0 bottom-0 right-0 w-1 cursor-ew-resize z-50 border-none' },
    { id: 'top-left', dir: 'NorthWest', className: 'absolute top-0 left-0 w-2 h-2 cursor-nwse-resize z-[60] border-none' },
    { id: 'top-right', dir: 'NorthEast', className: 'absolute top-0 right-0 w-2 h-2 cursor-nesw-resize z-[60] border-none' },
    { id: 'bottom-left', dir: 'SouthWest', className: 'absolute bottom-0 left-0 w-2 h-2 cursor-nesw-resize z-[60] border-none' },
    { id: 'bottom-right', dir: 'SouthEast', className: 'absolute bottom-0 right-0 w-2 h-2 cursor-nwse-resize z-[60] border-none' },
  ];

  return (
    <>
      {handles.map((h) => (
        <div
          key={h.id}
          className={h.className}
          onMouseDown={(e) => {
            if (e.button === 0) {
              e.preventDefault();
              startResizing(h.dir);
            }
          }}
        />
      ))}
    </>
  );
}

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
    setSidebarPage,
    setBackendPort,
    rehydrateSessions,
    backendPort
  } = useAppStore()

  const [windowState, setWindowState] = useState<'maximized' | 'restored'>('restored')
  const [, setResizeKey] = useState(0)

  useEffect(() => {
    if (backendPort !== 0) {
      rehydrateSessions()
    }
  }, [backendPort, rehydrateSessions])

  useEffect(() => {
    if (isDesktop) {
      getWindowState().then(setWindowState)
      getBackendPort().then((port) => {
        if (port !== 0) setBackendPort(port)
      })
      
      const unlistenWindow = onWindowStateChange(setWindowState)
      const unlistenBackend = onBackendReady((port) => {
        setBackendPort(port)
      })
      
      // Force repaint on any resize to fix Linux WebKit bugs
      const handleResize = () => {
        setResizeKey(prev => prev + 1)
      };
      window.addEventListener('resize', handleResize);
      
      // Manual drag fallback for Tauri on Linux
      const handleMouseDown = (e: MouseEvent) => {
        if (isTauri) {
          const target = e.target as HTMLElement;
          // Check if the target or any parent has the drag attribute
          if (target.closest('[data-tauri-drag-region]') && e.button === 0) {
            startDragging();
          }
        }
      };
      window.addEventListener('mousedown', handleMouseDown);
      
      return () => {
        unlistenWindow()
        unlistenBackend()
        window.removeEventListener('mousedown', handleMouseDown)
        window.removeEventListener('resize', handleResize)
      }
    } else {
      setBackendPort(8080)
    }
  }, [setBackendPort])

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

  if (isDesktop && backendPort === 0) {
    return (
      <div className="flex h-screen w-screen items-center justify-center bg-background text-foreground">
        <div className="flex flex-col items-center gap-4">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
          <p className="text-sm font-medium animate-pulse">Starting Backend...</p>
        </div>
      </div>
    )
  }

  return (
    <div 
      className={cn(
        "flex h-screen w-full bg-background text-foreground transition-all duration-300 relative",
        isDesktop && windowState !== 'maximized' && "rounded-xl border border-border shadow-2xl overflow-hidden"
      )}
    >
      <ResizeHandles windowState={windowState} />
      {/* Sidebar */}
      <aside
        className={cn(
          "flex flex-col border-r bg-background transition-all duration-300 ease-in-out",
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
              sidebarPage === 'hosts' ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/50 hover:text-accent-foreground"
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
              sidebarPage === 'keys' ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/50 hover:text-accent-foreground"
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
              sidebarPage === 'forwards' ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <ArrowLeftRight className="h-4 w-4" /> Port Forwards
          </button>
          <button 
            onClick={() => {
              setSidebarPage('sftp')
              setActiveSession(null)
            }} 
            className={cn(
              "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors", 
              sidebarPage === 'sftp' ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <Files className="h-4 w-4" /> SFTP
          </button>
          <button 
            onClick={() => {
              setSidebarPage('clipboard')
              setActiveSession(null)
            }} 
            className={cn(
              "flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium transition-colors", 
              sidebarPage === 'clipboard' ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <Clipboard className="h-4 w-4" /> Clipboard
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
              sidebarPage === 'settings' ? "bg-accent text-accent-foreground" : "text-foreground hover:bg-accent/50 hover:text-accent-foreground"
            )}
          >
            <Settings className="h-4 w-4" /> Settings
          </button>
        </div>
      </aside>

      {/* Main Area */}
      <main className="flex-1 flex flex-col min-w-0 bg-transparent overflow-hidden">
        {/* Tab Bar / Header */}
        <header 
          data-tauri-drag-region
          className={cn(
            "h-12 border-b flex items-center px-4 gap-4 bg-secondary/50 shrink-0",
            isDesktop && "drag-region"
          )}
          style={isDesktop ? { WebkitAppRegion: 'drag' } as any : {}}
        >
          <Button 
            variant="ghost" 
            size="icon" 
            onClick={toggleSidebar} 
            className="h-8 w-8 shrink-0 relative z-20 no-drag"
            style={isDesktop ? { WebkitAppRegion: 'no-drag' } as any : {}}
          >
            <PanelLeft className="h-4 w-4" />
          </Button>
          
          <div 
            className="flex-1 flex items-center gap-2 overflow-x-auto no-scrollbar relative z-20"
          >
            <TabBar />
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
                initialConnect={session.connectionId && session.status !== 'detached' ? { type: session.type, connectionId: session.connectionId, host: session.host, port: session.port, username: session.username } : undefined}
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
          {!activeSessionId && sidebarPage === 'sftp' && (
            <SFTPView />
          )}
          {!activeSessionId && sidebarPage === 'clipboard' && (
            <ClipboardView />
          )}
        </div>
      </main>


      {/* Overlays */}
      <ConnectionForm />
      <Toaster />
    </div>
  )
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AppThemeProvider>
        <TooltipProvider>
          <AppContent />
        </TooltipProvider>
      </AppThemeProvider>
    </QueryClientProvider>
  )
}

export default App
