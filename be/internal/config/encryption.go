package config

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
)

const (
	PasswordAAD = "webterm:password:v1"
	SSHKeyAAD   = "webterm:sshkey:v1"
)

// Encrypt encrypts plaintext using AES-256-GCM with the provided 32-byte key.
func Encrypt(plaintext string, key []byte) (string, error) {
	return EncryptWithAAD(plaintext, key, nil)
}

// Decrypt decrypts base64-encoded ciphertext using AES-256-GCM.
func Decrypt(encodedCiphertext string, key []byte) (string, error) {
	return DecryptWithAAD(encodedCiphertext, key, nil)
}

// EncryptWithAAD encrypts plaintext using AES-256-GCM with AAD and the provided 32-byte key.
func EncryptWithAAD(plaintext string, key []byte, aad []byte) (string, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return "", err
	}

	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", err
	}

	nonce := make([]byte, gcm.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return "", err
	}

	ciphertext := gcm.Seal(nonce, nonce, []byte(plaintext), aad)
	return base64.StdEncoding.EncodeToString(ciphertext), nil
}

// DecryptWithAAD decrypts base64-encoded ciphertext using AES-256-GCM with AAD.
func DecryptWithAAD(encodedCiphertext string, key []byte, aad []byte) (string, error) {
	ciphertext, err := base64.StdEncoding.DecodeString(encodedCiphertext)
	if err != nil {
		return "", err
	}

	block, err := aes.NewCipher(key)
	if err != nil {
		return "", err
	}

	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", err
	}

	nonceSize := gcm.NonceSize()
	if len(ciphertext) < nonceSize {
		return "", fmt.Errorf("ciphertext too short")
	}

	nonce, encryptedData := ciphertext[:nonceSize], ciphertext[nonceSize:]
	plaintext, err := gcm.Open(nil, nonce, encryptedData, aad)
	if err != nil {
		return "", err
	}

	return string(plaintext), nil
}
