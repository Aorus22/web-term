import { useState, useEffect, useRef } from 'react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

interface RenameDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  oldName: string
  onConfirm: (newName: string) => void
}

export function RenameDialog({ open, onOpenChange, oldName, onConfirm }: RenameDialogProps) {
  console.log('[RENAME] Render RenameDialog', { open, oldName })
  // Track the current input text. Initialize from oldName on first mount.
  const [name, setName] = useState(oldName)
  const inputRef = useRef<HTMLInputElement>(null)

  // Focus and select just the basename (not the extension) when the dialog opens,
  // matching macOS Finder behavior. Hidden files like ".gitignore" are selected whole.
  useEffect(() => {
    if (!open) return
    // Wait for the next frame so the input is mounted and the value is in state.
    const id = requestAnimationFrame(() => {
      const el = inputRef.current
      if (!el) return
      el.focus()
      const isHidden = oldName.startsWith('.') && oldName.indexOf('.') === oldName.lastIndexOf('.')
      const lastDot = oldName.lastIndexOf('.')
      const end = lastDot > 0 && !isHidden ? lastDot : oldName.length
      el.setSelectionRange(0, end)
    })
    return () => cancelAnimationFrame(id)
  }, [open, oldName])

  // When the dialog is closed, reset name to oldName so the next open picks up
  // the latest file name. Done in a layout effect to avoid flashing the wrong value.
  useEffect(() => {
    if (!open) {
      setName(oldName)
    }
  }, [open, oldName])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Rename</DialogTitle>
          <DialogDescription>
            Enter a new name for {oldName}.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="name" className="text-right">
              Name
            </Label>
            <Input
              id="name"
              ref={inputRef}
              value={name}
              onChange={(e) => {
                console.log('[RENAME] input onChange', e.target.value)
                setName(e.target.value)
              }}
              className="col-span-3"
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  onConfirm(name)
                  onOpenChange(false)
                }
              }}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={() => {
            console.log('[RENAME] Button clicked, name=', name)
            onConfirm(name)
            onOpenChange(false)
          }}>
            Rename
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface NewFolderDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onConfirm: (name: string) => void
}

export function NewFolderDialog({ open, onOpenChange, onConfirm }: NewFolderDialogProps) {
  const [name, setName] = useState('')

  useEffect(() => {
    if (open) setName('')
  }, [open])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>New Folder</DialogTitle>
          <DialogDescription>
            Enter a name for the new folder.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid grid-cols-4 items-center gap-4">
            <Label htmlFor="folder-name" className="text-right">
              Name
            </Label>
            <Input
              id="folder-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="col-span-3"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  onConfirm(name)
                  onOpenChange(false)
                }
              }}
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={() => {
            onConfirm(name)
            onOpenChange(false)
          }}>
            Create
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface OverwriteDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  fileName: string
  onConfirm: () => void
  onKeepBoth: () => void
}

export function OverwriteDialog({ open, onOpenChange, fileName, onConfirm, onKeepBoth }: OverwriteDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>File Already Exists</AlertDialogTitle>
          <AlertDialogDescription>
            A file named "{fileName}" already exists in the destination.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={onKeepBoth}
            className="bg-secondary text-secondary-foreground hover:bg-secondary/80"
          >
            Keep Both
          </AlertDialogAction>
          <AlertDialogAction onClick={onConfirm}>Overwrite</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

interface DeleteConfirmDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  fileName: string
  onConfirm: () => void
}

export function DeleteConfirmDialog({ open, onOpenChange, fileName, onConfirm }: DeleteConfirmDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This will permanently delete "{fileName}". This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onConfirm} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
