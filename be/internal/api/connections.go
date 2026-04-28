package api

import (
	"encoding/json"
	"net/http"
	"webterm/internal/config"
	"webterm/internal/db"

	"gorm.io/gorm"
)

type ConnectionHandler struct {
	DB  *gorm.DB
	Cfg *config.Config
}

func (h *ConnectionHandler) ListConnections(w http.ResponseWriter, r *http.Request) {
	var connections []db.Connection
	if err := h.DB.Find(&connections).Error; err != nil {
		sendError(w, "Failed to list connections", http.StatusInternalServerError)
		return
	}

	// Omit passwords in list
	for i := range connections {
		connections[i].Password = ""
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(connections)
}

func (h *ConnectionHandler) GetConnection(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	var conn db.Connection
	if err := h.DB.First(&conn, "id = ?", id).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Connection not found", http.StatusNotFound)
		} else {
			sendError(w, "Failed to get connection", http.StatusInternalServerError)
		}
		return
	}

	// Decrypt password for single view (edit form)
	if conn.Encrypted != "" {
		decrypted, err := config.Decrypt(conn.Encrypted, h.Cfg.EncryptionKey)
		if err == nil {
			conn.Password = decrypted
		}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(conn)
}

func (h *ConnectionHandler) CreateConnection(w http.ResponseWriter, r *http.Request) {
	var conn db.Connection
	if err := json.NewDecoder(r.Body).Decode(&conn); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	if conn.Label == "" || conn.Host == "" || conn.Username == "" {
		sendError(w, "Label, host, and username are required", http.StatusBadRequest)
		return
	}

	if conn.Port <= 0 || conn.Port > 65535 {
		conn.Port = 22
	}

	// Validate auth_method
	if conn.AuthMethod == "" {
		conn.AuthMethod = "password" // backward compatibility
	}
	if conn.AuthMethod != "password" && conn.AuthMethod != "key" {
		sendError(w, "auth_method must be 'password' or 'key'", http.StatusBadRequest)
		return
	}

	// Validate ssh_key_id when auth_method is key
	if conn.AuthMethod == "key" {
		if conn.SSHKeyID == nil || *conn.SSHKeyID == "" {
			sendError(w, "ssh_key_id is required when auth_method is 'key'", http.StatusBadRequest)
			return
		}
		// Verify key exists
		var key db.SSHKey
		if err := h.DB.First(&key, "id = ?", *conn.SSHKeyID).Error; err != nil {
			if err == gorm.ErrRecordNotFound {
				sendError(w, "SSH key not found", http.StatusBadRequest)
			} else {
				sendError(w, "Failed to validate SSH key", http.StatusInternalServerError)
			}
			return
		}
	}

	if conn.Password != "" {
		encrypted, err := config.Encrypt(conn.Password, h.Cfg.EncryptionKey)
		if err != nil {
			sendError(w, "Failed to encrypt password", http.StatusInternalServerError)
			return
		}
		conn.Encrypted = encrypted
	}

	if err := h.DB.Create(&conn).Error; err != nil {
		sendError(w, "Failed to create connection", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(conn)
}

func (h *ConnectionHandler) UpdateConnection(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	var existing db.Connection
	if err := h.DB.First(&existing, "id = ?", id).Error; err != nil {
		sendError(w, "Connection not found", http.StatusNotFound)
		return
	}

	var update db.Connection
	if err := json.NewDecoder(r.Body).Decode(&update); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	existing.Label = update.Label
	existing.Host = update.Host
	existing.Username = update.Username
	existing.Tags = update.Tags
	existing.AuthMethod = update.AuthMethod
	if update.SSHKeyID != nil {
		existing.SSHKeyID = update.SSHKeyID
	}

	// Validate auth configuration
	if existing.AuthMethod == "key" && (existing.SSHKeyID == nil || *existing.SSHKeyID == "") {
		sendError(w, "ssh_key_id is required when auth_method is 'key'", http.StatusBadRequest)
		return
	}

	// Verify key exists if switching to key auth
	if existing.AuthMethod == "key" && existing.SSHKeyID != nil {
		var key db.SSHKey
		if err := h.DB.First(&key, "id = ?", *existing.SSHKeyID).Error; err != nil {
			if err == gorm.ErrRecordNotFound {
				sendError(w, "SSH key not found", http.StatusBadRequest)
			} else {
				sendError(w, "Failed to validate SSH key", http.StatusInternalServerError)
			}
			return
		}
	}

	if update.Port > 0 && update.Port <= 65535 {
		existing.Port = update.Port
	} else if update.Port == 0 {
		existing.Port = 22
	}

	if update.Password != "" {
		encrypted, err := config.Encrypt(update.Password, h.Cfg.EncryptionKey)
		if err != nil {
			sendError(w, "Failed to encrypt password", http.StatusInternalServerError)
			return
		}
		existing.Encrypted = encrypted
	}

	if err := h.DB.Save(&existing).Error; err != nil {
		sendError(w, "Failed to update connection", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(existing)
}

func (h *ConnectionHandler) DeleteConnection(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	if err := h.DB.Delete(&db.Connection{}, "id = ?", id).Error; err != nil {
		sendError(w, "Failed to delete connection", http.StatusInternalServerError)
		return
	}
	w.WriteHeader(http.StatusNoContent)
}

func sendError(w http.ResponseWriter, message string, status int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(map[string]string{"error": message})
}
