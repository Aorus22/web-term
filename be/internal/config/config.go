package config

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"os"
	"path"
	"strings"

	"github.com/joho/godotenv"
)

type Config struct {
	Port            string
	DBPath          string
	EncryptionKey   []byte
	SSRFAllowlist   []string
	AllowedOrigins  []string
}

func Load() (*Config, error) {
	godotenv.Load()

	cfg := &Config{
		Port:           getEnv("WEBTERM_PORT", ":8080"),
		DBPath:         getEnv("WEBTERM_DB_PATH", "data/webterm.db"),
		SSRFAllowlist:  splitEnv("WEBTERM_SSRF_ALLOWLIST", ""),
		AllowedOrigins: splitEnv("WEBTERM_ALLOWED_ORIGINS", "http://localhost:5173,http://localhost:8080"),
	}

	keyHex := os.Getenv("WEBTERM_ENCRYPTION_KEY")
	if keyHex == "" {
		key := make([]byte, 32)
		if _, err := rand.Read(key); err != nil {
			return nil, fmt.Errorf("failed to generate random encryption key: %v", err)
		}
		keyHex = hex.EncodeToString(key)
		fmt.Printf("\n⚠️  WEBTERM_ENCRYPTION_KEY not set. Generating a random one for this session:\n")
		fmt.Printf("   export WEBTERM_ENCRYPTION_KEY=%s\n", keyHex)
		fmt.Printf("   (Store this key to keep existing passwords decryptable!)\n\n")
		cfg.EncryptionKey = key
	} else {
		key, err := hex.DecodeString(keyHex)
		if err != nil || len(key) != 32 {
			return nil, fmt.Errorf("WEBTERM_ENCRYPTION_KEY must be a 32-byte hex string")
		}
		cfg.EncryptionKey = key
	}

	return cfg, nil
}

func (c *Config) IsHostAllowed(host string) bool {
	if host == "169.254.169.4" {
		return false
	}
	if len(c.SSRFAllowlist) == 0 {
		return true
	}
	for _, pattern := range c.SSRFAllowlist {
		if matched, _ := path.Match(pattern, host); matched {
			return true
		}
	}
	return false
}

func getEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}

func splitEnv(key, fallback string) []string {
	value := getEnv(key, fallback)
	if value == "" {
		return []string{}
	}
	parts := strings.Split(value, ",")
	var result []string
	for _, p := range parts {
		p = strings.TrimSpace(p)
		if p != "" {
			result = append(result, p)
		}
	}
	return result
}
