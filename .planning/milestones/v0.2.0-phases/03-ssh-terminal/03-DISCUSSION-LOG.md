# Phase 3: SSH Terminal - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-27
**Phase:** 03-ssh-terminal
**Areas discussed:** Connection Flow & Password Handling

---

## Connection Flow & Password Handling

### Password prompt location (saved connections without password)

| Option | Description | Selected |
|--------|-------------|----------|
| Inline prompt | Small inline prompt in terminal pane area before connecting. No modals. | ✓ |
| Always prompt via modal | Modal/sheet dialog for password entry every time | |
| Prompt in terminal | SSH server's own prompt appears in terminal via keyboard-interactive | |

**User's choice:** Inline prompt (Recommended)
**Notes:** Clean single flow, no modals. Terminal pane is the interaction zone.

### Quick-connect password collection

| Option | Description | Selected |
|--------|-------------|----------|
| Prompt then connect | Type user@host:port → Enter → inline password prompt → connects | ✓ |
| Inline mini-form | Quick-connect bar expands into mini-form with all fields | |

**User's choice:** Prompt then connect (Recommended)
**Notes:** Password ephemeral, never stored unless user saves later.

### Keyboard-interactive SSH prompts (TERM-03)

| Option | Description | Selected |
|--------|-------------|----------|
| Render in terminal | SSH challenges appear as typed input in terminal, masked with asterisks | ✓ |
| Intercept as control messages | Backend intercepts, sends to frontend as WebSocket control messages | |

**User's choice:** Render in terminal (Recommended)
**Notes:** Simple, authentic terminal experience. No special frontend handling.

### Error display

| Option | Description | Selected |
|--------|-------------|----------|
| In terminal pane | SSH error + Retry button shown in terminal pane | ✓ |
| Toast notifications | Errors as toast at top, terminal stays empty | |
| Dedicated error screen | Structured error card with retry/back buttons | |

**User's choice:** In terminal pane (Recommended)
**Notes:** Terminal pane is the feedback zone for all connection states.

### Saved connection with stored password behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Immediate connect | Click → connect immediately, show "Connecting..." state | ✓ |
| Confirm then connect | Show connection details + Connect button first | |

**User's choice:** Immediate connect (Recommended)
**Notes:** Prioritize speed — saved connections should be one click.

### Save quick-connect session after disconnect

| Option | Description | Selected |
|--------|-------------|----------|
| Post-disconnect banner | Small "Save this connection?" banner with Save/Dismiss | ✓ |
| Never prompt | User manually saves via sidebar | |
| Always-visible save button | Save button in terminal header/tab bar | |

**User's choice:** Post-disconnect banner (Recommended)
**Notes:** Non-intrusive nudge. Easy to ignore.

### Password memory in backend

| Option | Description | Selected |
|--------|-------------|----------|
| Session-only | Password in memory only during WebSocket session, gone after | ✓ |
| Short-lived cache | Cache decrypted password for e.g. 5 minutes | |
| You decide | Agent decides technical detail | |

**User's choice:** Session-only (Recommended)
**Notes:** Security-first approach. Reconnection re-fetches from encrypted storage or re-prompts.

---

## Agent's Discretion

- Session lifecycle & reconnection behavior (auto-reconnect vs manual, overlay design)
- Terminal pane layout & interaction (tab bar, disconnect button placement, state visuals)
- WebSocket protocol extensions beyond spike's connect/resize/binary
- Frontend component architecture (hooks, context, component breakdown)
- Backend SSH session management (goroutine lifecycle, concurrent sessions)
- Terminal theme and color configuration
- Animation/transition timing for connecting states

## Deferred Ideas

None — discussion stayed within phase scope.
