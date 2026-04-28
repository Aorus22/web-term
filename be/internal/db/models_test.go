package db

import (
	"encoding/json"
	"testing"

	"github.com/google/uuid"
)

func TestSSHKeyModel(t *testing.T) {
	t.Run("BeforeCreate assigns UUID", func(t *testing.T) {
		key := &SSHKey{Name: "test"}
		err := key.BeforeCreate(nil)
		if err != nil {
			t.Fatalf("BeforeCreate failed: %v", err)
		}
		if _, err := uuid.Parse(key.ID); err != nil {
			t.Errorf("ID is not a valid UUID: %v", err)
		}
	})

	t.Run("JSON marshaling omits EncryptedKey", func(t *testing.T) {
		key := SSHKey{
			ID:           "test-id",
			Name:         "test-key",
			EncryptedKey: "secret-material",
		}
		data, err := json.Marshal(key)
		if err != nil {
			t.Fatalf("Marshal failed: %v", err)
		}
		
		var m map[string]interface{}
		json.Unmarshal(data, &m)
		
		if _, ok := m["encrypted_key"]; ok {
			t.Error("encrypted_key should be omitted from JSON")
		}
		if m["name"] != "test-key" {
			t.Errorf("expected name 'test-key', got %v", m["name"])
		}
	})
}
