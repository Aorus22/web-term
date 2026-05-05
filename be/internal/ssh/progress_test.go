package ssh

import (
	"bytes"
	"io"
	"testing"
)

func TestProgressWriter(t *testing.T) {
	tm := NewTransferManager()
	id := "test-id"
	tm.CreateTransfer(id, 10)

	var buf bytes.Buffer
	pw := NewProgressWriter(id, tm, 10, &buf)

	data := []byte("hello")
	n, err := pw.Write(data)
	if err != nil {
		t.Fatalf("Write failed: %v", err)
	}
	if n != 5 {
		t.Errorf("expected 5 bytes written, got %d", n)
	}

	status, _ := tm.GetStatus(id)
	if status.BytesTransferred != 5 {
		t.Errorf("expected 5 bytes in manager, got %d", status.BytesTransferred)
	}

	n, _ = pw.Write([]byte("world"))
	status, _ = tm.GetStatus(id)
	if status.BytesTransferred != 10 {
		t.Errorf("expected 10 bytes in manager, got %d", status.BytesTransferred)
	}
}

func TestProgressWriterWithIoCopy(t *testing.T) {
	tm := NewTransferManager()
	id := "test-id"
	tm.CreateTransfer(id, 10)

	src := bytes.NewReader([]byte("1234567890"))
	var dst bytes.Buffer
	pw := NewProgressWriter(id, tm, 10, &dst)

	_, err := io.Copy(pw, src)
	if err != nil {
		t.Fatalf("io.Copy failed: %v", err)
	}

	status, _ := tm.GetStatus(id)
	if status.BytesTransferred != 10 {
		t.Errorf("expected 10 bytes in manager, got %d", status.BytesTransferred)
	}
}
