import * as React from 'react'
import { useConnections, useDeleteConnection, useCreateConnection } from '../hooks/useConnections'
import { useAppStore } from '@/stores/app-store'
import { Badge } from '@/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
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
import { MoreVertical, Edit2, Trash2, Copy, ExternalLink, Terminal } from 'lucide-react'
import type { Connection } from '@/lib/api'
import type { SSHSession } from '@/features/terminal/types'

export const ConnectionList = () => {
  const { data: connections = [], isLoading } = useConnections()
  const { selectedTags, setEditingConnection, sessions, activeSessionId, addSession, setActiveSession } = useAppStore()
  const deleteMutation = useDeleteConnection()
  const createMutation = useCreateConnection()
  
  const [deleteId, setDeleteId] = React.useState<string | null>(null)
  const [deletingName, setDeletingName] = React.useState('')

  const filteredConnections = connections.filter((conn) => {
    if (selectedTags.length === 0) return true
    return selectedTags.every((tag) => conn.tags?.includes(tag))
  })

  const handleDelete = async () => {
    if (deleteId) {
      await deleteMutation.mutateAsync(deleteId)
      setDeleteId(null)
    }
  }

  const handleDuplicate = (conn: Connection) => {
    const { id, created_at, updated_at, ...rest } = conn
    createMutation.mutate({
      ...rest,
      label: `${conn.label} (copy)`,
    })
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
  }

  if (isLoading) return <div className="px-4 py-2 text-sm text-muted-foreground animate-pulse">Loading connections...</div>

  if (connections.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-10 px-4 text-center">
        <Terminal className="h-8 w-8 text-muted-foreground/50 mb-3" />
        <p className="text-sm text-muted-foreground">No connections saved yet.</p>
      </div>
    )
  }

  if (filteredConnections.length === 0) {
    return (
      <div className="px-4 py-6 text-center">
        <p className="text-xs text-muted-foreground">No connections match the selected tags.</p>
      </div>
    )
  }

  return (
    <div className="flex-1 px-2 pb-4">
      <div className="space-y-1">
        {filteredConnections.map((conn) => (
          <div
            key={conn.id}
            className="group flex items-center justify-between p-2 rounded-md hover:bg-accent hover:text-accent-foreground cursor-pointer transition-colors border border-transparent hover:border-border"
            onClick={() => {
              // Check if a session already exists for this connection (D-01)
              const existing = sessions.find((s) => s.connectionId === conn.id)
              if (existing) {
                setActiveSession(existing.id)
                return
              }
              // Create new session
              const sessionId = crypto.randomUUID()
              const session: SSHSession = {
                id: sessionId,
                connectionId: conn.id,
                host: conn.host,
                port: conn.port,
                username: conn.username,
                label: conn.label,
                status: 'connecting',
                isQuickConnect: false,
              }
              addSession(session)
            }}
          >
            <div className="flex flex-col min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium truncate">{conn.label}</span>
                {conn.tags?.map((tag) => (
                  <Badge key={tag} variant="secondary" className="text-[10px] px-1 py-0 h-4 leading-none bg-muted/50 border-none group-hover:bg-muted">
                    {tag}
                  </Badge>
                ))}
              </div>
              <span className="text-xs text-muted-foreground truncate">
                {conn.username}@{conn.host}:{conn.port}
              </span>
            </div>

            <DropdownMenu>
              <DropdownMenuTrigger 
                onClick={(e) => e.stopPropagation()} 
                className="h-8 w-8 flex items-center justify-center rounded-md opacity-0 group-hover:opacity-100 hover:bg-muted-foreground/10 transition-all outline-none"
              >
                <MoreVertical className="h-4 w-4" />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-48">
                <DropdownMenuItem onClick={() => setEditingConnection(conn)}>
                  <Edit2 className="mr-2 h-4 w-4" /> Edit
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleDuplicate(conn)}>
                  <Copy className="mr-2 h-4 w-4" /> Duplicate
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => copyToClipboard(`${conn.host}:${conn.port}`)}>
                  <ExternalLink className="mr-2 h-4 w-4" /> Copy host:port
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem 
                  className="text-destructive focus:text-destructive"
                  onClick={() => {
                    setDeleteId(conn.id)
                    setDeletingName(conn.label)
                  }}
                >
                  <Trash2 className="mr-2 h-4 w-4" /> Delete
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        ))}
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
