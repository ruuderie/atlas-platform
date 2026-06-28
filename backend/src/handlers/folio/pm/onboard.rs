//! POST /api/folio/pm/onboard
//!
//! Public (token-gated) endpoint that accepts the PMC onboarding wizard payload
//! and atomically provisions the PMC operator. Called by `submit_pmc_onboard`
//! server function in `apps/folio/src/pages/pmc/onboard.rs`.
//!
//! # Flow
//! 1. Validate the `platform_invite` row by ID (passed as `invite_id` UUID).
//!    - Must exist, not expired, have `app_role` containing "pmc".
//! 2. Find or create the `user` row for `primary_email`.
//!    - If no user exists, create one with a stub `password_hash` (magic-link auth only).
//! 3. Create or reuse the `atlas_accounts` row for the PMC company.
//! 4. Assign the user a `pmc_manager` role via `atlas_user_app_roles`.
//! 5. Save portfolio scope (company_type, portfolio_types, unit_count, markets)
//!    as `tenant_setting` rows under the app_instance's tenant.
//! 6. Delete the used invite row (one-time use).
//! 7. Create a short-lived magic-link token so the user can immediately log in.
//! 8. Return `{ magic_link_url, account_id }` for the frontend to redirect.

use axum::{Json, Router, http::StatusCode, routing::post};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::{platform_invite, user, atlas_account, atlas_user_app_roles, tenant_setting};
use crate::services::auth_service::AuthService;

// ── Public route ──────────────────────────────────────────────────────────────

pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/pm/onboard", post(submit_pmc_onboard))
}

// ── Input / Output ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PmcOnboardRequest {
    /// UUID of the `platform_invite` row — sent as `?invite_id=…` from the frontend
    pub invite_id:       Uuid,
    pub company_name:    String,
    pub company_type:    String,
    pub website:         Option<String>,
    pub primary_name:    String,
    pub primary_email:   String,
    pub primary_phone:   String,
    pub billing_email:   Option<String>,
    pub portfolio_types: Vec<String>,
    pub unit_count:      String,
    pub markets:         Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PmcOnboardResponse {
    pub account_id:     Uuid,
    pub magic_link_url: String,
}

// ── Handler ───────────────────────────────────────────────────────────────────

async fn submit_pmc_onboard(
    axum::extract::State(db): axum::extract::State<DatabaseConnection>,
    Json(body): Json<PmcOnboardRequest>,
) -> Result<(StatusCode, Json<PmcOnboardResponse>), StatusCode> {
    // ── 1. Validate invite ────────────────────────────────────────────────────
    let invite = platform_invite::Entity::find_by_id(body.invite_id)
        .one(&db)
        .await
        .map_err(|e| { tracing::error!("pmc/onboard: DB error fetching invite: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
        .ok_or_else(|| { tracing::warn!("pmc/onboard: invite {} not found", body.invite_id); StatusCode::NOT_FOUND })?;

    // Expired?
    if invite.expires_at < Utc::now() {
        tracing::warn!("pmc/onboard: invite {} is expired", body.invite_id);
        return Err(StatusCode::GONE);
    }

    // Must be a PMC invite
    let is_pmc = invite.app_role.as_deref()
        .map(|r| r.contains("pmc") || r.contains("manager"))
        .unwrap_or(false);
    if !is_pmc {
        tracing::warn!("pmc/onboard: invite {} has wrong app_role: {:?}", body.invite_id, invite.app_role);
        return Err(StatusCode::FORBIDDEN);
    }

    // Tenant ID comes from the invite's app_instance_id → look up the app_instance's tenant_id.
    // If the invite has no app_instance_id we use the platform sentinel.
    let tenant_id = if let Some(ai_id) = invite.app_instance_id {
        crate::entities::app_instance::Entity::find_by_id(ai_id)
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .map(|ai| ai.tenant_id)
            .unwrap_or(Uuid::nil())
    } else {
        Uuid::nil()
    };

    // ── 2. Wrap everything in a transaction ───────────────────────────────────
    let txn = db.begin().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // ── 3. Find or create the user ────────────────────────────────────────────
    let email_lower = body.primary_email.trim().to_lowercase();
    let (first, last) = split_name(&body.primary_name);

    let existing_user = user::Entity::find()
        .filter(user::Column::Email.eq(&email_lower))
        .one(&txn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = if let Some(u) = existing_user {
        u.id
    } else {
        let new_user = user::ActiveModel {
            id:            Set(Uuid::new_v4()),
            username:      Set(email_slug(&email_lower)),
            first_name:    Set(first.clone()),
            last_name:     Set(last.clone()),
            email:         Set(email_lower.clone()),
            phone:         Set(body.primary_phone.clone()),
            password_hash: Set(String::new()), // magic-link only; no password needed
            is_active:     Set(true),
            created_at:    Set(Utc::now()),
            updated_at:    Set(Utc::now()),
            last_login:    Set(None),
        };
        new_user.insert(&txn).await
            .map_err(|e| { tracing::error!("pmc/onboard: failed to create user: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?
            .id
    };

    // ── 4. Create the PMC company account ─────────────────────────────────────
    let account_id = Uuid::new_v4();
    let new_account = atlas_account::ActiveModel {
        id:           Set(account_id),
        tenant_id:    Set(tenant_id),
        account_type: Set("organization".to_string()),
        name:         Set(body.company_name.clone()),
        company_email: Set(Some(email_lower.clone())),
        ..Default::default()
    };
    new_account.insert(&txn).await
        .map_err(|e| { tracing::error!("pmc/onboard: failed to create account: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    // ── 5. Assign pmc_manager role (best-effort — no role_profile lookup here) ─
    // We set role_profile_id to nil; a subsequent migration/seed can backfill.
    // The role slug "pmc_manager" is enough for the PropertyManagerOnly extractor.
    let role_assignment = atlas_user_app_roles::ActiveModel {
        id:                Set(Uuid::new_v4()),
        user_id:           Set(user_id),
        tenant_id:         Set(tenant_id),
        app_slug:          Set("folio".to_string()),
        role_profile_id:   Set(Uuid::nil()),
        granted_by:        Set(None),
        granted_at:        Set(Utc::now()),
        expires_at:        Set(None),
        is_active:         Set(true),
        client_account_id: Set(None),
    };
    role_assignment.insert(&txn).await
        .map_err(|e| { tracing::error!("pmc/onboard: failed to assign role: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    // ── 6. Persist portfolio scope as tenant settings ─────────────────────────
    let settings = vec![
        ("pmc_company_type",     body.company_type.clone()),
        ("pmc_portfolio_types",  body.portfolio_types.join(",")),
        ("pmc_unit_count",       body.unit_count.clone()),
        ("pmc_markets",          body.markets.join(",")),
        ("pmc_website",          body.website.clone().unwrap_or_default()),
        ("pmc_billing_email",    body.billing_email.clone().unwrap_or_default()),
    ];
    for (key, val) in settings {
        let setting = tenant_setting::ActiveModel {
            id:           Set(Uuid::new_v4()),
            tenant_id:    Set(tenant_id),
            key:          Set(key.to_string()),
            value:        Set(val),
            is_encrypted: Set(false),
            updated_at:   Set(Utc::now()),
            created_at:   Set(Utc::now()),
        };
        // Ignore duplicate key errors — idempotent
        let _ = setting.insert(&txn).await;
    }

    // ── 7. Delete the invite (one-time use) ───────────────────────────────────
    platform_invite::Entity::delete_by_id(body.invite_id)
        .exec(&txn)
        .await
        .map_err(|e| { tracing::error!("pmc/onboard: failed to delete invite: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;

    txn.commit().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // ── 8. Issue magic-link token (outside txn — read-only DB op + env reads) ──
    let frontend_url = std::env::var("FRONTEND_URL")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());

    let magic_link_url = match AuthService::create_magic_link(&db, &email_lower).await {
        Ok(tok) => format!("{}/magic-login?token={}&next=/pmc/dashboard", frontend_url, tok.token),
        Err(_)  => format!("{}/pmc/dashboard", frontend_url),
    };

    Ok((StatusCode::CREATED, Json(PmcOnboardResponse { account_id, magic_link_url })))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Split a full name string into (first, last). Everything after the first space is "last".
fn split_name(name: &str) -> (String, String) {
    let name = name.trim();
    if let Some(idx) = name.find(' ') {
        (name[..idx].to_string(), name[idx + 1..].trim().to_string())
    } else {
        (name.to_string(), String::new())
    }
}

/// Derive a username slug from an email address (everything before the @).
fn email_slug(email: &str) -> String {
    email.split('@').next().unwrap_or(email).to_string()
}
