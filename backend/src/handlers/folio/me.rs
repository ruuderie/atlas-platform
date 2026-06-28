use axum::{Extension, Json, http::StatusCode};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{passkey, session, user};
use crate::services::rbac::RbacService;
use crate::types::pm::FolioRole;

/// Response body for `GET /api/folio/me`.
/// Consumed by the Folio Leptos frontend `check_session()` server fn.
#[derive(Debug, Serialize, Deserialize)]
pub struct FolioMeResponse {
    pub user_id:            Uuid,
    pub tenant_id:          Option<Uuid>,
    pub email:              String,
    pub display_name:       Option<String>,
    pub folio_role:         FolioRole,
    /// True if the user has at least one registered passkey.
    pub has_passkey:        bool,
    /// True when all required onboarding steps are complete for this instance.
    pub onboarding_complete: bool,
}

/// `GET /api/folio/me`
///
/// Returns the authenticated user's Folio identity and role.
/// Role is resolved from `atlas_user_app_roles` via `RbacService::get_user_app_role`.
/// Falls back to `FolioRole::Landlord` if no role assignment exists (safe default
/// for existing accounts created before G-32 was seeded).
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

    // ── 2. Resolve session → user_id + tenant context ─────────────────────────
    let session_row = session::Entity::find()
        .filter(session::Column::BearerTokenHash.eq(&token_hash))
        .filter(session::Column::IsActive.eq(true))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if session_row.token_expiration < chrono::Utc::now() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // ── 3. Fetch user ─────────────────────────────────────────────────────────
    let user_row = user::Entity::find_by_id(session_row.user_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // ── 4. Resolve tenant_id from user_account ────────────────────────────────
    use sea_orm::{ConnectionTrait, DbBackend, Statement};

    let tenant_id: Option<Uuid> = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT a.tenant_id
               FROM user_account ua
               JOIN account a ON ua.account_id = a.id
               WHERE ua.user_id = $1
                 AND ua.is_active = true
               ORDER BY ua.created_at ASC
               LIMIT 1"#,
            [session_row.user_id.into()],
        ))
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get::<Uuid>("", "tenant_id").ok());

    // ── 5. Resolve FolioRole via G-32 RbacService ────────────────────────────
    // No role assignment = 403. Defaulting to Landlord was a security gap:
    // any authenticated user without an explicit role silently got full PM access.
    let tid = tenant_id.ok_or_else(|| {
        tracing::warn!(user_id = %session_row.user_id, "folio/me: no tenant context");
        StatusCode::FORBIDDEN
    })?;

    let folio_role = RbacService::get_user_app_role(&db, session_row.user_id, tid, "folio")
        .await
        .and_then(|slug| FolioRole::try_from(slug).ok())
        .ok_or_else(|| {
            tracing::warn!(
                user_id = %session_row.user_id, tenant_id = %tid,
                "folio/me: no folio role assigned"
            );
            StatusCode::FORBIDDEN
        })?;

    let display_name = format!("{} {}", user_row.first_name, user_row.last_name)
        .trim()
        .to_owned();
    let display_name = if display_name.is_empty() { None } else { Some(display_name) };

    // ── 6. Check passkey registration ────────────────────────────────────────
    let has_passkey = passkey::Entity::find()
        .filter(passkey::Column::UserId.eq(user_row.id))
        .count(&db)
        .await
        .unwrap_or(0)
        > 0;

    // ── 7. Check onboarding completeness (non-fatal) ──────────────────────────
    // Resolve the app_instance_id for this tenant so we can query onboarding.
    let app_instance_id: Option<Uuid> = db
        .query_one(sea_orm::Statement::from_sql_and_values(
            sea_orm::DbBackend::Postgres,
            r#"SELECT id FROM app_instance
               WHERE tenant_id = $1
                 AND app_type = 'folio'
               ORDER BY created_at ASC
               LIMIT 1"#,
            [tid.into()],
        ))
        .await
        .ok()
        .flatten()
        .and_then(|r| r.try_get::<Uuid>("", "id").ok());

    let onboarding_complete = if let Some(ai) = app_instance_id {
        // Quick check: does every required step for this app_instance have a
        // completed_at in onboarding_progress?  We use a NOT EXISTS subquery
        // to avoid pulling the full step list — non-fatal, defaults to false.
        use sea_orm::{ConnectionTrait, DbBackend, Statement};
        let row = db
            .query_one(Statement::from_sql_and_values(
                DbBackend::Postgres,
                r#"SELECT NOT EXISTS (
                    SELECT 1
                    FROM onboarding_progress op
                    WHERE op.app_instance_id = $1
                      AND op.is_required = true
                      AND op.completed_at IS NULL
                ) AS all_done"#,
                [ai.into()],
            ))
            .await
            .ok()
            .flatten();
        row.and_then(|r| r.try_get::<bool>("", "all_done").ok())
            .unwrap_or(false)
    } else {
        false
    };

    Ok(Json(FolioMeResponse {
        user_id: user_row.id,
        tenant_id: Some(tid),
        email: user_row.email,
        display_name,
        folio_role,
        has_passkey,
        onboarding_complete,
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
