# Phase 12 Validation: Local Terminal Foundation

## Automated Verification
- **Backend:** `go test ./be/internal/ssh/...` to verify session management logic.
- **Frontend:** `npm run typecheck` and `npm run lint` in `fe/` to verify type safety and code quality.
- **Build:** `go build ./be/...` to ensure no compilation errors after refactoring.

## Manual Verification Checklist
- [ ] **Spawn Local Terminal:**
    - Navigate to "New Tab".
    - Click "Local Terminal".
    - Verify terminal opens and shows a shell prompt.
- [ ] **I/O Forwarding:**
    - Type `ls` and press Enter. Verify output is displayed.
    - Verify `Ctrl+C` works to terminate a running process (e.g., `sleep 10`).
- [ ] **Resizing:**
    - Resize the browser window.
    - Run `stty size` in the local terminal.
    - Verify rows/cols match the expected window size.
- [ ] **Cleanup:**
    - Close the terminal tab.
    - Verify in backend logs (if possible) or via `ps` that the shell process is terminated.
    - Verify backend session is removed after timeout if WebSocket is disconnected.
- [ ] **SSH Regression:**
    - Verify connecting to an existing SSH server still works as expected.
