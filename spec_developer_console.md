# Specification: Developer Console (API & Webhooks)

## 1. Overview
To scale the ecosystem, we need to allow Tenants (Networks and Anchors) to integrate their internal tools, ERPs, or third-party CRMs (like Salesforce/Hubspot) with the Atlas backend. The **Developer Console** provides secure egress and ingress for automated data flow.

## 2. Core Objectives
- Allow platform admins and tenant admins to generate scoped API Tokens.
- Provide a Webhook registry for real-time outbound event notifications.
- Maintain API request logs for debugging and rate limiting.

## 3. Architecture & Data Model

### API Key Management
An authentication middleware that validates Bearer tokens.
- `ApiTokens`:
  - `id`: Uuid
  - `tenant_id`: Uuid
  - `token_hash`: String (Argon2 or SHA256 hashed, never store raw tokens).
  - `scopes`: JSON (e.g., `["listings:read", "users:write"]`)
  - `expires_at`: Option<DateTime>

### Webhook Engine
- `WebhookEndpoints`:
  - `id`: Uuid
  - `tenant_id`: Uuid
  - `target_url`: String
  - `secret_key`: String (For HMAC signing payloads).
  - `subscribed_events`: JSON array (e.g., `["listing.created", "crm.deal.won"]`)
  - `is_active`: Boolean.

### Delivery Pipeline
When a backend module executes an action (e.g., Deal is won in CRM), it emits a domain event. A Rust background worker checks for active `WebhookEndpoints` mapped to `crm.deal.won` for the associated `tenant_id`, signs the payload with the `secret_key`, and executes a `reqwest` HTTP POST with exponential backoff on failures.

## 4. Platform Admin UI UX
1. **API Keys View**: Ability to revoke compromised keys platform-wide.
2. **Webhook Monitor**: A dashboard showing total webhook deliveries, failure rates (HTTP 500s from tenant URLs), and backoff queues.
3. **Usage Limits**: Ability for the Platform Admin to throttle API requests per tenant.
