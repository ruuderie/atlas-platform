use axum::{Extension, Json, http::StatusCode};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{session, user, user_account};
use crate::types::pm::FolioRole;

/// Response body for `GET /api/folio/me`.
/// Consumed by the Folio Leptos frontend `check_session()` server fn.
#[derive(Debug, Serialize, Deserialize)]
pub struct FolioMeResponse {
    pub user_id:      Uuid,
    pub tenant_id:    Option<Uuid>,
    pub email:        String,
    pub display_name: Option<String>,
    pub folio_role:   FolioRole,
}

/// `GET /api/folio/me`
///
/// Returns the authenticated user's folio-specific profile and role.
/// The role determines which frontend namespace (`/l`, `/t`, `/v`) the
/// client is routed to, and which backend endpoints it may call.
///
/// Authorization: self-contained — validates Bearer token / atlas_session cookie
/// directly against the sessions table. Listed in FolioApp::public_router() to
/// avoid double-wrapping by the outer session middleware.
pub async fn get_folio_me(
    Extension(db): Extension<DatabaseConnection>,
    headers: axum::http::HeaderMap,
) -> Result<Json<FolioMeResponse>, StatusCode> {
    // ── 1. Extract bearer token ───────────────────────────────────────────────
    let token = extract_bearer(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let token_hash = crate::auth::hash_token(&token);

    // ── 2. Resolve session → user_id ──────────────────────────────────────────
    let session_row = session::Entity::find()
        .filter(session::Column::BearerTokenHash.eq(&token_hash))
        .filter(session::Column::IsActive.eq(true))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Guard expiry manually (token_expiration field)
    if session_row.token_expiration < chrono::Utc::now() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // ── 3. Fetch user ─────────────────────────────────────────────────────────
    let user_row = user::Entity::find_by_id(session_row.user_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // ── 4. Fetch user_account — get folio_role and tenant_id ──────────────────
    // folio_role is a raw column added by m20260810_. SeaORM entity doesn't
    // have it yet (entity field is deferred to avoid re-generating). We use a
    // raw SQL fetch via sea_orm::Statement.
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT ua.folio_role, a.tenant_id
               FROM user_account ua
               JOIN account a ON ua.account_id = a.id
               WHERE ua.user_id = $1
                 AND ua.is_active = true
               ORDER BY ua.created_at ASC
               LIMIT 1"#,
            [session_row.user_id.into()],
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (folio_role, tenant_id) = match row {
        Some(r) => {
            let raw_role: Option<String> = r.try_get("", "folio_role").ok();
            let role = raw_role
                .and_then(|s| FolioRole::try_from(s).ok())
                .unwrap_or_default();
            let tid: Option<Uuid> = r.try_get("", "tenant_id").ok().flatten();
            (role, tid)
        }
        None => (FolioRole::Landlord, None),
    };

    let display_name = format!("{} {}", user_row.first_name, user_row.last_name).trim().to_owned();
    let display_name = if display_name.is_empty() { None } else { Some(display_name) };

    Ok(Json(FolioMeResponse {
        user_id: user_row.id,
        tenant_id,
        email: user_row.email,
        display_name,
        folio_role,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get(axum::http::header::COOKIE)
                .and_then(|v| v.to_str().ok())
                .and_then(|cookies| {
                    cookies.split(';').find_map(|part| {
                        part.trim()
                            .strip_prefix("atlas_session=")
                            .map(|t| t.to_string())
                    })
                })
        })
}
