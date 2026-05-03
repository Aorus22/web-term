package ssh

import (
	"io"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
	"golang.org/x/crypto/ssh"
)

// RingBuffer stores the last N bytes of output.
type RingBuffer struct {
	data []byte
	size int
	head int
	full bool
	mu   sync.Mutex
}

func NewRingBuffer(size int) *RingBuffer {
	return &RingBuffer{
		data: make([]byte, size),
		size: size,
	}
}

func (r *RingBuffer) Write(p []byte) (n int, err error) {
	r.mu.Lock()
	defer r.mu.Unlock()

	n = len(p)
	toWrite := p
	if n > r.size {
		toWrite = p[n-r.size:]
	}

	if r.head+len(toWrite) <= r.size {
		copy(r.data[r.head:], toWrite)
		r.head = (r.head + len(toWrite)) % r.size
		if r.head == 0 && len(toWrite) > 0 {
			r.full = true
		}
	} else {
		firstPart := r.size - r.head
		copy(r.data[r.head:], toWrite[:firstPart])
		copy(r.data[0:], toWrite[firstPart:])
		r.head = len(toWrite) - firstPart
		r.full = true
	}

	return n, nil
}

func (r *RingBuffer) Bytes() []byte {
	r.mu.Lock()
	defer r.mu.Unlock()

	if !r.full {
		return r.data[:r.head]
	}

	out := make([]byte, r.size)
	copy(out, r.data[r.head:])
	copy(out[r.size-r.head:], r.data[:r.head])
	return out
}

// ManagedSession represents an active SSH session that can survive WebSocket disconnects.
type ManagedSession struct {
	ID           string
	SSHClient    *ssh.Client
	SSHSession   *ssh.Session
	Stdin        io.WriteCloser
	Host         string
	User         string
	Port         int
	ConnectionID string
	Buffer       *RingBuffer
	WS           *websocket.Conn
	LastSeen     time.Time
	mu           sync.Mutex
}

// SessionManager tracks all active SSH sessions.
type SessionManager struct {
	sessions map[string]*ManagedSession
	mu       sync.RWMutex
}

var GlobalSessionManager *SessionManager

func init() {
	GlobalSessionManager = NewSessionManager()
	go GlobalSessionManager.CleanupWorker()
}

func NewSessionManager() *SessionManager {
	return &SessionManager{
		sessions: make(map[string]*ManagedSession),
	}
}

func (m *SessionManager) CreateSession(client *ssh.Client, session *ssh.Session, stdin io.WriteCloser, host string, user string, port int, connID string) *ManagedSession {
	id := uuid.New().String()
	s := &ManagedSession{
		ID:           id,
		SSHClient:    client,
		SSHSession:   session,
		Stdin:        stdin,
		Host:         host,
		User:         user,
		Port:         port,
		ConnectionID: connID,
		Buffer:       NewRingBuffer(64 * 1024), // 64KB
		LastSeen:     time.Now(),
	}

	m.mu.Lock()
	defer m.mu.Unlock()
	m.sessions[id] = s
	return s
}

func (m *SessionManager) GetSession(id string) (*ManagedSession, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	s, ok := m.sessions[id]
	return s, ok
}

func (m *SessionManager) RemoveSession(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if s, ok := m.sessions[id]; ok {
		s.SSHSession.Close()
		s.SSHClient.Close()
		delete(m.sessions, id)
	}
}

func (m *SessionManager) ListSessions() []*ManagedSession {
	m.mu.RLock()
	defer m.mu.RUnlock()
	list := make([]*ManagedSession, 0, len(m.sessions))
	for _, s := range m.sessions {
		list = append(list, s)
	}
	return list
}

func (m *SessionManager) CleanupWorker() {
	ticker := time.NewTicker(1 * time.Minute)
	for range ticker.C {
		m.mu.Lock()
		for id, s := range m.sessions {
			s.mu.Lock()
			if s.WS == nil && time.Since(s.LastSeen) > 10*time.Minute {
				s.SSHSession.Close()
				s.SSHClient.Close()
				delete(m.sessions, id)
			}
			s.mu.Unlock()
		}
		m.mu.Unlock()
	}
}
