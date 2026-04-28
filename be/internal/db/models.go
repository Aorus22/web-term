package db

import (
	"database/sql/driver"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"gorm.io/gorm"
)

type Connection struct {
	ID        string    `json:"id" gorm:"primaryKey;type:varchar(36)"`
	Label     string    `json:"label" gorm:"not null"`
	Host      string    `json:"host" gorm:"not null"`
	Port      int       `json:"port" gorm:"default:22"`
	Username  string    `json:"username" gorm:"not null"`
	Password  string    `json:"password,omitempty" gorm:"-"` // Omitted from GORM default if not handled
	Encrypted string    `json:"-" gorm:"column:password"`    // Stored in DB
	Tags      StringList `json:"tags" gorm:"type:text"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

func (c *Connection) BeforeCreate(tx *gorm.DB) (err error) {
	if c.ID == "" {
		c.ID = uuid.New().String()
	}
	return
}

type SSHKey struct {
	ID            string    `json:"id" gorm:"primaryKey;type:varchar(36)"`
	Name          string    `json:"name" gorm:"not null"`
	EncryptedKey  string    `json:"-" gorm:"column:encrypted_key;not null"`
	Fingerprint   string    `json:"fingerprint" gorm:"not null"`
	KeyType       string    `json:"key_type" gorm:"not null"`
	HasPassphrase bool      `json:"has_passphrase" gorm:"not null;default:false"`
	CreatedAt     time.Time `json:"created_at"`
	UpdatedAt     time.Time `json:"updated_at"`
}

func (k *SSHKey) BeforeCreate(tx *gorm.DB) error {
	if k.ID == "" {
		k.ID = uuid.New().String()
	}
	return nil
}

type StringList []string
func (sl StringList) Value() (driver.Value, error) {
	return json.Marshal(sl)
}

func (sl *StringList) Scan(value interface{}) error {
	bytes, ok := value.([]byte)
	if !ok {
		return errors.New("type assertion to []byte failed")
	}
	return json.Unmarshal(bytes, sl)
}
