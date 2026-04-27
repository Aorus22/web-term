package api

import (
	"encoding/json"
	"fmt"
	"net/http"
	"webterm/internal/db"
)

func (h *ConnectionHandler) ExportConnections(w http.ResponseWriter, r *http.Request) {
	var connections []db.Connection
	if err := h.DB.Find(&connections).Error; err != nil {
		sendError(w, "Failed to export connections", http.StatusInternalServerError)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Content-Disposition", `attachment; filename="webterm-connections.json"`)
	json.NewEncoder(w).Encode(connections)
}

func (h *ConnectionHandler) ImportConnections(w http.ResponseWriter, r *http.Request) {
	var imports []db.Connection
	if err := json.NewDecoder(r.Body).Decode(&imports); err != nil {
		sendError(w, "Invalid import format", http.StatusBadRequest)
		return
	}

	imported := 0
	skipped := 0

	for _, conn := range imports {
		var existing db.Connection
		err := h.DB.Where("host = ? AND port = ? AND username = ?", conn.Host, conn.Port, conn.Username).First(&existing).Error
		if err == nil {
			skipped++
			continue
		}

		// Reset ID to let GORM/BeforeCreate generate a new one
		conn.ID = ""
		if err := h.DB.Create(&conn).Error; err != nil {
			fmt.Printf("Error importing connection %s: %v\n", conn.Label, err)
			skipped++
		} else {
			imported++
		}
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(map[string]int{
		"imported": imported,
		"skipped":  skipped,
	})
}
