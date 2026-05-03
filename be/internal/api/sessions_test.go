package api

import (
	"net/http"
	"net/http/httptest"
	"testing"
	"webterm/internal/ssh"
)

func TestRemoveSession(t *testing.T) {
	// Create a dummy session
	s := ssh.GlobalSessionManager.CreateSession(ssh.SessionTypeLocal, nil, nil, nil, nil, "local", "local", 0, "local")
	sessionID := s.ID

	// Verify session exists
	_, ok := ssh.GlobalSessionManager.GetSession(sessionID)
	if !ok {
		t.Fatalf("Session should exist")
	}

	// Create request
	req := httptest.NewRequest("DELETE", "/api/sessions/"+sessionID, nil)
	// Mock PathValue (Go 1.22+)
	req.SetPathValue("id", sessionID)
	
	w := httptest.NewRecorder()

	// Call handler
	handler := RemoveSession()
	handler.ServeHTTP(w, req)

	// Check response
	if w.Code != http.StatusNoContent {
		t.Errorf("Expected status 204, got %d", w.Code)
	}

	// Verify session is gone
	_, ok = ssh.GlobalSessionManager.GetSession(sessionID)
	if ok {
		t.Errorf("Session should have been removed")
	}
}
