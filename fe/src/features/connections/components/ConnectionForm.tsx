import * as React from 'react'
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
  SheetFooter,
} from '@/components/ui/sheet'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Button } from '@/components/ui/button'
import { useAppStore } from '@/stores/app-store'
import { useCreateConnection, useUpdateConnection } from '../hooks/useConnections'
import { useSSHKeys } from '@/features/ssh-keys/hooks/useSSHKeys'
import { ExternalLink } from 'lucide-react'
import type { Connection } from '@/lib/api'

export const ConnectionForm = () => {
  const { 
    creatingConnection, 
    setCreatingConnection, 
    editingConnection, 
    setEditingConnection,
    setSidebarPage
  } = useAppStore()

  const { data: sshKeys = [] } = useSSHKeys()
  const createMutation = useCreateConnection()
  const updateMutation = useUpdateConnection()

  const [formData, setFormData] = React.useState<Partial<Connection>>({
    label: '',
    host: '',
    port: 22,
    username: 'root',
    password: '',
    tags: [],
    auth_method: 'password',
    ssh_key_id: null,
  })

  const [tagsInput, setTagsInput] = React.useState('')
  const [errors, setErrors] = React.useState<Record<string, string>>({})

  const isOpen = creatingConnection || !!editingConnection

  React.useEffect(() => {
    if (editingConnection) {
      setFormData({
        label: editingConnection.label,
        host: editingConnection.host,
        port: editingConnection.port,
        username: editingConnection.username,
        password: '', // Password not loaded for security, handled on backend
        tags: editingConnection.tags || [],
        auth_method: editingConnection.auth_method || 'password',
        ssh_key_id: editingConnection.ssh_key_id || null,
      })
      setTagsInput((editingConnection.tags || []).join(', '))
    } else {
      setFormData({
        label: '',
        host: '',
        port: 22,
        username: 'root',
        password: '',
        tags: [],
        auth_method: 'password',
        ssh_key_id: null,
      })
      setTagsInput('')
    }
    setErrors({})
  }, [editingConnection, creatingConnection])

  const validate = () => {
    const newErrors: Record<string, string> = {}
    if (!formData.label) newErrors.label = 'Label is required'
    if (!formData.host) newErrors.host = 'Host is required'
    if (!formData.username) newErrors.username = 'Username is required'
    if (formData.port && (formData.port < 1 || formData.port > 65535)) {
      newErrors.port = 'Invalid port (1-65535)'
    }
    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleClose = () => {
    setCreatingConnection(false)
    setEditingConnection(null)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validate()) return

    const tags = tagsInput
      .split(',')
      .map((t) => t.trim())
      .filter((t) => t !== '')

    const payload = { ...formData, tags }

    try {
      if (editingConnection) {
        await updateMutation.mutateAsync({ id: editingConnection.id, data: payload })
      } else {
        await createMutation.mutateAsync(payload)
      }
      handleClose()
    } catch (err) {
      console.error('Save failed:', err)
    }
  }

  return (
    <Sheet open={isOpen} onOpenChange={(open) => !open && handleClose()}>
      <SheetContent side="right" className="w-[400px] sm:w-[540px] flex flex-col">
        <SheetHeader>
          <SheetTitle>{editingConnection ? 'Edit Connection' : 'New Connection'}</SheetTitle>
          <SheetDescription>
            {editingConnection 
              ? 'Update your saved connection details.' 
              : 'Add a new SSH connection to your library.'}
          </SheetDescription>
        </SheetHeader>

        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto p-6 space-y-6">
          <div className="space-y-2">
            <Label htmlFor="label">Display Label</Label>
            <Input
              id="label"
              value={formData.label}
              onChange={(e) => setFormData({ ...formData, label: e.target.value })}
              placeholder="My Production Server"
              className={errors.label ? 'border-destructive' : ''}
            />
            {errors.label && <p className="text-xs text-destructive">{errors.label}</p>}
          </div>

          <div className="grid grid-cols-3 gap-4">
            <div className="col-span-2 space-y-2">
              <Label htmlFor="host">Hostname or IP</Label>
              <Input
                id="host"
                value={formData.host}
                onChange={(e) => setFormData({ ...formData, host: e.target.value })}
                placeholder="10.0.0.1"
                className={errors.host ? 'border-destructive' : ''}
              />
              {errors.host && <p className="text-xs text-destructive">{errors.host}</p>}
            </div>
            <div className="space-y-2">
              <Label htmlFor="port">Port</Label>
              <Input
                id="port"
                type="number"
                value={formData.port}
                onChange={(e) => setFormData({ ...formData, port: parseInt(e.target.value) || 0 })}
                className={errors.port ? 'border-destructive' : ''}
              />
              {errors.port && <p className="text-xs text-destructive">{errors.port}</p>}
            </div>
          </div>

          <div className="space-y-2">
            <Label htmlFor="username">Username</Label>
            <Input
              id="username"
              value={formData.username}
              onChange={(e) => setFormData({ ...formData, username: e.target.value })}
              placeholder="root"
              className={errors.username ? 'border-destructive' : ''}
            />
            {errors.username && <p className="text-xs text-destructive">{errors.username}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="password">Password (optional)</Label>
            <Input
              id="password"
              type="password"
              value={formData.password}
              onChange={(e) => setFormData({ ...formData, password: e.target.value })}
              placeholder={editingConnection ? 'Leave empty to keep existing' : 'Secure password'}
            />
            <p className="text-[10px] text-muted-foreground">
              {formData.ssh_key_id
                ? 'Password stored as fallback. Key auth will be primary.'
                : 'Passwords are encrypted with AES-256-GCM on the server.'}
            </p>
          </div>

          {/* SSH Key Selector — per D-19, D-20: both fields always visible */}
          <div className="space-y-2">
            <Label htmlFor="ssh-key">SSH Key (optional)</Label>
            <div className="flex gap-2">
              <select
                id="ssh-key"
                value={formData.ssh_key_id || ''}
                onChange={(e) => {
                  const keyId = e.target.value || null
                  setFormData({
                    ...formData,
                    ssh_key_id: keyId,
                    auth_method: keyId ? 'key' : 'password',
                  })
                }}
                className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2"
              >
                <option value="">None</option>
                {sshKeys.map((key) => (
                  <option key={key.id} value={key.id}>
                    {key.name} ({key.key_type})
                  </option>
                ))}
              </select>
            </div>
            {sshKeys.length === 0 && (
              <p className="text-[10px] text-muted-foreground">
                No SSH keys uploaded yet.
              </p>
            )}
            {/* Manage SSH Keys link — per D-22 */}
            <button
              type="button"
              onClick={() => {
                handleClose()
                setSidebarPage('keys')
              }}
              className="text-xs text-primary hover:underline inline-flex items-center gap-1"
            >
              <ExternalLink className="h-3 w-3" /> Manage SSH Keys
            </button>
          </div>

          <div className="space-y-2">
            <Label htmlFor="tags">Tags (comma-separated)</Label>
            <Input
              id="tags"
              value={tagsInput}
              onChange={(e) => setTagsInput(e.target.value)}
              placeholder="prod, aws, web"
            />
          </div>
        </form>

        <SheetFooter className="pt-6 border-t">
          <Button variant="outline" onClick={handleClose}>Cancel</Button>
          <Button 
            onClick={handleSubmit} 
            disabled={createMutation.isPending || updateMutation.isPending}
          >
            {editingConnection ? 'Save Changes' : 'Create Connection'}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
