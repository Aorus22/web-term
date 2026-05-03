import { Terminal, Plus, Tag as TagIcon } from 'lucide-react'
import { useConnections } from '@/features/connections/hooks/useConnections'
import { QuickConnect } from '@/features/connections/components/QuickConnect'
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from '@/components/ui/card'
import { useAppStore } from '@/stores/app-store'
import { generateId } from '@/lib/utils'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { keysApi, type Connection } from '@/lib/api'
import type { SSHSession } from '@/features/terminal/types'

export const NewTabView = () => {
  const { data: connections = [] } = useConnections()
  const { setCreatingConnection, addSession, sessions } = useAppStore()

  const handleConnect = async (conn: Connection) => {
    // Create new SSH session (allow multiple connections to same host)
    if (conn.auth_method === 'key' && conn.ssh_key_id) {
      try {
        const key = await keysApi.get(conn.ssh_key_id)
        const session: SSHSession = {
          id: generateId(),
          type: 'ssh',
          connectionId: conn.id,
          host: conn.host,
          port: conn.port,
          username: conn.username,
          label: conn.label,
          status: key.has_passphrase ? 'needs-passphrase' : 'connecting',
          isQuickConnect: false,
          auth_method: 'key',
          ssh_key_id: conn.ssh_key_id,
          key_name: key.name,
          key_type: key.key_type,
          has_passphrase: key.has_passphrase,
        }
        addSession(session)
      } catch (error) {
        console.error('Failed to fetch key metadata:', error)
        const session: SSHSession = {
          id: generateId(),
          type: 'ssh',
          connectionId: conn.id,
          host: conn.host,
          port: conn.port,
          username: conn.username,
          label: conn.label,
          status: 'connecting',
          isQuickConnect: false,
          auth_method: 'key',
          ssh_key_id: conn.ssh_key_id,
        }
        addSession(session)
      }
    } else {
      const session: SSHSession = {
        id: generateId(),
        type: 'ssh',
        connectionId: conn.id,
        host: conn.host,
        port: conn.port,
        username: conn.username,
        label: conn.label,
        status: 'connecting',
        isQuickConnect: false,
      }
      addSession(session)
    }
  }

  const handleLocalConnect = () => {
    const session: SSHSession = {
      id: generateId(),
      type: 'local',
      host: 'local',
      port: 0,
      username: 'local',
      label: 'Local Terminal',
      status: 'connecting',
      isQuickConnect: false,
    }
    addSession(session)
  }

  return (
    <div className="flex flex-col items-center justify-start min-h-full p-8 overflow-y-auto pt-24">
      <div className="w-full max-w-4xl space-y-12 pb-12">
        {/* QuickConnect at top */}
        <div className="space-y-4">
          <div className="text-center space-y-2">
            <h2 className="text-3xl font-bold tracking-tight">Welcome to WebTerm</h2>
            <p className="text-muted-foreground">
              Connect to a server to get started
            </p>
          </div>
          <div className="max-w-xl mx-auto">
            <QuickConnect />
          </div>
        </div>

        {/* Local Terminal and Saved Connections */}
        <div className="space-y-6">
          <div className="space-y-4">
            <h3 className="text-xl font-semibold tracking-tight">Direct Access</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              <Card
                className="cursor-pointer transition-all hover:scale-[1.02] hover:shadow-md border-primary/20 bg-primary/5 hover:border-primary relative group"
                onClick={handleLocalConnect}
              >
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <CardTitle className="flex items-center gap-2">
                      <Terminal className="h-5 w-5 text-primary" />
                      Local Terminal
                    </CardTitle>
                  </div>
                  <CardDescription>
                    Spawn a shell on the backend host
                  </CardDescription>
                </CardHeader>
              </Card>
            </div>
          </div>

          <div className="flex items-center justify-between pt-4 border-t">
            <div className="space-y-1">
              <h3 className="text-xl font-semibold tracking-tight">Saved Connections</h3>
              <p className="text-sm text-muted-foreground">
                Your frequently used SSH configurations
              </p>
            </div>
            <Button variant="outline" size="sm" onClick={() => setCreatingConnection(true)}>
              <Plus className="h-4 w-4 mr-2" /> New Connection
            </Button>
          </div>

          {connections.length === 0 ? (
            <Card className="border-dashed flex flex-col items-center justify-center py-12 text-center">
              <Terminal className="h-12 w-12 text-muted-foreground/30 mb-4" />
              <p className="text-muted-foreground">
                No saved connections yet. Create one to get started.
              </p>
            </Card>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {connections.map((conn) => {
                const isActive = sessions.some(s => s.connectionId === conn.id)
                
                return (
                  <Card
                    key={conn.id}
                    className={`cursor-pointer transition-all hover:scale-[1.02] hover:shadow-md border-border/50 hover:border-primary/50 relative group ${
                      isActive ? 'ring-2 ring-primary ring-offset-2 bg-primary/5' : ''
                    }`}
                    onClick={() => handleConnect(conn)}
                  >
                    <CardHeader className="pb-2">
                      <div className="flex items-center justify-between">
                        <CardTitle className="truncate pr-8">{conn.label || conn.host}</CardTitle>
                        <div className="flex items-center gap-2 absolute top-4 right-4">
                          {isActive && (
                            <div className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
                          )}
                          <Terminal className="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
                        </div>
                      </div>
                      <CardDescription className="truncate">
                        {conn.username}@{conn.host}
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="flex flex-wrap gap-1.5 min-h-[1.5rem]">
                        {conn.tags && conn.tags.length > 0 ? (
                          conn.tags.map((tag) => (
                            <Badge
                              key={tag}
                              variant="outline"
                              className="text-[10px] px-1.5 py-0 h-4 font-normal"
                            >
                              {tag}
                            </Badge>
                          ))
                        ) : (
                          <span className="text-[10px] text-muted-foreground italic flex items-center">
                            <TagIcon className="h-3 w-3 mr-1 opacity-50" /> No tags
                          </span>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                )
              })}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
