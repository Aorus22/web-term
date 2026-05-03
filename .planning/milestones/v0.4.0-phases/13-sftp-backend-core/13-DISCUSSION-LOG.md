# Phase 13: SFTP Backend Core - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-03
**Phase:** 13-sftp-backend-core
**Areas discussed:** SFTP Connection Strategy, Filesystem Access Policy, Data Transfer Pattern, Cross-Platform Support

---

## SFTP Connection Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse Session | Share existing ssh.Client with terminal tabs. | |
| Standalone | Open a separate SSH connection for SFTP operations. | ✓ |

**User's choice:** standalone.
**Notes:** User wants independence from terminal tab lifecycle.

---

## Filesystem Access Policy

| Option | Description | Selected |
|--------|-------------|----------|
| Restricted | Jail access to a specific directory (e.g., $HOME). | |
| Full Access | Expose the entire filesystem (restricted by OS user perms). | ✓ |

**User's choice:** full access.
**Notes:** WebTerm is intended as a sysadmin tool; jailing is seen as an unnecessary restriction.

---

## Data Transfer Pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Multipart | Standard HTTP multipart/form-data. | |
| Streaming | Streamed I/O using io.Pipe or similar. | ✓ |

**User's choice:** streaming.
**Notes:** Better for large files and memory efficiency.

---

## Cross-Platform Support

| Option | Description | Selected |
|--------|-------------|----------|
| Linux-First | Focus on Linux/macOS, use Linux-specific features. | |
| Cross-Platform | Support Windows and Linux equally from the start. | ✓ |

**User's choice:** crossplatform.
**Notes:** Core backend logic should be compatible across major OSs.

---

## Claude's Discretion

- Choice of specific Go libraries for streaming if `pkg/sftp` or `io` need supplements.
- Implementation details of the `FileSystem` interface.
- Error code mapping.

## Deferred Ideas

- UI Layout (Phase 14)
- Drag-and-drop transfers (Phase 15)

---

*Phase: 13-sftp-backend-core*
*Discussion log generated: 2026-05-03*
