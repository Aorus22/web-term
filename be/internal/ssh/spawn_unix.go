//go:build !windows

package ssh

import (
	"io"
	"os"
	"os/exec"

	"github.com/creack/pty"
)

func spawnLocalPTY(connectMsg ConnectMessage) (io.ReadWriteCloser, int, error) {
	shell := os.Getenv("SHELL")
	if shell == "" {
		shell = "/bin/bash"
	}
	c := exec.Command(shell)
	term := connectMsg.Term
	if term == "" {
		term = "xterm-256color"
	}
	c.Env = append(os.Environ(), "TERM="+term)

	// Set working directory: use Cwd from connect message, or fall back to $HOME
	if connectMsg.Cwd != "" {
		c.Dir = connectMsg.Cwd
	} else {
		home, err := os.UserHomeDir()
		if err == nil {
			c.Dir = home
		}
	}

	f, err := pty.Start(c)
	if err != nil {
		return nil, 0, err
	}

	if connectMsg.Rows > 0 && connectMsg.Cols > 0 {
		_ = pty.Setsize(f, &pty.Winsize{Rows: uint16(connectMsg.Rows), Cols: uint16(connectMsg.Cols)})
	}

	return f, c.Process.Pid, nil
}

func resizeLocalPTY(ptyRWC io.ReadWriteCloser, cols uint16, rows uint16) error {
	f, ok := ptyRWC.(*os.File)
	if !ok {
		return nil
	}
	return pty.Setsize(f, &pty.Winsize{Rows: rows, Cols: cols})
}
