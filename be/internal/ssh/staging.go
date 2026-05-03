package ssh

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"github.com/google/uuid"
)

type StagingManager struct {
	BaseDir string
}

func NewStagingManager(baseDir string) (*StagingManager, error) {
	if err := os.MkdirAll(baseDir, 0755); err != nil {
		return nil, err
	}
	return &StagingManager{BaseDir: baseDir}, nil
}

func (sm *StagingManager) StageFile(r io.Reader) (string, error) {
	tmpPath := filepath.Join(sm.BaseDir, uuid.New().String())
	f, err := os.Create(tmpPath)
	if err != nil {
		return "", fmt.Errorf("failed to create staging file at %s: %w", tmpPath, err)
	}
	defer f.Close()

	if _, err := io.Copy(f, r); err != nil {
		os.Remove(tmpPath)
		return "", err
	}
	return tmpPath, nil
}

func (sm *StagingManager) Cleanup(path string) error {
	return os.Remove(path)
}
