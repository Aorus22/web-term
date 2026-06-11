package api

import (
	"log"
	"webterm/internal/db"
	"webterm/internal/ssh"

	"gorm.io/gorm"
)

// AutoStartForwards tries to restart all port forwards marked with auto_start=true.
// Runs asynchronously so it doesn't block server startup. Failed forwards are
// gracefully disabled (auto_start set to false) so the UI shows them as off.
func AutoStartForwards(database *gorm.DB, tunnelMgr *ssh.TunnelManager) {
	go func() {
		var autoForwards []db.PortForward
		database.Where("auto_start = ?", true).Find(&autoForwards)

		for _, f := range autoForwards {
			func(f db.PortForward) {
				var conn db.Connection
				if err := database.First(&conn, "id = ?", f.ConnectionID).Error; err != nil {
					log.Printf("auto-start forward %s (%s): connection not found, disabling", f.ID, f.Name)
					database.Model(&f).Update("auto_start", false)
					return
				}
				if err := tunnelMgr.Start(f.ID, conn, f.LocalPort, f.RemotePort); err != nil {
					log.Printf("auto-start forward %s (%s) failed: %v — disabling", f.ID, f.Name, err)
					database.Model(&f).Update("auto_start", false)
				} else {
					log.Printf("auto-started port forward %s (%s)", f.ID, f.Name)
				}
			}(f)
		}
	}()
}
