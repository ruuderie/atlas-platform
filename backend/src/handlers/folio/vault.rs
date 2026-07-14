//! Folio — Document Vault handler.
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/folio/vault/documents` | Register a document (post-R2 upload) |
//! | GET  | `/api/folio/vault/documents` | List documents for a tenant |
//! | GET  | `/api/folio/vault/documents/{id}` | Get a single document |
//!
//! # Upload flow
//!
//! The client uploads directly to Cloudflare R2 (presigned PUT URL, generated
//! by a future `/api/folio/vault/presign` endpoint in Phase 7).
//! Once the upload completes, the client calls `POST /api/folio/vault/documents`
//! with the confirmed `r2_key` — this creates the `atlas_document` registry entry.
//!
//! # Data source
//!
//! `atlas_documents` (G-14) + `attachment` (G-02).
//! No net-new tables.

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
use crate::services::pm::vault::{PmDocumentType, VaultService};

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route(
            "/api/folio/vault/documents",
            get(list_documents).post(register_document),
        )
        .route("/api/folio/vault/documents/{id}", get(get_document))
}

// ── Request / response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct RegisterDocumentInput {
    /// The entity type this document is attached to.
    /// e.g. "atlas_contracts", "atlas_assets", "atlas_applications"
    pub entity_type: String,
    pub entity_id: Uuid,
    /// Typed document category. e.g. "lease_agreement", "str_permit", "id_document"
    pub document_type: String,
    /// The R2 object key (path within the bucket). e.g. "pm/leases/tenant_xyz/lease.pdf"
    pub r2_key: String,
    /// MIME type. Defaults to "application/octet-stream" if omitted.
    pub mime_type: Option<String>,
    /// File size in bytes.
    pub size_bytes: Option<i64>,
}

#[derive(Debug, Serialize)]
struct RegisterDocumentResponse {
    pub id: Uuid,
}

#[derive(Debug, Serialize)]
struct DocumentSummary {
    pub id: Uuid,
    pub document_category: String,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub is_counterparty_visible: bool,
    pub requires_signature: bool,
    pub is_signed: bool,
    pub version_number: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct ListDocumentsQuery {
    /// Filter by related entity type (optional).
    pub entity_type: Option<String>,
    /// Filter by related entity ID (optional).
    pub entity_id: Option<Uuid>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/vault/documents
///
/// Register a document in the PM vault after the file has been uploaded to R2.
async fn register_document(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<RegisterDocumentInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let doc_type = PmDocumentType::try_from(input.document_type.clone()).map_err(|_| {
        tracing::warn!(
            "register_document: unknown document_type '{}'",
            input.document_type
        );
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    let id = VaultService::register_document_full(
        &db,
        tenant_id,
        &input.entity_type,
        input.entity_id,
        doc_type,
        &input.r2_key,
        input
            .mime_type
            .as_deref()
            .unwrap_or("application/octet-stream"),
        input.size_bytes,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "register_document error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(RegisterDocumentResponse { id }),
    ))
}

/// GET /api/folio/vault/documents
///
/// List documents for the tenant, optionally filtered by entity_type/entity_id.
async fn list_documents(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let mut finder = crate::entities::atlas_document::Entity::find()
        .filter(crate::entities::atlas_document::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_document::Column::AppNamespace.eq("folio"))
        .order_by_desc(crate::entities::atlas_document::Column::CreatedAt);

    if let Some(et) = &query.entity_type {
        finder = finder
            .filter(crate::entities::atlas_document::Column::RelatedEntityType.eq(et.as_str()));
    }
    if let Some(eid) = query.entity_id {
        finder = finder.filter(crate::entities::atlas_document::Column::RelatedEntityId.eq(eid));
    }

    let documents = finder.all(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, "list_documents DB error: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let summaries: Vec<DocumentSummary> = documents
        .into_iter()
        .map(|d| DocumentSummary {
            id: d.id,
            document_category: d.document_category,
            related_entity_type: d.related_entity_type,
            related_entity_id: d.related_entity_id,
            is_counterparty_visible: d.is_counterparty_visible,
            requires_signature: d.requires_signature,
            is_signed: d.is_signed,
            version_number: d.version_number,
            created_at: d.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

/// GET /api/folio/vault/documents/{id}
async fn get_document(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(document_id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let document = crate::entities::atlas_document::Entity::find_by_id(document_id)
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, %document_id, "get_document DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Tenant isolation.
    if document.tenant_id != tenant_id {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(axum::response::Json(DocumentSummary {
        id: document.id,
        document_category: document.document_category,
        related_entity_type: document.related_entity_type,
        related_entity_id: document.related_entity_id,
        is_counterparty_visible: document.is_counterparty_visible,
        requires_signature: document.requires_signature,
        is_signed: document.is_signed,
        version_number: document.version_number,
        created_at: document.created_at,
    }))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
