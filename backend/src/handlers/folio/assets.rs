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
//! ```

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::user;
use crate::services::pm::asset::{AssetService, CreateUnitInput};
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

/// GET /api/folio/assets
async fn list_assets(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let assets = crate::entities::atlas_asset::Entity::find()
        .filter(crate::entities::atlas_asset::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_assets error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

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
            latitude: input.latitude,
            longitude: input.longitude,
        },
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_asset error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

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

    let asset = AssetService::get_unit(&db, tenant_id, asset_id)
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

    // Verify parent exists and belongs to tenant.
    let _parent = AssetService::get_unit(&db, tenant_id, asset_id)
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

    let _parent = AssetService::get_unit(&db, tenant_id, asset_id)
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

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
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
            })
        })
        .collect();

    Ok(axum::response::Json(pins))
}
