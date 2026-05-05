import { ChevronRight, Home } from 'lucide-react'

interface SftpBreadcrumbsProps {
  path: string
  onNavigate: (path: string) => void
}

export function SftpBreadcrumbs({ path, onNavigate }: SftpBreadcrumbsProps) {
  // If path is '.', treat it as "Home/Current"
  if (path === '.') {
    return (
      <div className="flex items-center text-xs">
        <button 
          onClick={() => onNavigate('.')}
          className="flex items-center gap-1 px-1 py-0.5 rounded hover:bg-accent hover:text-accent-foreground transition-colors"
        >
          <Home className="h-3 w-3" />
          <span>Current Directory</span>
        </button>
      </div>
    )
  }

  const isAbsolute = path.startsWith('/')
  const parts = path.split('/').filter(Boolean)
  
  const segments = []
  
  // Add root if absolute
  if (isAbsolute) {
    segments.push({
      name: '/',
      path: '/',
      isRoot: true
    })
  }

  let currentPath = isAbsolute ? '' : ''
  parts.forEach((part) => {
    currentPath = isAbsolute 
      ? (currentPath === '' ? '/' + part : currentPath + '/' + part)
      : (currentPath === '' ? part : currentPath + '/' + part)
    
    segments.push({
      name: part,
      path: currentPath,
      isRoot: false
    })
  })

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
            {segment.isRoot ? <Home className="h-3 w-3" /> : segment.name}
          </button>
        </div>
      ))}
    </div>
  )
}
