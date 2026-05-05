import { useEffect, useState, useCallback } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { settingsApi } from '@/lib/api'

export type Theme = 'light' | 'dark' | 'system'
export type ResolvedTheme = 'light' | 'dark'

export function useTheme() {
  const queryClient = useQueryClient()
  const [theme, setThemeState] = useState<Theme>(() => {
    // Fallback to localStorage on first load before API data arrives
    return (localStorage.getItem('webterm-theme') as Theme) || 'system'
  })

  const [resolvedTheme, setResolvedTheme] = useState<ResolvedTheme>(() => {
    if (theme !== 'system') return theme as ResolvedTheme
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  })

  // Apply theme to DOM
  useEffect(() => {
    const root = window.document.documentElement
    const applyTheme = (currentTheme: Theme) => {
      const effective: ResolvedTheme = currentTheme === 'system'
        ? (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
        : currentTheme as ResolvedTheme
      setResolvedTheme(effective)
      // root.classList.toggle('dark', effective === 'dark') // DISABLED: AppThemeProvider handles this now
    }
    applyTheme(theme)
    localStorage.setItem('webterm-theme', theme) // keep as fallback

    if (theme === 'system') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)')
      const handler = () => applyTheme('system')
      mq.addEventListener('change', handler)
      return () => mq.removeEventListener('change', handler)
    }
  }, [theme])

  // Sync from settings API data when it loads
  useEffect(() => {
    const settings = queryClient.getQueryData<{ settings: Record<string, string> }>(['settings'])
    if (settings?.settings?.theme_mode && ['light','dark','system'].includes(settings.settings.theme_mode)) {
      setThemeState(settings.settings.theme_mode as Theme)
    }
  }, [queryClient])

  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme)
    // Persist to backend (fire-and-forget, optimistic update)
    settingsApi.update({ theme_mode: newTheme }).then(() => {
      queryClient.invalidateQueries({ queryKey: ['settings'] })
    })
  }, [queryClient])

  return { theme, resolvedTheme, setTheme }
}
