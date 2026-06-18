package api

import (
	"encoding/json"
	"errors"
	"net/http"
	"strings"
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
	Type         string `json:"type"`
	Active       bool   `json:"active"`
	AutoStart    bool   `json:"auto_start"`
	Error        string `json:"error"`
	CreatedAt    string `json:"created_at"`
	UpdatedAt    string `json:"updated_at"`
}

// normaliseForwardType applies the default for the Type field.
// Empty (or whitespace) is treated as "local" for backwards compatibility.
func normaliseForwardType(t string) (string, bool) {
	t = strings.TrimSpace(t)
	if t == "" {
		return ssh.ForwardTypeLocal, true
	}
	if t != ssh.ForwardTypeLocal && t != ssh.ForwardTypeReverse {
		return "", false
	}
	return t, true
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
			Type:         f.Type,
			Active:       h.TunnelMgr.IsActive(f.ID),
			AutoStart:    f.AutoStart,
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

	if err := validateForwardFields(h.DB, &forward); err != nil {
		sendError(w, err.Error(), http.StatusBadRequest)
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
		Type:         forward.Type,
		Active:       false,
		AutoStart:    false,
		Error:        "",
		CreatedAt:    forward.CreatedAt.Format("2006-01-02T15:04:05Z07:00"),
		UpdatedAt:    forward.UpdatedAt.Format("2006-01-02T15:04:05Z07:00"),
	})
}

// UpdateForward updates an existing port forward rule. The PUT /api/forwards/{id}
// route was previously unregistered; the FE has been calling it.
func (h *ForwardHandler) UpdateForward(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	var existing db.PortForward
	if err := h.DB.First(&existing, "id = ?", id).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Forward not found", http.StatusNotFound)
		} else {
			sendError(w, "Failed to get forward", http.StatusInternalServerError)
		}
		return
	}

	var payload db.PortForward
	if err := json.NewDecoder(r.Body).Decode(&payload); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	existing.Name = payload.Name
	existing.ConnectionID = payload.ConnectionID
	existing.LocalPort = payload.LocalPort
	existing.RemotePort = payload.RemotePort
	existing.Type = payload.Type

	if err := validateForwardFields(h.DB, &existing); err != nil {
		sendError(w, err.Error(), http.StatusBadRequest)
		return
	}

	// Persist; auto_start is intentionally not editable through update.
	if err := h.DB.Model(&db.PortForward{}).Where("id = ?", id).Updates(map[string]interface{}{
		"name":          existing.Name,
		"connection_id": existing.ConnectionID,
		"local_port":    existing.LocalPort,
		"remote_port":   existing.RemotePort,
		"type":          existing.Type,
	}).Error; err != nil {
		sendError(w, "Failed to update forward", http.StatusInternalServerError)
		return
	}

	var updated db.PortForward
	if err := h.DB.First(&updated, "id = ?", id).Error; err != nil {
		sendError(w, "Failed to reload forward", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(forwardResponse{
		ID:           updated.ID,
		Name:         updated.Name,
		ConnectionID: updated.ConnectionID,
		LocalPort:    updated.LocalPort,
		RemotePort:   updated.RemotePort,
		Type:         updated.Type,
		Active:       h.TunnelMgr.IsActive(updated.ID),
		AutoStart:    updated.AutoStart,
		Error:        h.TunnelMgr.GetError(updated.ID),
		CreatedAt:    updated.CreatedAt.Format("2006-01-02T15:04:05Z07:00"),
		UpdatedAt:    updated.UpdatedAt.Format("2006-01-02T15:04:05Z07:00"),
	})
}

// validateForwardFields runs the shared field-level checks used by both
// CreateForward and UpdateForward. It normalises Type to its default and
// verifies connection_id references an existing row.
func validateForwardFields(database *gorm.DB, f *db.PortForward) error {
	if f.Name == "" {
		return errors.New("Name is required")
	}
	if f.ConnectionID == "" {
		return errors.New("connection_id is required")
	}
	if f.LocalPort < 1 || f.LocalPort > 65535 {
		return errors.New("local_port must be between 1 and 65535")
	}
	if f.RemotePort < 1 || f.RemotePort > 65535 {
		return errors.New("remote_port must be between 1 and 65535")
	}
	normalised, ok := normaliseForwardType(f.Type)
	if !ok {
		return errors.New("type must be 'local' or 'reverse'")
	}
	f.Type = normalised

	var conn db.Connection
	if err := database.First(&conn, "id = ?", f.ConnectionID).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			return errors.New("Connection not found")
		}
		return errors.New("Failed to validate connection")
	}
	return nil
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
	if err := h.TunnelMgr.Start(id, forward.Type, conn, forward.LocalPort, forward.RemotePort); err != nil {
		sendError(w, err.Error(), http.StatusConflict)
		return
	}

	// Persist auto_start so it restarts on next startup
	h.DB.Model(&forward).Update("auto_start", true)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status": "active",
	})
}

func (h *ForwardHandler) StopForward(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	h.TunnelMgr.Stop(id)

	// Persist auto_start so it doesn't restart on next startup
	h.DB.Model(&db.PortForward{}).Where("id = ?", id).Update("auto_start", false)

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]string{
		"status": "inactive",
	})
}
