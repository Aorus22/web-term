import { create } from 'zustand'
import type { Connection } from '@/lib/api'
import type { SSHSession } from '@/features/terminal/types'
import { generateId } from '@/lib/utils'

interface AppState {
  sidebarOpen: boolean
  toggleSidebar: () => void
  sidebarPage: 'hosts' | 'keys' | 'forwards' | 'settings' | 'new-tab'
  setSidebarPage: (page: 'hosts' | 'keys' | 'forwards' | 'settings' | 'new-tab') => void
  editingConnection: Connection | null
  setEditingConnection: (c: Connection | null) => void
  creatingConnection: boolean
  setCreatingConnection: (v: boolean) => void
  selectedTags: string[]
  toggleTag: (tag: string) => void
  clearTags: () => void
  // Session state
  sessions: SSHSession[]
  activeSessionId: string | null
  addSession: (session: SSHSession) => void
  removeSession: (id: string) => void
  updateSession: (id: string, updates: Partial<SSHSession>) => void
  setActiveSession: (id: string | null) => void
  duplicateSession: (sessionId: string, cwd: string) => string | null
  rehydrateSessions: () => Promise<void>
  // Backend port for dynamic discovery
  backendPort: number
  setBackendPort: (port: number) => void
}

export const useAppStore = create<AppState>((set) => ({
  sidebarOpen: true,
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
  sidebarPage: 'new-tab',
  setSidebarPage: (page) => set({ sidebarPage: page }),
  editingConnection: null,
  setEditingConnection: (c) => set({ editingConnection: c }),
  creatingConnection: false,
  setCreatingConnection: (v) => set({ creatingConnection: v }),
  selectedTags: [],
  toggleTag: (tag) => set((state) => ({
    selectedTags: state.selectedTags.includes(tag)
      ? state.selectedTags.filter((t) => t !== tag)
      : [...state.selectedTags, tag]
  })),
  clearTags: () => set({ selectedTags: [] }),
  // Session state
  sessions: [],
  activeSessionId: null,
  addSession: (session) => set((state) => ({
    sessions: [...state.sessions, session],
    activeSessionId: session.id,
  })),
  removeSession: (id) => set((state) => ({
    sessions: state.sessions.filter((s) => s.id !== id),
    activeSessionId: state.activeSessionId === id
      ? (state.sessions.find((s) => s.id !== id)?.id ?? null)
      : state.activeSessionId,
  })),
  updateSession: (id, updates) => set((state) => ({
    sessions: state.sessions.map((s) => s.id === id ? { ...s, ...updates } : s),
  })),
  setActiveSession: (id) => set({ activeSessionId: id }),
  duplicateSession: (sessionId, cwd) => {
    const state = useAppStore.getState()
    const source = state.sessions.find((s) => s.id === sessionId)
    if (!source) return null
    const newId = generateId()
    const newSession: SSHSession = {
      ...source,
      id: newId,
      status: 'connecting',
      error: undefined,
      cwd: cwd || undefined,
    }
    set((state) => ({
      sessions: [...state.sessions, newSession],
      activeSessionId: newId,
    }))
    return newId
  },
  rehydrateSessions: async () => {
    const { sessionsApi } = await import('@/lib/api')
    try {
      const backendSessions = await sessionsApi.list()
      if (backendSessions.length === 0) return

      set((state) => {
        const existingIds = new Set(state.sessions.map((s) => s.id))
        const newSessions: SSHSession[] = backendSessions
          .filter((bs) => !existingIds.has(bs.id))
          .map((bs) => ({
            id: bs.id, // Use backend ID as stable local ID for recovered sessions
            backendId: bs.id,
            connectionId: bs.connection_id,
            host: bs.host,
            port: bs.port,
            username: bs.user,
            label: `${bs.user}@${bs.host}`,
            status: 'detached',
            isQuickConnect: !bs.connection_id,
            cwd: bs.cwd,
            isRecovered: true,
          }))

        if (newSessions.length === 0) return state

        const updatedSessions = [...state.sessions, ...newSessions]
        return {
          sessions: updatedSessions,
          activeSessionId: state.activeSessionId ?? updatedSessions[0].id,
        }
      })
    } catch (error) {
      console.error('Failed to rehydrate sessions:', error)
    }
  },
  backendPort: 0,
  setBackendPort: (port) => set({ backendPort: port }),
}))
