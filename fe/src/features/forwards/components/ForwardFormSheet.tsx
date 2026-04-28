import * as React from 'react'
import { toast } from 'sonner'
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useCreateForward } from '../hooks/useForwards'
import { useConnections } from '@/features/connections/hooks/useConnections'
import type { PortForward } from '@/lib/api'

interface ForwardFormSheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  editForward?: PortForward | null
}

const isValidPort = (value: number): boolean => {
  return Number.isInteger(value) && value >= 1 && value <= 65535
}

export const ForwardFormSheet = ({ open, onOpenChange, editForward }: ForwardFormSheetProps) => {
  const createMutation = useCreateForward()
  const { data: connections = [] } = useConnections()

  const isEditMode = !!editForward

  const [name, setName] = React.useState('')
  const [connectionId, setConnectionId] = React.useState('')
  const [localPort, setLocalPort] = React.useState('')
  const [remotePort, setRemotePort] = React.useState('')
  const [errors, setErrors] = React.useState<Record<string, string>>({})

  // Reset form when opening/closing
  React.useEffect(() => {
    if (open) {
      if (editForward) {
        setName(editForward.name)
        setConnectionId(editForward.connection_id)
        setLocalPort(String(editForward.local_port))
        setRemotePort(String(editForward.remote_port))
      } else {
        setName('')
        setConnectionId('')
        setLocalPort('')
        setRemotePort('')
      }
      setErrors({})
    }
  }, [open, editForward])

  const validateField = (field: string, value: string) => {
    if (field === 'name' && !value.trim()) {
      setErrors((prev) => ({ ...prev, name: 'Name is required' }))
    } else if (field === 'name') {
      setErrors((prev) => {
        const { name, ...rest } = prev
        return rest
      })
    }

    if (field === 'localPort' || field === 'remotePort') {
      const num = parseInt(value, 10)
      if (!value || !isValidPort(num)) {
        setErrors((prev) => ({ ...prev, [field]: 'Port must be 1-65535' }))
      } else {
        setErrors((prev) => {
          const { [field]: _, ...rest } = prev
          return rest
        })
      }
    }
  }

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()

    // Validate all fields
    const newErrors: Record<string, string> = {}
    if (!name.trim()) newErrors.name = 'Name is required'
    if (!connectionId) newErrors.connectionId = 'Connection is required'

    const lp = parseInt(localPort, 10)
    const rp = parseInt(remotePort, 10)
    if (!localPort || !isValidPort(lp)) newErrors.localPort = 'Port must be 1-65535'
    if (!remotePort || !isValidPort(rp)) newErrors.remotePort = 'Port must be 1-65535'

    if (Object.keys(newErrors).length > 0) {
      setErrors(newErrors)
      return
    }

    try {
      await createMutation.mutateAsync({
        name: name.trim(),
        connection_id: connectionId,
        local_port: lp,
        remote_port: rp,
      })
      toast.success('Port forward created')
      onOpenChange(false)
    } catch (err: any) {
      const errorMsg = err?.error || err?.message || 'Failed to create forward'
      toast.error('Port conflict', { description: errorMsg })
      // Keep sheet open so user can adjust
    }
  }

  const isSubmitDisabled =
    createMutation.isPending ||
    !name.trim() ||
    !connectionId ||
    !localPort ||
    !remotePort ||
    !isValidPort(parseInt(localPort, 10)) ||
    !isValidPort(parseInt(remotePort, 10))

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-[400px] sm:w-[540px] flex flex-col">
        <SheetHeader>
          <SheetTitle>{isEditMode ? 'Edit Port Forward' : 'Create Port Forward'}</SheetTitle>
          <SheetDescription>
            {isEditMode
              ? 'Modify the port forwarding rule.'
              : 'Set up an SSH local port forwarding tunnel.'}
          </SheetDescription>
        </SheetHeader>

        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto p-6 space-y-6">
          <div className="space-y-2">
            <Label htmlFor="fwd-name">Name</Label>
            <Input
              id="fwd-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={() => validateField('name', name)}
              placeholder="e.g., Production Database"
            />
            {errors.name && <p className="text-xs text-destructive">{errors.name}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="fwd-connection">Connection</Label>
            <Select value={connectionId} onValueChange={(v) => v !== null && setConnectionId(v)}>
              <SelectTrigger id="fwd-connection">
                <SelectValue placeholder="Select a connection" />
              </SelectTrigger>
              <SelectContent>
                {connections.map((conn) => (
                  <SelectItem key={conn.id} value={conn.id}>
                    {conn.label} ({conn.host})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {errors.connectionId && (
              <p className="text-xs text-destructive">{errors.connectionId}</p>
            )}
            {connections.length === 0 && (
              <p className="text-xs text-muted-foreground">
                No connections available. Create a host first.
              </p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="fwd-local-port">Local Port</Label>
            <Input
              id="fwd-local-port"
              type="number"
              min={1}
              max={65535}
              value={localPort}
              onChange={(e) => setLocalPort(e.target.value)}
              onBlur={() => validateField('localPort', localPort)}
              placeholder="e.g., 5432"
            />
            {errors.localPort && <p className="text-xs text-destructive">{errors.localPort}</p>}
          </div>

          <div className="space-y-2">
            <Label htmlFor="fwd-remote-port">Remote Port</Label>
            <Input
              id="fwd-remote-port"
              type="number"
              min={1}
              max={65535}
              value={remotePort}
              onChange={(e) => setRemotePort(e.target.value)}
              onBlur={() => validateField('remotePort', remotePort)}
              placeholder="e.g., 5432"
            />
            {errors.remotePort && <p className="text-xs text-destructive">{errors.remotePort}</p>}
          </div>
        </form>

        <SheetFooter className="pt-6 border-t">
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={() => handleSubmit()} disabled={isSubmitDisabled}>
            {createMutation.isPending
              ? 'Creating...'
              : isEditMode
                ? 'Save Changes'
                : 'Create Forward'}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
