//! Folio Vendor — Profile handler.
//!
//! Exposes GET and PATCH endpoints for the authenticated vendor to read and
//! update their own `atlas_service_providers` record.

use axum::{
    Json, Router, extract::Extension, http::StatusCode, response::IntoResponse, routing::get,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_service_provider, user};
use crate::extractors::folio_role::VendorOnly;
use crate::handlers::folio::vendor::work_orders::resolve_vendor_context;

// ── Route Constructors ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route(
        "/api/folio/vendor/profile",
        get(get_profile).patch(update_profile),
    )
}

// ── Request / Response Types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VendorProfileDetail {
    pub id: Uuid,
    pub business_name: Option<String>,
    pub preferred_payment_rail: Option<String>,
    pub btc_wallet_address: Option<String>,
    pub stripe_connect_id: Option<String>,
    pub is_insured: bool,
    pub is_bonded: bool,
    pub is_marketplace_visible: bool,
    pub marketplace_bio: Option<String>,
    pub marketplace_trade_types: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileInput {
    pub business_name: Option<String>,
    pub preferred_payment_rail: Option<String>,
    pub btc_wallet_address: Option<String>,
    pub stripe_connect_id: Option<String>,
    pub is_insured: Option<bool>,
    pub is_bonded: Option<bool>,
    pub is_marketplace_visible: Option<bool>,
    pub marketplace_bio: Option<String>,
    pub marketplace_trade_types: Option<Vec<String>>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /api/folio/vendor/profile
async fn get_profile(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let (_tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    Ok(Json(VendorProfileDetail {
        id: sp.id,
        business_name: sp.business_name,
        preferred_payment_rail: sp.preferred_payment_rail,
        btc_wallet_address: sp.btc_wallet_address,
        stripe_connect_id: sp.stripe_connect_id,
        is_insured: sp.is_insured,
        is_bonded: sp.is_bonded,
        is_marketplace_visible: sp.is_marketplace_visible,
        marketplace_bio: sp.marketplace_bio,
        marketplace_trade_types: sp.marketplace_trade_types,
    }))
}

/// PATCH /api/folio/vendor/profile
async fn update_profile(
    _guard: VendorOnly,
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<UpdateProfileInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let (_tenant_id, sp) = resolve_vendor_context(&db, current_user.id).await?;

    // Validate bio length
    if let Some(bio) = &input.marketplace_bio {
        if bio.len() > 500 {
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    let mut active: atlas_service_provider::ActiveModel = sp.into();

    if let Some(name) = input.business_name {
        active.business_name = Set(Some(name));
    }
    if let Some(rail) = input.preferred_payment_rail {
        active.preferred_payment_rail = Set(Some(rail));
    }
    if let Some(btc) = input.btc_wallet_address {
        active.btc_wallet_address = Set(Some(btc));
    }
    if let Some(stripe) = input.stripe_connect_id {
        active.stripe_connect_id = Set(Some(stripe));
    }
    if let Some(insured) = input.is_insured {
        active.is_insured = Set(insured);
    }
    if let Some(bonded) = input.is_bonded {
        active.is_bonded = Set(bonded);
    }
    if let Some(visible) = input.is_marketplace_visible {
        active.is_marketplace_visible = Set(visible);
    }
    if let Some(bio) = input.marketplace_bio {
        active.marketplace_bio = Set(Some(bio));
    }
    if let Some(trade_types) = input.marketplace_trade_types {
        active.marketplace_trade_types = Set(trade_types);
    }

    let updated = active.update(&db).await.map_err(|e| {
        tracing::error!("update_profile failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(VendorProfileDetail {
        id: updated.id,
        business_name: updated.business_name,
        preferred_payment_rail: updated.preferred_payment_rail,
        btc_wallet_address: updated.btc_wallet_address,
        stripe_connect_id: updated.stripe_connect_id,
        is_insured: updated.is_insured,
        is_bonded: updated.is_bonded,
        is_marketplace_visible: updated.is_marketplace_visible,
        marketplace_bio: updated.marketplace_bio,
        marketplace_trade_types: updated.marketplace_trade_types,
    }))
}
