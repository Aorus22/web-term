package config

import (
	"testing"
)

func TestEncryptionWithAAD(t *testing.T) {
	key := make([]byte, 32)
	for i := range key {
		key[i] = byte(i)
	}

	plaintext := "hello world"
	aad := []byte("test-aad")

	t.Run("EncryptWithAAD and DecryptWithAAD", func(t *testing.T) {
		ciphertext, err := EncryptWithAAD(plaintext, key, aad)
		if err != nil {
			t.Fatalf("EncryptWithAAD failed: %v", err)
		}

		decrypted, err := DecryptWithAAD(ciphertext, key, aad)
		if err != nil {
			t.Fatalf("DecryptWithAAD failed: %v", err)
		}

		if decrypted != plaintext {
			t.Errorf("expected %q, got %q", plaintext, decrypted)
		}
	})

	t.Run("DecryptWithAAD fails with wrong AAD", func(t *testing.T) {
		ciphertext, err := EncryptWithAAD(plaintext, key, aad)
		if err != nil {
			t.Fatalf("EncryptWithAAD failed: %v", err)
		}

		_, err = DecryptWithAAD(ciphertext, key, []byte("wrong-aad"))
		if err == nil {
			t.Error("expected error with wrong AAD, got nil")
		}
	})

	t.Run("EncryptWithAAD differs from Encrypt (nil AAD)", func(t *testing.T) {
		ciphertextAAD, err := EncryptWithAAD(plaintext, key, aad)
		if err != nil {
			t.Fatalf("EncryptWithAAD failed: %v", err)
		}

		ciphertextNoAAD, err := Encrypt(plaintext, key)
		if err != nil {
			t.Fatalf("Encrypt failed: %v", err)
		}

		// They should differ because Encrypt uses nil AAD and different nonce (usually)
		// but even if nonce was same, they would differ.
		if ciphertextAAD == ciphertextNoAAD {
			t.Error("expected different ciphertexts for different AAD")
		}
	})

	t.Run("Backward compatibility with Encrypt/Decrypt", func(t *testing.T) {
		ciphertext, err := Encrypt(plaintext, key)
		if err != nil {
			t.Fatalf("Encrypt failed: %v", err)
		}

		decrypted, err := Decrypt(ciphertext, key)
		if err != nil {
			t.Fatalf("Decrypt failed: %v", err)
		}

		if decrypted != plaintext {
			t.Errorf("expected %q, got %q", plaintext, decrypted)
		}
	})
}
