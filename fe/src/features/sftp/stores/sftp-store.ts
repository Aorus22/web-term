import { create } from 'zustand'

export interface TransferStatus {
  id: string
  fileName: string
  status: 'pending' | 'active' | 'completed' | 'error'
  type: 'upload' | 'download' | 'transfer'
  progress: number
  bytesTotal: number
  bytesTransferred: number
  error?: string
  startTime: number
}

interface SFTPState {
  transfers: TransferStatus[]
  addTransfer: (transfer: Omit<TransferStatus, 'progress' | 'startTime'>) => void
  updateTransfer: (id: string, updates: Partial<TransferStatus>) => void
  removeTransfer: (id: string) => void
  clearCompleted: () => void
}

export const useSFTPStore = create<SFTPState>((set) => ({
  transfers: [],
  addTransfer: (transfer) => set((state) => ({
    transfers: [
      {
        ...transfer,
        progress: 0,
        startTime: Date.now(),
      },
      ...state.transfers,
    ],
  })),
  updateTransfer: (id, updates) => set((state) => ({
    transfers: state.transfers.map((t) =>
      t.id === id ? { ...t, ...updates } : t
    ),
  })),
  removeTransfer: (id) => set((state) => ({
    transfers: state.transfers.filter((t) => t.id !== id),
  })),
  clearCompleted: () => set((state) => ({
    transfers: state.transfers.filter((t) => t.status !== 'completed' && t.status !== 'error'),
  })),
}))
