//! POST   /api/folio/marketplace/vendors/:id/endorse — endorse a vendor
//! DELETE /api/folio/marketplace/vendors/:id/endorse — retract endorsement
//!
//! An endorsement is a G-22 `atlas_record_relationships` row:
//!   source_entity_type = "atlas_account"           (the landlord's account)
//!   source_entity_id   = landlord_account_id
//!   target_entity_type = "atlas_service_providers"
//!   target_entity_id   = vendor_service_provider_id
//!   relationship_type  = "marketplace_endorsement"
//!
//! Constraints:
//! - A landlord can endorse a vendor only once (upsert / unique on source+target+type).
//! - A landlord cannot endorse their own tenant's private vendors that they haven't
//!   yet published to the marketplace — `is_marketplace_visible` must be true.
//! - Endorsement count is visible cross-tenant; identity of endorsers is NOT.

use axum::{
    Extension, Json, Router,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, post},
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::Serialize;
use uuid::Uuid;
use chrono::Utc;

use crate::extractors::tenant::TenantContext;
use crate::entities::atlas_record_relationship;

/// Relationship type slug for marketplace endorsements.
const ENDORSEMENT_TYPE: &str = "marketplace_endorsement";
const VENDOR_ENTITY_TYPE: &str = "atlas_service_providers";

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/marketplace/vendors/:id/endorse",
            post(endorse_vendor).delete(retract_endorsement),
        )
}

#[derive(Serialize)]
pub struct EndorseResponse {
    pub endorsement_id:    Uuid,
    pub vendor_id:         Uuid,
    pub endorsing_account: Uuid,
}

// ── Endorse ───────────────────────────────────────────────────────────────────

/// POST — landlord endorses a marketplace vendor.
///
/// Requires: the landlord must have an account in this tenant.
/// The endorsement is anonymous to other users (only the count is public).
async fn endorse_vendor(
    ctx: TenantContext,
    Path(vendor_id): Path<Uuid>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    // ── Validate vendor is marketplace-visible ────────────────────────────────
    let _vendor = match validate_marketplace_vendor(&db, vendor_id).await {
        Ok(v) => v,
        Err(status) => return status.into_response(),
    };

    // ── Resolve the landlord's account_id ────────────────────────────────────
    let landlord_account_id = match resolve_primary_account(&db, ctx.tenant_id).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "endorse: could not resolve landlord account");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // ── Check for existing endorsement (idempotent) ───────────────────────────
    let existing = atlas_record_relationship::Entity::find()
        .filter(atlas_record_relationship::Column::SourceEntityType.eq("atlas_account"))
        .filter(atlas_record_relationship::Column::SourceEntityId.eq(landlord_account_id))
        .filter(atlas_record_relationship::Column::TargetEntityType.eq(VENDOR_ENTITY_TYPE))
        .filter(atlas_record_relationship::Column::TargetEntityId.eq(vendor_id))
        .filter(atlas_record_relationship::Column::RelationshipType.eq(ENDORSEMENT_TYPE))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "endorse: lookup failed");
            StatusCode::INTERNAL_SERVER_ERROR
        });

    match existing {
        Err(status) => return status.into_response(),
        Ok(Some(row)) => {
            // Already endorsed — return 200 with existing record (idempotent)
            return Json(EndorseResponse {
                endorsement_id:    row.id,
                vendor_id,
                endorsing_account: landlord_account_id,
            })
            .into_response();
        }
        Ok(None) => {} // proceed
    }

    // ── Create endorsement ────────────────────────────────────────────────────
    let new_rel = atlas_record_relationship::ActiveModel {
        id:                    Set(Uuid::new_v4()),
        // Endorsements live in the endorser's tenant (not vendor's tenant)
        tenant_id:             Set(ctx.tenant_id),
        source_entity_type:    Set("atlas_account".to_string()),
        source_entity_id:      Set(landlord_account_id),
        target_entity_type:    Set(VENDOR_ENTITY_TYPE.to_string()),
        target_entity_id:      Set(vendor_id),
        relationship_type:     Set(ENDORSEMENT_TYPE.to_string()),
        inverse_label:         Set(Some("endorsed_by".to_string())),
        relationship_metadata: Set(Some(serde_json::json!({
            "endorsed_at": Utc::now().to_rfc3339(),
            "context": "marketplace"
        }))),
        created_by_user_id:    Set(Some(ctx.user_id)),
        created_at:            Set(Utc::now()),
    };

    match new_rel.insert(&db).await {
        Ok(row) => {
            tracing::info!(
                %vendor_id,
                tenant_id = %ctx.tenant_id,
                user_id = %ctx.user_id,
                "marketplace: vendor endorsed"
            );
            (
                StatusCode::CREATED,
                Json(EndorseResponse {
                    endorsement_id:    row.id,
                    vendor_id,
                    endorsing_account: landlord_account_id,
                }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "endorse: insert failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Retract endorsement ───────────────────────────────────────────────────────

async fn retract_endorsement(
    ctx: TenantContext,
    Path(vendor_id): Path<Uuid>,
    Extension(db): Extension<DatabaseConnection>,
) -> impl IntoResponse {
    let landlord_account_id = match resolve_primary_account(&db, ctx.tenant_id).await {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "retract: could not resolve landlord account");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let existing = atlas_record_relationship::Entity::find()
        .filter(atlas_record_relationship::Column::SourceEntityType.eq("atlas_account"))
        .filter(atlas_record_relationship::Column::SourceEntityId.eq(landlord_account_id))
        .filter(atlas_record_relationship::Column::TargetEntityType.eq(VENDOR_ENTITY_TYPE))
        .filter(atlas_record_relationship::Column::TargetEntityId.eq(vendor_id))
        .filter(atlas_record_relationship::Column::RelationshipType.eq(ENDORSEMENT_TYPE))
        .one(&db)
        .await;

    match existing {
        Ok(Some(row)) => {
            let active: atlas_record_relationship::ActiveModel = row.into();
            match active.delete(&db).await {
                Ok(_) => {
                    tracing::info!(%vendor_id, "marketplace: endorsement retracted");
                    StatusCode::NO_CONTENT.into_response()
                }
                Err(e) => {
                    tracing::error!(error = %e, "retract: delete failed");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, "retract: lookup failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

async fn validate_marketplace_vendor(
    db: &DatabaseConnection,
    vendor_id: Uuid,
) -> Result<crate::entities::atlas_service_provider::Model, StatusCode> {
    crate::entities::atlas_service_provider::Entity::find_by_id(vendor_id)
        .one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter(|sp| sp.is_marketplace_visible)
        .ok_or(StatusCode::NOT_FOUND)
}

/// Resolve the primary account_id for a tenant (used as the endorsement source).
///
/// In production: each landlord/org has exactly one primary `account` row created
/// during onboarding. This helper finds it.
async fn resolve_primary_account(
    db: &DatabaseConnection,
    tenant_id: Uuid,
) -> Result<Uuid, sea_orm::DbErr> {
    let account = crate::entities::atlas_account::Entity::find()
        .filter(crate::entities::atlas_account::Column::TenantId.eq(tenant_id))
        .one(db)
        .await?
        .ok_or_else(|| sea_orm::DbErr::Custom("no account found for tenant".to_string()))?;

    Ok(account.id)
}
