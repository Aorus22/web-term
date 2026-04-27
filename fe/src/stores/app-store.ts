import { create } from 'zustand'
import type { Connection } from '@/lib/api'

interface AppState {
  sidebarOpen: boolean
  toggleSidebar: () => void
  editingConnection: Connection | null
  setEditingConnection: (c: Connection | null) => void
  creatingConnection: boolean
  setCreatingConnection: (v: boolean) => void
  selectedTags: string[]
  toggleTag: (tag: string) => void
  clearTags: () => void
}

export const useAppStore = create<AppState>((set) => ({
  sidebarOpen: true,
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
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
}))
