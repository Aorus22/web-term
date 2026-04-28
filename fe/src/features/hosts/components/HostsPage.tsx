import { useConnections } from '@/features/connections/hooks/useConnections'
import { useAppStore } from '@/stores/app-store'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Terminal, Plus } from 'lucide-react'
import { generateId } from '@/lib/utils'
import type { SSHSession } from '@/features/terminal/types'

export const HostsPage = () => {
  const { data: connections, isLoading } = useConnections()
  const { setCreatingConnection, addSession } = useAppStore()

  const handleConnect = (conn: any) => {
    const session: SSHSession = {
      id: generateId(),
      connectionId: conn.id,
      host: conn.host,
      port: conn.port,
      username: conn.username,
      label: conn.label,
      status: 'connecting',
      isQuickConnect: false
    }
    addSession(session)
  }

  if (isLoading) {
    return <div className="flex-1 flex items-center justify-center">Loading connections...</div>
  }

  if (!connections || connections.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6 text-center">
        <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-4">
          <Terminal className="w-8 h-8 text-muted-foreground" />
        </div>
        <h3 className="text-xl font-semibold mb-2">No connections yet</h3>
        <p className="text-muted-foreground mb-6 max-w-sm">
          Create your first connection to start using WebTerm.
        </p>
        <Button onClick={() => setCreatingConnection(true)}>
          <Plus className="mr-2 h-4 w-4" /> Create Connection
        </Button>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 p-6">
        {connections.map((conn) => (
          <Card 
            key={conn.id} 
            className="cursor-pointer hover:bg-accent/50 transition-colors group"
            onClick={() => handleConnect(conn)}
          >
            <CardHeader className="p-4 pb-2">
              <CardTitle className="text-base truncate">{conn.label}</CardTitle>
              <CardDescription className="text-xs truncate">
                {conn.username}@{conn.host}:{conn.port}
              </CardDescription>
            </CardHeader>
            <CardContent className="p-4 pt-0">
              <div className="flex flex-wrap gap-1 mt-2">
                {conn.tags.map((tag) => (
                  <Badge key={tag} variant="secondary" className="text-[10px] px-1 py-0 h-4">
                    {tag}
                  </Badge>
                ))}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
      <Button
        className="fixed bottom-6 right-6 h-12 w-12 rounded-full shadow-lg"
        size="icon"
        onClick={() => setCreatingConnection(true)}
      >
        <Plus className="h-6 w-6" />
      </Button>
    </div>
  )
}
