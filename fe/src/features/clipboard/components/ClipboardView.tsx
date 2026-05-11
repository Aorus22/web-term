import { useState, useEffect, useCallback } from 'react'
import { useAppStore } from '@/stores/app-store'
import { connectionsApi, type Connection, clipboardWS, type ClipboardMessage } from '@/lib/api'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Trash2, Copy, Search, Clipboard, Wifi, WifiOff, Loader2, RefreshCw } from 'lucide-react'
import { generateId } from '@/lib/utils'

interface ConnectedSession {
  connectionId: string
  connectionLabel: string
  os: string
}

export function ClipboardView() {
  const { clipboardHistory, clearClipboardHistory, addClipboardEntry } = useAppStore()
  const [searchQuery, setSearchQuery] = useState('')
  const [connections, setConnections] = useState<Connection[]>([])
  const [selectedConnectionId, setSelectedConnectionId] = useState<string>('')
  const [isConnecting, setIsConnecting] = useState(false)
  const [connectedSession, setConnectedSession] = useState<ConnectedSession | null>(null)
  const [connectionError, setConnectionError] = useState<string | null>(null)
  const [isSyncing, setIsSyncing] = useState(false)

  useEffect(() => {
    connectionsApi.list().then(setConnections).catch(console.error)
  }, [])

  useEffect(() => {
    const unsubscribe = clipboardWS.onMessage((msg: ClipboardMessage) => {
      if (msg.type === 'clipboard_update' && msg.content) {
        addClipboardEntry({
          id: generateId(),
          content: msg.content,
          timestamp: Date.now(),
          sessionId: connectedSession?.connectionId || '',
        })
      } else if (msg.type === 'error') {
        setConnectionError(msg.message || 'Connection error')
        setConnectedSession(null)
      } else if (msg.type === 'connected') {
        const conn = connections.find((c) => c.id === selectedConnectionId)
        // @ts-ignore - os is added in backend response
        const os = msg.os || 'Unknown'
        setConnectedSession({
          connectionId: selectedConnectionId,
          connectionLabel: conn?.label || 'Unknown',
          os,
        })
        setConnectionError(null)
        setIsConnecting(false)
      }
    })

    return () => {
      unsubscribe()
    }
  }, [addClipboardEntry, connectedSession, connections, selectedConnectionId])

  const handleConnect = useCallback(async () => {
    if (!selectedConnectionId) return

    setIsConnecting(true)
    setConnectionError(null)

    try {
      await clipboardWS.connect(selectedConnectionId)
    } catch (err) {
      setConnectionError(err instanceof Error ? err.message : 'Failed to connect')
      setIsConnecting(false)
    }
  }, [selectedConnectionId])

  const handleDisconnect = useCallback(() => {
    clipboardWS.disconnect()
    setConnectedSession(null)
    setSelectedConnectionId('')
  }, [])

  const handleSyncFromRemote = useCallback(async () => {
    if (!connectedSession) return
    setIsSyncing(true)
    clipboardWS.send({ type: 'get_clipboard' })
    setTimeout(() => setIsSyncing(false), 2000)
  }, [connectedSession])

  const filteredHistory = searchQuery
    ? clipboardHistory.filter((entry) =>
        entry.content.toLowerCase().includes(searchQuery.toLowerCase())
      )
    : clipboardHistory

  const copyToLocal = async (content: string) => {
    try {
      await navigator.clipboard.writeText(content)
    } catch (err) {
      console.error('Failed to copy to local clipboard:', err)
    }
  }

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  const truncateContent = (content: string, maxLength: number = 150) => {
    if (content.length <= maxLength) return content
    return content.substring(0, maxLength) + '...'
  }

  return (
    <div className="flex flex-col h-full p-4 gap-4">
      <div className="flex items-center gap-2">
        <Clipboard className="h-5 w-5" />
        <h2 className="text-lg font-semibold">Clipboard</h2>
      </div>

      <div className="flex flex-col gap-2">
        <div className="flex gap-2 items-center">
          <select
            className="flex h-9 w-full rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
            value={selectedConnectionId}
            onChange={(e) => setSelectedConnectionId(e.target.value)}
            disabled={isConnecting || connectedSession !== null}
          >
            <option value="">Select a connection...</option>
            {connections.map((conn) => (
              <option key={conn.id} value={conn.id}>
                {conn.label} ({conn.username}@{conn.host})
              </option>
            ))}
          </select>

          {connectedSession ? (
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleSyncFromRemote}
                disabled={isSyncing}
                title="Sync clipboard from remote"
              >
                {isSyncing ? (
                  <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                ) : (
                  <RefreshCw className="h-4 w-4 mr-1" />
                )}
                Sync
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleDisconnect}
                className="text-red-500 hover:text-red-600"
              >
                <WifiOff className="h-4 w-4 mr-1" />
                Disconnect
              </Button>
            </div>
          ) : (
            <Button
              size="sm"
              onClick={handleConnect}
              disabled={!selectedConnectionId || isConnecting}
            >
              {isConnecting ? (
                <>
                  <Loader2 className="h-4 w-4 mr-1 animate-spin" />
                  Connecting...
                </>
              ) : (
                <>
                  <Wifi className="h-4 w-4 mr-1" />
                  Connect
                </>
              )}
            </Button>
          )}
        </div>

        {connectedSession && (
          <div className="text-sm text-green-600 flex items-center gap-2">
            <Wifi className="h-3 w-3" />
            Connected to: {connectedSession.connectionLabel} ({connectedSession.os})
          </div>
        )}

        {connectionError && (
          <div className="text-sm text-red-500">{connectionError}</div>
        )}
      </div>

      <div className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search clipboard history..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
        <Button
          variant="outline"
          size="icon"
          onClick={clearClipboardHistory}
          title="Clear history"
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="space-y-2">
          {filteredHistory.length === 0 ? (
            <div className="text-center text-muted-foreground py-8">
              <Clipboard className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>No clipboard history yet</p>
              <p className="text-sm">
                {connectedSession
                  ? 'Clipboard entries from remote session will appear here'
                  : 'Select a connection above to start monitoring clipboard'}
              </p>
            </div>
          ) : (
            filteredHistory.map((entry) => (
              <Card key={entry.id} className="cursor-pointer hover:bg-accent/50">
                <CardContent className="p-3">
                  <div className="flex items-start justify-between gap-2">
                    <div className="flex-1 min-w-0">
                      <pre className="text-sm whitespace-pre-wrap break-all font-mono">
                        {truncateContent(entry.content)}
                      </pre>
                      <p className="text-xs text-muted-foreground mt-1">
                        {formatTime(entry.timestamp)}
                        {entry.sessionId && ` • ${connections.find((c) => c.id === entry.sessionId)?.label || entry.sessionId}`}
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => copyToLocal(entry.content)}
                      title="Copy to local clipboard"
                    >
                      <Copy className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))
          )}
        </div>
      </ScrollArea>
    </div>
  )
}