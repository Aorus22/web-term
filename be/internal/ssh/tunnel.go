package ssh

import (
	"context"
	"fmt"
	"io"
	"log"
	"net"
	"sync"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"

	"golang.org/x/crypto/ssh"
	"gorm.io/gorm"
)

// ActiveTunnel holds the runtime state of an active port forward.
type ActiveTunnel struct {
	Listener  net.Listener
	SSHClient *ssh.Client
	Cancel    context.CancelFunc
	Error     string
}

// TunnelManager manages active port forwarding tunnels in-memory.
type TunnelManager struct {
	mu      sync.Mutex
	tunnels map[string]*ActiveTunnel // keyed by forward ID
	db      *gorm.DB
	cfg     *config.Config
}

// NewTunnelManager creates a new TunnelManager.
func NewTunnelManager(database *gorm.DB, cfg *config.Config) *TunnelManager {
	return &TunnelManager{
		tunnels: make(map[string]*ActiveTunnel),
		db:      database,
		cfg:     cfg,
	}
}

// Start establishes an SSH tunnel for the given forward rule.
// It binds a local listener and dials the remote server via SSH.
func (tm *TunnelManager) Start(forwardID string, conn db.Connection, localPort int, remotePort int) error {
	tm.mu.Lock()
	defer tm.mu.Unlock()

	// Check if already active
	if _, exists := tm.tunnels[forwardID]; exists {
		return fmt.Errorf("forward %s is already active", forwardID)
	}

	// Bind local port — this catches port conflicts (D-10)
	listener, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", localPort))
	if err != nil {
		return fmt.Errorf("Port %d is already in use", localPort)
	}

	// Build SSH client config using same auth flow as proxy.go (D-04)
	addr := fmt.Sprintf("%s:%d", conn.Host, conn.Port)
	sshConfig := &ssh.ClientConfig{
		User:            conn.Username,
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	if conn.AuthMethod == "key" && conn.SSHKeyID != nil {
		// KEY AUTH PATH
		var key db.SSHKey
		if err := tm.db.First(&key, "id = ?", *conn.SSHKeyID).Error; err != nil {
			listener.Close()
			return fmt.Errorf("SSH key not found: %v", err)
		}

		decryptedKey, err := config.DecryptWithAAD(key.EncryptedKey, tm.cfg.EncryptionKey, []byte(config.SSHKeyAAD))
		if err != nil {
			listener.Close()
			return fmt.Errorf("failed to decrypt SSH key: %v", err)
		}

		parsedKey, err := ssh.ParseRawPrivateKey([]byte(decryptedKey))
		if err != nil {
			listener.Close()
			return fmt.Errorf("failed to parse SSH key: %v", err)
		}

		keySigner, err := ssh.NewSignerFromKey(parsedKey)
		if err != nil {
			listener.Close()
			return fmt.Errorf("failed to create SSH signer: %v", err)
		}

		sshConfig.Auth = []ssh.AuthMethod{ssh.PublicKeys(keySigner)}
	} else {
		// PASSWORD AUTH PATH
		decrypted, err := config.Decrypt(conn.Encrypted, tm.cfg.EncryptionKey)
		if err != nil {
			listener.Close()
			return fmt.Errorf("failed to decrypt connection credentials: %v", err)
		}

		sshConfig.Auth = []ssh.AuthMethod{
			ssh.Password(decrypted),
			ssh.KeyboardInteractive(func(user, instruction string, questions []string, echos []bool) ([]string, error) {
				answers := make([]string, len(questions))
				for i := range answers {
					answers[i] = decrypted
				}
				return answers, nil
			}),
		}
	}

	// Dial SSH
	client, err := ssh.Dial("tcp", addr, sshConfig)
	if err != nil {
		listener.Close()
		return fmt.Errorf("SSH connection failed: %v", err)
	}

	// Setup context for clean shutdown
	ctx, cancel := context.WithCancel(context.Background())

	tunnel := &ActiveTunnel{
		Listener:  listener,
		SSHClient: client,
		Cancel:    cancel,
	}

	tm.tunnels[forwardID] = tunnel

	// Accept loop in background goroutine
	go tm.acceptLoop(ctx, tunnel, forwardID, remotePort)

	return nil
}

// acceptLoop accepts incoming connections on the listener and forwards them
// through the SSH tunnel.
func (tm *TunnelManager) acceptLoop(ctx context.Context, tunnel *ActiveTunnel, forwardID string, remotePort int) {
	remoteAddr := fmt.Sprintf("localhost:%d", remotePort)

	for {
		select {
		case <-ctx.Done():
			return
		default:
		}

		localConn, err := tunnel.Listener.Accept()
		if err != nil {
			select {
			case <-ctx.Done():
				return
			default:
				// Store error and stop
				tm.mu.Lock()
				if t, exists := tm.tunnels[forwardID]; exists {
					t.Error = fmt.Sprintf("accept error: %v", err)
				}
				tm.mu.Unlock()
				return
			}
		}

		// Dial remote through SSH tunnel
		remoteConn, err := tunnel.SSHClient.Dial("tcp", remoteAddr)
		if err != nil {
			log.Printf("Forward %s: failed to dial remote %s: %v", forwardID, remoteAddr, err)
			localConn.Close()
			continue
		}

		// Bidirectional copy (D-06)
		go func(local, remote net.Conn) {
			defer local.Close()
			defer remote.Close()
			io.Copy(local, remote)
		}(localConn, remoteConn)

		go func(local, remote net.Conn) {
			defer local.Close()
			defer remote.Close()
			io.Copy(remote, local)
		}(localConn, remoteConn)
	}
}

// Stop deactivates a port forwarding tunnel.
func (tm *TunnelManager) Stop(forwardID string) error {
	tm.mu.Lock()
	defer tm.mu.Unlock()

	tunnel, exists := tm.tunnels[forwardID]
	if !exists {
		return nil // Already stopped, not an error
	}

	// Cancel accept loop goroutine
	tunnel.Cancel()

	// Close listener
	if tunnel.Listener != nil {
		tunnel.Listener.Close()
	}

	// Close SSH client
	if tunnel.SSHClient != nil {
		tunnel.SSHClient.Close()
	}

	delete(tm.tunnels, forwardID)
	return nil
}

// IsActive returns whether a tunnel is currently active for the given forward ID.
func (tm *TunnelManager) IsActive(forwardID string) bool {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	_, exists := tm.tunnels[forwardID]
	return exists
}

// GetError returns any stored error for the given forward's tunnel.
func (tm *TunnelManager) GetError(forwardID string) string {
	tm.mu.Lock()
	defer tm.mu.Unlock()
	if tunnel, exists := tm.tunnels[forwardID]; exists {
		return tunnel.Error
	}
	return ""
}

// StopAll stops all active tunnels (for graceful shutdown).
func (tm *TunnelManager) StopAll() {
	tm.mu.Lock()
	defer tm.mu.Unlock()

	for id, tunnel := range tm.tunnels {
		tunnel.Cancel()
		if tunnel.Listener != nil {
			tunnel.Listener.Close()
		}
		if tunnel.SSHClient != nil {
			tunnel.SSHClient.Close()
		}
		delete(tm.tunnels, id)
	}
}
