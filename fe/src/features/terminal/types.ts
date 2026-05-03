export type SessionStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'needs-passphrase' | 'detached'

export interface SSHSession {
  id: string                     // unique session ID (stable for UI)
  type: 'ssh' | 'local'          // session type
  backendId?: string             // canonical backend session ID for re-attachment
  connectionId?: string          // optional link to saved Connection
  host: string
  port: number
  username: string
  label: string                  // display label (connection.label or "user@host")
  status: SessionStatus
  error?: string                 // error message when status is 'error'
  isQuickConnect: boolean        // true if ephemeral quick-connect session
  auth_method?: string           // 'password' or 'key'
  ssh_key_id?: string            // for key-auth connections
  key_name?: string              // display name for passphrase prompt
  key_type?: string              // RSA/Ed25519/ECDSA for badge
  has_passphrase?: boolean       // whether passphrase is needed
  cwd?: string                   // initial working directory for duplicate tab auto-connect
  isRecovered?: boolean          // true if session was restored from backend on reload
}

export interface ConnectOptions {
  type: 'ssh' | 'local'
  // For saved connections — send connection_id, backend fetches password
  connectionId?: string
  // For quick-connect — send credentials directly
  host?: string
  port?: number
  username?: string
  password?: string
  // Key authentication fields
  auth_method?: 'password' | 'key'
  ssh_key_id?: string
  passphrase?: string
  // Initial working directory (for duplicate tab feature)
  cwd?: string
  // Initial terminal size
  rows?: number
  cols?: number
  term?: string  // Terminal type for TERM env var (e.g., "xterm-256color")
}
