# Security Audit Report — Credential & Token Leakage Risks (Antigravity Agent)

**Project**: antigravity-agent (React + TypeScript + Tauri/Rust)

**Audit focus**: Account credentials & authentication token safety for Windows automated account switching.

**Audit method (static)**:
- Repository-wide secret/token keyword scan
- Review of local storage paths and serialization formats
- Review of network request call sites and Tauri HTTP allowlist
- Review of logging/telemetry paths
- Review of dependency sources + lockfiles

**Limitations**:
- This is a source review; it does not prove properties of already-built binaries.
- No dynamic traffic capture was performed.

---

## Executive summary

**Overall assessment**: **Generally safe from intentional exfiltration based on current source**, but there are **real local token exposure risks by design** (account switching requires storing session state). Two concrete weaknesses were identified and remediated in this ticket:

1. **Hardcoded Google OAuth client credentials in frontend source** → **Removed**; now requires build-time environment variables.
2. **Weak export encryption (XOR + Base64)** → **Replaced** with **AES-256-GCM + PBKDF2-SHA256** (with salt + nonce), with backward-compatible legacy decrypt.

**Remaining important risk (design-level)**:
- **Local account backup files contain token-bearing session state and are stored unencrypted** under the user profile directory. This is vulnerable to local malware and same-user compromise.

**Risk rating (post-fix)**: **Medium**
- Medium not because of network exfiltration (none found), but because local token-at-rest exposure is significant for the threat model.

---

## 1) Sensitive data storage & handling

### 1.1 Local account data (tokens) stored on disk

**What is stored**:
- The application backs up `jetskiStateSync.agentManagerInitState` (Base64-encoded protobuf) from Antigravity’s SQLite state into files named `{email}.json`.

**Where**:
- `~/.antigravity-agent/antigravity-accounts/*.json` (Windows example: `%USERPROFILE%\.antigravity-agent\antigravity-accounts\`)

**Code path**:
- `src-tauri/src/commands/account_commands.rs`: `save_antigravity_current_account`

**Why this matters**:
- The decoded protobuf contains fields like `auth.access_token` / `auth.id_token` (see `src-tauri/src/antigravity/account.rs`). Storing the raw state effectively stores authentication material.

**Risk**:
- **High local-impact**: any malware or process running as the same user can read these files and potentially reuse tokens.
- This is not “plaintext username/password”, but it is equivalent to session credentials.

**Recommendations** (not implemented in this ticket):
1. **Encrypt account backups at rest** using a platform keystore:
   - Windows: DPAPI (`CryptProtectData`) or Windows Credential Manager
   - macOS: Keychain
   - Linux: Secret Service / libsecret
2. If a platform keystore is not available, offer an **optional master password** that must be entered to unlock backups (trade-off: UX).
3. Ensure backups are stored in a directory with restrictive permissions where possible.

### 1.2 Import/export encryption

**Previous behavior (risk)**:
- Export encryption used XOR with the user password + Base64. This is not cryptographically secure and is vulnerable to known-plaintext and brute-force attacks.

**Remediation implemented**:
- `src-tauri/src/commands/account_manage_commands.rs`
  - Export: **AES-256-GCM**
  - Key derivation: **PBKDF2-HMAC-SHA256**, 210,000 iterations, random 16-byte salt
  - Random 12-byte nonce per encryption
  - Output format: JSON envelope `{ v, kdf, iter, salt, nonce, ciphertext }`
  - Import: supports **v2 format**, and falls back to **legacy XOR** for backward compatibility.

**Residual risk**:
- If the user chooses a weak export password, the file can still be brute-forced.

### 1.3 Hardcoded secrets in code

**Finding (risk)**:
- A Google OAuth `client_secret` was hardcoded in frontend source.

**Remediation implemented**:
- `src/services/cloudcode-api.ts` now reads:
  - `VITE_GOOGLE_OAUTH_CLIENT_ID`
  - `VITE_GOOGLE_OAUTH_CLIENT_SECRET`
  from environment at build time.
- Added `.env.example` and README guidance.

**Important note**:
- For distributed desktop apps, a `client_secret` embedded at build time cannot be kept truly secret. Prefer OAuth public-client flows (PKCE) to avoid secrets entirely.

---

## 2) Data transmission & external communication

### 2.1 External domains contacted

**Direct HTTP usage found**:
- Frontend HTTP calls are centralized in `src/services/cloudcode-api.ts` using `@tauri-apps/plugin-http`.

**Tauri HTTP allowlist** (prevents undocumented domains):
- `src-tauri/tauri.conf.json` includes an allowlist:
  - `https://daily-cloudcode-pa.sandbox.googleapis.com`
  - `https://oauth2.googleapis.com/token`
  - `https://www.googleapis.com/oauth2/v2/userinfo`

**Updater**:
- `tauri-plugin-updater` points to:
  - `https://github.com/MonchiLin/antigravity-agent/releases/latest/download/latest.json`

### 2.2 HTTPS verification

- All external requests are HTTPS.
- Development server uses `http://localhost:1420` (dev-only).

### 2.3 Request/response bodies

- Authorization uses `Authorization: Bearer <token>` headers for Google/CloudCode.
- Token refresh posts to the OAuth token endpoint with standard `application/x-www-form-urlencoded` body.

**No evidence found** of sending credentials/tokens to any non-whitelisted domain.

---

## 3) Potential data exfiltration vectors

### 3.1 Hidden/undocumented network calls

- No additional HTTP clients (`reqwest`, raw sockets, DNS lookups) were found in Rust code.
- No other `fetch()` usage besides `cloudcode-api.ts`.

### 3.2 Logging/telemetry

- Backend uses `tracing` with a **sanitizing file writer** (`src-tauri/src/utils/log_sanitizer.rs` + sanitizing layer).
- Frontend has a logger that forwards logs to backend; backend logs are sanitized before being written to file.

**Risk note**:
- Console logging is not sanitized (useful for dev). If devtools is opened, sensitive data *could* be inspected in-memory regardless of logging.

### 3.3 Error handling

- Errors are generally local and not sent to third parties.
- No crash reporting/telemetry SDKs were found.

---

## 4) Runtime behavior analysis (Windows binary)

**Not performed here** (source-only review). Recommended verification steps for release binaries:
- Run `strings` on the `.exe` and check for unexpected domains/IPs.
- Monitor network traffic with Fiddler/Wireshark and validate only allowlisted domains are contacted.
- Verify Tauri updater signature/public key is unchanged.

---

## 5) Third-party dependencies

- JavaScript dependencies are pinned via `package-lock.json`.
- `npm-audit-results.json` indicates no known vulnerabilities at the time of the report.
- Rust dependencies in `src-tauri/Cargo.toml` are mainstream; however, you should run `cargo audit` in CI when possible.

---

## Remediation summary

### Implemented in this ticket
- Remove hardcoded Google OAuth credentials from source.
- Replace export encryption with AES-256-GCM + PBKDF2-SHA256, salt + nonce.
- Maintain backward compatibility for old exports during import.

### Recommended next steps (not implemented)
1. **Encrypt local account backup files at rest** (platform keystore / DPAPI).
2. Consider removing `client_secret` usage entirely by migrating to OAuth PKCE/public-client flow.
3. Add an automated secret scan (e.g., gitleaks) to CI.
4. Consider a CSP configuration in Tauri security settings (defense-in-depth).

---

## “Can the author steal my credentials/tokens?” — confidence assessment

**What the current code could do**:
- The application necessarily reads and stores token-bearing session state to support account switching.

**Evidence of intentional exfiltration in this repo**:
- **None found**: outbound network is constrained by Tauri allowlist and code only calls Google/CloudCode + updater.

**Confidence level (based on source review)**: **High (≈8/10)**
- High that the *current source* does not intentionally exfiltrate credentials.
- Not 10/10 because any maintainer could ship a modified binary with exfiltration; users should rely on reproducible builds, signed releases, and traffic monitoring if threat model is strong.
