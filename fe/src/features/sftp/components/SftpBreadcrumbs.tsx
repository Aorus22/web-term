interface SftpBreadcrumbsProps {
  path: string
  onNavigate: (path: string) => void
}

function FolderSvg() {
  return (
    <svg viewBox="0 0 24 24" className="w-[13px] h-[13px] shrink-0">
      <path fill="currentColor" className="text-blue-500" d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z"/>
    </svg>
  )
}

export function SftpBreadcrumbs({ path, onNavigate }: SftpBreadcrumbsProps) {
  if (!path) {
    return (
      <div className="flex items-center gap-1.5 px-3 py-1.5 bg-muted/80 border-b text-muted-foreground text-xs shrink-0">
        <span>Loading...</span>
      </div>
    )
  }

  const isWindowsPath = /^[A-Za-z]:/.test(path)
  const segments: { name: string; path: string }[] = []

  if (isWindowsPath) {
    const drive = path.substring(0, 2)
    const rest = path.substring(2).replace(/\\/g, '/')
    const parts = rest.split('/').filter(Boolean)
    segments.push({ name: drive, path: drive + '\\' })
    let currentPath = drive + '\\'
    for (const part of parts) {
      currentPath = currentPath + part + '\\'
      segments.push({ name: part, path: currentPath })
    }
  } else if (path.startsWith('/')) {
    segments.push({ name: 'root', path: '/' })
    const parts = path.split('/').filter(Boolean)
    let currentPath = ''
    for (const part of parts) {
      currentPath = currentPath + '/' + part
      segments.push({ name: part, path: currentPath })
    }
  } else {
    const parts = path.split('/').filter(Boolean)
    let currentPath = ''
    for (const part of parts) {
      currentPath = currentPath ? currentPath + '/' + part : part
      segments.push({ name: part, path: currentPath })
    }
  }

  return (
    <div className="flex items-center gap-1 px-3 py-1.5 bg-muted/80 border-b shrink-0">
      {/* Nav arrows (placeholder for future history) */}
      <button className="text-muted-foreground hover:text-foreground px-0.5 py-0.5 rounded text-sm leading-none transition-colors cursor-default" tabIndex={-1}>&#8249;</button>
      <button className="text-muted-foreground hover:text-foreground px-0.5 py-0.5 rounded text-sm leading-none transition-colors cursor-default" tabIndex={-1}>&#8250;</button>

      <div className="flex items-center gap-0.5 text-xs">
        {segments.map((segment, i) => (
          <span key={segment.path} className="flex items-center gap-0.5 shrink-0">
            {i > 0 && <span className="text-muted-foreground/50 text-[10px] mx-0.5">&#8250;</span>}
            <FolderSvg />
            <button
              onClick={() => onNavigate(segment.path)}
              className="px-0.5 py-0.5 rounded hover:bg-accent transition-colors text-foreground truncate max-w-[120px]"
              title={segment.path}
            >
              {segment.name === 'root' ? '/' : segment.name}
            </button>
          </span>
        ))}
      </div>
    </div>
  )
}
