import { useCallback } from 'react'
import { toast } from 'sonner'
import { useAppStore } from '@/stores/app-store'
import { useSFTPStore } from '@/features/sftp/stores/sftp-store'

interface SSETransferStatus {
  id: string
  status: 'pending' | 'active' | 'completed' | 'error'
  bytesTotal: number
  bytesTransferred: number
  error?: string
}

export function useSFTPTransfer() {
  const backendPort = useAppStore(state => state.backendPort)
  const baseUrl = backendPort !== 0 ? `http://localhost:${backendPort}` : (import.meta.env.VITE_API_URL || '')
  
  const addTransfer = useSFTPStore(state => state.addTransfer)
  const updateTransfer = useSFTPStore(state => state.updateTransfer)

  const trackTransfer = useCallback((transferId: string, fileName: string, type: 'upload' | 'transfer' = 'upload') => {
    const label = type === 'upload' ? 'Uploading' : 'Transferring'
    const successLabel = type === 'upload' ? 'Uploaded' : 'Transferred'
    
    // Add to global store
    addTransfer({
      id: transferId,
      fileName,
      status: 'pending',
      type,
      bytesTotal: 0,
      bytesTransferred: 0,
    })

    const toastId = toast.loading(`${label} ${fileName}...`, {
      description: 'Starting...',
    })

    const eventSource = new EventSource(`${baseUrl}/api/sftp/transfer/${transferId}/progress`)

    eventSource.onmessage = (event) => {
      try {
        const status: SSETransferStatus = JSON.parse(event.data)
        const progress = status.bytesTotal > 0 
          ? Math.round((status.bytesTransferred / status.bytesTotal) * 100) 
          : 0
        
        // Update global store
        updateTransfer(transferId, {
          status: status.status,
          progress,
          bytesTotal: status.bytesTotal,
          bytesTransferred: status.bytesTransferred,
          error: status.error,
        })

        if (status.status === 'completed') {
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
    }

    return () => {
      eventSource.close()
    }
  }, [baseUrl, addTransfer, updateTransfer])

  return { trackTransfer }
}
