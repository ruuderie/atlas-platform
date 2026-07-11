//! Admin handlers for `atlas_syndication_offer` — platform-admin-controlled
//! catalog of available syndication connections.
//!
//! All routes require `PlatformSuperAdmin` (enforced by the `PlatformAdminAuth` extractor).
//!
//! Routes:
//!   GET    /api/admin/syndication/offers                    → list_offers
//!   POST   /api/admin/syndication/offers                    → create_offer
//!   GET    /api/admin/syndication/offers/:id               → get_offer
//!   PUT    /api/admin/syndication/offers/:id               → update_offer
//!   DELETE /api/admin/syndication/offers/:id               → retire_offer
//!
//!   GET    /api/admin/syndication/links                     → list_links (all active links)
//!   POST   /api/admin/syndication/links                     → create_link (admin-manual)
//!   DELETE /api/admin/syndication/links/:id                 → revoke_link
//!
//!   POST   /api/admin/syndication/offers/:id/auto-provision → auto_provision_mandatory_links
//!          Scans all instances on mandatory tiers and creates missing links.

use axum::{
    Router,
    extract::{Extension, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::entities::atlas_app_instance_syndication::{
    self, ActiveModel as LinkActiveModel, SyndicationStatus,
};
use crate::entities::atlas_syndication_offer::{
    self, ActiveModel as OfferActiveModel, SyndicationLinkType, SyndicationOfferStatus,
};

// ── Router ──────────────────────────────────────────────────────────────────

pub fn syndication_admin_routes() -> Router<DatabaseConnection> {
    Router::new()
        // Offer catalog
        .route(
            "/api/admin/syndication/offers",
            get(list_offers).post(create_offer),
        )
        .route(
            "/api/admin/syndication/offers/{id}",
            get(get_offer).put(update_offer),
        )
        .route(
            "/api/admin/syndication/offers/{id}/retire",
            post(retire_offer),
        )
        .route(
            "/api/admin/syndication/offers/{id}/auto-provision",
            post(auto_provision_mandatory_links),
        )
        // Active links
        .route(
            "/api/admin/syndication/links",
            get(list_links).post(create_link),
        )
        .route("/api/admin/syndication/links/{id}", delete(revoke_link))
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SyndicationOfferResponse {
    pub id: Uuid,
    pub ni_config_id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    pub syndication_types: Value,
    pub link_type: String,
    pub is_mandatory_for_tiers: Value,
    pub self_service_allowed: bool,
    pub applies_to_folio_mode: Option<String>,
    pub applies_to_app_slug: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<atlas_syndication_offer::Model> for SyndicationOfferResponse {
    fn from(m: atlas_syndication_offer::Model) -> Self {
        SyndicationOfferResponse {
            id: m.id,
            ni_config_id: m.ni_config_id,
            display_name: m.display_name,
            description: m.description,
            syndication_types: m.syndication_types,
            link_type: m.link_type.to_string(),
            is_mandatory_for_tiers: m.is_mandatory_for_tiers,
            self_service_allowed: m.self_service_allowed,
            applies_to_folio_mode: m.applies_to_folio_mode,
            applies_to_app_slug: m.applies_to_app_slug,
            status: m.status.to_string(),
            created_at: m.created_at.to_rfc3339(),
            updated_at: m.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateOfferInput {
    pub ni_config_id: Uuid,
    pub display_name: String,
    pub description: Option<String>,
    /// e.g. ["ltr", "str", "for_sale"]
    pub syndication_types: Option<Value>,
    /// "branded_portal" | "marketplace_syndication"
    pub link_type: Option<String>,
    /// e.g. ["free", "starter"]
    pub is_mandatory_for_tiers: Option<Value>,
    pub self_service_allowed: Option<bool>,
    pub applies_to_folio_mode: Option<String>,
    pub applies_to_app_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOfferInput {
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub syndication_types: Option<Value>,
    pub link_type: Option<String>,
    pub is_mandatory_for_tiers: Option<Value>,
    pub self_service_allowed: Option<bool>,
    pub applies_to_folio_mode: Option<String>,
    pub applies_to_app_slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLinkInput {
    pub source_config_id: Uuid,
    pub ni_config_id: Uuid,
    pub offer_id: Option<Uuid>,
    /// e.g. ["ltr", "str"]
    pub syndication_types: Option<Value>,
    /// "branded_portal" | "marketplace_syndication"
    pub link_type: Option<String>,
    pub inbound_webhook_url: Option<String>,
    pub created_by_tenant_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyndicationLinkResponse {
    pub id: Uuid,
    pub source_config_id: Uuid,
    pub ni_config_id: Uuid,
    pub offer_id: Option<Uuid>,
    pub syndication_types: Value,
    pub link_type: String,
    pub is_mandatory: bool,
    pub status: String,
    pub inbound_webhook_url: Option<String>,
    pub created_by_tenant_id: Uuid,
    pub created_at: String,
}

impl From<atlas_app_instance_syndication::Model> for SyndicationLinkResponse {
    fn from(m: atlas_app_instance_syndication::Model) -> Self {
        SyndicationLinkResponse {
            id: m.id,
            source_config_id: m.source_config_id,
            ni_config_id: m.ni_config_id,
            offer_id: m.offer_id,
            syndication_types: m.syndication_types,
            link_type: m.link_type.to_string(),
            is_mandatory: m.is_mandatory,
            status: m.status.to_string(),
            inbound_webhook_url: m.inbound_webhook_url,
            created_by_tenant_id: m.created_by_tenant_id,
            created_at: m.created_at.to_rfc3339(),
        }
    }
}

// ── Offer handlers ────────────────────────────────────────────────────────────

pub async fn list_offers(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let offers = atlas_syndication_offer::Entity::find()
        .filter(atlas_syndication_offer::Column::Status.ne("retired"))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp: Vec<SyndicationOfferResponse> = offers.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

pub async fn get_offer(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let offer = atlas_syndication_offer::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(SyndicationOfferResponse::from(offer)))
}

pub async fn create_offer(
    State(db): State<DatabaseConnection>,
    Json(input): Json<CreateOfferInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let link_type = match input.link_type.as_deref() {
        Some("branded_portal") => SyndicationLinkType::BrandedPortal,
        _ => SyndicationLinkType::MarketplaceSyndication,
    };

    let model = OfferActiveModel {
        id: Set(Uuid::new_v4()),
        ni_config_id: Set(input.ni_config_id),
        display_name: Set(input.display_name),
        description: Set(input.description),
        syndication_types: Set(input.syndication_types.unwrap_or(json!([]))),
        link_type: Set(link_type),
        is_mandatory_for_tiers: Set(input.is_mandatory_for_tiers.unwrap_or(json!([]))),
        self_service_allowed: Set(input.self_service_allowed.unwrap_or(false)),
        applies_to_folio_mode: Set(input.applies_to_folio_mode),
        applies_to_app_slug: Set(input.applies_to_app_slug),
        status: Set(SyndicationOfferStatus::Active),
        ..Default::default()
    };

    let inserted = model.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create syndication offer: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(SyndicationOfferResponse::from(inserted)),
    ))
}

pub async fn update_offer(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateOfferInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let existing = atlas_syndication_offer::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut model: OfferActiveModel = existing.into();

    if let Some(name) = input.display_name {
        model.display_name = Set(name);
    }
    if let Some(desc) = input.description {
        model.description = Set(Some(desc));
    }
    if let Some(types) = input.syndication_types {
        model.syndication_types = Set(types);
    }
    if let Some(tiers) = input.is_mandatory_for_tiers {
        model.is_mandatory_for_tiers = Set(tiers);
    }
    if let Some(ss) = input.self_service_allowed {
        model.self_service_allowed = Set(ss);
    }
    if let Some(mode) = input.applies_to_folio_mode {
        model.applies_to_folio_mode = Set(Some(mode));
    }
    if let Some(slug) = input.applies_to_app_slug {
        model.applies_to_app_slug = Set(Some(slug));
    }
    if let Some(lt) = input.link_type {
        model.link_type = Set(match lt.as_str() {
            "branded_portal" => SyndicationLinkType::BrandedPortal,
            _ => SyndicationLinkType::MarketplaceSyndication,
        });
    }

    let updated = model
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(SyndicationOfferResponse::from(updated)))
}

pub async fn retire_offer(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let existing = atlas_syndication_offer::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut model: OfferActiveModel = existing.into();
    model.status = Set(SyndicationOfferStatus::Retired);
    model
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Link handlers ─────────────────────────────────────────────────────────────

pub async fn list_links(
    State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, StatusCode> {
    let links = atlas_app_instance_syndication::Entity::find()
        .filter(atlas_app_instance_syndication::Column::Status.ne("revoked"))
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let resp: Vec<SyndicationLinkResponse> = links.into_iter().map(Into::into).collect();
    Ok(Json(resp))
}

pub async fn create_link(
    State(db): State<DatabaseConnection>,
    Json(input): Json<CreateLinkInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let link_type = match input.link_type.as_deref() {
        Some("branded_portal") => SyndicationLinkType::BrandedPortal,
        _ => SyndicationLinkType::MarketplaceSyndication,
    };

    // Derive syndication_types from offer if not provided
    let syndication_types = if let Some(types) = input.syndication_types {
        types
    } else if let Some(oid) = input.offer_id {
        atlas_syndication_offer::Entity::find_by_id(oid)
            .one(&db)
            .await
            .ok()
            .flatten()
            .map(|o| o.syndication_types)
            .unwrap_or(json!([]))
    } else {
        json!([])
    };

    let model = LinkActiveModel {
        id: Set(Uuid::new_v4()),
        source_config_id: Set(input.source_config_id),
        ni_config_id: Set(input.ni_config_id),
        offer_id: Set(input.offer_id),
        syndication_types: Set(syndication_types),
        link_type: Set(link_type),
        is_mandatory: Set(false), // admin-manual links are never mandatory
        status: Set(SyndicationStatus::Active),
        inbound_webhook_url: Set(input.inbound_webhook_url),
        inbound_webhook_secret: Set(None),
        created_by_tenant_id: Set(input.created_by_tenant_id),
        ..Default::default()
    };

    let inserted = model.insert(&db).await.map_err(|e| {
        tracing::error!("Failed to create syndication link: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        Json(SyndicationLinkResponse::from(inserted)),
    ))
}

pub async fn revoke_link(
    State(db): State<DatabaseConnection>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let existing = atlas_app_instance_syndication::Entity::find_by_id(id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Cannot revoke mandatory links from admin panel — must use tier change
    if existing.is_mandatory {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({"error": "Mandatory links cannot be revoked. Change the operator's billing tier to remove the mandatory obligation."}))
        ).into_response());
    }

    let mut model: LinkActiveModel = existing.into();
    model.status = Set(SyndicationStatus::Revoked);
    model
        .update(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::NO_CONTENT, Json(json!({}))).into_response())
}

/// Scans all active Folio/app instances and auto-provisions mandatory syndication
/// links for any instance whose billing tier matches `offer.is_mandatory_for_tiers`.
///
/// This is idempotent — existing links are skipped (unique constraint on source+NI).
/// Run after creating or updating a mandatory offer.
pub async fn auto_provision_mandatory_links(
    State(db): State<DatabaseConnection>,
    Path(offer_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let offer = atlas_syndication_offer::Entity::find_by_id(offer_id)
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mandatory_tiers = offer.mandatory_tiers();
    if mandatory_tiers.is_empty() {
        return Ok(Json(json!({
            "provisioned": 0,
            "skipped": 0,
            "message": "Offer has no mandatory tiers configured."
        })));
    }

    // Load all app deployment configs that match the offer's app_slug filter
    use crate::entities::atlas_app_deployment_config;
    let mut query = atlas_app_deployment_config::Entity::find();
    if let Some(ref app_slug) = offer.applies_to_app_slug {
        query = query.filter(atlas_app_deployment_config::Column::AppSlug.eq(app_slug));
    }
    let configs = query
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut provisioned = 0u32;
    let mut skipped = 0u32;

    for config in configs {
        // Check if billing tier is in mandatory list
        let tier = config
            .config
            .get("billing_tier")
            .and_then(|v| v.as_str())
            .unwrap_or("free")
            .to_string();

        if !mandatory_tiers.contains(&tier) {
            skipped += 1;
            continue;
        }

        // Check if link already exists
        let existing = atlas_app_instance_syndication::Entity::find()
            .filter(atlas_app_instance_syndication::Column::SourceConfigId.eq(config.id))
            .filter(atlas_app_instance_syndication::Column::NiConfigId.eq(offer.ni_config_id))
            .one(&db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if existing.is_some() {
            skipped += 1;
            continue;
        }

        let link = LinkActiveModel {
            id: Set(Uuid::new_v4()),
            source_config_id: Set(config.id),
            ni_config_id: Set(offer.ni_config_id),
            offer_id: Set(Some(offer_id)),
            syndication_types: Set(offer.syndication_types.clone()),
            link_type: Set(offer.link_type.clone()),
            is_mandatory: Set(true),
            status: Set(SyndicationStatus::Active),
            inbound_webhook_url: Set(None),
            inbound_webhook_secret: Set(None),
            created_by_tenant_id: Set(config.tenant_id),
            ..Default::default()
        };

        match link.insert(&db).await {
            Ok(_) => provisioned += 1,
            Err(e) => {
                tracing::warn!("Skipping auto-provision for config {}: {:?}", config.id, e);
                skipped += 1;
            }
        }
    }

    Ok(Json(json!({
        "provisioned": provisioned,
        "skipped": skipped,
        "offer_id": offer_id,
        "mandatory_tiers": mandatory_tiers,
    })))
}
