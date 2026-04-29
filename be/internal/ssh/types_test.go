package ssh

import (
	"encoding/json"
	"testing"
)

func TestConnectMessageHasCwdField(t *testing.T) {
	// Test that ConnectMessage has Cwd field with json tag "cwd,omitempty"
	raw := `{"type":"connect","host":"example.com","port":22,"user":"test","password":"pass","cwd":"/tmp/mydir"}`
	var msg ConnectMessage
	if err := json.Unmarshal([]byte(raw), &msg); err != nil {
		t.Fatalf("failed to unmarshal ConnectMessage with cwd: %v", err)
	}
	if msg.Cwd != "/tmp/mydir" {
		t.Errorf("expected Cwd=%q, got %q", "/tmp/mydir", msg.Cwd)
	}
}

func TestConnectMessageCwdOmitEmpty(t *testing.T) {
	msg := ConnectMessage{
		Type: "connect",
		Host: "example.com",
		Port: 22,
		User: "test",
	}
	data, err := json.Marshal(msg)
	if err != nil {
		t.Fatalf("failed to marshal ConnectMessage: %v", err)
	}
	var m map[string]interface{}
	json.Unmarshal(data, &m)
	if _, ok := m["cwd"]; ok {
		t.Error("cwd should be omitted when empty (omitempty)")
	}
}

func TestCwdResponseMessage(t *testing.T) {
	// Test that CwdResponseMessage struct exists and serializes correctly
	resp := CwdResponseMessage{
		Type: "cwd",
		Path: "/home/user/projects",
	}
	data, err := json.Marshal(resp)
	if err != nil {
		t.Fatalf("failed to marshal CwdResponseMessage: %v", err)
	}

	var m map[string]interface{}
	json.Unmarshal(data, &m)
	if m["type"] != "cwd" {
		t.Errorf("expected type=%q, got %q", "cwd", m["type"])
	}
	if m["path"] != "/home/user/projects" {
		t.Errorf("expected path=%q, got %q", "/home/user/projects", m["path"])
	}
}

func TestConnectMessageCwdEmptyByDefault(t *testing.T) {
	msg := ConnectMessage{}
	if msg.Cwd != "" {
		t.Errorf("expected empty Cwd by default, got %q", msg.Cwd)
	}
}
