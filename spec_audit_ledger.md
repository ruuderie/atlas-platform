# Specification: Security Audit Ledger

## 1. Overview
In enterprise platform software, the question "Who changed this setting and when?" is paramount. The **Audit Ledger** acts as an immutable, append-only security log for critical operations performed by administrators, users, and the system itself.

## 2. Core Objectives
- Record CRUD operations on sensitive models (Users, Billings, API Keys, Network Settings).
- Provide an unalterable history for internal compliance and external B2B requirements.
- Allow platform admins to trace data compromises backward.

## 3. Architecture & Data Model

### `audit_logs` Table
- `id`: Uuid (Primary Key)
- `tenant_id`: Uuid (For scoped viewing)
- `actor_id`: Uuid (The User/Admin who performed the action, or System if automated)
- `action_type`: String (e.g., `role.updated`, `network.domain.changed`, `apiKey.generated`)
- `entity_type`: String (e.g., `User`, `Network`)
- `entity_id`: Uuid
- `old_state`: JSONB (A snapshot of the data before the action)
- `new_state`: JSONB (A snapshot of the data after the action)
- `ip_address`: String
- `created_at`: DateTime (Timestamp)

### Event Hooking
Audit logging should be integrated natively within the SeaORM active record cycle via Interceptors, or deeply bound into the backend Service layer. Direct database triggers can also be used if strict immutability is required outside of the application scope.

## 4. Platform Admin UI UX
1. **Global Ledger Viewer**: A dedicated panel in the Platform Admin to search logs by `actor_id` or `entity_id`.
2. **Diff Viewer**: A UI modal that compares `old_state` to `new_state` to highlight exactly what was changed (similar to a Git Diff block).
3. **Immutability Constraints**: The UI will not present a "Delete Log" button. Logs are rotated mathematically after 24-36 months based on compliance mandates.
