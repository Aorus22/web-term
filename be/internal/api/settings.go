package api

import (
	"encoding/json"
	"net/http"
	"webterm/internal/config"
	"webterm/internal/db"

	"gorm.io/gorm"
)

type SettingsHandler struct {
	DB  *gorm.DB
	Cfg *config.Config
}

var defaultSettings = map[string]string{
	"theme_mode":           "system",
	"terminal_color_theme": "default",
	"terminal_type":        "xterm-256color",
	"font_family":          "Geist Mono",
	"font_size":            "14",
	"cursor_style":         "block",
	"cursor_blink":         "true",
	"scrollback":           "1000",
}

type SettingsResponse struct {
	Settings map[string]string `json:"settings"`
}

type UpdateSettingsRequest struct {
	Settings map[string]string `json:"settings"`
}

func (h *SettingsHandler) GetSettings(w http.ResponseWriter, r *http.Request) {
	var dbSettings []db.Setting
	if err := h.DB.Find(&dbSettings).Error; err != nil {
		sendError(w, "Failed to fetch settings", http.StatusInternalServerError)
		return
	}

	settingsMap := make(map[string]string)
	for k, v := range defaultSettings {
		settingsMap[k] = v
	}

	for _, s := range dbSettings {
		settingsMap[s.Key] = s.Value
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(SettingsResponse{Settings: settingsMap})
}

func (h *SettingsHandler) UpdateSettings(w http.ResponseWriter, r *http.Request) {
	var req UpdateSettingsRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		sendError(w, "Invalid request body", http.StatusBadRequest)
		return
	}

	for k, v := range req.Settings {
		// Mitigation T-09-01: Validate settings keys against whitelist
		if _, ok := defaultSettings[k]; !ok {
			sendError(w, "Invalid setting key: "+k, http.StatusBadRequest)
			return
		}

		// Mitigation T-09-02: Basic value validation
		if k == "theme_mode" && v != "light" && v != "dark" && v != "system" {
			sendError(w, "Invalid theme_mode: "+v, http.StatusBadRequest)
			return
		}

		setting := db.Setting{
			Key:   k,
			Value: v,
		}
		if err := h.DB.Save(&setting).Error; err != nil {
			sendError(w, "Failed to save setting: "+k, http.StatusInternalServerError)
			return
		}
	}

	// Return updated settings
	h.GetSettings(w, r)
}
