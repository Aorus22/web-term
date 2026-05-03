import { useEffect, useRef, useState } from 'react'
import { Terminal } from '@wterm/react'
import '@wterm/react/css'
import { Loader2, RefreshCw, AlertTriangle } from 'lucide-react'
import { useSSHSession } from './useSSHSession'
import { useAppStore } from '@/stores/app-store'
import { useSettings } from '@/features/settings/hooks/useSettings'
import { cn } from '@/lib/utils'
import { PasswordPrompt } from './PasswordPrompt'
import { PassphrasePrompt } from './PassphrasePrompt'
import { ReconnectOverlay } from './ReconnectOverlay'
import { SaveConnectionBanner } from './SaveConnectionBanner'
import { useTerminalMouse } from './useTerminalMouse'
import type { ConnectOptions } from './types'

interface TerminalPaneProps {
  sessionId: string
  isActive?: boolean
  /** When provided, auto-connect on mount (saved connection with stored password) */
  initialConnect?: ConnectOptions
}

export function TerminalPane({ sessionId, isActive, initialConnect }: TerminalPaneProps) {
  const { ref, connect, attach, sendData, sendResize, signalReady } = useSSHSession(sessionId)
  const { onReady: onTerminalMouseReady } = useTerminalMouse(sendData)
  const session = useAppStore((s) => s.sessions.find((s) => s.id === sessionId))
  const { data: settings } = useSettings()

  const handleTerminalReady = (wt: any) => {
    onTerminalMouseReady(wt)
    signalReady()
  }

  const removeSession = useAppStore((s) => s.removeSession)
  const lastOptionsRef = useRef<ConnectOptions | null>(initialConnect ?? null)
  const passphraseRef = useRef<string | null>(null)
  const [showSaveBanner, setShowSaveBanner] = useState(false)
  const [passwordProvided, setPasswordProvided] = useState(false)

  const hasConnectedRef = useRef(false)

  console.log(`[TerminalPane:${sessionId}] Render. Status: ${session?.status}, HasAttach: ${!!attach}`)

  // Derive Terminal props from settings
  const terminalTheme = settings?.terminal_color_theme || 'default'
  const cursorBlinkSetting = settings?.cursor_blink !== 'false'  // default true
  const cursorStyleClass = settings?.cursor_style === 'underline' ? 'cursor-underline'
    : settings?.cursor_style === 'bar' ? 'cursor-bar'
    : undefined  // block is default (no extra class needed)
  const fontFamily = settings?.font_family || 'Geist Mono'
  const fontSize = settings?.font_size || '14'

  const terminalClassName = cn(
    'wterm',
    `theme-${terminalTheme}`,
    cursorStyleClass,
    'has-scrollback'
  )

  const terminalStyle = {
    height: '100%',
    fontFamily: `'${fontFamily}', monospace`,
    fontSize: `${fontSize}px`,
  }

  // Auto-connect on mount (or as soon as settings are available)
  useEffect(() => {
    if (hasConnectedRef.current || !settings || session?.isRecovered) return

    const rows = 24
    const cols = 80
    const term = settings?.terminal_type || 'screen-256color'

    if (initialConnect) {
      hasConnectedRef.current = true
      const opts = { 
        ...initialConnect, 
        ...(session?.cwd && { cwd: session.cwd }), 
        term,
        rows,
        cols
      }
      lastOptionsRef.current = opts
      connect(opts)
    } else if (session?.type === 'local' && session?.status === 'connecting') {
      hasConnectedRef.current = true
      const opts: ConnectOptions = {
        type: 'local',
        rows,
        cols,
        term,
      }
      lastOptionsRef.current = opts
      connect(opts)
    } else if (session?.auth_method === 'key' && !session?.has_passphrase && session?.status === 'connecting') {
      hasConnectedRef.current = true
      const opts: ConnectOptions = {
        connectionId: session.connectionId,
        host: session.host,
        port: session.port,
        username: session.username,
        auth_method: 'key',
        ssh_key_id: session.ssh_key_id,
        rows,
        cols,
        ...(session.cwd && { cwd: session.cwd }),
        term,
      }
      lastOptionsRef.current = opts
      connect(opts)
    } else if (session?.cwd && session?.status === 'connecting') {
      hasConnectedRef.current = true
      const opts: ConnectOptions = {
        connectionId: session.connectionId,
        host: session.host,
        port: session.port,
        username: session.username,
        auth_method: session.auth_method as 'password' | 'key',
        ssh_key_id: session.ssh_key_id,
        rows,
        cols,
        cwd: session.cwd,
        term,
      }
      lastOptionsRef.current = opts
      connect(opts)
    }
  }, [settings, initialConnect, session?.status, connect]) // eslint-disable-line react-hooks/exhaustive-deps

  // Auto-attach for recovered sessions
  useEffect(() => {
    if (session?.status === 'detached' && attach) {
      console.log(`[TerminalPane:${sessionId}] Triggering auto-attach...`)
      attach()
    }
  }, [session?.status, attach, sessionId])

  // Show save banner when quick-connect session disconnects (D-06)
  useEffect(() => {
    if (session?.status === 'disconnected' && session.isQuickConnect) {
      setShowSaveBanner(true)
    }
  }, [session?.status, session?.isQuickConnect])

  // Scroll to bottom when tab becomes active
  useEffect(() => {
    if (!isActive) return
    const handle = ref.current
    if (!handle?.instance) return
    const el = handle.instance.element
    const maxScroll = el.scrollHeight - el.clientHeight
    if (maxScroll <= 0) return
    const rh = (handle.instance as any)._rowHeight || 17
    el.scrollTop = Math.floor(maxScroll / rh) * rh
  }, [isActive]) // eslint-disable-line react-hooks/exhaustive-deps

  const handleRetry = () => {
    if (lastOptionsRef.current) {
      // If key-auth and we have cached passphrase, use it (D-09)
      if (lastOptionsRef.current.auth_method === 'key' && passphraseRef.current) {
        lastOptionsRef.current = {
          ...lastOptionsRef.current,
          passphrase: passphraseRef.current,
        }
      }
      connect({ ...lastOptionsRef.current, term: settings?.terminal_type || 'screen-256color' })
    }
  }

  const handlePasswordConnect = (password: string) => {
    setPasswordProvided(true)
    const opts: ConnectOptions = {
      host: session!.host,
      port: session!.port,
      username: session!.username,
      password,
      term: settings?.terminal_type || 'screen-256color',
    }
    lastOptionsRef.current = opts
    connect(opts)
  }

  const handlePasswordCancel = () => {
    // Remove the session — user cancelled password prompt
    removeSession(sessionId)
  }

  const handlePassphraseConnect = (passphrase: string) => {
    passphraseRef.current = passphrase
    const opts: ConnectOptions = {
      connectionId: session!.connectionId,
      host: session!.host,
      port: session!.port,
      username: session!.username,
      auth_method: 'key',
      ssh_key_id: session!.ssh_key_id,
      passphrase,
      rows: 24,
      cols: 80,
      term: settings?.terminal_type || 'screen-256color',
    }
    lastOptionsRef.current = opts
    connect(opts)
  }

  const handlePassphraseCancel = () => {
    removeSession(sessionId)
  }

  const handleReconnectDismiss = () => {
    // Keep session in disconnected state, just hide the overlay
  }

  // No session found — shouldn't happen but handle gracefully
  if (!session) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <p>Session not found</p>
      </div>
    )
  }

  // Connecting with no initialConnect and no password provided → show PasswordPrompt (D-02, D-03)
  if (session.status === 'connecting' && !initialConnect && !passwordProvided) {
    return (
      <PasswordPrompt
        host={session.host}
        username={session.username}
        onConnect={handlePasswordConnect}
        onCancel={handlePasswordCancel}
      />
    )
  }

  // Detached state — spinner for re-attachment initiation
  if (session.status === 'detached') {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-3">
        <Loader2 className="h-8 w-8 animate-spin" />
        <p className="text-sm">
          {session.type === 'local' ? 'Resuming local session...' : `Resuming session to ${session.host}...`}
        </p>
      </div>
    )
  }

  // Connecting state — spinner (auto-connect via useEffect for saved connections)
  if (session.status === 'connecting') {
    return (
      <div className="flex flex-col items-center justify-center h-full text-muted-foreground gap-3">
        <Loader2 className="h-8 w-8 animate-spin" />
        <p className="text-sm">
          {session.type === 'local' ? 'Spawning local shell...' : `Connecting to ${session.host}...`}
        </p>
      </div>
    )
  }

  // Key-based auth needing passphrase (D-05, D-06)
  if (session.status === 'needs-passphrase') {
    return (
      <PassphrasePrompt
        keyName={session.key_name || 'SSH Key'}
        keyType={session.key_type || 'RSA'}
        host={session.host}
        onConnect={handlePassphraseConnect}
        onCancel={handlePassphraseCancel}
      />
    )
  }

  // Connected — render terminal
  if (session.status === 'connected') {
    return (
      <div className="h-full w-full">
        <Terminal
          ref={ref}
          autoResize
          cursorBlink={cursorBlinkSetting}
          theme={terminalTheme}
          className={terminalClassName}
          onData={sendData}
          onResize={sendResize}
          onReady={handleTerminalReady}
          style={terminalStyle}
        />
      </div>
    )
  }

  // Error state — show error with retry button (D-05)
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

  // Disconnected state — show frozen terminal + reconnect overlay + save banner (UI-04, D-06)
  if (session.status === 'disconnected') {
    return (
      <div className="h-full w-full relative flex flex-col">
        {/* Save connection banner for quick-connect sessions (D-06) */}
        {showSaveBanner && session.isQuickConnect && (
          <SaveConnectionBanner
            host={session.host}
            port={session.port}
            username={session.username}
            onDismiss={() => setShowSaveBanner(false)}
          />
        )}
        {/* Frozen terminal content underneath */}
        <div className="flex-1 relative">
          <Terminal
            ref={ref}
            autoResize
            cursorBlink={false}
            theme={terminalTheme}
            className={terminalClassName}
            onData={() => {}}
            onResize={sendResize}
            style={terminalStyle}
          />
          {/* Reconnect overlay on top (UI-04) */}
          <ReconnectOverlay
            host={session.host}
            onReconnect={handleRetry}
            onDismiss={handleReconnectDismiss}
          />
        </div>
      </div>
    )
  }

  return null
}
