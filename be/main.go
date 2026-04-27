package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  4096,
	WriteBufferSize: 4096,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

// ConnectMessage is the initial JSON message from client with SSH connection params.
type ConnectMessage struct {
	Type     string `json:"type"`
	Host     string `json:"host"`
	Port     int    `json:"port"`
	User     string `json:"user"`
	Password string `json:"password"`
	Rows     int    `json:"rows,omitempty"`
	Cols     int    `json:"cols,omitempty"`
}

// ResizeMessage is a JSON control message for terminal resize.
type ResizeMessage struct {
	Type string `json:"type"`
	Cols int    `json:"cols"`
	Rows int    `json:"rows"`
}

func handleWS(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Printf("WebSocket upgrade error: %v", err)
		return
	}
	defer conn.Close()

	log.Printf("WebSocket connection established from %s", conn.RemoteAddr())

	// Step 1: Read the initial connect message (text frame).
	msgType, msgData, err := conn.ReadMessage()
	if err != nil {
		log.Printf("Error reading connect message: %v", err)
		return
	}

	if msgType != websocket.TextMessage {
		log.Printf("Expected text message for connect, got binary")
		conn.WriteMessage(websocket.TextMessage, []byte(`{"type":"error","message":"first message must be JSON connect"}`))
		return
	}

	var connectMsg ConnectMessage
	if err := json.Unmarshal(msgData, &connectMsg); err != nil {
		log.Printf("Error parsing connect message: %v", err)
		conn.WriteMessage(websocket.TextMessage, []byte(`{"type":"error","message":"invalid JSON"}`))
		return
	}

	if connectMsg.Type != "connect" {
		log.Printf("Expected connect message, got type: %s", connectMsg.Type)
		conn.WriteMessage(websocket.TextMessage, []byte(`{"type":"error","message":"expected connect message"}`))
		return
	}

	if connectMsg.Host == "" || connectMsg.User == "" {
		conn.WriteMessage(websocket.TextMessage, []byte(`{"type":"error","message":"host and user are required"}`))
		return
	}

	if connectMsg.Port == 0 {
		connectMsg.Port = 22
	}
	if connectMsg.Rows == 0 {
		connectMsg.Rows = 24
	}
	if connectMsg.Cols == 0 {
		connectMsg.Cols = 80
	}

	// Step 2: Dial SSH.
	addr := fmt.Sprintf("%s:%d", connectMsg.Host, connectMsg.Port)
	sshConfig := &ssh.ClientConfig{
		User: connectMsg.User,
		Auth: []ssh.AuthMethod{
			ssh.Password(connectMsg.Password),
		},
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	log.Printf("Connecting to SSH at %s as %s", addr, connectMsg.User)
	client, err := ssh.Dial("tcp", addr, sshConfig)
	if err != nil {
		log.Printf("SSH dial error: %v", err)
		errMsg := fmt.Sprintf(`{"type":"error","message":"SSH connection failed: %s"}`, jsonEscape(err.Error()))
		conn.WriteMessage(websocket.TextMessage, []byte(errMsg))
		return
	}
	defer client.Close()

	session, err := client.NewSession()
	if err != nil {
		log.Printf("SSH session error: %v", err)
		errMsg := fmt.Sprintf(`{"type":"error","message":"SSH session failed: %s"}`, jsonEscape(err.Error()))
		conn.WriteMessage(websocket.TextMessage, []byte(errMsg))
		return
	}
	defer session.Close()

	// Step 3: Request PTY.
	modes := ssh.TerminalModes{
		ssh.ECHO:          1,
		ssh.TTY_OP_ISPEED: 14400,
		ssh.TTY_OP_OSPEED: 14400,
	}
	if err := session.RequestPty("xterm-256color", connectMsg.Rows, connectMsg.Cols, modes); err != nil {
		log.Printf("PTY request error: %v", err)
		return
	}

	// Step 4: Get SSH pipes.
	stdinPipe, err := session.StdinPipe()
	if err != nil {
		log.Printf("SSH stdin pipe error: %v", err)
		return
	}
	stdoutPipe, err := session.StdoutPipe()
	if err != nil {
		log.Printf("SSH stdout pipe error: %v", err)
		return
	}
	stderrPipe, err := session.StderrPipe()
	if err != nil {
		log.Printf("SSH stderr pipe error: %v", err)
		return
	}

	// Step 5: Start shell.
	if err := session.Shell(); err != nil {
		log.Printf("Shell start error: %v", err)
		return
	}

	log.Printf("SSH session established to %s", addr)

	// Notify client that connection is ready.
	conn.WriteMessage(websocket.TextMessage, []byte(`{"type":"connected"}`))

	// Step 6: Bidirectional pipe with context for cleanup.
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	done := make(chan struct{}, 3)

	// Goroutine 1: WebSocket → SSH stdin.
	go func() {
		defer func() { done <- struct{}{} }()
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			msgType, msgData, err := conn.ReadMessage()
			if err != nil {
				if !websocket.IsCloseError(err, websocket.CloseNormalClosure, websocket.CloseGoingAway) {
					log.Printf("WebSocket read error: %v", err)
				}
				cancel()
				return
			}

			switch msgType {
			case websocket.TextMessage:
				// Handle control messages (resize).
				var ctrl ResizeMessage
				if err := json.Unmarshal(msgData, &ctrl); err == nil && ctrl.Type == "resize" {
					if ctrl.Cols > 0 && ctrl.Rows > 0 {
						winSize := struct{ Cols, Rows uint32 }{uint32(ctrl.Cols), uint32(ctrl.Rows)}
						payload := ssh.Marshal(winSize)
						session.SendRequest("window-change", false, payload)
						log.Printf("Resize: %dx%d", ctrl.Cols, ctrl.Rows)
					}
				}
			case websocket.BinaryMessage:
				// SSH data — forward to stdin.
				if _, err := stdinPipe.Write(msgData); err != nil {
					log.Printf("SSH stdin write error: %v", err)
					cancel()
					return
				}
			}
		}
	}()

	// Goroutine 2: SSH stdout → WebSocket.
	go func() {
		defer func() { done <- struct{}{} }()
		buf := make([]byte, 4096)
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			n, err := stdoutPipe.Read(buf)
			if err != nil {
				if err != io.EOF {
					log.Printf("SSH stdout read error: %v", err)
				}
				cancel()
				return
			}
			if n > 0 {
				if err := conn.WriteMessage(websocket.BinaryMessage, buf[:n]); err != nil {
					log.Printf("WebSocket write error: %v", err)
					cancel()
					return
				}
			}
		}
	}()

	// Goroutine 3: SSH stderr → WebSocket.
	go func() {
		defer func() { done <- struct{}{} }()
		buf := make([]byte, 4096)
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			n, err := stderrPipe.Read(buf)
			if err != nil {
				if err != io.EOF {
					log.Printf("SSH stderr read error: %v", err)
				}
				cancel()
				return
			}
			if n > 0 {
				if err := conn.WriteMessage(websocket.BinaryMessage, buf[:n]); err != nil {
					log.Printf("WebSocket write error (stderr): %v", err)
					cancel()
					return
				}
			}
		}
	}()

	// Wait for any goroutine to finish, then cleanup.
	<-done
	cancel()
	log.Printf("SSH session to %s closed", addr)
}

func jsonEscape(s string) string {
	s = strings.ReplaceAll(s, `\`, `\\`)
	s = strings.ReplaceAll(s, `"`, `\"`)
	s = strings.ReplaceAll(s, "\n", `\n`)
	return s
}

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/ws", handleWS)

	server := &http.Server{
		Addr:    "127.0.0.1:8080",
		Handler: mux,
	}

	// Graceful shutdown on SIGINT.
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-sigChan
		log.Println("Shutting down server...")
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		server.Shutdown(ctx)
	}()

	log.Printf("WebTerm SSH proxy starting on 127.0.0.1:8080")
	if err := server.ListenAndServe(); err != http.ErrServerClosed {
		log.Fatalf("Server error: %v", err)
	}
	log.Println("Server stopped")
}
