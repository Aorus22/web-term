import * as React from 'react'
import { useConnections, useDeleteConnection, useCreateConnection } from '@/features/connections/hooks/useConnections'
import { useAppStore } from '@/stores/app-store'
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  MoreVertical, Edit2, Trash2, Copy, Plus, Server, Circle, ExternalLink 
} from 'lucide-react'
import { 
  DropdownMenu, 
  DropdownMenuContent, 
  DropdownMenuItem, 
  DropdownMenuSeparator, 
  DropdownMenuTrigger 
} from '@/components/ui/dropdown-menu'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { TagFilter } from '@/features/connections/components/TagFilter'
import { generateId } from '@/lib/utils'
import type { Connection } from '@/lib/api'
import type { SSHSession } from '@/features/terminal/types'

export const HostsPage = () => {
  const { data: connections = [], isLoading } = useConnections()
  const { 
    setCreatingConnection, 
    setEditingConnection,
    selectedTags,
    sessions, 
    addSession, 
    setActiveSession 
  } = useAppStore()
  
  const deleteMutation = useDeleteConnection()
  const createMutation = useCreateConnection()

  const [deleteId, setDeleteId] = React.useState<string | null>(null)
  const [deletingName, setDeletingName] = React.useState('')

  const filteredConnections = connections.filter((conn) => {
    if (selectedTags.length === 0) return true
    return selectedTags.every((tag) => conn.tags?.includes(tag))
  })

  const handleConnect = (conn: Connection) => {
    // Check if session already exists for this connection
    const existing = sessions.find((s) => s.connectionId === conn.id)
    if (existing) {
      setActiveSession(existing.id)
      return
    }
    
    // Create new session
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

  const handleDuplicate = (conn: Connection) => {
    // Remove ID and timestamps for duplication
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { id, created_at, updated_at, ...rest } = conn
    createMutation.mutate({
      ...rest,
      label: `${conn.label} (copy)`,
    })
  }

  const handleDelete = async () => {
    if (deleteId) {
      await deleteMutation.mutateAsync(deleteId)
      setDeleteId(null)
    }
  }

  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="flex flex-col items-center gap-2">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent"></div>
          <p className="text-sm text-muted-foreground font-medium">Loading connections...</p>
        </div>
      </div>
    )
  }

  if (!connections || connections.length === 0) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6 text-center">
        <div className="w-16 h-16 rounded-full bg-muted flex items-center justify-center mb-4">
          <Server className="w-8 h-8 text-muted-foreground/30" />
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
    <div className="flex-1 flex flex-col h-full overflow-hidden">
      <div className="flex flex-col border-b bg-muted/5">
        <div className="flex items-center justify-between px-6 py-4">
          <h2 className="text-xl font-semibold">Hosts</h2>
        </div>
        <div className="px-2">
          <TagFilter />
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {filteredConnections.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center text-center">
            <p className="text-muted-foreground">No connections match the selected tags.</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 pb-24">
            {filteredConnections.map((conn) => {
              const isActive = sessions.some(s => s.connectionId === conn.id)
              
              return (
                <Card 
                  key={conn.id} 
                  className={`relative cursor-pointer transition-all hover:shadow-md group flex flex-col ${
                    isActive ? 'border-l-4 border-l-primary shadow-sm' : ''
                  }`}
                  onClick={() => handleConnect(conn)}
                >
                  <div className="absolute top-2 right-2 flex items-center gap-1">
                    {isActive && (
                      <Circle className="h-2 w-2 fill-green-500 text-green-500 mr-1" />
                    )}
                    <DropdownMenu>
                      <DropdownMenuTrigger 
                        onClick={(e) => e.stopPropagation()}
                        className="h-8 w-8 flex items-center justify-center rounded-md opacity-0 group-hover:opacity-100 hover:bg-muted-foreground/10 transition-all outline-none"
                      >
                        <MoreVertical className="h-4 w-4" />
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={(e) => { e.stopPropagation(); handleConnect(conn) }}>
                          <ExternalLink className="mr-2 h-4 w-4" /> Connect
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={(e) => { e.stopPropagation(); setEditingConnection(conn) }}>
                          <Edit2 className="mr-2 h-4 w-4" /> Edit
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={(e) => { e.stopPropagation(); handleDuplicate(conn) }}>
                          <Copy className="mr-2 h-4 w-4" /> Duplicate
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem 
                          className="text-destructive focus:text-destructive"
                          onClick={(e) => {
                            e.stopPropagation()
                            setDeleteId(conn.id)
                            setDeletingName(conn.label)
                          }}
                        >
                          <Trash2 className="mr-2 h-4 w-4" /> Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <CardHeader className="p-4 pb-2">
                    <CardTitle className="text-base font-bold truncate pr-10">{conn.label}</CardTitle>
                    <CardDescription className="text-xs truncate">
                      {conn.username}@{conn.host}:{conn.port}
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="p-4 pt-0 flex-1">
                    <div className="flex flex-wrap gap-1.5 mt-2">
                      {conn.tags && conn.tags.length > 0 ? (
                        conn.tags.map((tag) => (
                          <Badge key={tag} variant="outline" className="text-[10px] px-1 py-0 h-4 leading-none font-normal">
                            {tag}
                          </Badge>
                        ))
                      ) : (
                        <span className="text-[10px] text-muted-foreground italic">No tags</span>
                      )}
                    </div>
                  </CardContent>
                </Card>
              )
            })}
          </div>
        )}
      </div>

      <Button
        className="fixed bottom-6 right-6 h-12 w-12 rounded-full shadow-lg z-10"
        size="icon"
        onClick={() => setCreatingConnection(true)}
      >
        <Plus className="h-6 w-6" />
      </Button>

      <AlertDialog open={!!deleteId} onOpenChange={(open) => !open && setDeleteId(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Connection</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete <strong>{deletingName}</strong>? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction 
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
