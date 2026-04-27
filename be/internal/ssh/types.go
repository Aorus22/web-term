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
	ConnectionID string `json:"connection_id,omitempty"` // Saved connection ID (alternative to host/user/password)
	Rows         int    `json:"rows,omitempty"`         // Initial terminal rows
	Cols         int    `json:"cols,omitempty"`         // Initial terminal cols
}

// ResizeMessage is a JSON control message for terminal resize.
type ResizeMessage struct {
	Type string `json:"type"` // "resize"
	Cols int    `json:"cols"`
	Rows int    `json:"rows"`
}

// ServerMessage is a structured response sent from the backend to the client.
type ServerMessage struct {
	Type      string `json:"type"`                  // "connected", "error", "disconnected"
	SessionID string `json:"session_id,omitempty"`  // unique per WebSocket connection
	Message   string `json:"message,omitempty"`      // error/info message
}
