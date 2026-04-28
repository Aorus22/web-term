package api

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/pem"
	"encoding/base64"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"webterm/internal/config"
	"webterm/internal/db"

	"golang.org/x/crypto/ssh"
	"gorm.io/driver/sqlite"
	"gorm.io/gorm"
)

func setupTestDB(t *testing.T) *gorm.DB {
	database, err := gorm.Open(sqlite.Open(":memory:"), &gorm.Config{})
	if err != nil {
		t.Fatalf("failed to connect database: %v", err)
	}
	database.AutoMigrate(&db.SSHKey{})
	return database
}

func TestSSHKeyHandlers(t *testing.T) {
	database := setupTestDB(t)
	cfg := &config.Config{
		EncryptionKey: make([]byte, 32),
	}
	handler := &SSHKeyHandler{
		DB:  database,
		Cfg: cfg,
	}

	// Generate a test Ed25519 key
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		t.Fatalf("failed to generate key: %v", err)
	}
	
	privBytes, err := ssh.MarshalPrivateKey(priv, "")
	if err != nil {
		t.Fatalf("failed to marshal private key: %v", err)
	}
	
	privPEM := pem.EncodeToMemory(privBytes)
	privBase64 := base64.StdEncoding.EncodeToString(privPEM)

	t.Run("CreateKey", func(t *testing.T) {
		reqBody, _ := json.Marshal(CreateKeyRequest{
			Name:      "test-key",
			KeyBase64: privBase64,
		})
		req := httptest.NewRequest("POST", "/api/keys", bytes.NewBuffer(reqBody))
		rr := httptest.NewRecorder()

		handler.CreateKey(rr, req)

		if rr.Code != http.StatusCreated {
			t.Errorf("expected status 201, got %v. Body: %s", rr.Code, rr.Body.String())
		}

		var created db.SSHKey
		json.Unmarshal(rr.Body.Bytes(), &created)
		if created.Name != "test-key" {
			t.Errorf("expected name 'test-key', got %v", created.Name)
		}
		if created.KeyType != "Ed25519" {
			t.Errorf("expected key type 'Ed25519', got %v", created.KeyType)
		}
		
		// Verify fingerprint
		sshPub, _ := ssh.NewPublicKey(pub)
		expectedFingerprint := ssh.FingerprintLegacyMD5(sshPub)
		if created.Fingerprint != expectedFingerprint {
			t.Errorf("expected fingerprint %v, got %v", expectedFingerprint, created.Fingerprint)
		}
	})

	t.Run("ListKeys", func(t *testing.T) {
		req := httptest.NewRequest("GET", "/api/keys", nil)
		rr := httptest.NewRecorder()

		handler.ListKeys(rr, req)

		if rr.Code != http.StatusOK {
			t.Errorf("expected status 200, got %v", rr.Code)
		}

		var keys []db.SSHKey
		json.Unmarshal(rr.Body.Bytes(), &keys)
		if len(keys) != 1 {
			t.Errorf("expected 1 key, got %v", len(keys))
		}
	})

	t.Run("CreateKey invalid PEM", func(t *testing.T) {
		reqBody, _ := json.Marshal(CreateKeyRequest{
			Name:      "bad-key",
			KeyBase64: base64.StdEncoding.EncodeToString([]byte("not a pem")),
		})
		req := httptest.NewRequest("POST", "/api/keys", bytes.NewBuffer(reqBody))
		rr := httptest.NewRecorder()

		handler.CreateKey(rr, req)

		if rr.Code != http.StatusBadRequest {
			t.Errorf("expected status 400, got %v", rr.Code)
		}
	})
}
