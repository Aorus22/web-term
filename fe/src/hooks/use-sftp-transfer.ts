import { useCallback } from 'react'
import { toast } from 'sonner'
import { useAppStore } from '@/stores/app-store'

interface TransferStatus {
  id: string
  status: 'pending' | 'active' | 'completed' | 'error'
  bytesTotal: number
  bytesTransferred: number
  error?: string
}

export function useSFTPTransfer() {
  const backendPort = useAppStore(state => state.backendPort)
  const baseUrl = backendPort !== 0 ? `http://localhost:${backendPort}` : (import.meta.env.VITE_API_URL || '')

  const trackTransfer = useCallback((transferId: string, fileName: string, type: 'upload' | 'transfer' = 'upload') => {
    const label = type === 'upload' ? 'Uploading' : 'Transferring'
    const successLabel = type === 'upload' ? 'Uploaded' : 'Transferred'
    
    const toastId = toast.loading(`${label} ${fileName}...`, {
      description: 'Starting...',
    })

    const eventSource = new EventSource(`${baseUrl}/api/sftp/transfer/${transferId}/progress`)

    eventSource.onmessage = (event) => {
      try {
        const status: TransferStatus = JSON.parse(event.data)
        
        if (status.status === 'active') {
          const progress = status.bytesTotal > 0 
            ? Math.round((status.bytesTransferred / status.bytesTotal) * 100) 
            : 0
          
          toast.loading(`${label} ${fileName}...`, {
            id: toastId,
            description: `Progress: ${progress}%`,
          })
        } else if (status.status === 'completed') {
          toast.success(`Successfully ${successLabel.toLowerCase()} ${fileName}`, {
            id: toastId,
            description: 'Complete',
          })
          eventSource.close()
        } else if (status.status === 'error') {
          toast.error(`Failed to ${type} ${fileName}`, {
            id: toastId,
            description: status.error || 'Unknown error',
          })
          eventSource.close()
        }
      } catch (err) {
        console.error('Failed to parse SSE data', err)
      }
    }

    eventSource.onerror = (err) => {
      console.error('SSE error', err)
      // SSE might reconnect automatically, but if it fails repeatedly we might want to close it.
      // However, sonner toast might already be in a terminal state if the backend closed it.
    }

    return () => {
      eventSource.close()
    }
  }, [baseUrl])

  return { trackTransfer }
}
