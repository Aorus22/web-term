package ssh

import (
	"bytes"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"webterm/internal/config"
)

func TestParseRsyncProgress2(t *testing.T) {
	cases := []struct {
		name  string
		line  string
		bytes int64
		total int64
		pct   int
		ok    bool
	}{
		{
			name:  "to-chk basic",
			line:  "     12,345,678  45%    3.21MB/s    0:00:04 (xfr#3, to-chk=99/101)",
			bytes: 12345678,
			total: 101,
			pct:   45,
			ok:    true,
		},
		{
			name:  "ir-chk variant",
			line:  "        512  50%    1.00MB/s    0:00:01 (xfr#1, ir-chk=4/8)",
			bytes: 512,
			total: 8,
			pct:   50,
			ok:    true,
		},
		{
			name:  "hundred percent",
			line:  "  1,024  100%    0.00kB/s    0:00:00 (xfr#1, to-chk=0/1)",
			bytes: 1024,
			total: 1,
			pct:   100,
			ok:    true,
		},
		{
			name:  "no commas in byte count",
			line:  "   42   10%    1.00kB/s    0:00:00 (xfr#0, to-chk=90/100)",
			bytes: 42,
			total: 100,
			pct:   10,
			ok:    true,
		},
		{
			name:  "mid transfer with to-chk",
			line:  "        102,400  25%    2.00MB/s    0:00:02 (xfr#2, to-chk=3/4)",
			bytes: 102400,
			total: 4,
			pct:   25,
			ok:    true,
		},
		{
			name:  "parseable but no check token",
			line:  "  12,345  45%    1.00MB/s    0:00:01",
			bytes: 12345,
			total: 0,
			pct:   45,
			ok:    true,
		},
		{
			name:  "rsync error message",
			line:  "rsync error: some problem happened",
			bytes: 0,
			total: 0,
			pct:   0,
			ok:    false,
		},
		{
			name:  "empty line",
			line:  "",
			bytes: 0,
			total: 0,
			pct:   0,
			ok:    false,
		},
		{
			name:  "whitespace only",
			line:  "      ",
			bytes: 0,
			total: 0,
			pct:   0,
			ok:    false,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			b, total, pct, ok := parseRsyncProgress2(tc.line)
			if ok != tc.ok {
				t.Fatalf("ok = %v, want %v", ok, tc.ok)
			}
			if !ok {
				return
			}
			if b != tc.bytes {
				t.Errorf("bytes = %d, want %d", b, tc.bytes)
			}
			if total != tc.total {
				t.Errorf("total = %d, want %d", total, tc.total)
			}
			if pct != tc.pct {
				t.Errorf("pct = %d, want %d", pct, tc.pct)
			}
		})
	}
}

func TestBuildEphemeralAuthorizedKeyLine(t *testing.T) {
	pubKey := "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAII7x0eJhY9yDqQ5epQ1uY7JcIq2tXzMlWfYQqKoZl8cV example-key"
	id := "test-id-123"

	line := buildEphemeralAuthorizedKeyLine(pubKey, id)

	for _, want := range []string{
		`command="rsync --server $SSH_ORIGINAL_COMMAND"`,
		"no-agent-forwarding",
		"no-port-forwarding",
		"no-X11-forwarding",
		"no-pty",
		pubKey,
		"webterm-ephemeral-" + id,
	} {
		if !strings.Contains(line, want) {
			t.Errorf("authorized_keys line missing %q\nfull line: %s", want, line)
		}
	}

	if strings.Contains(line, "\n") {
		t.Errorf("authorized_keys line must be a single line: %q", line)
	}
}

func TestCheckLocalRsync(t *testing.T) {
	a := CheckLocalRsync()

	if a.Available {
		if a.RsyncPath == "" {
			t.Error("Available but RsyncPath is empty")
		}
		if a.SSHPath == "" {
			t.Error("Available but SSHPath is empty")
		}
		if a.RsyncVersion == "" {
			t.Error("Available but RsyncVersion is empty")
		}
		if a.SSHVersion == "" {
			t.Error("Available but SSHVersion is empty")
		}
	}

	// If the binaries exist on PATH, the struct must report their paths.
	if _, err := exec.LookPath("rsync"); err == nil && a.RsyncPath == "" {
		t.Error("rsync found on PATH but RsyncPath is empty")
	}
	if _, err := exec.LookPath("ssh"); err == nil && a.SSHPath == "" {
		t.Error("ssh found on PATH but SSHPath is empty")
	}
}

func TestRunRsyncTransferLocalToLocal(t *testing.T) {
	if !CheckLocalRsync().Available {
		t.Skip("rsync and/or ssh not available locally; skipping local-to-local transfer test")
	}

	srcDir := t.TempDir()
	dstDir := t.TempDir()

	srcFile := filepath.Join(srcDir, "hello.txt")
	content := []byte("hello from webterm rsync local transfer\n")
	if err := os.WriteFile(srcFile, content, 0644); err != nil {
		t.Fatalf("failed to write source file: %v", err)
	}

	tm := NewTransferManager()
	err := RunRsyncTransfer(nil, &config.Config{}, tm, "test", "local", srcDir, "local", dstDir)
	if err != nil {
		t.Fatalf("RunRsyncTransfer failed: %v", err)
	}

	status, err := tm.GetStatus("test")
	if err != nil {
		t.Fatalf("GetStatus failed: %v", err)
	}
	if status.Status != TransferPhaseCompleted {
		t.Fatalf("status = %s, want %s", status.Status, TransferPhaseCompleted)
	}

	// rsync -a <srcDir> <dstDir> copies the source directory into the
	// destination as a subdirectory (no trailing slash on the source).
	copied := filepath.Join(dstDir, filepath.Base(srcDir), "hello.txt")
	got, err := os.ReadFile(copied)
	if err != nil {
		t.Fatalf("expected copied file at %s: %v", copied, err)
	}
	if !bytes.Equal(got, content) {
		t.Errorf("content mismatch:\n got %q\nwant %q", got, content)
	}
}
