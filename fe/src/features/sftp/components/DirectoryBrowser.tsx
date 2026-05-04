import { useState, useRef, useMemo, useCallback } from 'react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { connectionsApi, sftpApi } from '@/lib/api'
import type { FileInfo } from '@/lib/api'
import { Loader2, AlertCircle, CornerLeftUp, Download } from 'lucide-react'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { useAppStore } from '@/stores/app-store'
import { DirectoryToolbar } from './DirectoryToolbar'
import { RenameDialog, NewFolderDialog, DeleteConfirmDialog, OverwriteDialog } from './OperationDialogs'
import { useSFTPTransfer } from '@/hooks/use-sftp-transfer'
import { toast } from 'sonner'
import { FileIcon } from './FileIcon'
import { SftpBreadcrumbs } from './SftpBreadcrumbs'
import { useSFTPShortcuts } from '../hooks/use-sftp-shortcuts'

const formatSize = (size: number) => {
  if (size === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(size) / Math.log(k))
  return parseFloat((size / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const formatDate = (dateString: string) => {
  if (!dateString) return ''
  try {
    return new Date(dateString).toLocaleString()
  } catch (e) {
    return dateString
  }
}

const getParentPath = (currentPath: string) => {
  if (currentPath === '.' || currentPath === '' || currentPath === '/') return '.'
  const isAbsolute = currentPath.startsWith('/')
  const cleanPath = currentPath.replace(/\/+$/, '')
  const parts = cleanPath.split('/')
  parts.pop()
  
  if (parts.length === 0) return isAbsolute ? '/' : '.'
  if (parts.length === 1 && parts[0] === '') return '/'
  
  return parts.join('/')
}

export function DirectoryBrowser() {
  const [selectedConnection, setSelectedConnection] = useState<string>("local")
  const [path, setPath] = useState<string>(".")
  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null)
  const [isFocused, setIsFocused] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  
  // Dialog states
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false)
  const [isNewFolderDialogOpen, setIsNewFolderDialogOpen] = useState(false)
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [isOverwriteDialogOpen, setIsOverwriteDialogOpen] = useState(false)
  const [pendingUpload, setPendingUpload] = useState<File | null>(null)
  const [pendingTransfer, setPendingTransfer] = useState<{data: any, targetPath: string} | null>(null)

  const fileInputRef = useRef<HTMLInputElement>(null)
  const queryClient = useQueryClient()
  const { sftpClipboard, setSftpClipboard } = useAppStore()
  const { trackTransfer } = useSFTPTransfer()
  
  const { data: connections } = useQuery({
    queryKey: ['connections'],
    queryFn: () => connectionsApi.list()
  })

  const { data: files, isLoading, error } = useQuery({
    queryKey: ['sftp', selectedConnection, path],
    queryFn: () => sftpApi.list(selectedConnection, path),
    enabled: !!selectedConnection
  })

  const refreshDir = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ['sftp', selectedConnection, path] })
  }, [queryClient, selectedConnection, path])

  const handleAction = useCallback((action: 'cut' | 'copy', file: FileInfo) => {
    const fullPath = path === '.' ? file.name : path === '/' ? `/${file.name}` : `${path}/${file.name}`
    setSftpClipboard({
      action,
      connectionId: selectedConnection,
      path: fullPath,
      fileName: file.name,
      isDir: file.isDir
    })
  }, [path, selectedConnection, setSftpClipboard])

  const handlePaste = useCallback(async () => {
    if (!sftpClipboard) return
    const targetPath = path === '.' || path === '/' ? `/${sftpClipboard.fileName}` : `${path}/${sftpClipboard.fileName}`

    // Overwrite check
    if (files?.some(f => f.name === sftpClipboard.fileName)) {
      setPendingTransfer({ data: sftpClipboard, targetPath })
      setIsOverwriteDialogOpen(true)
      return
    }

    executeTransfer(sftpClipboard, targetPath)
  }, [sftpClipboard, path, files, executeTransfer])

  const executeTransfer = useCallback(async (data: any, targetPath: string) => {
    try {
      const { transferId } = await sftpApi.transfer(data.connectionId, data.path, selectedConnection, targetPath)
      trackTransfer(transferId, data.fileName, 'transfer')
      
      if (data.action === 'cut') {
         await sftpApi.remove(data.connectionId, data.path)
         setSftpClipboard(null)
      }
      refreshDir()
    } catch (e) {
      console.error('Transfer error:', e)
      toast.error('Failed to transfer file')
    }
  }, [selectedConnection, trackTransfer, setSftpClipboard, refreshDir])

  const handleRenameConfirm = async (newName: string) => {
    if (!selectedFile || !newName || newName === selectedFile.name) return
    const oldPath = path === '.' ? selectedFile.name : path === '/' ? `/${selectedFile.name}` : `${path}/${selectedFile.name}`
    const newPath = path === '.' ? newName : path === '/' ? `/${newName}` : `${path}/${newName}`
    
    try {
      await sftpApi.rename(selectedConnection, oldPath, newPath)
      refreshDir()
      setSelectedFile(null)
    } catch (e) {
      console.error('Rename error:', e)
      toast.error('Failed to rename')
    }
  }

  const handleDeleteConfirm = async () => {
    if (!selectedFile) return
    const fullPath = path === '.' ? selectedFile.name : path === '/' ? `/${selectedFile.name}` : `${path}/${selectedFile.name}`
    try {
      await sftpApi.remove(selectedConnection, fullPath)
      refreshDir()
      setSelectedFile(null)
    } catch (e) {
      console.error('Delete error:', e)
      toast.error('Failed to delete')
    }
  }

  const handleMkdirConfirm = async (name: string) => {
    if (!name) return
    const fullPath = path === '.' ? name : path === '/' ? `/${name}` : `${path}/${name}`
    try {
      await sftpApi.mkdir(selectedConnection, fullPath)
      refreshDir()
    } catch (e) {
      console.error('Mkdir error:', e)
      toast.error('Failed to create folder')
    }
  }

  const handleUploadClick = () => {
    fileInputRef.current?.click()
  }

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    
    // Overwrite check
    if (files?.some(f => f.name === file.name)) {
      setPendingUpload(file)
      setIsOverwriteDialogOpen(true)
    } else {
      executeUpload(file)
    }
    
    // Clear input so same file can be selected again
    e.target.value = ''
  }

  const executeUpload = useCallback(async (file: File) => {
    const targetPath = path === '.' || path === '/' ? `/${file.name}` : `${path}/${file.name}`
    try {
      const { transferId } = await sftpApi.upload(selectedConnection, targetPath, file)
      trackTransfer(transferId, file.name, 'upload')
      refreshDir()
    } catch (e) {
      console.error('Upload error:', e)
      toast.error(`Failed to upload ${file.name}`)
    }
  }, [path, selectedConnection, trackTransfer, refreshDir])

  const handleOverwriteConfirm = () => {
    if (pendingUpload) {
      executeUpload(pendingUpload)
      setPendingUpload(null)
    } else if (pendingTransfer) {
      executeTransfer(pendingTransfer.data, pendingTransfer.targetPath)
      setPendingTransfer(null)
    }
    setIsOverwriteDialogOpen(false)
  }

  const handleDownload = useCallback((file: FileInfo) => {
    const fullPath = path === '.' ? file.name : path === '/' ? `/${file.name}` : `${path}/${file.name}`
    const url = sftpApi.downloadUrl(selectedConnection, fullPath)
    
    const a = document.createElement('a')
    a.href = url
    a.download = file.name
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
  }, [path, selectedConnection])

  const handleDragStart = (e: React.DragEvent, file: FileInfo) => {
    const fullPath = path === '.' ? file.name : path === '/' ? `/${file.name}` : `${path}/${file.name}`
    e.dataTransfer.setData('application/json', JSON.stringify({
      connectionId: selectedConnection,
      path: fullPath,
      fileName: file.name,
      isDir: file.isDir,
      action: 'copy'
    }))
  }

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault()
    
    // Check for OS files
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0]
      if (files?.some(f => f.name === file.name)) {
        setPendingUpload(file)
        setIsOverwriteDialogOpen(true)
      } else {
        executeUpload(file)
      }
      return
    }

    try {
      const dataStr = e.dataTransfer.getData('application/json')
      if (!dataStr) return
      const data = JSON.parse(dataStr)
      // skip drop on same folder and same file name
      const targetPath = path === '.' || path === '/' ? `/${data.fileName}` : `${path}/${data.fileName}`
      if (data.connectionId === selectedConnection && data.path === targetPath) return

      // Overwrite check
      if (files?.some(f => f.name === data.fileName)) {
        setPendingTransfer({ data, targetPath })
        setIsOverwriteDialogOpen(true)
        return
      }

      executeTransfer(data, targetPath)
    } catch (err) {
      console.error('Drop error:', err)
      toast.error('Failed to transfer file')
    }
  }

  const handleNavigate = useCallback((entry: FileInfo) => {
    setSelectedFile(null)
    if (!entry.isDir) return
    
    if (entry.name === '..') {
      setPath(getParentPath(path))
    } else {
      const newPath = path === '.' 
        ? entry.name 
        : path === '/' 
          ? `/${entry.name}` 
          : `${path}/${entry.name}`
      setPath(newPath)
    }
  }, [path])

  const handleSourceChange = (v: string | null) => {
    if (!v) return
    setSelectedConnection(v)
    setPath(".") // Reset path when source changes
    setSelectedFile(null)
  }

  const shortcutHandlers = useMemo(() => ({
    onEnter: () => {
      if (selectedFile) {
        if (selectedFile.isDir) {
          handleNavigate(selectedFile)
        } else {
          handleDownload(selectedFile)
        }
      }
    },
    onBackspace: () => {
      setPath(getParentPath(path))
      setSelectedFile(null)
    },
    onDelete: () => {
      if (selectedFile) {
        setIsDeleteDialogOpen(true)
      }
    },
    onRefresh: refreshDir,
    onCopy: () => selectedFile && handleAction('copy', selectedFile),
    onCut: () => selectedFile && handleAction('cut', selectedFile),
    onPaste: handlePaste,
  }), [selectedFile, path, refreshDir, handleAction, handlePaste, handleNavigate, handleDownload])

  useSFTPShortcuts(containerRef, shortcutHandlers)

  // Virtual entry for parent directory if not at root/default
  const displayFiles = (() => {
    if (!files) return []
    const sortedFiles = [...files].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1
      if (!a.isDir && b.isDir) return 1
      return a.name.localeCompare(b.name)
    })

    if (path !== '.' && path !== '/') {
      return [
        { name: '..', isDir: true, size: 0, mode: 0, modTime: '' } as FileInfo,
        ...sortedFiles
      ]
    }
    return sortedFiles
  })()

  return (
    <div 
      ref={containerRef}
      tabIndex={0}
      onFocus={() => setIsFocused(true)}
      onBlur={() => setIsFocused(false)}
      className={`flex flex-col h-full bg-background border rounded-lg overflow-hidden transition-all outline-none ${isFocused ? 'ring-1 ring-primary border-primary' : ''}`} 
      onClick={() => setSelectedFile(null)}
    >
      <div className="p-2 border-b bg-muted/50 flex items-center gap-2">
        <Select value={selectedConnection} onValueChange={handleSourceChange}>
          <SelectTrigger className="w-[180px] h-8 text-xs">
            <SelectValue placeholder="Select Source" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="local">Local Machine</SelectItem>
            {connections?.map((conn) => (
              <SelectItem key={conn.id} value={conn.id}>
                {conn.label} ({conn.host})
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div className="flex-1 px-2 overflow-hidden">
          <SftpBreadcrumbs path={path} onNavigate={(newPath) => {
            setPath(newPath)
            setSelectedFile(null)
          }} />
        </div>
      </div>

      <DirectoryToolbar 
        onRefresh={refreshDir}
        onUpload={handleUploadClick}
        onNewFolder={() => setIsNewFolderDialogOpen(true)}
        onRename={() => setIsRenameDialogOpen(true)}
        onDelete={() => setIsDeleteDialogOpen(true)}
        canModify={true}
        hasSelection={!!selectedFile}
      />
      
      <div className="flex-1 overflow-hidden relative">
        <input 
          type="file" 
          ref={fileInputRef} 
          className="hidden" 
          onChange={handleFileChange}
        />

        {isLoading && (
          <div className="absolute inset-0 z-10 bg-background/50 flex items-center justify-center">
            <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
          </div>
        )}
        
        {error ? (
          <div className="p-4 flex flex-col items-center justify-center h-full text-destructive text-sm gap-2 text-center">
            <AlertCircle className="h-8 w-8" />
            <p>Error loading directory</p>
            <p className="text-xs opacity-80">{error instanceof Error ? error.message : String(error)}</p>
          </div>
        ) : (
          <ScrollArea 
            className="h-full" 
            onDragOver={(e) => e.preventDefault()} 
            onDrop={handleDrop}
          >
            <table className="w-full text-sm">
              <thead className="sticky top-0 bg-background border-b z-10">
                <tr className="text-muted-foreground text-left text-xs">
                  <th className="font-medium p-2 pl-4">Name</th>
                  <th className="font-medium p-2 w-24">Size</th>
                  <th className="font-medium p-2 w-40">Modified</th>
                </tr>
              </thead>
              <tbody>
                {displayFiles.length === 0 && !isLoading && (
                  <tr>
                    <td colSpan={3} className="p-4 text-center text-muted-foreground text-xs">
                      Directory is empty
                    </td>
                  </tr>
                )}
                {displayFiles.map((file, i) => {
                  if (file.name === '..') {
                    return (
                      <tr 
                        key={`${file.name}-${i}`}
                        className={`border-b border-border/50 hover:bg-muted/50 transition-colors cursor-pointer select-none`}
                        onDoubleClick={(e) => {
                          e.stopPropagation()
                          handleNavigate(file)
                        }}
                      >
                        <td className="p-2 pl-4 flex items-center gap-2 font-medium">
                          <CornerLeftUp className="h-4 w-4 text-muted-foreground" />
                          <span className="truncate">{file.name}</span>
                        </td>
                        <td className="p-2 text-xs text-muted-foreground whitespace-nowrap">
                        </td>
                        <td className="p-2 text-xs text-muted-foreground whitespace-nowrap">
                        </td>
                      </tr>
                    )
                  }

                  const isSelected = selectedFile?.name === file.name

                  return (
                    <ContextMenu key={`${file.name}-${i}`}>
                      <ContextMenuTrigger render={
                        <tr 
                          className={`border-b border-border/50 hover:bg-muted/50 transition-colors ${file.isDir ? 'cursor-pointer' : ''} select-none ${isSelected ? 'bg-accent text-accent-foreground' : ''}`}
                          onClick={(e) => {
                            e.stopPropagation()
                            setSelectedFile(file)
                          }}
                          onDoubleClick={(e) => {
                            e.stopPropagation()
                            handleNavigate(file)
                          }}
                          draggable
                          onDragStart={(e) => handleDragStart(e as unknown as React.DragEvent, file)}
                        />
                      }>
                        <td className="p-2 pl-4 flex items-center gap-2 font-medium">
                          <FileIcon name={file.name} isDir={file.isDir} />
                          <span className="truncate">{file.name}</span>
                        </td>
                        <td className="p-2 text-xs text-muted-foreground whitespace-nowrap">
                          {!file.isDir ? formatSize(file.size) : ''}
                        </td>
                        <td className="p-2 text-xs text-muted-foreground whitespace-nowrap">
                          {file.modTime ? formatDate(file.modTime) : ''}
                        </td>
                      </ContextMenuTrigger>
                      <ContextMenuContent className="w-48">
                        {!file.isDir && (
                          <>
                            <ContextMenuItem onClick={() => handleDownload(file)}>
                              <Download className="h-4 w-4 mr-2" /> Download
                            </ContextMenuItem>
                            <ContextMenuSeparator />
                          </>
                        )}
                        <ContextMenuItem onClick={() => handleAction('cut', file)}>Cut</ContextMenuItem>
                        <ContextMenuItem onClick={() => handleAction('copy', file)}>Copy</ContextMenuItem>
                        <ContextMenuItem disabled={!sftpClipboard} onClick={handlePaste}>Paste Here</ContextMenuItem>
                        <ContextMenuSeparator />
                        <ContextMenuItem onClick={() => {
                          setSelectedFile(file)
                          setIsRenameDialogOpen(true)
                        }}>Rename</ContextMenuItem>
                        <ContextMenuItem className="text-destructive focus:text-destructive" onClick={() => {
                          setSelectedFile(file)
                          setIsDeleteDialogOpen(true)
                        }}>Delete</ContextMenuItem>
                      </ContextMenuContent>
                    </ContextMenu>
                  )
                })}
              </tbody>
            </table>
          </ScrollArea>
        )}
      </div>

      <RenameDialog 
        open={isRenameDialogOpen} 
        onOpenChange={setIsRenameDialogOpen} 
        oldName={selectedFile?.name || ''} 
        onConfirm={handleRenameConfirm} 
      />
      <NewFolderDialog 
        open={isNewFolderDialogOpen} 
        onOpenChange={setIsNewFolderDialogOpen} 
        onConfirm={handleMkdirConfirm} 
      />
      <DeleteConfirmDialog 
        open={isDeleteDialogOpen} 
        onOpenChange={setIsDeleteDialogOpen} 
        fileName={selectedFile?.name || ''} 
        onConfirm={handleDeleteConfirm} 
      />
      <OverwriteDialog 
        open={isOverwriteDialogOpen} 
        onOpenChange={setIsOverwriteDialogOpen} 
        fileName={pendingUpload?.name || pendingTransfer?.data.fileName || ''} 
        onConfirm={handleOverwriteConfirm} 
      />
    </div>
  )
}

