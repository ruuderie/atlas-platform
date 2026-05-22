# Atlas Platform — Auth & Security Observability Runbook

**Version**: 1.1  
**Last Updated**: 2026-05-15  
**Owner**: Platform Team  
**Status**: Production-Ready

---

## 1. Purpose

This runbook provides operational, security, and architectural guidance for the authentication and authorization layer of the Atlas Platform. It covers:

- Structured logging standards
- Key metrics and dashboards
- Alert rules and thresholds
- Common failure scenarios + remediation
- Security playbooks
- On-call checklist

---

## 2. Architecture Overview (May 2026)

### Auth Flow
1. User requests magic link → `request_magic_link`
2. Backend enforces strict double-send prevention & lockout safety via a 3-layer guard:
   - **Layer 1:** Bounded in-memory `MAGIC_LINK_REQUEST_CACHE` (60s TTL) to catch same-pod UI double-clicks.
   - **Layer 2:** PostgreSQL transaction-scoped advisory locks (`pg_try_advisory_xact_lock`) using XOR-derived user UUID keys to isolate concurrent multi-pod requests.
   - **Layer 3:** Active but expired token cleanup (`UPDATE is_used = true WHERE expires_at < NOW()`) within the same transaction to prevent partial unique index lockouts.
3. Token upserted or rotated in place (`ON CONFLICT (user_id) WHERE is_used = false DO UPDATE`) inside the explicit transaction.
4. Email sent via Lettre.
5. User clicks link → `verify_magic_link` (sets HttpOnly + SameSite=Strict session cookie).
6. Optional passkey nudge shown if user has no passkeys.
7. Session created, permissions mapped, and stored securely using SHA-256 token hashing (`bearer_token_hash` and `refresh_token_hash` columns in Postgres).


### Key Components
- **Frontend**: Leptos 0.8 (WASM) + shared-ui
- **Backend**: Axum + SeaORM + PostgreSQL
- **Observability**: Prometheus + Grafana + Loki (via NixForge)
- **Metrics**: Custom Prometheus counters/histograms for auth events

---

## 3. Structured Logging Standard

All auth-related functions **must** emit structured logs with the following fields:

| Field            | Type     | Required | Description |
|------------------|----------|----------|-------------|
| `event`          | string   | Yes      | e.g. `magic_link.requested`, `session.created`, `auth.failed` |
| `user_id`        | uuid     | When known | User identifier |
| `tenant_id`      | uuid     | When known | Tenant context |
| `request_id`     | uuid     | Yes      | Unique per HTTP request for correlation |
| `duration_ms`    | number   | Yes      | Time taken for the operation |
| `ip`             | string   | Yes      | Client IP (from X-Forwarded-For or direct) |
| `user_agent`     | string   | Yes      | Browser / client user agent |
| `status`         | string   | Yes      | `success` / `failed` / `blocked` |
| `reason`         | string   | On failure | Specific failure reason |

**Example**:
```rust
tracing::info!(
    event = "magic_link.requested",
    user_id = %user_id,
    tenant_id = %tenant_id,
    request_id = %request_id,
    duration_ms = start.elapsed().as_millis(),
    ip = %ip,
    status = "success"
);
```

---

## 4. Key Metrics (Prometheus)

| Metric Name                              | Type     | Labels                     | Purpose |
|------------------------------------------|----------|----------------------------|---------|
| `magic_link_requests_total`              | Counter  | `tenant_id`, `status`      | Track volume + success rate |
| `magic_link_duplicates_prevented_total`  | Counter  | `tenant_id`                | Guardrail for Bug B (should stay low) |
| `auth_requests_total`                    | Counter  | `action`, `status`         | Overall auth health |
| `auth_request_duration_seconds`          | Histogram| `action`                   | Latency (P95/P99) |

**Critical Alert Thresholds** (see section 6)

---

## 5. Grafana Dashboard

**Dashboard UID**: `atlas-auth-v1`  
**Title**: "Atlas Platform — Auth & Magic Link Telemetry"

**Recommended Panels**:
- Magic Link Requests per Tenant (timeseries)
- Duplicate Magic Links Prevented (stat + timeseries)
- Auth Success Rate (timeseries)
- Auth Latency P95 (timeseries)
- Magic Link Requests vs Duplicates Prevented (comparison)

---

## 6. Alert Rules (Prometheus)

```yaml
# 1. High duplicate prevention rate (possible abuse or regression)
- alert: MagicLinkDuplicatePreventionHigh
  expr: rate(magic_link_duplicates_prevented_total[10m]) > 5
  for: 5m
  labels:
    severity: warning
    team: platform
  annotations:
    summary: "High rate of duplicate magic link prevention detected"
    description: "{{ $value }} duplicates prevented in last 10 minutes. Investigate possible abuse or bug in idempotency logic."

# 2. Auth success rate drop
- alert: AuthSuccessRateLow
  expr: (sum(rate(auth_requests_total{status="success"}[5m])) / clamp_max(sum(rate(auth_requests_total[5m])), 1)) < 0.95
  for: 5m
  labels:
    severity: critical
    team: platform
  annotations:
    summary: "Auth success rate dropped below 95%"
    description: "Current success rate: {{ $value | humanizePercentage }}. Check logs for common failure reasons."

# 3. High auth latency
- alert: AuthLatencyHigh
  expr: histogram_quantile(0.95, rate(auth_request_duration_seconds_bucket[5m])) > 1.5
  for: 5m
  labels:
    severity: warning
    team: platform
  annotations:
    summary: "P95 auth latency > 1.5s"
    description: "Auth operations are slow. Check database, email provider, or WASM hydration issues."

# 4. Magic link verification failures
- alert: MagicLinkVerificationFailures
  expr: rate(auth_requests_total{action="verify_magic_link",status="failed"}[5m]) > 0.2
  for: 2m
  labels:
    severity: warning
  annotations:
    summary: "High magic link verification failure rate"
    description: "Possible expired tokens, tenant mismatch, or cookie issues."
```

---

## 7. Common Failure Scenarios & Runbooks

### Scenario A: Duplicate Magic Link Emails or Lockout Issues
**Symptoms**: Users receive 2+ magic link emails in quick succession, or report receiving no email and being permanently unable to request a new magic link.
**Checks**:
1. Check `magic_link_duplicates_prevented_total` metric in Prometheus.
2. Search Loki logs for deduplication and lock-contention events:
   - Same-pod cache hits: `event="magic_link.deduplicated"`
   - Cross-pod advisory lock contention: `event="magic_link.deduplicated_cross_pod"` (verify `reason="advisory_lock_contention"`)
3. Verify the partial unique index exists on Postgres:
   `SELECT indexname FROM pg_indexes WHERE tablename = 'magic_link_token';`
4. Check if expired, unused tokens are correctly being cleaned up by querying active expired rows:
   `SELECT id, user_id, expires_at, is_used FROM magic_link_token WHERE is_used = false AND expires_at < NOW();`
   *(Should yield 0 rows under normal operations since they are cleaned up pre-upsert).*
**Remediation**:
- If users are locked out and query 4 returns rows, check that the `UPDATE magic_link_token SET is_used = true` query is executing successfully prior to the upsert in `auth_frontend.rs` inside the database transaction.
- If duplicate emails are sent and Loki shows no `deduplicated` logs, verify that the in-memory `MAGIC_LINK_REQUEST_CACHE` check occurs *before* database writes and that `pg_try_advisory_xact_lock` successfully blocks concurrent transaction attempts.


### Scenario B: Admin Dashboard Unclickable / Hydration Panic
**Symptoms**: Nav items and buttons do nothing after magic-link login.
**Checks**:
1. Browser console for `hydration` or `Effect` errors
2. Check `admin.hydration.panic` logs (if instrumented)
**Remediation**:
- Ensure `LocalResource` + `Effect` pattern from commit `8ab7dc66` is still in place in `admin.rs`
- Force hard reload: `window.location.replace('/admin')`

### Scenario C: Magic Link “Already Used” on First Click
**Symptoms**: Users click magic link and immediately see “expired or already used”.
**Root Cause**: `Set-Cookie` header not reaching browser (most common cause).
**Checks**:
1. Verify response from `/api/auth/magic-link/verify` contains `Set-Cookie` header
2. Check browser DevTools → Application → Cookies
**Remediation**:
- Confirm `session_cookie_header()` helper is used in `verify_magic_link`
- Ensure `SameSite=Strict` + `Secure` flags are present

### Scenario D: High Auth Latency
**Symptoms**: Slow login / magic link verification.
**Checks**:
1. P95 latency in Grafana
2. Database slow query log for `magic_link_token` and `session` tables
3. Email provider (Lettre) latency
**Remediation**:
- Add connection pool tuning
- Consider caching user lookup in `request_magic_link`

---

## 8. Security Playbooks

### Playbook: Suspected Account Takeover via Magic Link Abuse
1. Check `magic_link_requests_total` for unusual spikes from single IP or email domain
2. Look for `magic_link_duplicates_prevented_total` anomalies
3. Temporarily increase rate limit in `RateLimiter`
4. Force password reset + passkey enrollment for affected users
5. Audit `audit_logs` table for the user

### Playbook: Session Hijacking Detection
1. Monitor for sudden `last_accessed_at` jumps from different IPs in `session` table
2. Check `session.verify_integrity()` failures in logs
3. Revoke all sessions for the user via admin tool
4. Force re-authentication with passkey

### Playbook: Passkey Adoption Drop
1. Track passkey registration success rate
2. If < 40% after 7 days of magic-link login → trigger in-app nudge + email
3. Consider policy: “Passkey required after 14 days for admin actions”

---

## 9. On-Call Checklist (Daily)

- [ ] Check Grafana “Atlas Auth” dashboard for anomalies
- [ ] Review `MagicLinkDuplicatePreventionHigh` and `AuthSuccessRateLow` alerts
- [ ] Search Loki for `event="auth.failed"` in last 24h
- [ ] Verify no new hydration panics in browser console (production)
- [ ] Confirm email delivery success rate (Lettre logs)
- [ ] Review rate limiter stats for abuse patterns

---

## 10. Future Enhancements (Roadmap)

- [ ] Distributed tracing (OpenTelemetry) across Leptos → Axum → DB
- [ ] Per-tenant auth SLO dashboards
- [ ] Automated passkey enforcement policy engine
- [ ] Cryptographic audit log chaining
- [ ] Short-lived JWT + refresh token rotation (zero-trust)

---

**End of Runbook** — Update this document whenever auth architecture or logging standards change.

---

## 11. Security Audit Findings (2026-05-15)

Full analysis documented in `security_analysis.md` from session `f55aba1a`. Summary of all 7 findings and their current state:

### ✅ Finding #1 — JWT_SECRET Fallback to Plaintext
**File**: `backend/src/auth.rs`  
**Status**: **Fixed**  
**Resolution**: Changed `unwrap_or_else(|_| "your-secret-key")` to `.expect("JWT_SECRET must be set")` across all 4 call sites. Pod will refuse to start if secret is absent rather than running with a known public key.

### ✅ Finding #5 — Token Prefix Logged (8 chars)
**File**: `backend/src/handlers/auth_frontend.rs`  
**Status**: **Confirmed Resolved** — `token_preview` variable no longer present in file.

### ✅ Finding #6 — Secure Flag Missing on Setup Cookie
**File**: `apps/anchor/src/auth.rs`  
**Status**: **Fixed**  
**Resolution**: Cookie now conditionally sets `; Secure` based on whether host is `localhost` / `127.*`.

### ✅ Finding #7 — Refresh & Bearer Tokens Stored in Plaintext
**File**: `backend/src/handlers/sessions.rs`  
**Status**: **Fixed**  
**Resolution**: 
1. Added SHA-256 hashing to session and refresh token creation and verification processes (`hash_token` function using `sha2` crate).
2. Registered and executed migration `m20260515_000001_hash_session_tokens.rs` to add `bearer_token_hash VARCHAR(64)` and `refresh_token_hash VARCHAR(64)` columns to the `session` table.
3. Created a unique index on `bearer_token_hash` for fast, secure O(1) lookups: `idx_session_bearer_token_hash`.
4. Configured session creation to write both hashed values (dual-write) and plaintext values.
5. Session validation performs a lookup using the SHA-256 hash of the incoming token against the `bearer_token_hash` column.
6. Implemented a zero-downtime, backward-compatible fallback: if no session is matched by hash, the query falls back to matching by the plaintext `bearer_token` specifically where `bearer_token_hash` is NULL, protecting pre-migration active user sessions from being abruptly terminated.
7. Dropping plaintext columns will be scheduled in a future release cycle after active sessions expire.


### 🟡 Finding #2 — Rate Limiter Not Shared Across Replicas
**File**: `backend/src/middleware/rate_limiter.rs`  
**Status**: **Documented** — bypass factor `2×` with current 2 replicas

**Short-term**: Backend `Deployment` MUST be kept at `replicas: 1` until Redis is wired. Comment added to `rate_limiter.rs`.

**Long-term**: Replace `DashMap` stores with Redis `INCR` + `EXPIRE`. Alternatively, configure a Cloudflare WAF rate-limit rule on `/magic-links/request` by `CF-Connecting-IP` (Cloudflare sees the true IP before the pod does — this is the most reliable defense regardless of replica count).

**Prometheus alert to add**:
```yaml
- alert: AuthRateLimiterBypassRisk
  expr: kube_deployment_spec_replicas{deployment="atlas-backend"} > 1
  labels:
    severity: warning
  annotations:
    summary: "Backend replicas > 1 — in-process rate limiter is bypassable"
```

### 🟡 Finding #3 — /metrics Publicly Accessible
**File**: `backend/src/main.rs`  
**Status**: **Fixed** — Bearer token auth added to `metrics_endpoint`

**Operational steps**:
1. Generate a token: `openssl rand -hex 32`
2. Add `METRICS_TOKEN=<value>` to the backend k8s Secret (`kubectl edit secret atlas-backend-secrets -n atlas`).
3. Update the Prometheus scrape config:
```yaml
scrape_configs:
  - job_name: atlas-backend
    bearer_token: <same token>
    static_configs:
      - targets: ['atlas-backend:8000']
```
4. Restart backend pod after the secret is updated.
5. Verify: `curl -H "Authorization: Bearer <token>" https://api.buildwithruud.com/metrics` — must return `200`.
6. Verify: `curl https://api.buildwithruud.com/metrics` — must return `401`.

### 🟢 Finding #4 — No Rate Limit on /magic-links/verify
**File**: `backend/src/handlers/magic_links.rs`  
**Status**: **Fixed** — General IP rate limiter (100/min) now applied to verify endpoint

The per-IP limit uses the existing `RateLimiter` Extension. An `x-forwarded-for` IP is extracted and checked before token verification. Bulk verification attempts will now generate `verify.rate_limited` log events and `429` responses observable in Loki and Prometheus.

---

**End of Runbook** — Update this document whenever auth architecture or logging standards change.
