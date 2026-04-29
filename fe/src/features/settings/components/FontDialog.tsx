import { useState, useEffect } from 'react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'

const MONO_FONTS = [
  'Geist Mono',
  'JetBrains Mono',
  'Fira Code',
  'Source Code Pro',
  'IBM Plex Mono',
  'Cascadia Code',
  'Inconsolata',
  'Ubuntu Mono',
  'Menlo',
  'Consolas',
  'Monaco',
  'monospace'
]

interface FontDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  currentFont: string
  currentSize: string
  onSave: (font: string, size: string) => void
}

export function FontDialog({ open, onOpenChange, currentFont, currentSize, onSave }: FontDialogProps) {
  const [font, setFont] = useState(currentFont)
  const [size, setSize] = useState(currentSize)

  useEffect(() => {
    if (open) {
      setFont(currentFont)
      setSize(currentSize)
    }
  }, [open, currentFont, currentSize])

  // Load web fonts dynamically
  useEffect(() => {
    if (font === 'Geist Mono' || font === 'monospace' || ['Menlo', 'Consolas', 'Monaco'].includes(font)) return
    
    const fontId = `font-link-${font.replace(/\s+/g, '-').toLowerCase()}`
    if (document.getElementById(fontId)) return

    const link = document.createElement('link')
    link.id = fontId
    link.rel = 'stylesheet'
    link.href = `https://fonts.googleapis.com/css2?family=${font.replace(/\s+/g, '+')}:wght@400;700&display=swap`
    document.head.appendChild(link)
  }, [font])

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[550px]">
        <DialogHeader>
          <DialogTitle>Terminal Font</DialogTitle>
        </DialogHeader>
        <div className="grid gap-6 py-4">
          <div className="grid gap-2">
            <Label htmlFor="font-family" className="text-xs font-medium text-muted-foreground uppercase tracking-wider">Font Family</Label>
            <Select value={font} onValueChange={setFont}>
              <SelectTrigger id="font-family" className="w-full">
                <SelectValue placeholder="Select font" />
              </SelectTrigger>
              <SelectContent>
                {MONO_FONTS.map((f) => (
                  <SelectItem key={f} value={f}>
                    <span style={{ fontFamily: f }}>{f}</span>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-2">
            <div className="flex items-center justify-between">
              <Label htmlFor="font-size">Font Size ({size}px)</Label>
            </div>
            <input
              id="font-size"
              type="range"
              min="10"
              max="24"
              step="1"
              value={size}
              onChange={(e) => setSize(e.target.value)}
              className="w-full h-2 bg-secondary rounded-lg appearance-none cursor-pointer accent-primary"
            />
          </div>

          <div className="mt-4 min-w-0">
            <Label>Preview</Label>
            <div 
              className="mt-2 p-4 rounded bg-muted border font-mono whitespace-pre overflow-x-auto w-full h-32"
              style={{ fontFamily: font, fontSize: `${size}px`, lineHeight: '1.2' }}
            >
              $ ls -la /home/user{'\n'}
              drwxr-xr-x  5 user staff 160 Apr 28 10:30 .{'\n'}
              -rw-r--r--  1 user staff  42 Apr 28 10:30 .bashrc{'\n'}
              ${' '}
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
          <Button onClick={() => {
            onSave(font, size)
            onOpenChange(false)
          }}>Save Changes</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
