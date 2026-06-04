//go:build windows

package ssh

import (
	"io"
	"os"
	"os/exec"

	"github.com/UserExistsError/conpty"
)

func spawnLocalPTY(connectMsg ConnectMessage) (io.ReadWriteCloser, int, error) {
	shell := os.Getenv("COMSPEC")
	if shell == "" {
		shell = "cmd.exe"
	}

	if _, err := exec.LookPath("powershell.exe"); err == nil {
		shell = "powershell.exe"
	}

	var opts []conpty.ConPtyOption
	if connectMsg.Cols > 0 && connectMsg.Rows > 0 {
		opts = append(opts, conpty.ConPtyDimensions(connectMsg.Cols, connectMsg.Rows))
	}

	cpty, err := conpty.Start(shell, opts...)
	if err != nil {
		return nil, 0, err
	}

	return cpty, cpty.Pid(), nil
}

func resizeLocalPTY(ptyRWC io.ReadWriteCloser, cols uint16, rows uint16) error {
	if cpty, ok := ptyRWC.(*conpty.ConPty); ok {
		return cpty.Resize(int(cols), int(rows))
	}
	return nil
}
