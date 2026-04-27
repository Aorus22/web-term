package main

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
	"webterm/internal/config"
	"webterm/internal/db"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	database, err := db.Init(cfg.DBPath)
	if err != nil {
		log.Fatalf("Failed to initialize database: %v", err)
	}
	_ = database

	mux := http.NewServeMux()
	// Routes will be registered here in Task 2
	// api.SetupRoutes(mux, database, cfg)

	server := &http.Server{
		Addr:    "0.0.0.0" + cfg.Port, // Config.Port usually starts with :
		Handler: mux,
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
	}()

	log.Printf("WebTerm backend starting on %s (DB: %s)", cfg.Port, cfg.DBPath)
	log.Printf("TLS: Use reverse proxy for production")
	if len(cfg.SSRFAllowlist) > 0 {
		log.Printf("SSRF allowlist: loaded %d patterns", len(cfg.SSRFAllowlist))
	} else {
		log.Printf("SSRF allowlist: empty (allowing all)")
	}

	if err := server.ListenAndServe(); err != http.ErrServerClosed {
		log.Fatalf("Server error: %v", err)
	}
	log.Println("Server stopped")
}
