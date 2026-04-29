package ssh

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
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

		var keySigner ssh.Signer

		// Resolve connection credentials: either from saved connection_id or direct fields.
		if connectMsg.ConnectionID != "" {
			// Saved connection flow: fetch from DB.
			var conn db.Connection
			if err := database.First(&conn, "id = ?", connectMsg.ConnectionID).Error; err != nil {
				log.Printf("Connection not found: %s (session %s)", connectMsg.ConnectionID, sessionID)
				sendWSError(wsWrite, "Connection not found")
				return
			}

			// Determine auth method
			if connectMsg.AuthMethod == "key" && conn.AuthMethod == "key" && conn.SSHKeyID != nil {
				// KEY AUTH PATH
				var key db.SSHKey
				if err := database.First(&key, "id = ?", *conn.SSHKeyID).Error; err != nil {
					log.Printf("SSH Key not found: %s (session %s)", *conn.SSHKeyID, sessionID)
					sendWSError(wsWrite, "SSH key not found")
					return
				}

				decryptedKey, err := config.DecryptWithAAD(key.EncryptedKey, cfg.EncryptionKey, []byte(config.SSHKeyAAD))
				if err != nil {
					log.Printf("Failed to decrypt SSH key for connection %s (session %s): %v", connectMsg.ConnectionID, sessionID, err)
					sendWSError(wsWrite, "Failed to decrypt SSH key")
					return
				}

				var parsedKey interface{}
				if connectMsg.Passphrase != "" {
					parsedKey, err = ssh.ParseRawPrivateKeyWithPassphrase([]byte(decryptedKey), []byte(connectMsg.Passphrase))
				} else {
					parsedKey, err = ssh.ParseRawPrivateKey([]byte(decryptedKey))
				}

				if err != nil {
					log.Printf("Failed to parse SSH key for connection %s (session %s): %v", connectMsg.ConnectionID, sessionID, err)
					sendWSError(wsWrite, "Failed to parse SSH key: invalid passphrase or corrupted key")
					return
				}

				keySigner, err = ssh.NewSignerFromKey(parsedKey)
				if err != nil {
					log.Printf("Failed to create SSH signer for connection %s (session %s): %v", connectMsg.ConnectionID, sessionID, err)
					sendWSError(wsWrite, "Failed to create SSH signer from key")
					return
				}

				connectMsg.Host = conn.Host
				connectMsg.Port = conn.Port
				connectMsg.User = conn.Username
			} else if connectMsg.AuthMethod == "key" || conn.AuthMethod == "key" {
				// Mismatch or missing key ID
				if conn.AuthMethod == "key" && conn.SSHKeyID == nil {
					sendWSError(wsWrite, "Connection configured for key auth but no SSH key assigned")
				} else {
					sendWSError(wsWrite, "Authentication method mismatch")
				}
				return
			} else {
				// EXISTING PASSWORD PATH
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
			User:            connectMsg.User,
			HostKeyCallback: ssh.InsecureIgnoreHostKey(), // TODO: Implement host key verification
			Timeout:         10 * time.Second,
		}

		if keySigner != nil {
			// Key-based auth
			sshConfig.Auth = []ssh.AuthMethod{ssh.PublicKeys(keySigner)}
		} else {
			// Password-based auth (existing flow)
			sshConfig.Auth = []ssh.AuthMethod{
				ssh.Password(connectMsg.Password),
				ssh.KeyboardInteractive(func(user, instruction string, questions []string, echos []bool) ([]string, error) {
					answers := make([]string, len(questions))
					for i := range answers {
						answers[i] = connectMsg.Password
					}
					return answers, nil
				}),
			}
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
		termType := connectMsg.Term
		if termType == "" {
			termType = "xterm-256color"
		}
		if err := session.RequestPty(termType, connectMsg.Rows, connectMsg.Cols, modes); err != nil {
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

		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		// Capture the shell PID before starting forwarding goroutines.
		// If cwd is specified, cd into it first. Then echo the PID marker.
		// All output is collected and forwarded to the client in one batch.
		const pidMarker = "__WTERM_PID__"
		{
			var initCmd string
			if connectMsg.Cwd != "" && !strings.Contains(connectMsg.Cwd, "'") && !strings.Contains(connectMsg.Cwd, "\n") {
				initCmd = fmt.Sprintf(" cd '%s' && echo %s$$\n", connectMsg.Cwd, pidMarker)
			} else {
				initCmd = fmt.Sprintf(" echo %s$$\n", pidMarker, )
			}
			stdinPipe.Write([]byte(initCmd))
		}

		var shellPid int
		{
			pidBuf := make([]byte, 8192)
			var collected []byte
			readDeadline := time.After(5 * time.Second)
		pidLoop:
			for {
				select {
				case <-readDeadline:
					break pidLoop
				default:
				}
				n, err := stdoutPipe.Read(pidBuf)
				if n > 0 {
					collected = append(collected, pidBuf[:n]...)
					if bytes.Contains(collected, []byte(pidMarker)) {
						break
					}
				}
				if err != nil {
					break
				}
			}
			// Forward collected data to client, but filter out PID marker lines
			// so the init command is invisible to the user.
			var filtered []byte
			for _, line := range bytes.Split(collected, []byte("\n")) {
				if !bytes.Contains(line, []byte(pidMarker)) {
					filtered = append(filtered, line...)
					filtered = append(filtered, '\n')
				}
			}
			if len(filtered) > 0 {
				wsWrite(websocket.BinaryMessage, filtered)
			}
			// Parse the PID from the marker line.
			for _, line := range strings.Split(string(collected), "\n") {
				trimmed := strings.TrimSpace(line)
				if idx := strings.Index(trimmed, pidMarker); idx >= 0 {
					pidStr := strings.TrimSpace(trimmed[idx+len(pidMarker):])
					shellPid, _ = strconv.Atoi(pidStr)
				}
			}
		}
		if shellPid > 0 {
			log.Printf("Session %s: captured shell PID %d", sessionID, shellPid)
		}

		// Forwarding goroutines
		go func() {
			for {
				select {
				case <-ctx.Done():
					return
				default:
				}
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
					} else if err == nil && ctrl.Type == "get-cwd" {
						go func(pid int) {
							path := ""
							if pid > 0 {
								s, sessErr := client.NewSession()
								if sessErr != nil {
									log.Printf("get-cwd: NewSession failed: %v (pid=%d)", sessErr, pid)
								} else {
									defer s.Close()
									cmd := fmt.Sprintf("readlink /proc/%d/cwd 2>/dev/null", pid)
									out, outErr := s.Output(cmd)
									if outErr != nil {
										log.Printf("get-cwd: readlink failed: %v (pid=%d)", outErr, pid)
									} else {
										path = strings.TrimSpace(string(out))
										log.Printf("get-cwd: pid=%d path=%q", pid, path)
									}
								}
							} else {
								log.Printf("get-cwd: shellPid is 0, cannot resolve cwd")
							}
							if path == "" {
								path = "/"
							}
							cwdResp, _ := json.Marshal(CwdResponseMessage{Type: "cwd", Path: path})
							wsWrite(websocket.TextMessage, cwdResp)
						}(shellPid)
					}
				} else if msgType == websocket.BinaryMessage {
					stdinPipe.Write(msgData)
				}
			}
		}()

		forward := func(src io.Reader) {
			buf := make([]byte, 4096)
			for {
				select {
				case <-ctx.Done():
					return
				default:
				}
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

		// Wait for context cancellation (any goroutine signals done).
		<-ctx.Done()

		// Send disconnected message so frontend knows session ended cleanly.
		disconnectedMsg := ServerMessage{
			Type:      "disconnected",
			SessionID: sessionID,
		}
		disconnectedJSON, _ := json.Marshal(disconnectedMsg)
		wsWrite(websocket.TextMessage, disconnectedJSON)
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
