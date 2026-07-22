//! Folio — Assets handler.
//!
//! # Routes
//!
//! ```ignore
//! GET  /api/folio/assets
//!      List all assets for the tenant (properties + units).
//!      -> 200 [AssetSummary]
//!
//! POST /api/folio/assets
//!      Register a new property or unit.
//!      Body: CreateAssetHttpInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/assets/:id
//!      Fetch a single asset with folio number and attributes.
//!      -> 200 AssetDetail
//!
//! PUT  /api/folio/assets/:id/details
//!      Merge beds/baths/sqft/year/notes into attributes.property_details.
//!
//! PUT  /api/folio/assets/:id/capital
//!      Merge purchase/mortgage/other debt into attributes.capital (cents).
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::asset::{AssetService, CreateUnitInput};
use crate::services::pm::asset_archive::{
    validate_alert_types, ArchiveBlocker, AssetAlertType, AssetArchiveService,
};
use crate::services::pm::asset_purge::AssetPurgeService;
use crate::services::pm::lease::LeaseService;
use crate::services::pm::management_delegation::ManagementDelegationService;
use crate::services::pm::record_relationship::RecordRelationshipService;
use crate::types::pm::PropertyType;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/assets", get(list_assets).post(create_asset))
        // Map pin endpoint — all properties with lat/lon for portfolio map.
        // Must be registered BEFORE /{id} to avoid route shadowing.
        .route("/api/folio/assets/map", get(list_assets_map))
        .route("/api/folio/assets/{id}", get(get_asset))
        .route(
            "/api/folio/assets/{id}/children",
            get(list_asset_children),
        )
        .route(
            "/api/folio/assets/{id}/documents",
            get(list_property_documents),
        )
        // Default contractor for this asset — backed by G-22 (atlas_record_relationships).
        // This is the preferred dispatch suggestion, not ownership.
        // Event/inspection history is served by:
        //   GET /api/folio/assets/{id}/inspections  (maintenance.rs — G-13 cases)
        //   GET /api/folio/events?subject_entity_type=atlas_asset&subject_entity_id={id}  (events.rs — G-21)
        .route(
            "/api/folio/assets/{id}/contractor",
            get(get_asset_contractor),
        )
        // Same-tenant PM hire (G-11 management_agreement + G-32 asset grants).
        .route(
            "/api/folio/assets/{id}/manager",
            get(get_asset_manager).delete(revoke_asset_manager),
        )
        .route(
            "/api/folio/assets/{id}/manager/invite",
            post(invite_asset_manager).delete(cancel_asset_manager_invite),
        )
        .route("/api/folio/assets/{id}/archive", post(archive_asset))
        .route("/api/folio/assets/{id}/purge", post(purge_asset))
        .route(
            "/api/folio/assets/{id}/alert-prefs",
            get(get_alert_prefs).put(put_alert_prefs),
        )
        .route(
            "/api/folio/assets/{id}/details",
            put(put_asset_details),
        )
        .route(
            "/api/folio/assets/{id}/capital",
            put(put_asset_capital),
        )
        .route(
            "/api/folio/assets/{id}/coordinates",
            put(put_asset_coordinates),
        )
        .route(
            "/api/folio/assets/{id}/geocode",
            post(geocode_asset),
        )
}

#[derive(Debug, Deserialize)]
struct CoordinatesBody {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize)]
struct CoordinatesResponse {
    pub lat: f64,
    pub lng: f64,
}

/// PUT /api/folio/assets/{id}/coordinates
async fn put_asset_coordinates(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<CoordinatesBody>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    AssetService::set_coordinates(&db, tenant_id, asset_id, body.lat, body.lng)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else if msg.contains("invalid") || msg.contains("range") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                tracing::error!(%tenant_id, %asset_id, "put_asset_coordinates: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    Ok(axum::response::Json(CoordinatesResponse {
        lat: body.lat,
        lng: body.lng,
    }))
}

/// POST /api/folio/assets/{id}/geocode — Nominatim from stored address.
async fn geocode_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let (lat, lng) = AssetService::geocode_from_address(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else if msg.contains("address") || msg.contains("no geocode") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                tracing::error!(%tenant_id, %asset_id, "geocode_asset: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    Ok(axum::response::Json(CoordinatesResponse { lat, lng }))
}

#[derive(Debug, Serialize)]
struct ArchiveBlockedBody {
    error: &'static str,
    blockers: Vec<ArchiveBlocker>,
}

/// POST /api/folio/assets/{id}/archive — soft-archive (status=decommissioned).
async fn archive_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<Response, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let blockers = AssetArchiveService::collect_blockers(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %asset_id, "archive blockers: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !blockers.is_empty() {
        return Ok((
            StatusCode::CONFLICT,
            axum::response::Json(ArchiveBlockedBody {
                error: "archive_blocked",
                blockers,
            }),
        )
            .into_response());
    }

    AssetArchiveService::archive(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "archive_asset: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
struct PurgeAssetBody {
    pub confirm: String,
}

/// POST /api/folio/assets/{id}/purge — permanently delete asset subtree.
async fn purge_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<PurgeAssetBody>,
) -> Result<Response, StatusCode> {
    if body.confirm != "PURGE" {
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    AssetPurgeService::purge_tree(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "purge_asset: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Serialize)]
struct AlertPrefsResponse {
    pub enabled: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AlertPrefsBody {
    pub enabled: Vec<String>,
}

/// GET /api/folio/assets/{id}/alert-prefs
async fn get_alert_prefs(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::entities::atlas_asset;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let asset = atlas_asset::Entity::find_by_id(asset_id)
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let enabled = asset
        .attributes
        .as_ref()
        .and_then(|a| a.get("folio_alert_prefs"))
        .and_then(|p| p.get("enabled"))
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            AssetAlertType::defaults()
                .into_iter()
                .map(|s| s.to_string())
                .collect()
        });

    Ok(axum::response::Json(AlertPrefsResponse { enabled }))
}

/// PUT /api/folio/assets/{id}/alert-prefs
async fn put_alert_prefs(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<AlertPrefsBody>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::entities::atlas_asset;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let enabled = validate_alert_types(&body.enabled).map_err(|e| {
        tracing::warn!(%tenant_id, %e, "put_alert_prefs validation");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let asset = atlas_asset::Entity::find_by_id(asset_id)
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut attrs = asset
        .attributes
        .clone()
        .unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = attrs.as_object_mut() {
        obj.insert(
            "folio_alert_prefs".into(),
            serde_json::json!({ "enabled": enabled }),
        );
    }

    let mut am: atlas_asset::ActiveModel = asset.into();
    am.attributes = Set(Some(attrs));
    am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "put_alert_prefs: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(axum::response::Json(AlertPrefsResponse { enabled }))
}

/// Property details stored under `attributes.property_details`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PropertyDetailsBody {
    #[serde(default)]
    pub beds: Option<f64>,
    #[serde(default)]
    pub baths: Option<f64>,
    #[serde(default)]
    pub sqft: Option<i32>,
    #[serde(default)]
    pub year_built: Option<i32>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Capital figures stored under `attributes.capital` (cents).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapitalBody {
    #[serde(default)]
    pub purchase_price_cents: Option<i64>,
    #[serde(default)]
    pub mortgage_balance_cents: Option<i64>,
    #[serde(default)]
    pub other_debt_cents: Option<i64>,
}

fn merge_attribute_key(
    existing: Option<serde_json::Value>,
    key: &str,
    value: serde_json::Value,
) -> serde_json::Value {
    let mut attrs = existing.unwrap_or_else(|| serde_json::json!({}));
    if let Some(obj) = attrs.as_object_mut() {
        obj.insert(key.into(), value);
    }
    attrs
}

fn validate_property_details(body: &PropertyDetailsBody) -> Result<(), &'static str> {
    if body.beds.is_some_and(|v| !(0.0..=100.0).contains(&v)) {
        return Err("beds out of range");
    }
    if body.baths.is_some_and(|v| !(0.0..=100.0).contains(&v)) {
        return Err("baths out of range");
    }
    if body.sqft.is_some_and(|v| !(0..=10_000_000).contains(&v)) {
        return Err("sqft out of range");
    }
    if body
        .year_built
        .is_some_and(|v| !(1600..=2200).contains(&v))
    {
        return Err("year_built out of range");
    }
    Ok(())
}

fn validate_capital(body: &CapitalBody) -> Result<(), &'static str> {
    for (label, v) in [
        ("purchase_price_cents", body.purchase_price_cents),
        ("mortgage_balance_cents", body.mortgage_balance_cents),
        ("other_debt_cents", body.other_debt_cents),
    ] {
        if v.is_some_and(|n| n < 0) {
            let _ = label;
            return Err("capital amounts must be non-negative");
        }
    }
    Ok(())
}

/// PUT /api/folio/assets/{id}/details — merge into attributes.property_details.
async fn put_asset_details(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<PropertyDetailsBody>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::entities::atlas_asset;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    validate_property_details(&body).map_err(|e| {
        tracing::warn!(%tenant_id, %asset_id, %e, "put_asset_details validation");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let asset = atlas_asset::Entity::find_by_id(asset_id)
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let notes = body
        .notes
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let stored = PropertyDetailsBody {
        beds: body.beds,
        baths: body.baths,
        sqft: body.sqft,
        year_built: body.year_built,
        notes,
    };
    let value = serde_json::to_value(&stored).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let attrs = merge_attribute_key(asset.attributes.clone(), "property_details", value);

    let mut am: atlas_asset::ActiveModel = asset.into();
    am.attributes = Set(Some(attrs));
    am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "put_asset_details: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(axum::response::Json(stored))
}

/// PUT /api/folio/assets/{id}/capital — merge into attributes.capital.
async fn put_asset_capital(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<CapitalBody>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::entities::atlas_asset;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    validate_capital(&body).map_err(|e| {
        tracing::warn!(%tenant_id, %asset_id, %e, "put_asset_capital validation");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let asset = atlas_asset::Entity::find_by_id(asset_id)
        .filter(atlas_asset::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let value = serde_json::to_value(&body).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let attrs = merge_attribute_key(asset.attributes.clone(), "capital", value);

    let mut am: atlas_asset::ActiveModel = asset.into();
    am.attributes = Set(Some(attrs));
    am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "put_asset_capital: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(axum::response::Json(body))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct AssetSummary {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub portfolio_id: Option<Uuid>,
    pub parent_asset_id: Option<Uuid>,
    /// `asset_type` in the DB — property type string e.g. "residential_unit"
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub str_eligible: bool,
    #[serde(default)]
    pub str_listing_active: bool,
    pub address_line_1: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAssetHttpInput {
    pub portfolio_id: Uuid,
    pub parent_asset_id: Option<Uuid>,
    pub property_type: String,
    pub name: String,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country_code: String,
    /// County appraiser folio number (e.g. "01-4141-008-0010"). Optional —
    /// if absent, an asset code (e.g. "US-FL-001") is auto-generated.
    pub folio_number: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize)]
struct CreateAssetResponse {
    pub id: Uuid,
}

/// The default contractor for this asset, resolved via G-22
/// `atlas_record_relationships` (`relationship_type = "default_contractor"`).
/// This is a dispatch suggestion pre-filled when scheduling maintenance.
/// The actual contractor on a specific job lives on `atlas_cases.assigned_service_provider_id`.
/// Returns `None` when no default has been set.
#[derive(Debug, Serialize)]
struct AssetContractorSummary {
    pub vendor_id: Uuid,
    pub business_name: String,
    /// First entry of `marketplace_trade_types`, if any.
    pub primary_trade: Option<String>,
    /// Always `"default_contractor"` for Folio. Other verticals use their own labels.
    pub relationship_type: String,
}

#[derive(Debug, Serialize)]
struct AssetDetail {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub portfolio_id: Option<Uuid>,
    pub parent_asset_id: Option<Uuid>,
    pub asset_type: String,
    pub name: String,
    pub serial_or_folio_number: Option<String>,
    pub status: String,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub country_code: Option<String>,
    pub postal_code: Option<String>,
    pub attributes: Option<serde_json::Value>,
    /// Short-term rental eligible (asset trait from attributes / columns).
    pub str_eligible: bool,
    pub str_listing_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListAssetsQuery {
    /// When true, include soft-archived (`decommissioned`) assets. Default: hide them.
    pub show_archived: Option<bool>,
}

/// GET /api/folio/assets
async fn list_assets(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(query): Query<ListAssetsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let show_archived = query.show_archived.unwrap_or(false);

    let mut finder = crate::entities::atlas_asset::Entity::find()
        .filter(crate::entities::atlas_asset::Column::TenantId.eq(tenant_id));
    if !show_archived {
        finder = finder.filter(
            crate::entities::atlas_asset::Column::Status.ne("decommissioned"),
        );
    }

    let assets = finder.all(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "list_assets error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let grants = ManagementDelegationService::accessible_asset_ids(&db, current_user.id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_assets grants: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let assets = AssetService::filter_by_asset_grants(assets, grants.as_deref());

    let summaries: Vec<AssetSummary> = assets
        .into_iter()
        .map(|a| AssetSummary {
            id: a.id,
            tenant_id: a.tenant_id,
            portfolio_id: a.portfolio_id,
            parent_asset_id: a.parent_asset_id,
            asset_type: a.asset_type,
            name: a.name,
            serial_or_folio_number: a.serial_or_folio_number,
            status: a.status,
            created_at: a.created_at,
            str_eligible: a.str_eligible,
            str_listing_active: a.str_listing_active,
            address_line_1: a.address_line_1,
            city: a.city,
            state_province: a.state_province,
            postal_code: a.postal_code,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// POST /api/folio/assets
async fn create_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateAssetHttpInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let property_type = PropertyType::try_from(input.property_type.clone()).map_err(|_| {
        tracing::warn!(
            "create_asset: invalid property_type '{}'",
            input.property_type
        );
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let lat = input.latitude;
    let lng = input.longitude;
    let id = AssetService::create_unit(
        &db,
        tenant_id,
        CreateUnitInput {
            portfolio_id: input.portfolio_id,
            parent_asset_id: input.parent_asset_id,
            name: input.name,
            address_line_1: input.address_line_1,
            address_line_2: input.address_line_2,
            city: input.city,
            state_province: input.state_province,
            postal_code: input.postal_code,
            country_code: input.country_code,
            property_type,
            folio_number: input.folio_number,
            latitude: lat,
            longitude: lng,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_asset error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    AssetService::maybe_geocode_new_asset(&db, tenant_id, id, lat, lng).await;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateAssetResponse { id }),
    ))
}

/// GET /api/folio/assets/:id
async fn get_asset(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let asset = AssetService::get_unit_scoped(&db, tenant_id, asset_id, current_user.id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "get_asset error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(axum::response::Json(AssetDetail {
        id: asset.id,
        tenant_id: asset.tenant_id,
        portfolio_id: asset.portfolio_id,
        parent_asset_id: asset.parent_asset_id,
        asset_type: asset.asset_type,
        name: asset.name,
        serial_or_folio_number: asset.serial_or_folio_number,
        status: asset.status,
        address_line_1: asset.address_line_1,
        address_line_2: asset.address_line_2,
        city: asset.city,
        state_province: asset.state_province,
        country_code: asset.country_code,
        postal_code: asset.postal_code,
        attributes: asset.attributes,
        str_eligible: asset.str_eligible,
        str_listing_active: asset.str_listing_active,
        created_at: asset.created_at,
    }))
}

/// GET /api/folio/assets/:id/children
async fn list_asset_children(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Verify parent exists, belongs to tenant, and is in hired-PM scope.
    let _parent = AssetService::get_unit_scoped(&db, tenant_id, asset_id, current_user.id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "list_asset_children parent error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let children = crate::services::asset_service::AssetService::list_children(
        &db, tenant_id, asset_id, 500,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "list_asset_children error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let grants = ManagementDelegationService::accessible_asset_ids(&db, current_user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let children = AssetService::filter_by_asset_grants(children, grants.as_deref());

    let summaries: Vec<AssetSummary> = children
        .into_iter()
        .map(|a| AssetSummary {
            id: a.id,
            tenant_id: a.tenant_id,
            portfolio_id: a.portfolio_id,
            parent_asset_id: a.parent_asset_id,
            asset_type: a.asset_type,
            name: a.name,
            serial_or_folio_number: a.serial_or_folio_number,
            status: a.status,
            created_at: a.created_at,
            str_eligible: a.str_eligible,
            str_listing_active: a.str_listing_active,
            address_line_1: a.address_line_1,
            city: a.city,
            state_province: a.state_province,
            postal_code: a.postal_code,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

#[derive(Debug, Deserialize)]
struct PropertyDocumentsQuery {
    /// Optional G-13 project id — only expenses from child WOs of that project.
    project_id: Option<Uuid>,
}

/// Kind of row in the property documents compose feed.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PropertyDocumentKind {
    Vault,
    Expense,
}

#[derive(Debug, Serialize)]
struct PropertyDocumentRow {
    pub id: Uuid,
    pub kind: PropertyDocumentKind,
    pub title: String,
    pub category: String,
    pub amount_cents: Option<i64>,
    pub asset_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub project_id: Option<Uuid>,
}

/// GET /api/folio/assets/:id/documents
///
/// Composes G-14 vault docs for this asset (+ direct children) with paid WO
/// costs (expense rows). Optional `?project_id=` filters expenses to that
/// project's child work orders.
async fn list_property_documents(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Query(query): Query<PropertyDocumentsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let _parent = AssetService::get_unit_scoped(&db, tenant_id, asset_id, current_user.id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "list_property_documents parent error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let children = crate::services::asset_service::AssetService::list_children(
        &db, tenant_id, asset_id, 500,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "list_property_documents children error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut scope_ids: Vec<Uuid> = children.into_iter().map(|c| c.id).collect();
    scope_ids.push(asset_id);

    let mut rows: Vec<PropertyDocumentRow> = Vec::new();

    // Vault docs attached to this property or its units.
    let docs = crate::entities::atlas_document::Entity::find()
        .filter(crate::entities::atlas_document::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_document::Column::AppNamespace.eq("folio"))
        .filter(
            crate::entities::atlas_document::Column::RelatedEntityId
                .is_in(scope_ids.clone()),
        )
        .order_by_desc(crate::entities::atlas_document::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %asset_id, "list_property_documents vault error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    for d in docs {
        rows.push(PropertyDocumentRow {
            id: d.id,
            kind: PropertyDocumentKind::Vault,
            title: d.document_category.clone(),
            category: d.document_category,
            amount_cents: None,
            asset_id: d.related_entity_id,
            created_at: d.created_at,
            project_id: None,
        });
    }

    // Paid / completed WO costs as expense rows.
    let mut case_query = crate::entities::atlas_case::Entity::find()
        .filter(crate::entities::atlas_case::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_case::Column::AssetId.is_in(scope_ids))
        .filter(crate::entities::atlas_case::Column::ActualCostCents.is_not_null())
        .order_by_desc(crate::entities::atlas_case::Column::CreatedAt);

    if let Some(project_id) = query.project_id {
        // Restrict to G-22 children of the renovation project.
        let rels = RecordRelationshipService::find_targets(
            &db,
            tenant_id,
            "atlas_case",
            project_id,
            &crate::types::pm::PmRelationshipType::ChildWorkOrder.to_string(),
        )
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %project_id, "list_property_documents rel error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        let child_ids: Vec<Uuid> = rels.into_iter().map(|r| r.target_entity_id).collect();
        if child_ids.is_empty() {
            return Ok(axum::response::Json(rows));
        }
        case_query =
            case_query.filter(crate::entities::atlas_case::Column::Id.is_in(child_ids));
    }

    let cases = case_query.all(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "list_property_documents cases error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    for c in cases {
        rows.push(PropertyDocumentRow {
            id: c.id,
            kind: PropertyDocumentKind::Expense,
            title: c.subject.clone(),
            category: "work_order_cost".into(),
            amount_cents: c.actual_cost_cents,
            asset_id: c.asset_id,
            created_at: c.created_at,
            project_id: query.project_id,
        });
    }

    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(axum::response::Json(rows))
}

/// GET /api/folio/assets/:id/contractor
///
/// Returns the default contractor for this asset, or `null` if none has been set.
///
/// Backed by G-22 `RecordRelationshipService::find_targets` with
/// `relationship_type = "default_contractor"`. This is a dispatch suggestion
/// pre-filled when scheduling maintenance — the actual contractor on a specific job
/// lives on `atlas_cases.assigned_service_provider_id`.
///
/// The relationship is created/deleted via `POST|DELETE /api/folio/relationships`.
/// Other verticals use their own semantic labels (e.g. "default_garage" for fleet).
async fn get_asset_contractor(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Verify asset ownership before traversing relationships.
    let _asset = AssetService::get_unit(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "get_asset_contractor: asset lookup error: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Traverse G-22: atlas_asset → [default_contractor] → atlas_service_providers.
    //
    // relationship_type = "default_contractor" signals this is a dispatch
    // *suggestion* pre-filled when scheduling maintenance — not ownership.
    // The actual contractor on a specific job lives on atlas_cases.assigned_service_provider_id.
    // Other verticals use their own semantic labels (e.g. "default_garage" for fleet).
    let relationships = RecordRelationshipService::find_targets(
        &db,
        tenant_id,
        "atlas_asset",
        asset_id,
        "default_contractor",
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, %asset_id, "get_asset_contractor: relationship lookup error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Take the most recently created assignment (last in the ordered list).
    let Some(rel) = relationships.into_iter().last() else {
        return Ok(axum::response::Json(serde_json::Value::Null));
    };

    let vendor_id = rel.target_entity_id;

    let vendor = crate::entities::atlas_service_provider::Entity::find_by_id(vendor_id)
        .filter(crate::entities::atlas_service_provider::Column::TenantId.eq(tenant_id))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %vendor_id, "get_asset_contractor: vendor lookup error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let Some(vendor) = vendor else {
        // Relationship record exists but vendor was deleted — return null gracefully.
        tracing::warn!(%tenant_id, %asset_id, %vendor_id, "get_asset_contractor: vendor not found (deleted?)");
        return Ok(axum::response::Json(serde_json::Value::Null));
    };

    let summary = AssetContractorSummary {
        vendor_id: vendor.id,
        business_name: vendor
            .business_name
            .unwrap_or_else(|| "Unknown Vendor".to_string()),
        primary_trade: vendor.marketplace_trade_types.into_iter().next(),
        relationship_type: rel.relationship_type,
    };

    Ok(axum::response::Json(serde_json::json!(summary)))
}

// ── Property manager (same-tenant hire) ───────────────────────────────────────

#[derive(Debug, Deserialize)]
struct InviteManagerBody {
    /// When true, omit asset scope — PM covers the employer's whole book.
    #[serde(default)]
    pub portfolio_scope: bool,
    pub label: Option<String>,
}

/// GET /api/folio/assets/:id/manager
async fn get_asset_manager(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let _asset = AssetService::get_unit(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                tracing::error!(%tenant_id, %asset_id, "get_asset_manager: asset lookup: {e:#}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let state = ManagementDelegationService::get_manager_for_asset(&db, current_user.id, asset_id)
        .await
        .map_err(|e| {
            tracing::error!(%asset_id, "get_asset_manager: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(axum::response::Json(state))
}

/// POST /api/folio/assets/:id/manager/invite
async fn invite_asset_manager(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<InviteManagerBody>,
) -> Result<impl IntoResponse, StatusCode> {
    reject_hired_pm_admin(&db, current_user.id).await?;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let _asset = AssetService::get_unit(&db, tenant_id, asset_id)
        .await
        .map_err(|e| {
            let msg = e.to_string();
            if msg.contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let invite = ManagementDelegationService::create_pm_invite(
        &db,
        current_user.id,
        asset_id,
        body.portfolio_scope,
        body.label,
    )
    .await
    .map_err(|e| {
        tracing::warn!(%asset_id, "invite_asset_manager: {e}");
        if e.contains("already") {
            StatusCode::CONFLICT
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok((StatusCode::CREATED, axum::response::Json(invite)))
}

/// DELETE /api/folio/assets/:id/manager/invite
async fn cancel_asset_manager_invite(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    reject_hired_pm_admin(&db, current_user.id).await?;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let _asset = AssetService::get_unit(&db, tenant_id, asset_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ManagementDelegationService::cancel_pending_invite(&db, current_user.id, asset_id)
        .await
        .map_err(|e| {
            tracing::warn!(%asset_id, "cancel_asset_manager_invite: {e}");
            if e.contains("No pending") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/folio/assets/:id/manager
async fn revoke_asset_manager(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(asset_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    reject_hired_pm_admin(&db, current_user.id).await?;
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let _asset = AssetService::get_unit(&db, tenant_id, asset_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    ManagementDelegationService::revoke_manager_for_asset(&db, current_user.id, asset_id)
        .await
        .map_err(|e| {
            tracing::warn!(%asset_id, "revoke_asset_manager: {e}");
            if e.contains("No active") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}

async fn reject_hired_pm_admin(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(), StatusCode> {
    let hired = ManagementDelegationService::is_hired_property_manager(db, user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if hired {
        tracing::warn!(%user_id, "hired PM blocked from account-admin manager action");
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}
// ── GET /api/folio/assets/map ─────────────────────────────────────────────────
//
// Returns all tenant assets with lat/lon stored in attributes.coordinates.
// Used to render the portfolio map.

#[derive(serde::Serialize)]
struct MapPin {
    pub id: uuid::Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address_line_1: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    /// Open / in-progress / scheduled maintenance on this asset or its children.
    pub open_wo_count: i64,
    /// Soonest scheduled_at among open maintenance cases (ISO), if any.
    pub next_wo_at: Option<chrono::DateTime<chrono::Utc>>,
    pub str_eligible: bool,
    /// True when draft/active/pending lease occupies this asset (or a child rolled up).
    pub has_occupying_lease: bool,
}

async fn list_assets_map(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::types::pm::PmCaseType;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use std::collections::HashMap;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let assets = crate::entities::atlas_asset::Entity::find()
        .filter(crate::entities::atlas_asset::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_assets_map error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let grants = ManagementDelegationService::accessible_asset_ids(&db, current_user.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let assets = AssetService::filter_by_asset_grants(assets, grants.as_deref());

    // Child → parent for rolling unit WOs up to a building pin.
    let parent_of: HashMap<uuid::Uuid, uuid::Uuid> = assets
        .iter()
        .filter_map(|a| a.parent_asset_id.map(|p| (a.id, p)))
        .collect();

    let cases = crate::entities::atlas_case::Entity::find()
        .filter(crate::entities::atlas_case::Column::TenantId.eq(tenant_id))
        .filter(
            crate::entities::atlas_case::Column::CaseType.eq(PmCaseType::Maintenance.to_string()),
        )
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_assets_map maintenance error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let mut open_wo_by_asset: HashMap<uuid::Uuid, i64> = HashMap::new();
    let mut next_wo_by_asset: HashMap<uuid::Uuid, chrono::DateTime<chrono::Utc>> = HashMap::new();
    for c in cases {
        let status = c.status.to_ascii_lowercase();
        let is_open = matches!(
            status.as_str(),
            "open" | "in_progress" | "scheduled" | "assigned"
        );
        if !is_open {
            continue;
        }
        let Some(aid) = c.asset_id else { continue };
        // Prefer parent pin when the WO is on a unit.
        let pin_key = parent_of.get(&aid).copied().unwrap_or(aid);
        *open_wo_by_asset.entry(pin_key).or_insert(0) += 1;
        *open_wo_by_asset.entry(aid).or_insert(0) += 1;
        if let Some(when) = c.scheduled_at {
            next_wo_by_asset
                .entry(pin_key)
                .and_modify(|cur| {
                    if when < *cur {
                        *cur = when;
                    }
                })
                .or_insert(when);
            next_wo_by_asset
                .entry(aid)
                .and_modify(|cur| {
                    if when < *cur {
                        *cur = when;
                    }
                })
                .or_insert(when);
        }
    }

    let occupying = LeaseService::occupying_asset_ids(&db, tenant_id)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_assets_map occupancy: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let pins: Vec<MapPin> = assets
        .into_iter()
        .filter_map(|a| {
            // Coordinates stored in attributes.coordinates.{lat,lng}
            let attrs = a.attributes.as_ref()?;
            let coords = attrs.get("coordinates")?;
            let lat = coords.get("lat").and_then(|v| v.as_f64())?;
            let lng = coords.get("lng").and_then(|v| v.as_f64())?;
            // Skip zero-zero pins (unset)
            if lat == 0.0 && lng == 0.0 {
                return None;
            }
            let self_occ = occupying.contains(&a.id);
            let child_occ = a.parent_asset_id.is_none()
                && occupying.iter().any(|oid| parent_of.get(oid) == Some(&a.id));
            Some(MapPin {
                id: a.id,
                name: a.name,
                asset_type: a.asset_type,
                status: a.status,
                latitude: lat,
                longitude: lng,
                address_line_1: a.address_line_1,
                city: a.city,
                state_province: a.state_province,
                postal_code: a.postal_code,
                open_wo_count: *open_wo_by_asset.get(&a.id).unwrap_or(&0),
                next_wo_at: next_wo_by_asset.get(&a.id).copied(),
                str_eligible: a.str_eligible,
                has_occupying_lease: self_occ || child_occ,
            })
        })
        .collect();

    Ok(axum::response::Json(pins))
}
