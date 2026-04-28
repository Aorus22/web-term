package api

import (
	"encoding/json"
	"net/http"
	"webterm/internal/config"
	"webterm/internal/db"
	"webterm/internal/ssh"

	"gorm.io/gorm"
)

type ForwardHandler struct {
	DB        *gorm.DB
	Cfg       *config.Config
	TunnelMgr *ssh.TunnelManager
}

// forwardResponse is the JSON shape returned for each port forward.
type forwardResponse struct {
	ID           string `json:"id"`
	Name         string `json:"name"`
	ConnectionID string `json:"connection_id"`
	LocalPort    int    `json:"local_port"`
	RemotePort   int    `json:"remote_port"`
	Active       bool   `json:"active"`
	Error        string `json:"error"`
	CreatedAt    string `json:"created_at"`
	UpdatedAt    string `json:"updated_at"`
}

func (h *ForwardHandler) ListForwards(w http.ResponseWriter, r *http.Request) {
	var forwards []db.PortForward
	if err := h.DB.Find(&forwards).Error; err != nil {
		sendError(w, "Failed to list forwards", http.StatusInternalServerError)
		return
	}

	response := make([]forwardResponse, 0, len(forwards))
	for _, f := range forwards {
		response = append(response, forwardResponse{
			ID:           f.ID,
			Name:         f.Name,
			ConnectionID: f.ConnectionID,
			LocalPort:    f.LocalPort,
			RemotePort:   f.RemotePort,
			Active:       h.TunnelMgr.IsActive(f.ID),
			Error:        h.TunnelMgr.GetError(f.ID),
			CreatedAt:    f.CreatedAt.Format("2006-01-02T15:04:05Z07:00"),
			UpdatedAt:    f.UpdatedAt.Format("2006-01-02T15:04:05Z07:00"),
		})
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(response)
}

func (h *ForwardHandler) CreateForward(w http.ResponseWriter, r *http.Request) {
	var forward db.PortForward
	if err := json.NewDecoder(r.Body).Decode(&forward); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	// Validate required fields (T-08-01)
	if forward.Name == "" {
		sendError(w, "Name is required", http.StatusBadRequest)
		return
	}
	if forward.ConnectionID == "" {
		sendError(w, "connection_id is required", http.StatusBadRequest)
		return
	}
	if forward.LocalPort < 1 || forward.LocalPort > 65535 {
		sendError(w, "local_port must be between 1 and 65535", http.StatusBadRequest)
		return
	}
	if forward.RemotePort < 1 || forward.RemotePort > 65535 {
		sendError(w, "remote_port must be between 1 and 65535", http.StatusBadRequest)
		return
	}

	// Validate connection_id exists in DB
	var conn db.Connection
	if err := h.DB.First(&conn, "id = ?", forward.ConnectionID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Connection not found", http.StatusBadRequest)
		} else {
			sendError(w, "Failed to validate connection", http.StatusInternalServerError)
		}
		return
	}

	if err := h.DB.Create(&forward).Error; err != nil {
		sendError(w, "Failed to create forward", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(forwardResponse{
		ID:           forward.ID,
		Name:         forward.Name,
		ConnectionID: forward.ConnectionID,
		LocalPort:    forward.LocalPort,
		RemotePort:   forward.RemotePort,
		Active:       false,
		Error:        "",
		CreatedAt:    forward.CreatedAt.Format("2006-01-02T15:04:05Z07:00"),
		UpdatedAt:    forward.UpdatedAt.Format("2006-01-02T15:04:05Z07:00"),
	})
}

func (h *ForwardHandler) DeleteForward(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	// Stop active tunnel if running
	if h.TunnelMgr.IsActive(id) {
		h.TunnelMgr.Stop(id)
	}

	if err := h.DB.Delete(&db.PortForward{}, "id = ?", id).Error; err != nil {
		sendError(w, "Failed to delete forward", http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func (h *ForwardHandler) StartForward(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	// Get forward rule
	var forward db.PortForward
	if err := h.DB.First(&forward, "id = ?", id).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Forward not found", http.StatusNotFound)
		} else {
			sendError(w, "Failed to get forward", http.StatusInternalServerError)
		}
		return
	}

	// Get connection for credentials
	var conn db.Connection
	if err := h.DB.First(&conn, "id = ?", forward.ConnectionID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Associated connection not found", http.StatusBadRequest)
		} else {
			sendError(w, "Failed to get connection", http.StatusInternalServerError)
		}
		return
	}

	// Start the tunnel
	if err := h.TunnelMgr.Start(id, conn, forward.LocalPort, forward.RemotePort); err != nil {
		sendError(w, err.Error(), http.StatusConflict)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status": "active",
	})
}

func (h *ForwardHandler) StopForward(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	h.TunnelMgr.Stop(id)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status": "inactive",
	})
}
