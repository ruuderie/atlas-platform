//! # G20 Attribution HTTP handlers — Folio (Phase 6)
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | /api/folio/attribution/touchpoints | Capture a marketing touchpoint |
//! | POST   | /api/folio/attribution/resolve | Resolve anonymous_id → user_id |
//! | POST   | /api/folio/attribution/conversions | Record a conversion + distribute credit |
//! | GET    | /api/folio/attribution/path/{entity_id} | Get conversion path for an entity |
//! | GET    | /api/folio/attribution/journey/{user_id} | Get full user journey |
//! | GET    | /api/folio/attribution/campaign/{campaign_id} | All touchpoints for a campaign |

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::attribution::{AttributionService, CapturePayload, ConversionPayload, UtmParams},
    types::pm::{AttributionChannel, AttributionModel},
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/attribution/touchpoints",
            post(capture_touchpoint),
        )
        .route("/api/folio/attribution/resolve", post(resolve_identity))
        .route(
            "/api/folio/attribution/conversions",
            post(record_conversion),
        )
        .route(
            "/api/folio/attribution/path/{entity_id}",
            get(get_conversion_path),
        )
        .route(
            "/api/folio/attribution/journey/{user_id}",
            get(get_user_journey),
        )
        .route(
            "/api/folio/attribution/campaign/{campaign_id}",
            get(get_campaign_touchpoints),
        )
}

// ── Shared tenant resolution ──────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    let user_accounts = crate::entities::user_account::Entity::find()
        .filter(crate::entities::user_account::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let account_ids: Vec<Uuid> = user_accounts.into_iter().map(|ua| ua.account_id).collect();

    let profile = crate::entities::profile::Entity::find()
        .filter(crate::entities::profile::Column::AccountId.is_in(account_ids))
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::FORBIDDEN)?;

    Ok(profile.tenant_id)
}

// ── Request types ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CaptureRequest {
    channel: String,
    utm_source: Option<String>,
    utm_medium: Option<String>,
    utm_campaign: Option<String>,
    utm_content: Option<String>,
    utm_term: Option<String>,
    contact_email: Option<String>,
    anonymous_id: Option<String>,
    campaign_id: Option<Uuid>,
    enrollment_id: Option<Uuid>,
    event_id: Option<Uuid>,
    landing_page_url: Option<String>,
    referrer_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResolveIdentityRequest {
    anonymous_id: String,
    user_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct RecordConversionRequest {
    contact_email: Option<String>,
    target_user_id: Option<Uuid>,
    conversion_entity_type: String,
    conversion_entity_id: Uuid,
    conversion_value_cents: i64,
    model: Option<String>,
    attribution_window_days: Option<i64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn capture_touchpoint(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<CaptureRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let channel = match AttributionChannel::try_from(req.channel.as_str()) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };

    let payload = CapturePayload {
        channel,
        utm: UtmParams {
            utm_source: req.utm_source,
            utm_medium: req.utm_medium,
            utm_campaign: req.utm_campaign,
            utm_content: req.utm_content,
            utm_term: req.utm_term,
        },
        user_id: Some(current_user.id),
        contact_email: req.contact_email,
        anonymous_id: req.anonymous_id,
        campaign_id: req.campaign_id,
        enrollment_id: req.enrollment_id,
        event_id: req.event_id,
        landing_page_url: req.landing_page_url,
        referrer_url: req.referrer_url,
    };

    match AttributionService::capture_touchpoint(&db, tenant_id, payload).await {
        Ok(tp) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "touchpoint": tp })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn resolve_identity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<ResolveIdentityRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    match AttributionService::resolve_identity(&db, tenant_id, &req.anonymous_id, req.user_id).await
    {
        Ok(rows) => Json(serde_json::json!({ "rows_updated": rows })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn record_conversion(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<RecordConversionRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let model = match req
        .model
        .as_deref()
        .map(AttributionModel::try_from)
        .unwrap_or(Ok(AttributionModel::LastTouch))
    {
        Ok(m) => m,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e })),
            )
                .into_response();
        }
    };

    let payload = ConversionPayload {
        user_id: req.target_user_id.or(Some(current_user.id)),
        contact_email: req.contact_email,
        conversion_entity_type: req.conversion_entity_type,
        conversion_entity_id: req.conversion_entity_id,
        conversion_value_cents: req.conversion_value_cents,
        model,
        attribution_window_days: req.attribution_window_days,
    };

    match AttributionService::record_conversion(&db, tenant_id, payload).await {
        Ok(ids) => Json(serde_json::json!({
            "credited_touchpoint_ids": ids,
            "count": ids.len()
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_conversion_path(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(entity_id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match AttributionService::get_conversion_path(&db, tenant_id, entity_id).await {
        Ok(tps) => Json(serde_json::json!({ "touchpoints": tps })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_user_journey(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(user_id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match AttributionService::get_user_journey(&db, tenant_id, user_id).await {
        Ok(tps) => Json(serde_json::json!({ "touchpoints": tps })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn get_campaign_touchpoints(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(campaign_id): Path<Uuid>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };
    match AttributionService::get_campaign_touchpoints(&db, tenant_id, campaign_id).await {
        Ok(tps) => Json(serde_json::json!({ "touchpoints": tps })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
