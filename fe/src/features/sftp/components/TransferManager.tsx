import React, { useState } from 'react'
import { useSFTPStore, TransferStatus } from '../stores/sftp-store'
import { Progress } from '@/components/ui/progress'
import { Button } from '@/components/ui/button'
import { 
  X, 
  ChevronDown, 
  ChevronUp, 
  Upload, 
  Download, 
  ArrowRightLeft, 
  CheckCircle2, 
  XCircle,
  Loader2
} from 'lucide-react'
import { cn } from '@/lib/utils'
import { ScrollArea } from '@/components/ui/scroll-area'

export function TransferManager() {
  const { transfers, clearCompleted, removeTransfer } = useSFTPStore()
  const [isExpanded, setIsExpanded] = useState(false)

  const activeTransfers = transfers.filter(t => t.status === 'active' || t.status === 'pending')
  const completedTransfers = transfers.filter(t => t.status === 'completed' || t.status === 'error')

  if (transfers.length === 0) return null

  const getIcon = (type: TransferStatus['type'], status: TransferStatus['status']) => {
    if (status === 'active' || status === 'pending') {
      return <Loader2 className="h-4 w-4 animate-spin text-primary" />
    }
    if (status === 'completed') {
      return <CheckCircle2 className="h-4 w-4 text-green-500" />
    }
    if (status === 'error') {
      return <XCircle className="h-4 w-4 text-destructive" />
    }

    switch (type) {
      case 'upload': return <Upload className="h-4 w-4" />
      case 'download': return <Download className="h-4 w-4" />
      case 'transfer': return <ArrowRightLeft className="h-4 w-4" />
      default: return null
    }
  }

  return (
    <div className={cn(
      "fixed bottom-0 right-8 w-80 bg-card border border-border rounded-t-lg shadow-lg overflow-hidden transition-all duration-300 z-50",
      isExpanded ? "h-96" : "h-12"
    )}>
      {/* Header */}
      <div 
        className="h-12 px-4 flex items-center justify-between cursor-pointer hover:bg-accent/50"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <div className="flex items-center gap-2">
          <span className="font-semibold text-sm">Transfers</span>
          {activeTransfers.length > 0 && (
            <span className="bg-primary text-primary-foreground text-[10px] px-1.5 py-0.5 rounded-full font-bold">
              {activeTransfers.length}
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {isExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronUp className="h-4 w-4" />}
        </div>
      </div>

      {/* Content */}
      {isExpanded && (
        <div className="h-[calc(100%-3rem)] flex flex-col">
          <div className="px-4 py-2 border-b border-border flex justify-between items-center bg-muted/30">
            <span className="text-xs text-muted-foreground uppercase font-bold tracking-wider">
              {activeTransfers.length > 0 ? 'Active' : 'History'}
            </span>
            {completedTransfers.length > 0 && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="h-6 px-2 text-[10px]" 
                onClick={(e) => {
                  e.stopPropagation()
                  clearCompleted()
                }}
              >
                Clear History
              </Button>
            )}
          </div>
          
          <ScrollArea className="flex-1">
            <div className="p-2 space-y-1">
              {transfers.map((transfer) => (
                <div 
                  key={transfer.id} 
                  className="p-2 rounded-md hover:bg-accent/50 group border border-transparent hover:border-border transition-colors"
                >
                  <div className="flex items-center justify-between mb-1.5">
                    <div className="flex items-center gap-2 overflow-hidden">
                      {getIcon(transfer.type, transfer.status)}
                      <span className="text-xs font-medium truncate" title={transfer.fileName}>
                        {transfer.fileName}
                      </span>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-5 w-5 opacity-0 group-hover:opacity-100 transition-opacity"
                      onClick={(e) => {
                        e.stopPropagation()
                        removeTransfer(transfer.id)
                      }}
                    >
                      <X className="h-3 w-3" />
                    </Button>
                  </div>
                  
                  {transfer.status === 'active' || transfer.status === 'pending' ? (
                    <div className="space-y-1">
                      <Progress value={transfer.progress} className="h-1" />
                      <div className="flex justify-between text-[10px] text-muted-foreground">
                        <span>{transfer.progress}%</span>
                        <span>
                          {formatBytes(transfer.bytesTransferred)} / {formatBytes(transfer.bytesTotal)}
                        </span>
                      </div>
                    </div>
                  ) : transfer.status === 'error' ? (
                    <span className="text-[10px] text-destructive block truncate">
                      {transfer.error || 'Transfer failed'}
                    </span>
                  ) : (
                    <span className="text-[10px] text-muted-foreground block">
                      Completed {formatTime(transfer.startTime)}
                    </span>
                  )}
                </div>
              ))}
            </div>
          </ScrollArea>
        </div>
      )}
    </div>
  )
}

function formatBytes(bytes: number, decimals = 2) {
  if (!+bytes) return '0 Bytes'
  const k = 1024
  const dm = decimals < 0 ? 0 : decimals
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(dm))} ${sizes[i]}`
}

function formatTime(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}
