import { useState } from 'react'
import { Button } from '@/components/ui/button'
import { Bookmark, X } from 'lucide-react'
import { connectionsApi } from '@/lib/api'
import { useQueryClient } from '@tanstack/react-query'

interface SaveConnectionBannerProps {
  host: string
  port: number
  username: string
  onDismiss: () => void
}

/**
 * Save-connection banner shown after a quick-connect session disconnects.
 * Per D-06: offers to save the connection details (no password — ephemeral).
 * Per T-03-08: does NOT send the ephemeral password — only host/port/username.
 */
export function SaveConnectionBanner({ host, port, username, onDismiss }: SaveConnectionBannerProps) {
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const queryClient = useQueryClient()

  const handleSave = async () => {
    setSaving(true)
    setError(null)
    try {
      await connectionsApi.create({
        label: `${username}@${host}`,
        host,
        port,
        username,
        tags: [],
      })
      // Invalidate connections cache so sidebar updates
      await queryClient.invalidateQueries({ queryKey: ['connections'] })
      onDismiss()
    } catch {
      setError('Failed to save connection')
    } finally {
      setSaving(false)
    }
  }

  return (
    <div className="flex items-center justify-between gap-3 px-4 py-2 bg-muted/50 border-b text-sm">
      <div className="flex items-center gap-2 text-muted-foreground">
        <Bookmark className="h-4 w-4" />
        <span>Save this connection?</span>
        {error && <span className="text-destructive text-xs">({error})</span>}
      </div>
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm" onClick={onDismiss} disabled={saving}>
          <X className="h-3 w-3 mr-1" />
          Dismiss
        </Button>
        <Button size="sm" onClick={handleSave} disabled={saving}>
          Save
        </Button>
      </div>
    </div>
  )
}
