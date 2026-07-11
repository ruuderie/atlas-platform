//! Property value history handler — G-10 extension.
//!
//! # Routes
//!
//! ```ignore
//! POST /api/folio/properties/:id/value
//!      Log a new valuation entry for an asset.
//!      Auth: property_owner_lite or landlord (must own the asset).
//!      Body: LogValueInput
//!      -> 201 { "id": uuid }
//!
//! GET  /api/folio/properties/:id/value-history
//!      Return the full value history for an asset, ordered by valued_on DESC.
//!      Each entry includes source, value_cents, currency_code, source_ref, note, valued_on.
//!      Auth: property_owner_lite or landlord.
//!      -> 200 [ValueHistoryEntry]
//! ```

use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use chrono::NaiveDate;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::pm::PropertyValueSource;

// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/properties/{id}/value", post(log_property_value))
        .route(
            "/api/folio/properties/{id}/value-history",
            get(get_value_history),
        )
}

// ── Request / Response types ──────────────────────────────────────────────────

/// Request body for logging a new property valuation entry.
#[derive(Debug, Deserialize)]
pub struct LogValueInput {
    /// Source type — must match a `PropertyValueSource` variant string.
    /// Valid values: "manual", "purchase_price", "zillow_avm", "county_record",
    ///               "certified_appraisal", "bank_appraisal", "agent_cma".
    pub source: String,
    /// Optional external reference — URL, document ID, AVM report ID, appraiser name.
    /// Stored verbatim; not validated by the server.
    pub source_ref: Option<String>,
    /// Property value in the minor unit of the currency (cents for USD).
    /// Must be positive. Example: $450,000 → 45_000_000.
    pub value_cents: i64,
    /// ISO 4217 currency code. Default: "USD".
    pub currency_code: Option<String>,
    /// The date this valuation applies to. Format: "YYYY-MM-DD".
    /// May differ from today (e.g. logging a past appraisal).
    pub valued_on: NaiveDate,
    /// Optional free-text note (e.g. "after new roof installation").
    pub note: Option<String>,
}

/// A single value history entry returned in the history list.
#[derive(Debug, Serialize)]
pub struct ValueHistoryEntry {
    pub id: Uuid,
    pub source: String,
    pub source_ref: Option<String>,
    pub value_cents: i64,
    pub currency_code: String,
    pub valued_on: NaiveDate,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct LogValueResponse {
    pub id: Uuid,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/properties/:id/value
///
/// Validates the source enum, inserts a new `atlas_asset_value_history` row,
/// and returns the new entry ID.
pub async fn log_property_value(
    State(db): State<DatabaseConnection>,
    Path(asset_id): Path<Uuid>,
    Json(body): Json<LogValueInput>,
) -> impl IntoResponse {
    // Validate source enum
    let source =
        match PropertyValueSource::try_from(body.source.clone()) {
            Ok(s) => s,
            Err(_) => {
                return (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(serde_json::json!({
                "error": format!(
                    "Invalid source '{}'. Valid values: manual, purchase_price, zillow_avm, \
                     county_record, certified_appraisal, bank_appraisal, agent_cma.",
                    body.source
                )
            }))).into_response();
            }
        };

    if body.value_cents <= 0 {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            axum::Json(serde_json::json!({
                "error": "value_cents must be a positive integer (e.g. $450,000 = 45000000)."
            })),
        )
            .into_response();
    }

    let currency = body
        .currency_code
        .as_deref()
        .unwrap_or("USD")
        .to_uppercase();
    let entry_id = Uuid::new_v4();
    let valued_on = body.valued_on.format("%Y-%m-%d").to_string();
    let source_str = source.to_string();

    // Raw SQL insert — atlas_asset_value_history was created in migration m20261015.
    // tenant_id and user_id will be injected from auth middleware in production;
    // using gen_random_uuid() placeholders until auth middleware is wired.
    let sql = format!(
        "INSERT INTO atlas_asset_value_history \
         (id, asset_id, tenant_id, user_id, source, source_ref, value_cents, currency_code, valued_on, note, created_at) \
         VALUES ('{id}'::uuid, '{asset_id}'::uuid, gen_random_uuid(), gen_random_uuid(), \
                 '{source}', {source_ref}, {value_cents}, '{currency}', '{valued_on}'::date, {note}, NOW());",
        id = entry_id,
        asset_id = asset_id,
        source = source_str,
        source_ref = body
            .source_ref
            .as_deref()
            .map(|s| format!("'{}'", s.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string()),
        value_cents = body.value_cents,
        currency = currency,
        valued_on = valued_on,
        note = body
            .note
            .as_deref()
            .map(|n| format!("'{}'", n.replace('\'', "''")))
            .unwrap_or_else(|| "NULL".to_string()),
    );

    match db
        .execute(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await
    {
        Ok(_) => {
            tracing::info!(
                event = "property_value.logged",
                asset_id = %asset_id,
                source = %source,
                value_cents = body.value_cents,
            );
            (
                StatusCode::CREATED,
                axum::Json(LogValueResponse { id: entry_id }),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "log_property_value: insert failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

/// GET /api/folio/properties/:id/value-history
///
/// Returns all valuation entries for the asset ordered newest-first.
/// The frontend uses this to render the source-keyed time-series chart
/// on the po_property_value page.
pub async fn get_value_history(
    State(db): State<DatabaseConnection>,
    Path(asset_id): Path<Uuid>,
) -> impl IntoResponse {
    use sea_orm::ConnectionTrait;

    let sql = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"SELECT id, source, source_ref, value_cents, currency_code,
                  valued_on, note, created_at
           FROM atlas_asset_value_history
           WHERE asset_id = $1
           ORDER BY valued_on DESC, created_at DESC"#,
        [asset_id.into()],
    );

    match db.query_all(sql).await {
        Ok(rows) => {
            let entries: Vec<ValueHistoryEntry> = rows
                .into_iter()
                .filter_map(|row| {
                    Some(ValueHistoryEntry {
                        id: row.try_get("", "id").ok()?,
                        source: row.try_get("", "source").ok()?,
                        source_ref: row.try_get("", "source_ref").ok().unwrap_or(None),
                        value_cents: row.try_get("", "value_cents").ok()?,
                        currency_code: row.try_get("", "currency_code").ok()?,
                        valued_on: row.try_get("", "valued_on").ok()?,
                        note: row.try_get("", "note").ok().unwrap_or(None),
                        created_at: row
                            .try_get::<chrono::DateTime<chrono::Utc>>("", "created_at")
                            .ok()
                            .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
                            .unwrap_or_default(),
                    })
                })
                .collect();

            (StatusCode::OK, axum::Json(entries)).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, asset_id = %asset_id, "get_value_history: query failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
