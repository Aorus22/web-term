package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"webterm/internal/config"
	"webterm/internal/db"
	"webterm/internal/ssh"

	"gorm.io/gorm"
)

type SFTPHandler struct {
	DB  *gorm.DB
	Cfg *config.Config
}

func (h *SFTPHandler) getFS(r *http.Request) (ssh.FileSystem, io.Closer, error) {
	connID := r.URL.Query().Get("connectionId")
	if connID == "local" || connID == "" {
		return &ssh.LocalFS{}, nil, nil
	}

	var dbConn db.Connection
	if err := h.DB.First(&dbConn, "id = ?", connID).Error; err != nil {
		return nil, nil, fmt.Errorf("connection not found")
	}

	passphrase := r.Header.Get("X-SSH-Passphrase")

	sftpClient, sshClient, err := ssh.ConnectSFTP(h.DB, dbConn, h.Cfg, passphrase)
	if err != nil {
		return nil, nil, err
	}

	return &ssh.SFTPFS{Client: sftpClient}, sshClient, nil
}

func (h *SFTPHandler) List(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	path := r.URL.Query().Get("path")
	if path == "" {
		path = "."
	}

	files, err := fs.List(path)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(files)
}

func (h *SFTPHandler) Download(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	path := r.URL.Query().Get("path")
	reader, err := fs.Read(path)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	defer reader.Close()

	info, err := fs.Stat(path)
	if err == nil {
		w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=\"%s\"", info.Name))
		w.Header().Set("Content-Type", "application/octet-stream")
		w.Header().Set("Content-Length", fmt.Sprintf("%d", info.Size))
	}

	io.Copy(w, reader)
}

func (h *SFTPHandler) Upload(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	path := r.URL.Query().Get("path")
	
	// Handle multipart form
	err = r.ParseMultipartForm(32 << 20) // 32MB max in memory
	if err != nil {
		sendError(w, "Failed to parse multipart form", http.StatusBadRequest)
		return
	}

	file, _, err := r.FormFile("file")
	if err != nil {
		sendError(w, "File is required", http.StatusBadRequest)
		return
	}
	defer file.Close()

	err = fs.Write(path, file)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
}

func (h *SFTPHandler) Remove(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	path := r.URL.Query().Get("path")
	err = fs.Remove(path)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *SFTPHandler) Rename(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	oldPath := r.URL.Query().Get("oldPath")
	newPath := r.URL.Query().Get("newPath")
	err = fs.Rename(oldPath, newPath)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
}

func (h *SFTPHandler) Mkdir(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	path := r.URL.Query().Get("path")
	err = fs.Mkdir(path)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusCreated)
}
