package ssh

import (
	"testing"
	"time"
)

func TestSpawnLocalPTY_Default(t *testing.T) {
	msg := ConnectMessage{
		Cols: 80,
		Rows: 24,
	}

	pty, pid, err := spawnLocalPTY(msg)
	if err != nil {
		t.Fatalf("spawnLocalPTY failed: %v", err)
	}
	defer pty.Close()

	if pid <= 0 {
		t.Errorf("expected positive pid, got %d", pid)
	}

	// Test write to local PTY
	_, err = pty.Write([]byte("\n"))
	if err != nil {
		t.Errorf("failed to write newline to PTY: %v", err)
	}

	// Test resize
	err = resizeLocalPTY(pty, 100, 30)
	if err != nil {
		t.Errorf("resizeLocalPTY failed: %v", err)
	}

	// Allow child process a moment to receive input before teardown
	time.Sleep(50 * time.Millisecond)
}

func TestSpawnLocalPTY_WithCwd(t *testing.T) {
	tempDir := t.TempDir()

	msg := ConnectMessage{
		Cols: 80,
		Rows: 24,
		Cwd:  tempDir,
	}

	pty, pid, err := spawnLocalPTY(msg)
	if err != nil {
		t.Fatalf("spawnLocalPTY with custom Cwd failed: %v", err)
	}
	defer pty.Close()

	if pid <= 0 {
		t.Errorf("expected positive pid, got %d", pid)
	}

	err = resizeLocalPTY(pty, 120, 35)
	if err != nil {
		t.Errorf("resizeLocalPTY with custom Cwd failed: %v", err)
	}
}
