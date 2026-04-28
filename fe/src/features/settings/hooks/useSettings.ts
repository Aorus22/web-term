import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { settingsApi } from '@/lib/api'

export const DEFAULT_SETTINGS = {
  theme_mode: 'system' as const,
  terminal_color_theme: 'default',
  terminal_type: 'xterm-256color',
  font_family: 'Geist Mono',
  font_size: '14',
  cursor_style: 'block' as const,
  cursor_blink: 'true',
  scrollback: '1000',
}

export type AppSettings = typeof DEFAULT_SETTINGS

export function useSettings() {
  return useQuery({
    queryKey: ['settings'],
    queryFn: async () => {
      const { settings } = await settingsApi.list()
      return { ...DEFAULT_SETTINGS, ...settings } as AppSettings
    },
    staleTime: Infinity, // settings change infrequently
  })
}

export function useUpdateSettings() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (settings: Partial<AppSettings>) => {
      // Convert to Record<string, string> for API
      const params: Record<string, string> = {}
      for (const [k, v] of Object.entries(settings)) {
        if (v !== undefined) params[k] = String(v)
      }
      return settingsApi.update(params)
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['settings'] })
    },
  })
}
