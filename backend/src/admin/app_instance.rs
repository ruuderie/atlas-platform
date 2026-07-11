//! Admin — App Instance Public Config handler
//!
//! Manages per-instance `public_slug`, `custom_domain`, and live stats on
//! `atlas_app_deployment_config`. These two fields enable the
//! zero-tenant domain resolver at `GET /api/pub/tenant-context`.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/admin/app-instances/{id}/public-config
//!      Returns the current public_slug, custom_domain, and instance_status.
//!      -> 200 PublicConfigResponse
//!
//! PUT  /api/admin/app-instances/{id}/public-config
//!      Set/update public_slug and/or custom_domain.
//!      Validates global uniqueness. Returns DNS CNAME instructions.
//!      Body: { public_slug?, custom_domain? }
//!      -> 200 PublicConfigResponse (includes dns_instructions)
//!
//! POST /api/admin/app-instances/{id}/suspend
//!      Sets instance_status = "suspended".  Body: { reason }
//!      -> 200
//!
//! POST /api/admin/app-instances/{id}/resume
//!      Sets instance_status = "active".
//!      -> 200
//!
//! POST /api/admin/app-instances/{id}/archive
//!      Sets instance_status = "archived".  Body: { reason, data_retention_days? }
//!      -> 200
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_app_deployment_config;
use crate::services::ingress_provisioner::IngressProvisioner;
use std::sync::Arc;

// ── Route registration ────────────────────────────────────────────────────────

/// State-free router — merge before outer .with_state(db) in admin_routes().
pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/admin/app-instances/{id}/public-config",
            get(get_public_config).put(update_public_config),
        )
        .route(
            "/api/admin/app-instances/{id}/suspend",
            post(suspend_instance),
        )
        .route(
            "/api/admin/app-instances/{id}/resume",
            post(resume_instance),
        )
        .route(
            "/api/admin/app-instances/{id}/archive",
            post(archive_instance),
        )
        // DELETE /api/admin/app-instances/{id} — alias for archive (soft-delete)
        .route("/api/admin/app-instances/{id}", delete(delete_instance))
        // POST /api/admin/app-instances/{id}/reset — re-queue onboarding wizard
        .route("/api/admin/app-instances/{id}/reset", post(reset_instance))
        // POST /api/admin/app-instances/{id}/reprovision-domain — re-fire ingress provisioning
        .route(
            "/api/admin/app-instances/{id}/reprovision-domain",
            post(reprovision_domain),
        )
        .route(
            "/api/admin/app-instances/{id}/operational-config",
            axum::routing::patch(update_operational_config),
        )
        // Phase 5: live per-instance activity stats
        .route(
            "/api/admin/app-instances/{id}/stats",
            get(get_instance_stats),
        )
}

/// Convenience wrapper with state applied (for standalone use / tests).
pub fn routes(db: DatabaseConnection) -> Router {
    routes_raw().with_state(db)
}

// ── Response / input types ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicConfigResponse {
    pub instance_id: Uuid,
    pub tenant_id: Uuid,
    /// Human-readable tenant name (e.g. "buildwithruud").
    /// Resolved by joining the tenant table — never a UUID.
    pub tenant_name: String,
    pub app_slug: String,
    pub public_slug: Option<String>,
    pub custom_domain: Option<String>,
    pub instance_status: String,
    /// Folio operational mode: "standard" | "pmc" | "brokerage"
    pub folio_mode: String,
    /// Billing tier key stored in config JSON: "free" | "starter" | "growth" | "enterprise"
    pub billing_tier: String,
    /// Whether tenant self-service portal is active
    pub tenant_portal_enabled: bool,
    /// Whether vendor self-service portal is active
    pub vendor_portal_enabled: bool,
    /// Present when instance has a custom_domain set — always included in GET + PUT responses.
    pub dns_instructions: Option<DnsInstructions>,
}

#[derive(Debug, Serialize)]
pub struct DnsInstructions {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub note: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePublicConfigBody {
    pub public_slug: Option<String>,
    pub custom_domain: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SuspendBody {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ArchiveBody {
    pub reason: String,
    pub data_retention_days: Option<u32>,
}

/// Body for PATCH /api/admin/app-instances/{id}/operational-config
#[derive(Debug, Deserialize)]
pub struct UpdateOperationalConfigBody {
    /// "standard" | "pmc" | "brokerage"
    pub folio_mode: Option<String>,
    /// "free" | "starter" | "growth" | "enterprise"
    pub billing_tier: Option<String>,
    pub tenant_portal_enabled: Option<bool>,
    pub vendor_portal_enabled: Option<bool>,
    /// Branding fields — stored in config["branding"] JSON object
    /// Theme: "dark-slate" | "light-clean" | "high-contrast"
    pub branding_theme: Option<String>,
    /// Primary brand color hex, e.g. "#0A84FF"
    pub branding_color: Option<String>,
    /// Font key: "inter" | "roboto" | "outfit"
    pub branding_font: Option<String>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn platform_cname_target() -> String {
    // Configurable per-environment via ATLAS_CNAME_TARGET in the k8s ConfigMap.
    // Dev:  api.dev.atlas.oply.co
    // UAT:  api.uat.atlas.oply.co
    // Prod: api.atlas.oply.co
    std::env::var("ATLAS_CNAME_TARGET").unwrap_or_else(|_| "api.atlas.oply.co".to_string())
}

// ── Handlers ──────────────────────────────────────────────────────────────────

pub async fn get_public_config(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::entities::app_instance;
    use sea_orm::IntoActiveModel;

    // Step 1: Resolve the app_instance row — this is always keyed correctly on instance_id.
    let inst_opt = app_instance::Entity::find_by_id(instance_id)
        .one(&db)
        .await
        .unwrap_or(None);

    let Some(inst) = inst_opt else {
        return (StatusCode::NOT_FOUND, "instance not found").into_response();
    };

    let tenant_id = inst.tenant_id;

    // Step 2: Look up the deployment config keyed on tenant_id (NOT config.id).
    // atlas_app_deployment_config.id is auto-generated by the DB and is NOT the same
    // as app_instances.id. The provision_tenant handler creates config rows with their
    // own random UUIDs. We MUST join via tenant_id.
    let deployment_cfg = atlas_app_deployment_config::Entity::find()
        .filter(atlas_app_deployment_config::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await;

    let cfg = match deployment_cfg {
        Err(e) => {
            tracing::error!("get_public_config: db error {e:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
        Ok(Some(c)) => c,
        Ok(None) => {
            // No deployment config row yet — seed a minimal one.
            // This covers pre-G33 instances provisioned before atlas_app_deployment_config existed.
            let derived_slug = format!("{}-{}", inst.app_type, &inst.id.to_string()[..8]);
            let seed = atlas_app_deployment_config::ActiveModel {
                tenant_id: Set(tenant_id),
                app_slug: Set(inst.app_type.clone()),
                public_slug: Set(Some(derived_slug)),
                custom_domain: Set(None),
                instance_status: Set(atlas_app_deployment_config::AppInstanceStatus::Active),
                folio_mode: Set(atlas_app_deployment_config::FolioMode::Standard),
                config: Set(serde_json::json!({ "billing_tier": "starter" })),
                ..Default::default()
            };

            // ON CONFLICT DO NOTHING — if two requests race, the second is a no-op.
            // The unique constraint is (tenant_id, app_slug) — both columns must be
            // listed in the conflict target, otherwise PostgreSQL rejects the INSERT
            // with "no unique or exclusion constraint matching the ON CONFLICT spec".
            if let Err(e) = atlas_app_deployment_config::Entity::insert(seed)
                .on_conflict(
                    sea_orm::sea_query::OnConflict::columns([
                        atlas_app_deployment_config::Column::TenantId,
                        atlas_app_deployment_config::Column::AppSlug,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .exec(&db)
                .await
            {
                // DbErr::RecordNotInserted is the expected "did nothing" result from
                // on_conflict().do_nothing() when a conflict is detected — not an error.
                use sea_orm::DbErr;
                if !matches!(e, DbErr::RecordNotInserted) {
                    tracing::error!(
                        "get_public_config: seed INSERT failed for tenant {tenant_id}: {e:#}"
                    );
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
            }

            // Re-fetch after seeding.
            match atlas_app_deployment_config::Entity::find()
                .filter(atlas_app_deployment_config::Column::TenantId.eq(tenant_id))
                .one(&db)
                .await
            {
                Ok(Some(c)) => c,
                Ok(None) => {
                    tracing::error!(
                        "get_public_config: seed produced no row for tenant {tenant_id} (conflict on existing row?)"
                    );
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "failed to initialize instance config",
                    )
                        .into_response();
                }
                Err(e) => {
                    tracing::error!(
                        "get_public_config: re-fetch after seed failed for tenant {tenant_id}: {e:#}"
                    );
                    return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
                }
            }
        }
    };

    let billing_tier = cfg
        .config
        .get("billing_tier")
        .and_then(|v| v.as_str())
        .unwrap_or("starter")
        .to_string();
    let tenant_portal = cfg
        .config
        .get("tenant_portal_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let vendor_portal = cfg
        .config
        .get("vendor_portal_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let dns_instructions = cfg.custom_domain.as_ref().map(|domain| DnsInstructions {
        record_type: "CNAME".to_string(),
        name: domain.clone(),
        value: platform_cname_target().to_string(),
        note: format!(
            "Point {domain} as a CNAME to {target}. SSL is provisioned automatically.",
            target = platform_cname_target()
        ),
    });
    let tenant_name = crate::entities::tenant::Entity::find_by_id(tenant_id)
        .one(&db)
        .await
        .unwrap_or(None)
        .map(|t| t.page_title.unwrap_or(t.name))
        .unwrap_or_else(|| tenant_id.to_string());

    let resp = PublicConfigResponse {
        instance_id,
        tenant_id,
        tenant_name,
        app_slug: cfg.app_slug.clone(),
        public_slug: cfg.public_slug.clone(),
        custom_domain: cfg.custom_domain.clone(),
        instance_status: cfg.instance_status.to_string(),
        folio_mode: cfg.folio_mode.to_string(),
        billing_tier,
        tenant_portal_enabled: tenant_portal,
        vendor_portal_enabled: vendor_portal,
        dns_instructions,
    };
    (StatusCode::OK, Json(resp)).into_response()
}

pub async fn update_public_config(
    State(db): State<DatabaseConnection>,
    Extension(ingress_provisioner): Extension<Arc<IngressProvisioner>>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<UpdatePublicConfigBody>,
) -> impl IntoResponse {
    use crate::entities::app_instance;
    // Resolve tenant_id from the instance first — config.id != instance_id.
    let inst_opt = app_instance::Entity::find_by_id(instance_id)
        .one(&db)
        .await
        .unwrap_or(None);
    let Some(inst) = inst_opt else {
        return (StatusCode::NOT_FOUND, "instance not found").into_response();
    };
    let tenant_id = inst.tenant_id;

    let existing = match atlas_app_deployment_config::Entity::find()
        .filter(atlas_app_deployment_config::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Validate slug format (lowercase alphanumeric + hyphens)
    if let Some(ref slug) = body.public_slug {
        if slug.is_empty() || !slug.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                "public_slug must be lowercase alphanumeric with hyphens only",
            )
                .into_response();
        }
    }

    let mut active: atlas_app_deployment_config::ActiveModel = existing.clone().into();

    if let Some(slug) = body.public_slug.clone() {
        active.public_slug = Set(Some(slug));
    }
    if let Some(domain) = body.custom_domain.clone() {
        active.custom_domain = Set(Some(domain));
    }

    match active.update(&db).await {
        Ok(updated) => {
            // Build DNS instructions if custom_domain was set
            let dns_instructions = body.custom_domain.as_ref().map(|domain| DnsInstructions {
                record_type: "CNAME".to_string(),
                name: domain.clone(),
                value: platform_cname_target().to_string(),
                note: format!(
                    "Point {domain} as a CNAME to {target}. \
                     SSL is provisioned automatically via Cloudflare.",
                    target = platform_cname_target()
                ),
            });
            let billing_tier = updated
                .config
                .get("billing_tier")
                .and_then(|v| v.as_str())
                .unwrap_or("starter")
                .to_string();
            let tenant_portal = updated
                .config
                .get("tenant_portal_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let vendor_portal = updated
                .config
                .get("vendor_portal_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let tenant_name = crate::entities::tenant::Entity::find_by_id(updated.tenant_id)
                .one(&db)
                .await
                .unwrap_or(None)
                .map(|t| t.page_title.unwrap_or(t.name))
                .unwrap_or_else(|| updated.tenant_id.to_string());
            let resp = PublicConfigResponse {
                instance_id: updated.id,
                tenant_id: updated.tenant_id,
                tenant_name,
                app_slug: updated.app_slug.clone(),
                public_slug: updated.public_slug,
                custom_domain: updated.custom_domain.clone(),
                instance_status: updated.instance_status.to_string(),
                folio_mode: updated.folio_mode.to_string(),
                billing_tier,
                tenant_portal_enabled: tenant_portal,
                vendor_portal_enabled: vendor_portal,
                dns_instructions,
            };

            // Trigger ingress + TLS provisioning for the new domain.
            // Non-fatal: if the sidecar is unavailable the domain is still saved and
            // the admin can re-trigger by saving again once the sidecar is healthy.
            if let Some(ref domain) = body.custom_domain {
                // ── Register domain in app_domains ────────────────────────────────────
                // The provisioning flow registers the auto-generated folio.{base} subdomain.
                // When an operator sets a *custom* domain here, that new hostname must also
                // exist in app_domains so that domain-aware backend endpoints (e.g.
                // /api/auth/magic-link/request redirect_url validation) can resolve it.
                // Without this upsert, the ingress routes the domain correctly but the
                // backend rejects it with 400, which surfaces as a 500 to the browser.
                use crate::entities::app_domain;
                let upsert_domain = app_domain::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    app_instance_id: Set(instance_id),
                    domain_name: Set(domain.clone()),
                    created_at: Set(chrono::Utc::now()),
                };
                if let Err(e) = app_domain::Entity::insert(upsert_domain)
                    .on_conflict(
                        sea_orm::sea_query::OnConflict::column(app_domain::Column::DomainName)
                            .do_nothing()
                            .to_owned(),
                    )
                    .exec(&db)
                    .await
                {
                    use sea_orm::DbErr;
                    if !matches!(e, DbErr::RecordNotInserted) {
                        tracing::error!(
                            event  = "update_config.app_domain_upsert.failed",
                            domain = %domain,
                            error  = %e,
                        );
                    }
                } else {
                    tracing::info!(
                        event     = "update_config.app_domain_upsert.ok",
                        domain    = %domain,
                        instance  = %instance_id,
                    );
                }

                let tenant_slug = inst.tenant_id.to_string();
                let app_slug = updated.app_slug.clone();
                let dom = domain.clone();
                let ip = ingress_provisioner.clone();
                tokio::spawn(async move {
                    if let Err(e) = ip.provision_domain(&tenant_slug, &dom, &app_slug).await {
                        tracing::error!(
                            event       = "update_config.ingress.failed",
                            domain      = %dom,
                            tenant_slug = %tenant_slug,
                            error       = %e,
                        );
                    } else {
                        tracing::info!(
                            event  = "update_config.ingress.ok",
                            domain = %dom,
                        );
                    }
                });
            }

            (StatusCode::OK, Json(resp)).into_response()
        }

        Err(e) if e.to_string().contains("unique") || e.to_string().contains("duplicate") => (
            StatusCode::CONFLICT,
            "public_slug or custom_domain is already taken by another instance",
        )
            .into_response(),
        Err(e) => {
            tracing::error!("update_public_config: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

/// PATCH /api/admin/app-instances/{id}/operational-config
///
/// Updates folio_mode and/or config-JSON keys (billing_tier, portal flags)
/// in a single atomic DB write.
pub async fn update_operational_config(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<UpdateOperationalConfigBody>,
) -> impl IntoResponse {
    use crate::entities::atlas_app_deployment_config::FolioMode;
    use sea_orm::JsonValue;

    let existing = match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let mut active: atlas_app_deployment_config::ActiveModel = existing.clone().into();

    // Update folio_mode if provided
    if let Some(ref mode_str) = body.folio_mode {
        let mode = match mode_str.as_str() {
            "pmc" => FolioMode::Pmc,
            "brokerage" => FolioMode::Brokerage,
            _ => FolioMode::Standard,
        };
        active.folio_mode = Set(mode);
    }

    // Merge config-JSON keys
    let mut config: serde_json::Map<String, JsonValue> =
        existing.config.as_object().cloned().unwrap_or_default();

    if let Some(tier) = &body.billing_tier {
        config.insert("billing_tier".into(), JsonValue::String(tier.clone()));
    }
    if let Some(tp) = body.tenant_portal_enabled {
        config.insert("tenant_portal_enabled".into(), JsonValue::Bool(tp));
    }
    if let Some(vp) = body.vendor_portal_enabled {
        config.insert("vendor_portal_enabled".into(), JsonValue::Bool(vp));
    }
    // Branding: merge into config["branding"] object
    if body.branding_theme.is_some()
        || body.branding_color.is_some()
        || body.branding_font.is_some()
    {
        let mut branding: serde_json::Map<String, JsonValue> = config
            .get("branding")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        if let Some(theme) = &body.branding_theme {
            branding.insert("theme".into(), JsonValue::String(theme.clone()));
        }
        if let Some(color) = &body.branding_color {
            branding.insert("primary_color".into(), JsonValue::String(color.clone()));
        }
        if let Some(font) = &body.branding_font {
            branding.insert("font".into(), JsonValue::String(font.clone()));
        }
        config.insert("branding".into(), JsonValue::Object(branding));
    }
    active.config = Set(sea_orm::entity::prelude::Json::Object(config));

    match active.update(&db).await {
        Ok(updated) => {
            let billing_tier = updated
                .config
                .get("billing_tier")
                .and_then(|v| v.as_str())
                .unwrap_or("starter")
                .to_string();
            let tenant_portal = updated
                .config
                .get("tenant_portal_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let vendor_portal = updated
                .config
                .get("vendor_portal_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let tenant_name = crate::entities::tenant::Entity::find_by_id(updated.tenant_id)
                .one(&db)
                .await
                .unwrap_or(None)
                .map(|t| t.page_title.unwrap_or(t.name))
                .unwrap_or_else(|| updated.tenant_id.to_string());
            let resp = PublicConfigResponse {
                instance_id: updated.id,
                tenant_id: updated.tenant_id,
                tenant_name,
                app_slug: updated.app_slug,
                public_slug: updated.public_slug,
                custom_domain: updated.custom_domain,
                instance_status: updated.instance_status.to_string(),
                folio_mode: updated.folio_mode.to_string(),
                billing_tier,
                tenant_portal_enabled: tenant_portal,
                vendor_portal_enabled: vendor_portal,
                dns_instructions: None,
            };
            (StatusCode::OK, Json(resp)).into_response()
        }
        Err(e) => {
            tracing::error!("update_operational_config: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

pub async fn suspend_instance(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<SuspendBody>,
) -> impl IntoResponse {
    set_instance_status(&db, instance_id, "suspended", &body.reason).await
}

pub async fn resume_instance(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    set_instance_status(&db, instance_id, "active", "resumed by platform admin").await
}

pub async fn archive_instance(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
    Json(body): Json<ArchiveBody>,
) -> impl IntoResponse {
    set_instance_status(&db, instance_id, "archived", &body.reason).await
}

async fn set_instance_status(
    db: &DatabaseConnection,
    instance_id: Uuid,
    status: &str,
    reason: &str,
) -> axum::response::Response {
    use crate::entities::atlas_app_deployment_config::AppInstanceStatus;
    let existing = match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let status_enum = match status {
        "active" => AppInstanceStatus::Active,
        "suspended" => AppInstanceStatus::Suspended,
        "archived" => AppInstanceStatus::Archived,
        _ => return (StatusCode::BAD_REQUEST, "invalid status").into_response(),
    };

    let mut active: atlas_app_deployment_config::ActiveModel = existing.into();
    active.instance_status = Set(status_enum);

    match active.update(db).await {
        Ok(_) => {
            tracing::info!(
                instance_id = %instance_id,
                status = status,
                reason = reason,
                "instance status changed"
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "instance_id": instance_id,
                    "status": status,
                    "reason": reason,
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("set_instance_status: {e:#}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

// ── Per-instance stats ────────────────────────────────────────────────────────

/// Response for GET /api/admin/app-instances/{id}/stats
/// All counts are scoped to the instance's tenant_id.
#[derive(Debug, Serialize)]
pub struct InstanceStatsResponse {
    pub instance_id: Uuid,
    pub tenant_id: Uuid,
    pub app_slug: String,
    /// atlas_assets count (Folio: properties/units)
    pub asset_count: u64,
    /// atlas_contracts with status = 'active' (Folio: active leases)
    pub active_contract_count: u64,
    /// atlas_lead total (all apps that surface leads)
    pub lead_count: u64,
    /// atlas_cases with status != 'closed' (open cases/work orders)
    pub open_case_count: u64,
    /// atlas_service_providers (Folio: active vendors)
    pub vendor_count: u64,
    /// listing count (Network Instance: active listings)
    pub active_listing_count: u64,
}

/// GET /api/admin/app-instances/{id}/stats
///
/// Returns per-instance activity counts scoped to the instance's `tenant_id`.
/// All counts are live DB queries — no hardcoded values.
pub async fn get_instance_stats(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::entities::{
        atlas_asset, atlas_case, atlas_contract, atlas_lead, atlas_service_provider, listing,
    };
    use crate::models::listing::ListingStatus;

    // Resolve the instance config to get tenant_id and app_slug
    let cfg = match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => {
            tracing::error!("get_instance_stats: config lookup failed: {e:#}");
            return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }
    };

    let tenant_id = cfg.tenant_id;
    let app_slug = cfg.app_slug.clone();

    // Sequential counts — sea_orm's count() is not Send+Sync-compatible inside tokio::join!
    let asset_count = atlas_asset::Entity::find()
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .unwrap_or(0);

    let active_contract_count = atlas_contract::Entity::find()
        .filter(atlas_contract::Column::TenantId.eq(tenant_id))
        .filter(atlas_contract::Column::Status.eq("active"))
        .count(&db)
        .await
        .unwrap_or(0);

    let lead_count = atlas_lead::Entity::find()
        .filter(atlas_lead::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .unwrap_or(0);

    let open_case_count = atlas_case::Entity::find()
        .filter(atlas_case::Column::TenantId.eq(tenant_id))
        .filter(atlas_case::Column::Status.ne("closed"))
        .count(&db)
        .await
        .unwrap_or(0);

    let vendor_count = atlas_service_provider::Entity::find()
        .filter(atlas_service_provider::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .unwrap_or(0);

    let active_listing_count = listing::Entity::find()
        .filter(listing::Column::TenantId.eq(tenant_id))
        .filter(listing::Column::Status.eq(ListingStatus::Approved))
        .count(&db)
        .await
        .unwrap_or(0);

    let stats = InstanceStatsResponse {
        instance_id,
        tenant_id,
        app_slug,
        asset_count,
        active_contract_count,
        lead_count,
        open_case_count,
        vendor_count,
        active_listing_count,
    };

    (StatusCode::OK, Json(stats)).into_response()
}

// ── DELETE /api/admin/app-instances/{id} ─────────────────────────────────────
// Alias for archive — soft-deletes the instance (sets status = 'archived').
// Data is retained; Ingress/DNS are NOT removed automatically.

pub async fn delete_instance(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    set_instance_status(
        &db,
        instance_id,
        "archived",
        "archived via platform-admin DELETE",
    )
    .await
}

// ── POST /api/admin/app-instances/{id}/reset ─────────────────────────────────
// Resets instance_status to 'active' and clears the onboarding wizard's
// dismissed_at timestamp so the wizard re-appears on the tenant portal.
// Configuration data is not deleted.

pub async fn reset_instance(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    use crate::entities::onboarding_progress;

    // 1. Set instance status back to active
    let status_result =
        set_instance_status(&db, instance_id, "active", "reset by platform admin").await;
    let status_code = status_result.status();
    if !status_code.is_success() {
        return status_result;
    }

    // 2. Clear the onboarding dismissed_at flag so the wizard re-appears.
    //    Best-effort: log failure but don't block the response.
    let onboarding_clear = onboarding_progress::Entity::find()
        .filter(onboarding_progress::Column::AppInstanceId.eq(instance_id))
        .all(&db)
        .await;

    if let Ok(rows) = onboarding_clear {
        for row in rows {
            let mut am: onboarding_progress::ActiveModel = row.into();
            am.dismissed_at = Set(None);
            let _ = am.update(&db).await;
        }
    }

    (StatusCode::OK, Json(serde_json::json!({
        "instance_id": instance_id,
        "status": "active",
        "note": "Instance reset to active. Onboarding wizard will re-appear on the tenant portal."
    }))).into_response()
}

// ── POST /api/admin/app-instances/{id}/reprovision-domain ────────────────────
// Re-fires the ingress provisioning event for the instance's custom_domain.
// Useful when the instance was created before ingress-sidecar was deployed,
// or when DNS propagation caused the initial provisioning to fail.

pub async fn reprovision_domain(
    State(db): State<DatabaseConnection>,
    Path(instance_id): Path<Uuid>,
) -> impl IntoResponse {
    // Step 1: fetch the deployment config row.
    let cfg = match atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => return (StatusCode::NOT_FOUND, "instance not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let domain = match cfg.custom_domain.as_deref().filter(|d| !d.is_empty()) {
        Some(d) => d.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "no custom_domain configured for this instance",
            )
                .into_response();
        }
    };

    // Step 2: look up the tenant slug for a readable ingress label.
    let tenant_slug = crate::entities::tenant::Entity::find_by_id(cfg.tenant_id)
        .one(&db)
        .await
        .ok()
        .flatten()
        .map(|t| t.name)
        .unwrap_or_else(|| cfg.tenant_id.to_string());

    let provisioner = IngressProvisioner::new();
    match provisioner
        .provision_domain(&tenant_slug, &domain, &cfg.app_slug)
        .await
    {
        Ok(_) => {
            tracing::info!(
                instance_id = %instance_id,
                domain = %domain,
                "reprovision-domain triggered successfully"
            );
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "instance_id": instance_id,
                    "domain": domain,
                    "status": "reprovisioning"
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(instance_id = %instance_id, domain = %domain, error = %e, "reprovision-domain failed");
            (
                StatusCode::BAD_GATEWAY,
                format!("ingress sidecar error: {e}"),
            )
                .into_response()
        }
    }
}
