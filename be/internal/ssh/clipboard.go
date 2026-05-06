package ssh

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
)

type ClipboardManager struct {
	sessions      map[string]*ClipboardSession
	mu            sync.RWMutex
	localHelperPath string
	localProcess   *os.Process
	localClients   map[*websocket.Conn]bool
	localClientsMu sync.RWMutex
}

type ClipboardSession struct {
	SessionID    string
	SSHSession   *ssh.Session
	Stdin        io.WriteCloser
	Stdout       *bytes.Buffer
	Stderr       *bytes.Buffer
	Running      bool
	OS           string
	mu           sync.RWMutex
}

type ClipboardMessage struct {
	Type    string `json:"type"`
	Content string `json:"content,omitempty"`
}

var GlobalClipboardManager *ClipboardManager

func init() {
	GlobalClipboardManager = NewClipboardManager()
}

func NewClipboardManager() *ClipboardManager {
	return &ClipboardManager{
		sessions:       make(map[string]*ClipboardSession),
		localHelperPath: "be/cmd/clip-helper",
	}
}

func (cm *ClipboardManager) detectOS(sshClient *ssh.Client) (string, error) {
	session, err := sshClient.NewSession()
	if err != nil {
		return "", err
	}
	defer session.Close()

	output, err := session.Output("uname -s")
	if err != nil {
		return "Linux", nil
	}

	os := strings.TrimSpace(string(output))
	if strings.Contains(os, "Linux") {
		return "Linux", nil
	} else if strings.Contains(os, "MINGW") || strings.Contains(os, "MSYS") || strings.Contains(os, "CYGWIN") {
		return "Windows", nil
	}

	return "Linux", nil
}

func (cm *ClipboardManager) StartClipboardListener(sessionID string, sshClient *ssh.Client, managedSession *ManagedSession) error {
	cm.mu.Lock()
	defer cm.mu.Unlock()

	if _, exists := cm.sessions[sessionID]; exists {
		return nil
	}

	detectedOS, err := cm.detectOS(sshClient)
	if err != nil {
		return fmt.Errorf("failed to detect OS: %w", err)
	}

	var helperName string
	if detectedOS == "Windows" {
		helperName = "web-term-clip.exe"
	} else {
		helperName = "web-term-clip"
	}

	tmpDir := "/tmp"
	if detectedOS == "Windows" {
		tmpDir = os.Getenv("TEMP")
		if tmpDir == "" {
			tmpDir = "C:\\Temp"
		}
	}

	helperPath := filepath.Join(tmpDir, helperName)

	checkSession, err := sshClient.NewSession()
	if err != nil {
		return err
	}
	defer checkSession.Close()

	exists, _ := checkSession.Output(fmt.Sprintf("test -f %s && echo exists", helperPath))
	if !strings.Contains(string(exists), "exists") {
		localBinary := cm.localHelperPath
		if detectedOS == "Windows" {
			localBinary += ".exe"
		}

		if _, err := os.Stat(localBinary); err != nil {
			fmt.Printf("Note: Clipboard helper binary not found at %s. Run 'go build -o %s %s' to create it.\n", localBinary, localBinary, cm.localHelperPath)
		}
	}

	session, err := sshClient.NewSession()
	if err != nil {
		return fmt.Errorf("failed to create clipboard session: %w", err)
	}

	stdin, err := session.StdinPipe()
	if err != nil {
		session.Close()
		return fmt.Errorf("failed to get stdin pipe: %w", err)
	}

	var stdout, stderr bytes.Buffer
	session.Stdout = &stdout
	session.Stderr = &stderr

	err = session.Start(helperPath)
	if err != nil {
		session.Close()
		return fmt.Errorf("failed to start clipboard helper: %w", err)
	}

	clipSession := &ClipboardSession{
		SessionID:  sessionID,
		SSHSession: session,
		Stdin:      stdin,
		Stdout:     &stdout,
		Stderr:     &stderr,
		Running:    true,
		OS:         detectedOS,
	}

	cm.sessions[sessionID] = clipSession

	go cm.readClipboardOutput(sessionID, managedSession)

	return nil
}

func (cm *ClipboardManager) readClipboardOutput(sessionID string, managedSession *ManagedSession) {
	cm.mu.RLock()
	clipSession, exists := cm.sessions[sessionID]
	cm.mu.RUnlock()

	if !exists {
		return
	}

	ticker := time.NewTicker(100 * time.Millisecond)
	defer ticker.Stop()

	for range ticker.C {
		clipSession.mu.RLock()
		if !clipSession.Running {
			clipSession.mu.RUnlock()
			break
		}

		output := clipSession.Stdout.String()
		clipSession.Stdout.Reset()
		clipSession.mu.RUnlock()

		lines := strings.Split(output, "\n")
		for _, line := range lines {
			if strings.HasPrefix(line, "CLIP:") {
				encoded := strings.TrimPrefix(line, "CLIP:")
				decoded, err := base64.StdEncoding.DecodeString(encoded)
				if err != nil {
					continue
				}

				msg := ClipboardMessage{
					Type:    "clipboard_update",
					Content: string(decoded),
				}
				jsonMsg, _ := json.Marshal(msg)

				if managedSession != nil && managedSession.WS != nil {
					managedSession.WriteWS(websocket.TextMessage, jsonMsg)
				}
			}
		}
	}
}

func (cm *ClipboardManager) StopClipboardListener(sessionID string) error {
	cm.mu.Lock()
	defer cm.mu.Unlock()

	clipSession, exists := cm.sessions[sessionID]
	if !exists {
		return nil
	}

	clipSession.mu.Lock()
	clipSession.Running = false
	clipSession.mu.Unlock()

	if clipSession.SSHSession != nil {
		clipSession.SSHSession.Close()
	}

	delete(cm.sessions, sessionID)
	return nil
}

func (cm *ClipboardManager) GetHistory(sessionID string) []string {
	return []string{}
}

func (cm *ClipboardManager) AddToHistory(sessionID string, content string) {

}

func (cm *ClipboardManager) WriteToClipboard(sessionID string, content string) error {
	cm.mu.RLock()
	clipSession, exists := cm.sessions[sessionID]
	cm.mu.RUnlock()

	if !exists {
		return fmt.Errorf("no clipboard session for session %s", sessionID)
	}

	clipSession.mu.Lock()
	defer clipSession.mu.Unlock()

	if clipSession.Stdin == nil {
		return fmt.Errorf("clipboard stdin not available")
	}

	encoded := base64.StdEncoding.EncodeToString([]byte(content))
	_, err := clipSession.Stdin.Write([]byte(fmt.Sprintf("SET:%s\n", encoded)))
	return err
}

func (cm *ClipboardManager) StartLocalClipboard() error {
	cm.mu.Lock()
	defer cm.mu.Unlock()

	if cm.localProcess != nil {
		return nil
	}

	cmd := exec.Command("be/cmd/clip-local")
	cmd.Stdout = pipe{cm, true}
	cmd.Stderr = os.Stderr

	if err := cmd.Start(); err != nil {
		return fmt.Errorf("failed to start local clipboard helper: %w", err)
	}

	cm.localProcess = cmd.Process
	cm.localClients = make(map[*websocket.Conn]bool)

	go func() {
		cmd.Wait()
		cm.mu.Lock()
		cm.localProcess = nil
		cm.mu.Unlock()
	}()

	return nil
}

func (cm *ClipboardManager) RegisterLocalClient(conn *websocket.Conn) {
	cm.localClientsMu.Lock()
	defer cm.localClientsMu.Unlock()
	cm.localClients[conn] = true
}

func (cm *ClipboardManager) UnregisterLocalClient(conn *websocket.Conn) {
	cm.localClientsMu.Lock()
	defer cm.localClientsMu.Unlock()
	delete(cm.localClients, conn)
}

func (cm *ClipboardManager) BroadcastLocalClipboard(content string) {
	msg := ClipboardMessage{
		Type:    "clipboard_update",
		Content: content,
	}
	data, _ := json.Marshal(msg)

	cm.localClientsMu.RLock()
	defer cm.localClientsMu.RUnlock()

	for conn := range cm.localClients {
		if conn != nil {
			conn.WriteMessage(websocket.TextMessage, data)
		}
	}
}

type pipe struct {
	cm     *ClipboardManager
	isStdout bool
}

func (p pipe) Write(b []byte) (int, error) {
	line := string(b)
	if strings.HasPrefix(line, "CLIP:") {
		encoded := strings.TrimPrefix(line, "CLIP:")
		encoded = strings.TrimSuffix(encoded, "\n")
		decoded, err := base64.StdEncoding.DecodeString(encoded)
		if err == nil {
			p.cm.BroadcastLocalClipboard(string(decoded))
		}
	}
	return len(b), nil
}