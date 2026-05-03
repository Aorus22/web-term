package api

import (
	"encoding/json"
	"net/http"
	"webterm/internal/ssh"
)

type SessionInfo struct {
	ID           string          `json:"id"`
	Type         ssh.SessionType `json:"type"`
	Host         string          `json:"host"`
	User         string          `json:"user"`
	Port         int             `json:"port"`
	ConnectionID string          `json:"connection_id,omitempty"`
	Status       string          `json:"status"` // "active" | "detached"
	Cwd          string          `json:"cwd,omitempty"`
}

func ListSessions() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		managedSessions := ssh.GlobalSessionManager.ListSessions()
		
		sessions := make([]SessionInfo, 0, len(managedSessions))
		for _, s := range managedSessions {
			status := "detached"
			if s.IsActive() {
				status = "active"
			}

			sessions = append(sessions, SessionInfo{
				ID:           s.ID,
				Type:         s.Type,
				Host:         s.Host,
				User:         s.User,
				Port:         s.Port,
				ConnectionID: s.ConnectionID,
				Status:       status,
				Cwd:          s.GetCwd(),
			})
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(sessions)
	}
}

func RemoveSession() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		id := r.PathValue("id")
		if id == "" {
			http.Error(w, "Missing session ID", http.StatusBadRequest)
			return
		}

		ssh.GlobalSessionManager.RemoveSession(id)
		w.WriteHeader(http.StatusNoContent)
	}
}
