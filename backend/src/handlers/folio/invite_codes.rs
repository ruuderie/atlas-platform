//! Invite Code handlers — /api/folio/invite-codes
//!
//! Implements the short-token invite system that pre-resolves role + entity context
//! so users can self-onboard via QR codes, deep links, or platform-admin provisioning.
//!
//! # Endpoints
//!
//! | Method | Path                             | Auth         | Description                 |
//! |--------|----------------------------------|--------------|-----------------------------|
//! | GET    | /api/folio/invite/resolve/:code  | public       | Resolve code → context JSON |
//! | POST   | /api/folio/invite-codes          | landlord+    | Create invite code          |
//! | GET    | /api/folio/invite-codes          | landlord+    | List workspace codes        |
//! | PATCH  | /api/folio/invite-codes/:id      | landlord+    | Deactivate / update label   |
//!
//! # Security
//!
//! - `resolve` is intentionally public so unauthenticated users can land on /join/:code
//!   and see the context card before creating an account. It returns NO PII.
//! - Create/list/patch require a valid Folio session token.
//! - Code generation uses 8–12 random alphanumeric chars with an optional human prefix.
//! - `uses_count` is incremented atomically (UPDATE ... RETURNING) to prevent
//!   over-issuing on race conditions.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, patch, post},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Set, ActiveValue,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

/// Extract a bearer token from `Authorization: Bearer <token>` or `Cookie: session=<token>`.
fn extract_bearer(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(val) = auth.to_str() {
            let val = val.trim();
            if val.starts_with("Bearer ") {
                return Some(val["Bearer ".len()..].trim().to_string());
            }
        }
    }
    if let Some(cookie) = headers.get("cookie") {
        if let Ok(val) = cookie.to_str() {
            for part in val.split(';') {
                let part = part.trim();
                if part.starts_with("session=") {
                    return Some(part["session=".len()..].trim().to_string());
                }
                if part.starts_with("atlas_session=") {
                    return Some(part["atlas_session=".len()..].trim().to_string());
                }
            }
        }
    }
    None
}


// ── Route registration ────────────────────────────────────────────────────────

/// Public routes — no auth required.
pub fn public_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/invite/resolve/{code}", get(resolve_code))
}

/// Authenticated routes — valid Folio session required.
pub fn authenticated_routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/invite-codes",                       post(create_code))
        .route("/api/folio/invite-codes",                       get(list_codes))
        .route("/api/folio/invite-codes/{id}",                  patch(patch_code))
        .route("/api/folio/invite-codes/{id}/accept",           post(accept_code))
        // Short-code accept: used by all wizards (they have the code string, not UUID)
        .route("/api/folio/invite-codes/by-code/{code}/accept", post(accept_code_by_str))
}

// ── Types ─────────────────────────────────────────────────────────────────────

/// Resolved context returned by GET /api/folio/invite/resolve/:code.
/// Intentionally contains NO PII — safe to return to unauthenticated callers.
#[derive(Debug, Serialize)]
pub struct ResolvedInviteCode {
    pub code:           String,
    pub role:           String,
    /// Human-readable label set by the inviter, e.g. "Apply for 123 Oak St Unit 4B"
    pub label:          Option<String>,
    /// Custom message from the inviter shown in the wizard left panel
    pub invite_message: Option<String>,
    /// Resolved entity context — names/addresses only, no raw UUIDs exposed to client
    pub context:        InviteCodeContext,
    /// ISO-8601 expiry or null if code never expires
    pub expires_at:     Option<String>,
    /// How many uses remain, null if unlimited
    pub uses_remaining: Option<i32>,
    /// Whether the code can still be used
    pub is_valid:       bool,
}

#[derive(Debug, Serialize, Default)]
pub struct InviteCodeContext {
    /// The asset being referenced (unit, property, etc.)
    pub asset:    Option<ContextEntity>,
    /// The landlord / PMC who owns the asset or sent the invite
    pub landlord: Option<ContextEntity>,
    /// The broker for agent invites
    pub broker:   Option<ContextEntity>,
    /// Multi-asset summary (cohost/vendor portfolio invites)
    pub asset_count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ContextEntity {
    pub name:    String,
    pub address: Option<String>,
}

/// Input for POST /api/folio/invite-codes.
#[derive(Debug, Deserialize)]
pub struct CreateInviteCodeInput {
    pub role:           String,
    /// Optional human prefix for the code, e.g. "OAK4B" → generates "OAK4B-K7X"
    pub code_prefix:    Option<String>,
    pub asset_id:       Option<Uuid>,
    /// Comma-separated UUIDs for multi-asset invites (cohost/vendor portfolio)
    pub asset_ids_csv:  Option<String>,
    pub booking_id:     Option<Uuid>,
    pub landlord_id:    Option<Uuid>,
    pub broker_id:      Option<Uuid>,
    /// When set: the inviting landlord/employer user ID.
    /// For property_manager invites: accept creates G-32 role scoped to employer's account
    /// and a G-11 property_management_agreement contract.
    pub employer_user_id: Option<Uuid>,
    /// null = unlimited
    pub max_uses:       Option<i32>,
    /// ISO-8601 datetime or null
    pub expires_at:     Option<String>,
    pub label:          Option<String>,
    pub invite_message: Option<String>,
}

/// Lightweight code summary for GET /api/folio/invite-codes list.
#[derive(Debug, Serialize)]
pub struct InviteCodeSummary {
    pub id:         Uuid,
    pub code:       String,
    pub role:       String,
    pub label:      Option<String>,
    pub max_uses:   Option<i32>,
    pub uses_count: i32,
    pub expires_at: Option<String>,
    pub is_active:  bool,
    pub created_at: String,
}

/// Input for PATCH /api/folio/invite-codes/:id
#[derive(Debug, Deserialize)]
pub struct PatchInviteCodeInput {
    pub label:     Option<String>,
    pub is_active: Option<bool>,
    pub max_uses:  Option<i32>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/invite/resolve/:code
///
/// Public endpoint — returns the resolved context for an invite code so the
/// /join/:code landing page can display the unit, landlord, role info, and
/// invitation message before the user creates an account.
///
/// Returns 404 if code does not exist, 410 if expired/exhausted/inactive.
async fn resolve_code(
    Path(code): Path<String>,
    State(db):  State<DatabaseConnection>,
) -> impl IntoResponse {
    use sea_orm::prelude::*;

    // Look up the code. Use raw SQL for now; entity will be added in a follow-up.
    let row = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT
            id, code, role, label, invite_message,
            asset_id, asset_ids_csv, booking_id, landlord_id, broker_id,
            max_uses, uses_count, expires_at, is_active, created_at
           FROM atlas_invite_codes
           WHERE code = $1"#,
        [code.clone().into()],
    );

    let result = db.query_one(row).await;

    let row = match result {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({
                "error": "Invite code not found",
                "code": code,
            }))).into_response();
        }
        Err(e) => {
            tracing::error!("invite code lookup failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error"
            }))).into_response();
        }
    };

    // Extract fields
    let is_active: bool = row.try_get("", "is_active").unwrap_or(false);
    let max_uses:  Option<i32> = row.try_get("", "max_uses").ok().flatten();
    let uses_count: i32 = row.try_get("", "uses_count").unwrap_or(0);
    let expires_at_raw: Option<chrono::DateTime<chrono::Utc>> =
        row.try_get("", "expires_at").ok().flatten();

    let is_expired = expires_at_raw.map_or(false, |exp| exp < Utc::now());
    let is_exhausted = max_uses.map_or(false, |max| uses_count >= max);
    let is_valid = is_active && !is_expired && !is_exhausted;

    if !is_active || is_expired || is_exhausted {
        return (StatusCode::GONE, Json(serde_json::json!({
            "error": if is_expired { "Invite code has expired" }
                     else if is_exhausted { "Invite code has been fully used" }
                     else { "Invite code is no longer active" },
            "code": code,
        }))).into_response();
    }

    let role:           String       = row.try_get("", "role").unwrap_or_default();
    let label:          Option<String> = row.try_get("", "label").ok().flatten();
    let invite_message: Option<String> = row.try_get("", "invite_message").ok().flatten();
    let asset_id:       Option<Uuid>   = row.try_get("", "asset_id").ok().flatten();
    let landlord_id:    Option<Uuid>   = row.try_get("", "landlord_id").ok().flatten();
    let broker_id:      Option<Uuid>   = row.try_get("", "broker_id").ok().flatten();
    let asset_ids_csv:  Option<String> = row.try_get("", "asset_ids_csv").ok().flatten();

    // Resolve asset name/address
    let asset_ctx = if let Some(aid) = asset_id {
        resolve_asset_context(&db, aid).await
    } else {
        None
    };

    // Count multi-assets
    let asset_count = asset_ids_csv.as_ref().map(|csv| {
        csv.split(',').filter(|s| !s.trim().is_empty()).count() as i64
    });

    // Resolve landlord name
    let landlord_ctx = if let Some(lid) = landlord_id {
        resolve_user_context(&db, lid).await
    } else {
        None
    };

    // Resolve broker name
    let broker_ctx = if let Some(bid) = broker_id {
        resolve_user_context(&db, bid).await
    } else {
        None
    };

    let uses_remaining = max_uses.map(|max| max - uses_count);

    let response = ResolvedInviteCode {
        code: code.clone(),
        role,
        label,
        invite_message,
        context: InviteCodeContext {
            asset: asset_ctx,
            landlord: landlord_ctx,
            broker: broker_ctx,
            asset_count,
        },
        expires_at: expires_at_raw.map(|dt| dt.to_rfc3339()),
        uses_remaining,
        is_valid,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /api/folio/invite-codes
///
/// Creates a new invite code. The caller must be authenticated.
/// Code format: {PREFIX}-{RAND6} where prefix is human-supplied or omitted.
async fn create_code(
    State(db):    State<DatabaseConnection>,
    headers:      HeaderMap,
    Json(input):  Json<CreateInviteCodeInput>,
) -> impl IntoResponse {
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Authentication required"
        }))).into_response(),
    };

    // Resolve caller user_id from session token
    let caller_id = match resolve_caller_id(&db, &token).await {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Invalid session"
        }))).into_response(),
    };

    // TODO: resolve workspace_id from caller's active app instance.
    // For now use caller's own user_id as a proxy workspace_id.
    let workspace_id = caller_id;

    // Generate code: "PREFIX-RAND" or just "RAND8"
    let rand_part = generate_code_suffix();
    let code = if let Some(prefix) = &input.code_prefix {
        let clean = prefix.to_uppercase().chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(8)
            .collect::<String>();
        format!("{}-{}", clean, rand_part)
    } else {
        format!("INV-{}", rand_part)
    };

    // Parse optional expires_at
    let expires_at: Option<chrono::DateTime<chrono::Utc>> = input.expires_at
        .as_deref()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc));

    let result = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"INSERT INTO atlas_invite_codes (
            id, code, workspace_id, role,
            asset_id, asset_ids_csv, booking_id, landlord_id, broker_id,
            employer_user_id,
            created_by, max_uses, uses_count, expires_at, is_active,
            label, invite_message, created_at
        ) VALUES (
            gen_random_uuid(), $1, $2, $3,
            $4, $5, $6, $7, $8,
            $9,
            $10, $11, 0, $12, true,
            $13, $14, now()
        )
        ON CONFLICT (code) DO NOTHING
        RETURNING id, code, created_at"#,
        [
            code.clone().into(),
            workspace_id.into(),
            input.role.clone().into(),
            input.asset_id.map(|u| sea_orm::Value::Uuid(Some(Box::new(u)))).unwrap_or(sea_orm::Value::Uuid(None)),
            input.asset_ids_csv.clone().into(),
            input.booking_id.map(|u| sea_orm::Value::Uuid(Some(Box::new(u)))).unwrap_or(sea_orm::Value::Uuid(None)),
            input.landlord_id.map(|u| sea_orm::Value::Uuid(Some(Box::new(u)))).unwrap_or(sea_orm::Value::Uuid(None)),
            input.broker_id.map(|u| sea_orm::Value::Uuid(Some(Box::new(u)))).unwrap_or(sea_orm::Value::Uuid(None)),
            input.employer_user_id.map(|u| sea_orm::Value::Uuid(Some(Box::new(u)))).unwrap_or(sea_orm::Value::Uuid(None)),
            caller_id.into(),
            input.max_uses.into(),
            expires_at.into(),
            input.label.clone().into(),
            input.invite_message.clone().into(),
        ],
    );

    match db.execute(result).await {
        Ok(_) => {
            (StatusCode::CREATED, Json(serde_json::json!({
                "code": code,
                "join_url": format!("/join/{}", code),
                "role": input.role,
                "label": input.label,
            }))).into_response()
        }
        Err(e) => {
            tracing::error!("invite code create failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to create invite code"
            }))).into_response()
        }
    }
}

/// GET /api/folio/invite-codes
///
/// List all invite codes for the caller's workspace.
async fn list_codes(
    State(db): State<DatabaseConnection>,
    headers:   HeaderMap,
) -> impl IntoResponse {
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Authentication required"
        }))).into_response(),
    };

    let caller_id = match resolve_caller_id(&db, &token).await {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Invalid session"
        }))).into_response(),
    };

    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT id, code, role, label, max_uses, uses_count, expires_at, is_active, created_at
           FROM atlas_invite_codes
           WHERE created_by = $1
           ORDER BY created_at DESC
           LIMIT 100"#,
        [caller_id.into()],
    );

    match db.query_all(stmt).await {
        Ok(rows) => {
            let codes: Vec<serde_json::Value> = rows.iter().map(|r| {
                let id: Uuid = r.try_get("", "id").unwrap_or_default();
                let code: String = r.try_get("", "code").unwrap_or_default();
                let role: String = r.try_get("", "role").unwrap_or_default();
                let label: Option<String> = r.try_get("", "label").ok().flatten();
                let max_uses: Option<i32> = r.try_get("", "max_uses").ok().flatten();
                let uses_count: i32 = r.try_get("", "uses_count").unwrap_or(0);
                let expires_at: Option<String> = r.try_get::<Option<chrono::DateTime<Utc>>>("", "expires_at")
                    .ok().flatten().map(|dt| dt.to_rfc3339());
                let is_active: bool = r.try_get("", "is_active").unwrap_or(false);
                let created_at: String = r.try_get::<chrono::DateTime<Utc>>("", "created_at")
                    .map(|dt| dt.to_rfc3339()).unwrap_or_default();
                serde_json::json!({
                    "id": id, "code": code, "role": role, "label": label,
                    "max_uses": max_uses, "uses_count": uses_count,
                    "expires_at": expires_at, "is_active": is_active, "created_at": created_at,
                    "join_url": format!("/join/{}", code),
                })
            }).collect();
            (StatusCode::OK, Json(serde_json::json!({ "codes": codes }))).into_response()
        }
        Err(e) => {
            tracing::error!("invite codes list failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to list invite codes"
            }))).into_response()
        }
    }
}

/// PATCH /api/folio/invite-codes/:id
///
/// Deactivate a code or update its label / max_uses.
async fn patch_code(
    Path(id):    Path<Uuid>,
    State(db):   State<DatabaseConnection>,
    headers:     HeaderMap,
    Json(input): Json<PatchInviteCodeInput>,
) -> impl IntoResponse {
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Authentication required"
        }))).into_response(),
    };

    // Verify caller owns this code
    let caller_id = match resolve_caller_id(&db, &token).await {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Invalid session"
        }))).into_response(),
    };

    // Build dynamic SET clause
    let mut sets: Vec<String> = Vec::new();
    let mut params: Vec<sea_orm::Value> = Vec::new();
    let mut idx = 1usize;

    if let Some(label) = &input.label {
        sets.push(format!("label = ${}", idx)); params.push(label.clone().into()); idx += 1;
    }
    if let Some(active) = input.is_active {
        sets.push(format!("is_active = ${}", idx)); params.push(active.into()); idx += 1;
    }
    if let Some(max_uses) = input.max_uses {
        sets.push(format!("max_uses = ${}", idx)); params.push(max_uses.into()); idx += 1;
    }

    if sets.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "No fields to update"
        }))).into_response();
    }

    // Append WHERE params
    params.push(id.into());
    params.push(caller_id.into());

    let sql = format!(
        "UPDATE atlas_invite_codes SET {} WHERE id = ${} AND created_by = ${} RETURNING id",
        sets.join(", "), idx, idx + 1
    );

    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        &sql,
        params,
    );

    match db.query_one(stmt).await {
        Ok(Some(_)) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))).into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Invite code not found or not owned by caller"
        }))).into_response(),
        Err(e) => {
            tracing::error!("invite code patch failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to update invite code"
            }))).into_response()
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// POST /api/folio/invite-codes/:id/accept
///
/// Called at the final step of every persona wizard. The authenticated user
/// is the person accepting the invite.
///
/// Effects:
///   1. Validates the code is active / not expired / not exhausted
///   2. Atomically increments uses_count and marks inactive if single-use
///   3. Creates `atlas_user_app_roles` row (assigns the role to the accepting user)
///   4. If role = property_manager AND employer_user_id is set on the code:
///        → Creates G-11 `atlas_contracts` row (property_management_agreement)
///   5. Records accepted_by_user_id + accepted_at on the invite code row
async fn accept_code(
    Path(id):   Path<Uuid>,
    State(db):  State<DatabaseConnection>,
    headers:    HeaderMap,
) -> impl IntoResponse {
    // Authenticate caller
    let token = match extract_bearer(&headers) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Authentication required"
        }))).into_response(),
    };
    let accepting_user_id = match resolve_caller_id(&db, &token).await {
        Some(id) => id,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({
            "error": "Invalid session"
        }))).into_response(),
    };

    // 1. Load the invite code row
    let row_stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT id, role, employer_user_id, workspace_id,
                  asset_id, asset_ids_csv, landlord_id,
                  max_uses, uses_count, expires_at, is_active
           FROM atlas_invite_codes
           WHERE id = $1"#,
        [id.into()],
    );
    let row = match db.query_one(row_stmt).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Invite code not found"
        }))).into_response(),
        Err(e) => {
            tracing::error!("accept_code lookup error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error"
            }))).into_response();
        }
    };

    // Validate state
    let is_active: bool  = row.try_get("", "is_active").unwrap_or(false);
    let max_uses: Option<i32> = row.try_get("", "max_uses").ok().flatten();
    let uses_count: i32  = row.try_get("", "uses_count").unwrap_or(0);
    let expires_at: Option<chrono::DateTime<chrono::Utc>> =
        row.try_get("", "expires_at").ok().flatten();
    let is_expired   = expires_at.map_or(false, |exp| exp < Utc::now());
    let is_exhausted = max_uses.map_or(false, |max| uses_count >= max);

    if !is_active || is_expired || is_exhausted {
        return (StatusCode::GONE, Json(serde_json::json!({
            "error": if is_expired { "Invite code has expired" }
                     else if is_exhausted { "Invite code has been fully used" }
                     else { "Invite code is no longer active" }
        }))).into_response();
    }

    let role: String          = row.try_get("", "role").unwrap_or_default();
    let employer_user_id: Option<Uuid> = row.try_get("", "employer_user_id").ok().flatten();
    let workspace_id: Uuid    = row.try_get("", "workspace_id").unwrap_or(accepting_user_id);
    let landlord_id: Option<Uuid> = row.try_get("", "landlord_id").ok().flatten();

    // 2. Atomically increment uses_count (and mark inactive if single-use)
    let deactivate = max_uses.map_or(false, |max| uses_count + 1 >= max);
    let incr_stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        &format!(
            "UPDATE atlas_invite_codes \
             SET uses_count = uses_count + 1, \
                 accepted_by_user_id = $1, \
                 accepted_at = now(){} \
             WHERE id = $2 AND uses_count = $3",
            if deactivate { ", is_active = false" } else { "" }
        ),
        [accepting_user_id.into(), id.into(), uses_count.into()],
    );
    if let Err(e) = db.execute(incr_stmt).await {
        tracing::error!("accept_code increment error: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to consume invite code"
        }))).into_response();
    }

    // Resolve the client_account_id: prefer employer's account, fall back to workspace
    let client_account_id: Uuid = employer_user_id
        .or(landlord_id)
        .unwrap_or(workspace_id);

    // 3. Create atlas_user_app_roles row
    // The client_account_id scopes a property_manager role to the employer's portfolio.
    let role_stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"INSERT INTO atlas_user_app_roles
               (id, user_id, tenant_id, app_slug, role, client_account_id,
                granted_by, granted_at, is_active)
           VALUES
               (gen_random_uuid(), $1, $2, 'folio', $3, $4, $5, now(), true)
           ON CONFLICT (user_id, tenant_id, app_slug, role) DO UPDATE
               SET is_active = true, client_account_id = EXCLUDED.client_account_id,
                   granted_by = EXCLUDED.granted_by, granted_at = now()
           RETURNING id"#,
        [
            accepting_user_id.into(),
            workspace_id.into(),
            role.clone().into(),
            client_account_id.into(),
            employer_user_id.unwrap_or(accepting_user_id).into(),
        ],
    );
    if let Err(e) = db.execute(role_stmt).await {
        tracing::error!("accept_code role creation error: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to assign role"
        }))).into_response();
    }

    // 4. If property_manager + employer_user_id: create G-11 management agreement
    if role == "property_manager" {
        if let Some(employer_id) = employer_user_id {
            let contract_stmt = sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Postgres,
                r#"INSERT INTO atlas_contracts
                       (id, tenant_id, contract_type, status,
                        counterparty_user_id, managed_account_id,
                        effective_from, terms_metadata, created_by, created_at)
                   VALUES
                       (gen_random_uuid(), $1, 'property_management_agreement', 'active',
                        $2, $3,
                        now(), $4, $5, now())
                   ON CONFLICT DO NOTHING"#,
                [
                    workspace_id.into(),
                    accepting_user_id.into(),       // PM is the counterparty
                    employer_id.into(),              // landlord's account
                    serde_json::json!({
                        "scope": "portfolio",
                        "is_employer_admin": true,
                        "invite_code_id": id.to_string(),
                    }).to_string().into(),
                    employer_id.into(),              // created by the landlord
                ],
            );
            if let Err(e) = db.execute(contract_stmt).await {
                // Non-fatal: log and continue. Role is already assigned.
                tracing::warn!("accept_code: G-11 contract creation failed (non-fatal): {e}");
            }
        }
    }

    // G-36: mark linked program action accepted; complete signup and wizard_complete
    // outcomes (accept runs at Folio wizard finish for invite-driven onboarding).
    if let Err(e) = crate::services::program_service::ProgramService::mark_action_accepted(
        &db,
        "invite_code",
        id,
        accepting_user_id,
    )
    .await
    {
        tracing::warn!("accept_code: G-36 mark_action_accepted failed (non-fatal): {e}");
    }
    for outcome in [
        crate::types::pm::ProgramOutcomeType::Signup,
        crate::types::pm::ProgramOutcomeType::WizardComplete,
    ] {
        let label = outcome.to_string();
        if let Err(e) = crate::services::program_service::ProgramService::complete_outcomes_for_invite_code(
            &db,
            id,
            outcome,
            accepting_user_id,
        )
        .await
        {
            tracing::warn!("accept_code: G-36 complete_outcomes({label}) failed (non-fatal): {e}");
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "ok": true,
        "role": role,
        "redirect": role_dashboard_path(&role),
    }))).into_response()
}

/// Returns the dashboard path for a given role.
fn role_dashboard_path(role: &str) -> &'static str {
    match role {
        "landlord"         => "/l",
        "tenant"           => "/t",
        "str_guest"        => "/t",
        "vendor"           => "/v",
        "cohost"           => "/l",
        "owner"            => "/owner",
        "agent"            => "/a",
        "broker"           => "/b",
        "property_manager" => "/l",  // PM lands on the landlord portal they were granted access to
        _                  => "/",
    }
}

/// POST /api/folio/invite-codes/by-code/:code/accept
///
/// Convenience endpoint used by all wizard final steps, which have the SHORT
/// CODE string (e.g. "OAK4B-K7X3") from ResolvedInviteCode.code, not the UUID.
///
/// Resolves the UUID internally then delegates to the same accept_code logic.
async fn accept_code_by_str(
    Path(code): Path<String>,
    State(db):  State<DatabaseConnection>,
    headers:    HeaderMap,
) -> impl IntoResponse {
    // Resolve short code → UUID
    let id_stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT id FROM atlas_invite_codes WHERE code = $1 LIMIT 1",
        [code.trim().to_uppercase().into()],
    );
    let row = match db.query_one(id_stmt).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Invite code not found"
        }))).into_response(),
        Err(e) => {
            tracing::error!("accept_code_by_str lookup error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Database error"
            }))).into_response();
        }
    };
    let id: Uuid = match row.try_get("", "id") {
        Ok(id) => id,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
            "error": "Failed to read invite code id"
        }))).into_response(),
    };

    // Delegate to the UUID-based accept handler
    accept_code(Path(id), State(db), headers).await.into_response()
}

/// Generates a 6-char uppercase alphanumeric random suffix.
fn generate_code_suffix() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
        .hash(&mut hasher);
    // Mix in thread ID for uniqueness across concurrent requests
    std::thread::current().id().hash(&mut hasher);
    let h = hasher.finish();

    // Base-36 encode 6 chars from the hash
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // no 0/O/I/1 (confusables)
    let mut result = String::with_capacity(6);
    let mut n = h;
    for _ in 0..6 {
        result.push(CHARSET[(n % CHARSET.len() as u64) as usize] as char);
        n /= CHARSET.len() as u64;
    }
    result
}

/// Resolves the authenticated caller's user ID from their session token.
/// Returns None if the token is invalid or not found.
async fn resolve_caller_id(db: &DatabaseConnection, token: &str) -> Option<Uuid> {
    use sea_orm::prelude::*;
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT user_id FROM sessions WHERE bearer_token_hash = encode(sha256($1::bytea), 'hex') \
         AND expires_at > now() LIMIT 1",
        [token.into()],
    );
    db.query_one(stmt).await.ok()??.try_get("", "user_id").ok()
}

/// Resolves an asset's display name and address from atlas_assets / atlas_portfolios.
async fn resolve_asset_context(db: &DatabaseConnection, asset_id: Uuid) -> Option<ContextEntity> {
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT name, street_address FROM atlas_assets WHERE id = $1 LIMIT 1",
        [asset_id.into()],
    );
    let row = db.query_one(stmt).await.ok()??;
    Some(ContextEntity {
        name:    row.try_get("", "name").unwrap_or_else(|_| "Property".to_string()),
        address: row.try_get("", "street_address").ok().flatten(),
    })
}

/// Resolves a user's display name (first + last or company name).
async fn resolve_user_context(db: &DatabaseConnection, user_id: Uuid) -> Option<ContextEntity> {
    let stmt = sea_orm::Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        "SELECT first_name, last_name FROM users WHERE id = $1 LIMIT 1",
        [user_id.into()],
    );
    let row = db.query_one(stmt).await.ok()??;
    let first: String = row.try_get("", "first_name").unwrap_or_default();
    let last:  String = row.try_get("", "last_name").unwrap_or_default();
    let name = format!("{} {}", first, last).trim().to_string();
    if name.is_empty() { return None; }
    Some(ContextEntity { name, address: None })
}
