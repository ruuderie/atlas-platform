# Atlas Platform — Auth & Security Observability Runbook

**Version**: 1.0  
**Last Updated**: 2026-05-13  
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
2. Backend enforces idempotency via PostgreSQL partial unique index + `ON CONFLICT DO NOTHING`
3. Email sent via Lettre
4. User clicks link → `verify_magic_link` (sets HttpOnly + SameSite=Strict session cookie)
5. Optional passkey nudge shown if user has no passkeys
6. Session created with JWT + refresh token

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

### Scenario A: Duplicate Magic Link Emails (Regression of Bug B)
**Symptoms**: Users receive 2+ magic link emails in quick succession.
**Checks**:
1. Check `magic_link_duplicates_prevented_total` metric
2. Search logs for `event="magic_link.requested"` + `status="blocked"`
3. Verify partial unique index still exists: `SELECT indexname FROM pg_indexes WHERE tablename = 'magic_link_token';`
**Remediation**:
- If metric is high → investigate recent code changes to `request_magic_link`
- If index missing → re-apply migration `m20260513_000002_unique_active_magic_link_per_user`

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
