package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"
	"webterm/internal/ssh"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type SFTPHandler struct {
	DB  *gorm.DB
	Cfg *config.Config
	SM  *ssh.StagingManager
	TM  *ssh.TransferManager
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
	connID := r.URL.Query().Get("connectionId")
	path := r.URL.Query().Get("path")

	// Handle multipart form
	err := r.ParseMultipartForm(32 << 20) // 32MB max in memory
	if err != nil {
		sendError(w, "Failed to parse multipart form", http.StatusBadRequest)
		return
	}

	file, header, err := r.FormFile("file")
	if err != nil {
		sendError(w, "File is required", http.StatusBadRequest)
		return
	}
	defer file.Close()

	// Stage file first
	stagingPath, err := h.SM.StageFile(file)
	if err != nil {
		sendError(w, fmt.Sprintf("Failed to stage file: %v", err), http.StatusInternalServerError)
		return
	}

	transferID := uuid.New().String()
	h.TM.CreateTransfer(transferID, header.Size)

	// Return transferId immediately
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"transferId": transferID})

	// Start background transfer from staging to destination
	go func() {
		defer h.SM.Cleanup(stagingPath)

		getFS := func(connID string) (ssh.FileSystem, io.Closer, error) {
			if connID == "local" || connID == "" {
				return &ssh.LocalFS{}, nil, nil
			}
			var dbConn db.Connection
			if err := h.DB.First(&dbConn, "id = ?", connID).Error; err != nil {
				return nil, nil, fmt.Errorf("connection %s not found", connID)
			}
			sftpClient, sshClient, err := ssh.ConnectSFTP(h.DB, dbConn, h.Cfg, "")
			if err != nil {
				return nil, nil, err
			}
			return &ssh.SFTPFS{Client: sftpClient}, sshClient, nil
		}

		dstFS, dstCloser, err := getFS(connID)
		if err != nil {
			h.TM.SetError(transferID, err)
			return
		}
		if dstCloser != nil {
			defer dstCloser.Close()
		}

		stagingFile, err := os.Open(stagingPath)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to open staging file: %v", err))
			return
		}
		defer stagingFile.Close()

		pr := ssh.NewProgressReader(transferID, h.TM, header.Size, stagingFile)
		err = dstFS.Write(path, pr)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to write to destination: %v", err))
			return
		}

		h.TM.SetComplete(transferID)
	}()
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

func (h *SFTPHandler) Progress(w http.ResponseWriter, r *http.Request) {
	transferID := r.PathValue("id")
	if transferID == "" {
		transferID = r.URL.Query().Get("id")
	}

	// Set headers for SSE
	w.Header().Set("Content-Type", "text/event-stream")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}

	ticker := time.NewTicker(200 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case <-ticker.C:
			status, err := h.TM.GetStatus(transferID)
			if err != nil {
				fmt.Fprintf(w, "event: error\ndata: {\"message\": \"%s\"}\n\n", err.Error())
				flusher.Flush()
				return
			}

			data, _ := json.Marshal(status)
			fmt.Fprintf(w, "data: %s\n\n", string(data))
			flusher.Flush()

			if status.Status == ssh.TransferPhaseCompleted || status.Status == ssh.TransferPhaseError {
				// Wait a bit before closing to ensure client gets the final state
				time.Sleep(500 * time.Millisecond)
				return
			}
		}
	}
}

func (h *SFTPHandler) Transfer(w http.ResponseWriter, r *http.Request) {
	srcConnID := r.URL.Query().Get("srcConnectionId")
	srcPath := r.URL.Query().Get("srcPath")
	dstConnID := r.URL.Query().Get("dstConnectionId")
	dstPath := r.URL.Query().Get("dstPath")

	transferID := uuid.New().String()

	// Return transferId immediately
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"transferId": transferID})

	// Start background transfer
	go func() {
		getFS := func(connID string) (ssh.FileSystem, io.Closer, error) {
			if connID == "local" || connID == "" {
				return &ssh.LocalFS{}, nil, nil
			}
			var dbConn db.Connection
			if err := h.DB.First(&dbConn, "id = ?", connID).Error; err != nil {
				return nil, nil, fmt.Errorf("connection %s not found", connID)
			}
			sftpClient, sshClient, err := ssh.ConnectSFTP(h.DB, dbConn, h.Cfg, "")
			if err != nil {
				return nil, nil, err
			}
			return &ssh.SFTPFS{Client: sftpClient}, sshClient, nil
		}

		srcFS, srcCloser, err := getFS(srcConnID)
		if err != nil {
			h.TM.SetError(transferID, err)
			return
		}
		if srcCloser != nil {
			defer srcCloser.Close()
		}

		dstFS, dstCloser, err := getFS(dstConnID)
		if err != nil {
			h.TM.SetError(transferID, err)
			return
		}
		if dstCloser != nil {
			defer dstCloser.Close()
		}

		info, err := srcFS.Stat(srcPath)
		if err != nil {
			h.TM.SetError(transferID, err)
			return
		}

		h.TM.CreateTransfer(transferID, info.Size)

		reader, err := srcFS.Read(srcPath)
		if err != nil {
			h.TM.SetError(transferID, err)
			return
		}
		defer reader.Close()

		stagingPath, err := h.SM.StageFile(reader)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to stage file: %v", err))
			return
		}
		defer h.SM.Cleanup(stagingPath)

		// Stream from staging to destination with progress
		stagingFile, err := os.Open(stagingPath)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to open staging file: %v", err))
			return
		}
		defer stagingFile.Close()

		pr := ssh.NewProgressReader(transferID, h.TM, info.Size, stagingFile)
		err = dstFS.Write(dstPath, pr)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to write to destination: %v", err))
			return
		}

		h.TM.SetComplete(transferID)
	}()
}
