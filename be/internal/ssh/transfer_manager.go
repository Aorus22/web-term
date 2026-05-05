package ssh

import (
	"fmt"
	"sync"
)

type TransferPhase string

const (
	TransferPhasePending     TransferPhase = "pending"
	TransferPhaseTransferring TransferPhase = "transferring"
	TransferPhaseCompleted    TransferPhase = "completed"
	TransferPhaseError        TransferPhase = "error"
)

type TransferStatus struct {
	ID               string        `json:"id"`
	BytesTransferred int64         `json:"bytes_transferred"`
	TotalBytes       int64         `json:"total_bytes"`
	Status           TransferPhase `json:"status"`
	Error            string        `json:"error,omitempty"`
}

type TransferManager struct {
	mu        sync.RWMutex
	transfers map[string]*TransferStatus
}

func NewTransferManager() *TransferManager {
	return &TransferManager{
		transfers: make(map[string]*TransferStatus),
	}
}

func (m *TransferManager) CreateTransfer(id string, totalBytes int64) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.transfers[id] = &TransferStatus{
		ID:         id,
		TotalBytes: totalBytes,
		Status:     TransferPhasePending,
	}
}

func (m *TransferManager) UpdateProgress(id string, bytesTransferred int64) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if t, ok := m.transfers[id]; ok {
		t.BytesTransferred = bytesTransferred
		t.Status = TransferPhaseTransferring
	}
}

func (m *TransferManager) SetComplete(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if t, ok := m.transfers[id]; ok {
		t.Status = TransferPhaseCompleted
		t.BytesTransferred = t.TotalBytes
	}
}

func (m *TransferManager) SetError(id string, err error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if t, ok := m.transfers[id]; ok {
		t.Status = TransferPhaseError
		t.Error = err.Error()
	}
}

func (m *TransferManager) GetStatus(id string) (*TransferStatus, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if t, ok := m.transfers[id]; ok {
		// Return a copy to avoid race conditions when the caller reads it
		copy := *t
		return &copy, nil
	}
	return nil, fmt.Errorf("transfer not found: %s", id)
}

func (m *TransferManager) RemoveTransfer(id string) {
	m.mu.Lock()
	defer m.mu.Unlock()
	delete(m.transfers, id)
}
