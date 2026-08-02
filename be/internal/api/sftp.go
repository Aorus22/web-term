package api

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"sync"
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

	// rsync probe cache keyed by connectionId (remote connections only).
	rsyncMu    sync.Mutex
	rsyncCache map[string]cachedRsync
}

// rsyncCacheTTL is how long a cached remote rsync probe result is considered valid.
const rsyncCacheTTL = 60 * time.Second

type cachedRsync struct {
	info     ssh.RsyncAvailability
	probedAt time.Time
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

func (h *SFTPHandler) rsyncEnabled() bool {
	if h.DB == nil {
		return false
	}
	var setting db.Setting
	if err := h.DB.First(&setting, "key = ?", "rsync_enabled").Error; err != nil {
		return false
	}
	return setting.Value == "true"
}

// rsyncAvailable reports whether the rsync engine can be used to reach connID.
// For local connections it checks the backend machine's rsync + ssh binaries.
// For remote connections it resolves the connection and probes rsync over ssh.
func (h *SFTPHandler) rsyncAvailable(connID string) bool {
	if connID == "" || connID == "local" {
		return ssh.CheckLocalRsync().Available
	}
	if h.DB == nil {
		return false
	}
	var dbConn db.Connection
	if err := h.DB.First(&dbConn, "id = ?", connID).Error; err != nil {
		return false
	}
	available, _, err := ssh.ProbeRemoteRsync(h.DB, dbConn, h.Cfg)
	if err != nil {
		return false
	}
	return available
}

// Engines reports which transfer engines are available for a connection.
// Response shape:
//
//	{
//	  "available": ["sftp", "rsync"],
//	  "rsync": { "available": true, "rsync_path": "...", "rsync_version": "...", "ssh_path": "...", "ssh_version": "..." }
//	}
//
// "rsync" is null when rsync is unavailable; "available" always contains
// "sftp". A failed remote probe is not an error: it just means no rsync.
func (h *SFTPHandler) Engines(w http.ResponseWriter, r *http.Request) {
	connID := r.URL.Query().Get("connectionId")
	fresh := r.URL.Query().Get("fresh") == "1"

	w.Header().Set("Content-Type", "application/json")

	resp := map[string]interface{}{
		"available": []string{"sftp"},
		"rsync":     nil,
	}

	rsyncInfo := func(info ssh.RsyncAvailability) {
		if info.Available {
			resp["available"] = []string{"sftp", "rsync"}
			resp["rsync"] = info
		}
	}

	if connID == "" || connID == "local" {
		rsyncInfo(ssh.CheckLocalRsync())
		json.NewEncoder(w).Encode(resp)
		return
	}

	var dbConn db.Connection
	if err := h.DB.First(&dbConn, "id = ?", connID).Error; err != nil {
		sendError(w, "connection not found", http.StatusNotFound)
		return
	}

	if !fresh {
		h.rsyncMu.Lock()
		cached, ok := h.rsyncCache[connID]
		h.rsyncMu.Unlock()
		if ok && time.Since(cached.probedAt) < rsyncCacheTTL {
			rsyncInfo(cached.info)
			json.NewEncoder(w).Encode(resp)
			return
		}
	}

	available, version, err := ssh.ProbeRemoteRsync(h.DB, dbConn, h.Cfg)
	if err != nil {
		// A failed probe just means rsync is not available.
		json.NewEncoder(w).Encode(resp)
		return
	}

	info := ssh.RsyncAvailability{Available: available, RsyncVersion: version}

	h.rsyncMu.Lock()
	if h.rsyncCache == nil {
		h.rsyncCache = make(map[string]cachedRsync)
	}
	h.rsyncCache[connID] = cachedRsync{info: info, probedAt: time.Now()}
	h.rsyncMu.Unlock()

	rsyncInfo(info)
	json.NewEncoder(w).Encode(resp)
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

func (h *SFTPHandler) Home(w http.ResponseWriter, r *http.Request) {
	fs, closer, err := h.getFS(r)
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}
	if closer != nil {
		defer closer.Close()
	}

	homePath, err := fs.Home()
	if err != nil {
		sendError(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"path": homePath})
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

	engine := r.URL.Query().Get("engine")
	if engine == "" {
		engine = "sftp"
	}
	if engine != "sftp" && engine != "rsync" {
		sendError(w, "Invalid engine: "+engine, http.StatusBadRequest)
		return
	}

	if engine == "rsync" {
		if !h.rsyncEnabled() {
			sendError(w, "Rsync engine is disabled in settings", http.StatusBadRequest)
			return
		}
		// Upload source is always local; only the destination can be remote.
		if !h.rsyncAvailable(connID) {
			sendError(w, "Rsync is not available on the source/destination (install rsync and ssh)", http.StatusBadRequest)
			return
		}
	}

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

		if engine == "rsync" {
			// RunRsyncUpload is the sole SetComplete/SetError authority for the
			// transfer; it sets total bytes from os.Stat. Re-reporting the
			// returned error via SetError is idempotent.
			if err := ssh.RunRsyncUpload(h.DB, h.Cfg, h.TM, transferID, stagingPath, connID, path); err != nil {
				h.TM.SetError(transferID, err)
			}
			return
		}

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

	// Send initial status immediately
	sendStatus := func() bool {
		status, err := h.TM.GetStatus(transferID)
		if err != nil {
			fmt.Fprintf(w, "event: error\ndata: {\"message\": \"%s\"}\n\n", err.Error())
			flusher.Flush()
			return false
		}
		data, _ := json.Marshal(status)
		fmt.Fprintf(w, "data: %s\n\n", string(data))
		flusher.Flush()
		return status.Status != ssh.TransferPhaseCompleted && status.Status != ssh.TransferPhaseError
	}

	if !sendStatus() {
		return
	}

	ticker := time.NewTicker(200 * time.Millisecond)
	defer ticker.Stop()

	for {
		select {
		case <-r.Context().Done():
			return
		case <-ticker.C:
			if !sendStatus() {
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

	engine := r.URL.Query().Get("engine")
	if engine == "" {
		engine = "sftp"
	}
	if engine != "sftp" && engine != "rsync" {
		sendError(w, "Invalid engine: "+engine, http.StatusBadRequest)
		return
	}

	if engine == "rsync" {
		if !h.rsyncEnabled() {
			sendError(w, "Rsync engine is disabled in settings", http.StatusBadRequest)
			return
		}
		if !h.rsyncAvailable(srcConnID) || !h.rsyncAvailable(dstConnID) {
			sendError(w, "Rsync is not available on the source/destination (install rsync and ssh)", http.StatusBadRequest)
			return
		}
	}

	transferID := uuid.New().String()
	h.TM.CreateTransfer(transferID, 0)

	// Return transferId immediately
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{"transferId": transferID})

	// Start background transfer
	go func() {
		if engine == "rsync" {
			// RunRsyncTransfer is the sole SetComplete/SetError authority for
			// the transfer; it reports progress and terminal state itself.
			// Re-reporting the returned error via SetError is idempotent.
			if err := ssh.RunRsyncTransfer(h.DB, h.Cfg, h.TM, transferID, srcConnID, srcPath, dstConnID, dstPath); err != nil {
				h.TM.SetError(transferID, err)
			}
			return
		}

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

		h.TM.SetTotalBytes(transferID, info.Size)

		reader, err := srcFS.Read(srcPath)
		if err != nil {
			h.TM.SetError(transferID, err)
			return
		}
		defer reader.Close()

		pr := ssh.NewProgressReader(transferID, h.TM, info.Size, reader)
		stagingPath, err := h.SM.StageFile(pr)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to stage file: %v", err))
			return
		}
		defer h.SM.Cleanup(stagingPath)

		stagingFile, err := os.Open(stagingPath)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to open staging file: %v", err))
			return
		}
		defer stagingFile.Close()

		err = dstFS.Write(dstPath, stagingFile)
		if err != nil {
			h.TM.SetError(transferID, fmt.Errorf("failed to write to destination: %v", err))
			return
		}

		h.TM.SetComplete(transferID)
	}()
}
