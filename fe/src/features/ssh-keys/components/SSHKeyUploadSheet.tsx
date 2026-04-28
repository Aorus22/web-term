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
import { Textarea } from '@/components/ui/textarea'
import { useCreateSSHKey } from '../hooks/useSSHKeys'
import { cn } from '@/lib/utils'

interface SSHKeyUploadSheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export const SSHKeyUploadSheet = ({ open, onOpenChange }: SSHKeyUploadSheetProps) => {
  const createMutation = useCreateSSHKey()

  const [name, setName] = React.useState('')
  const [keyContent, setKeyContent] = React.useState('')
  const [inputMode, setInputMode] = React.useState<'paste' | 'file'>('paste')
  const [error, setError] = React.useState<string | null>(null)
  
  const fileInputRef = React.useRef<HTMLInputElement>(null)

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = (event) => {
      const content = event.target?.result as string
      setKeyContent(content)
      if (!name) {
        // Default name to filename if not set
        setName(file.name.replace(/\.[^/.]+$/, ""))
      }
    }
    reader.readAsText(file)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError(null)

    if (!name.trim()) {
      setError('Key name is required')
      return
    }

    if (!keyContent.trim()) {
      setError('Key content is required')
      return
    }

    try {
      const key_base64 = btoa(keyContent.trim())
      await createMutation.mutateAsync({ name, key_base64 })
      
      // Reset and close
      setName('')
      setKeyContent('')
      onOpenChange(false)
    } catch (err: any) {
      setError(err.message || 'Failed to upload key')
    }
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-[400px] sm:w-[540px] flex flex-col">
        <SheetHeader>
          <SheetTitle>Upload SSH Key</SheetTitle>
          <SheetDescription>
            Add a private key to your secure pool. Key material is encrypted on the server.
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
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <Label>SSH Private Key</Label>
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
                onChange={(e) => setKeyContent(e.target.value)}
                placeholder="-----BEGIN OPENSSH PRIVATE KEY-----..."
                className="font-mono text-xs min-h-[300px] resize-none"
              />
            ) : (
              <div 
                className={cn(
                  "border-2 border-dashed rounded-lg p-12 text-center cursor-pointer hover:bg-accent/50 transition-colors",
                  keyContent ? "border-primary/50" : "border-muted"
                )}
                onClick={() => fileInputRef.current?.click()}
              >
                <input
                  type="file"
                  ref={fileInputRef}
                  className="hidden"
                  onChange={handleFileChange}
                  accept=".pem,.key,id_rsa,id_ed25519"
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
                        e.stopPropagation();
                        setKeyContent('');
                      }}
                      className="text-destructive h-auto p-0"
                    >
                      Clear
                    </Button>
                  </div>
                ) : (
                  <div className="space-y-2">
                    <p className="text-sm font-medium">Click to select file</p>
                    <p className="text-xs text-muted-foreground">.pem, .key, id_rsa, etc.</p>
                  </div>
                )}
              </div>
            )}
          </div>

          {error && <p className="text-sm text-destructive font-medium">{error}</p>}
        </form>

        <SheetFooter className="pt-6 border-t">
          <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button 
            onClick={handleSubmit} 
            disabled={createMutation.isPending}
          >
            {createMutation.isPending ? 'Uploading...' : 'Upload Key'}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  )
}
