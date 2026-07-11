//! # G22 Record Relationship HTTP handlers — Folio (Phase 6)
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST   | /api/folio/relationships | Create a relationship |
//! | DELETE | /api/folio/relationships | Delete a specific relationship |
//! | GET    | /api/folio/relationships/from/{entity_type}/{entity_id} | Forward traversal (targets) |
//! | GET    | /api/folio/relationships/to/{entity_type}/{entity_id} | Reverse traversal (sources / Related List) |
//! | GET    | /api/folio/relationships/all/{entity_type}/{entity_id} | All relationships for entity |

use axum::{
    Router,
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::record_relationship::{CreateRelationshipPayload, RecordRelationshipService},
};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/relationships",
            post(create_relationship).delete(delete_relationship),
        )
        .route(
            "/api/folio/relationships/from/{entity_type}/{entity_id}",
            get(find_targets),
        )
        .route(
            "/api/folio/relationships/to/{entity_type}/{entity_id}",
            get(find_sources),
        )
        .route(
            "/api/folio/relationships/all/{entity_type}/{entity_id}",
            get(find_all_for_entity),
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
struct CreateRelationshipRequest {
    source_entity_type: String,
    source_entity_id: Uuid,
    target_entity_type: String,
    target_entity_id: Uuid,
    relationship_type: String,
    inverse_label: Option<String>,
    relationship_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct DeleteRelationshipRequest {
    source_entity_type: String,
    source_entity_id: Uuid,
    target_entity_type: String,
    target_entity_id: Uuid,
    relationship_type: String,
}

#[derive(Debug, Deserialize)]
struct RelationshipTypeQuery {
    relationship_type: String,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn create_relationship(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<CreateRelationshipRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let payload = CreateRelationshipPayload {
        source_entity_type: req.source_entity_type,
        source_entity_id: req.source_entity_id,
        target_entity_type: req.target_entity_type,
        target_entity_id: req.target_entity_id,
        relationship_type: req.relationship_type,
        inverse_label: req.inverse_label,
        relationship_metadata: req.relationship_metadata,
        created_by_user_id: Some(current_user.id),
    };

    match RecordRelationshipService::upsert(&db, tenant_id, payload).await {
        Ok(rel) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "relationship": rel })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

async fn delete_relationship(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(req): Json<DeleteRelationshipRequest>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    match RecordRelationshipService::delete(
        &db,
        tenant_id,
        &req.source_entity_type,
        req.source_entity_id,
        &req.target_entity_type,
        req.target_entity_id,
        &req.relationship_type,
    )
    .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Forward traversal: "find all records that this entity links TO".
/// e.g. GET /api/folio/relationships/from/atlas_campaigns/{id}?relationship_type=promotes
async fn find_targets(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(q): Query<RelationshipTypeQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    match RecordRelationshipService::find_targets(
        &db,
        tenant_id,
        &entity_type,
        entity_id,
        &q.relationship_type,
    )
    .await
    {
        Ok(rels) => Json(serde_json::json!({ "relationships": rels })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Reverse traversal (Related List): "find all records that link TO this entity".
/// e.g. GET /api/folio/relationships/to/atlas_assets/{id}?relationship_type=promotes
async fn find_sources(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    Query(q): Query<RelationshipTypeQuery>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    match RecordRelationshipService::find_sources(
        &db,
        tenant_id,
        &entity_type,
        entity_id,
        &q.relationship_type,
    )
    .await
    {
        Ok(rels) => Json(serde_json::json!({ "relationships": rels })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// All relationships of any type touching an entity.
/// Powers the "Related Records" side panel.
async fn find_all_for_entity(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> impl IntoResponse {
    let tenant_id = match resolve_tenant_id(&db, current_user.id).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    match RecordRelationshipService::find_all_for_entity(&db, tenant_id, &entity_type, entity_id)
        .await
    {
        Ok(rels) => Json(serde_json::json!({ "relationships": rels })).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
