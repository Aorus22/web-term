import { useAppStore } from '@/stores/app-store'
import { isDesktop } from './desktop-ipc'

const getBaseUrl = () => {
  // Mode 1: Desktop (Tauri/Electron)
  // If we have a discovered backend port, prioritize it.
  const port = useAppStore.getState().backendPort
  if (isDesktop && port !== 0) {
    return `http://localhost:${port}`
  }

  // Mode 2: Decoupled (FE served separately, e.g., Vite dev server or Vercel)
  // Prioritize explicit environment variable if set.
  if (import.meta.env.VITE_API_URL) {
    return import.meta.env.VITE_API_URL
  }

  // Mode 3: Unified (FE served by Go Backend)
  // Fallback to the origin the app was loaded from.
  if (typeof window !== 'undefined' && window.location.origin && window.location.origin !== 'null') {
    // If we are in dev mode (localhost:5173), we likely want Mode 2's fallback or .env
    // but if no .env is set, we use origin.
    return window.location.origin
  }

  // Absolute fallback
  return 'http://localhost:8080'
}

const getApiBase = () => `${getBaseUrl()}/api/connections`
const getKeysApiBase = () => `${getBaseUrl()}/api/keys`
const getForwardsApiBase = () => `${getBaseUrl()}/api/forwards`
const getSettingsApiBase = () => `${getBaseUrl()}/api/settings`

export interface SSHKey {
  id: string
  name: string
  fingerprint: string
  key_type: string      // "RSA" | "Ed25519" | "ECDSA"
  has_passphrase: boolean
  created_at: string
  updated_at: string
}

export interface Connection {
  id: string
  label: string
  host: string
  port: number
  username: string
  password?: string
  auth_method: string
  ssh_key_id?: string | null
  tags: string[]
  created_at: string
  updated_at: string
}

export const connectionsApi = {
  list: (): Promise<Connection[]> => fetch(getApiBase()).then(r => r.json()),
  get: (id: string): Promise<Connection> => fetch(`${getApiBase()}/${id}`).then(r => r.json()),
  create: (data: Partial<Connection>): Promise<Connection> =>
    fetch(getApiBase(), { 
        method: 'POST', 
        headers: {'Content-Type': 'application/json'}, 
        body: JSON.stringify(data) 
    }).then(r => r.json()),
  update: (id: string, data: Partial<Connection>): Promise<Connection> =>
    fetch(`${getApiBase()}/${id}`, { 
        method: 'PUT', 
        headers: {'Content-Type': 'application/json'}, 
        body: JSON.stringify(data) 
    }).then(r => r.json()),
  delete: (id: string): Promise<void> =>
    fetch(`${getApiBase()}/${id}`, { method: 'DELETE' }).then(r => { 
        if (!r.ok) throw new Error('Delete failed') 
    }),
  export: (): Promise<Blob> =>
    fetch(`${getApiBase()}/export`).then(r => r.blob()),
  import: (data: Connection[]): Promise<{imported: number, skipped: number}> =>
    fetch(`${getApiBase()}/import`, { 
        method: 'POST', 
        headers: {'Content-Type': 'application/json'}, 
        body: JSON.stringify({connections: data}) 
    }).then(r => r.json()),
}

export interface PortForward {
  id: string
  name: string
  connection_id: string
  local_port: number
  remote_port: number
  active: boolean
  error: string
  created_at: string
  updated_at: string
}

export const forwardsApi = {
  list: (): Promise<PortForward[]> => fetch(getForwardsApiBase()).then(r => r.json()),
  create: (data: Omit<PortForward, 'id' | 'active' | 'error' | 'created_at' | 'updated_at'>): Promise<PortForward> =>
    fetch(getForwardsApiBase(), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }).then(r => {
      if (!r.ok) return r.json().then(e => Promise.reject(e))
      return r.json()
    }),
  delete: (id: string): Promise<void> =>
    fetch(`${getForwardsApiBase()}/${id}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error('Delete failed')
    }),
  start: (id: string): Promise<PortForward> =>
    fetch(`${getForwardsApiBase()}/${id}/start`, { method: 'POST' }).then(r => {
      if (!r.ok) return r.json().then(e => Promise.reject(e))
      return r.json()
    }),
  stop: (id: string): Promise<PortForward> =>
    fetch(`${getForwardsApiBase()}/${id}/stop`, { method: 'POST' }).then(r => {
      if (!r.ok) throw new Error('Failed to stop forward')
      return r.json()
    }),
}

export const keysApi = {
  list: (): Promise<SSHKey[]> => fetch(getKeysApiBase()).then(r => r.json()),
  get: (id: string): Promise<SSHKey> => fetch(`${getKeysApiBase()}/${id}`).then(r => r.json()),
  create: (data: { name: string; key_base64: string }): Promise<SSHKey> =>
    fetch(getKeysApiBase(), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    }).then(r => r.json()),
  update: (id: string, data: { name: string }): Promise<SSHKey> =>
    fetch(`${getKeysApiBase()}/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    }).then(r => r.json()),
  delete: (id: string): Promise<{ warning?: string; affected_connections?: number } | void> =>
    fetch(`${getKeysApiBase()}/${id}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error('Delete failed')
      if (r.status === 200) return r.json()
    }),
}

export type SettingsResponse = { settings: Record<string, string> }

export const settingsApi = {
  list: (): Promise<SettingsResponse> =>
    fetch(getSettingsApiBase()).then(r => r.json()),
  update: (settings: Record<string, string>): Promise<SettingsResponse> =>
    fetch(getSettingsApiBase(), {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ settings }),
    }).then(r => r.json()),
}

export interface BackendSession {
  id: string
  host: string
  user: string
  port: number
  connection_id: string
  status: 'active' | 'detached'
}

export const sessionsApi = {
  list: (): Promise<BackendSession[]> => fetch(`${getBaseUrl()}/api/sessions`).then(r => r.json()),
}
