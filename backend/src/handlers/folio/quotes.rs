//! # G24 Quote HTTP handlers — Folio
//!
//! Routes:
//!
//! | Method | Path                                  | Description                          |
//! |--------|---------------------------------------|--------------------------------------|
//! | POST   | /api/folio/quotes                     | Create quote (with line items)       |
//! | GET    | /api/folio/quotes                     | List quotes (filterable)             |
//! | GET    | /api/folio/quotes/{id}                | Get single quote                     |
//! | GET    | /api/folio/quotes/{id}/line-items     | List line items for a quote          |
//! | POST   | /api/folio/quotes/{id}/status         | Transition status (state machine)    |
//! | POST   | /api/folio/quotes/{id}/revise         | Clone as new revision (supersedes)   |
//! | POST   | /api/folio/quotes/{id}/convert        | Mark converted → reservation         |

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::user,
    services::pm::quote::{
        CreateLineItemPayload, CreateQuotePayload, QuoteFilter, QuoteService,
    },
    types::pm::{QuoteLineItemType, QuoteStatus},
};

// ── Route constructor ─────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/quotes", post(create_quote).get(list_quotes))
        .route("/api/folio/quotes/{id}", get(get_quote))
        .route("/api/folio/quotes/{id}/line-items", get(list_line_items))
        .route("/api/folio/quotes/{id}/status", post(transition_status))
        .route("/api/folio/quotes/{id}/revise", post(revise_quote))
        .route("/api/folio/quotes/{id}/convert", post(convert_to_reservation))
}

// ── Tenant resolution ─────────────────────────────────────────────────────────

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

// ── Request / Response types ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ListQuotesQuery {
    status: Option<String>,
    subject_entity_type: Option<String>,
    subject_entity_id: Option<Uuid>,
    campaign_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
struct CreateQuoteInput {
    title: String,
    subject_entity_type: Option<String>,
    subject_entity_id: Option<Uuid>,
    recipient_user_id: Option<Uuid>,
    recipient_email: Option<String>,
    recipient_name: Option<String>,
    campaign_id: Option<Uuid>,
    catalog_entry_id: Option<Uuid>,
    quote_number: Option<String>,
    notes: Option<String>,
    currency: Option<String>,
    valid_from: Option<chrono::DateTime<chrono::Utc>>,
    valid_until: Option<chrono::DateTime<chrono::Utc>>,
    quote_metadata: Option<serde_json::Value>,
    line_items: Vec<LineItemInput>,
}

#[derive(Debug, Deserialize)]
struct LineItemInput {
    line_item_type: String,
    catalog_entry_id: Option<Uuid>,
    description: String,
    quantity: i32,
    unit_price_cents: i64,
    discount_basis_points: Option<i32>,
    sort_order: Option<i32>,
    line_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct TransitionStatusInput {
    status: String,
}

#[derive(Debug, Deserialize)]
struct ConvertInput {
    reservation_id: Uuid,
}

#[derive(Debug, Serialize)]
struct QuoteResponse {
    id: Uuid,
    status: String,
    title: String,
    total_cents: i64,
    currency: String,
    revision_number: i32,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn create_quote(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(body): Json<CreateQuoteInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Validate and convert line item types at the handler boundary.
    let mut line_items = Vec::new();
    for li in body.line_items {
        let line_item_type = QuoteLineItemType::try_from(li.line_item_type.as_str())
            .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
        line_items.push(CreateLineItemPayload {
            line_item_type,
            catalog_entry_id: li.catalog_entry_id,
            description: li.description,
            quantity: li.quantity,
            unit_price_cents: li.unit_price_cents,
            discount_basis_points: li.discount_basis_points,
            sort_order: li.sort_order,
            line_metadata: li.line_metadata,
        });
    }

    let payload = CreateQuotePayload {
        title: body.title,
        subject_entity_type: body.subject_entity_type,
        subject_entity_id: body.subject_entity_id,
        recipient_user_id: body.recipient_user_id,
        recipient_email: body.recipient_email,
        recipient_name: body.recipient_name,
        campaign_id: body.campaign_id,
        catalog_entry_id: body.catalog_entry_id,
        quote_number: body.quote_number,
        notes: body.notes,
        currency: body.currency,
        valid_from: body.valid_from,
        valid_until: body.valid_until,
        quote_metadata: body.quote_metadata,
        created_by_user_id: Some(current_user.id),
        line_items,
    };

    let quote = QuoteService::create(&db, tenant_id, payload)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(QuoteResponse {
            id: quote.id,
            status: quote.status,
            title: quote.title,
            total_cents: quote.total_cents,
            currency: quote.currency,
            revision_number: quote.revision_number,
        }),
    ))
}

async fn list_quotes(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Query(q): Query<ListQuotesQuery>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let status = q
        .status
        .as_deref()
        .map(QuoteStatus::try_from)
        .transpose()
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let filter = QuoteFilter {
        status,
        subject_entity_type: q.subject_entity_type,
        subject_entity_id: q.subject_entity_id,
        campaign_id: q.campaign_id,
    };

    let quotes = QuoteService::list(&db, tenant_id, filter)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(quotes))
}

async fn get_quote(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let quote = QuoteService::get(&db, tenant_id, id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(quote))
}

async fn list_line_items(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let items = QuoteService::list_line_items(&db, tenant_id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(items))
}

async fn transition_status(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<TransitionStatusInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let new_status =
        QuoteStatus::try_from(body.status.as_str()).map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;

    let quote = QuoteService::transition_status(&db, tenant_id, id, new_status)
        .await
        .map_err(|e| {
            // Distinguish invalid-transition (422) from not-found (404)
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("Invalid quote transition") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json(QuoteResponse {
        id: quote.id,
        status: quote.status,
        title: quote.title,
        total_cents: quote.total_cents,
        currency: quote.currency,
        revision_number: quote.revision_number,
    }))
}

async fn revise_quote(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let revision = QuoteService::revise(&db, tenant_id, id, Some(current_user.id))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(QuoteResponse {
            id: revision.id,
            status: revision.status,
            title: revision.title,
            total_cents: revision.total_cents,
            currency: revision.currency,
            revision_number: revision.revision_number,
        }),
    ))
}

async fn convert_to_reservation(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Path(id): Path<Uuid>,
    Json(body): Json<ConvertInput>,
) -> Result<impl axum::response::IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;
    let quote =
        QuoteService::convert_to_reservation(&db, tenant_id, id, body.reservation_id)
            .await
            .map_err(|e| {
                if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else if e.to_string().contains("Invalid quote transition") {
                    StatusCode::UNPROCESSABLE_ENTITY
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            })?;

    Ok(Json(QuoteResponse {
        id: quote.id,
        status: quote.status,
        title: quote.title,
        total_cents: quote.total_cents,
        currency: quote.currency,
        revision_number: quote.revision_number,
    }))
}
