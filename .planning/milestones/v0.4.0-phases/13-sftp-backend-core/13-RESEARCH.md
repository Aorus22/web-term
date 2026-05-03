# Phase 13: SFTP Backend Core - Research

## SFTP Library Analysis
- **Library**: `github.com/pkg/sftp`
- **Capabilities**: Full SFTP protocol support, compatible with `golang.org/x/crypto/ssh`.
- **Streaming**: Supports `io.Reader` and `io.Writer` via `File` objects returned by `Open` or `Create`.

## Architectural Integration
- **Independent Connections**: Each SFTP request (or a pool of them) will establish its own SSH connection to avoid interfering with terminal sessions.
- **Authentication**: Reuse `internal/db/models.go`'s `Connection` and `SSHKey` models. Decryption logic already exists in `internal/api/connections.go`.
- **Streaming Construct**: Use `io.Copy` between the SFTP file and the HTTP response body (for downloads) or the request body (for uploads).

## API Endpoints Design
- `GET /api/sftp/ls?connectionId=X&path=Y`: List directory contents.
- `GET /api/sftp/download?connectionId=X&path=Y`: Download file (streamed).
- `POST /api/sftp/upload?connectionId=X&path=Y`: Upload file (streamed).
- `DELETE /api/sftp/remove?connectionId=X&path=Y`: Remove file or directory.
- `POST /api/sftp/rename?connectionId=X&oldPath=Y&newPath=Z`: Rename/Move.
- `POST /api/sftp/mkdir?connectionId=X&path=Y`: Create directory.

## Implementation Details
- **FileSystem Interface**:
  ```go
  type FileSystem interface {
      List(path string) ([]FileInfo, error)
      Read(path string) (io.ReadCloser, error)
      Write(path string, r io.Reader) error
      Remove(path string) error
      Rename(oldPath, newPath string) error
      Mkdir(path string) error
  }
  ```
- **Error Handling**: Map SFTP errors to appropriate HTTP status codes (e.g., `os.IsNotExist` -> 404).

## Potential Pitfalls
- **Zombie Connections**: Must ensure SSH clients and SFTP sessions are closed promptly, especially for streaming operations. Use `defer` and `context` if possible.
- **Concurrent Access**: While standalone, multiple rapid requests for the same connection might be inefficient. For now, we follow "Standalone" per request or simple caching.
