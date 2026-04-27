export type SessionStatus = 'connecting' | 'connected' | 'disconnected' | 'error'

export interface SSHSession {
  id: string                     // unique session ID (matches backend session_id)
  connectionId?: string          // optional link to saved Connection
  host: string
  port: number
  username: string
  label: string                  // display label (connection.label or "user@host")
  status: SessionStatus
  error?: string                 // error message when status is 'error'
  isQuickConnect: boolean        // true if ephemeral quick-connect session
}

export interface ConnectOptions {
  // For saved connections — send connection_id, backend fetches password
  connectionId?: string
  // For quick-connect — send credentials directly
  host?: string
  port?: number
  username?: string
  password?: string
  // Initial terminal size
  rows?: number
  cols?: number
}
