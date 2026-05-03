package ssh

import (
	"fmt"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"

	"github.com/pkg/sftp"
	"golang.org/x/crypto/ssh"
	"gorm.io/gorm"
)

// DialSSH establishes an SSH connection using the provided database connection record
func DialSSH(database *gorm.DB, dbConn db.Connection, cfg *config.Config, passphrase string) (*ssh.Client, error) {
	var keySigner ssh.Signer
	var password string

	if dbConn.AuthMethod == "key" && dbConn.SSHKeyID != nil {
		var key db.SSHKey
		if err := database.First(&key, "id = ?", *dbConn.SSHKeyID).Error; err != nil {
			return nil, fmt.Errorf("SSH key not found: %w", err)
		}

		decryptedKey, err := config.DecryptWithAAD(key.EncryptedKey, cfg.EncryptionKey, []byte(config.SSHKeyAAD))
		if err != nil {
			return nil, fmt.Errorf("failed to decrypt SSH key: %w", err)
		}

		var parsedKey interface{}
		var parseErr error
		if passphrase != "" {
			parsedKey, parseErr = ssh.ParseRawPrivateKeyWithPassphrase([]byte(decryptedKey), []byte(passphrase))
		} else {
			parsedKey, parseErr = ssh.ParseRawPrivateKey([]byte(decryptedKey))
		}

		if parseErr != nil {
			return nil, fmt.Errorf("failed to parse SSH key: %w", parseErr)
		}

		keySigner, err = ssh.NewSignerFromKey(parsedKey)
		if err != nil {
			return nil, fmt.Errorf("failed to create SSH signer: %w", err)
		}
	} else if dbConn.Encrypted != "" {
		decrypted, err := config.Decrypt(dbConn.Encrypted, cfg.EncryptionKey)
		if err != nil {
			return nil, fmt.Errorf("failed to decrypt credentials: %w", err)
		}
		password = decrypted
	}

	// Security Fix CR-03: SSRF Protection
	if !cfg.IsHostAllowed(dbConn.Host) {
		return nil, fmt.Errorf("host not authorized by security policy")
	}

	port := dbConn.Port
	if port <= 0 || port > 65535 {
		port = 22
	}

	addr := fmt.Sprintf("%s:%d", dbConn.Host, port)
	sshConfig := &ssh.ClientConfig{
		User:            dbConn.Username,
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         10 * time.Second,
	}

	if keySigner != nil {
		sshConfig.Auth = []ssh.AuthMethod{ssh.PublicKeys(keySigner)}
	} else {
		sshConfig.Auth = []ssh.AuthMethod{
			ssh.Password(password),
			ssh.KeyboardInteractive(func(user, instruction string, questions []string, echos []bool) ([]string, error) {
				answers := make([]string, len(questions))
				for i := range answers {
					answers[i] = password
				}
				return answers, nil
			}),
		}
	}

	return ssh.Dial("tcp", addr, sshConfig)
}

// ConnectSFTP establishes an SFTP session using the provided database connection record
// It returns both the sftp.Client and the underlying ssh.Client so they can both be closed.
func ConnectSFTP(database *gorm.DB, dbConn db.Connection, cfg *config.Config, passphrase string) (*sftp.Client, *ssh.Client, error) {
	client, err := DialSSH(database, dbConn, cfg, passphrase)
	if err != nil {
		return nil, nil, err
	}
	sftpClient, err := sftp.NewClient(client)
	if err != nil {
		client.Close()
		return nil, nil, err
	}
	return sftpClient, client, nil
}
