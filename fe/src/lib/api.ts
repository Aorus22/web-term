const API_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/connections`
const KEYS_API_BASE = `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/api/keys`

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
