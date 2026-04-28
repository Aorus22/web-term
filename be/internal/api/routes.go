package api

import (
	"net/http"
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

	fh := &ForwardHandler{
		DB:        database,
		Cfg:       cfg,
		TunnelMgr: tunnelMgr,
	}

	sh := &SettingsHandler{
		DB:  database,
		Cfg: cfg,
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

	// WebSocket handler
	mux.HandleFunc("GET /ws", ssh.HandleWebSocket(database, cfg))
}
