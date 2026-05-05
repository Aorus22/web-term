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

type ProgressReader struct {
	transferID     string
	manager        *TransferManager
	bytesRead      int64
	totalBytes     int64
	originalReader io.Reader
}

func NewProgressReader(transferID string, manager *TransferManager, totalBytes int64, originalReader io.Reader) *ProgressReader {
	return &ProgressReader{
		transferID:     transferID,
		manager:        manager,
		totalBytes:     totalBytes,
		originalReader: originalReader,
	}
}

func (pr *ProgressReader) Read(p []byte) (n int, err error) {
	n, err = pr.originalReader.Read(p)
	if n > 0 {
		pr.bytesRead += int64(n)
		pr.manager.UpdateProgress(pr.transferID, pr.bytesRead)
	}
	return n, err
}
