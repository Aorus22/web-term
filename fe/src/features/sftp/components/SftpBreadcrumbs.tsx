import { useState } from 'react'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import type { FileInfo } from '@/lib/api'

interface SftpBreadcrumbsProps {
  path: string
  onNavigate: (path: string) => void
  onBack: () => void
  onForward: () => void
  canGoBack: boolean
  canGoForward: boolean
  drives?: FileInfo[]
}

function FolderSvg() {
  return (
    <svg viewBox="0 0 24 24" className="w-3.5 h-3.5 shrink-0 text-blue-500">
      <path fill="currentColor" d="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V7z"/>
    </svg>
  )
}

function DriveSvg() {
  return (
    <svg viewBox="0 0 24 24" className="w-3.5 h-3.5 shrink-0 text-amber-500">
      <path fill="currentColor" d="M6 2h12a2 2 0 012 2v16a2 2 0 01-2 2H6a2 2 0 01-2-2V4a2 2 0 012-2zm0 2v16h12V4H6zm2 2h8v4H8V6zm0 6h8v2H8v-2z"/>
    </svg>
  )
}

function CrumbBtn({ name, path, onNavigate }: { name: string; path: string; onNavigate: (p: string) => void }) {
  return (
    <button
      onClick={() => onNavigate(path)}
      className="px-1 py-0.5 rounded hover:bg-accent transition-colors truncate max-w-[140px]"
      title={path}
    >
      {name === 'root' ? '/' : name}
    </button>
  )
}

export function SftpBreadcrumbs({ path, onNavigate, onBack, onForward, canGoBack, canGoForward, drives }: SftpBreadcrumbsProps) {
  const [popoverOpen, setPopoverOpen] = useState(false)
  const [drivePopoverOpen, setDrivePopoverOpen] = useState(false)

  if (!path) {
    return (
      <div className="flex items-center px-3 py-2 bg-muted/80 border-b text-muted-foreground text-sm shrink-0">
        <span>Loading...</span>
      </div>
    )
  }

  // ── Build segments ──
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

  // ── Collapse logic ──
  const MAX = 4
  const TAIL = 2
  const collapsed = segments.length > MAX
  const head = collapsed ? segments.slice(0, 1) : []
  const tail = collapsed ? segments.slice(-TAIL) : segments
  const hidden = collapsed ? segments.slice(1, segments.length - TAIL) : []

  // Available drives (for Windows volume switching)
  const driveEntries = drives?.filter(d => d.isDir && /^[A-Z]:$/.test(d.name)) || []

  return (
    <div className="flex items-center gap-1.5 px-3 py-2 bg-muted/80 border-b shrink-0 overflow-hidden">
      {/* Nav arrows with history */}
      <button
        onClick={onBack}
        disabled={!canGoBack}
        className={`px-1.5 rounded text-lg leading-none transition-colors shrink-0 ${canGoBack ? 'text-muted-foreground hover:text-foreground cursor-pointer' : 'text-muted-foreground/20 cursor-default'}`}
      >&#8249;</button>
      <button
        onClick={onForward}
        disabled={!canGoForward}
        className={`px-1.5 rounded text-lg leading-none transition-colors shrink-0 ${canGoForward ? 'text-muted-foreground hover:text-foreground cursor-pointer' : 'text-muted-foreground/20 cursor-default'}`}
      >&#8250;</button>

      <div className="flex items-center text-sm min-w-0 overflow-hidden">
        {!collapsed ? (
          // ── Not collapsed: show all ──
          segments.map((seg, i) => {
            const isDrive = isWindowsPath && i === 0
            return (
              <span key={seg.path} className="flex items-center shrink-0">
                {i > 0 && <span className="text-muted-foreground/30 text-xs mx-1 shrink-0">&#8250;</span>}
                {isDrive ? (
                  <DriveSvg />
                ) : (
                  <FolderSvg />
                )}
                {isDrive && driveEntries.length > 1 ? (
                  <Popover open={drivePopoverOpen} onOpenChange={setDrivePopoverOpen}>
                    <PopoverTrigger className="flex items-center gap-0.5 px-1 py-0.5 rounded hover:bg-accent transition-colors outline-none shrink-0 cursor-pointer">
                      <span className="truncate max-w-[80px]">{seg.name}</span>
                      <svg className="w-3 h-3 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                        <polyline points="6 9 12 15 18 9"/>
                      </svg>
                    </PopoverTrigger>
                    <PopoverContent align="start" side="bottom" sideOffset={4} className="w-44 p-1.5">
                      <div className="text-xs text-muted-foreground px-2 py-1">Volumes</div>
                      {driveEntries.map((d) => (
                        <button
                          key={d.name}
                          onClick={() => { onNavigate(d.name + '\\'); setDrivePopoverOpen(false) }}
                          className="flex items-center gap-2 w-full px-2 py-1.5 rounded hover:bg-accent text-sm transition-colors text-left"
                        >
                          <DriveSvg />
                          <span>{d.name}</span>
                        </button>
                      ))}
                    </PopoverContent>
                  </Popover>
                ) : (
                  <CrumbBtn name={seg.name} path={seg.path} onNavigate={onNavigate} />
                )}
              </span>
            )
          })
        ) : (
          // ── Collapsed: head + ... + tail ──
          <>
            {/* Head (root) */}
            {head.map((seg) => {
              const isDrive = isWindowsPath
              return (
                <span key={seg.path} className="flex items-center shrink-0">
                  {isDrive ? (
                    <DriveSvg />
                  ) : (
                    <FolderSvg />
                  )}
                  {isDrive && driveEntries.length > 1 ? (
                    <Popover open={drivePopoverOpen} onOpenChange={setDrivePopoverOpen}>
                      <PopoverTrigger className="flex items-center gap-0.5 px-1 py-0.5 rounded hover:bg-accent transition-colors outline-none shrink-0 cursor-pointer">
                        <span className="truncate max-w-[80px]">{seg.name}</span>
                        <svg className="w-3 h-3 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                          <polyline points="6 9 12 15 18 9"/>
                        </svg>
                      </PopoverTrigger>
                      <PopoverContent align="start" side="bottom" sideOffset={4} className="w-44 p-1.5">
                        <div className="text-xs text-muted-foreground px-2 py-1">Volumes</div>
                        {driveEntries.map((d) => (
                          <button
                            key={d.name}
                            onClick={() => { onNavigate(d.name + '\\'); setDrivePopoverOpen(false) }}
                            className="flex items-center gap-2 w-full px-2 py-1.5 rounded hover:bg-accent text-sm transition-colors text-left"
                          >
                            <DriveSvg />
                            <span>{d.name}</span>
                          </button>
                        ))}
                      </PopoverContent>
                    </Popover>
                  ) : (
                    <CrumbBtn name={seg.name} path={seg.path} onNavigate={onNavigate} />
                  )}
                </span>
              )
            })}

            {/* Ellipsis popover */}
            <Popover open={popoverOpen} onOpenChange={setPopoverOpen}>
              <PopoverTrigger className="inline-flex items-center mx-0.5 px-1 py-0.5 rounded hover:bg-accent text-muted-foreground hover:text-foreground transition-colors text-xs font-bold shrink-0 tracking-wider outline-none">
                ...
              </PopoverTrigger>
              <PopoverContent align="start" side="bottom" sideOffset={6} className="w-52 p-1.5">
                <div className="text-xs text-muted-foreground px-2 py-1">Path</div>
                {hidden.map((hs) => (
                  <button
                    key={hs.path}
                    onClick={() => { onNavigate(hs.path); setPopoverOpen(false) }}
                    className="flex items-center gap-2 w-full px-2 py-1.5 rounded hover:bg-accent text-sm transition-colors text-left"
                  >
                    <FolderSvg />
                    <span className="truncate">{hs.name}</span>
                  </button>
                ))}
              </PopoverContent>
            </Popover>

            {/* Tail segments */}
            {tail.map((seg) => (
              <span key={seg.path} className="flex items-center shrink-0">
                <span className="text-muted-foreground/30 text-xs mx-1 shrink-0">&#8250;</span>
                <FolderSvg />
                <CrumbBtn name={seg.name} path={seg.path} onNavigate={onNavigate} />
              </span>
            ))}
          </>
        )}
      </div>
    </div>
  )
}
