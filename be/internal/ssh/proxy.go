package ssh

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"sync"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
	"gorm.io/gorm"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  4096,
	WriteBufferSize: 4096,
}

func HandleWebSocket(database *gorm.DB, cfg *config.Config) http.HandlerFunc {
	upgrader.CheckOrigin = func(r *http.Request) bool {
		origin := r.Header.Get("Origin")
		if origin == "" {
			return true // Allow non-browser clients (curl/etc)
		}
		for _, allowed := range cfg.AllowedOrigins {
			if origin == allowed {
				return true
			}
		}
		log.Printf("WebSocket origin rejected: %s", origin)
		return false
	}

	return func(w http.ResponseWriter, r *http.Request) {
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Printf("WebSocket upgrade error: %v", err)
			return
		}
		defer conn.Close()

		var wsMu sync.Mutex
		wsWrite := func(msgType int, data []byte) error {
			wsMu.Lock()
			defer wsMu.Unlock()
			return conn.WriteMessage(msgType, data)
		}

		// Step 1: Read the initial connect message (text frame).
		_, msgData, err := conn.ReadMessage()
		if err != nil {
			return
		}

		var connectMsg ConnectMessage
		if err := json.Unmarshal(msgData, &connectMsg); err != nil {
			sendWSError(wsWrite, "Invalid JSON connect message")
			return
		}

		if connectMsg.Type != "connect" {
			sendWSError(wsWrite, "Expected connect message")
			return
		}

		// Generate session ID early for logging and response.
		sessionID := uuid.New().String()

		// Resolve connection credentials: either from saved connection_id or direct fields.
		if connectMsg.ConnectionID != "" {
			// Saved connection flow: fetch from DB and decrypt password.
			var conn db.Connection
			if err := database.First(&conn, "id = ?", connectMsg.ConnectionID); err != nil {
				log.Printf("Connection not found: %s (session %s)", connectMsg.ConnectionID, sessionID)
				sendWSError(wsWrite, "Connection not found")
				return
			}
			decrypted, err := config.Decrypt(conn.Encrypted, cfg.EncryptionKey)
			if err != nil {
				log.Printf("Failed to decrypt password for connection %s (session %s): %v", connectMsg.ConnectionID, sessionID, err)
				sendWSError(wsWrite, "Failed to decrypt connection credentials")
				return
			}
			connectMsg.Host = conn.Host
			connectMsg.Port = conn.Port
			connectMsg.User = conn.Username
			connectMsg.Password = decrypted
		}

		// Security Fix CR-03: SSRF Protection (applies to both flows, including DB-fetched hosts)
		if !cfg.IsHostAllowed(connectMsg.Host) {
			log.Printf("SSRF Blocked: attempt to connect to unauthorized host %s (session %s)", connectMsg.Host, sessionID)
			sendWSError(wsWrite, "Host not authorized by security policy")
			return
		}

		if connectMsg.Port <= 0 || connectMsg.Port > 65535 {
			connectMsg.Port = 22
		}

		// Step 2: Dial SSH.
		addr := fmt.Sprintf("%s:%d", connectMsg.Host, connectMsg.Port)
		sshConfig := &ssh.ClientConfig{
			User: connectMsg.User,
			Auth: []ssh.AuthMethod{
				ssh.Password(connectMsg.Password),
			},
			HostKeyCallback: ssh.InsecureIgnoreHostKey(), // TODO: Implement host key verification
			Timeout:         10 * time.Second,
		}

		client, err := ssh.Dial("tcp", addr, sshConfig)
		if err != nil {
			sendWSError(wsWrite, fmt.Sprintf("SSH connection failed: %v", err))
			return
		}
		defer client.Close()

		session, err := client.NewSession()
		if err != nil {
			sendWSError(wsWrite, fmt.Sprintf("SSH session failed: %v", err))
			return
		}
		defer session.Close()

		// Step 3: Request PTY.
		if connectMsg.Rows == 0 {
			connectMsg.Rows = 24
		}
		if connectMsg.Cols == 0 {
			connectMsg.Cols = 80
		}
		modes := ssh.TerminalModes{
			ssh.ECHO:          1,
			ssh.TTY_OP_ISPEED: 14400,
			ssh.TTY_OP_OSPEED: 14400,
		}
		if err := session.RequestPty("xterm-256color", connectMsg.Rows, connectMsg.Cols, modes); err != nil {
			return
		}

		stdinPipe, _ := session.StdinPipe()
		stdoutPipe, _ := session.StdoutPipe()
		stderrPipe, _ := session.StderrPipe()

		if err := session.Shell(); err != nil {
			return
		}

		// Notify client that connection is ready with session_id.
		connectedMsg := ServerMessage{
			Type:      "connected",
			SessionID: sessionID,
		}
		connectedJSON, _ := json.Marshal(connectedMsg)
		wsWrite(websocket.TextMessage, connectedJSON)

		_, cancel := context.WithCancel(context.Background())
		defer cancel()
		done := make(chan struct{}, 3)

		// Forwarding goroutines
		go func() {
			defer func() { done <- struct{}{} }()
			for {
				msgType, msgData, err := conn.ReadMessage()
				if err != nil {
					cancel()
					return
				}
				if msgType == websocket.TextMessage {
					var ctrl ResizeMessage
					if err := json.Unmarshal(msgData, &ctrl); err == nil && ctrl.Type == "resize" {
						if ctrl.Cols > 0 && ctrl.Rows > 0 {
							winSize := struct {
								Cols uint32
								Rows uint32
								W    uint32
								H    uint32
							}{uint32(ctrl.Cols), uint32(ctrl.Rows), 0, 0}
							session.SendRequest("window-change", false, ssh.Marshal(winSize))
						}
					}
				} else if msgType == websocket.BinaryMessage {
					stdinPipe.Write(msgData)
				}
			}
		}()

		forward := func(src io.Reader) {
			defer func() { done <- struct{}{} }()
			buf := make([]byte, 4096)
			for {
				n, err := src.Read(buf)
				if n > 0 {
					if err := wsWrite(websocket.BinaryMessage, buf[:n]); err != nil {
						cancel()
						return
					}
				}
				if err != nil {
					cancel()
					return
				}
			}
		}

		go forward(stdoutPipe)
		go forward(stderrPipe)

		<-done
		cancel()
	}
}

func sendWSError(wsWrite func(int, []byte) error, message string) {
	// Security Fix CR-04: Use json.Marshal instead of manual escaping
	resp, _ := json.Marshal(map[string]string{
		"type":    "error",
		"message": message,
	})
	wsWrite(websocket.TextMessage, resp)
}
