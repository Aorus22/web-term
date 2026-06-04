import { useCallback } from 'react'
import { useAppStore } from '@/stores/app-store'
import { useSFTPStore, type TransferStatus } from '@/features/sftp/stores/sftp-store'

interface SSETransferStatus {
  id: string
  total_bytes: number
  bytes_transferred: number
  status: string
  error?: string
}

export function useSFTPTransfer() {
  const backendPort = useAppStore(state => state.backendPort)
  const baseUrl = backendPort !== 0 ? `http://localhost:${backendPort}` : (import.meta.env.VITE_API_URL || '')
  
  const addTransfer = useSFTPStore(state => state.addTransfer)
  const updateTransfer = useSFTPStore(state => state.updateTransfer)

  const trackTransfer = useCallback((transferId: string, fileName: string, type: 'upload' | 'transfer' = 'upload') => {
    // Add to global store
    addTransfer({
      id: transferId,
      fileName,
      status: 'pending',
      type,
      bytesTotal: 0,
      bytesTransferred: 0,
    })

    const eventSource = new EventSource(`${baseUrl}/api/sftp/transfer/${transferId}/progress`)

    eventSource.onmessage = (event) => {
      try {
        const sse: SSETransferStatus = JSON.parse(event.data)
        const progress = sse.total_bytes > 0 
          ? Math.round((sse.bytes_transferred / sse.total_bytes) * 100) 
          : 0
        
        // Map SSE data to store format (snake_case keys + status mapping)
        const storeStatus = sse.status === 'transferring' ? 'active' : sse.status

        updateTransfer(transferId, {
          status: storeStatus as TransferStatus['status'],
          progress,
          bytesTotal: sse.total_bytes,
          bytesTransferred: sse.bytes_transferred,
          error: sse.error,
        })

        if (sse.status === 'completed' || sse.status === 'error') {
          eventSource.close()
        }
      } catch (err) {
        console.error('Failed to parse SSE data', err)
      }
    }

    eventSource.onerror = (err) => {
      console.error('SSE error', err)
    }

    return () => {
      eventSource.close()
    }
  }, [baseUrl, addTransfer, updateTransfer])

  return { trackTransfer }
}
