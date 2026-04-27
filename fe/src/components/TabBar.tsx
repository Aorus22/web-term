import { X } from 'lucide-react'
import { useAppStore } from '@/stores/app-store'
import { cn } from '@/lib/utils'

/**
 * Horizontal tab bar showing active SSH sessions.
 * Click a tab to switch, click × to close/disconnect.
 */
export function TabBar() {
  const sessions = useAppStore((s) => s.sessions)
  const activeSessionId = useAppStore((s) => s.activeSessionId)
  const setActiveSession = useAppStore((s) => s.setActiveSession)
  const removeSession = useAppStore((s) => s.removeSession)

  // No sessions — render nothing
  if (sessions.length === 0) return null

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
          <span className="truncate max-w-[120px]">
            {session.label || `${session.username}@${session.host}`}
          </span>
          <span
            role="button"
            tabIndex={0}
            className={cn(
              "rounded-sm p-0.5 transition-opacity",
              "opacity-0 group-hover:opacity-100 hover:bg-muted-foreground/10"
            )}
            onClick={(e) => {
              e.stopPropagation()
              removeSession(session.id)
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter' || e.key === ' ') {
                e.stopPropagation()
                removeSession(session.id)
              }
            }}
            aria-label={`Close ${session.label}`}
          >
            <X className="h-3 w-3" />
          </span>
        </button>
      ))}
    </div>
  )
}
