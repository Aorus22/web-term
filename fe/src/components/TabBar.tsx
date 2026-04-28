import { X, Circle, Plus } from 'lucide-react'
import { useState } from 'react'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { Button } from '@/components/ui/button'

/**
 * Horizontal tab bar showing active SSH sessions.
 * Click a tab to switch, click × to close/disconnect.
 */
export function TabBar() {
  const sessions = useAppStore((s) => s.sessions)
  const activeSessionId = useAppStore((s) => s.activeSessionId)
  const setActiveSession = useAppStore((s) => s.setActiveSession)
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
          
          <Tooltip open={confirmingClose === session.id} onOpenChange={(open) => !open && setConfirmingClose(null)}>
            <TooltipTrigger asChild>
              <span
                role="button"
                tabIndex={0}
                className={cn(
                  "rounded-sm p-0.5 transition-opacity",
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
                onKeyDown={(e) => {
                  if (e.key === 'Enter' || e.key === ' ') {
                    e.stopPropagation()
                    if (session.status === 'connected') {
                      setConfirmingClose(session.id)
                    } else {
                      removeSession(session.id)
                    }
                  }
                }}
                aria-label={`Close ${session.label}`}
              >
                <X className="h-3 w-3" />
              </span>
            </TooltipTrigger>
            <TooltipContent 
              side="bottom" 
              className="flex flex-col items-center gap-2 p-3 bg-popover text-popover-foreground border shadow-md min-w-[180px]"
              onClick={(e) => e.stopPropagation()}
            >
              <p className="text-sm font-medium">Disconnect from {session.host}?</p>
              <div className="flex gap-2 w-full">
                <Button 
                  size="xs" 
                  variant="destructive" 
                  className="flex-1"
                  onClick={() => {
                    removeSession(session.id)
                    setConfirmingClose(null)
                  }}
                >
                  Disconnect
                </Button>
                <Button 
                  size="xs" 
                  variant="outline" 
                  className="flex-1"
                  onClick={() => setConfirmingClose(null)}
                >
                  Cancel
                </Button>
              </div>
            </TooltipContent>
          </Tooltip>
        </button>
      ))}

      <button
        className="flex items-center justify-center h-8 w-8 rounded-md text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors shrink-0"
        onClick={() => setActiveSession(null)}
        aria-label="New tab"
      >
        <Plus className="h-4 w-4" />
      </button>
    </div>
  )
}
