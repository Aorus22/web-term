package api

import (
	"log"
	"net/http"
	"os"
	"path"
	"path/filepath"
	"strings"
	"webterm/internal/config"
	"webterm/internal/ssh"

	"gorm.io/gorm"
)

func SetupRoutes(mux *http.ServeMux, database *gorm.DB, cfg *config.Config) {
	h := &ConnectionHandler{
		DB:  database,
		Cfg: cfg,
	}

	kh := &SSHKeyHandler{
		DB:  database,
		Cfg: cfg,
	}

	// Create tunnel manager for port forwarding
	tunnelMgr := ssh.NewTunnelManager(database, cfg)

	tm := ssh.NewTransferManager()

	sm, err := ssh.NewStagingManager(filepath.Join("storage", "staging"))
	if err != nil {
		log.Printf("Failed to init staging: %v", err)
	}

	fh := &ForwardHandler{
		DB:        database,
		Cfg:       cfg,
		TunnelMgr: tunnelMgr,
	}

	sh := &SettingsHandler{
		DB:  database,
		Cfg: cfg,
	}

	sfh := &SFTPHandler{
		DB:  database,
		Cfg: cfg,
		SM:  sm,
		TM:  tm,
	}

	// Go 1.22+ method routing
	mux.HandleFunc("GET /api/connections", h.ListConnections)
	mux.HandleFunc("GET /api/connections/{id}", h.GetConnection)
	mux.HandleFunc("POST /api/connections", h.CreateConnection)
	mux.HandleFunc("PUT /api/connections/{id}", h.UpdateConnection)
	mux.HandleFunc("DELETE /api/connections/{id}", h.DeleteConnection)
	mux.HandleFunc("GET /api/connections/export", h.ExportConnections)
	mux.HandleFunc("POST /api/connections/import", h.ImportConnections)

	// SSH Key endpoints
	mux.HandleFunc("GET /api/keys", kh.ListKeys)
	mux.HandleFunc("GET /api/keys/{id}", kh.GetKey)
	mux.HandleFunc("POST /api/keys", kh.CreateKey)
	mux.HandleFunc("PUT /api/keys/{id}", kh.UpdateKey)
	mux.HandleFunc("DELETE /api/keys/{id}", kh.DeleteKey)

	// Port Forward endpoints
	mux.HandleFunc("GET /api/forwards", fh.ListForwards)
	mux.HandleFunc("POST /api/forwards", fh.CreateForward)
	mux.HandleFunc("DELETE /api/forwards/{id}", fh.DeleteForward)
	mux.HandleFunc("POST /api/forwards/{id}/start", fh.StartForward)
	mux.HandleFunc("POST /api/forwards/{id}/stop", fh.StopForward)

	// Settings endpoints
	mux.HandleFunc("GET /api/settings", sh.GetSettings)
	mux.HandleFunc("PUT /api/settings", sh.UpdateSettings)

	// SFTP endpoints
	mux.HandleFunc("GET /api/sftp/ls", sfh.List)
	mux.HandleFunc("GET /api/sftp/download", sfh.Download)
	mux.HandleFunc("POST /api/sftp/upload", sfh.Upload)
	mux.HandleFunc("DELETE /api/sftp/remove", sfh.Remove)
	mux.HandleFunc("POST /api/sftp/rename", sfh.Rename)
	mux.HandleFunc("POST /api/sftp/mkdir", sfh.Mkdir)
	mux.HandleFunc("POST /api/sftp/transfer", sfh.Transfer)
	mux.HandleFunc("GET /api/sftp/transfer/{id}/progress", sfh.Progress)

	// Session endpoints
	mux.HandleFunc("GET /api/sessions", ListSessions())
	mux.HandleFunc("DELETE /api/sessions/{id}", RemoveSession())

	// WebSocket handlers
	mux.HandleFunc("GET /ws", ssh.HandleWebSocket(database, cfg))
	mux.HandleFunc("GET /api/clipboard/ws", ssh.HandleClipboardWebSocket(database, cfg))

	// Static file serving for the frontend (Web mode)
	// Priority: fe/dist (local build) -> current directory (production bundle)
	distPath := "./fe/dist"
	if _, err := os.Stat(distPath); os.IsNotExist(err) {
		// Fallback to current directory if fe/dist doesn't exist
		distPath = "."
	}

	fs := http.FileServer(http.Dir(distPath))

	// Handle all other routes by serving static files (for SPA support, we should ideally serve index.html)
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		// If it's a request for a file that exists, serve it
		fpath := path.Join(distPath, r.URL.Path)
		if _, err := os.Stat(fpath); err == nil && !strings.HasSuffix(fpath, "/") {
			fs.ServeHTTP(w, r)
			return
		}
		// Otherwise serve index.html for SPA routing
		http.ServeFile(w, r, path.Join(distPath, "index.html"))
	})
}
