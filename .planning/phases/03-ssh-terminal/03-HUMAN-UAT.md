---
status: partial
phase: 03-ssh-terminal
source: [03-VERIFICATION.md]
started: 2026-04-27T15:30:00Z
updated: 2026-04-27T15:30:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Live SSH Connection
expected: Connect to real server, run vim/htop/tmux without rendering artifacts
result: [pending]

### 2. Terminal Auto-Resize
expected: Resize browser window while connected — terminal redraws correctly
result: [pending]

### 3. Copy/Paste
expected: Select and copy terminal text via native browser text selection
result: [pending]

### 4. Disconnection & Reconnection
expected: Stop backend, see reconnection overlay, click Reconnect to restore session
result: [pending]

### 5. Quick-Connect Save Banner
expected: Disconnect after quick-connect session, see "Save this connection?" banner
result: [pending]

### 6. Saved Connection Auto-Connect
expected: Click saved connection with stored password — auto-connects without prompt
result: [pending]

### 7. Inline Password Prompt
expected: Click saved connection without stored password — shows inline password prompt
result: [pending]

## Summary

total: 7
passed: 0
issues: 0
pending: 7
skipped: 0
blocked: 0

## Gaps
