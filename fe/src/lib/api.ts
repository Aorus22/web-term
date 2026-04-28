const API_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/connections`
const KEYS_API_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/keys`
const FORWARDS_API_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/forwards`

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
  list: (): Promise<Connection[]> => fetch(API_BASE).then(r => r.json()),
  get: (id: string): Promise<Connection> => fetch(`${API_BASE}/${id}`).then(r => r.json()),
  create: (data: Partial<Connection>): Promise<Connection> =>
    fetch(API_BASE, { 
        method: 'POST', 
        headers: {'Content-Type': 'application/json'}, 
        body: JSON.stringify(data) 
    }).then(r => r.json()),
  update: (id: string, data: Partial<Connection>): Promise<Connection> =>
    fetch(`${API_BASE}/${id}`, { 
        method: 'PUT', 
        headers: {'Content-Type': 'application/json'}, 
        body: JSON.stringify(data) 
    }).then(r => r.json()),
  delete: (id: string): Promise<void> =>
    fetch(`${API_BASE}/${id}`, { method: 'DELETE' }).then(r => { 
        if (!r.ok) throw new Error('Delete failed') 
    }),
  export: (): Promise<Blob> =>
    fetch(`${API_BASE}/export`).then(r => r.blob()),
  import: (data: Connection[]): Promise<{imported: number, skipped: number}> =>
    fetch(`${API_BASE}/import`, { 
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
  list: (): Promise<PortForward[]> => fetch(FORWARDS_API_BASE).then(r => r.json()),
  create: (data: Omit<PortForward, 'id' | 'active' | 'error' | 'created_at' | 'updated_at'>): Promise<PortForward> =>
    fetch(FORWARDS_API_BASE, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    }).then(r => {
      if (!r.ok) return r.json().then(e => Promise.reject(e))
      return r.json()
    }),
  delete: (id: string): Promise<void> =>
    fetch(`${FORWARDS_API_BASE}/${id}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error('Delete failed')
    }),
  start: (id: string): Promise<PortForward> =>
    fetch(`${FORWARDS_API_BASE}/${id}/start`, { method: 'POST' }).then(r => {
      if (!r.ok) return r.json().then(e => Promise.reject(e))
      return r.json()
    }),
  stop: (id: string): Promise<PortForward> =>
    fetch(`${FORWARDS_API_BASE}/${id}/stop`, { method: 'POST' }).then(r => {
      if (!r.ok) throw new Error('Failed to stop forward')
      return r.json()
    }),
}

export const keysApi = {
  list: (): Promise<SSHKey[]> => fetch(KEYS_API_BASE).then(r => r.json()),
  get: (id: string): Promise<SSHKey> => fetch(`${KEYS_API_BASE}/${id}`).then(r => r.json()),
  create: (data: { name: string; key_base64: string }): Promise<SSHKey> =>
    fetch(KEYS_API_BASE, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    }).then(r => r.json()),
  update: (id: string, data: { name: string }): Promise<SSHKey> =>
    fetch(`${KEYS_API_BASE}/${id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data)
    }).then(r => r.json()),
  delete: (id: string): Promise<{ warning?: string; affected_connections?: number } | void> =>
    fetch(`${KEYS_API_BASE}/${id}`, { method: 'DELETE' }).then(r => {
      if (!r.ok) throw new Error('Delete failed')
      if (r.status === 200) return r.json()
    }),
}

const SETTINGS_API_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/settings`

export type SettingsResponse = { settings: Record<string, string> }

export const settingsApi = {
  list: (): Promise<SettingsResponse> =>
    fetch(SETTINGS_API_BASE).then(r => r.json()),
  update: (settings: Record<string, string>): Promise<SettingsResponse> =>
    fetch(SETTINGS_API_BASE, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ settings }),
    }).then(r => r.json()),
}
