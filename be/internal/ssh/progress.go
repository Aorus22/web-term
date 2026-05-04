package ssh

import (
	"io"
)

type ProgressWriter struct {
	transferID     string
	manager        *TransferManager
	bytesWritten   int64
	totalBytes     int64
	originalWriter io.Writer
}

func NewProgressWriter(transferID string, manager *TransferManager, totalBytes int64, originalWriter io.Writer) *ProgressWriter {
	return &ProgressWriter{
		transferID:     transferID,
		manager:        manager,
		totalBytes:     totalBytes,
		originalWriter: originalWriter,
	}
}

func (pw *ProgressWriter) Write(p []byte) (n int, err error) {
	n, err = pw.originalWriter.Write(p)
	if n > 0 {
		pw.bytesWritten += int64(n)
		pw.manager.UpdateProgress(pw.transferID, pw.bytesWritten)
	}
	return n, err
}

func (pw *ProgressWriter) BytesWritten() int64 {
	return pw.bytesWritten
}
