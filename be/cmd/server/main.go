package main

import (
	"context"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"
	"webterm/internal/api"
	"webterm/internal/config"
	"webterm/internal/db"
	"webterm/internal/ssh"
)

func corsMux(next http.Handler, origins []string) http.Handler {
	allowed := make(map[string]bool, len(origins))
	allowAll := false
	for _, o := range origins {
		if o == "*" {
			allowAll = true
		}
		allowed[o] = true
	}
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		origin := r.Header.Get("Origin")
		
		// Robust same-host check (allows different ports on the same machine/IP)
		isSameHost := false
		if origin != "" {
			if origin == "null" {
				isSameHost = true // Support some electron/local file contexts
			} else {
				// Parse origin
				if parts := strings.Split(origin, "://"); len(parts) > 1 {
					originHostWithPort := strings.Split(parts[1], "/")[0]
					originHost := strings.Split(originHostWithPort, ":")[0]
					
					// Parse request host
					requestHost := strings.Split(r.Host, ":")[0]
					
					if originHost == requestHost || originHost == "localhost" || originHost == "127.0.0.1" {
						isSameHost = true
					}
				}
			}
		}

		if allowAll || allowed[origin] || isSameHost {
			w.Header().Set("Access-Control-Allow-Origin", origin)
			w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
			w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
			w.Header().Set("Access-Control-Allow-Credentials", "true")
		}
		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusNoContent)
			return
		}
		next.ServeHTTP(w, r)
	})
}

func main() {
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	database, err := db.Init(cfg.DBPath)
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}

	// Start local clipboard helper (system-wide clipboard on backend machine)
	if err := ssh.GlobalClipboardManager.StartLocalClipboard(); err != nil {
		log.Printf("Note: Local clipboard helper not available (optional): %v", err)
	} else {
		log.Println("Local clipboard helper started")
	}

	mux := http.NewServeMux()
	api.SetupRoutes(mux, database, cfg)

	handler := corsMux(mux, cfg.AllowedOrigins)

	// Create listener first to support port 0 (dynamic port)
	ln, err := net.Listen("tcp", "0.0.0.0"+cfg.Port)
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}
	actualPort := ln.Addr().(*net.TCPAddr).Port

	server := &http.Server{
		Handler: handler,
	}

	// Graceful shutdown
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		<-sigChan
		log.Println("Shutting down server...")
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		server.Shutdown(ctx)
		ln.Close()
	}()

	log.Printf("WebTerm backend starting on :%d (DB: %s)", actualPort, cfg.DBPath)
	// Output the port in a specific format for parent process to catch
	fmt.Printf("BACKEND_PORT:%d\n", actualPort)
	log.Printf("TLS: Use reverse proxy for production")
	if len(cfg.SSRFAllowlist) > 0 {
		log.Printf("SSRF allowlist: loaded %d patterns", len(cfg.SSRFAllowlist))
	} else {
		log.Printf("SSRF allowlist: empty (allowing all)")
	}

	if err := server.Serve(ln); err != http.ErrServerClosed {
		log.Fatalf("Server error: %v", err)
	}
	log.Println("Server stopped")
}
