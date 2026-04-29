package ssh

// ConnectMessage is the initial JSON message from client with SSH connection params.
// Supports two flows:
//   - Quick-connect: client provides Host/Port/User/Password directly
//   - Saved connection: client provides ConnectionID, backend fetches from DB
type ConnectMessage struct {
	Type         string `json:"type"`                  // "connect"
	Host         string `json:"host"`                  // SSH hostname (quick-connect)
	Port         int    `json:"port"`                  // SSH port (quick-connect)
	User         string `json:"user"`                  // SSH username (quick-connect)
	Password     string `json:"password"`              // SSH password (quick-connect)
	AuthMethod   string `json:"auth_method,omitempty"` // "password" or "key"
	SSHKeyID     string `json:"ssh_key_id,omitempty"`   // SSH key ID for key-based auth
	Passphrase   string `json:"passphrase,omitempty"`   // Passphrase for encrypted private keys
	ConnectionID string `json:"connection_id,omitempty"` // Saved connection ID (alternative to host/user/password)
	Cwd          string `json:"cwd,omitempty"`          // Initial working directory for new session (duplicate tab)
	Rows         int    `json:"rows,omitempty"`         // Initial terminal rows
	Cols         int    `json:"cols,omitempty"`         // Initial terminal cols
	Term         string `json:"term,omitempty"`         // Terminal type for TERM env var (e.g., "xterm-256color")
}

// ResizeMessage is a JSON control message for terminal resize.
type ResizeMessage struct {
	Type string `json:"type"` // "resize"
	Cols int    `json:"cols"`
	Rows int    `json:"rows"`
}

// ServerMessage is a structured response sent from the backend to the client.
type ServerMessage struct {
	Type      string `json:"type"`                 // "connected", "error", "disconnected"
	SessionID string `json:"session_id,omitempty"` // unique per WebSocket connection
	Message   string `json:"message,omitempty"`    // error/info message
}

// CwdResponseMessage is sent in response to a "get-cwd" WebSocket message.
type CwdResponseMessage struct {
	Type string `json:"type"` // "cwd"
	Path string `json:"path"` // the current working directory path
}
