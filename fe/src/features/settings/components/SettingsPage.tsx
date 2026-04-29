import { useState } from 'react'
import { Monitor, Moon, Sun, Type, MousePointer2, History, Terminal, Check } from 'lucide-react'
import { useSettings, useUpdateSettings } from '../hooks/useSettings'
import type { AppSettings } from '../hooks/useSettings'
import { terminalThemes } from '../data/terminal-themes'
import { useTheme } from '@/hooks/use-theme'
import type { Theme } from '@/hooks/use-theme'
import { cn } from '@/lib/utils'
import { Switch } from '@/components/ui/switch'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { FontDialog } from './FontDialog'

export function SettingsPage() {
  const { data: settings, isLoading } = useSettings()
  const { mutate: updateSettings } = useUpdateSettings()
  const { theme: appTheme, setTheme: setAppTheme } = useTheme()
  const [fontDialogOpen, setFontDialogOpen] = useState(false)

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
    <div className="flex flex-col items-center justify-start min-h-full p-8 overflow-y-auto pt-12">
      <div className="w-full max-w-2xl space-y-8 pb-12">
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>

        {/* Appearance Section */}
        <div className="space-y-4">
          <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">Appearance</h2>
          <div className="bg-card rounded-lg border divide-y overflow-hidden">
            <div className="flex items-center justify-between px-4 py-3">
              <div className="flex flex-col gap-0.5">
                <span className="text-sm font-medium">Theme Mode</span>
                <span className="text-xs text-muted-foreground">App color scheme</span>
              </div>
              <div className="flex bg-muted p-1 rounded-md">
                <button
                  onClick={() => setAppTheme('light')}
                  className={cn(
                    "flex items-center gap-2 px-3 py-1.5 rounded-sm text-xs font-medium transition-all",
                    appTheme === 'light' ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
                  )}
                >
                  <Sun className="h-3.5 w-3.5" /> Light
                </button>
                <button
                  onClick={() => setAppTheme('dark')}
                  className={cn(
                    "flex items-center gap-2 px-3 py-1.5 rounded-sm text-xs font-medium transition-all",
                    appTheme === 'dark' ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
                  )}
                >
                  <Moon className="h-3.5 w-3.5" /> Dark
                </button>
                <button
                  onClick={() => setAppTheme('system')}
                  className={cn(
                    "flex items-center gap-2 px-3 py-1.5 rounded-sm text-xs font-medium transition-all",
                    appTheme === 'system' ? "bg-background text-foreground shadow-sm" : "text-muted-foreground hover:text-foreground"
                  )}
                >
                  <Monitor className="h-3.5 w-3.5" /> System
                </button>
              </div>
            </div>

            <div className="px-4 py-4 space-y-4">
              <div className="flex flex-col gap-0.5">
                <span className="text-sm font-medium">Terminal Color Theme</span>
                <span className="text-xs text-muted-foreground">ANSI and background colors</span>
              </div>
              <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
                {terminalThemes.map((theme) => (
                  <button
                    key={theme.name}
                    onClick={() => handleUpdate('terminal_color_theme', theme.name)}
                    className={cn(
                      "group relative flex flex-col gap-2 p-2 rounded-lg border-2 transition-all text-left",
                      settings.terminal_color_theme === theme.name
                        ? "border-primary/20 bg-accent/40"
                        : "border-transparent bg-muted/30 hover:bg-muted/50"
                    )}
                  >
                    <div 
                      className="h-16 w-full rounded-md border border-muted flex flex-col p-2 gap-1 overflow-hidden"
                      style={{ backgroundColor: theme.colors.background }}
                    >
                      <div className="flex gap-1">
                        <div className="h-2 w-2 rounded-full" style={{ backgroundColor: theme.colors.red }} />
                        <div className="h-2 w-2 rounded-full" style={{ backgroundColor: theme.colors.green }} />
                        <div className="h-2 w-2 rounded-full" style={{ backgroundColor: theme.colors.yellow }} />
                      </div>
                      <div className="flex-1 flex items-end">
                        <div className="w-3/4 h-1 rounded-sm" style={{ backgroundColor: theme.colors.foreground }} />
                      </div>
                    </div>
                    <span className="text-xs font-medium truncate px-1 text-muted-foreground group-hover:text-foreground">
                      {theme.label}
                    </span>
                    {settings.terminal_color_theme === theme.name && (
                      <Check className="absolute top-2 right-2 h-3.5 w-3.5 text-primary/50" />
                    )}
                  </button>
                ))}
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
                onValueChange={(v) => handleUpdate('terminal_type', v)}
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
                <Type className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium">Font</span>
              </div>
              <button 
                onClick={() => setFontDialogOpen(true)}
                className="text-xs font-mono bg-muted px-2 py-1 rounded border hover:bg-muted/80 transition-colors"
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
                onValueChange={(v) => handleUpdate('cursor_style', v)}
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
                onValueChange={(v) => handleUpdate('scrollback', v)}
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
