//! # G31 Lead HTTP handlers — Folio (PM-tier)
//!
//! Routes:
//!
//! | Method | Path                                        | Auth | Description                           |
//! |--------|---------------------------------------------|------|---------------------------------------|
//! | POST   | /api/folio/leads                            | ✅   | Create lead (authenticated)           |
//! | GET    | /api/folio/leads                            | ✅   | List leads (filterable)               |
//! | GET    | /api/folio/leads/{id}                       | ✅   | Get single lead                       |
//! | POST   | /api/folio/leads/{id}/status                | ✅   | Advance pipeline status               |
//! | POST   | /api/folio/leads/{id}/qualify               | ✅   | Qualify shorthand                     |
//! | POST   | /api/folio/leads/{id}/disqualify            | ✅   | Disqualify (terminal, with reason)    |
//! | POST   | /api/folio/leads/{id}/convert               | ✅   | Convert → Account + Contact + Opp     |
//! | POST   | /api/folio/leads/{id}/duplicate             | ✅   | Mark as duplicate of canonical        |
//! | POST   | /api/folio/leads/{id}/campaigns             | ✅   | Enroll in a G19 campaign              |
//! | POST   | /api/folio/leads/ingest                     | ❌   | Public web-form ingest (rate-limited) |
//!
//! Note: These routes live under `/api/folio/leads` (PM-tier, tenant-scoped).
//! The legacy `/api/leads` routes in `src/handlers/leads.rs` operate on the old
//! `lead` table and remain separate.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::lead::{ConvertLeadPayload, CreateLeadPayload, LeadFilter, LeadService},
    types::lead::LeadStatus,
};

// ── Ingest rate limiter ───────────────────────────────────────────────────────
// Independent from the platform RateLimiter — tuned for public web-form submissions.
// 5 submissions per IP per 60 seconds.

const INGEST_MAX_REQUESTS: u32 = 5;
const INGEST_WINDOW: Duration = Duration::from_secs(60);

static INGEST_RATE_LIMITER: Lazy<Arc<DashMap<String, (u32, Instant)>>> =
    Lazy::new(|| Arc::new(DashMap::new()));

fn check_ingest_rate_limit(ip: &str) -> bool {
    let now = Instant::now();
    let mut entry = INGEST_RATE_LIMITER
        .entry(ip.to_string())
        .or_insert((0, now));
    let (count, window_start) = &mut *entry;
    if now.duration_since(*window_start) > INGEST_WINDOW {
        *count = 1;
        *window_start = now;
        true
    } else {
        *count += 1;
        *count <= INGEST_MAX_REQUESTS
    }
}

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/leads", post(create_lead).get(list_leads))
        .route("/api/folio/leads/{id}", get(get_lead))
        .route("/api/folio/leads/{id}/status", post(advance_status))
        .route("/api/folio/leads/{id}/qualify", post(qualify))
        .route("/api/folio/leads/{id}/disqualify", post(disqualify))
        .route("/api/folio/leads/{id}/convert", post(convert))
        .route("/api/folio/leads/{id}/duplicate", post(mark_duplicate))
        .route("/api/folio/leads/{id}/campaigns", post(enroll_campaign))
}

/// Unauthenticated routes — public web-form lead ingest.
/// Rate-limited at the handler level (5 req/IP/60s).
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/folio/leads/ingest", post(ingest_lead))
}

// ── Tenant resolution ─────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListLeadsQuery {
    status: Option<String>,
    source: Option<String>,
    data_source: Option<String>,
    is_converted: Option<bool>,
    is_duplicate: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CreateLeadInput {
    first_name: Option<String>,
    last_name: Option<String>,
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    source: Option<String>,
    data_source: Option<String>,
    data_source_id: Option<String>,
    listing_id: Option<Uuid>,
    message: Option<String>,
    lead_metadata: Option<serde_json::Value>,
    country: Option<String>,
    street_address: Option<String>,
    city: Option<String>,
    state: Option<String>,
    postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdvanceStatusInput {
    status: String,
}

#[derive(Debug, Deserialize)]
struct DisqualifyInput {
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ConvertInput {
    converted_opportunity_id: Option<Uuid>,
    converted_account_id: Option<Uuid>,
    converted_contact_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct MarkDuplicateInput {
    canonical_lead_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct EnrollCampaignInput {
    campaign_id: Uuid,
}

/// Input for the public web-form ingest endpoint.
///
/// `_honeypot` is an invisible field — if it contains a value the request
/// is silently accepted but the lead is **not** created (bot protection).
#[derive(Debug, Deserialize)]
struct IngestLeadInput {
    // ── Honeypot (bot protection) ──────────────────────────────────────────
    /// Must be empty. Hidden with CSS `display:none` on web forms.
    #[serde(default)]
    website_url: String, // honeypot field name chosen to look legitimate

    // ── Identity ───────────────────────────────────────────────────────────
    first_name: Option<String>,
    last_name: Option<String>,
    company: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    message: Option<String>,

    // ── Routing ────────────────────────────────────────────────────────────
    /// Explicit tenant ID — required when `SiteConfig` is not available
    /// (e.g. headless API integrations, Zapier, third-party forms).
    tenant_id: Option<Uuid>,
    /// Optional listing this form was embedded on.
    listing_id: Option<Uuid>,
    /// Optional campaign to auto-enroll the lead into after creation.
    campaign_id: Option<Uuid>,
    /// Traffic source label (e.g. `"google_ads"`, `"facebook"`, `"organic"`).
    utm_source: Option<String>,
}

#[derive(Debug, Serialize)]
struct IngestLeadResponse {
    lead_id: Uuid,
    status: &'static str,
}

async fn create_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(body): Json<CreateLeadInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let lead = LeadService::create(
        &db,
        tenant_id,
        CreateLeadPayload {
            first_name: body.first_name,
            last_name: body.last_name,
            company: body.company,
            email: body.email,
            phone: body.phone,
            source: body.source,
            data_source: body.data_source,
            data_source_id: body.data_source_id,
            listing_id: body.listing_id,
            message: body.message,
            lead_metadata: body.lead_metadata,
            country: body.country,
            street_address: body.street_address,
            city: body.city,
            state: body.state,
            postal_code: body.postal_code,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(lead)))
}

async fn list_leads(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListLeadsQuery>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let status = q
        .status
        .as_deref()
        .map(|s| LeadStatus::try_from(s.to_string()))
        .transpose()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let leads = LeadService::list(
        &db,
        tenant_id,
        LeadFilter {
            status,
            source: q.source,
            data_source: q.data_source,
            is_converted: q.is_converted,
            is_duplicate: q.is_duplicate,
        },
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(leads))
}

async fn get_lead(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let lead = LeadService::get(&db, tenant_id, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(lead))
}

async fn advance_status(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<AdvanceStatusInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let new_status =
        LeadStatus::try_from(body.status).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let lead = LeadService::advance_status(&db, tenant_id, id, new_status)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("terminal") || e.to_string().contains("Use Lead") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    Ok(Json(lead))
}

async fn qualify(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let lead = LeadService::qualify(&db, tenant_id, id)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("terminal") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    Ok(Json(lead))
}

async fn disqualify(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<DisqualifyInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let lead = LeadService::disqualify(&db, tenant_id, id, body.reason)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("terminal") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    Ok(Json(lead))
}

async fn convert(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<ConvertInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let lead = LeadService::convert(
        &db,
        tenant_id,
        id,
        ConvertLeadPayload {
            converted_opportunity_id: body.converted_opportunity_id,
            converted_account_id: body.converted_account_id,
            converted_contact_id: body.converted_contact_id,
        },
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else if e.to_string().contains("terminal") {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok(Json(lead))
}

async fn mark_duplicate(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<MarkDuplicateInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let lead = LeadService::mark_duplicate(&db, tenant_id, id, body.canonical_lead_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(lead))
}

async fn enroll_campaign(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<EnrollCampaignInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let enrollment = LeadService::enroll_in_campaign(
        &db,
        tenant_id,
        id,
        body.campaign_id,
        Some(current_user.id),
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else if e.to_string().contains("terminal") {
            StatusCode::UNPROCESSABLE_ENTITY
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;
    Ok((StatusCode::CREATED, Json(enrollment)))
}

// ── Public ingest handler ─────────────────────────────────────────────────────

/// POST /api/folio/leads/ingest — Public web-form lead ingest.
///
/// ## Security model
/// - **No authentication required** — intended for public web forms.
/// - **Rate limited**: 5 submissions / IP / 60 seconds.
/// - **Honeypot**: `website_url` field; if non-empty the request is silently
///   accepted (200 OK) but no lead is created — bots never get a rejection signal.
///
/// ## Tenant resolution (in priority order)
/// 1. `SiteConfig.tenant_id` — injected by `site_context_middleware` from subdomain
/// 2. `body.tenant_id` — explicit, for headless/Zapier integrations
/// 3. Returns `422 Unprocessable Entity` if neither is available
///
/// ## Post-creation
/// If `campaign_id` is provided, the new lead is enrolled in that G19 campaign.
async fn ingest_lead(
    Extension(db): Extension<DatabaseConnection>,
    site_config_opt: Option<Extension<crate::config::site_config::SiteConfig>>,
    headers: HeaderMap,
    Json(body): Json<IngestLeadInput>,
) -> impl axum::response::IntoResponse {
    // ── Rate limit ────────────────────────────────────────────────────────────
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown");

    if !check_ingest_rate_limit(ip) {
        tracing::warn!(ip, "lead ingest: rate limit exceeded");
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }

    // ── Honeypot ──────────────────────────────────────────────────────────────
    if !body.website_url.is_empty() {
        // Silent accept — do not create a lead. Bot gets no rejection signal.
        tracing::debug!(ip, "lead ingest: honeypot triggered");
        return (
            StatusCode::OK,
            Json(IngestLeadResponse {
                lead_id: Uuid::nil(),
                status: "accepted",
            }),
        )
            .into_response();
    }

    // ── Tenant resolution ─────────────────────────────────────────────────────
    let tenant_id = site_config_opt
        .map(|Extension(sc)| sc.tenant_id)
        .or(body.tenant_id);

    let tenant_id = match tenant_id {
        Some(id) => id,
        None => {
            tracing::warn!(ip, "lead ingest: no tenant_id could be resolved");
            return StatusCode::UNPROCESSABLE_ENTITY.into_response();
        }
    };

    // ── Create lead ───────────────────────────────────────────────────────────
    let source = body.utm_source.as_deref().unwrap_or("web_form").to_string();

    let lead = match LeadService::create(
        &db,
        tenant_id,
        CreateLeadPayload {
            first_name: body.first_name,
            last_name: body.last_name,
            company: body.company,
            email: body.email,
            phone: body.phone,
            source: Some(source),
            data_source: None,
            data_source_id: None,
            listing_id: body.listing_id,
            message: body.message,
            lead_metadata: None,
            country: None,
            street_address: None,
            city: None,
            state: None,
            postal_code: None,
        },
    )
    .await
    {
        Ok(l) => l,
        Err(e) => {
            tracing::error!(ip, error = %e, "lead ingest: create failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    tracing::info!(
        lead_id = %lead.id,
        %tenant_id,
        ip,
        "lead ingest: created"
    );

    // ── Optional campaign enrollment ──────────────────────────────────────────
    if let Some(campaign_id) = body.campaign_id {
        if let Err(e) = LeadService::enroll_in_campaign(
            &db,
            tenant_id,
            lead.id,
            campaign_id,
            None, // no enrolling user on public forms
        )
        .await
        {
            // Non-fatal — lead was created; log and continue.
            tracing::warn!(
                lead_id = %lead.id,
                %campaign_id,
                error = %e,
                "lead ingest: campaign enrollment failed (non-fatal)"
            );
        }
    }

    (
        StatusCode::CREATED,
        Json(IngestLeadResponse {
            lead_id: lead.id,
            status: "created",
        }),
    )
        .into_response()
}
