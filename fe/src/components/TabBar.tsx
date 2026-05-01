import { X, Circle, Plus, Copy, Minus, Square, X as CloseIcon } from 'lucide-react'
import { useState, useEffect } from 'react'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'
import { 
  Popover, 
  PopoverContent, 
  PopoverTrigger 
} from '@/components/ui/popover'
import { Button } from '@/components/ui/button'
import { getSessionWebSocket } from '@/features/terminal/useSSHSession'

/**
 * Horizontal tab bar showing active SSH sessions.
 * Click a tab to switch, click × to close/disconnect.
 */
export function TabBar() {
  const sessions = useAppStore((s) => s.sessions)
  const activeSessionId = useAppStore((s) => s.activeSessionId)
  const setActiveSession = useAppStore((s) => s.setActiveSession)
  const setSidebarPage = useAppStore((s) => s.setSidebarPage)
  const removeSession = useAppStore((s) => s.removeSession)
  const duplicateSession = useAppStore((s) => s.duplicateSession)
  const [confirmingClose, setConfirmingClose] = useState<string | null>(null)
  const [plusPopoverOpen, setPlusPopoverOpen] = useState(false)
  const [windowState, setWindowState] = useState<'maximized' | 'restored'>('restored')
  const isElectron = !!window.electron
  const isWindows = window.electron?.platform === 'win32'
  
  // On Windows, we use titleBarOverlay (native caption buttons).
  // On Linux and macOS, we use custom web-based controls since we use a hidden title bar / frameless window.
  const showCustomControls = isElectron && !isWindows;

  useEffect(() => {
    if (isElectron && window.electron) {
      window.electron.getWindowState().then(setWindowState)
      window.electron.onWindowStateChange(setWindowState)
    }
  }, [isElectron])

  const handleDuplicate = async () => {
    if (!activeSessionId) return
    const source = sessions.find((s) => s.id === activeSessionId)
    if (!source || source.status !== 'connected') return

    const ws = getSessionWebSocket(activeSessionId)
    if (!ws || ws.readyState !== WebSocket.OPEN) return

    try {
      const cwd = await new Promise<string>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error('timeout')), 5000)
        const handler = (event: MessageEvent) => {
          if (typeof event.data === 'string') {
            try {
              const msg = JSON.parse(event.data)
              if (msg.type === 'cwd') {
                clearTimeout(timeout)
                ws.removeEventListener('message', handler)
                resolve(msg.path || '')
              }
            } catch {}
          }
        }
        ws.addEventListener('message', handler)
        ws.send(JSON.stringify({ type: 'get-cwd' }))
      })

      duplicateSession(activeSessionId, cwd)
    } catch {
      // Fallback: duplicate without cwd (starts in home directory)
      duplicateSession(activeSessionId, '')
    }
  }

  return (
    <div className="flex items-center w-full justify-between select-none">
      <div className="flex items-center gap-0.5 overflow-x-auto no-scrollbar flex-1 px-1 pt-1">
        {sessions.map((session) => (
          <button
            key={session.id}
            className={cn(
              "group flex items-center gap-2 px-3 py-1.5 rounded-t-md text-sm font-medium whitespace-nowrap transition-colors border border-b-0",
              session.id === activeSessionId
                ? "bg-background border-border text-foreground"
                : "bg-transparent border-transparent text-muted-foreground hover:text-foreground hover:bg-muted/50"
            )}
            style={isElectron ? { WebkitAppRegion: 'no-drag' } as any : {}}
            onClick={() => setActiveSession(session.id)}
          >
            <Circle
              className={cn(
                "h-1.5 w-1.5 shrink-0",
                session.status === 'connected' && "fill-green-500 text-green-500",
                session.status === 'connecting' && "fill-yellow-500 text-yellow-500 animate-pulse",
                session.status === 'disconnected' && "fill-none text-muted-foreground",
                session.status === 'error' && "fill-destructive text-destructive"
              )}
            />
            <span className="truncate max-w-[120px]">
              {session.label || `${session.username}@${session.host}`}
            </span>
            <Popover open={confirmingClose === session.id} onOpenChange={(open) => !open && setConfirmingClose(null)}>
              <PopoverTrigger
                render={
                  <span
                    role="button"
                    tabIndex={0}
                    className={cn(
                      "rounded-sm p-0.5 transition-opacity outline-none",
                      "opacity-0 group-hover:opacity-100 hover:bg-muted-foreground/10",
                      confirmingClose === session.id && "opacity-100"
                    )}
                    onClick={(e) => {
                      e.stopPropagation()
                      if (session.status === 'connected') {
                        setConfirmingClose(session.id)
                      } else {
                        removeSession(session.id)
                      }
                    }}
                  >
                    <X className={cn("h-3.5 w-3.5", confirmingClose === session.id && "text-destructive")} />
                  </span>
                }
              />
              <PopoverContent 
                side="bottom" 
                className="flex flex-col items-center gap-2 p-3 border shadow-md min-w-[180px]"
                onClick={(e) => e.stopPropagation()}
              >
                <p className="text-xs font-medium">Disconnect from {session.host}?</p>
                <div className="flex gap-2 w-full">
                  <Button 
                    size="sm" 
                    variant="destructive" 
                    className="flex-1 h-7 text-[10px]"
                    onClick={() => {
                      removeSession(session.id)
                      setConfirmingClose(null)
                    }}
                  >
                    Disconnect
                  </Button>
                  <Button 
                    size="sm" 
                    variant="outline" 
                    className="flex-1 h-7 text-[10px]"
                    onClick={() => setConfirmingClose(null)}
                  >
                    Cancel
                  </Button>
                </div>
              </PopoverContent>
            </Popover>
          </button>
        ))}

        {activeSessionId ? (
          <Popover open={plusPopoverOpen} onOpenChange={setPlusPopoverOpen}>
            <PopoverTrigger
              className="flex items-center justify-center h-8 w-8 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors shrink-0"
              style={isElectron ? { WebkitAppRegion: 'no-drag' } as any : {}}
              aria-label="New tab"
            >
              <Plus className="h-4 w-4" />
            </PopoverTrigger>
            <PopoverContent
              align="start"
              className="w-52 p-1"
            >
              <button
                className="flex items-center gap-3 w-full rounded-md px-3 py-2 text-sm text-left hover:bg-muted transition-colors"
                onClick={() => {
                  setPlusPopoverOpen(false)
                  handleDuplicate()
                }}
              >
                <Copy className="h-4 w-4 shrink-0 text-muted-foreground" />
                <div className="flex flex-col items-start">
                  <span className="font-medium">Duplicate</span>
                  <span className="text-xs text-muted-foreground">Same connection &amp; directory</span>
                </div>
              </button>
              <button
                className="flex items-center gap-3 w-full rounded-md px-3 py-2 text-sm text-left hover:bg-muted transition-colors"
                onClick={() => {
                  setPlusPopoverOpen(false)
                  setActiveSession(null)
                  setSidebarPage('new-tab')
                }}
              >
                <Plus className="h-4 w-4 shrink-0 text-muted-foreground" />
                <div className="flex flex-col items-start">
                  <span className="font-medium">New Connection</span>
                  <span className="text-xs text-muted-foreground">Connect to a server</span>
                </div>
              </button>
            </PopoverContent>
          </Popover>
        ) : (
          <button
            className="flex items-center justify-center h-8 w-8 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors shrink-0"
            style={isElectron ? { WebkitAppRegion: 'no-drag' } as any : {}}
            aria-label="New tab"
            onClick={() => {
              setActiveSession(null)
              setSidebarPage('new-tab')
            }}
          >
            <Plus className="h-4 w-4" />
          </button>
        )}
      </div>

      {/* Custom Window Controls (for Linux and macOS) */}
      {showCustomControls && (
        <div className="flex items-center h-full">
          <button 
            onClick={() => window.electron?.minimize()}
            className="flex items-center justify-center h-9 w-11 text-muted-foreground hover:bg-muted hover:text-foreground transition-all duration-200 ease-in-out"
            style={isElectron ? { WebkitAppRegion: 'no-drag' } as any : {}}
            title="Minimize"
          >
            <Minus className="h-4 w-4 stroke-[1.5]" />
          </button>
          <button 
            onClick={() => window.electron?.maximize()}
            className="flex items-center justify-center h-9 w-11 text-muted-foreground hover:bg-muted hover:text-foreground transition-all duration-200 ease-in-out"
            style={isElectron ? { WebkitAppRegion: 'no-drag' } as any : {}}
            title={windowState === 'maximized' ? "Restore" : "Maximize"}
          >
            {windowState === 'maximized' ? (
              <Copy className="h-3.5 w-3.5 stroke-[1.5]" />
            ) : (
              <Square className="h-3.5 w-3.5 stroke-[1.5]" />
            )}
          </button>
          <button 
            onClick={() => window.electron?.close()}
            className="flex items-center justify-center h-9 w-11 text-muted-foreground hover:bg-destructive hover:text-destructive-foreground transition-all duration-200 ease-in-out"
            style={isElectron ? { WebkitAppRegion: 'no-drag' } as any : {}}
            title="Close"
          >
            <CloseIcon className="h-4 w-4 stroke-[1.5]" />
          </button>
        </div>
      )}

      {/* Reserve space for Windows native caption buttons if using titleBarOverlay */}
      {isElectron && isWindows && (
        <div className="w-[138px] h-full shrink-0" />
      )}
    </div>
  )
}
