package ssh

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"

	"github.com/creack/pty"
	"github.com/gorilla/websocket"
	"github.com/pkg/sftp"
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
			if connectMsg.ConnectionID == "local" || connectMsg.SessionType == SessionTypeLocal {
				ptyFile, pid, err := spawnLocalPTY(connectMsg)
				if err != nil {
					sendWSError(conn, fmt.Sprintf("Failed to spawn local PTY: %v", err))
					return
				}

				session = GlobalSessionManager.CreateSession(SessionTypeLocal, nil, nil, ptyFile, ptyFile, "local", "local", 0, "local")
				session.mu.Lock()
				session.WS = conn
				session.ShellPid = pid
				session.mu.Unlock()

				go masterForwarder(session, ptyFile)

				connectedMsg := ServerMessage{
					Type:      "connected",
					SessionID: session.ID,
				}
				connectedJSON, _ := json.Marshal(connectedMsg)
				_ = session.WriteWS(websocket.TextMessage, connectedJSON)
			} else {
				// Resolve connection credentials: either from saved connection_id or direct fields.
				var keySigner ssh.Signer
				if connectMsg.ConnectionID != "" {
					var dbConn db.Connection
					if err := database.First(&dbConn, "id = ?", connectMsg.ConnectionID).Error; err != nil {
						sendWSError(conn, "Connection not found")
						return
					}

					// Use auth_method from database when connection_id is provided
					authMethod := dbConn.AuthMethod
					if authMethod == "" {
						authMethod = "password"
					}

					if authMethod == "key" && dbConn.SSHKeyID != nil {
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
					} else if authMethod == "key" {
						sendWSError(conn, "Connection has auth_method=key but no ssh_key_id configured")
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

				session = GlobalSessionManager.CreateSession(SessionTypeSSH, client, sshSession, nil, stdinPipe, connectMsg.Host, connectMsg.User, connectMsg.Port, connectMsg.ConnectionID)
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

				// Register for local clipboard (system-wide clipboard on backend machine)
				GlobalClipboardManager.RegisterLocalClient(conn)
			}
		} else {

			sendWSError(conn, "Invalid message type: expected connect or attach")
			return
		}

		wsMessageLoop(conn, session)
	}
}

func spawnLocalPTY(connectMsg ConnectMessage) (*os.File, int, error) {
	shell := os.Getenv("SHELL")
	if shell == "" {
		shell = "/bin/bash"
	}
	c := exec.Command(shell)
	f, err := pty.Start(c)
	if err != nil {
		return nil, 0, err
	}

	if connectMsg.Rows > 0 && connectMsg.Cols > 0 {
		_ = pty.Setsize(f, &pty.Winsize{Rows: uint16(connectMsg.Rows), Cols: uint16(connectMsg.Cols)})
	}

	return f, c.Process.Pid, nil
}

func masterForwarder(session *ManagedSession, readers ...io.Reader) {
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
				// session ended
				GlobalSessionManager.RemoveSession(session.ID)
				return
			}
		}
	}
	for _, r := range readers {
		if r != nil {
			go forward(r)
		}
	}
}

func wsMessageLoop(conn *websocket.Conn, session *ManagedSession) {
	defer func() {
		session.mu.Lock()
		if session.WS == conn {
			session.WS = nil
			session.LastSeen = time.Now()
		}
		session.mu.Unlock()

		// Unregister from local clipboard
		GlobalClipboardManager.UnregisterLocalClient(conn)
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
						if session.Type == SessionTypeSSH && session.SSHSession != nil {
							winSize := struct {
								Cols uint32
								Rows uint32
								W    uint32
								H    uint32
							}{uint32(ctrl.Cols), uint32(ctrl.Rows), 0, 0}
							_, _ = session.SSHSession.SendRequest("window-change", false, ssh.Marshal(winSize))
						} else if session.Type == SessionTypeLocal && session.LocalPTY != nil {
							_ = pty.Setsize(session.LocalPTY, &pty.Winsize{Rows: uint16(ctrl.Rows), Cols: uint16(ctrl.Cols)})
						}
					}
				case "ready":
					// Send scrollback when client says it's ready (D-11-01 refactor)
					scrollback := session.Buffer.Bytes()
					if len(scrollback) > 0 {
						log.Printf("[WS:%s] Client ready, sending scrollback (%d bytes)", session.ID, len(scrollback))
						_ = session.WriteWS(websocket.BinaryMessage, scrollback)
					}
				case "get-cwd":
					go func(s *ManagedSession) {
						path := s.GetCwd()
						if path == "" {
							path = "/"
						}
						cwdResp, _ := json.Marshal(CwdResponseMessage{Type: "cwd", Path: path})
						_ = s.WriteWS(websocket.TextMessage, cwdResp)
					}(session)
				case "disconnect":
					GlobalSessionManager.RemoveSession(session.ID)
					return
				case "clipboard_write":
					var clipboardMsg struct {
						Content string `json:"content"`
					}
					if err := json.Unmarshal(msgData, &clipboardMsg); err == nil && clipboardMsg.Content != "" {
						if session.Type == SessionTypeSSH {
							go func() {
								if err := GlobalClipboardManager.WriteToClipboard(session.ID, clipboardMsg.Content); err != nil {
									log.Printf("[WS:%s] clipboard write error: %v", session.ID, err)
								}
							}()
						}
					}
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

type ClipboardConnectMessage struct {
	Host         string `json:"host"`
	Port         int    `json:"port"`
	User         string `json:"user"`
	Password     string `json:"password"`
	AuthMethod   string `json:"auth_method,omitempty"`
	SSHKeyID     string `json:"ssh_key_id,omitempty"`
	Passphrase   string `json:"passphrase,omitempty"`
	ConnectionID string `json:"connection_id,omitempty"`
}

func HandleClipboardWebSocket(database *gorm.DB, cfg *config.Config) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		conn, err := upgrader.Upgrade(w, r, nil)
		if err != nil {
			log.Printf("[ClipboardWS] WebSocket upgrade error: %v", err)
			return
		}
		defer conn.Close()

		_, msgData, err := conn.ReadMessage()
		if err != nil {
			log.Printf("[ClipboardWS] Failed to read initial message: %v", err)
			return
		}

		var connectMsg ClipboardConnectMessage
		if err := json.Unmarshal(msgData, &connectMsg); err != nil {
			sendWSError(conn, "Invalid JSON message")
			return
		}

		// Resolve connection credentials
		var host, user, password string
		var port int
		var keySigner ssh.Signer

		if connectMsg.ConnectionID != "" {
			var dbConn db.Connection
			if err := database.First(&dbConn, "id = ?", connectMsg.ConnectionID).Error; err != nil {
				sendWSError(conn, "Connection not found")
				return
			}

			authMethod := dbConn.AuthMethod
			if authMethod == "" {
				authMethod = "password"
			}

			host = dbConn.Host
			port = dbConn.Port
			user = dbConn.Username

			if authMethod == "key" && dbConn.SSHKeyID != nil {
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
			} else if authMethod == "key" {
				sendWSError(conn, "Connection has auth_method=key but no ssh_key_id configured")
				return
			} else {
				decrypted, err := config.Decrypt(dbConn.Encrypted, cfg.EncryptionKey)
				if err != nil {
					sendWSError(conn, "Failed to decrypt connection credentials")
					return
				}
				password = decrypted
			}
		} else {
			host = connectMsg.Host
			port = connectMsg.Port
			user = connectMsg.User
			password = connectMsg.Password
		}

		// SSH client config
		config := &ssh.ClientConfig{
			User: user,
			Auth: []ssh.AuthMethod{},
		}

		if keySigner != nil {
			config.Auth = append(config.Auth, ssh.PublicKeys(keySigner))
		} else if password != "" {
			config.Auth = append(config.Auth, ssh.Password(password))
		}

		config.HostKeyCallback = ssh.InsecureIgnoreHostKey()

		// Connect to SSH
		client, err := ssh.Dial("tcp", fmt.Sprintf("%s:%d", host, port), config)
		if err != nil {
			sendWSError(conn, fmt.Sprintf("SSH connection failed: %v", err))
			return
		}
		defer client.Close()

		// Create a session specifically for clipboard monitoring (no PTY)
		session, err := client.NewSession()
		if err != nil {
			sendWSError(conn, "Failed to create SSH session for clipboard")
			return
		}
		defer session.Close()

		// Detect remote OS
		var remoteOS string
		if stdout, err := session.Output("uname -s"); err == nil {
			os := strings.TrimSpace(string(stdout))
			if strings.Contains(os, "Linux") {
				remoteOS = "Linux"
			} else if strings.Contains(os, "MINGW") || strings.Contains(os, "MSYS") || strings.Contains(os, "CYGWIN") || strings.Contains(os, "Windows") {
				remoteOS = "Windows"
			} else {
				remoteOS = os
			}
		} else {
			if _, err := session.Output("powershell -Command Get-Date"); err == nil {
				remoteOS = "Windows"
			} else {
				remoteOS = "Unknown"
			}
		}

		// Upload and run clip-helper binary on remote
		binName := "web-term-clip.exe"
		vbsStart := "run-clipper.vbs"
		vbsStop := "stop-clipper.vbs"

		// Upload via SFTP
		log.Printf("[ClipboardWS] Uploading clip-helper files via SFTP...")

		sftpClient, err := sftp.NewClient(client)
		if err != nil {
			log.Printf("[ClipboardWS] SFTP creation failed: %v", err)
		} else {
			defer sftpClient.Close()

			// Get remote home path via new session
			var remoteHome string
			homeSession, homeErr := client.NewSession()
			if homeErr == nil {
				homeOut, _ := homeSession.Output("powershell -NoProfile -Command \"[Environment]::GetFolderPath('UserProfile')\"")
				remoteHome = strings.TrimSpace(string(homeOut))
				homeSession.Close()
			}
			if remoteHome == "" {
				sendWSError(conn, "Failed to determine user home directory")
				return
			}

			// Use backslash for Windows
			remotePath := remoteHome + "\\testing-clip"
			log.Printf("[ClipboardWS] Remote path: %s", remotePath)

			// Create directory
			if err := sftpClient.Mkdir(remotePath); err != nil {
				log.Printf("[ClipboardWS] Mkdir: %v (may already exist)", err)
			}

			// Upload binary
			localBin := filepath.Join(getExeDir(), "cmd", "clip-helper", binName)
			data, err := os.ReadFile(localBin)
			if err != nil {
				log.Printf("[ClipboardWS] Read binary: %v", err)
			} else {
				dst, err := sftpClient.Create(remotePath + "/" + binName)
				if err != nil {
					log.Printf("[ClipboardWS] Create binary: %v", err)
				} else {
					if _, err := dst.Write(data); err != nil {
						log.Printf("[ClipboardWS] Write binary: %v", err)
					} else {
						log.Printf("[ClipboardWS] Binary uploaded")
					}
					dst.Close()
				}
			}

			// Upload VBS start
			localVBS := filepath.Join(getExeDir(), "cmd", "clip-helper", vbsStart)
			vbsData, err := os.ReadFile(localVBS)
			if err != nil {
				log.Printf("[ClipboardWS] Read VBS: %v", err)
			} else {
				dst, err := sftpClient.Create(remotePath + "/" + vbsStart)
				if err != nil {
					log.Printf("[ClipboardWS] Create VBS: %v", err)
				} else {
					if _, err := dst.Write(vbsData); err != nil {
						log.Printf("[ClipboardWS] Write VBS: %v", err)
					} else {
						log.Printf("[ClipboardWS] VBS uploaded")
					}
					dst.Close()
				}
			}

			// Upload VBS stop
			localVBSStop := filepath.Join(getExeDir(), "cmd", "clip-helper", vbsStop)
			vbsStopData, err := os.ReadFile(localVBSStop)
			if err != nil {
				log.Printf("[ClipboardWS] Read VBS stop: %v", err)
			} else {
				dst, err := sftpClient.Create(remotePath + "/" + vbsStop)
				if err != nil {
					log.Printf("[ClipboardWS] Create VBS stop: %v", err)
				} else {
					if _, err := dst.Write(vbsStopData); err != nil {
						log.Printf("[ClipboardWS] Write VBS stop: %v", err)
					} else {
						log.Printf("[ClipboardWS] VBS stop uploaded")
					}
					dst.Close()
				}
			}

			log.Printf("[ClipboardWS] All files uploaded via SFTP")

			// Run clipboard monitor via PowerShell Start-Process (more reliable than VBS)
			log.Printf("[ClipboardWS] Starting clip-helper via PowerShell...")
			runSession, _ := client.NewSession()
			if runSession != nil {
				exePath := remotePath + "\\web-term-clip.exe"
				outputPath := remoteHome + "\\clipboard-output.log"
				cmd := fmt.Sprintf("Start-Process -FilePath '%s' -WindowStyle Hidden -RedirectStandardOutput '%s' -RedirectStandardError '%s'", exePath, outputPath, strings.Replace(outputPath, ".log", "-err.log", 1))
				log.Printf("[ClipboardWS] Running: %s", cmd)
				out, err := runSession.Output("powershell -NoProfile -Command \"" + cmd + "\"")
				log.Printf("[ClipboardWS] Start-Process result: err=%v, out=%s", err, string(out))
				runSession.Close()
			}
			
			// Wait a bit and check if binary is running
			time.Sleep(3 * time.Second)
			checkSession, _ := client.NewSession()
			if checkSession != nil {
				taskOut, _ := checkSession.Output("powershell -NoProfile -Command \"Get-Process web-term-clip -ErrorAction SilentlyContinue | Select-Object Name,Id\"")
				log.Printf("[ClipboardWS] Process check: %s", string(taskOut))
				checkSession.Close()
			}

			// Start polling output file
			go func() {
				lastContent := ""
				ticker := time.NewTicker(1 * time.Second)
				defer ticker.Stop()

				for range ticker.C {
					var out bytes.Buffer
					readSession, err := client.NewSession()
					if err != nil {
						continue
					}

					if remoteOS == "Windows" {
						outputPath := remoteHome + "\\clipboard-output.log"
						readSession.Stdout = &out
						readSession.Run("powershell -NoProfile -Command \"Get-Content '" + outputPath + "'\"")
					}
					readSession.Close()

					content := strings.TrimSpace(out.String())
					if content != "" && content != lastContent {
						lastContent = content

						// Parse CLIP: lines
						lines := strings.Split(content, "\n")
						for _, line := range lines {
							line = strings.TrimSpace(line)
							if strings.HasPrefix(line, "CLIP:") {
								clipData := strings.TrimPrefix(line, "CLIP:")
								if decoded, err := base64.StdEncoding.DecodeString(clipData); err == nil {
									clipMsg := ClipboardMessage{
										Type:    "text",
										Content: string(decoded),
									}
									if err := conn.WriteJSON(clipMsg); err != nil {
										log.Printf("[ClipboardWS] Send error: %v", err)
										return
									}
								}
							}
						}
					}
				}
			}()
		}

		// Keep connection alive and handle incoming messages (e.g., write to remote clipboard)
		// Cleanup on disconnect
		defer func() {
			log.Printf("[ClipboardWS] Connection closing, stopping clipper...")
			if remoteOS == "Windows" {
				_, _ = session.Output("cscript //B stop-clipper.vbs")
			}
		}()

		for {
			_, msg, err := conn.ReadMessage()
			if err != nil {
				break
			}

			// Handle incoming messages - e.g., client wants to set remote clipboard
			var clientMsg map[string]interface{}
			if err := json.Unmarshal(msg, &clientMsg); err != nil {
				continue
			}

			if clientMsg["type"] == "set_clipboard" {
				content, ok := clientMsg["content"].(string)
				if ok && content != "" {
					// Create a new session for writing clipboard
					writeSession, err := client.NewSession()
					if err != nil {
						log.Printf("[ClipboardWS] Set clipboard session error: %v", err)
						continue
					}

					var stdout, stderr bytes.Buffer
					writeSession.Stdout = &stdout
					writeSession.Stderr = &stderr

					if remoteOS == "Windows" {
						escaped := strings.ReplaceAll(content, "\"", "`\"")
						_ = writeSession.Run(fmt.Sprintf("powershell -Command Set-Clipboard -Value \"%s\"", escaped))
					} else {
						escaped := strings.ReplaceAll(content, "'", "'\\''")
						_ = writeSession.Run(fmt.Sprintf("echo '%s' | xclip -i -selection clipboard 2>/dev/null || echo '%s' | pbcopy 2>/dev/null", escaped, escaped))
					}
					writeSession.Close()
				}
			} else if clientMsg["type"] == "get_clipboard" {
				// Get clipboard from remote and send it back
				readSession, err := client.NewSession()
				if err != nil {
					continue
				}

				var stdout, stderr bytes.Buffer
				readSession.Stdout = &stdout
				readSession.Stderr = &stderr

				var cmd string
				if remoteOS == "Windows" {
					cmd = "powershell -NoProfile -Command Get-Clipboard"
				} else {
					cmd = "xclip -o -selection clipboard 2>/dev/null || pbpaste 2>/dev/null"
				}

				_ = readSession.Run(cmd)
				readSession.Close()

				clipContent := strings.TrimSpace(stdout.String())
				if clipContent != "" {
					msg := ClipboardMessage{
						Type:    "clipboard_update",
						Content: clipContent,
					}
					data, _ := json.Marshal(msg)
					_ = conn.WriteMessage(websocket.TextMessage, data)
				}
			}
		}
	}
}
