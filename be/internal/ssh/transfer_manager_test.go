package ssh

import (
	"errors"
	"testing"
)

func TestTransferManager(t *testing.T) {
	tm := NewTransferManager()
	id := "test-id"

	// Create
	tm.CreateTransfer(id, 100)
	status, err := tm.GetStatus(id)
	if err != nil {
		t.Fatalf("expected status, got error: %v", err)
	}
	if status.TotalBytes != 100 {
		t.Errorf("expected 100 total bytes, got %d", status.TotalBytes)
	}
	if status.Status != TransferPhasePending {
		t.Errorf("expected status pending, got %s", status.Status)
	}

	// Update Progress
	tm.UpdateProgress(id, 50)
	status, _ = tm.GetStatus(id)
	if status.BytesTransferred != 50 {
		t.Errorf("expected 50 bytes transferred, got %d", status.BytesTransferred)
	}
	if status.Status != TransferPhaseTransferring {
		t.Errorf("expected status transferring, got %s", status.Status)
	}

	// Set Complete
	tm.SetComplete(id)
	status, _ = tm.GetStatus(id)
	if status.Status != TransferPhaseCompleted {
		t.Errorf("expected status completed, got %s", status.Status)
	}
	if status.BytesTransferred != 100 {
		t.Errorf("expected 100 bytes transferred on completion, got %d", status.BytesTransferred)
	}

	// Set Error
	id2 := "test-id-2"
	tm.CreateTransfer(id2, 200)
	tm.SetError(id2, errors.New("failed"))
	status, _ = tm.GetStatus(id2)
	if status.Status != TransferPhaseError {
		t.Errorf("expected status error, got %s", status.Status)
	}
	if status.Error != "failed" {
		t.Errorf("expected error message 'failed', got %s", status.Error)
	}

	// Remove
	tm.RemoveTransfer(id)
	_, err = tm.GetStatus(id)
	if err == nil {
		t.Error("expected error getting removed transfer, got nil")
	}
}
