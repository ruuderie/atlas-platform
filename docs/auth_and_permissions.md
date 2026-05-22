# Atlas Platform — Authentication & Permissions

> See also:
> - [`docs/atlas_app_integration.md`](atlas_app_integration.md) — how apps register routes and provision tenant data
> - [`docs/platform_overview.md`](platform_overview.md) — system architecture overview

---

## Overview

The Atlas Platform uses a **passwordless-first** authentication model. There are no passwords stored
anywhere in the system. Authentication is handled through two mechanisms:

| Mechanism | Purpose | TTL |
|---|---|---|
| **Passkey (WebAuthn)** | Primary login. Biometric, phishing-proof, origin-bound. | Session |
| **Magic Link** | Recovery only. Single-use email token. | 15 minutes |

**Passwords are not a supported credential type.** Do not add password-based auth to any Atlas app.

---

## The Two-Layer Permission Model

Atlas uses a **two-layer** permission system. Understanding this split is critical before writing
any route handler or auth guard.

```
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Universal TenantRole                          │
│  Stable. 4 values. Never changes as new apps are added. │
│                                                         │
│  PlatformSuperAdmin | Owner | Admin | Member            │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│  Layer 2: App-Specific Permissions                      │
│  Each AtlasApp defines its own permission enum.         │
│  Stored in user_app_permission table (keyed by slug).   │
│                                                         │
│  NetworkPermission | PropertyPermission | FileShare...  │
└─────────────────────────────────────────────────────────┘
```

### Why Two Layers?

If you put app-specific roles directly into `TenantRole`, every new application requires a
database migration on the core auth table. The enum becomes a permanent catch-all that couples
the auth system to individual app business logic. **This is the wrong approach.**

The two-layer model means:
- Adding a new `AtlasApp` requires **zero changes** to core auth code or the `TenantRole` enum
- Each app owns and evolves its own permission vocabulary independently
- `Owner` and `Admin` implicitly hold all permissions in any app — no explicit grants needed

---

## Layer 1: Universal `TenantRole`

Defined in `backend/src/entities/user_account.rs`.

```rust
pub enum TenantRole {
    PlatformSuperAdmin,  // Cross-tenant access. Oply operators ONLY. Never grant to tenants.
    Owner,               // Full control + billing. Can invite admins. One per tenant.
    Admin,               // Full control. Cannot change billing or transfer ownership.
    Member,              // Authenticated user. App-specific permissions govern what they can do.
}
```

**Rules:**
- `PlatformSuperAdmin` must only be granted via the `platform-admin` UI or direct DB seed by Oply.
  It is never available in any self-serve or tenant-facing flow.
- `Owner` is assigned once at tenant provisioning time (see `CorePlatformApp::provision()`).
- An `Admin` or `Owner` has **implicit full access** to all app-level permissions on that tenant.
  Do not write redundant permission checks for these roles — the middleware handles it.
- `Member` is the default for all newly registered users. App permissions must be explicitly granted.

---

## Layer 2: App-Specific Permissions

Each `AtlasApp` declares its own permission enum. These are serialized as strings and stored in
the `user_app_permission` table:

```
user_app_permission
├── user_id    (UUID → user)
├── tenant_id  (UUID → tenant)
├── app_slug   (String, e.g. "network-instance")
└── permissions (JSON array of permission strings)
```

### Defining Permissions for a New App

In your app's module (e.g. `backend/src/atlas_apps/my_app.rs`):

```rust
use std::fmt;

/// All permissions this app understands.
/// Add variants here as features require them — no other files need changing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MyAppPermission {
    ViewContent,
    CreateContent,
    ManageOwnContent,
    ModerateAllContent,
    ManageSettings,
}

impl fmt::Display for MyAppPermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self) // serializes as "ViewContent", "CreateContent", etc.
    }
}

impl MyApp {
    /// Default permissions granted when a Member is added to this app.
    pub fn default_member_permissions() -> Vec<MyAppPermission> {
        vec![MyAppPermission::ViewContent]
    }

    /// Default permissions granted when a user with Admin role uses this app.
    /// Note: Admin/Owner role already bypasses permission checks — this is for
    /// explicit grant seeding only (e.g. so the permission appears in the UI).
    pub fn default_admin_permissions() -> Vec<MyAppPermission> {
        vec![
            MyAppPermission::ViewContent,
            MyAppPermission::CreateContent,
            MyAppPermission::ManageOwnContent,
            MyAppPermission::ModerateAllContent,
            MyAppPermission::ManageSettings,
        ]
    }
}
```

### Checking Permissions in a Route Handler

```rust
use crate::auth::session::UserSession;
use crate::atlas_apps::my_app::MyAppPermission;

pub async fn create_content_handler(
    Extension(session): Extension<UserSession>,
    // ...
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // require_app_permission handles the Owner/Admin bypass automatically
    session.require_app_permission("my-app", MyAppPermission::CreateContent)?;

    // handler logic...
}
```

`UserSession::require_app_permission` internally:
1. If `session.tenant_role` is `Owner` or `Admin` → returns `Ok(())` immediately
2. Looks up the `user_app_permission` row for `(user_id, tenant_id, app_slug)`
3. Checks if the permission string is in the JSON array
4. Returns `Err(401)` if not found

---

## Current App Permission Sets

### `anchor-app` (Tenant CMS)

```rust
pub enum AnchorPermission {
    ViewCms,             // Access /admin at all
    EditContent,         // Edit pages, blog posts
    ManageNav,           // Edit navigation menus
    ManageSettings,      // Change site settings
    ManageUsers,         // Invite / remove other admins
}
```

### `network-instance` (Listings)

```rust
pub enum NetworkPermission {
    ViewListings,           // Browse public listings (also granted to anonymous)
    CreateListing,          // Post a new listing
    ManageOwnListings,      // Edit/delete own listings
    ModerateAllListings,    // Edit/delete any listing
    ManageCategories,       // Create/edit listing categories
}
```

### Future Apps

When implementing a new `AtlasApp`, define its permission enum following the pattern above.
**Do not add variants to existing enums or to `TenantRole`.**

**Property Management (planned):**
- Use `Renter` (not `Tenant`) for the lessee role to avoid collision with the platform's
  use of "tenant" as the business/site entity.
- Expected permissions: `ViewProperties`, `SubmitApplication`, `ManageOwnProperties`,
  `ReviewApplications`, `ApproveApplications`, `ManageMaintenanceRequests`, `AccessFinancials`

**File Sharing / Production Asset Platform (planned):**
- External clients (e.g. film studios reviewing dailies) use **guest share tokens**, not
  platform accounts. See "Guest Access" section below.
- Expected permissions: `Upload`, `CreateProject`, `InviteCollaborators`, `GenerateShareLink`,
  `ReviewOnly`, `DownloadAssets`, `ManageStorage`

---

## Authentication Flows

### Canonical Login UI: `AtlasLoginPanel`

All Atlas platform apps use a single shared login component from `shared-ui`:

```rust
use shared_ui::components::auth::atlas_login_panel::AtlasLoginPanel;

// Minimal usage — just pass an app title:
view! { <AtlasLoginPanel app_title="MY APP" /> }

// With a custom on_authenticated callback (e.g. for modal flows):
view! {
    <AtlasLoginPanel
        app_title="NETWORK"
        on_authenticated=Callback::new(|_| navigate("/dashboard", Default::default()))
    />
}
```

**Do not re-implement the login form in individual apps.** Bug fixes, UX improvements, and
security hardening to the login flow only need to happen in one place.

Mode switching between passkey and magic link is driven by `?mode=email` in the URL — not a
client-side signal. This means the email form is SSR-rendered and visible immediately without
waiting for WASM to hydrate. It also means the URL is shareable and bookmarkable.

---

### Flow 1: Passkey Login (Primary)

Used by: all apps, all user types.

```
1. User visits /login (or /admin for anchor-app)
2. Client calls: POST /api/passkeys/start-login { email?: string }
      └─ email is optional for discoverable credentials (resident keys)
3. Backend returns WebAuthn challenge
4. Browser invokes authenticator (FaceID, TouchID, Windows Hello, hardware key)
      └─ Private key never leaves the device. Origin is cryptographically verified.
5. Client calls: POST /api/passkeys/finish-login { credential }
6. Backend verifies credential against stored public key
7. Backend sets: Set-Cookie: session=<token>; HttpOnly; Secure; SameSite=Strict
8. All subsequent requests send cookie automatically (credentials: include)
```

The session cookie is `HttpOnly` — **it is never accessible to JavaScript**. Do not read
auth state from `document.cookie` or `localStorage` in any app.

### Flow 2: Magic Link (Recovery Only)

Used when: user has no passkey registered on the current device.

```
1. User clicks "I don't have my passkey" / "Send Recovery Link"
2. Client calls: POST /api/auth/magic-link/request { email: string }
      └─ Always returns HTTP 200 regardless of whether email exists (anti-enumeration)
3. If email matches a user: backend processes request through a 3-layer deduplication guard:
      ├─ Layer 1: In-memory same-pod TTL cache (60s) to absorb double-clicks
      ├─ Layer 2: Transaction-scoped PostgreSQL advisory lock on user UUID to prevent multi-pod races
      └─ Layer 3: Pre-upsert cleanup of expired-but-active tokens to prevent index lockout
4. Backend sends email with link to /magic-login?token=<uuid> (token is rotated if active)
5. User clicks link → app calls: POST /api/auth/magic-link/verify { token: string }
6. Backend validates: token exists, not used, not expired (15-min TTL)
7. Backend marks token as used (one-time only), creates session, sets HttpOnly cookie
8. App immediately prompts: "Register a passkey on this device for next time"
      └─ This is non-optional UX — magic link login should bootstrap passkey registration
```

#### Magic Link 3-Layer Deduplication Architecture

To guarantee that magic link emails are sent **once and only once** without locking out users permanently, the platform implements a robust, state-of-the-art three-layer deduplication architecture:

```
┌─────────────────────────────────────────────────────────────┐
│  Layer 1: In-Memory TTL Cache (Same-Pod)                     │
│  - Bounded moka Cache (1000 cap, 60s TTL)                   │
│  - Checked BEFORE DB read/write; committed AFTER txn commit │
│  - Absorbs rapid UI double-clicks / pre-hydration mismatch  │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 2: PostgreSQL Advisory Lock (Cross-Pod / Concurrency)│
│  - Single explicit transaction per user request             │
│  - pg_try_advisory_xact_lock(lock_key)                      │
│  - Stable 64-bit key: XOR of high/low 64-bit halves of UUID │
│  - Releases atomically on COMMIT or ROLLBACK                │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│  Layer 3: Stale Token DB Cleanup                            │
│  - Expired, unused tokens block the partial unique index    │
│  - UPDATE magic_link_token SET is_used = true               │
│    WHERE user_id = $1 AND is_used = false AND expires < NOW │
│  - Prevents permanent user lockouts from stale rows          │
└─────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│  Token Rotation & Upsert                                    │
│  - ON CONFLICT (user_id) WHERE is_used = false DO UPDATE    │
│  - xmax check: returns was_inserted = true/false            │
│  - Rotates active token and sends email on manual retry      │
└─────────────────────────────────────────────────────────────┘
```

##### 1. Layer 1: In-Memory same-pod cache
An in-memory `moka` cache (`max_capacity: 1000`, `ttl: 60s`) acts as a front door. When a request arrives, the pod checks the cache. If present, it returns `200 OK` immediately without accessing the database.
* **Cache Poisoning Prevention:** The cache is not populated until *after* the database transaction successfully commits. A non-existent user or database connection failure will not block legitimate retries.

##### 2. Layer 2: PostgreSQL Advisory Lock
To handle concurrent requests landing on different pod replicas, the backend utilizes PostgreSQL's transaction-scoped advisory locks (`pg_try_advisory_xact_lock`). 
* **Key Derivation:** The per-user 64-bit key is derived by XORing the high and low 64-bit halves of the user's UUID.
* **Automatic Release:** Because the lock is transaction-scoped, it is held *only* during active execution of the token rotation transaction and is released atomically by Postgres upon `COMMIT` or `ROLLBACK`.
* **Lock Contention:** If pod B tries to process a request for user X while pod A holds the lock, the lock acquisition immediately returns `false`. Pod B aborts the transaction, logs a `magic_link.deduplicated_cross_pod` event, and returns `200 OK` (so attackers gain no timing information).

##### 3. Layer 3: Stale Token Cleanup & Rotation
The database uses a partial unique index:
`CREATE UNIQUE INDEX magic_link_token_one_per_user_active ON magic_link_token (user_id) WHERE is_used = false;`
* **Stale Token Cleanup:** If a token expires without being clicked, `is_used` remains `false`, which would block new inserts. Before writing a new token, the transaction executes:
  `UPDATE magic_link_token SET is_used = true WHERE user_id = $1 AND is_used = false AND expires_at < NOW();`
  This cleans up stale tokens, freeing up the partial index predicate.
* **Token Rotation:** If a valid active token already exists, the transaction performs an upsert:
  `ON CONFLICT (user_id) WHERE is_used = false DO UPDATE SET token = EXCLUDED.token...`
  This rotates the token in place for active requests.


### Flow 3: New Tenant Admin Provisioning

Used when: a new tenant is onboarded via platform-admin.

```
1. platform-admin: POST /api/admin/tenants/{id}/provision-admin { email }
      └─ PlatformSuperAdmin role required
2. Backend upserts user (creates if not exists), assigns TenantOwner role
3. Backend generates a 24-hour one-time setup token
4. Backend sends email: "Your site is ready — set up your passkey"
5. Link goes to: anchor-app /setup-passkey?token=<setup_token>
6. App exchanges setup token for a temporary session
7. User registers passkey → setup token consumed → passkey is primary credential
```

---

## Session Validation

All apps validate sessions by calling the backend — never by decoding the cookie client-side.

```
GET /api/auth/session/validate
Cookie: session=<token>

Response:
{
  "valid": true,
  "user": {
    "id": "uuid",
    "email": "user@example.com"
  },
  "tenant_role": "Admin",
  "app_permissions": {
    "network-instance": ["ViewListings", "CreateListing", "ManageOwnListings"]
  }
}
```

The Leptos auth state pattern used in all apps:

```rust
// SSR always renders Pending — no auth-conditional content on first render.
// This prevents hydration mismatches (the root cause of the /admin panic bug, 2026-05-04).
#[derive(Clone, PartialEq)]
pub enum AuthState {
    Pending,              // SSR + initial WASM frame. Render a skeleton/spinner only.
    Unauthenticated,      // Session check complete: no valid session.
    Authenticated(UserSession),  // Session valid. Render protected content.
}
```

> **Critical:** Do NOT create `Resource` or `LocalResource` values inside a `view!` macro
> or inside a match branch that isn't always rendered on the server. This causes hydration
> ID mismatches and panics in `leptos_dom::dyn_child`. Always declare resources at the top
> of the component function body, before the `view!` macro. See commit `07e63163` for the
> canonical fix example.

---

## WebAuthn RP_ID Scoping

Each application domain gets its own WebAuthn Relying Party ID. A passkey registered at
`buildwithruud.com` is cryptographically bound to that origin and **cannot** be used at
`atlas.oply.co`. This is enforced by the browser's WebAuthn implementation — it is not
a software check.

The backend maintains a `WebauthnRegistry` (a `HashMap<String, Arc<Webauthn>>`) resolved from
the `Origin` header of each passkey request:

```
uat.atlas.oply.co         → RP_ID: "uat.atlas.oply.co"       (platform-admin)
uat.buildwithruud.com     → RP_ID: "uat.buildwithruud.com"   (anchor-app tenant)
network.uat.atlas.oply.co → RP_ID: "network.uat.atlas.oply.co" (network-instance)
```

When a new tenant domain is provisioned, a `Webauthn` instance for that RP_ID is registered
in the `WebauthnRegistry`. This happens in `CorePlatformApp::provision()`.

---

## Guest / External Access (Share Tokens)

For apps that need to grant temporary access to external parties who are **not** platform
users (e.g. a film client reviewing project dailies, or a prospective renter viewing a
locked property listing):

- Do not create `user` records for guests.
- Issue a **share token**: a signed, time-bounded, scope-limited token stored in a
  `guest_access_token` table.
- The token encodes: `tenant_id`, `resource_id`, `resource_type`, `permissions[]`, `expires_at`.
- The route guard checks the token, not a session cookie.
- The `GenerateShareLink` (or equivalent) app permission controls who can create tokens.

This is a distinct third tier, separate from both `TenantRole` and `AppPermission`. It is
planned for the file-sharing and property management apps.

---

## What NOT To Do

| ❌ Don't | ✅ Do Instead |
|---|---|
| Store JWT or session tokens in `localStorage` | Use `HttpOnly` server-set cookies |
| Store JWT or session tokens in client-accessible `document.cookie` | Use `HttpOnly` server-set cookies |
| Add app-specific roles to the `TenantRole` enum | Define a new `AppPermission` enum in your app |
| Decode the session cookie client-side | Call `GET /api/auth/session/validate` |
| Create `Resource` inside a `view!` macro | Declare resources at the top of the component body |
| Add a `is_admin: bool` flag to any entity | Use `TenantRole::Admin` via `user_account` |
| Call `is_system_initialized()` for gating UI | It's permanently stubbed `Ok(true)` — remove it |
| Use passwords | This platform does not support passwords. Use passkeys + magic links. |
| Implement a custom login form in a new app | Use `<AtlasLoginPanel>` from `shared-ui` |
| Use a JS signal to toggle login mode | Use `?mode=email` URL param — SSR-safe, bookmarkable |

---

## Security Properties

| Property | Implementation |
|---|---|
| No passwords stored | Passkeys (public key only) + magic links (single-use UUID tokens) |
| Phishing resistance | WebAuthn challenges are origin-bound at the hardware level |
| XSS-safe session storage | `HttpOnly; Secure; SameSite=Strict` cookies — no JS access |
| Token enumeration prevention | Magic link request always returns HTTP 200 |
| One-time magic link tokens | `is_used` flag set on first use; 15-minute expiry |
| Audit trail | `AuditService::log_action()` on all auth events (`auth.magic_link.created`, `auth.magic_link.consumed`, `auth.passkey.registered`, `auth.session.created`) |
| Cross-tenant isolation | `tenant_id` on all resource tables; session claims include tenant context |
| Passkey cross-domain isolation | Per-domain `Webauthn` instances in `WebauthnRegistry` |
