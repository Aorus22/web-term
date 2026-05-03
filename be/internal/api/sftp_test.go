package api

import (
	"bytes"
	"encoding/json"
	"mime/multipart"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
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
	h := &SFTPHandler{
		Cfg: cfg,
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
