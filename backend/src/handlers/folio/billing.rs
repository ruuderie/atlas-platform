//! Folio — Billing handler (Phase 3).
//!
//! Implements rent invoice creation and BTC payment submission via the
//! `PaymentRailAdapter` trait system and `PmLedgerService`.
//!
//! # Route surface
//!
//! | Method | Path | Description |
//! |--------|------|-------------|
//! | POST | `/api/folio/billing/invoice/btc` | Submit a BTC txid for a pending invoice |
//! | GET  | `/api/folio/billing/invoice/btc/audit` | List BTC invoice statuses for tenant |
//! | POST | `/api/folio/billing/invoice/fiat` | Create Stripe/PIX/Kelviq payment intent |
//! | POST | `/api/folio/billing/invoice/verify` | Submit manual payment receipt (Phase 4) |
//!
//! # Rails dispatch
//!
//! For `/invoice/fiat` the handler:
//!   1. Reads the tenant's active `atlas_payment_credentials` row for the requested rail.
//!   2. Decrypts the credentials via `payment_rail::decrypt_credentials()`.
//!   3. Calls `payment_rail::resolve_adapter(credential_type, credentials)`.
//!   4. Creates a pending ledger entry via `PmLedgerService::create_pending()`.
//!   5. Calls `adapter.create_invoice()`.
//!   6. Returns `201 { "ledger_entry_id", "payment_instructions" }`.

use axum::{
    Router,
    body::Bytes,
    extract::{Extension, Json},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::{atlas_payment_credential, user};
use crate::services::pm::ledger::PmLedgerService;
use crate::services::pm::payment_rail::{decrypt_credentials, resolve_adapter};
use crate::types::pm::Currency;

// ── Shared helpers ────────────────────────────────────────────────────────────

async fn resolve_tenant_id(db: &DatabaseConnection, user_id: Uuid) -> Result<Uuid, StatusCode> {
    crate::extractors::tenant::resolve_tenant_id(db, user_id).await
}
// ── Route registration ────────────────────────────────────────────────────────

pub fn authenticated_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/billing/invoice/btc", post(submit_btc_txid))
        .route("/api/folio/billing/invoice/btc/audit", get(btc_audit))
        .route("/api/folio/billing/invoice/fiat", post(create_fiat_invoice))
        .route(
            "/api/folio/billing/invoice/verify",
            post(verify_receipt_stub),
        )
        // ── Ledger routes (G-03 read + ad-hoc charge write) ─────────────────
        .route("/api/folio/ledger", get(list_ledger_entries))
        .route("/api/folio/ledger/charge", post(create_ad_hoc_charge))
}

/// Unauthenticated routes — webhook endpoints authenticate via their own
/// mechanism (HMAC signature or shared secret) rather than session auth.
pub fn public_routes_raw() -> Router<DatabaseConnection> {
    Router::new()
        .route("/api/folio/billing/webhook/stripe", post(stripe_webhook))
        .route(
            "/api/folio/billing/webhook/infinitepay",
            post(infinitepay_webhook),
        )
        .route("/api/folio/billing/webhook/kelviq", post(kelviq_webhook))
}

// ── Request / Response types ──────────────────────────────────────────────────

/// POST /api/folio/billing/invoice/btc
/// Body: tenant submits their BTC txid after broadcasting on-chain.
#[derive(Debug, Deserialize)]
struct SubmitBtcTxidInput {
    /// The ledger entry this payment is for.
    pub ledger_entry_id: Uuid,
    /// Bitcoin transaction ID — exactly 64 hex chars.
    pub txid: String,
}

#[derive(Debug, Serialize)]
struct SubmitBtcTxidResponse {
    pub ledger_entry_id: Uuid,
    pub status: &'static str,
    pub message: &'static str,
}

/// POST /api/folio/billing/invoice/fiat
#[derive(Debug, Deserialize)]
struct CreateFiatInvoiceInput {
    /// The entity being invoiced — typically an atlas_contract (lease).
    pub billable_entity_type: String,
    pub billable_entity_id: Uuid,
    /// Amount in smallest currency unit (cents / centavos / satoshis).
    pub gross_amount_cents: i64,
    /// ISO-4217 or "BTC" / "SAT"
    pub currency: String,
    /// Human-readable line item for the invoice description.
    pub description: String,
    /// Payment rail to use. Must match `atlas_payment_credentials.credential_type`.
    /// Examples: "stripe_connect_express", "pix_key", "kelviq"
    pub rail: String,
    pub due_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
struct CreateFiatInvoiceResponse {
    pub ledger_entry_id: Uuid,
    pub provider_invoice_id: String,
    pub payment_instructions: serde_json::Value,
    pub expires_in_seconds: Option<u64>,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// POST /api/folio/billing/invoice/btc
///
/// Tenant submits a BTC txid after broadcasting their on-chain payment.
/// Records the txid on the ledger entry and transitions status → processing.
/// The `pm_btc_mempool_poll` background job then polls for confirmation.
async fn submit_btc_txid(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<SubmitBtcTxidInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    PmLedgerService::record_tx_id(&db, input.ledger_entry_id, tenant_id, &input.txid)
        .await
        .map_err(|e| {
            tracing::warn!(
                ledger_entry_id = %input.ledger_entry_id, %tenant_id,
                txid = %input.txid,
                "submit_btc_txid: failed to record txid: {e:#}"
            );
            // 422 for invalid txid format, 404 for missing ledger entry
            if e.to_string().contains("invalid txid") {
                StatusCode::UNPROCESSABLE_ENTITY
            } else {
                StatusCode::NOT_FOUND
            }
        })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(SubmitBtcTxidResponse {
            ledger_entry_id: input.ledger_entry_id,
            status: "processing",
            message: "txid recorded — awaiting mempool confirmation",
        }),
    ))
}

/// GET /api/folio/billing/invoice/btc/audit
///
/// Returns recent BTC on-chain invoice statuses for the tenant.
/// Used by the landlord dashboard to surface in_mempool / confirmed / failed states.
async fn btc_audit(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let invoices = PmLedgerService::list_btc_invoices(&db, tenant_id, 50)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "btc_audit: ledger query failed: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(invoices))
}

/// POST /api/folio/billing/invoice/fiat
///
/// Creates a payment intent / invoice for Stripe, PIX, or Kelviq.
/// The handler resolves the tenant's active credential for the requested rail,
/// creates a pending ledger entry, then calls the adapter.
async fn create_fiat_invoice(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateFiatInvoiceInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    // Parse currency at the handler boundary.
    let currency = Currency::try_from(input.currency.clone()).map_err(|e| {
        tracing::warn!(%tenant_id, currency = %input.currency, "create_fiat_invoice: invalid currency: {e}");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    // Load the tenant's active credential for this rail.
    let credential = atlas_payment_credential::Entity::find()
        .filter(atlas_payment_credential::Column::TenantId.eq(tenant_id))
        .filter(atlas_payment_credential::Column::CredentialType.eq(&input.rail))
        .filter(atlas_payment_credential::Column::IsActive.eq(true))
        .one(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, rail = %input.rail, "create_fiat_invoice: DB error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            tracing::warn!(%tenant_id, rail = %input.rail, "create_fiat_invoice: no active credential for rail");
            StatusCode::UNPROCESSABLE_ENTITY
        })?;

    // Decrypt credentials (Phase 3: identity function; Phase 4: AES-256-GCM).
    let decrypted = decrypt_credentials(&credential.credentials_encrypted).map_err(|e| {
        tracing::error!(%tenant_id, "create_fiat_invoice: credential decryption failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Resolve the adapter for this credential type.
    let adapter = resolve_adapter(&credential.credential_type, &decrypted).map_err(|e| {
        tracing::warn!(%tenant_id, credential_type = %credential.credential_type, "create_fiat_invoice: no adapter: {e:#}");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    // Create the pending ledger entry BEFORE calling the adapter
    // so ledger_entry_id can be embedded in webhook metadata for idempotent reconciliation.
    let ledger_entry_id = PmLedgerService::create_pending(
        &db,
        tenant_id,
        &input.billable_entity_type,
        input.billable_entity_id,
        Some(current_user.id),
        input.gross_amount_cents,
        currency,
        adapter.credential_type(),
        input.due_date,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_fiat_invoice: ledger create failed: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Call the rail adapter to create the provider-side invoice.
    let result = adapter
        .create_invoice(
            ledger_entry_id,
            tenant_id,
            input.gross_amount_cents,
            &input.currency,
            &input.description,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                %tenant_id, %ledger_entry_id,
                rail = %input.rail,
                "create_fiat_invoice: adapter.create_invoice failed: {e:#}"
            );
            StatusCode::BAD_GATEWAY
        })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateFiatInvoiceResponse {
            ledger_entry_id,
            provider_invoice_id: result.provider_invoice_id,
            payment_instructions: result.payment_instructions,
            expires_in_seconds: result.expires_in_seconds,
        }),
    ))
}

/// POST /api/folio/billing/invoice/verify
/// Manual receipt upload — Phase 4 implementation.
async fn verify_receipt_stub(
    Extension(_db): Extension<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> impl IntoResponse {
    tracing::debug!("folio/billing/invoice/verify: Phase 4 implementation pending");
    StatusCode::NOT_IMPLEMENTED
}
// ── Stripe webhook handler ───────────────────────────────────────────────────────────────────────

/// POST /api/folio/billing/webhook/stripe
///
/// Receives incoming Stripe webhook events and reconciles the ledger.
/// Authenticates via `Stripe-Signature` header — no session required.
/// Returns `200 OK` immediately for all accepted/unhandled events to prevent
/// Stripe retry storms. Returns `400` only on signature verification failure.
async fn stripe_webhook(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();

    let raw_body = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match crate::services::pm::rails::stripe_connect::StripeConnectWebhookHandler::handle(
        &db,
        raw_body,
        signature,
        &webhook_secret,
    )
    .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            // Signature failure → 400 to tell Stripe it was a bad request.
            // Other errors → 500 so Stripe retries.
            if e.to_string().contains("signature") {
                tracing::warn!(error = %e, "Stripe webhook signature verification failed");
                StatusCode::BAD_REQUEST.into_response()
            } else {
                tracing::error!(error = %e, "Stripe webhook handler error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

// ── InfinitePay webhook handler ───────────────────────────────────────────────

/// POST /api/folio/billing/webhook/infinitepay
///
/// Receives InfinitePay webhook events and reconciles the ledger.
/// Authenticates via `X-InfinitePay-Signature: sha256=<hex>` HMAC.
/// Returns `200 OK` for all handled/unhandled events; `400` on bad signature.
async fn infinitepay_webhook(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let signature = headers
        .get("x-infinitepay-signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let webhook_secret = std::env::var("INFINITEPAY_WEBHOOK_SECRET").unwrap_or_default();

    let raw_body = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match crate::services::pm::rails::infinitepay::InfinitePayWebhookHandler::handle(
        &db,
        raw_body,
        signature,
        &webhook_secret,
    )
    .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            if e.to_string().contains("signature") {
                tracing::warn!(error = %e, "InfinitePay webhook signature verification failed");
                StatusCode::BAD_REQUEST.into_response()
            } else {
                tracing::error!(error = %e, "InfinitePay webhook handler error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

// ── Kelviq webhook handler ────────────────────────────────────────────────────

/// POST /api/folio/billing/webhook/kelviq
///
/// Receives Kelviq webhook events and reconciles the ledger.
/// Authenticates via `X-Kelviq-Secret` shared-secret header (constant-time compare).
/// Returns `200 OK` for all handled/unhandled events; `400` on bad secret.
async fn kelviq_webhook(
    Extension(db): Extension<DatabaseConnection>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let secret_header = headers
        .get("x-kelviq-secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let webhook_secret = std::env::var("KELVIQ_WEBHOOK_SECRET").unwrap_or_default();

    let raw_body = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    match crate::services::pm::rails::kelviq::KelviqWebhookHandler::handle(
        &db,
        raw_body,
        secret_header,
        &webhook_secret,
    )
    .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            if e.to_string().contains("signature") || e.to_string().contains("verification") {
                tracing::warn!(error = %e, "Kelviq webhook verification failed");
                StatusCode::BAD_REQUEST.into_response()
            } else {
                tracing::error!(error = %e, "Kelviq webhook handler error");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

// ── Ledger — list all entries ─────────────────────────────────────────────────

/// Full ledger entry surfaced to the Ledger page.
///
/// `description` carries the human-readable charge label. Currently mapped
/// from `reconciliation_note` using the `[charge_type] label` prefix pattern.
/// Phase 4 will promote `description` and `charge_type` to first-class columns.
#[derive(Debug, Serialize)]
struct LedgerEntrySummary {
    pub id: Uuid,
    pub billable_entity_type: String,
    pub billable_entity_id: Uuid,
    pub description: Option<String>,
    pub gross_amount_cents: i64,
    pub fee_amount_cents: i64,
    pub net_amount_cents: i64,
    pub currency: String,
    pub payment_rail: Option<String>,
    pub status: String,
    pub due_date: Option<chrono::NaiveDate>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reconciled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub reconciliation_note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/folio/ledger
///
/// Returns all `atlas_ledger_entries` for this tenant ordered newest-first.
/// Covers rent, late fees, reimbursements, booking charges, violation fines —
/// every billable event in the system.
async fn list_ledger_entries(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    use sea_orm::QueryOrder;

    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    let entries = crate::entities::atlas_ledger_entry::Entity::find()
        .filter(crate::entities::atlas_ledger_entry::Column::TenantId.eq(tenant_id))
        .order_by_desc(crate::entities::atlas_ledger_entry::Column::CreatedAt)
        .all(&db)
        .await
        .map_err(|e| {
            tracing::error!(%tenant_id, "list_ledger_entries error: {e:#}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let summaries: Vec<LedgerEntrySummary> = entries
        .into_iter()
        .map(|e| LedgerEntrySummary {
            id: e.id,
            billable_entity_type: e.billable_entity_type,
            billable_entity_id: e.billable_entity_id,
            // description is surfaced from reconciliation_note pending Phase 4 migration
            description: e.reconciliation_note.clone(),
            gross_amount_cents: e.gross_amount_cents,
            fee_amount_cents: e.fee_amount_cents,
            net_amount_cents: e.net_amount_cents,
            currency: e.currency,
            payment_rail: e.payment_rail,
            status: e.status,
            due_date: e.due_date,
            paid_at: e.paid_at,
            reconciled_at: e.reconciled_at,
            reconciliation_note: e.reconciliation_note,
            created_at: e.created_at,
        })
        .collect();

    Ok(axum::response::Json(summaries))
}

// ── Ledger — ad-hoc charge ────────────────────────────────────────────────────

/// POST /api/folio/ledger/charge body.
///
/// Charge types that the landlord can apply manually:
///   - `late_fee`                   → e.g. "5% of $2,400 — 7 days past due"
///   - `maintenance_reimbursement`  → e.g. "Broken window repair — Unit 4B"
///   - `incidental`                 → e.g. "Lost key replacement"
///   - `security_deposit_deduction` → move-out deduction
///   - `utility_chargeback`         → water/electric over-usage rebill
///   - `other`                      → catch-all
///
/// The `charge_type` is stored as a bracketed prefix in `reconciliation_note`:
///   `"[late_fee] 5% of $2,400 — payment 7 days past due"`
/// This lets the Ledger page render the correct badge without a schema change.
/// Phase 4 will promote this to dedicated `charge_type` + `description` columns.
#[derive(Debug, Deserialize)]
struct CreateAdHocChargeInput {
    pub billable_entity_type: String,
    pub billable_entity_id: Uuid,
    /// Human-readable line item, e.g. "Payment received 7 days past due".
    pub description: String,
    /// "late_fee" | "maintenance_reimbursement" | "incidental"
    /// | "security_deposit_deduction" | "utility_chargeback" | "other"
    pub charge_type: String,
    pub gross_amount_cents: i64,
    pub currency: String,
    pub due_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize)]
struct CreateAdHocChargeResponse {
    pub ledger_entry_id: Uuid,
    pub status: &'static str,
}

/// POST /api/folio/ledger/charge
///
/// Landlord-initiated ad-hoc charge — late fees, reimbursements, incidentals.
///
/// Creates a `pending` ledger entry with NO payment rail. The charge surfaces
/// as an outstanding balance on the tenant's statement. The tenant resolves it
/// via the existing `/billing/invoice/fiat` or `/billing/invoice/btc` flow,
/// passing `ledger_entry_id` in the request body.
async fn create_ad_hoc_charge(
    Extension(db): Extension<DatabaseConnection>,
    Extension(current_user): Extension<user::Model>,
    Json(input): Json<CreateAdHocChargeInput>,
) -> Result<impl IntoResponse, StatusCode> {
    let tenant_id = resolve_tenant_id(&db, current_user.id).await?;

    const VALID_TYPES: &[&str] = &[
        "late_fee",
        "maintenance_reimbursement",
        "incidental",
        "security_deposit_deduction",
        "utility_chargeback",
        "other",
    ];
    if !VALID_TYPES.contains(&input.charge_type.as_str()) {
        tracing::warn!(
            %tenant_id,
            charge_type = %input.charge_type,
            "create_ad_hoc_charge: invalid charge_type"
        );
        return Err(StatusCode::UNPROCESSABLE_ENTITY);
    }

    let currency = crate::types::pm::Currency::try_from(input.currency.clone()).map_err(|_| {
        tracing::warn!(%tenant_id, currency = %input.currency, "create_ad_hoc_charge: invalid currency");
        StatusCode::UNPROCESSABLE_ENTITY
    })?;

    // Store charge_type + description as tagged reconciliation_note.
    let tagged_note = format!("[{}] {}", input.charge_type, input.description);

    let id = PmLedgerService::create_pending(
        &db,
        tenant_id,
        &input.billable_entity_type,
        input.billable_entity_id,
        None, // payer_user_id — unknown until tenant resolves
        input.gross_amount_cents,
        currency,
        "", // no payment rail yet
        input.due_date,
    )
    .await
    .map_err(|e| {
        tracing::error!(%tenant_id, "create_ad_hoc_charge: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Back-patch the tagged note.
    use sea_orm::ActiveModelTrait;
    use sea_orm::ActiveValue::Set as SeaSet;
    let am = crate::entities::atlas_ledger_entry::ActiveModel {
        id: SeaSet(id),
        reconciliation_note: SeaSet(Some(tagged_note)),
        ..Default::default()
    };
    am.update(&db).await.map_err(|e| {
        tracing::error!(%tenant_id, %id, "create_ad_hoc_charge: failed to patch note: {e:#}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((
        StatusCode::CREATED,
        axum::response::Json(CreateAdHocChargeResponse {
            ledger_entry_id: id,
            status: "pending",
        }),
    ))
}
