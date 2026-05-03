# Phase 5: Backend SSH Key Storage - Discussion Log

**Date:** 2026-04-28

---

## Gray Areas Discussed

All 5 gray areas were selected for discussion:

1. SSH Key Model Structure
2. Encryption Domain Separation
3. API Design
4. Connection Schema Migration
5. Key Validation on Upload

---

## Discussion Details

### Area 1: SSH Key Model Structure
**Question:** What fields should the SSHKey model have?  
**Options presented:** Minimal, With Metadata, Full Parse  
**Selected:** Full Parse (add public_key, comment, key_size)  
**Notes:** Store all extractable metadata at upload time so UI can display without decryption

**Question:** For key type storage - use Go's crypto library type names or standard naming?  
**Selected:** Standard names (RSA, Ed25519, ECDSA)  
**Notes:** User-friendly in API/UI responses

---

### Area 2: Encryption Domain Separation
**Question:** How to separate key encryption from password encryption?  
**Options presented:** AAD domain strings, Separate derived keys, Both  
**Selected:** AAD domain strings (password:aes, key:aes)  
**Notes:** Simplest approach - same master key with different AAD context strings: "webterm:password:v1" and "webterm:sshkey:v1"

---

### Area 3: API Design
**Question:** Upload endpoint - how should clients send the private key?  
**Options presented:** Multipart form, Base64 in JSON, Raw PEM  
**Selected:** Base64 in JSON  
**Notes:** Text-safe, works with standard JSON APIs

**Question:** List keys endpoint - what to return?  
**Options presented:** Full metadata only, Include encrypted blob  
**Selected:** Full metadata only (no encrypted blob)  
**Notes:** Encrypted key never leaves backend - security requirement

**Question:** REST endpoint structure - which pattern?  
**Options presented:** Flat, Nested versioned, RESTful nested  
**Selected:** Flat (/api/keys)  
**Notes:** Consistent with existing /api/connections pattern

---

### Area 4: Connection Schema Migration
**Question:** Connection schema - how to add auth_method?  
**Options presented:** String enum, Integer enum, Boolean  
**Selected:** String enum ('password', 'key')  
**Notes:** Clear, extensible, validated in Go

**Question:** ssh_key_id relationship - foreign key constraint or loose reference?  
**Options presented:** Foreign key with constraint, Loose string reference  
**Selected:** Foreign key with constraint  
**Notes:** Database-enforced integrity

**Question:** Foreign key ON DELETE behavior?  
**Options presented:** RESTRICT, SET NULL, CASCADE  
**Selected:** SET NULL (clear ssh_key_id, auth_method stays 'key')  
**Notes:** Connection survives key deletion but becomes broken state (user must update)

---

### Area 5: Key Validation on Upload
**Question:** Validate key format on upload?  
**Options presented:** Full parse, PEM block only, No validation  
**Selected:** Full parse attempt with crypto library  
**Notes:** Actually parse with ssh.ParseRawPrivateKey to detect key type and passphrase

---

## Decisions Summary

| # | Decision | Value |
|---|----------|-------|
| D-01 | SSHKey model fields | id, name, encrypted_key, fingerprint, key_type, has_passphrase, created_at, updated_at |
| D-02 | Metadata extraction | Store at upload time (fingerprint, key_type, has_passphrase) |
| D-03 | Key type storage | Standard names (RSA, Ed25519, ECDSA) |
| D-04 | Full parse | Extract public_key, comment, key_size where available |
| D-05 | AAD domain separation | "webterm:password:v1" and "webterm:sshkey:v1" |
| D-06 | Encryption approach | Same master key, different AAD strings |
| D-07 | Key derivation | Not used - AAD separation sufficient |
| D-08 | Endpoint structure | Flat /api/keys |
| D-09 | Upload encoding | Base64 in JSON body |
| D-10 | List response | Metadata only, no encrypted blob |
| D-11 | Response fields | id, name, fingerprint, key_type, has_passphrase, timestamps |
| D-12 | auth_method type | String enum |
| D-13 | auth_method values | "password" | "key" |
| D-14 | Foreign key | Yes, with constraint |
| D-15 | ON DELETE | SET NULL |
| D-16 | Default auth_method | "password" for existing connections |
| D-17 | Migration safety | AutoMigrate, no data loss |
| D-18 | Key validation | Full parse with crypto library |
| D-19 | Validation scope | Detect type, fingerprint, passphrase status |
| D-20 | Invalid key handling | 400 Bad Request with clear message |

## Deferred Ideas

| Idea | Target Phase |
|------|--------------|
| SSH key export/download (encrypted blob) | v0.4.0 |
| PuTTY PPK key format support | Backlog |
| Key generation in-browser | Out of scope |
| Public key deployment to server | Out of scope |

---

*End of discussion log*
