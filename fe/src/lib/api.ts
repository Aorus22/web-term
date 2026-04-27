const API_BASE = '/api/connections'

export interface Connection {
  id: string
  label: string
  host: string
  port: number
  username: string
  password?: string
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
