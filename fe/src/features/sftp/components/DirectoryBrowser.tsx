import { useState, useRef, useMemo, useCallback, useEffect } from 'react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { connectionsApi, sftpApi } from '@/lib/api'
import type { FileInfo } from '@/lib/api'
import { CornerLeftUp, Download, RefreshCw, Upload, FolderPlus, Edit2, Trash2 } from 'lucide-react'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useAppStore } from '@/stores/app-store'
import { RenameDialog, NewFolderDialog, DeleteConfirmDialog, OverwriteDialog } from './OperationDialogs'
import { useSFTPTransfer } from '@/hooks/use-sftp-transfer'
import { toast } from 'sonner'
import { SftpBreadcrumbs } from './SftpBreadcrumbs'
import { useSFTPShortcuts } from '../hooks/use-sftp-shortcuts'
import { FileIcon } from './FileIcon'

// ── Helpers ──

const formatSize = (size: number) => {
  if (size === 0) return '- -'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(size) / Math.log(k))
  return parseFloat((size / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

const formatDate = (dateString: string) => {
  if (!dateString) return ''
  try {
    const d = new Date(dateString)
    return d.toLocaleDateString('en-US', { month: 'numeric', day: 'numeric', year: 'numeric' }) +
           ', ' + d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit' })
  } catch { return dateString }
}

const formatPermissions = (mode: number, isDir: boolean): string => {
  const r = (m: number) => mode & m ? 'r' : '-'
  const w = (m: number) => mode & m ? 'w' : '-'
  const x = (m: number) => mode & m ? 'x' : '-'
  return (isDir ? 'd' : '-') + r(0o400) + w(0o200) + x(0o100) + r(0o040) + w(0o020) + x(0o010) + r(0o004) + w(0o002) + x(0o001)
}

const getFileKind = (file: FileInfo): string => {
  if (file.isDir) return 'folder'
  const ext = file.name.split('.').pop()?.toLowerCase()
  const map: Record<string, string> = {
    png: 'PNG image', jpg: 'JPEG image', jpeg: 'JPEG image', gif: 'GIF image', svg: 'SVG image',
    mp3: 'MP3 audio', wav: 'WAV audio', flac: 'FLAC audio',
    mp4: 'MP4 video', mkv: 'MKV video',
    zip: 'ZIP archive', tar: 'TAR archive', gz: 'GZip archive', rar: 'RAR archive',
    pdf: 'PDF document', doc: 'DOC document', docx: 'DOCX document',
    txt: 'Text', md: 'Markdown', json: 'JSON', xml: 'XML', yaml: 'YAML', yml: 'YAML',
    ts: 'TypeScript', tsx: 'TSX', js: 'JavaScript', jsx: 'JSX',
    py: 'Python', rs: 'Rust', go: 'Go', java: 'Java', cpp: 'C++', c: 'C',
    css: 'CSS', scss: 'SCSS', html: 'HTML',
    sh: 'Shell script', bash: 'Shell script',
  }
  return ext && map[ext] ? map[ext] : (ext ? ext.toUpperCase() + ' file' : 'File')
}

const getParentPath = (currentPath: string) => {
  if (!currentPath || currentPath === '.' || currentPath === '/' || currentPath === '') return '/'
  if (/^[A-Za-z]:\\$/.test(currentPath)) return '/'
  if (/^[A-Za-z]:/.test(currentPath)) {
    const cleanPath = currentPath.replace(/\\+$/, '')
    const parts = cleanPath.split('\\')
    parts.pop()
    if (parts.length === 1) return '/'
    return parts.join('\\')
  }
  const cleanPath = currentPath.replace(/\/+$/, '')
  const parts = cleanPath.split('/')
  parts.pop()
  if (parts.length === 0) return '/'
  if (parts.length === 1 && parts[0] === '') return '/'
  return parts.join('/')
}

// ── Component ──

interface DirectoryBrowserProps {
  panelId: 'left' | 'right'
}

export function DirectoryBrowser({ panelId }: DirectoryBrowserProps) {
  const {
    sftpLeftPanel, sftpRightPanel,
    setSftpLeftPanel, setSftpRightPanel,
    sftpClipboard, setSftpClipboard,
  } = useAppStore()

  const panel = panelId === 'left' ? sftpLeftPanel : sftpRightPanel
  const setPanel = panelId === 'left' ? setSftpLeftPanel : setSftpRightPanel

  const selectedConnection = panel.connectionId
  const path = panel.path

  const [selectedFile, setSelectedFile] = useState<FileInfo | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  
  const [isRenameDialogOpen, setIsRenameDialogOpen] = useState(false)
  const [isNewFolderDialogOpen, setIsNewFolderDialogOpen] = useState(false)
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false)
  const [isOverwriteDialogOpen, setIsOverwriteDialogOpen] = useState(false)
  const [pendingUpload, setPendingUpload] = useState<File | null>(null)
  const [pendingTransfer, setPendingTransfer] = useState<{data: any, targetPath: string} | null>(null)

  const fileInputRef = useRef<HTMLInputElement>(null)
  const queryClient = useQueryClient()
  const { trackTransfer } = useSFTPTransfer()
  
  const { data: connections } = useQuery({
    queryKey: ['connections'],
    queryFn: () => connectionsApi.list()
  })

  // Resolve home directory when path is empty
  useEffect(() => {
    if (path) return
    const fetchHome = async () => {
      try {
        const { path: homePath } = await sftpApi.home(selectedConnection)
        setPanel({ connectionId: selectedConnection, path: homePath })
      } catch (e) {
        console.error('Failed to fetch home directory:', e)
        setPanel({ connectionId: selectedConnection, path: '/' })
      }
    }
    fetchHome()
  }, [selectedConnection, path, setPanel])

  const { data: files, isLoading, error } = useQuery({
    queryKey: ['sftp', selectedConnection, path],
    queryFn: () => sftpApi.list(selectedConnection, path),
    enabled: !!selectedConnection && !!path
  })

  const refreshDir = useCallback(() => {
    queryClient.invalidateQueries({ queryKey: ['sftp', selectedConnection, path] })
  }, [queryClient, selectedConnection, path])

  // ── Transfer / Upload / Operations (unchanged logic) ──

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

  const handleAction = useCallback((action: 'cut' | 'copy', file: FileInfo) => {
    const fullPath = path === '/' ? `/${file.name}` : `${path}/${file.name}`
    setSftpClipboard({ action, connectionId: selectedConnection, path: fullPath, fileName: file.name, isDir: file.isDir })
  }, [path, selectedConnection, setSftpClipboard])

  const handlePaste = useCallback(async () => {
    if (!sftpClipboard) return
    const targetPath = path === '/' ? `/${sftpClipboard.fileName}` : `${path}/${sftpClipboard.fileName}`
    if (files?.some(f => f.name === sftpClipboard.fileName)) {
      setPendingTransfer({ data: sftpClipboard, targetPath })
      setIsOverwriteDialogOpen(true)
      return
    }
    executeTransfer(sftpClipboard, targetPath)
  }, [sftpClipboard, path, files, executeTransfer])

  const handleRenameConfirm = async (newName: string) => {
    if (!selectedFile || !newName || newName === selectedFile.name) return
    const oldPath = path === '/' ? `/${selectedFile.name}` : `${path}/${selectedFile.name}`
    const newPath = path === '/' ? `/${newName}` : `${path}/${newName}`
    try { await sftpApi.rename(selectedConnection, oldPath, newPath); refreshDir(); setSelectedFile(null) }
    catch (e: any) { toast.error('Failed to rename') }
  }

  const handleDeleteConfirm = async () => {
    if (!selectedFile) return
    const fullPath = path === '/' ? `/${selectedFile.name}` : `${path}/${selectedFile.name}`
    try { await sftpApi.remove(selectedConnection, fullPath); refreshDir(); setSelectedFile(null) }
    catch (e: any) { toast.error('Failed to delete') }
  }

  const handleMkdirConfirm = async (name: string) => {
    if (!name) return
    const fullPath = path === '/' ? `/${name}` : `${path}/${name}`
    try { await sftpApi.mkdir(selectedConnection, fullPath); refreshDir() }
    catch (e: any) { toast.error('Failed to create folder') }
  }

  const handleUploadClick = () => fileInputRef.current?.click()

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    if (files?.some(f => f.name === file.name)) { setPendingUpload(file); setIsOverwriteDialogOpen(true) }
    else executeUpload(file)
    e.target.value = ''
  }

  const executeUpload = useCallback(async (file: File) => {
    const targetPath = path === '/' ? `/${file.name}` : `${path}/${file.name}`
    try {
      const { transferId } = await sftpApi.upload(selectedConnection, targetPath, file)
      trackTransfer(transferId, file.name, 'upload')
      refreshDir()
    } catch (e: any) { toast.error(`Failed to upload ${file.name}`) }
  }, [path, selectedConnection, trackTransfer, refreshDir])

  const handleOverwriteConfirm = () => {
    if (pendingUpload) { executeUpload(pendingUpload); setPendingUpload(null) }
    else if (pendingTransfer) { executeTransfer(pendingTransfer.data, pendingTransfer.targetPath); setPendingTransfer(null) }
    setIsOverwriteDialogOpen(false)
  }

  const handleDownload = useCallback((file: FileInfo) => {
    const fullPath = path === '/' ? `/${file.name}` : `${path}/${file.name}`
    const url = sftpApi.downloadUrl(selectedConnection, fullPath)
    const a = document.createElement('a'); a.href = url; a.download = file.name
    document.body.appendChild(a); a.click(); document.body.removeChild(a)
  }, [path, selectedConnection])

  const handleDragStart = (e: React.DragEvent, file: FileInfo) => {
    const fullPath = path === '/' ? `/${file.name}` : `${path}/${file.name}`
    e.dataTransfer.setData('application/json', JSON.stringify({
      connectionId: selectedConnection, path: fullPath, fileName: file.name, isDir: file.isDir, action: 'copy'
    }))
  }

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault()
    if (e.dataTransfer.files && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0]
      if (files?.some(f => f.name === file.name)) { setPendingUpload(file); setIsOverwriteDialogOpen(true) }
      else executeUpload(file)
      return
    }
    try {
      const dataStr = e.dataTransfer.getData('application/json')
      if (!dataStr) return
      const data = JSON.parse(dataStr)
      const targetPath = path === '/' ? `/${data.fileName}` : `${path}/${data.fileName}`
      if (data.connectionId === selectedConnection && data.path === targetPath) return
      if (files?.some(f => f.name === data.fileName)) { setPendingTransfer({ data, targetPath }); setIsOverwriteDialogOpen(true); return }
      executeTransfer(data, targetPath)
    } catch (err) { toast.error('Failed to transfer file') }
  }

  const handleNavigate = useCallback((newPath: string) => {
    setSelectedFile(null)
    setPanel({ connectionId: selectedConnection, path: newPath })
  }, [selectedConnection, setPanel])

  const handleEntryClick = useCallback((entry: FileInfo) => {
    setSelectedFile(null)
    if (!entry.isDir) return
    if (entry.name === '..') {
      setPanel({ connectionId: selectedConnection, path: getParentPath(path) })
    } else {
      const newPath = path === '/' ? `/${entry.name}` : `${path}/${entry.name}`
      setPanel({ connectionId: selectedConnection, path: newPath })
    }
  }, [path, selectedConnection, setPanel])

  const handleSourceChange = (v: string | null) => {
    if (!v) return
    setPanel({ connectionId: v, path: '' })
    setSelectedFile(null)
  }

  const shortcutHandlers = useMemo(() => ({
    onEnter: () => { if (selectedFile) selectedFile.isDir ? handleEntryClick(selectedFile) : handleDownload(selectedFile) },
    onBackspace: () => { setPanel({ connectionId: selectedConnection, path: getParentPath(path) }); setSelectedFile(null) },
    onDelete: () => { if (selectedFile) setIsDeleteDialogOpen(true) },
    onRefresh: refreshDir,
    onCopy: () => selectedFile && handleAction('copy', selectedFile),
    onCut: () => selectedFile && handleAction('cut', selectedFile),
    onPaste: handlePaste,
  }), [selectedFile, path, selectedConnection, setPanel, refreshDir, handleAction, handlePaste, handleEntryClick, handleDownload])

  useSFTPShortcuts(containerRef, shortcutHandlers)

  // ── Display files ──
  const displayFiles = (() => {
    if (!files) return []
    const sorted = [...files].sort((a, b) => {
      if (a.isDir && !b.isDir) return -1
      if (!a.isDir && b.isDir) return 1
      return a.name.toLowerCase().localeCompare(b.name.toLowerCase())
    })
    if (path !== '.' && path !== '/' && !(/^[A-Za-z]:\\$/.test(path))) {
      return [{ name: '..', isDir: true, size: 0, mode: 0, modTime: '' } as FileInfo, ...sorted]
    }
    return sorted
  })()

  const isLoadingHome = !path

  return (
    <div 
      ref={containerRef}
      tabIndex={0}
      className="flex flex-col h-full bg-background outline-none overflow-hidden"
      onClick={() => setSelectedFile(null)}
    >
      {/* ── Toolbar ── */}
      <div className="flex items-center gap-2.5 px-3 py-1.5 bg-muted border-b shrink-0">
        <Select value={selectedConnection} onValueChange={handleSourceChange}>
          <SelectTrigger className="h-7 border-0 bg-transparent text-sm font-semibold px-1 hover:bg-accent/50 w-auto min-w-0 [&>svg]:text-muted-foreground [&>svg]:h-3 [&>svg]:w-3">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="local">Local Machine</SelectItem>
            {connections?.map((conn) => (
              <SelectItem key={conn.id} value={conn.id}>{conn.label} ({conn.host})</SelectItem>
            ))}
          </SelectContent>
        </Select>
        <div className="flex-1" />
        <DropdownMenu>
          <DropdownMenuTrigger className="flex items-center gap-1.5 bg-transparent border-0 text-muted-foreground text-xs cursor-pointer py-1 px-2 rounded hover:bg-accent hover:text-foreground transition-colors outline-none">
            Actions
            <svg className="w-3 h-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="6 9 12 15 18 9"/></svg>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-44">
            <DropdownMenuItem onClick={refreshDir}>
              <RefreshCw className="h-4 w-4" /> Refresh
            </DropdownMenuItem>
            <DropdownMenuItem onClick={handleUploadClick}>
              <Upload className="h-4 w-4" /> Upload File
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setIsNewFolderDialogOpen(true)}>
              <FolderPlus className="h-4 w-4" /> New Folder
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onClick={() => setIsRenameDialogOpen(true)} disabled={!selectedFile}>
              <Edit2 className="h-4 w-4" /> Rename
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setIsDeleteDialogOpen(true)} disabled={!selectedFile} variant="destructive">
              <Trash2 className="h-4 w-4" /> Delete
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* ── Breadcrumb ── */}
      <SftpBreadcrumbs path={path} onNavigate={handleNavigate} />

      {/* ── Column headers ── */}
      <div className="grid grid-cols-[1fr_150px_60px_80px] px-3 py-1 bg-muted/80 border-b text-muted-foreground text-[11px] font-semibold tracking-wider uppercase shrink-0">
        <div>Name</div>
        <div>Date Modified</div>
        <div>Size</div>
        <div>Kind</div>
      </div>

      {/* ── File list ── */}
      <div className="flex-1 overflow-hidden relative">
        <input type="file" ref={fileInputRef} className="hidden" onChange={handleFileChange} />

        {(isLoadingHome || isLoading) && (
          <div className="absolute inset-0 z-10 bg-background/70 flex items-center justify-center">
            <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-t-transparent" />
          </div>
        )}
        
        {!isLoadingHome && error ? (
          <div className="p-6 flex flex-col items-center justify-center h-full text-destructive text-sm gap-2 text-center">
            <div className="text-base">Error loading directory</div>
            <div className="text-xs opacity-70">{error instanceof Error ? error.message : String(error)}</div>
          </div>
        ) : !isLoadingHome && (
          <div 
            className="h-full overflow-y-auto"
            onDragOver={(e) => e.preventDefault()} 
            onDrop={handleDrop}
          >
            {displayFiles.length === 0 && (
              <div className="p-4 text-center text-muted-foreground text-xs">Directory is empty</div>
            )}
            {displayFiles.map((file, i) => {
              if (file.name === '..') {
                return (
                  <div 
                    key={`${file.name}-${i}`}
                    className="grid grid-cols-[1fr_150px_60px_80px] items-center px-3 h-[38px] cursor-pointer transition-colors hover:bg-accent"
                    onDoubleClick={(e) => { e.stopPropagation(); handleEntryClick(file) }}
                  >
                    <div className="flex items-center gap-2 overflow-hidden">
                      <CornerLeftUp className="h-[18px] w-[18px] text-muted-foreground shrink-0" />
                      <span className="text-[13px] text-foreground truncate">..</span>
                    </div>
                    <div className="text-muted-foreground text-[11.5px]" />
                    <div className="text-muted-foreground text-[11.5px] text-center" />
                    <div className="text-muted-foreground text-[11.5px]" />
                  </div>
                )
              }

              const isSelected = selectedFile?.name === file.name

              return (
                <ContextMenu key={`${file.name}-${i}`}>
                  <ContextMenuTrigger render={
                    <div 
                      className={`grid grid-cols-[1fr_150px_60px_80px] items-center px-3 h-[38px] cursor-pointer transition-colors ${isSelected ? 'bg-accent text-accent-foreground' : 'hover:bg-accent'}`}
                      onClick={(e) => { e.stopPropagation(); setSelectedFile(file) }}
                      onDoubleClick={(e) => { e.stopPropagation(); handleEntryClick(file) }}
                      draggable
                      onDragStart={(e) => handleDragStart(e as unknown as React.DragEvent, file)}
                    />
                  }>
                    <div className="flex items-center gap-2 overflow-hidden">
                      {file.isDir ? (
                        <svg viewBox="0 0 24 24" className="w-[18px] h-[18px] shrink-0"><path fill="currentColor" className="text-blue-500" d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z"/></svg>
                      ) : (
                        <FileIcon name={file.name} isDir={false} />
                      )}
                      <div className="flex flex-col overflow-hidden min-w-0">
                        <span className="text-[13px] text-foreground truncate leading-tight">{file.name}</span>
                        <span className="text-[10px] text-muted-foreground leading-tight">{formatPermissions(file.mode, file.isDir)}</span>
                      </div>
                    </div>
                    <div className="text-muted-foreground text-[11.5px]">{file.modTime ? formatDate(file.modTime) : ''}</div>
                    <div className="text-muted-foreground text-[11.5px] text-center">{file.isDir ? '- -' : formatSize(file.size)}</div>
                    <div className="text-muted-foreground text-[11.5px]">{getFileKind(file)}</div>
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
                    <ContextMenuItem onClick={() => { setSelectedFile(file); setIsRenameDialogOpen(true) }}>Rename</ContextMenuItem>
                    <ContextMenuItem className="text-destructive focus:text-destructive" onClick={() => { setSelectedFile(file); setIsDeleteDialogOpen(true) }}>Delete</ContextMenuItem>
                  </ContextMenuContent>
                </ContextMenu>
              )
            })}
          </div>
        )}
      </div>

      {/* ── Dialogs ── */}
      <RenameDialog open={isRenameDialogOpen} onOpenChange={setIsRenameDialogOpen} oldName={selectedFile?.name || ''} onConfirm={handleRenameConfirm} />
      <NewFolderDialog open={isNewFolderDialogOpen} onOpenChange={setIsNewFolderDialogOpen} onConfirm={handleMkdirConfirm} />
      <DeleteConfirmDialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen} fileName={selectedFile?.name || ''} onConfirm={handleDeleteConfirm} />
      <OverwriteDialog open={isOverwriteDialogOpen} onOpenChange={setIsOverwriteDialogOpen} fileName={pendingUpload?.name || pendingTransfer?.data.fileName || ''} onConfirm={handleOverwriteConfirm} />
    </div>
  )
}
