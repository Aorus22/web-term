import { useEffect, useRef } from 'react'
import { Terminal } from '@wterm/react'
import '@wterm/react/css'
import { Loader2, RefreshCw, AlertTriangle } from 'lucide-react'
import { useSSHSession } from './useSSHSession'
import { useAppStore } from '@/stores/app-store'
import type { ConnectOptions } from './types'

interface TerminalPaneProps {
  sessionId: string
  /** When provided and session is not connected/connecting, auto-connect on mount */
  initialConnect?: ConnectOptions
}

export function TerminalPane({ sessionId, initialConnect }: TerminalPaneProps) {
  const { ref, connect, sendData, sendResize, disconnect } = useSSHSession(sessionId)
  const session = useAppStore((s) => s.sessions.find((s) => s.id === sessionId))
  const lastOptionsRef = useRef<ConnectOptions | null>(initialConnect ?? null)

  // Auto-connect on mount if initialConnect provided and not already connected
  useEffect(() => {
    if (initialConnect) {
      lastOptionsRef.current = initialConnect
      const currentStatus = session?.status
      if (!currentStatus || currentStatus === 'disconnected' || currentStatus === 'error') {
        connect(initialConnect)
      }
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps

  const handleRetry = () => {
    if (lastOptionsRef.current) {
      connect(lastOptionsRef.current)
    }
  }

  // No session found — shouldn't happen but handle gracefully
  if (!session) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <p>Session not found</p>
      </div>
    )
  }

  // Connecting state — spinner
  if (session.status === 'connecting') {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-3">
        <Loader2 className="h-8 w-8 animate-spin" />
        <p className="text-sm">Connecting to {session.host}...</p>
      </div>
    )
  }

  // Connected — render terminal
  if (session.status === 'connected') {
    return (
      <div className="h-full w-full">
        <Terminal
          ref={ref}
          autoResize
          cursorBlink
          onData={sendData}
          onResize={sendResize}
        />
      </div>
    )
  }

  // Error state — show error with retry button
  if (session.status === 'error') {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-4">
        <AlertTriangle className="h-8 w-8 text-destructive" />
        <div className="text-center space-y-2">
          <p className="text-sm text-foreground">{session.error || 'Connection failed'}</p>
          <p className="text-xs text-muted-foreground">Failed to connect to {session.host}</p>
        </div>
        <button
          onClick={handleRetry}
          className="inline-flex items-center gap-2 px-4 py-2 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          <RefreshCw className="h-4 w-4" />
          Retry
        </button>
      </div>
    )
  }

  // Disconnected state — show reconnect button
  if (session.status === 'disconnected') {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-4">
        <div className="text-center space-y-2">
          <p className="text-sm text-foreground">Disconnected from {session.host}</p>
          <p className="text-xs text-muted-foreground">The SSH session has ended</p>
        </div>
        <button
          onClick={handleRetry}
          className="inline-flex items-center gap-2 px-4 py-2 text-sm rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          <RefreshCw className="h-4 w-4" />
          Reconnect
        </button>
      </div>
    )
  }

  return null
}
