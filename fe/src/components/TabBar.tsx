import { X, Circle, Plus } from 'lucide-react'
import { useState } from 'react'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'
import { 
  Popover, 
  PopoverContent, 
  PopoverTrigger 
} from '@/components/ui/popover'
import { Button } from '@/components/ui/button'

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
  const [confirmingClose, setConfirmingClose] = useState<string | null>(null)

  return (
    <div className="flex items-center gap-0.5 overflow-x-auto no-scrollbar">
      {sessions.map((session) => (
        <button
          key={session.id}
          className={cn(
            "group flex items-center gap-2 px-3 py-1.5 rounded-t-md text-sm font-medium whitespace-nowrap transition-colors border border-b-0",
            session.id === activeSessionId
              ? "bg-background border-border text-foreground"
              : "bg-transparent border-transparent text-muted-foreground hover:text-foreground hover:bg-muted/50"
          )}
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

      <button
        className="flex items-center justify-center h-8 w-8 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors shrink-0"
        onClick={() => {
          setActiveSession(null)
          setSidebarPage('new-tab')
        }}
        aria-label="New tab"
      >
        <Plus className="h-4 w-4" />
      </button>
    </div>
  )
}
