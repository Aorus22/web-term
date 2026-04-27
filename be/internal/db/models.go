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
