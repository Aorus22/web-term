import { useCallback, useRef, useEffect } from 'react'
import { useTerminal } from '@wterm/react'
import { useAppStore } from '@/stores/app-store'
import type { ConnectOptions } from './types'

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

  const addSession = useAppStore((s) => s.addSession)
  const updateSession = useAppStore((s) => s.updateSession)

  // Cleanup WebSocket on unmount
  useEffect(() => {
    return () => {
      const ws = wsRef.current
      if (ws) {
        ws.onopen = null
        ws.onmessage = null
        ws.onclose = null
        ws.onerror = null
        ws.close()
        wsRef.current = null
      }
    }
  }, [])

  const connect = useCallback(
    (opts: ConnectOptions) => {
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
        ? { connectionId: opts.connectionId, host: opts.host, port: opts.port, username: opts.username, rows: opts.rows, cols: opts.cols }
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
      const wsUrl = import.meta.env.VITE_WS_URL || `ws://localhost:8080`
      const socket = new WebSocket(`${wsUrl}/ws`)
      wsRef.current = socket
      socket.binaryType = 'arraybuffer'

      socket.onopen = () => {
        // Send connect message based on options type
        const connectMsg = opts.connectionId
          ? JSON.stringify({
              type: 'connect',
              connection_id: opts.connectionId,
              rows,
              cols,
            })
          : JSON.stringify({
              type: 'connect',
              host: opts.host,
              port: opts.port ?? 22,
              user: opts.username,
              password: opts.password,
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
              updateSession(sessionId, {
                status: 'connected',
                host: host,
                label: `${username}@${host}`,
              })
              // Focus terminal after connection
              setTimeout(() => focus(), 100)
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
          write(data)
        }
      }

      socket.onclose = () => {
        wsRef.current = null
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

  const disconnect = useCallback(() => {
    const socket = wsRef.current
    if (socket) {
      socket.close()
      wsRef.current = null
    }
    updateSession(sessionId, { status: 'disconnected' })
  }, [sessionId, updateSession])

  return {
    ref,
    connect,
    sendData,
    sendResize,
    disconnect,
    write,
    focus,
  }
}
