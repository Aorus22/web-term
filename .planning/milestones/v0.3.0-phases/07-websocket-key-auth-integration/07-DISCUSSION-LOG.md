# Phase 7: WebSocket Key Auth Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-28
**Phase:** 07-websocket-key-auth-integration
**Areas discussed:** Passphrase Handshake Flow, Reconnection & Passphrase Caching, Quick-connect with Key Auth

---

## Passphrase Handshake Flow

| Option | Description | Selected |
|--------|-------------|----------|
| Frontend-initiated (single-pass) | Frontend checks has_passphrase from key metadata, prompts upfront, sends passphrase with connect message. No WebSocket protocol change needed. | ✓ |
| Backend-driven (two-step) | Client sends connect → backend sends 'passphrase_required' → frontend prompts → sends passphrase back. Adds new WebSocket message type. | |
| You decide | Planner picks the best approach. | |

**User's choice:** Frontend-initiated (single-pass)
**Notes:** Simpler approach, no need for multi-step WebSocket handshake.

### Passphrase Prompt Design

| Option | Description | Selected |
|--------|-------------|----------|
| Reuse PasswordPrompt | Add authType prop. Same inline form, different text. | |
| New PassphrasePrompt | Dedicated component with key-specific icon and text like "Enter passphrase for [key name]". Shows which key needs unlocking. | ✓ |
| You decide | Planner picks. | |

**User's choice:** New PassphrasePrompt
**Notes:** Better UX — shows key-specific context (key name, type).

### Connect Message Format

| Option | Description | Selected |
|--------|-------------|----------|
| Add auth_method + ssh_key_id + passphrase | Extend ConnectMessage with three new fields. Backward-compatible — password connections omit or default to "password". | ✓ |
| You decide | Planner decides exact format. | |

**User's choice:** Add auth_method + ssh_key_id + passphrase
**Notes:** Recommended approach for backward compatibility.

---

## Reconnection & Passphrase Caching

### Caching Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Prompt every time | No caching. Most secure, worst UX. | |
| Cache in memory | Keep passphrase in JS ref during tab lifetime. Cleared on tab close/refresh. Never persisted to disk. | ✓ |
| Cache with timeout | Cache in memory but expire after 15-30 minutes. Balance of security and UX. | |
| You decide | Planner picks. | |

**User's choice:** Cache in memory
**Notes:** Right balance — never stored (persisted), but convenient for reconnects.

### Cache Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Per-session (tab) | Each TerminalPane caches its own passphrase in a ref. Independent per tab. | ✓ |
| Global (app-level) | One cache in Zustand store. All tabs share it. Passphrase lives longer in memory. | |
| You decide | Planner picks. | |

**User's choice:** Per-session (tab)
**Notes:** Better isolation — each tab manages its own lifecycle.

---

## Quick-connect with Key Auth

| Option | Description | Selected |
|--------|-------------|----------|
| Key auth for saved connections only | Quick-connect remains password-only. Key auth only via saved connections (click card on Hosts page). | ✓ |
| Add key selector to quick-connect | Quick-connect bar gets key selector dropdown. More flexible but adds complexity. | |
| You decide | Planner picks. | |

**User's choice:** Key auth for saved connections only
**Notes:** Keeps quick-connect simple. Key auth is a saved-connection feature.

---

## Agent's Discretion

- Exact error messages for key auth failures
- How TerminalPane determines if passphrase is needed
- Exact PassphrasePrompt styling and layout
- Edge case handling (key deleted between save and connect)

## Deferred Ideas

None — discussion stayed within phase scope.
