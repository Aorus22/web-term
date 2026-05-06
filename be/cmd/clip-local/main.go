package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"
	"os/exec"
	"strings"
	"time"
)

var lastContent string

func main() {
	osName := detectOS()

	var readCmd []string

	switch osName {
	case "windows":
		readCmd = []string{"powershell", "-Command", "Get-Clipboard -Raw"}
	case "linux":
		if exists("wl-paste") {
			readCmd = []string{"wl-paste", "--no-newline"}
		} else if exists("xclip") {
			readCmd = []string{"xclip", "-selection", "clipboard", "-o"}
		} else if exists("xsel") {
			readCmd = []string{"xsel", "--clipboard", "--output"}
		} else {
			fmt.Fprintf(os.Stderr, "No clipboard tool found. Install wl-clipboard (Wayland) or xclip/xsel (X11).\n")
			os.Exit(1)
		}
	default:
		fmt.Fprintf(os.Stderr, "Unsupported OS: %s\n", osName)
		os.Exit(1)
	}

	ticker := time.NewTicker(500 * time.Millisecond)
	defer ticker.Stop()

	for range ticker.C {
		content := getClipboard(readCmd)
		if content == "" || content == lastContent {
			continue
		}

		lastContent = content
		encoded := base64.StdEncoding.EncodeToString([]byte(content))
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