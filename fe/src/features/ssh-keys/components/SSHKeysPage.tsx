import * as React from 'react'
import { Plus, Key, Loader2 } from 'lucide-react'
import { Button } from '@/components/ui/button'
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
import { useSSHKeys, useDeleteSSHKey, useUpdateSSHKey } from '../hooks/useSSHKeys'
import { SSHKeyCard } from './SSHKeyCard'
import { SSHKeyUploadSheet } from './SSHKeyUploadSheet'
import type { SSHKey } from '@/lib/api'

export const SSHKeysPage = () => {
  const { data: sshKeys = [], isLoading } = useSSHKeys()
  const deleteMutation = useDeleteSSHKey()
  const updateMutation = useUpdateSSHKey()

  const [uploadOpen, setUploadOpen] = React.useState(false)
  const [deleteKey, setDeleteKey] = React.useState<SSHKey | null>(null)
  const [deleteWarning, setDeleteWarning] = React.useState<string | null>(null)
  const [affectedCount, setAffectedCount] = React.useState<number>(0)

  const handleRename = async (id: string, name: string) => {
    try {
      await updateMutation.mutateAsync({ id, data: { name } })
    } catch (err) {
      console.error('Rename failed:', err)
    }
  }

  const handleDeleteRequest = async (key: SSHKey) => {
    setDeleteKey(key)
    setDeleteWarning(null)
    setAffectedCount(0)
  }

  const handleConfirmDelete = async () => {
    if (!deleteKey) return

    try {
      const result = await deleteMutation.mutateAsync(deleteKey.id) as any
      if (result && result.warning) {
        // This is a special case: backend returned a warning instead of 204
        // But our useDeleteSSHKey hook might already have succeeded if it just checks r.ok
        // Let's refine the logic if needed. 
        // In Task 1 api.ts: delete returns r.json() if status 200.
        setDeleteWarning(result.warning)
        setAffectedCount(result.affected_connections || 0)
        // We don't close the dialog yet, let the user see the warning and confirm again?
        // Actually the backend might NOT have deleted it yet if it returns a warning.
        // Wait, the backend logic (D-12) says DELETE /api/keys/{id} returns 204 if no connections,
        // OR 200 with a warning if there ARE connections.
        // If we get 200, we should show the warning.
      } else {
        setDeleteKey(null)
      }
    } catch (err) {
      console.error('Delete failed:', err)
    }
  }

  if (isLoading) {
    return (
      <div className="flex-1 flex flex-col items-center justify-center p-6 text-center">
        <Loader2 className="h-8 w-8 text-muted-foreground animate-spin mb-4" />
        <p className="text-muted-foreground">Loading SSH keys...</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Termius-style Header */}
      <header className="flex items-center justify-between px-6 py-2 border-b bg-muted/5">
        <div className="flex items-center gap-4">
          <h1 className="text-lg font-semibold tracking-tight">SSH Keys</h1>
          <p className="text-[11px] text-muted-foreground uppercase tracking-wider font-bold">Secure Storage</p>
        </div>
        <Button size="sm" onClick={() => setUploadOpen(true)} className="h-8 shadow-sm">
          <Plus className="mr-2 h-4 w-4" /> Upload Key
        </Button>
      </header>

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-[1600px] mx-auto">
          {sshKeys.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-32 text-center border-2 border-dashed rounded-lg bg-muted/10">
              <Key className="h-10 w-10 text-muted-foreground/20 mb-3" />
              <p className="text-sm text-muted-foreground">
                No private keys found.
              </p>
              <Button variant="link" size="sm" onClick={() => setUploadOpen(true)}>
                Upload your first key
              </Button>
            </div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              {sshKeys.map((key) => (
                <SSHKeyCard 
                  key={key.id} 
                  sshKey={key} 
                  onRename={handleRename} 
                  onDelete={handleDeleteRequest}
                />
              ))}
            </div>
          )}
        </div>
      </div>

      <SSHKeyUploadSheet open={uploadOpen} onOpenChange={setUploadOpen} />

      <AlertDialog open={!!deleteKey} onOpenChange={(open) => !open && setDeleteKey(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete SSH Key</AlertDialogTitle>
            <AlertDialogDescription>
              {deleteWarning ? (
                <span className="space-y-2 block">
                  <span className="text-destructive font-medium block">Warning: {deleteWarning}</span>
                  <span>
                    Deleting <strong>{deleteKey?.name}</strong> will remove key-based auth from <strong>{affectedCount}</strong> connection(s).
                  </span>
                  <span className="block pt-2">Are you sure you want to proceed? This cannot be undone.</span>
                </span>
              ) : (
                <span>
                  Are you sure you want to delete <strong>{deleteKey?.name}</strong>? This action cannot be undone.
                </span>
              )}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel onClick={() => {
              setDeleteKey(null)
              setDeleteWarning(null)
            }}>
              Cancel
            </AlertDialogCancel>
            <AlertDialogAction 
              onClick={handleConfirmDelete}
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
