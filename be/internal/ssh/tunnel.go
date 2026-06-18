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
// Listener is the listening end: a local TCP listener for "local" type, or
// an SSH-side remote listener (via ssh.Client.Listen) for "reverse" type.
type ActiveTunnel struct {
	Listener  net.Listener
	SSHClient *ssh.Client
	Cancel    context.CancelFunc
	Error     string
	Type      string // "local" or "reverse"
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

// Forward type constants
const (
	ForwardTypeLocal   = "local"
	ForwardTypeReverse = "reverse"
)

// Start establishes an SSH tunnel for the given forward rule.
// fwdType must be ForwardTypeLocal or ForwardTypeReverse.
// For "local":  binds 127.0.0.1:localPort locally, forwards to remoteHost:remotePort through SSH (-L semantics).
// For "reverse": opens a listener on remoteHost 0.0.0.0:remotePort through SSH, forwards back to 127.0.0.1:localPort (-R semantics).
func (tm *TunnelManager) Start(forwardID string, fwdType string, conn db.Connection, localPort int, remotePort int) error {
	if fwdType != ForwardTypeLocal && fwdType != ForwardTypeReverse {
		return fmt.Errorf("invalid forward type %q (must be %q or %q)", fwdType, ForwardTypeLocal, ForwardTypeReverse)
	}

	tm.mu.Lock()
	defer tm.mu.Unlock()

	// Check if already active
	if _, exists := tm.tunnels[forwardID]; exists {
		return fmt.Errorf("forward %s is already active", forwardID)
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
			return fmt.Errorf("SSH key not found: %v", err)
		}

		decryptedKey, err := config.DecryptWithAAD(key.EncryptedKey, tm.cfg.EncryptionKey, []byte(config.SSHKeyAAD))
		if err != nil {
			return fmt.Errorf("failed to decrypt SSH key: %v", err)
		}

		parsedKey, err := ssh.ParseRawPrivateKey([]byte(decryptedKey))
		if err != nil {
			return fmt.Errorf("failed to parse SSH key: %v", err)
		}

		keySigner, err := ssh.NewSignerFromKey(parsedKey)
		if err != nil {
			return fmt.Errorf("failed to create SSH signer: %v", err)
		}

		sshConfig.Auth = []ssh.AuthMethod{ssh.PublicKeys(keySigner)}
	} else {
		// PASSWORD AUTH PATH
		decrypted, err := config.Decrypt(conn.Encrypted, tm.cfg.EncryptionKey)
		if err != nil {
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
		return fmt.Errorf("SSH connection failed: %v", err)
	}

	var listener net.Listener
	if fwdType == ForwardTypeReverse {
		// Open a remote-side listener via SSH. The remote host will accept
		// connections on 0.0.0.0:remotePort and tunnel them back to us.
		sshListener, err := client.Listen("tcp", fmt.Sprintf("0.0.0.0:%d", remotePort))
		if err != nil {
			client.Close()
			return fmt.Errorf("failed to open reverse listener on remote %s:%d: %v", conn.Host, remotePort, err)
		}
		listener = sshListener
	} else {
		// Local forward: bind a TCP listener on 127.0.0.1:localPort.
		// This catches port conflicts (D-10).
		localListener, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", localPort))
		if err != nil {
			client.Close()
			return fmt.Errorf("Port %d is already in use", localPort)
		}
		listener = localListener
	}

	// Setup context for clean shutdown
	ctx, cancel := context.WithCancel(context.Background())

	tunnel := &ActiveTunnel{
		Listener:  listener,
		SSHClient: client,
		Cancel:    cancel,
		Type:      fwdType,
	}

	tm.tunnels[forwardID] = tunnel

	// Accept loop in background goroutine
	go tm.acceptLoop(ctx, tunnel, forwardID, localPort, remotePort)

	return nil
}

// acceptLoop accepts incoming connections on the listener and forwards them
// through the SSH tunnel. The per-connection behaviour depends on tunnel.Type:
//   - "local":  each Accept yields a local-side conn; we Dial through SSH to remoteHost:remotePort.
//   - "reverse": each Accept already returns a tunneled conn; we Dial 127.0.0.1:localPort on the backend.
func (tm *TunnelManager) acceptLoop(ctx context.Context, tunnel *ActiveTunnel, forwardID string, localPort int, remotePort int) {
	for {
		select {
		case <-ctx.Done():
			return
		default:
		}

		conn, err := tunnel.Listener.Accept()
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

		if tunnel.Type == ForwardTypeReverse {
			// Reverse forward: the accepted conn is already tunneled through SSH.
			// Dial the local target on the backend and pipe bidirectionally.
			localConn, err := net.Dial("tcp", fmt.Sprintf("127.0.0.1:%d", localPort))
			if err != nil {
				log.Printf("Forward %s: failed to dial local target 127.0.0.1:%d: %v", forwardID, localPort, err)
				conn.Close()
				continue
			}
			pipeConns(conn, localConn)
		} else {
			// Local forward: dial the remote target through the SSH tunnel.
			remoteAddr := fmt.Sprintf("localhost:%d", remotePort)
			remoteConn, err := tunnel.SSHClient.Dial("tcp", remoteAddr)
			if err != nil {
				log.Printf("Forward %s: failed to dial remote %s: %v", forwardID, remoteAddr, err)
				conn.Close()
				continue
			}
			pipeConns(conn, remoteConn)
		}
	}
}

// pipeConns bidirectionally copies bytes between two net.Conns and closes
// them when either side finishes. Both directions run as separate goroutines
// (D-06); deferring Close on each side means a net.Conn.Close call may run
// twice (once per goroutine) but net.Conn.Close is idempotent.
func pipeConns(a, b net.Conn) {
	go func() {
		defer a.Close()
		defer b.Close()
		io.Copy(a, b)
	}()

	go func() {
		defer a.Close()
		defer b.Close()
		io.Copy(b, a)
	}()
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
