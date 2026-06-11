import { useState, useEffect, useMemo } from 'react'
import { Type, MousePointer2, History, Terminal, Check, Cog, Paintbrush } from 'lucide-react'
import { useSettings, useUpdateSettings } from '../hooks/useSettings'
import type { AppSettings } from '../hooks/useSettings'
import { themes } from '../data/themes'
import { cn } from '@/lib/utils'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useAppTheme, setAppTheme, getLuminance } from '@/components/AppThemeProvider'
import { FontDialog } from './FontDialog'

type ThemeMode = 'all' | 'dark' | 'light'

export function SettingsPage() {
  const { data: settings, isLoading } = useSettings()
  const { mutate: updateSettings } = useUpdateSettings()
  const [fontDialogOpen, setFontDialogOpen] = useState(false)
  const [current, setCurrent] = useState(() => useAppTheme())
  const [mode, setMode] = useState<ThemeMode>(() => {
    const stored = typeof window !== 'undefined' ? localStorage.getItem('web-term-theme-mode') : null
    return (stored === 'all' || stored === 'dark' || stored === 'light') ? stored : 'all'
  })

  useEffect(() => {
    const handler = () => setCurrent(useAppTheme())
    window.addEventListener('storage', handler)
    return () => window.removeEventListener('storage', handler)
  }, [])

  const handleSelect = (name: string) => {
    setAppTheme(name)
    setCurrent(name)
  }

  const filtered = useMemo(() => {
    if (mode === 'all') return themes
    return themes.filter(t => {
      const lum = getLuminance(t.colors.background)
      return mode === 'dark' ? lum < 0.5 : lum >= 0.5
    })
  }, [mode])

  // Persist mode filter to localStorage
  useEffect(() => {
    localStorage.setItem('web-term-theme-mode', mode)
  }, [mode])

  // Auto-match theme when switching filter mode
  useEffect(() => {
    if (mode === 'all') return
    const currentTheme = themes.find(t => t.name === current)
    if (!currentTheme) return
    const currentLum = getLuminance(currentTheme.colors.background)
    const matches = mode === 'dark' ? currentLum < 0.5 : currentLum >= 0.5
    if (!matches) {
      const base = currentTheme.name.replace(/-(dark|light)$/, '')
      const counterpart = mode === 'dark' ? `${base}-dark` : `${base}-light`
      const found = themes.find(t => t.name === counterpart)
      if (found) {
        setAppTheme(found.name)
        setCurrent(found.name)
      }
    }
  }, [mode, current])

  if (isLoading || !settings) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
      </div>
    )
  }

  const handleUpdate = (key: keyof AppSettings, value: string | boolean) => {
    updateSettings({ [key]: value })
  }

  return (
    <div className="flex flex-col items-center justify-start min-h-full p-4 md:p-8 overflow-y-auto pt-12">
      <div className="w-full max-w-2xl space-y-8 pb-12">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>

        {/* Appearance Section */}
        <div className="space-y-4">
          <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">Appearance</h2>
          <div className="bg-card rounded-lg border divide-y overflow-hidden">
            {/* Theme Mode Filter */}
            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <Paintbrush className="h-4 w-4 text-muted-foreground" />
                <div className="flex flex-col">
                  <span className="text-sm font-medium">Theme Mode</span>
                  <span className="text-xs text-muted-foreground">Filter by dark or light appearance</span>
                </div>
              </div>
              <Select value={mode} onValueChange={(v) => setMode(v as ThemeMode)}>
                <SelectTrigger className="w-[110px] h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All themes</SelectItem>
                  <SelectItem value="dark">Dark</SelectItem>
                  <SelectItem value="light">Light</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* UI Theme Cards */}
            <div className="px-4 py-4 space-y-4">
              <div className="flex flex-col gap-0.5">
                <span className="text-sm font-medium">Color Theme</span>
                <span className="text-xs text-muted-foreground">{filtered.length} themes available</span>
              </div>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {filtered.map((theme) => {
                  const isActive = current === theme.name
                  const c = theme.colors
                  return (
                    <button
                      key={theme.name}
                      onClick={() => handleSelect(theme.name)}
                      className={cn(
                        "group relative flex flex-col gap-2 p-2 rounded-lg border-2 transition-all text-left",
                        isActive
                          ? "border-primary/40 bg-accent"
                          : "border-border/50 bg-secondary hover:bg-secondary/80"
                      )}
                    >
                      <div
                        className="h-14 w-full rounded-md flex flex-col p-2 gap-1 overflow-hidden"
                        style={{ backgroundColor: c.background }}
                      >
                        <div className="flex gap-1">
                          <div className="h-2 w-2 rounded-full" style={{ backgroundColor: c.primary }} />
                          <div className="h-2 w-2 rounded-full" style={{ backgroundColor: c.accent }} />
                          <div className="h-2 w-2 rounded-full" style={{ backgroundColor: c.destructive }} />
                        </div>
                        <div className="flex-1 flex items-end gap-1">
                          <div className="w-2/3 h-1 rounded-sm" style={{ backgroundColor: c.foreground }} />
                          <div className="w-1/4 h-1 rounded-sm" style={{ backgroundColor: c.mutedForeground }} />
                        </div>
                      </div>
                      <span className="text-xs font-medium truncate px-1 text-muted-foreground group-hover:text-foreground">
                        {theme.label}
                      </span>
                      {isActive && (
                        <Check className="absolute top-2 right-2 h-3.5 w-3.5 text-primary" />
                      )}
                    </button>
                  )
                })}
              </div>
            </div>

          </div>
        </div>

        {/* Terminal Section */}
        <div className="space-y-4">
          <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">Terminal</h2>
          <div className="bg-card rounded-lg border divide-y overflow-hidden">
            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <Terminal className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Terminal Type</span>
              </div>
              <Select 
                value={settings.terminal_type} 
                onValueChange={(v) => handleUpdate('terminal_type', v ?? '')}
              >
                <SelectTrigger className="w-[180px] h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="xterm-256color">xterm-256color</SelectItem>
                  <SelectItem value="xterm">xterm</SelectItem>
                  <SelectItem value="xterm-color">xterm-color</SelectItem>
                  <SelectItem value="vt100">vt100</SelectItem>
                  <SelectItem value="vt220">vt220</SelectItem>
                  <SelectItem value="ansi">ansi</SelectItem>
                  <SelectItem value="linux">linux</SelectItem>
                  <SelectItem value="screen-256color">screen-256color</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <Cog className="h-4 w-4 text-muted-foreground" />
                <div className="flex flex-col">
                  <span className="text-sm font-medium">Terminal Engine</span>
                  <span className="text-xs text-muted-foreground">Choose terminal implementation</span>
                </div>
              </div>
              <Select 
                value={settings.terminal_engine} 
                onValueChange={(v) => handleUpdate('terminal_engine', v ?? 'wterm')}
              >
                <SelectTrigger className="w-[180px] h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="wterm">wterm (Experimental)</SelectItem>
                  <SelectItem value="xterm">xterm.js (Stable)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <Type className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Font</span>
              </div>
              <button 
                onClick={() => setFontDialogOpen(true)}
                className="text-xs bg-muted px-2 py-1 rounded border hover:bg-muted/80 transition-colors"
              >
                {settings.font_family} {settings.font_size}px
              </button>
            </div>

            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <MousePointer2 className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Cursor Style</span>
              </div>
              <Select 
                value={settings.cursor_style} 
                onValueChange={(v) => handleUpdate('cursor_style', v ?? 'block')}
              >
                <SelectTrigger className="w-[120px] h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="block">Block (▮)</SelectItem>
                  <SelectItem value="underline">Underline (_)</SelectItem>
                  <SelectItem value="bar">Bar (|)</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <div className={cn(
                  "h-1.5 w-1.5 rounded-full animate-pulse", 
                  settings.cursor_blink === 'true' ? "bg-primary" : "bg-muted-foreground"
                )} />
                <span className="text-sm font-medium">Cursor Blink</span>
              </div>
              <Switch 
                checked={settings.cursor_blink === 'true'} 
                onCheckedChange={(checked) => handleUpdate('cursor_blink', String(checked))}
              />
            </div>

            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex items-center gap-3">
                <History className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Scrollback Buffer</span>
              </div>
              <Select 
                value={settings.scrollback} 
                onValueChange={(v) => handleUpdate('scrollback', v ?? '1000')}
              >
                <SelectTrigger className="w-[140px] h-8 text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1000">1,000 lines</SelectItem>
                  <SelectItem value="5000">5,000 lines</SelectItem>
                  <SelectItem value="10000">10,000 lines</SelectItem>
                  <SelectItem value="50000">50,000 lines</SelectItem>
                  <SelectItem value="0">Unlimited</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>

      <FontDialog 
        open={fontDialogOpen}
        onOpenChange={setFontDialogOpen}
        currentFont={settings.font_family}
        currentSize={settings.font_size}
        onSave={(font, size) => {
          updateSettings({ font_family: font, font_size: size })
        }}
      />
    </div>
  )
}
