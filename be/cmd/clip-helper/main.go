package main

import (
	"bufio"
	"context"
	"encoding/base64"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"
)

var lastContent string

// Simple debug logging to file
func debugLog(format string, args ...interface{}) {
	f, _ := os.OpenFile("clip-helper.log", os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if f != nil {
		fmt.Fprintf(f, format+"\n", args...)
		f.Close()
	}
	fmt.Fprintf(os.Stderr, format+"\n", args...)
}

func main() {
	debugLog("[DEBUG] Starting clip-helper...")

	osName := detectOS()
	debugLog("[DEBUG] Detected OS: %s", osName)

	var readCmd []string
	var writeCmd func(string) *exec.Cmd

	switch osName {
	case "windows":
		readCmd = []string{"powershell", "-NoProfile", "-Command", "Get-Clipboard -Raw"}
		fmt.Fprintf(os.Stderr, "[DEBUG] Windows read cmd: %v\n", readCmd)
		writeCmd = func(content string) *exec.Cmd {
			cmd := exec.Command("powershell", "-NoProfile", "-Command", fmt.Sprintf("Set-Clipboard -Value '%s'", escapePowerShell(content)))
			return cmd
		}
	case "linux":
		if exists("wl-paste") && exists("wl-copy") {
			readCmd = []string{"wl-paste", "--no-newline"}
			writeCmd = func(content string) *exec.Cmd {
				return exec.Command("wl-copy", content)
			}
		} else if exists("xclip") {
			readCmd = []string{"xclip", "-selection", "clipboard", "-o"}
			writeCmd = func(content string) *exec.Cmd {
				return exec.Command("sh", "-c", fmt.Sprintf("printf '%s' | xclip -selection clipboard", escapeShell(content)))
			}
		} else if exists("xsel") {
			readCmd = []string{"xsel", "--clipboard", "--output"}
			writeCmd = func(content string) *exec.Cmd {
				return exec.Command("sh", "-c", fmt.Sprintf("printf '%s' | xsel --clipboard --input", escapeShell(content)))
			}
		} else {
			fmt.Fprintf(os.Stderr, "No clipboard tool found. Install wl-clipboard (Wayland) or xclip/xsel (X11).\n")
			os.Exit(1)
		}
	default:
		fmt.Fprintf(os.Stderr, "Unsupported OS: %s\n", osName)
		os.Exit(1)
	}

	debugLog("[DEBUG] Read cmd: %v", readCmd)

	// Channel to handle writes from stdin
	writeChan := make(chan string, 10)

	// Goroutine to read stdin for SET commands
	go func() {
		scanner := bufio.NewScanner(os.Stdin)
		for scanner.Scan() {
			line := scanner.Text()
			if strings.HasPrefix(line, "SET:") {
				encoded := strings.TrimPrefix(line, "SET:")
				decoded, err := base64.StdEncoding.DecodeString(encoded)
				if err == nil {
					writeChan <- string(decoded)
				}
			}
		}
	}()

	// Handle write commands
	go func() {
		for content := range writeChan {
			cmd := writeCmd(content)
			cmd.Run()
			// Update lastContent after write to prevent re-reading
			lastContent = content
		}
	}()

	// Poll for clipboard changes (read)
	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	debugLog("[DEBUG] Starting poll loop...")

	for range ticker.C {
		// Check for pending writes first
		select {
		case content := <-writeChan:
			lastContent = content
			debugLog("[DEBUG] Got write: %d chars", len(content))
		default:
		}

		content := getClipboard(readCmd)
		debugLog("[DEBUG] getClipboard result: len=%d, lastLen=%d", len(content), len(lastContent))

		if content == "" || content == lastContent {
			continue
		}

		lastContent = content
		encoded := base64.StdEncoding.EncodeToString([]byte(content))
		debugLog("[DEBUG] Sending CLIP: %d chars", len(content))
		// Also write to log file
		f, _ := os.OpenFile("clipboard-output.log", os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
		if f != nil {
			fmt.Fprintf(f, "CLIP:%s\n", encoded)
			f.Close()
		}
		fmt.Printf("CLIP:%s\n", encoded)
	}
}

func detectOS() string {
	out, err := exec.Command("uname", "-s").Output()
	if err == nil {
		result := strings.TrimSpace(string(out))
		if strings.Contains(result, "MINGW") || strings.Contains(result, "MSYS") || strings.Contains(result, "CYGWIN") {
			return "windows"
		}
	}

	if os.Getenv("SYSTEMROOT") != "" && strings.Contains(os.Getenv("SYSTEMROOT"), "Windows") {
		return "windows"
	}

	if exists("cmd") {
		out, _ := exec.Command("cmd", "/c", "echo windows").Output()
		if strings.Contains(string(out), "windows") {
			return "windows"
		}
	}

	return "linux"
}

func exists(cmd string) bool {
	_, err := exec.LookPath(cmd)
	return err == nil
}

func getClipboard(cmd []string) string {
	ctx, cancel := context.WithTimeout(context.Background(), 1*time.Second)
	defer cancel()

	c := exec.CommandContext(ctx, cmd[0], cmd[1:]...)
	out, err := c.Output()
	if err != nil {
		return ""
	}
	return string(out)
}

func escapePowerShell(s string) string {
	s = strings.ReplaceAll(s, "'", "''")
	return s
}

func escapeShell(s string) string {
	s = strings.ReplaceAll(s, "'", "'\\''")
	return s
}