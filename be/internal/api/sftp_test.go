package api

import (
	"bytes"
	"encoding/json"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
	"webterm/internal/config"
	"webterm/internal/ssh"
)

func TestSFTPHandler_Local(t *testing.T) {
	tmpDir, err := os.MkdirTemp("", "sftp-handler-test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	cfg := &config.Config{}
	sm, err := ssh.NewStagingManager(filepath.Join(tmpDir, "staging"))
	if err != nil {
		t.Fatal(err)
	}
	h := &SFTPHandler{
		Cfg: cfg,
		SM:  sm,
		TM:  ssh.NewTransferManager(),
	}

	// 1. Test Mkdir
	path := filepath.Join(tmpDir, "testdir")
	req := httptest.NewRequest("POST", "/api/sftp/mkdir?connectionId=local&path="+path, nil)
	rr := httptest.NewRecorder()
	h.Mkdir(rr, req)
	if rr.Code != http.StatusCreated {
		t.Errorf("Mkdir failed: %d %s", rr.Code, rr.Body.String())
	}

	// 2. Test Upload
	filePath := filepath.Join(path, "test.txt")
	body := &bytes.Buffer{}
	writer := multipart.NewWriter(body)
	part, _ := writer.CreateFormFile("file", "test.txt")
	part.Write([]byte("hello world"))
	writer.Close()

	req = httptest.NewRequest("POST", "/api/sftp/upload?connectionId=local&path="+filePath, body)
	req.Header.Set("Content-Type", writer.FormDataContentType())
	rr = httptest.NewRecorder()
	h.Upload(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("Upload failed: %d %s", rr.Code, rr.Body.String())
	}

	// Upload completes asynchronously in a goroutine; wait for the destination
	// file to appear before asserting on it.
	waitForFile(t, filePath)

	// 3. Test List
	req = httptest.NewRequest("GET", "/api/sftp/ls?connectionId=local&path="+path, nil)
	rr = httptest.NewRecorder()
	h.List(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("List failed: %d %s", rr.Code, rr.Body.String())
	}
	var files []ssh.FileInfo
	json.Unmarshal(rr.Body.Bytes(), &files)
	if len(files) != 1 || files[0].Name != "test.txt" {
		t.Errorf("List output mismatch: %v", files)
	}

	// 4. Test Download
	req = httptest.NewRequest("GET", "/api/sftp/download?connectionId=local&path="+filePath, nil)
	rr = httptest.NewRecorder()
	h.Download(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("Download failed: %d %s", rr.Code, rr.Body.String())
	}
	if rr.Body.String() != "hello world" {
		t.Errorf("Download content mismatch: %s", rr.Body.String())
	}

	// 5. Test Rename
	newFilePath := filepath.Join(path, "test2.txt")
	req = httptest.NewRequest("POST", "/api/sftp/rename?connectionId=local&oldPath="+filePath+"&newPath="+newFilePath, nil)
	rr = httptest.NewRecorder()
	h.Rename(rr, req)
	if rr.Code != http.StatusOK {
		t.Errorf("Rename failed: %d %s", rr.Code, rr.Body.String())
	}

	// 6. Test Remove
	req = httptest.NewRequest("DELETE", "/api/sftp/remove?connectionId=local&path="+newFilePath, nil)
	rr = httptest.NewRecorder()
	h.Remove(rr, req)
	if rr.Code != http.StatusNoContent {
		t.Errorf("Remove failed: %d %s", rr.Code, rr.Body.String())
	}
}

func TestEnginesEndpoint_Local(t *testing.T) {
	h := &SFTPHandler{}

	req := httptest.NewRequest("GET", "/api/sftp/engines?connectionId=local", nil)
	rr := httptest.NewRecorder()
	h.Engines(rr, req)

	if rr.Code != http.StatusOK {
		t.Fatalf("Engines failed: %d %s", rr.Code, rr.Body.String())
	}
	if ct := rr.Header().Get("Content-Type"); ct != "application/json" {
		t.Errorf("Content-Type should be application/json, got %q", ct)
	}

	var resp struct {
		Available []string         `json:"available"`
		Rsync     json.RawMessage  `json:"rsync"`
	}
	if err := json.Unmarshal(rr.Body.Bytes(), &resp); err != nil {
		t.Fatalf("Failed to unmarshal response: %v", err)
	}

	foundSFTP := false
	for _, e := range resp.Available {
		if e == "sftp" {
			foundSFTP = true
			break
		}
	}
	if !foundSFTP {
		t.Errorf("available should contain 'sftp', got %v", resp.Available)
	}

	if resp.Rsync == nil {
		t.Fatal("rsync key should be present in response")
	}
	if string(resp.Rsync) == "null" {
		// rsync not installed on this machine; null is valid.
		return
	}
	var rsyncObj map[string]interface{}
	if err := json.Unmarshal(resp.Rsync, &rsyncObj); err != nil {
		t.Fatalf("rsync should be an object or null, got %s", resp.Rsync)
	}
	if _, ok := rsyncObj["available"]; !ok {
		t.Errorf("rsync object should contain an 'available' field, got %v", rsyncObj)
	}
	if rsv := rsyncObj["rsync_version"]; rsv != nil {
		if s, ok := rsv.(string); !ok || s == "" {
			t.Errorf("rsync_version should be a non-empty string when present, got %v", rsv)
		}
	}
}

func TestTransferEngineInvalid(t *testing.T) {
	h := &SFTPHandler{TM: ssh.NewTransferManager()}

	req := httptest.NewRequest("POST", "/api/sftp/transfer?srcConnectionId=local&srcPath=x&dstConnectionId=local&dstPath=y&engine=ftp", nil)
	rr := httptest.NewRecorder()
	h.Transfer(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Fatalf("Expected 400 for invalid engine, got %d %s", rr.Code, rr.Body.String())
	}
}

func TestTransferEngineRsyncDisabled(t *testing.T) {
	// nil DB => rsyncEnabled() returns false.
	h := &SFTPHandler{TM: ssh.NewTransferManager()}

	req := httptest.NewRequest("POST", "/api/sftp/transfer?srcConnectionId=local&srcPath=x&dstConnectionId=local&dstPath=y&engine=rsync", nil)
	rr := httptest.NewRecorder()
	h.Transfer(rr, req)

	if rr.Code != http.StatusBadRequest {
		t.Fatalf("Expected 400 when rsync disabled, got %d %s", rr.Code, rr.Body.String())
	}
	if !strings.Contains(rr.Body.String(), "disabled") {
		t.Errorf("Message should mention 'disabled', got %s", rr.Body.String())
	}
}

// waitForFile polls until path exists or the deadline elapses, so that tests
// can assert on files produced by asynchronous handlers.
func waitForFile(t *testing.T, path string) {
	t.Helper()
	deadline := time.Now().Add(5 * time.Second)
	for time.Now().Before(deadline) {
		if _, err := os.Stat(path); err == nil {
			return
		}
		time.Sleep(20 * time.Millisecond)
	}
	t.Fatalf("timed out waiting for %s", path)
}
