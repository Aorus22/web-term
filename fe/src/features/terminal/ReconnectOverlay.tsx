import { Button } from '@/components/ui/button'
import { WifiOff, RefreshCw, X } from 'lucide-react'

interface ReconnectOverlayProps {
  host: string
  onReconnect: () => void
  onDismiss: () => void
}

/**
 * Semi-transparent overlay shown on top of a disconnected terminal pane.
 * Per UI-04: clear reconnection prompt on disconnect.
 */
export function ReconnectOverlay({ host, onReconnect, onDismiss }: ReconnectOverlayProps) {
  return (
    <div className="absolute inset-0 z-10 bg-background/80 backdrop-blur-sm flex items-center justify-center">
      <div className="flex flex-col items-center gap-4 p-6">
        <WifiOff className="h-8 w-8 text-muted-foreground" />
        <div className="text-center space-y-1">
          <p className="text-sm font-medium text-foreground">
            Connection to {host} lost
          </p>
          <p className="text-xs text-muted-foreground">
            The SSH session was disconnected
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="ghost" size="sm" onClick={onDismiss}>
            <X className="h-4 w-4 mr-1" />
            Close
          </Button>
          <Button size="sm" onClick={onReconnect}>
            <RefreshCw className="h-4 w-4 mr-1" />
            Reconnect
          </Button>
        </div>
      </div>
    </div>
  )
}
