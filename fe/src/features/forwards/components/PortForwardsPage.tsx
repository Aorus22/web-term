import * as React from 'react'
import { Plus, ArrowLeftRight, Loader2, MoreVertical, Trash2, Edit2 } from 'lucide-react'
import { toast } from 'sonner'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
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
import {
  useForwards,
  useDeleteForward,
  useStartForward,
  useStopForward,
} from '../hooks/useForwards'
import { useConnections } from '@/features/connections/hooks/useConnections'
import { ForwardFormSheet } from './ForwardFormSheet'
import type { PortForward } from '@/lib/api'

export const PortForwardsPage = () => {
  const { data: forwards = [], isLoading } = useForwards()
  const { data: connections = [] } = useConnections()
  const deleteMutation = useDeleteForward()
  const startMutation = useStartForward()
  const stopMutation = useStopForward()

  const [formOpen, setFormOpen] = React.useState(false)
  const [editForward, setEditForward] = React.useState<PortForward | null>(null)
  const [deleteTarget, setDeleteTarget] = React.useState<PortForward | null>(null)

  // Build connection lookup map
  const connectionMap = React.useMemo(() => {
    const map = new Map<string, string>()
    for (const conn of connections) {
      map.set(conn.id, conn.label)
    }
    return map
  }, [connections])

  const handleToggle = async (forward: PortForward) => {
    try {
      if (forward.active) {
        await stopMutation.mutateAsync(forward.id)
      } else {
        await startMutation.mutateAsync(forward.id)
      }
    } catch (err: any) {
      toast.error(forward.active ? 'Failed to stop forward' : 'Failed to start forward', {
        description: err?.error || err?.message || 'Unknown error',
      })
    }
  }

  const handleDelete = async () => {
    if (!deleteTarget) return
    try {
      await deleteMutation.mutateAsync(deleteTarget.id)
      setDeleteTarget(null)
    } catch (err) {
      console.error('Delete failed:', err)
    }
  }

  if (isLoading) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6 text-center">
        <Loader2 className="h-8 w-8 text-muted-foreground animate-spin mb-4" />
        <p className="text-muted-foreground">Loading port forwards...</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-2 border-b bg-muted/5">
        <div className="flex items-center gap-4">
          <h1 className="text-lg font-semibold tracking-tight">Port Forwards</h1>
          <p className="text-xs text-muted-foreground uppercase tracking-wider font-bold">
            SSH Tunneling
          </p>
        </div>
        <Button size="sm" onClick={() => setFormOpen(true)} className="h-9 shadow-sm px-4">
          <Plus className="mr-2 h-4 w-4" /> Create Forward
        </Button>
      </header>

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-[1600px] mx-auto">
          {forwards.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-32 text-center border-2 border-dashed rounded-lg bg-muted/10">
              <ArrowLeftRight className="h-10 w-10 text-muted-foreground/20 mb-3" />
              <p className="text-sm text-muted-foreground">No port forwards yet.</p>
              <Button variant="link" size="sm" onClick={() => setFormOpen(true)}>
                Create your first forward
              </Button>
            </div>
          ) : (
            <div className="flex flex-col gap-3">
              {forwards.map((forward) => {
                const connLabel = connectionMap.get(forward.connection_id) || 'Unknown Connection'
                const isToggling = startMutation.isPending || stopMutation.isPending

                return (
                  <Card
                    key={forward.id}
                    className={`group relative transition-all border-border/40 hover:border-primary/40 hover:shadow-md overflow-hidden py-0 gap-0 ${
                      forward.active ? 'bg-emerald-500/5 border-emerald-500/30' : ''
                    }`}
                  >
                    <div className="flex items-center p-3 gap-4">
                      {/* Left: Icon */}
                      <div
                        className={`flex-shrink-0 w-9 h-9 rounded-md flex items-center justify-center ${
                          forward.active ? 'bg-emerald-500/10' : 'bg-muted/50'
                        }`}
                      >
                        <ArrowLeftRight
                          className={`h-4 w-4 ${forward.active ? 'text-emerald-600' : 'text-muted-foreground/70'}`}
                        />
                      </div>

                      {/* Middle: Info */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <h3 className="text-base font-bold truncate leading-tight">
                            {forward.name}
                          </h3>
                          {forward.active && (
                            <span className="inline-flex items-center rounded-full px-2 py-0.5 text-[10px] font-medium bg-emerald-500/10 text-emerald-600">
                              Active
                            </span>
                          )}
                        </div>
                        <p className="text-xs text-muted-foreground truncate leading-none mt-0.5">
                          {connLabel}
                        </p>
                        <p className="text-xs text-muted-foreground truncate leading-none mt-1">
                          localhost:{forward.local_port} → :{forward.remote_port}
                        </p>
                        {forward.error && (
                          <p className="text-[11px] text-destructive truncate leading-none mt-1">
                            {forward.error}
                          </p>
                        )}
                      </div>

                      {/* Right: Toggle + Kebab */}
                      <div className="flex items-center gap-2 flex-shrink-0 pr-1">
                        <Switch
                          checked={forward.active}
                          onCheckedChange={() => handleToggle(forward)}
                          disabled={isToggling}
                          className={forward.active ? 'data-[state=checked]:bg-emerald-500' : ''}
                        />
                        <DropdownMenu>
                          <DropdownMenuTrigger
                            className="h-7 w-7 flex items-center justify-center rounded-md opacity-0 group-hover:opacity-100 hover:bg-muted transition-all outline-none"
                          >
                            <MoreVertical className="h-4 w-4 text-muted-foreground" />
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end" className="w-40">
                            <DropdownMenuItem onClick={(e) => { e.stopPropagation(); setEditForward(forward); setFormOpen(true) }}>
                              <Edit2 className="mr-2 h-3.5 w-3.5" /> Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              className="text-destructive focus:bg-destructive/10"
                              onClick={() => setDeleteTarget(forward)}
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

      <ForwardFormSheet
        open={formOpen}
        onOpenChange={(open) => {
          setFormOpen(open)
          if (!open) setEditForward(null)
        }}
        editForward={editForward}
      />

      <AlertDialog open={!!deleteTarget} onOpenChange={(open) => !open && setDeleteTarget(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Port Forward</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete{' '}
              <strong>{deleteTarget?.name}</strong>?{' '}
              {deleteTarget?.active && 'The active tunnel will be stopped. '}
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDelete}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}
