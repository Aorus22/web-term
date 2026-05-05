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
import { Textarea } from '@/components/ui/textarea'
import { useUpdateSSHKey } from '../hooks/useSSHKeys'
import { cn } from '@/lib/utils'
import type { SSHKey } from '@/lib/api'

interface SSHKeyEditSheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  sshKey: SSHKey | null
}

export const SSHKeyEditSheet = ({ open, onOpenChange, sshKey }: SSHKeyEditSheetProps) => {
  const updateMutation = useUpdateSSHKey()

  const [name, setName] = React.useState('')
  const [keyContent, setKeyContent] = React.useState('')
  const [hasNewKey, setHasNewKey] = React.useState(false)
  const [inputMode, setInputMode] = React.useState<'paste' | 'file'>('paste')
  const [error, setError] = React.useState<string | null>(null)
  
  const fileInputRef = React.useRef<HTMLInputElement>(null)
  const [isDragging, setIsDragging] = React.useState(false)

  React.useEffect(() => {
    if (open && sshKey) {
      setName(sshKey.name)
      setKeyContent('')
      setHasNewKey(false)
      setError(null)
    }
  }, [open, sshKey])

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = (event) => {
      const content = event.target?.result as string
      setKeyContent(content)
      setHasNewKey(true)
    }
    reader.readAsText(file)
  }

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)

    const file = e.dataTransfer.files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = (event) => {
        const content = event.target?.result as string
        setKeyContent(content)
        setHasNewKey(true)
      }
      reader.readAsText(file)
    }
  }

  const handleSubmit = async (e?: React.FormEvent) => {
    e?.preventDefault()
    setError(null)

    if (!name.trim()) {
      setError('Key name is required')
      return
    }

    if (!sshKey) return

    try {
      const data: { name: string; key_base64?: string } = { name: name.trim() }
      
      if (hasNewKey && keyContent.trim()) {
        data.key_base64 = btoa(keyContent.trim())
      }

      await updateMutation.mutateAsync({ id: sshKey.id, data })
      toast.success(hasNewKey ? 'SSH key updated with new key' : 'SSH key updated')
      onOpenChange(false)
    } catch (err: any) {
      setError(err.message || 'Failed to update key')
    }
  }

  const isSubmitDisabled = updateMutation.isPending || !name.trim() || (hasNewKey && !keyContent.trim())

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-[400px] sm:w-[540px] flex flex-col">
        <SheetHeader>
          <SheetTitle>Edit SSH Key</SheetTitle>
          <SheetDescription>
            Update the name for your SSH key. You can also upload a new key file to replace the existing one.
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

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label>Replace Key File</Label>
              <div className="flex bg-muted rounded-md p-0.5">
                <Button
                  type="button"
                  variant={inputMode === 'paste' ? 'secondary' : 'ghost'}
                  size="sm"
                  className="h-7 text-xs px-2"
                  onClick={() => setInputMode('paste')}
                >
                  Paste Text
                </Button>
                <Button
                  type="button"
                  variant={inputMode === 'file' ? 'secondary' : 'ghost'}
                  size="sm"
                  className="h-7 text-xs px-2"
                  onClick={() => setInputMode('file')}
                >
                  Upload File
                </Button>
              </div>
            </div>

            {inputMode === 'paste' ? (
              <Textarea
                value={keyContent}
                onChange={(e) => { setKeyContent(e.target.value); setHasNewKey(true) }}
                placeholder="-----BEGIN OPENSSH PRIVATE KEY-----... (leave empty to keep current key)"
                className="text-xs min-h-[200px] resize-none"
              />
            ) : (
              <div 
                className={cn(
                  "border-2 border-dashed rounded-lg p-8 text-center cursor-pointer transition-colors",
                  keyContent ? "border-primary/50" : isDragging ? "border-primary bg-primary/5" : "border-muted hover:bg-accent/50"
                )}
                onClick={() => fileInputRef.current?.click()}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
              >
                <input
                  type="file"
                  ref={fileInputRef}
                  className="hidden"
                  onChange={handleFileChange}
                />
                {keyContent ? (
                  <div className="space-y-2">
                    <p className="text-sm font-medium text-primary">File Selected</p>
                    <p className="text-xs text-muted-foreground truncate max-w-[200px] mx-auto">
                      {keyContent.substring(0, 50)}...
                    </p>
                    <Button 
                      type="button" 
                      variant="link" 
                      size="sm" 
                      onClick={(e) => {
                        e.stopPropagation()
                        setKeyContent('')
                        setHasNewKey(false)
                      }}
                      className="text-destructive h-auto p-0"
                    >
                      Clear
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-2">
                    <p className="text-sm font-medium">Drag & drop file here</p>
                    <p className="text-xs text-muted-foreground">or click to browse</p>
                  </div>
                )}
              </div>
            )}
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