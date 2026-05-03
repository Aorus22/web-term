import { useState } from 'react'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useQuery } from '@tanstack/react-query'
import { connectionsApi, sftpApi } from '@/lib/api'
import type { FileInfo } from '@/lib/api'
import { Folder, File as FileIcon, Loader2, AlertCircle, CornerLeftUp } from 'lucide-react'
import { ScrollArea } from '@/components/ui/scroll-area'

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
  
  const { data: connections } = useQuery({
    queryKey: ['connections'],
    queryFn: () => connectionsApi.list()
  })

  const { data: files, isLoading, error } = useQuery({
    queryKey: ['sftp', selectedConnection, path],
    queryFn: () => sftpApi.list(selectedConnection, path),
    enabled: !!selectedConnection
  })

  const handleNavigate = (entry: FileInfo) => {
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
  }

  const handleSourceChange = (v: string | null) => {
    if (!v) return
    setSelectedConnection(v)
    setPath(".") // Reset path when source changes
  }

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
    <div className="flex flex-col h-full bg-background border rounded-lg overflow-hidden">
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
        <div className="flex-1 px-2 text-xs font-mono text-muted-foreground truncate" title={path === '.' ? 'Current Directory' : path}>
          {path === '.' ? 'Current Directory' : path}
        </div>
      </div>
      
      <div className="flex-1 overflow-hidden relative">
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
          <ScrollArea className="h-full">
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
                {displayFiles.map((file, i) => (
                  <tr 
                    key={`${file.name}-${i}`}
                    className={`border-b border-border/50 hover:bg-muted/50 transition-colors ${file.isDir ? 'cursor-pointer' : ''} select-none`}
                    onDoubleClick={() => handleNavigate(file)}
                  >
                    <td className="p-2 pl-4 flex items-center gap-2 font-medium">
                      {file.name === '..' ? (
                        <CornerLeftUp className="h-4 w-4 text-muted-foreground" />
                      ) : file.isDir ? (
                        <Folder className="h-4 w-4 text-blue-500 fill-blue-500/20" />
                      ) : (
                        <FileIcon className="h-4 w-4 text-muted-foreground" />
                      )}
                      <span className="truncate">{file.name}</span>
                    </td>
                    <td className="p-2 text-xs text-muted-foreground whitespace-nowrap">
                      {!file.isDir && file.name !== '..' ? formatSize(file.size) : ''}
                    </td>
                    <td className="p-2 text-xs text-muted-foreground whitespace-nowrap">
                      {file.name !== '..' && file.modTime ? formatDate(file.modTime) : ''}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </ScrollArea>
        )}
      </div>
    </div>
  )
}
