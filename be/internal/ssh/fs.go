package ssh

import (
	"io"
	"os"
	"time"

	"github.com/pkg/sftp"
)

// FileInfo mimics os.FileInfo but is serializable to JSON easily
type FileInfo struct {
	Name    string      `json:"name"`
	Size    int64       `json:"size"`
	Mode    os.FileMode `json:"mode"`
	ModTime time.Time   `json:"modTime"`
	IsDir   bool        `json:"isDir"`
}

// FileSystem defines a unified interface for Local and Remote (SFTP) file operations
type FileSystem interface {
	List(path string) ([]FileInfo, error)
	Read(path string) (io.ReadCloser, error)
	Write(path string, r io.Reader) error
	Remove(path string) error
	Rename(oldPath, newPath string) error
	Mkdir(path string) error
	Stat(path string) (FileInfo, error)
}

// LocalFS implements FileSystem for the local machine
type LocalFS struct{}

func (fs *LocalFS) List(path string) ([]FileInfo, error) {
	entries, err := os.ReadDir(path)
	if err != nil {
		return nil, err
	}

	var files []FileInfo
	for _, entry := range entries {
		info, err := entry.Info()
		if err != nil {
			continue
		}
		files = append(files, FileInfo{
			Name:    info.Name(),
			Size:    info.Size(),
			Mode:    info.Mode(),
			ModTime: info.ModTime(),
			IsDir:   info.IsDir(),
		})
	}
	return files, nil
}

func (fs *LocalFS) Read(path string) (io.ReadCloser, error) {
	return os.Open(path)
}

func (fs *LocalFS) Write(path string, r io.Reader) error {
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = io.Copy(f, r)
	return err
}

func (fs *LocalFS) Remove(path string) error {
	return os.RemoveAll(path)
}

func (fs *LocalFS) Rename(oldPath, newPath string) error {
	return os.Rename(oldPath, newPath)
}

func (fs *LocalFS) Mkdir(path string) error {
	return os.MkdirAll(path, 0755)
}

func (fs *LocalFS) Stat(path string) (FileInfo, error) {
	info, err := os.Stat(path)
	if err != nil {
		return FileInfo{}, err
	}
	return FileInfo{
		Name:    info.Name(),
		Size:    info.Size(),
		Mode:    info.Mode(),
		ModTime: info.ModTime(),
		IsDir:   info.IsDir(),
	}, nil
}

// SFTPFS implements FileSystem for a remote host via SFTP
type SFTPFS struct {
	Client *sftp.Client
}

func (fs *SFTPFS) List(path string) ([]FileInfo, error) {
	entries, err := fs.Client.ReadDir(path)
	if err != nil {
		return nil, err
	}

	var files []FileInfo
	for _, info := range entries {
		files = append(files, FileInfo{
			Name:    info.Name(),
			Size:    info.Size(),
			Mode:    info.Mode(),
			ModTime: info.ModTime(),
			IsDir:   info.IsDir(),
		})
	}
	return files, nil
}

func (fs *SFTPFS) Read(path string) (io.ReadCloser, error) {
	return fs.Client.Open(path)
}

func (fs *SFTPFS) Write(path string, r io.Reader) error {
	f, err := fs.Client.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_TRUNC)
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = io.Copy(f, r)
	return err
}

func (fs *SFTPFS) Remove(path string) error {
	info, err := fs.Client.Stat(path)
	if err != nil {
		return err
	}
	if info.IsDir() {
		return fs.Client.RemoveDirectory(path)
	}
	return fs.Client.Remove(path)
}

func (fs *SFTPFS) Rename(oldPath, newPath string) error {
	return fs.Client.Rename(oldPath, newPath)
}

func (fs *SFTPFS) Mkdir(path string) error {
	return fs.Client.MkdirAll(path)
}

func (fs *SFTPFS) Stat(path string) (FileInfo, error) {
	info, err := fs.Client.Stat(path)
	if err != nil {
		return FileInfo{}, err
	}
	return FileInfo{
		Name:    info.Name(),
		Size:    info.Size(),
		Mode:    info.Mode(),
		ModTime: info.ModTime(),
		IsDir:   info.IsDir(),
	}, nil
}
