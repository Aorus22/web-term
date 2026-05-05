import * as React from 'react'
import { useConnections, useDeleteConnection, useCreateConnection } from '@/features/connections/hooks/useConnections'
import { useAppStore } from '@/stores/app-store'
import { Card } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { 
  MoreVertical, Edit2, Trash2, Copy, Plus, Server, ExternalLink
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
import { ExportImport } from '@/features/connections/components/ExportImport'
import { generateId } from '@/lib/utils'
import { keysApi, type Connection } from '@/lib/api'
import type { SSHSession } from '@/features/terminal/types'

export const HostsPage = () => {
  const { data: connections = [] } = useConnections()
  const { 
    setEditingConnection,
    selectedTags,
    sessions, 
    addSession 
  } = useAppStore()
  
  const deleteMutation = useDeleteConnection()
  const createMutation = useCreateConnection()

  const [deleteId, setDeleteId] = React.useState<string | null>(null)
  const [deletingName, setDeletingName] = React.useState('')

  const filteredConnections = connections.filter((conn) => {
    if (selectedTags.length === 0) return true
    return selectedTags.every((tag) => conn.tags?.includes(tag))
  })

  const handleConnect = async (conn: Connection) => {
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
        isQuickConnect: false
      }
      addSession(session)
    }
  }

  const handleDuplicate = (conn: Connection) => {
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

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Termius-style Header */}
      <header className="flex items-center justify-between px-6 py-2 border-b bg-muted/5">
        <div className="flex items-center gap-4">
          <h1 className="text-3xl font-bold tracking-tight">Hosts</h1>
          <div className="h-6 w-[1px] bg-border" />
          <TagFilter />
        </div>
        <div className="flex items-center gap-2">
          <ExportImport />
          <Button size="sm" onClick={() => useAppStore.getState().setCreatingConnection(true)} className="h-9 shadow-sm px-4">
            <Plus className="mr-2 h-4 w-4" /> New Host
          </Button>
        </div>
      </header>

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-[1600px] mx-auto">
          {filteredConnections.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-32 text-center border-2 border-dashed rounded-lg bg-muted/10">
              <Server className="h-10 w-10 text-muted-foreground/20 mb-3" />
              <p className="text-sm text-muted-foreground">
                {connections.length === 0 ? "No hosts saved yet." : "No hosts match your filters."}
              </p>
              {connections.length === 0 ? (
                <Button variant="link" size="sm" onClick={() => useAppStore.getState().setCreatingConnection(true)}>
                  Create your first host
                </Button>
              ) : (
                <Button variant="link" size="sm" onClick={() => useAppStore.getState().clearTags()}>
                  Clear all filters
                </Button>
              )}
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {filteredConnections.map((conn) => {
                const connSessions = sessions.filter(s => s.connectionId === conn.id)
                const sessionCount = connSessions.length
                const isActive = sessionCount > 0
                
                return (
                  <Card 
                    key={conn.id} 
                    className={`group relative cursor-pointer transition-all border-border/40 hover:border-primary/40 hover:shadow-md overflow-hidden py-0 gap-0 ${
                      isActive ? 'bg-primary/5 border-primary/30' : ''
                    }`}
                    onClick={() => handleConnect(conn)}
                  >
                    <div className="flex items-center p-4 gap-4">
                      {/* Left: Icon */}
                      <div className={`flex-shrink-0 w-8 h-8 rounded-md flex items-center justify-center ${
                        isActive ? 'bg-primary/10' : 'bg-muted/50'
                      }`}>
                        <Server className={`h-4 w-4 ${isActive ? 'text-primary' : 'text-muted-foreground/70'}`} />
                      </div>

                      {/* Middle: Info */}
                      <div className="flex-1 min-w-0 py-0">
                        <h3 className="text-base font-bold truncate leading-tight">
                          {conn.label || conn.host}
                        </h3>
                        <p className="text-xs text-muted-foreground truncate leading-none mt-1">
                          {conn.username}@{conn.host}
                        </p>
                        {conn.tags && conn.tags.length > 0 && (
                          <div className="flex flex-wrap gap-1 mt-1.5">
                            {conn.tags.map((tag) => (
                              <Badge key={tag} variant="secondary" className="text-[10px] px-1.5 py-0 h-4 font-normal bg-muted border-none leading-none">
                                {tag}
                              </Badge>
                            ))}
                          </div>
                        )}
                      </div>

                      {/* Right: Actions */}
                      <div className="flex items-center gap-1 pr-2 flex-shrink-0">
                        {isActive && (
                          <div className="w-5 h-5 rounded-full bg-primary flex items-center justify-center shadow-sm">
                            <span className="text-[9px] font-bold text-primary-foreground leading-none">
                              {sessionCount}
                            </span>
                          </div>
                        )}
                        <DropdownMenu>
                          <DropdownMenuTrigger 
                            onClick={(e) => e.stopPropagation()}
                            className="h-7 w-7 flex items-center justify-center rounded-md opacity-0 group-hover:opacity-100 hover:bg-muted transition-all outline-none"
                          >
                            <MoreVertical className="h-4 w-4 text-muted-foreground" />
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end" className="w-40">
                            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); handleConnect(conn) }}>
                              <ExternalLink className="mr-2 h-3.5 w-3.5" /> Connect
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); setEditingConnection(conn) }}>
                              <Edit2 className="mr-2 h-3.5 w-3.5" /> Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); handleDuplicate(conn) }}>
                              <Copy className="mr-2 h-3.5 w-3.5" /> Duplicate
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem 
                              className="text-destructive focus:bg-destructive/10"
                              onClick={(e) => {
                                e.stopPropagation()
                                setDeleteId(conn.id)
                                setDeletingName(conn.label)
                              }}
                            >
                              <Trash2 className="mr-2 h-3.5 w-3.5" /> Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>
                    </div>
                  </Card>
                )
              })}
            </div>
          )}
        </div>
      </div>

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
