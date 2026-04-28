package api

import (
	"crypto/ecdsa"
	"crypto/ed25519"
	"crypto/rsa"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"webterm/internal/config"
	"webterm/internal/db"

	"golang.org/x/crypto/ssh"
	"gorm.io/gorm"
)

type SSHKeyHandler struct {
	DB  *gorm.DB
	Cfg *config.Config
}

type CreateKeyRequest struct {
	Name      string `json:"name"`
	KeyBase64 string `json:"key_base64"`
}

type UpdateKeyRequest struct {
	Name string `json:"name"`
}

func (h *SSHKeyHandler) ListKeys(w http.ResponseWriter, r *http.Request) {
	var keys []db.SSHKey
	if err := h.DB.Find(&keys).Error; err != nil {
		sendError(w, "Failed to list keys", http.StatusInternalServerError)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(keys)
}

func (h *SSHKeyHandler) GetKey(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	var key db.SSHKey
	if err := h.DB.First(&key, "id = ?", id).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Key not found", http.StatusNotFound)
		} else {
			sendError(w, "Failed to get key", http.StatusInternalServerError)
		}
		return
	}
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(key)
}

func (h *SSHKeyHandler) CreateKey(w http.ResponseWriter, r *http.Request) {
	var req CreateKeyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	if req.Name == "" {
		sendError(w, "Name is required", http.StatusBadRequest)
		return
	}

	if req.KeyBase64 == "" {
		sendError(w, "Key is required", http.StatusBadRequest)
		return
	}

	// Decode base64
	keyData, err := base64.StdEncoding.DecodeString(req.KeyBase64)
	if err != nil {
		sendError(w, "Invalid key encoding", http.StatusBadRequest)
		return
	}

	// Parse and validate the key
	privateKey, err := ssh.ParseRawPrivateKey(keyData)
	if err != nil {
		sendError(w, "Invalid key format", http.StatusBadRequest)
		return
	}

	// Determine key type
	var keyType string
	switch privateKey.(type) {
	case *rsa.PrivateKey:
		keyType = "RSA"
	case *ed25519.PrivateKey:
		keyType = "Ed25519"
	case *ecdsa.PrivateKey:
		keyType = "ECDSA"
	default:
		keyType = "Unknown"
	}

	// Extract public key for fingerprint
	sshPubKey, err := ssh.NewPublicKey(extractPublicKey(privateKey))
	if err != nil {
		sendError(w, "Failed to extract public key", http.StatusInternalServerError)
		return
	}
	fingerprint := ssh.FingerprintLegacyMD5(sshPubKey)

	// Encrypt the key with AAD
	encrypted, err := config.EncryptWithAAD(string(keyData), h.Cfg.EncryptionKey, []byte(config.SSHKeyAAD))
	if err != nil {
		sendError(w, "Failed to encrypt key", http.StatusInternalServerError)
		return
	}

	key := db.SSHKey{
		Name:          req.Name,
		EncryptedKey:  encrypted,
		Fingerprint:   fingerprint,
		KeyType:       keyType,
		HasPassphrase: false, // ssh.ParseRawPrivateKey only works with unencrypted keys or we need a passphrase
	}

	if err := h.DB.Create(&key).Error; err != nil {
		sendError(w, "Failed to create key", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusCreated)
	json.NewEncoder(w).Encode(key)
}

func (h *SSHKeyHandler) UpdateKey(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	var existing db.SSHKey
	if err := h.DB.First(&existing, "id = ?", id).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			sendError(w, "Key not found", http.StatusNotFound)
		} else {
			sendError(w, "Failed to get key", http.StatusInternalServerError)
		}
		return
	}

	var req UpdateKeyRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	if req.Name != "" {
		existing.Name = req.Name
	}

	if err := h.DB.Save(&existing).Error; err != nil {
		sendError(w, "Failed to update key", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(existing)
}

func (h *SSHKeyHandler) DeleteKey(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")

	// Check if key is referenced by any connections
	var count int64
	if err := h.DB.Model(&db.Connection{}).Where("ssh_key_id = ?", id).Count(&count).Error; err != nil {
		sendError(w, "Failed to check key references", http.StatusInternalServerError)
		return
	}

	// Delete the key
	if err := h.DB.Delete(&db.SSHKey{}, "id = ?", id).Error; err != nil {
		sendError(w, "Failed to delete key", http.StatusInternalServerError)
		return
	}

	// Return warning if key was referenced
	if count > 0 {
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]interface{}{
			"warning":              "Key was referenced by connections",
			"affected_connections": count,
		})
		return
	}

	w.WriteHeader(http.StatusNoContent)
}

func extractPublicKey(privateKey interface{}) interface{} {
	switch k := privateKey.(type) {
	case *rsa.PrivateKey:
		return &k.PublicKey
	case *ed25519.PrivateKey:
		return k.Public().(ed25519.PublicKey)
	case *ecdsa.PrivateKey:
		return &k.PublicKey
	default:
		return nil
	}
}
