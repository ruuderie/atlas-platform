//! PATCH /api/folio/marketplace/my-listing
//!
//! Allows a landlord to publish or update their vendor's marketplace listing.
//! This is the "opt-in" gate — vendors don't appear in the marketplace until
//! a landlord with Landlord (or PropertyManager) role sets `is_marketplace_visible = true`.
//!
//! # Who uses this?
//!
//! - An individual landlord who has a trusted plumber they want to share
//! - A PMC who has vetted contractors and wants to publish them to the network
//! - The vendor themselves (if they also hold a Vendor role) — future
//!
//! # Privacy
//!
//! Only the fields explicitly sent in the PATCH body are updated.
//! Internal fields (notes, btc_wallet, stripe_connect_id) are never touched here.

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::patch};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::folio_role::LandlordOnly;
use crate::extractors::tenant::TenantContext;

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new().route(
        "/api/folio/marketplace/my-listing",
        patch(update_my_listing),
    )
}

#[derive(Deserialize)]
pub struct ListingUpdateRequest {
    /// The service provider ID to update (must belong to this tenant)
    pub service_provider_id: Uuid,
    /// Set to true to make the vendor discoverable in the marketplace
    pub is_visible: Option<bool>,
    /// Short public bio (max 500 chars)
    pub bio: Option<String>,
    /// Trade type slugs to advertise (e.g. ["plumber", "hvac"])
    pub trade_types: Option<Vec<String>>,
    /// Public latitude (decimal degrees, WGS84). Required if lng is set.
    pub lat: Option<f64>,
    /// Public longitude. Required if lat is set.
    pub lng: Option<f64>,
}

#[derive(Serialize)]
pub struct ListingResponse {
    pub service_provider_id: Uuid,
    pub is_marketplace_visible: bool,
    pub marketplace_bio: Option<String>,
    pub marketplace_trade_types: Vec<String>,
}

async fn update_my_listing(
    _guard: LandlordOnly,
    ctx: TenantContext,
    Extension(db): Extension<DatabaseConnection>,
    Json(body): Json<ListingUpdateRequest>,
) -> impl IntoResponse {
    // ── Verify SP belongs to this tenant ────────────────────────────────────
    let sp =
        match crate::entities::atlas_service_provider::Entity::find_by_id(body.service_provider_id)
            .one(&db)
            .await
        {
            Ok(Some(sp)) if sp.tenant_id == ctx.tenant_id => sp,
            Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(e) => {
                tracing::error!(error = %e, "marketplace/listing: DB error");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };

    // ── Validate bio length ──────────────────────────────────────────────────
    if let Some(bio) = &body.bio {
        if bio.len() > 500 {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({ "error": "bio must be 500 characters or fewer" })),
            )
                .into_response();
        }
    }

    // ── Build update ─────────────────────────────────────────────────────────
    let mut active: crate::entities::atlas_service_provider::ActiveModel = sp.into();

    if let Some(visible) = body.is_visible {
        active.is_marketplace_visible = Set(visible);
    }
    if let Some(bio) = body.bio {
        active.marketplace_bio = Set(Some(bio));
    }
    if let Some(trade_types) = body.trade_types {
        active.marketplace_trade_types = Set(trade_types);
    }

    // Geo update — only if both lat+lng are provided
    if let (Some(lat), Some(lng)) = (body.lat, body.lng) {
        // Store as WKT POINT for Sea-ORM string column; PostGIS parses it.
        // In production with a proper GEOGRAPHY type, use ST_MakePoint().
        // This raw WKT string works because the column is GEOGRAPHY(Point, 4326).
        active.marketplace_location = Set(Some(format!("SRID=4326;POINT({lng} {lat})")));
    }

    match active.update(&db).await {
        Ok(updated) => Json(ListingResponse {
            service_provider_id: updated.id,
            is_marketplace_visible: updated.is_marketplace_visible,
            marketplace_bio: updated.marketplace_bio,
            marketplace_trade_types: updated.marketplace_trade_types,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "marketplace/listing: update failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
