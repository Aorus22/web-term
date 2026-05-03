import { useCallback, useRef, useEffect } from 'react'
import { useTerminal } from '@wterm/react'
import { useAppStore } from '@/stores/app-store'
import { isDesktop } from '@/lib/desktop-ipc'
import type { ConnectOptions } from './types'

// Global map for WebSocket references so TabBar can access them for get-cwd
const wsMap = new Map<string, WebSocket>()

export function getSessionWebSocket(sessionId: string): WebSocket | undefined {
  return wsMap.get(sessionId)
}

const getWsBaseUrl = () => {
  // Mode 1: Desktop (Tauri/Electron)
  const bPort = useAppStore.getState().backendPort
  if (isDesktop && bPort !== 0) {
    return `ws://localhost:${bPort}`
  }

  // Mode 2: Decoupled (VITE_WS_URL)
  if (import.meta.env.VITE_WS_URL) {
    return import.meta.env.VITE_WS_URL
  }

  // Mode 3: Unified (Same Host)
  if (typeof window !== 'undefined' && window.location.host) {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${protocol}//${window.location.host}`
  }

  // Absolute fallback
  return 'ws://localhost:8080'
}

/**
 * Hook that encapsulates the full WebSocket lifecycle for a single SSH session.
 *
 * Manages: connect, send data, receive data, resize, disconnect.
 * Integrates with @wterm/react terminal and Zustand store for session state.
 */
export function useSSHSession(sessionId: string) {
  const { ref, write, focus } = useTerminal()
  const wsRef = useRef<WebSocket | null>(null)
  const lastOptionsRef = useRef<ConnectOptions | null>(null)
  const binaryBufferRef = useRef<Uint8Array[]>([])

  const addSession = useAppStore((s) => s.addSession)
  const updateSession = useAppStore((s) => s.updateSession)
  const status = useAppStore((s) => s.sessions.find((s) => s.id === sessionId)?.status)

  // Flush buffer when terminal is ready
  useEffect(() => {
    if (status === 'connected' && binaryBufferRef.current.length > 0) {
      // Delay to ensure terminal instance is fully ready to render
      const timer = setTimeout(() => {
        // Re-check status inside timer
        const currentStatus = useAppStore.getState().sessions.find((s) => s.id === sessionId)?.status
        if (currentStatus === 'connected' && binaryBufferRef.current.length > 0) {
          console.log(`[WS:${sessionId}] Flushing ${binaryBufferRef.current.length} buffered binary chunks to terminal`)
          binaryBufferRef.current.forEach(chunk => write(chunk))
          binaryBufferRef.current = []
        }
      }, 500)
      return () => clearTimeout(timer)
    }
  }, [sessionId, write, status])

  // Cleanup WebSocket on unmount
  useEffect(() => {
    return () => {
      const ws = wsRef.current
      if (ws) {
        ws.onopen = null
        ws.onmessage = null
        ws.onclose = null
        ws.onerror = null
        // Don't explicitly disconnect here as it might be a reload or navigation.
        // Explicit disconnect is handled by disconnect() function.
        ws.close()
        wsRef.current = null
      }
      wsMap.delete(sessionId)
    }
  }, [sessionId])

  const connect = useCallback(
    (opts: ConnectOptions) => {
      binaryBufferRef.current = [] // reset buffer
      
      // Close any existing connection for this session
      const existing = wsRef.current
      if (existing) {
        existing.onopen = null
        existing.onmessage = null
        existing.onclose = null
        existing.onerror = null
        existing.close()
        wsRef.current = null
      }

      // Store options for retry/reconnect (strip password per D-07 for saved connections)
      lastOptionsRef.current = opts.connectionId
        ? { 
            connectionId: opts.connectionId, 
            host: opts.host, 
            port: opts.port, 
            username: opts.username, 
            cwd: opts.cwd,
            rows: opts.rows, 
            cols: opts.cols,
            auth_method: opts.auth_method,
            ssh_key_id: opts.ssh_key_id,
            passphrase: opts.passphrase,
            term: opts.term
          }
        : opts

      const host = opts.host ?? ''
      const port = opts.port ?? 22
      const username = opts.username ?? ''
      const rows = opts.rows ?? 24
      const cols = opts.cols ?? 80
      const isQuickConnect = !opts.connectionId
      const label = opts.connectionId ? (host || 'Connecting...') : `${username}@${host}`

      // Update or add session to store with connecting status
      const found = useAppStore.getState().sessions.find((s) => s.id === sessionId)
      if (found) {
        updateSession(sessionId, { status: 'connecting' })
      } else {
        addSession({
          id: sessionId,
          connectionId: opts.connectionId,
          host,
          port,
          username,
          label,
          status: 'connecting',
          isQuickConnect,
        })
      }

      // Create WebSocket connection
      const wsBase = getWsBaseUrl()
      
      const socket = new WebSocket(`${wsBase}/ws`)
      wsRef.current = socket
      wsMap.set(sessionId, socket)
      socket.binaryType = 'arraybuffer'

      socket.onopen = () => {
        // Send connect message based on options type
        const connectMsg = opts.connectionId
          ? JSON.stringify({
              type: 'connect',
              connection_id: opts.connectionId,
              auth_method: opts.auth_method || 'password',
              ...(opts.ssh_key_id && { ssh_key_id: opts.ssh_key_id }),
              ...(opts.passphrase && { passphrase: opts.passphrase }),
              ...(opts.cwd && { cwd: opts.cwd }),
              ...(opts.term && { term: opts.term }),
              rows,
              cols,
            })
          : JSON.stringify({
              type: 'connect',
              host: opts.host,
              port: opts.port ?? 22,
              user: opts.username,
              password: opts.password,
              ...(opts.cwd && { cwd: opts.cwd }),
              ...(opts.term && { term: opts.term }),
              rows,
              cols,
            })
        socket.send(connectMsg)
      }

      socket.onmessage = (event: MessageEvent) => {
        if (typeof event.data === 'string') {
          // JSON control message from server
          try {
            const msg = JSON.parse(event.data)
            if (msg.type === 'connected') {
              const bId = msg.session_id || msg.sessionId;
              if (bId) {
                updateSession(sessionId, { 
                  backendId: bId,
                  status: 'connected',
                  host: host,
                  label: `${username}@${host}`,
                })
              } else {
                updateSession(sessionId, {
                  status: 'connected',
                  host: host,
                  label: `${username}@${host}`,
                })
              }
              
              setTimeout(() => {
                focus()
                // 2. Refresh CWD
                getCwd().then(path => updateSession(sessionId, { cwd: path })).catch(() => {})
              }, 200)
            } else if (msg.type === 'error') {
              updateSession(sessionId, {
                status: 'error',
                error: msg.message || 'Connection error',
              })
              socket.close()
              wsRef.current = null
            } else if (msg.type === 'disconnected') {
              updateSession(sessionId, { status: 'disconnected' })
            }
          } catch {
            // Ignore malformed JSON
          }
        } else {
          // Binary frame — SSH output data
          const data =
            event.data instanceof ArrayBuffer
              ? new Uint8Array(event.data)
              : event.data
          
          const session = useAppStore.getState().sessions.find((s) => s.id === sessionId)
          if (session?.status === 'connected') {
            // console.log(`[WS:${sessionId}] Binary data received, writing to terminal (${data.length} bytes)`)
            write(data)
          } else {
            console.log(`[WS:${sessionId}] Binary data received while status is ${session?.status}, buffering (${data.length} bytes)`)
            binaryBufferRef.current.push(data)
          }
        }
      }

      socket.onclose = () => {
        wsRef.current = null
        wsMap.delete(sessionId)
        // Only update if not already in error or disconnected state
        const session = useAppStore.getState().sessions.find((s) => s.id === sessionId)
        if (session && session.status !== 'error' && session.status !== 'disconnected') {
          updateSession(sessionId, { status: 'disconnected' })
        }
      }

      socket.onerror = () => {
        wsRef.current = null
        updateSession(sessionId, {
          status: 'error',
          error: 'WebSocket connection failed',
        })
      }
    },
    [sessionId, addSession, updateSession, write, focus],
  )

  const sendData = useCallback((data: string) => {
    const socket = wsRef.current
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(new TextEncoder().encode(data))
    }
  }, [])

  const sendResize = useCallback((cols: number, rows: number) => {
    const socket = wsRef.current
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({ type: 'resize', cols, rows }))
    }
  }, [])

  const signalReady = useCallback(() => {
    const socket = wsRef.current
    if (socket && socket.readyState === WebSocket.OPEN) {
      console.log(`[WS:${sessionId}] Sending 'ready' signal (terminal initialized)`)
      socket.send(JSON.stringify({ type: 'ready' }))
    }
  }, [sessionId])

  const disconnect = useCallback(() => {
    const socket = wsRef.current
    if (socket) {
      if (socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({ type: 'disconnect' }))
      }
      socket.close()
      wsRef.current = null
    }
    wsMap.delete(sessionId)
    updateSession(sessionId, { status: 'disconnected' })
  }, [sessionId, updateSession])

  const attach = useCallback(async () => {
    binaryBufferRef.current = [] // reset buffer
    
    // Close any existing connection for this session
    const existing = wsRef.current
    if (existing) {
      existing.onopen = null
      existing.onmessage = null
      existing.onclose = null
      existing.onerror = null
      existing.close()
      wsRef.current = null
    }

    updateSession(sessionId, { status: 'connecting' })

    const wsBase = getWsBaseUrl()

    const connectWithRetry = (retryCount = 0) => {
      console.log(`[WS-Attach] Connecting to ${wsBase}/ws (attempt ${retryCount + 1})...`)
      const socket = new WebSocket(`${wsBase}/ws`)
      wsRef.current = socket
      wsMap.set(sessionId, socket)
      socket.binaryType = 'arraybuffer'

      socket.onopen = () => {
        const session = useAppStore.getState().sessions.find((s) => s.id === sessionId)
        const bId = session?.backendId || sessionId
        console.log(`[WS-Attach] WebSocket opened, sending attach message for session: ${bId}`)
        socket.send(
          JSON.stringify({
            type: 'attach',
            session_id: bId,
          })
        )
      }

      socket.onmessage = (event: MessageEvent) => {
        if (typeof event.data === 'string') {
          try {
            const msg = JSON.parse(event.data)
            if (msg.type === 'connected') {
              console.log(`[WS-Attach:${sessionId}] Successfully attached to session`)
              updateSession(sessionId, { status: 'connected' })
              setTimeout(() => {
                focus()
                // 2. Refresh CWD after attachment
                getCwd().then(path => updateSession(sessionId, { cwd: path })).catch(() => {})
              }, 200)
            } else if (msg.type === 'error') {
              console.error(`[WS-Attach:${sessionId}] Received error:`, msg.message)
              updateSession(sessionId, {
                status: 'error',
                error: msg.message || 'Attach failed',
              })
              socket.close()
            } else if (msg.type === 'disconnected') {
              console.log(`[WS-Attach:${sessionId}] Received disconnected message`)
              updateSession(sessionId, { status: 'disconnected' })
            }
          } catch {
            // ignore
          }
        } else {
          // Binary frame — SSH output data
          const data =
            event.data instanceof ArrayBuffer
              ? new Uint8Array(event.data)
              : event.data
          
          const session = useAppStore.getState().sessions.find((s) => s.id === sessionId)
          if (session?.status === 'connected') {
            write(data)
          } else {
            binaryBufferRef.current.push(data)
          }
        }
      }

      socket.onclose = () => {
        console.log(`[WS-Attach] WebSocket closed (sessionId: ${sessionId}, retryCount: ${retryCount})`)
        wsRef.current = null
        wsMap.delete(sessionId)
        
        const session = useAppStore.getState().sessions.find((s) => s.id === sessionId)
        if (session && session.status === 'connecting' && retryCount < 3) {
          console.warn(`[WS-Attach] Unexpected close during attachment, retrying in 2s...`)
          setTimeout(() => connectWithRetry(retryCount + 1), 2000)
        } else if (session && session.status !== 'error' && session.status !== 'disconnected' && session.status !== 'connecting') {
          updateSession(sessionId, { status: 'disconnected' })
        }
      }

      socket.onerror = (err) => {
        console.error(`[WS-Attach] WebSocket error:`, err)
        wsRef.current = null
      }
    }

    connectWithRetry()
  }, [sessionId, updateSession, write, focus])

  const getCwd = useCallback((): Promise<string> => {
    return new Promise((resolve, reject) => {
      const socket = wsRef.current
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        reject(new Error('Not connected'))
        return
      }

      const timeout = setTimeout(() => {
        socket.removeEventListener('message', handler)
        reject(new Error('get-cwd timeout'))
      }, 5000)

      const handler = (event: MessageEvent) => {
        if (typeof event.data === 'string') {
          try {
            const msg = JSON.parse(event.data)
            if (msg.type === 'cwd') {
              clearTimeout(timeout)
              socket.removeEventListener('message', handler)
              resolve(msg.path || '')
              return
            }
          } catch {
            // Not JSON, ignore
          }
        }
      }

      socket.addEventListener('message', handler)
      socket.send(JSON.stringify({ type: 'get-cwd' }))
    })
  }, [])

  return {
    ref,
    connect,
    attach,
    sendData,
    sendResize,
    disconnect,
    write,
    focus,
    getCwd,
    signalReady,
  }
}
