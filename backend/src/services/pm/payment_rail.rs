//! Folio — PM Payment Rail Adapter trait (Phase 3)
//!
//! # Distinction from `traits::payment::PaymentProvider`
//!
//! `PaymentProvider` handles platform SaaS billing (Stripe subscriptions, Atlas
//! plan payments). `PaymentRailAdapter` handles **rent invoicing** — tenant-to-
//! landlord money movement inside the Folio PM app.
//!
//! # Rail dispatch
//!
//! At runtime, the billing handler resolves the tenant's active credential for
//! the requested rail from `atlas_payment_credentials`, then calls
//! `PaymentRailRegistry::resolve(credential_type)` to get the matching adapter.
//!
//! # Ledger integration
//!
//! All adapters write to `atlas_ledger_entries` (G-03) via `LedgerService`.
//! The ledger is the source of truth for payment status — the adapter is
//! stateless beyond the credential it is constructed with.
//!
//! # Invoice flow
//!
//! ```text
//! 1. POST /api/folio/billing/invoice/fiat|btc
//!         → handler calls PaymentRailRegistry::resolve(rail)
//!         → adapter.create_invoice() → returns InvoiceResult
//!         → LedgerService::record_pending() writes atlas_ledger_entries row
//!         → handler returns 201 { invoice_id, payment_instructions }
//!
//! 2. async (webhook / mempool poll / manual verify):
//!         → adapter signals payment received (webhook, tx confirmed, receipt)
//!         → LedgerService::mark_paid() updates atlas_ledger_entries.status
//!
//! 3. GET /api/folio/billing/invoice/btc/audit
//!         → handler reads atlas_ledger_entries for tenant BTC invoices
//! ```

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Invoice result ────────────────────────────────────────────────────────────

/// Returned by `PaymentRailAdapter::create_invoice()`.
///
/// `payment_instructions` is rail-specific:
///   - Stripe: `{ "client_secret": "pi_xxx_secret_yyy" }`
///   - PIX/InfinitePay: `{ "qr_code": "...", "expiry_seconds": 3600 }`
///   - BTC on-chain: `{ "address": "bc1q...", "amount_btc": "0.00041" }`
///   - Lightning: `{ "bolt11": "lnbc...", "expiry_seconds": 3600 }`
///   - Kelviq: `{ "payment_url": "https://pay.kelviq.com/..." }`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResult {
    /// Provider-assigned invoice or payment-intent ID.
    pub provider_invoice_id: String,
    /// Rail-specific instructions the UI renders to the payer.
    pub payment_instructions: serde_json::Value,
    /// Estimated expiry in seconds. None = no expiry (on-chain address reuse).
    pub expires_in_seconds: Option<u64>,
}

/// Status of a BTC on-chain payment as polled from mempool.space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStatus {
    pub txid: String,
    pub confirmed: bool,
    pub confirmations: u32,
    pub fee_sats: Option<u64>,
}

// ── Adapter trait ─────────────────────────────────────────────────────────────

/// PM-specific payment rail adapter.
///
/// Each adapter is constructed from the decrypted credentials stored in
/// `atlas_payment_credentials.credentials_encrypted` for the tenant's
/// configured rail. The adapter is stateless — all state lives in the ledger.
#[async_trait]
pub trait PaymentRailAdapter: Send + Sync {
    /// The `credential_type` value this adapter handles.
    /// Must match a value stored in `atlas_payment_credentials.credential_type`.
    fn credential_type(&self) -> &'static str;

    /// Create a payment invoice / intent for a rent amount.
    ///
    /// `ledger_entry_id` is the pre-created `atlas_ledger_entries` row so the
    /// adapter can embed it in webhook metadata for idempotent reconciliation.
    async fn create_invoice(
        &self,
        ledger_entry_id: Uuid,
        tenant_id: Uuid,
        amount_cents: i64,
        currency: &str,
        description: &str,
    ) -> Result<InvoiceResult>;

    /// Poll / verify that a payment has been received.
    ///
    /// For BTC: calls mempool.space to check tx confirmation.
    /// For Stripe: this is handled by webhook — returns `Ok(false)` if not confirmed.
    /// For Lightning: checks bolt11 invoice status via the node API.
    async fn is_payment_confirmed(
        &self,
        provider_invoice_id: &str,
    ) -> Result<bool>;
}

// ── Credential encryption placeholder ─────────────────────────────────────────

/// Decrypts `atlas_payment_credentials.credentials_encrypted` for a tenant + rail.
///
/// Phase 3 implementation: AES-256-GCM with per-tenant key from `ATLAS_CREDENTIAL_KEY`
/// env var (or HSM in production). For now returns the JSONB as-is since the test
/// environment does not have real secrets in CI.
///
/// # Security note
/// Real production deployments must rotate `ATLAS_CREDENTIAL_KEY` via the ops
/// runbook. The encrypt/decrypt functions live here so the blast radius of a
/// key change is limited to this module.
pub fn decrypt_credentials(
    encrypted: &serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    // Phase 3 stub: return plaintext for dev/CI.
    // Production: AES-256-GCM decrypt from ATLAS_CREDENTIAL_KEY env var.
    Ok(encrypted.clone())
}

// ── Registry ──────────────────────────────────────────────────────────────────

/// Resolves a `PaymentRailAdapter` for a given `credential_type` string.
///
/// Called by the billing handler after loading the tenant's active credential row.
/// Returns an error if no adapter is registered for the requested type.
pub fn resolve_adapter(
    credential_type: &str,
    credentials: &serde_json::Value,
) -> anyhow::Result<Box<dyn PaymentRailAdapter>> {
    match credential_type {
        "stripe_connect_express" | "stripe_connect_standard" => {
            let secret_key = credentials
                .get("secret_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("stripe credential missing 'secret_key'"))?;
            let account_id = credentials
                .get("account_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("stripe credential missing 'account_id'"))?;
            Ok(Box::new(
                crate::services::pm::rails::stripe_connect::StripeConnectRail::new(
                    secret_key.to_string(),
                    account_id.to_string(),
                ),
            ))
        }
        "pix_key" => {
            let api_key = credentials
                .get("api_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("infinitepay credential missing 'api_key'"))?;
            Ok(Box::new(
                crate::services::pm::rails::infinitepay::InfinitePayRail::new(
                    api_key.to_string(),
                ),
            ))
        }
        "btc_onchain_address" => {
            let address = credentials
                .get("address")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("btc_onchain credential missing 'address'"))?;
            Ok(Box::new(
                crate::services::pm::rails::bitcoin_onchain::BitcoinOnchainRail::new(
                    address.to_string(),
                ),
            ))
        }
        "btc_lightning_node" => {
            let base_url = credentials
                .get("base_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("lightning credential missing 'base_url'"))?;
            let api_key = credentials
                .get("api_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("lightning credential missing 'api_key'"))?;
            Ok(Box::new(
                crate::services::pm::rails::lightning::LightningRail::new(
                    base_url.to_string(),
                    api_key.to_string(),
                ),
            ))
        }
        "kelviq" => {
            let api_key = credentials
                .get("api_key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("kelviq credential missing 'api_key'"))?;
            let merchant_id = credentials
                .get("merchant_id")
                .and_then(|v| v.as_str())
                .unwrap_or("") // optional — some Kelviq accounts omit it
                .to_string();
            Ok(Box::new(
                crate::services::pm::rails::kelviq::KelviqRail::new(
                    api_key.to_string(),
                    merchant_id,
                ),
            ))
        }
        other => Err(anyhow::anyhow!(
            "no PaymentRailAdapter registered for credential_type '{other}'"
        )),
    }
}
