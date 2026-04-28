import { create } from 'zustand'
import type { Connection } from '@/lib/api'
import type { SSHSession } from '@/features/terminal/types'

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
}))
