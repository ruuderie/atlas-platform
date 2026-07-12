//! POST /api/folio/onboarding/submit  — atomic first-run wizard save
//! GET  /api/folio/onboarding/draft   — resume state (saved values + completed steps)
//!
//! Accepts the Folio first-run wizard payload and atomically:
//!   1. Updates the authenticated user's display name (if provided)
//!   2. Saves `folio_jurisdiction_code` to tenant_setting (jurisdiction step)
//!   3. Ensures a default portfolio exists (creates "My Portfolio" if none)
//!   4. Creates the first property asset in that portfolio (first_property step)
//!   5. Returns { portfolio_id, asset_id, applied } for the frontend to confirm
//!
//! The onboarding status checks in `get_onboarding_status` detect completion
//! automatically via their `StepCompletionCheck` queries — no manual progress
//! row is needed for `jurisdiction` (TenantSettingExists) or `first_property`
//! (EntityCountGte).
//!
//! Auth: injected via Folio session middleware (Extension<user::Model>).
//!       Listed in FolioApp::authenticated_router → shared_router.

use axum::routing::get;
use axum::routing::post;
use axum::{Extension, Json, Router, http::HeaderMap, http::StatusCode};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DbBackend, EntityTrait,
    QueryFilter, Set, Statement,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{
    account, atlas_user_notification_pref, onboarding_progress, profile, tenant_setting, user,
    user_account,
};
use crate::services::crm_validator::validate_and_sanitize_phone;
use crate::services::pm::asset::{AssetService, CreateUnitInput};
use crate::services::portfolio_service::PortfolioService;
use crate::types::pm::PropertyType;

// ── Input ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct OnboardingSubmitInput {
    /// Profile step — update display name if non-empty
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    /// Profile step — required E.164 phone when saving profile
    pub phone: Option<String>,
    /// Self-declared WhatsApp use on the registered phone
    pub whatsapp_opt_in: Option<bool>,

    /// Jurisdiction step — saves `folio_jurisdiction_code` tenant setting
    pub jurisdiction_code: Option<String>,

    /// First property step — all three required for property creation
    pub property_name: Option<String>,
    pub property_address: Option<String>,
    pub property_city: Option<String>,
    /// Defaults to `"residential_unit"` if omitted
    pub property_type: Option<String>,
}

// ── Output ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OnboardingSubmitResponse {
    pub portfolio_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    /// Which wizard steps were actually applied this call
    pub applied: Vec<String>,
}

// ── Routes ────────────────────────────────────────────────────────────────────

// ── Draft response ───────────────────────────────────────────────────────────

/// Response body for `GET /api/folio/onboarding/draft`.
/// Returns whatever the wizard has already saved so the frontend can resume.
#[derive(Debug, Serialize)]
pub struct OnboardingDraftResponse {
    /// User's saved first name (from the Profile step), if set.
    pub first_name: Option<String>,
    /// User's saved last name (from the Profile step), if set.
    pub last_name: Option<String>,
    /// User's saved phone (E.164), if set.
    pub phone: Option<String>,
    /// Saved jurisdiction code (e.g. "US", "BR"), if the Jurisdiction step was completed.
    pub jurisdiction_code: Option<String>,
    /// Step IDs that have a `completed_at` timestamp.
    /// Possible values: "profile", "jurisdiction", "first_property".
    pub completed_steps: Vec<String>,
}

// ── Routes ────────────────────────────────────────────────────────────────────

pub fn routes() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/onboarding/submit", post(submit_onboarding))
        .route("/api/folio/onboarding/dismiss", post(dismiss_onboarding))
        .route("/api/folio/onboarding/draft", get(get_onboarding_draft))
}

// ── Handler ───────────────────────────────────────────────────────────────────

/// `POST /api/folio/onboarding/submit`
pub async fn submit_onboarding(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    headers: HeaderMap,
    Json(input): Json<OnboardingSubmitInput>,
) -> Result<Json<OnboardingSubmitResponse>, StatusCode> {
    let user_id = current_user.id;

    // ── 1. Resolve tenant_id (provision landlord workspace if first-run) ──────
    let tenant_id = resolve_or_provision_tenant(&db, &current_user, &headers).await?;

    // Resolve app_instance_id for onboarding_progress writes.
    // Completion is detected two ways:
    //   a) StepCompletionCheck queries (TenantSettingExists / EntityCountGte) — primary
    //   b) onboarding_progress rows — records skips, dismissals, and explicit completes
    let app_instance_id = resolve_app_instance_id(&db, tenant_id).await;

    let mut applied: Vec<String> = Vec::new();

    // ── 2. Update display name + phone (+ WhatsApp pref) ──────────────────────
    let first = input.first_name.as_deref().map(str::trim).unwrap_or("");
    let last = input.last_name.as_deref().map(str::trim).unwrap_or("");
    let phone_raw = input.phone.as_deref().map(str::trim).unwrap_or("");
    let is_profile = !first.is_empty()
        || !last.is_empty()
        || input.phone.is_some()
        || input.whatsapp_opt_in.is_some();

    if is_profile {
        if phone_raw.is_empty() {
            tracing::warn!("onboarding/submit: phone required on profile step");
            return Err(StatusCode::BAD_REQUEST);
        }
        let phone_e164 = validate_and_sanitize_phone(phone_raw).map_err(|e| {
            tracing::warn!(error = %e, "onboarding/submit: invalid phone");
            StatusCode::BAD_REQUEST
        })?;

        let mut am: user::ActiveModel = current_user.clone().into();
        if !first.is_empty() {
            am.first_name = Set(first.to_string());
        }
        if !last.is_empty() {
            am.last_name = Set(last.to_string());
        }
        am.phone = Set(phone_e164.clone());
        am.updated_at = Set(Utc::now());
        am.update(&db).await.map_err(|e| {
            tracing::error!("onboarding/submit: user update failed: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        applied.push("profile".to_string());
        write_progress(&db, tenant_id, app_instance_id, "profile").await;

        // Persist WhatsApp self-declaration into G-07 channel prefs (same tenant_id
        // resolution path as Folio notification prefs).
        if let Some(opt_in) = input.whatsapp_opt_in {
            upsert_whatsapp_pref(&db, user_id, tenant_id, &phone_e164, opt_in).await?;
        }
    }

    // ── 3. Save jurisdiction setting ──────────────────────────────────────────
    let jcode = input
        .jurisdiction_code
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    if !jcode.is_empty() {
        upsert_setting(&db, tenant_id, "folio_jurisdiction_code", jcode).await?;
        applied.push("jurisdiction".to_string());
        write_progress(&db, tenant_id, app_instance_id, "jurisdiction").await;
    }

    // ── 4. Create first property ──────────────────────────────────────────────
    let mut portfolio_id: Option<Uuid> = None;
    let mut asset_id: Option<Uuid> = None;

    let prop_name = input.property_name.as_deref().map(str::trim).unwrap_or("");
    let prop_addr = input
        .property_address
        .as_deref()
        .map(str::trim)
        .unwrap_or("");
    let prop_city = input.property_city.as_deref().map(str::trim).unwrap_or("");

    if !prop_name.is_empty() && !prop_addr.is_empty() {
        // 4a. Find or create a default portfolio
        let pid = find_or_create_default_portfolio(&db, tenant_id, user_id).await?;
        portfolio_id = Some(pid);

        // 4b. Derive country code from jurisdiction
        let country_code = match jcode {
            "BR" => "BR",
            "DR" => "DO",
            "HT" => "HT",
            "USVI" => "VI",
            _ => "US",
        };

        // 4c. Parse property type (default: residential_unit)
        let pt_str = input.property_type.as_deref().unwrap_or("residential_unit");
        let property_type =
            PropertyType::try_from(pt_str.to_string()).unwrap_or(PropertyType::SingleFamily);

        let aid = AssetService::create_unit(
            &db,
            tenant_id,
            CreateUnitInput {
                portfolio_id: pid,
                parent_asset_id: None,
                name: prop_name.to_string(),
                address_line_1: prop_addr.to_string(),
                address_line_2: None,
                city: prop_city.to_string(),
                state_province: String::new(),
                postal_code: String::new(),
                country_code: country_code.to_string(),
                property_type,
                folio_number: None,
                latitude: None,
                longitude: None,
            },
        )
        .await
        .map_err(|e| {
            tracing::error!("onboarding/submit: create_unit failed: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        asset_id = Some(aid);
        applied.push("first_property".to_string());
        write_progress(&db, tenant_id, app_instance_id, "first_property").await;
    }

    Ok(Json(OnboardingSubmitResponse {
        portfolio_id,
        asset_id,
        applied,
    }))
}

// ── Draft handler ────────────────────────────────────────────────────────────

/// `GET /api/folio/onboarding/draft`
///
/// Returns the wizard's current saved state so the frontend can:
///   1. Pre-populate form fields with previously entered values.
///   2. Resume at the correct step (first one without a `completed_at`).
///
/// Auth: Folio session middleware (Extension<user::Model>).
pub async fn get_onboarding_draft(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<Json<OnboardingDraftResponse>, StatusCode> {
    let user_id = current_user.id;
    let tenant_id = resolve_tenant_id(&db, user_id).await?;
    let app_instance_id = resolve_app_instance_id(&db, tenant_id).await;

    // ── 1. User's saved name ─────────────────────────────────────────────────
    let first_name = if current_user.first_name.trim().is_empty() {
        None
    } else {
        Some(current_user.first_name.clone())
    };
    let last_name = if current_user.last_name.trim().is_empty() {
        None
    } else {
        Some(current_user.last_name.clone())
    };
    let phone = if current_user.phone.trim().is_empty() {
        None
    } else {
        Some(current_user.phone.clone())
    };

    // ── 2. Saved jurisdiction code ────────────────────────────────────────────
    let jurisdiction_code = tenant_setting::Entity::find()
        .filter(tenant_setting::Column::TenantId.eq(tenant_id))
        .filter(tenant_setting::Column::Key.eq("folio_jurisdiction_code"))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|s| s.value);

    // ── 3. Completed step IDs ────────────────────────────────────────────────
    let completed_steps = if let Some(ai) = app_instance_id {
        use crate::entities::onboarding_progress;
        let rows = onboarding_progress::Entity::find()
            .filter(onboarding_progress::Column::AppInstanceId.eq(ai))
            .filter(onboarding_progress::Column::StepId.ne("wizard_dismissed"))
            .all(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        rows.into_iter()
            .filter(|r| r.completed_at.is_some())
            .map(|r| r.step_id)
            .collect()
    } else {
        Vec::new()
    };

    Ok(Json(OnboardingDraftResponse {
        first_name,
        last_name,
        phone,
        jurisdiction_code,
        completed_steps,
    }))
}

// ── Dismiss handler ───────────────────────────────────────────────────────────

/// `POST /api/folio/onboarding/dismiss`
///
/// Writes `dismissed_at` to the onboarding_progress "wizard" row so the
/// OnboardingBanner stops showing after the user clicks "I'll do this later".
pub async fn dismiss_onboarding(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<StatusCode, StatusCode> {
    let user_id = current_user.id;
    let tenant_id = resolve_tenant_id(&db, user_id).await?;
    let app_instance_id = resolve_app_instance_id(&db, tenant_id).await;

    // Upsert a special "wizard_dismissed" progress row.
    let existing = onboarding_progress::Entity::find()
        .filter(onboarding_progress::Column::TenantId.eq(tenant_id))
        .filter(onboarding_progress::Column::StepId.eq("wizard_dismissed"))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = existing {
        let mut am: onboarding_progress::ActiveModel = row.into();
        am.dismissed_at = Set(Some(Utc::now()));
        am.updated_at = Set(Utc::now());
        am.update(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        let ai = app_instance_id.unwrap_or_else(Uuid::new_v4);
        onboarding_progress::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            app_instance_id: Set(ai),
            step_id: Set("wizard_dismissed".to_string()),
            completed_at: Set(None),
            skipped: Set(false),
            dismissed_at: Set(Some(Utc::now())),
            metadata: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

const LANDLORD_ROLE_PROFILE_ID: &str = "00000000-0000-0000-0001-000000000001";

/// Resolves the tenant for onboarding — provisions a landlord workspace under
/// the Folio host's tenant when the user has a valid session but no account yet
/// (fresh magic-link / OTP signup).
async fn resolve_or_provision_tenant(
    db: &DatabaseConnection,
    user: &user::Model,
    headers: &HeaderMap,
) -> Result<Uuid, StatusCode> {
    match resolve_tenant_id(db, user.id).await {
        Ok(tid) => Ok(tid),
        Err(StatusCode::FORBIDDEN) => {
            let tenant_id = resolve_tenant_from_host(db, headers).await?;
            provision_landlord_workspace(db, user, tenant_id).await?;
            tracing::info!(
                event = "onboarding.workspace_provisioned",
                user_id = %user.id,
                tenant_id = %tenant_id,
            );
            Ok(tenant_id)
        }
        Err(e) => Err(e),
    }
}

/// Resolve Folio tenant from `X-Forwarded-Host` / `Host` via `app_domains`.
async fn resolve_tenant_from_host(
    db: &DatabaseConnection,
    headers: &HeaderMap,
) -> Result<Uuid, StatusCode> {
    let host = headers
        .get("x-forwarded-host")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            headers
                .get(axum::http::header::HOST)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .map(|h| h.split(':').next().unwrap_or(&h).to_lowercase())
        .ok_or_else(|| {
            tracing::warn!("onboarding/submit: no host header to resolve Folio tenant");
            StatusCode::FORBIDDEN
        })?;

    let row = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT ai.tenant_id
               FROM app_domains ad
               JOIN app_instance ai ON ai.id = ad.app_instance_id
               WHERE lower(ad.domain_name) = $1
               LIMIT 1"#,
            [host.clone().into()],
        ))
        .await
        .map_err(|e| {
            tracing::error!("onboarding/submit: host tenant lookup failed: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .and_then(|r| r.try_get::<Uuid>("", "tenant_id").ok());

    row.ok_or_else(|| {
        tracing::warn!(
            host = %host,
            "onboarding/submit: Folio host not registered in app_domains"
        );
        StatusCode::FORBIDDEN
    })
}

/// Create account + user_account + profile + landlord Folio role for a new user.
async fn provision_landlord_workspace(
    db: &DatabaseConnection,
    user: &user::Model,
    tenant_id: Uuid,
) -> Result<(), StatusCode> {
    let display = {
        let name = format!("{} {}", user.first_name.trim(), user.last_name.trim())
            .trim()
            .to_string();
        if name.is_empty() {
            user.email.clone()
        } else {
            name
        }
    };

    let account_id = Uuid::new_v4();
    account::ActiveModel {
        id: Set(account_id),
        tenant_id: Set(tenant_id),
        name: Set(display.clone()),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        stripe_customer_id: sea_orm::NotSet,
        stripe_payment_method_id: sea_orm::NotSet,
    }
    .insert(db)
    .await
    .map_err(|e| {
        tracing::error!("onboarding/submit: account create failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    user_account::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(user.id),
        account_id: Set(account_id),
        role: Set(user_account::UserRole::Owner),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .map_err(|e| {
        tracing::error!("onboarding/submit: user_account create failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    profile::ActiveModel {
        id: Set(Uuid::new_v4()),
        account_id: Set(account_id),
        tenant_id: Set(tenant_id),
        profile_type: Set(profile::ProfileType::Individual),
        display_name: Set(display),
        contact_info: Set(user.email.clone()),
        business_name: Set(None),
        business_address: Set(None),
        business_phone: Set(None),
        business_website: Set(None),
        additional_info: Set(None),
        is_active: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
        properties: Set(None),
        service_area_zips: Set(None),
    }
    .insert(db)
    .await
    .map_err(|e| {
        tracing::error!("onboarding/submit: profile create failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let role_profile_id =
        Uuid::parse_str(LANDLORD_ROLE_PROFILE_ID).expect("const landlord role profile uuid");

    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"INSERT INTO atlas_user_app_roles
               (id, user_id, tenant_id, app_slug, role_profile_id, is_active, granted_at)
           VALUES ($1, $2, $3, 'folio', $4, true, NOW())
           ON CONFLICT (user_id, tenant_id, app_slug) DO NOTHING"#,
        [
            Uuid::new_v4().into(),
            user.id.into(),
            tenant_id.into(),
            role_profile_id.into(),
        ],
    ))
    .await
    .map_err(|e| {
        tracing::error!("onboarding/submit: landlord role seed failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(())
}

/// Upsert WhatsApp channel preference for the landlord (self-declared onboarding).
async fn upsert_whatsapp_pref(
    db: &DatabaseConnection,
    user_id: Uuid,
    tenant_id: Uuid,
    phone_e164: &str,
    opt_in: bool,
) -> Result<(), StatusCode> {
    let config = serde_json::json!({
        "phone": phone_e164,
        "source": "onboarding_self_declare",
    });

    let existing = atlas_user_notification_pref::Entity::find()
        .filter(atlas_user_notification_pref::Column::UserId.eq(user_id))
        .filter(atlas_user_notification_pref::Column::TenantId.eq(tenant_id))
        .filter(atlas_user_notification_pref::Column::Channel.eq("whatsapp"))
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("onboarding/submit: whatsapp pref lookup failed: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match existing {
        Some(row) => {
            let mut am: atlas_user_notification_pref::ActiveModel = row.into();
            am.config = Set(config);
            am.enabled = Set(opt_in);
            am.updated_at = Set(Utc::now());
            am.update(db).await.map_err(|e| {
                tracing::error!("onboarding/submit: whatsapp pref update failed: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
        None => {
            atlas_user_notification_pref::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(user_id),
                tenant_id: Set(tenant_id),
                channel: Set("whatsapp".to_string()),
                config: Set(config),
                enabled: Set(opt_in),
                applies_to: Set(Vec::new()),
                created_at: Set(Utc::now()),
                updated_at: Set(Utc::now()),
            }
            .insert(db)
            .await
            .map_err(|e| {
                tracing::error!("onboarding/submit: whatsapp pref insert failed: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        }
    }

    Ok(())
}

/// Resolves the tenant_id for `user_id` via the first active `user_account → account` join.
async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    db.query_one(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT a.tenant_id
           FROM user_account ua
           JOIN account a ON ua.account_id = a.id
           WHERE ua.user_id = $1 AND ua.is_active = true
           ORDER BY ua.created_at ASC LIMIT 1"#,
        [user_id.into()],
    ))
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .and_then(|r| r.try_get::<Uuid>("", "tenant_id").ok())
    .ok_or(StatusCode::FORBIDDEN)
}

/// Resolves the app_instance_id for the tenant's Folio deployment.
/// Returns None if not yet provisioned (onboarding_progress write is best-effort).
async fn resolve_app_instance_id(db: &DatabaseConnection, tenant_id: Uuid) -> Option<Uuid> {
    db.query_one(Statement::from_sql_and_values(
        DbBackend::Postgres,
        r#"SELECT id FROM atlas_app_deployment_config
           WHERE tenant_id = $1
           ORDER BY created_at ASC LIMIT 1"#,
        [tenant_id.into()],
    ))
    .await
    .ok()
    .flatten()
    .and_then(|r| r.try_get::<Uuid>("", "id").ok())
}

/// Write a completed onboarding_progress row for `step_id` (idempotent, best-effort).
async fn write_progress(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_instance_id: Option<Uuid>,
    step_id: &str,
) {
    // Best-effort — don't fail the whole request if this write fails.
    let ai = match app_instance_id {
        Some(id) => id,
        None => return, // can't write without an app_instance_id FK
    };

    let existing = onboarding_progress::Entity::find()
        .filter(onboarding_progress::Column::TenantId.eq(tenant_id))
        .filter(onboarding_progress::Column::AppInstanceId.eq(ai))
        .filter(onboarding_progress::Column::StepId.eq(step_id))
        .one(db)
        .await
        .ok()
        .flatten();

    if let Some(row) = existing {
        let mut am: onboarding_progress::ActiveModel = row.into();
        am.completed_at = Set(Some(Utc::now()));
        am.updated_at = Set(Utc::now());
        let _ = am.update(db).await;
    } else {
        let _ = onboarding_progress::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            app_instance_id: Set(ai),
            step_id: Set(step_id.to_string()),
            completed_at: Set(Some(Utc::now())),
            skipped: Set(false),
            dismissed_at: Set(None),
            metadata: Set(None),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(db)
        .await;
    }
}

/// Upserts a tenant setting (create-or-update, idempotent).
async fn upsert_setting(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    key: &str,
    value: &str,
) -> Result<(), StatusCode> {
    let existing = tenant_setting::Entity::find()
        .filter(tenant_setting::Column::TenantId.eq(tenant_id))
        .filter(tenant_setting::Column::Key.eq(key))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(row) = existing {
        let mut am: tenant_setting::ActiveModel = row.into();
        am.value = Set(value.to_string());
        am.updated_at = Set(Utc::now());
        am.update(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        tenant_setting::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            key: Set(key.to_string()),
            value: Set(value.to_string()),
            is_encrypted: Set(false),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
        }
        .insert(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(())
}

/// Returns the existing default portfolio or creates "My Portfolio" if none exists.
async fn find_or_create_default_portfolio(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    owner_user_id: Uuid,
) -> Result<Uuid, StatusCode> {
    // Look for the oldest portfolio for this tenant
    let existing = db
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"SELECT id FROM atlas_portfolio
               WHERE tenant_id = $1
               ORDER BY created_at ASC LIMIT 1"#,
            [tenant_id.into()],
        ))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .and_then(|r| r.try_get::<Uuid>("", "id").ok());

    if let Some(id) = existing {
        return Ok(id);
    }

    PortfolioService::create_portfolio(
        db,
        tenant_id,
        owner_user_id,
        "real_estate",
        "My Portfolio",
        None,
        None,
    )
    .await
    .map_err(|e| {
        tracing::error!("onboarding/submit: create_portfolio failed: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}
