import { ChevronRight, Home } from 'lucide-react'

interface SftpBreadcrumbsProps {
  path: string
  onNavigate: (path: string) => void
}

export function SftpBreadcrumbs({ path, onNavigate }: SftpBreadcrumbsProps) {
  // Loading state – path not yet resolved
  if (!path || path === '.') {
    return (
      <div className="flex items-center text-xs text-muted-foreground">
        <Home className="h-3 w-3 mr-1" />
        <span>Loading...</span>
      </div>
    )
  }

  // Check if path starts with Windows drive like "C:" or "C:\"
  const isWindowsPath = /^[A-Za-z]:/.test(path)

  const segments: { name: string; path: string }[] = []

  if (isWindowsPath) {
    // Windows path like "C:\Users\username"
    const drive = path.substring(0, 2) // e.g. "C:"
    const rest = path.substring(2).replace(/\\/g, '/') // normalize backslashes
    const parts = rest.split('/').filter(Boolean)

    // Drive segment
    segments.push({ name: drive, path: drive + '\\' })

    let currentPath = drive + '\\'
    for (const part of parts) {
      currentPath = currentPath + part + '\\'
      segments.push({ name: part, path: currentPath })
    }
  } else if (path.startsWith('/')) {
    // Unix absolute path
    segments.push({ name: '/', path: '/' })

    const parts = path.split('/').filter(Boolean)
    let currentPath = ''
    for (const part of parts) {
      currentPath = currentPath + '/' + part
      segments.push({ name: part, path: currentPath })
    }
  } else {
    // Relative path (unlikely now, but fallback)
    const parts = path.split('/').filter(Boolean)
    let currentPath = ''
    for (const part of parts) {
      currentPath = currentPath ? currentPath + '/' + part : part
      segments.push({ name: part, path: currentPath })
    }
  }

  return (
    <div className="flex items-center text-xs overflow-hidden">
      {segments.map((segment, i) => (
        <div key={segment.path} className="flex items-center shrink-0">
          {i > 0 && <ChevronRight className="h-3 w-3 text-muted-foreground mx-0.5 shrink-0" />}
          <button
            onClick={() => onNavigate(segment.path)}
            className="px-1 py-0.5 rounded hover:bg-accent hover:text-accent-foreground transition-colors truncate max-w-[120px]"
            title={segment.path}
          >
            {segment.name === '/' ? <Home className="h-3 w-3" /> : segment.name}
          </button>
        </div>
      ))}
    </div>
  )
}
