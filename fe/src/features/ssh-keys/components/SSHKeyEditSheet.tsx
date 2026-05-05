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
import { useUpdateSSHKey } from '../hooks/useSSHKeys'
import type { SSHKey } from '@/lib/api'

interface SSHKeyEditSheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  sshKey: SSHKey | null
}

export const SSHKeyEditSheet = ({ open, onOpenChange, sshKey }: SSHKeyEditSheetProps) => {
  const updateMutation = useUpdateSSHKey()

  const [name, setName] = React.useState('')
  const [error, setError] = React.useState<string | null>(null)

  React.useEffect(() => {
    if (open && sshKey) {
      setName(sshKey.name)
      setError(null)
    }
  }, [open, sshKey])

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()
    setError(null)

    if (!name.trim()) {
      setError('Key name is required')
      return
    }

    if (!sshKey) return

    try {
      await updateMutation.mutateAsync({ id: sshKey.id, data: { name: name.trim() } })
      toast.success('SSH key updated')
      onOpenChange(false)
    } catch (err: any) {
      setError(err.message || 'Failed to update key')
    }
  }

  const isSubmitDisabled = updateMutation.isPending || !name.trim()

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-[400px] flex flex-col">
        <SheetHeader>
          <SheetTitle>Edit SSH Key</SheetTitle>
          <SheetDescription>
            Update the name for your SSH key.
          </SheetDescription>
        </SheetHeader>

        <form onSubmit={handleSubmit} className="flex-1 overflow-y-auto p-6 space-y-6">
          <div className="space-y-2">
            <Label htmlFor="key-name">Key Name</Label>
            <Input
              id="key-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. My Production Key"
            />
            {error && <p className="text-xs text-destructive">{error}</p>}
          </div>
        </form>

        <SheetFooter className="pt-6 border-t">
          <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button onClick={handleSubmit} disabled={isSubmitDisabled}>
            {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}