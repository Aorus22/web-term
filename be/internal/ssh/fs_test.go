package ssh

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"testing"
)

func TestLocalFS(t *testing.T) {
	fs := &LocalFS{}
	tmpDir, err := os.MkdirTemp("", "wterm-fs-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// Test Mkdir
	path := filepath.Join(tmpDir, "subdir")
	if err := fs.Mkdir(path); err != nil {
		t.Fatalf("Mkdir failed: %v", err)
	}

	// Test Write
	filePath := filepath.Join(path, "test.txt")
	content := "hello world"
	if err := fs.Write(filePath, bytes.NewBufferString(content)); err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	// Test Stat
	info, err := fs.Stat(filePath)
	if err != nil {
		t.Fatalf("Stat failed: %v", err)
	}
	if info.Name != "test.txt" {
		t.Errorf("Expected test.txt, got %s", info.Name)
	}
	if info.Size != int64(len(content)) {
		t.Errorf("Expected size %d, got %d", len(content), info.Size)
	}

	// Test List
	files, err := fs.List(path)
	if err != nil {
		t.Fatalf("List failed: %v", err)
	}
	if len(files) != 1 || files[0].Name != "test.txt" {
		t.Errorf("List failed, expected [test.txt], got %v", files)
	}

	// Test Read
	r, err := fs.Read(filePath)
	if err != nil {
		t.Fatalf("Read failed: %v", err)
	}
	defer r.Close()
	readContent, err := io.ReadAll(r)
	if err != nil {
		t.Fatalf("ReadAll failed: %v", err)
	}
	if string(readContent) != content {
		t.Errorf("Read content mismatch: %s", readContent)
	}

	// Test Rename
	newFilePath := filepath.Join(path, "test2.txt")
	if err := fs.Rename(filePath, newFilePath); err != nil {
		t.Fatalf("Rename failed: %v", err)
	}

	// Test Remove
	if err := fs.Remove(newFilePath); err != nil {
		t.Fatalf("Remove failed: %v", err)
	}
	if _, err := os.Stat(newFilePath); !os.IsNotExist(err) {
		t.Errorf("File should be removed")
	}
}
