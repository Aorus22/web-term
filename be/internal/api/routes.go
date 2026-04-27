package api

import (
	"net/http"
	"webterm/internal/config"
	"webterm/internal/ssh"

	"gorm.io/gorm"
)

func SetupRoutes(mux *http.ServeMux, db *gorm.DB, cfg *config.Config) {
	h := &ConnectionHandler{
		DB:  db,
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

	// WebSocket handler
	mux.HandleFunc("GET /ws", ssh.HandleWebSocket(db, cfg))
}
