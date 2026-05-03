package ssh

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"

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
			return true
		}
		
		// Robust same-host check (allows different ports on the same machine/IP)
		isSameHost := false
		if origin == "null" {
			isSameHost = true
		} else {
			if parts := strings.Split(origin, "://"); len(parts) > 1 {
				originHostWithPort := strings.Split(parts[1], "/")[0]
				originHost := strings.Split(originHostWithPort, ":")[0]
				requestHost := strings.Split(r.Host, ":")[0]
				if originHost == requestHost || originHost == "localhost" || originHost == "127.0.0.1" {
					isSameHost = true
				}
			}
		}

		if isSameHost {
			return true
		}
		for _, allowed := range cfg.AllowedOrigins {
			if allowed == "*" {
				return true
			}
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

		// Step 1: Read the initial message (text frame).
		_, msgData, err := conn.ReadMessage()
		if err != nil {
			return
		}

		var connectMsg ConnectMessage
		if err := json.Unmarshal(msgData, &connectMsg); err != nil {
			sendWSError(conn, "Invalid JSON message")
			return
		}

		var session *ManagedSession

		if connectMsg.Type == "attach" {
			log.Printf("[WS] Attempting to attach to session: %s", connectMsg.SessionID)
			var ok bool
			session, ok = GlobalSessionManager.GetSession(connectMsg.SessionID)
			if !ok {
				log.Printf("[WS] Session NOT FOUND for attach: %s", connectMsg.SessionID)
				sendWSError(conn, "Session not found")
				return
			}

			log.Printf("[WS] Session found, re-binding WebSocket for session: %s", session.ID)
			session.mu.Lock()
			if session.WS != nil {
				log.Printf("[WS] Closing old WebSocket for session: %s", session.ID)
				session.WS.Close()
			}
			session.WS = conn
			session.mu.Unlock()

			// 1. Notify client that re-attachment is ready FIRST.
			connectedMsg := ServerMessage{
				Type:      "connected",
				SessionID: session.ID,
			}
			connectedJSON, _ := json.Marshal(connectedMsg)
			log.Printf("[WS] Sending connected message for session attach: %s", session.ID)
			if err := session.WriteWS(websocket.TextMessage, connectedJSON); err != nil {
				log.Printf("[WS] Error sending connected message: %v", err)
				return
			}
		} else if connectMsg.Type == "connect" {
			// Resolve connection credentials: either from saved connection_id or direct fields.
			var keySigner ssh.Signer
			if connectMsg.ConnectionID != "" {
				var dbConn db.Connection
				if err := database.First(&dbConn, "id = ?", connectMsg.ConnectionID).Error; err != nil {
					sendWSError(conn, "Connection not found")
					return
				}

				if connectMsg.AuthMethod == "key" && dbConn.AuthMethod == "key" && dbConn.SSHKeyID != nil {
					var key db.SSHKey
					if err := database.First(&key, "id = ?", *dbConn.SSHKeyID).Error; err != nil {
						sendWSError(conn, "SSH key not found")
						return
					}

					decryptedKey, err := config.DecryptWithAAD(key.EncryptedKey, cfg.EncryptionKey, []byte(config.SSHKeyAAD))
					if err != nil {
						sendWSError(conn, "Failed to decrypt SSH key")
						return
					}

					var parsedKey interface{}
					if connectMsg.Passphrase != "" {
						parsedKey, err = ssh.ParseRawPrivateKeyWithPassphrase([]byte(decryptedKey), []byte(connectMsg.Passphrase))
					} else {
						parsedKey, err = ssh.ParseRawPrivateKey([]byte(decryptedKey))
					}

					if err != nil {
						sendWSError(conn, "Failed to parse SSH key: invalid passphrase or corrupted key")
						return
					}

					keySigner, err = ssh.NewSignerFromKey(parsedKey)
					if err != nil {
						sendWSError(conn, "Failed to create SSH signer from key")
						return
					}

					connectMsg.Host = dbConn.Host
					connectMsg.Port = dbConn.Port
					connectMsg.User = dbConn.Username
				} else if connectMsg.AuthMethod == "key" || dbConn.AuthMethod == "key" {
					sendWSError(conn, "Authentication method mismatch")
					return
				} else {
					decrypted, err := config.Decrypt(dbConn.Encrypted, cfg.EncryptionKey)
					if err != nil {
						sendWSError(conn, "Failed to decrypt connection credentials")
						return
					}
					connectMsg.Host = dbConn.Host
					connectMsg.Port = dbConn.Port
					connectMsg.User = dbConn.Username
					connectMsg.Password = decrypted
				}
			}

			// Security Fix CR-03: SSRF Protection
			if !cfg.IsHostAllowed(connectMsg.Host) {
				sendWSError(conn, "Host not authorized by security policy")
				return
			}

			if connectMsg.Port <= 0 || connectMsg.Port > 65535 {
				connectMsg.Port = 22
			}

			// Step 2: Dial SSH.
			addr := fmt.Sprintf("%s:%d", connectMsg.Host, connectMsg.Port)
			sshConfig := &ssh.ClientConfig{
				User:            connectMsg.User,
				HostKeyCallback: ssh.InsecureIgnoreHostKey(),
				Timeout:         10 * time.Second,
			}

			if keySigner != nil {
				sshConfig.Auth = []ssh.AuthMethod{ssh.PublicKeys(keySigner)}
			} else {
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
				sendWSError(conn, fmt.Sprintf("SSH connection failed: %v", err))
				return
			}

			sshSession, err := client.NewSession()
			if err != nil {
				client.Close()
				sendWSError(conn, fmt.Sprintf("SSH session failed: %v", err))
				return
			}

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
				termType = "screen-256color"
			}
			if err := sshSession.RequestPty(termType, connectMsg.Rows, connectMsg.Cols, modes); err != nil {
				sshSession.Close()
				client.Close()
				sendWSError(conn, "Failed to request PTY")
				return
			}

			stdinPipe, _ := sshSession.StdinPipe()
			stdoutPipe, _ := sshSession.StdoutPipe()
			stderrPipe, _ := sshSession.StderrPipe()

			if connectMsg.Cwd != "" && !strings.Contains(connectMsg.Cwd, "'") && !strings.Contains(connectMsg.Cwd, "\n") {
				if err := sshSession.Start(fmt.Sprintf("cd '%s' && exec -l $SHELL", connectMsg.Cwd)); err != nil {
					sshSession.Close()
					client.Close()
					return
				}
			} else {
				if err := sshSession.Shell(); err != nil {
					sshSession.Close()
					client.Close()
					return
				}
			}

			// Find shell PID
			var shellPid int
			{
				s, err := client.NewSession()
				if err == nil {
					out, err := s.Output("echo $$; ps -eo pid,ppid --no-headers")
					s.Close()
					if err == nil {
						lines := strings.Split(string(out), "\n")
						if len(lines) > 0 {
							execPid, _ := strconv.Atoi(strings.TrimSpace(lines[0]))
							type proc struct{ pid, ppid int }
							var procs []proc
							for _, line := range lines[1:] {
								fields := strings.Fields(line)
								if len(fields) >= 2 {
									p, _ := strconv.Atoi(fields[0])
									pp, _ := strconv.Atoi(fields[1])
									procs = append(procs, proc{p, pp})
								}
							}
							var execPpid int
							for _, pr := range procs {
								if pr.pid == execPid {
									execPpid = pr.ppid
									break
								}
							}
							for _, pr := range procs {
								if pr.ppid == execPpid && pr.pid != execPid {
									shellPid = pr.pid
								}
							}
						}
					}
				}
			}

			session = GlobalSessionManager.CreateSession(client, sshSession, stdinPipe, connectMsg.Host, connectMsg.User, connectMsg.Port, connectMsg.ConnectionID)
			session.mu.Lock()
			session.WS = conn
			session.ShellPid = shellPid
			session.mu.Unlock()

			go masterForwarder(session, stdoutPipe, stderrPipe)

			// Notify client that connection is ready.
			connectedMsg := ServerMessage{
				Type:      "connected",
				SessionID: session.ID,
			}
			connectedJSON, _ := json.Marshal(connectedMsg)
			_ = session.WriteWS(websocket.TextMessage, connectedJSON)
		} else {
			sendWSError(conn, "Invalid message type: expected connect or attach")
			return
		}

		wsMessageLoop(conn, session)
	}
}

func masterForwarder(session *ManagedSession, stdout, stderr io.Reader) {
	forward := func(src io.Reader) {
		buf := make([]byte, 4096)
		for {
			n, err := src.Read(buf)
			if n > 0 {
				data := make([]byte, n)
				copy(data, buf[:n])
				session.Buffer.Write(data)
				_ = session.WriteWS(websocket.BinaryMessage, data)
			}
			if err != nil {
				// SSH session ended
				GlobalSessionManager.RemoveSession(session.ID)
				return
			}
		}
	}
	go forward(stdout)
	go forward(stderr)
}

func wsMessageLoop(conn *websocket.Conn, session *ManagedSession) {
	defer func() {
		session.mu.Lock()
		if session.WS == conn {
			session.WS = nil
			session.LastSeen = time.Now()
		}
		session.mu.Unlock()
	}()

	for {
		msgType, msgData, err := conn.ReadMessage()
		if err != nil {
			return
		}

		if msgType == websocket.TextMessage {
			var ctrl struct {
				Type string `json:"type"`
				Cols int    `json:"cols"`
				Rows int    `json:"rows"`
			}
			if err := json.Unmarshal(msgData, &ctrl); err == nil {
				switch ctrl.Type {
				case "resize":
					if ctrl.Cols > 0 && ctrl.Rows > 0 {
						winSize := struct {
							Cols uint32
							Rows uint32
							W    uint32
							H    uint32
						}{uint32(ctrl.Cols), uint32(ctrl.Rows), 0, 0}
						_, _ = session.SSHSession.SendRequest("window-change", false, ssh.Marshal(winSize))
					}
				case "ready":
					// Send scrollback when client says it's ready (D-11-01 refactor)
					scrollback := session.Buffer.Bytes()
					if len(scrollback) > 0 {
						log.Printf("[WS:%s] Client ready, sending scrollback (%d bytes)", session.ID, len(scrollback))
						_ = session.WriteWS(websocket.BinaryMessage, scrollback)
					}
				case "get-cwd":
					go func(pid int, client *ssh.Client) {
						path := ""
						if pid > 0 {
							s, sessErr := client.NewSession()
							if sessErr == nil {
								defer s.Close()
								out, outErr := s.Output(fmt.Sprintf("readlink /proc/%d/cwd 2>/dev/null", pid))
								if outErr == nil {
									path = strings.TrimSpace(string(out))
								}
							}
						}
						if path == "" {
							path = "/"
						}
						cwdResp, _ := json.Marshal(CwdResponseMessage{Type: "cwd", Path: path})
						_ = session.WriteWS(websocket.TextMessage, cwdResp)
					}(session.ShellPid, session.SSHClient)
				case "disconnect":
					GlobalSessionManager.RemoveSession(session.ID)
					return
				}
			}
		} else if msgType == websocket.BinaryMessage {
			_, _ = session.Stdin.Write(msgData)
		}
	}
}

func sendWSError(conn *websocket.Conn, message string) {
	resp, _ := json.Marshal(map[string]string{
		"type":    "error",
		"message": message,
	})
	_ = conn.WriteMessage(websocket.TextMessage, resp)
}
