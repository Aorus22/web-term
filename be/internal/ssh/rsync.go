package ssh

import (
	"bufio"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/pem"
	"errors"
	"fmt"
	"io"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
	"sync"

	"webterm/internal/config"
	"webterm/internal/db"

	"golang.org/x/crypto/ssh"
	"gorm.io/gorm"
)

// RsyncAvailability describes whether rsync + ssh are usable on the backend machine.
type RsyncAvailability struct {
	Available    bool   `json:"available"`
	RsyncPath    string `json:"rsync_path,omitempty"`
	RsyncVersion string `json:"rsync_version,omitempty"`
	SSHPath      string `json:"ssh_path,omitempty"`
	SSHVersion   string `json:"ssh_version,omitempty"`
}

// maxRawLines is how many raw stderr lines are kept for the error tail.
const maxRawLines = 20

// ---------------------------------------------------------------------------
// Availability probes
// ---------------------------------------------------------------------------

// CheckLocalRsync checks whether rsync and ssh are usable on the backend
// machine and returns their paths and versions. Availability requires both
// binaries to be present.
func CheckLocalRsync() RsyncAvailability {
	var a RsyncAvailability

	if p, err := exec.LookPath("rsync"); err == nil {
		a.RsyncPath = p
		if out, err := exec.Command("rsync", "--version").Output(); err == nil {
			a.RsyncVersion = firstLine(string(out))
		}
	}

	if p, err := exec.LookPath("ssh"); err == nil {
		a.SSHPath = p
		// ssh -V writes its version banner to stderr.
		out, _ := exec.Command("ssh", "-V").CombinedOutput()
		if len(out) > 0 {
			a.SSHVersion = firstLine(string(out))
		}
	}

	a.Available = a.RsyncPath != "" && a.SSHPath != ""
	return a
}

// ProbeRemoteRsync checks whether rsync is installed on a remote host.
// It returns (available, versionOrEmpty, err). err is only non-nil when the
// SSH connection itself fails.
func ProbeRemoteRsync(database *gorm.DB, conn db.Connection, cfg *config.Config) (bool, string, error) {
	client, err := DialSSH(database, conn, cfg, "")
	if err != nil {
		return false, "", err
	}
	defer client.Close()

	session, err := client.NewSession()
	if err != nil {
		return false, "", err
	}
	defer session.Close()

	out, err := session.Output("command -v rsync 2>/dev/null || which rsync 2>/dev/null")
	if err != nil {
		// Not found is not an error; command -v exits non-zero when absent.
		return false, "", nil
	}
	if strings.TrimSpace(string(out)) == "" {
		return false, "", nil
	}

	versionOut, err := session.Output("rsync --version 2>/dev/null | head -1")
	if err != nil {
		return true, "", nil
	}
	return true, strings.TrimSpace(string(versionOut)), nil
}

// ---------------------------------------------------------------------------
// Top-level transfer entry points
// ---------------------------------------------------------------------------

// RunRsyncTransfer executes an rsync transfer between any combination of local
// and remote endpoints. It runs synchronously and reports progress/final state
// through the TransferManager.
func RunRsyncTransfer(database *gorm.DB, cfg *config.Config, tm *TransferManager, transferID, srcConnID, srcPath, dstConnID, dstPath string) error {
	if tm == nil {
		return fmt.Errorf("transfer manager is nil")
	}
	ensureTransfer(tm, transferID)

	srcConn, srcLocal, err := resolveConnection(database, srcConnID)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}
	dstConn, dstLocal, err := resolveConnection(database, dstConnID)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}

	switch {
	case srcLocal && dstLocal:
		return runLocalToLocal(tm, transferID, srcPath, dstPath)
	case srcLocal && !dstLocal:
		return runLocalToRemote(tm, transferID, cfg, database, srcPath, dstConn, dstPath)
	case !srcLocal && dstLocal:
		return runRemoteToLocal(tm, transferID, cfg, database, srcConn, srcPath, dstPath)
	default:
		return runRemoteToRemote(tm, transferID, cfg, database, srcConn, srcPath, dstConn, dstPath)
	}
}

// RunRsyncUpload uploads an already-staged local file (absolute path) to a
// remote destination, reusing the local->remote strategy.
func RunRsyncUpload(database *gorm.DB, cfg *config.Config, tm *TransferManager, transferID, stagedLocalPath, dstConnID, dstPath string) error {
	if tm == nil {
		return fmt.Errorf("transfer manager is nil")
	}
	ensureTransfer(tm, transferID)

	absPath, err := filepath.Abs(stagedLocalPath)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}

	info, err := os.Stat(absPath)
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("staged source not accessible: %w", err))
	}
	if info.Mode().IsRegular() {
		tm.SetTotalBytes(transferID, info.Size())
	}

	dstConn, dstLocal, err := resolveConnection(database, dstConnID)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}
	if dstLocal {
		return failTransfer(tm, transferID, fmt.Errorf("rsync upload requires a remote destination connection"))
	}

	return runLocalToRemote(tm, transferID, cfg, database, absPath, dstConn, dstPath)
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

func runLocalToLocal(tm *TransferManager, transferID, srcPath, dstPath string) error {
	workDir, paths := rsyncLocalize([]string{srcPath, dstPath})
	cmd := exec.Command("rsync", "-az", "--partial", "--info=progress2", paths[0], paths[1])
	if workDir != "" {
		cmd.Dir = workDir
	}
	return runRsyncCommand(tm, transferID, cmd)
}

func runLocalToRemote(tm *TransferManager, transferID string, cfg *config.Config, database *gorm.DB, localPath string, remote db.Connection, remotePath string) error {
	return runLocalRemote(tm, transferID, cfg, database, true, localPath, remote, remotePath)
}

func runRemoteToLocal(tm *TransferManager, transferID string, cfg *config.Config, database *gorm.DB, remote db.Connection, remotePath, localPath string) error {
	return runLocalRemote(tm, transferID, cfg, database, false, localPath, remote, remotePath)
}

// runLocalRemote covers both local->remote (upload) and remote->local
// (download). It injects an ephemeral key into the remote host's
// authorized_keys and runs a local rsync using that key over ssh.
func runLocalRemote(tm *TransferManager, transferID string, cfg *config.Config, database *gorm.DB, upload bool, localPath string, remote db.Connection, remotePath string) error {
	if err := checkHostAllowed(cfg, remote); err != nil {
		return failTransfer(tm, transferID, err)
	}
	if err := validateTransferID(transferID); err != nil {
		return failTransfer(tm, transferID, err)
	}

	privPEM, authLine, err := generateEphemeralKey(transferID)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}

	keyPath, err := writeTempKey(privPEM)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}
	defer func() {
		if err := os.Remove(keyPath); err != nil && !os.IsNotExist(err) {
			log.Printf("rsync: failed to remove temp key %s: %v", keyPath, err)
		}
	}()

	if err := injectAuthorizedKey(cfg, database, remote, authLine); err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to inject ephemeral key on %s: %w", remote.Host, err))
	}
	defer func() {
		if err := removeAuthorizedKeyLine(cfg, database, remote, transferID); err != nil {
			log.Printf("rsync: cleanup of authorized_keys on %s failed: %v", remote.Host, err)
		}
	}()

	sshOpts := fmt.Sprintf("ssh -i %s -p %d -o StrictHostKeyChecking=no -o IdentitiesOnly=yes",
		shellQuotePath(keyPath), connPort(remote))

	workDir, rel := rsyncLocalize([]string{localPath})
	localArg := rel[0]

	var args []string
	if upload {
		args = []string{"-az", "--partial", "--info=progress2", "-e", sshOpts, localArg,
			fmt.Sprintf("%s@%s:%s", remote.Username, remote.Host, remotePath)}
	} else {
		args = []string{"-az", "--partial", "--info=progress2", "-e", sshOpts,
			fmt.Sprintf("%s@%s:%s", remote.Username, remote.Host, remotePath), localArg}
	}

	cmd := exec.Command("rsync", args...)
	if workDir != "" {
		cmd.Dir = workDir
	}
	return runRsyncCommand(tm, transferID, cmd)
}

// runRemoteToRemote copies from one remote host to another by running rsync on
// the source host. The ephemeral public key is injected into the destination's
// authorized_keys and the matching private key is dropped on the source host.
func runRemoteToRemote(tm *TransferManager, transferID string, cfg *config.Config, database *gorm.DB, srcConn db.Connection, srcPath string, dstConn db.Connection, dstPath string) error {
	if err := checkHostAllowed(cfg, srcConn); err != nil {
		return failTransfer(tm, transferID, err)
	}
	if err := checkHostAllowed(cfg, dstConn); err != nil {
		return failTransfer(tm, transferID, err)
	}
	if err := validateTransferID(transferID); err != nil {
		return failTransfer(tm, transferID, err)
	}
	quotedSrc, err := quoteSafePath(srcPath)
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("source path unsafe: %w", err))
	}
	if strings.ContainsAny(dstConn.Username, "'\r\n") || strings.ContainsAny(dstConn.Host, "'\r\n") {
		return failTransfer(tm, transferID, fmt.Errorf("destination username/host contains unsafe characters"))
	}
	quotedDst, err := quoteSafePath(dstConn.Username + "@" + dstConn.Host + ":" + dstPath)
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("destination path unsafe: %w", err))
	}

	privPEM, authLine, err := generateEphemeralKey(transferID)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}

	keyPath, err := writeTempKey(privPEM)
	if err != nil {
		return failTransfer(tm, transferID, err)
	}
	defer func() {
		if err := os.Remove(keyPath); err != nil && !os.IsNotExist(err) {
			log.Printf("rsync: failed to remove temp key %s: %v", keyPath, err)
		}
	}()

	// Inject ephemeral public key into the DESTINATION host.
	if err := injectAuthorizedKey(cfg, database, dstConn, authLine); err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to inject ephemeral key on %s: %w", dstConn.Host, err))
	}
	defer func() {
		if err := removeAuthorizedKeyLine(cfg, database, dstConn, transferID); err != nil {
			log.Printf("rsync: cleanup of authorized_keys on %s failed: %v", dstConn.Host, err)
		}
	}()

	// Connect to the SOURCE host where rsync will run.
	srcClient, err := DialSSH(database, srcConn, cfg, "")
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to connect to source host %s: %w", srcConn.Host, err))
	}
	defer srcClient.Close()

	// Place the ephemeral PRIVATE key on the source host.
	keySess, err := srcClient.NewSession()
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to open session on source host: %w", err))
	}
	err = placePrivateKeyOnRemote(keySess, transferID, privPEM)
	keySess.Close()
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to place ephemeral key on source host %s: %w", srcConn.Host, err))
	}

	// Best-effort cleanup of the private key on the source host.
	defer func() {
		cleanupSess, err := srcClient.NewSession()
		if err != nil {
			log.Printf("rsync: cleanup of source-host key failed: %v", err)
			return
		}
		defer cleanupSess.Close()
		if err := cleanupSess.Run("rm -f ~/.ssh/webterm-" + transferID + ".key"); err != nil {
			log.Printf("rsync: cleanup of source-host key failed: %v", err)
		}
	}()

	sshOpts := fmt.Sprintf("ssh -i ~/.ssh/webterm-%s.key -p %d -o StrictHostKeyChecking=no -o IdentitiesOnly=yes",
		transferID, connPort(dstConn))
	command := fmt.Sprintf("rsync -az --partial --info=progress2 -e \"%s\" %s %s", sshOpts, quotedSrc, quotedDst)

	sess, err := srcClient.NewSession()
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to open rsync session on source host: %w", err))
	}
	defer sess.Close()

	return runRsyncSession(tm, transferID, sess, command)
}

// ---------------------------------------------------------------------------
// Progress scanning
// ---------------------------------------------------------------------------

// runRsyncCommand runs a local rsync process while scanning its stderr for
// --info=progress2 output.
func runRsyncCommand(tm *TransferManager, transferID string, cmd *exec.Cmd) error {
	stderr, err := cmd.StderrPipe()
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to open rsync stderr: %w", err))
	}
	if err := cmd.Start(); err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to start rsync: %v", err))
	}

	tail := scanProgress(tm, transferID, stderr)
	err = cmd.Wait()
	if err == nil {
		tm.SetComplete(transferID)
		return nil
	}

	var exitErr *exec.ExitError
	exitCode := "?"
	if errors.As(err, &exitErr) {
		exitCode = strconv.Itoa(exitErr.ExitCode())
	}
	msg := fmt.Sprintf("rsync failed (exit %s): %s", exitCode, tail)
	tm.SetError(transferID, errors.New(msg))
	return errors.New(msg)
}

// runRsyncSession runs an rsync command through a remote ssh session while
// scanning its stderr for --info=progress2 output.
func runRsyncSession(tm *TransferManager, transferID string, session *ssh.Session, command string) error {
	stderr, err := session.StderrPipe()
	if err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to open remote rsync stderr: %w", err))
	}
	if err := session.Start(command); err != nil {
		return failTransfer(tm, transferID, fmt.Errorf("failed to start remote rsync: %w", err))
	}

	tail := scanProgress(tm, transferID, stderr)
	err = session.Wait()
	if err == nil {
		tm.SetComplete(transferID)
		return nil
	}

	var exitErr *ssh.ExitError
	exitCode := "?"
	if errors.As(err, &exitErr) {
		exitCode = strconv.Itoa(exitErr.ExitStatus())
	}
	msg := fmt.Sprintf("rsync failed (exit %s): %s", exitCode, tail)
	tm.SetError(transferID, errors.New(msg))
	return errors.New(msg)
}

// scanProgress consumes an io.Reader (rsync stderr) splitting on both '\r' and
// '\n', parses --info=progress2 lines and updates the TransferManager. It
// returns the last maxRawLines raw lines for use in error messages.
//
// The parsed "total" is rsync's file-count from the to-chk/ir-chk denominator.
// A pre-set byte total (e.g. set by RunRsyncUpload from os.Stat) is preserved
// rather than clobbered with the file count.
func scanProgress(tm *TransferManager, transferID string, r io.Reader) string {
	var mu sync.Mutex
	var raw []string

	appendRaw := func(line string) {
		mu.Lock()
		defer mu.Unlock()
		raw = append(raw, line)
		if len(raw) > maxRawLines {
			raw = raw[len(raw)-maxRawLines:]
		}
	}

	scanner := bufio.NewScanner(r)
	scanner.Split(splitCRLF)
	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}
		appendRaw(line)

		if bytesDone, total, _, ok := parseRsyncProgress2(line); ok {
			if total > 0 {
				if st, err := tm.GetStatus(transferID); err == nil && st.TotalBytes == 0 {
					tm.SetTotalBytes(transferID, total)
				}
			}
			tm.UpdateProgress(transferID, bytesDone)
		}
	}

	mu.Lock()
	defer mu.Unlock()
	return strings.Join(raw, "\n")
}

// splitCRLF is a bufio.SplitFunc that splits on either '\r' or '\n' (rsync uses
// carriage returns to redraw progress lines when stderr is a tty, newlines
// otherwise).
func splitCRLF(data []byte, atEOF bool) (advance int, token []byte, err error) {
	if atEOF && len(data) == 0 {
		return 0, nil, nil
	}
	for i, b := range data {
		if b == '\r' || b == '\n' {
			return i + 1, data[:i], nil
		}
	}
	if atEOF {
		return len(data), data, nil
	}
	return 0, nil, nil
}

// parseRsyncProgress2 parses a single --info=progress2 stderr line such as:
//
//	     12,345,678  45%    3.21MB/s    0:00:04 (xfr#3, to-chk=99/101)
//
// It returns (bytes, total, pct, ok). bytes is the leading comma-separated
// byte count, pct the NN% value, and total the denominator from the
// to-chk=NN/NN (or ir-chk=NN/NN) token when present and > 0. ok is false when
// the line cannot be parsed as a progress line.
func parseRsyncProgress2(line string) (bytes int64, total int64, pct int, ok bool) {
	fields := strings.Fields(line)
	if len(fields) < 2 {
		return 0, 0, 0, false
	}

	byteStr := strings.ReplaceAll(fields[0], ",", "")
	n, err := strconv.ParseInt(byteStr, 10, 64)
	if err != nil || n < 0 {
		return 0, 0, 0, false
	}

	p := -1
	for _, f := range fields {
		if strings.HasSuffix(f, "%") {
			if v, err := strconv.Atoi(strings.TrimSuffix(f, "%")); err == nil {
				p = v
				break
			}
		}
	}
	if p < 0 {
		return 0, 0, 0, false
	}

	for _, f := range fields {
		if idx := strings.Index(f, "to-chk="); idx >= 0 {
			total = checkDenominator(f[idx+len("to-chk="):])
			break
		}
		if idx := strings.Index(f, "ir-chk="); idx >= 0 {
			total = checkDenominator(f[idx+len("ir-chk="):])
			break
		}
	}

	return n, total, p, true
}

// checkDenominator extracts the denominator from a to-chk/ir-chk value such as
// "99/101", "4/8)", or "0/1". It returns 0 when no denominator can be parsed.
func checkDenominator(s string) int64 {
	idx := strings.LastIndex(s, "/")
	if idx < 0 {
		return 0
	}
	var sb strings.Builder
	for _, r := range s[idx+1:] {
		if r >= '0' && r <= '9' {
			sb.WriteRune(r)
		} else {
			break
		}
	}
	if sb.Len() == 0 {
		return 0
	}
	v, err := strconv.ParseInt(sb.String(), 10, 64)
	if err != nil {
		return 0
	}
	return v
}

// ---------------------------------------------------------------------------
// Ephemeral key handling
// ---------------------------------------------------------------------------

// generateEphemeralKey creates an in-memory ed25519 keypair. It returns the PEM
// encoding of the private key and the full authorized_keys line to inject on a
// remote host.
func generateEphemeralKey(id string) (privPEM []byte, authorizedLine string, err error) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return nil, "", fmt.Errorf("failed to generate ephemeral ed25519 key: %w", err)
	}

	block, err := ssh.MarshalPrivateKey(priv, "webterm-ephemeral-"+id)
	if err != nil {
		return nil, "", fmt.Errorf("failed to marshal ephemeral private key: %w", err)
	}
	privPEM = pem.EncodeToMemory(block)

	sshPub, err := ssh.NewPublicKey(pub)
	if err != nil {
		return nil, "", fmt.Errorf("failed to derive ephemeral public key: %w", err)
	}
	pubKeyStr := strings.TrimSpace(string(ssh.MarshalAuthorizedKey(sshPub)))

	return privPEM, buildEphemeralAuthorizedKeyLine(pubKeyStr, id), nil
}

// buildEphemeralAuthorizedKeyLine builds an authorized_keys entry that only
// permits the ephemeral key to run rsync (via $SSH_ORIGINAL_COMMAND) and
// carries a unique webterm-ephemeral-<id> marker used for cleanup.
func buildEphemeralAuthorizedKeyLine(pubKeyStr string, id string) string {
	return `command="rsync --server $SSH_ORIGINAL_COMMAND",no-agent-forwarding,no-port-forwarding,no-X11-forwarding,no-pty ` +
		strings.TrimSpace(pubKeyStr) + " webterm-ephemeral-" + id
}

// writeTempKey writes a private key PEM to a 0600 temp file and returns its path.
func writeTempKey(privPEM []byte) (string, error) {
	f, err := os.CreateTemp("", "webterm-rsync-*.key")
	if err != nil {
		return "", fmt.Errorf("failed to create temp key file: %w", err)
	}
	path := f.Name()
	if _, err := f.Write(privPEM); err != nil {
		f.Close()
		os.Remove(path)
		return "", fmt.Errorf("failed to write temp key file: %w", err)
	}
	if err := f.Close(); err != nil {
		os.Remove(path)
		return "", fmt.Errorf("failed to close temp key file: %w", err)
	}
	if err := os.Chmod(path, 0600); err != nil {
		os.Remove(path)
		return "", fmt.Errorf("failed to chmod temp key file: %w", err)
	}
	return path, nil
}

// injectAuthorizedKey appends an ephemeral authorized_keys entry on a remote
// host over ssh.
func injectAuthorizedKey(cfg *config.Config, database *gorm.DB, conn db.Connection, entry string) error {
	client, err := DialSSH(database, conn, cfg, "")
	if err != nil {
		return err
	}
	defer client.Close()

	session, err := client.NewSession()
	if err != nil {
		return err
	}
	defer session.Close()

	stdin, err := session.StdinPipe()
	if err != nil {
		return err
	}
	cmd := "mkdir -p ~/.ssh && chmod 700 ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys"
	if err := session.Start(cmd); err != nil {
		return err
	}
	if _, err := io.WriteString(stdin, entry+"\n"); err != nil {
		return err
	}
	if err := stdin.Close(); err != nil {
		return err
	}
	return session.Wait()
}

// removeAuthorizedKeyLine removes the webterm-ephemeral-<transferID> marker
// line from a remote host's authorized_keys. transferID must be validated with
// validateTransferID before calling.
func removeAuthorizedKeyLine(cfg *config.Config, database *gorm.DB, conn db.Connection, transferID string) error {
	client, err := DialSSH(database, conn, cfg, "")
	if err != nil {
		return err
	}
	defer client.Close()

	session, err := client.NewSession()
	if err != nil {
		return err
	}
	defer session.Close()

	cmd := fmt.Sprintf("sed -i '/webterm-ephemeral-%s/d' ~/.ssh/authorized_keys", transferID)
	return session.Run(cmd)
}

// placePrivateKeyOnRemote writes the ephemeral private key to
// ~/.ssh/webterm-<id>.key on a remote host and chmods it 600. id must be
// validated with validateTransferID before calling.
func placePrivateKeyOnRemote(session *ssh.Session, id string, keyPEM []byte) error {
	stdin, err := session.StdinPipe()
	if err != nil {
		return err
	}
	cmd := "mkdir -p ~/.ssh && cat > ~/.ssh/webterm-" + id + ".key && chmod 600 ~/.ssh/webterm-" + id + ".key"
	if err := session.Start(cmd); err != nil {
		return err
	}
	if _, err := stdin.Write(keyPEM); err != nil {
		return err
	}
	if err := stdin.Close(); err != nil {
		return err
	}
	return session.Wait()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

// ensureTransfer creates a transfer record if one does not already exist so
// that progress/final state always has somewhere to go.
func ensureTransfer(tm *TransferManager, transferID string) {
	if _, err := tm.GetStatus(transferID); err != nil {
		tm.CreateTransfer(transferID, 0)
	}
}

// failTransfer records the error on the transfer and returns it.
func failTransfer(tm *TransferManager, transferID string, err error) error {
	tm.SetError(transferID, err)
	return err
}

// resolveConnection loads a connection record by ID. The id "local" (or an
// empty id) resolves to the local machine.
func resolveConnection(database *gorm.DB, connID string) (db.Connection, bool, error) {
	if connID == "" || connID == "local" {
		return db.Connection{}, true, nil
	}
	if database == nil {
		return db.Connection{}, false, fmt.Errorf("cannot resolve remote connection %q: database is nil", connID)
	}
	var conn db.Connection
	if err := database.First(&conn, "id = ?", connID).Error; err != nil {
		return db.Connection{}, false, fmt.Errorf("connection not found: %s: %w", connID, err)
	}
	return conn, false, nil
}

// checkHostAllowed enforces the SSRF allowlist before any host is dialed or
// placed on a command line.
func checkHostAllowed(cfg *config.Config, conn db.Connection) error {
	if cfg == nil {
		return fmt.Errorf("config is nil; cannot authorize host")
	}
	if !cfg.IsHostAllowed(conn.Host) {
		return fmt.Errorf("host not authorized by security policy: %s", conn.Host)
	}
	return nil
}

// connPort returns a sane port for a connection, defaulting to 22.
func connPort(conn db.Connection) int {
	if conn.Port <= 0 || conn.Port > 65535 {
		return 22
	}
	return conn.Port
}

// validateTransferID ensures a transfer id is safe to embed in shell commands
// (sed expressions, file names) and as an authorized_keys comment.
func validateTransferID(id string) error {
	if id == "" {
		return fmt.Errorf("transfer id is empty")
	}
	for _, r := range id {
		switch {
		case r >= 'a' && r <= 'z', r >= 'A' && r <= 'Z', r >= '0' && r <= '9', r == '-', r == '_', r == '.':
		default:
			return fmt.Errorf("transfer id contains unsafe characters: %q", id)
		}
	}
	return nil
}

// quoteSafePath single-quotes a path for use in a remote shell command line,
// rejecting paths that cannot be safely quoted.
func quoteSafePath(p string) (string, error) {
	if strings.ContainsAny(p, "'\r\n") {
		return "", fmt.Errorf("path contains a single quote or newline and cannot be safely quoted: %q", p)
	}
	return "'" + p + "'", nil
}

// shellQuotePath quotes a local filesystem path for embedding inside an rsync
// -e value (which rsync passes to a shell). Paths without spaces pass through.
func shellQuotePath(p string) string {
	if p == "" || strings.ContainsAny(p, " \t") {
		return "'" + p + "'"
	}
	return p
}

// rsyncLocalize rewrites absolute local paths into relative arguments run from
// a shared working directory. Some Cygwin-based Windows rsync builds misparse
// drive-letter paths ("C:\..." or "C:/...") as remote host:path specs, so
// running rsync from a common parent with relative operands sidesteps the
// problem entirely. On Linux the same transformation is harmless. When the
// paths cannot share a working directory (relative input, different volumes,
// or a common ancestor that cannot be computed) the originals are returned
// with an empty working directory.
func rsyncLocalize(paths []string) (workDir string, relPaths []string) {
	for _, p := range paths {
		if !filepath.IsAbs(p) {
			return "", paths
		}
	}

	if len(paths) == 1 {
		return filepath.Dir(paths[0]), []string{filepath.Base(paths[0])}
	}

	dir := filepath.Dir(paths[0])
	for _, p := range paths[1:] {
		dir = commonDir(dir, p)
		if dir == "" {
			return "", paths
		}
	}

	rel := make([]string, len(paths))
	for i, p := range paths {
		r, err := filepath.Rel(dir, p)
		if err != nil {
			return "", paths
		}
		rel[i] = r
	}
	return dir, rel
}

// commonDir returns the longest common ancestor directory of two absolute
// paths (case-insensitive for drive/component comparison so it behaves
// correctly on Windows), or "" when no common ancestor exists.
func commonDir(a, b string) string {
	va, vb := filepath.VolumeName(a), filepath.VolumeName(b)
	if va != vb {
		return ""
	}
	ca, cb := pathComponents(a), pathComponents(b)
	n := 0
	for n < len(ca) && n < len(cb) && strings.EqualFold(ca[n], cb[n]) {
		n++
	}
	if n == 0 {
		return ""
	}
	joined := filepath.Join(ca[:n]...)
	if va == "" {
		if filepath.IsAbs(a) && len(ca) > 0 {
			return string(filepath.Separator) + joined
		}
		return joined
	}
	// Re-attach the drive volume with a separator; filepath.Join("C:", "x")
	// would otherwise produce the drive-relative "C:x" instead of "C:\x".
	return va + string(filepath.Separator) + joined
}

// pathComponents splits a path into its non-empty components, accepting both
// '/' and '\' as separators.
func pathComponents(p string) []string {
	p = strings.TrimPrefix(p, filepath.VolumeName(p))
	var comps []string
	for _, part := range strings.FieldsFunc(p, func(r rune) bool { return r == '/' || r == '\\' }) {
		if part != "" && part != "." {
			comps = append(comps, part)
		}
	}
	return comps
}

// firstLine returns the first line of s with surrounding whitespace trimmed.
func firstLine(s string) string {
	s = strings.TrimSpace(s)
	if i := strings.IndexByte(s, '\n'); i >= 0 {
		return s[:i]
	}
	return s
}
