//! Kelviq PM rail adapter — Caribbean / USVI market.
//!
//! Kelviq is a payment processor operating in the US Virgin Islands and broader
//! Caribbean markets where traditional US ACH rails have poor coverage.
//! It handles USD payments via local Caribbean payment networks.
//!
//! # Flow
//! 1. `create_invoice()` → POST to Kelviq API → returns a hosted payment URL.
//! 2. Returns `{ "payment_url": "https://pay.kelviq.com/..." }`.
//! 3. Kelviq fires a webhook on `charge.completed` → handler marks ledger paid.
//!
//! # Keys required in `credentials_encrypted`
//! ```json
//! {
//!   "api_key":     "kelviq_live_...",
//!   "merchant_id": "mer_..."
//! }
//! ```
//!
//! # Jurisdiction
//! Active when `folio_jurisdiction_code IN ('USVI', 'PR', 'VI', 'DO', 'HT')`.
//! The billing handler checks this setting before constructing the adapter.
//!
//! # Webhook verification
//! Kelviq signs webhooks with a shared secret in the `X-Kelviq-Secret` header.
//! Constant-time comparison is used to prevent timing attacks.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use serde_json::json;
use uuid::Uuid;

use crate::services::pm::payment_rail::{InvoiceResult, PaymentRailAdapter};

const KELVIQ_API_BASE: &str = "https://api.kelviq.com/v1";

/// Base URL for the Folio app's Kelviq webhook endpoint.
/// In production this should be set via `ATLAS_PUBLIC_URL` environment variable.
fn webhook_base_url() -> String {
    std::env::var("ATLAS_PUBLIC_URL")
        .unwrap_or_else(|_| "https://atlas.app".to_string())
}

pub struct KelviqRail {
    api_key: String,
    merchant_id: String,
    client: reqwest::Client,
}

impl KelviqRail {
    pub fn new(api_key: String, merchant_id: String) -> Self {
        Self {
            api_key,
            merchant_id,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PaymentRailAdapter for KelviqRail {
    fn credential_type(&self) -> &'static str {
        "kelviq"
    }

    async fn create_invoice(
        &self,
        ledger_entry_id: Uuid,
        tenant_id: Uuid,
        amount_cents: i64,
        currency: &str,
        description: &str,
    ) -> Result<InvoiceResult> {
        let body = json!({
            "amount_cents": amount_cents,
            "currency": if currency.is_empty() { "USD" } else { currency },
            "description": description,
            "merchant_id": self.merchant_id,
            "external_reference": ledger_entry_id.to_string(),
            "success_url": format!("{}/folio/billing/success", webhook_base_url()),
            "webhook_url": format!("{}/api/folio/billing/webhook/kelviq", webhook_base_url()),
            "metadata": {
                "ledger_entry_id": ledger_entry_id.to_string(),
                "tenant_id": tenant_id.to_string(),
            }
        });

        let resp = self
            .client
            .post(format!("{KELVIQ_API_BASE}/charges"))
            .header("X-Api-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .context("KelviqRail: HTTP request to /charges failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("KelviqRail: /charges returned {status}: {text}"));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .context("KelviqRail: failed to parse /charges response")?;

        let provider_invoice_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("KelviqRail: response missing 'id'"))?
            .to_string();

        let payment_url = data
            .get("payment_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        tracing::info!(
            %ledger_entry_id, %tenant_id,
            amount_cents, currency,
            %provider_invoice_id,
            "KelviqRail: charge created"
        );

        Ok(InvoiceResult {
            provider_invoice_id,
            payment_instructions: json!({
                "rail": "kelviq",
                "payment_url": payment_url,
                "note": "Tenant is redirected to Kelviq's hosted payment page",
            }),
            expires_in_seconds: Some(86400), // 24 hours
        })
    }

    async fn is_payment_confirmed(&self, provider_invoice_id: &str) -> Result<bool> {
        let url = format!("{KELVIQ_API_BASE}/charges/{provider_invoice_id}");
        let resp = self
            .client
            .get(&url)
            .header("X-Api-Key", &self.api_key)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = r.json().await.unwrap_or_default();
                let status = body.get("status").and_then(|s| s.as_str()).unwrap_or("");
                Ok(status == "completed" || status == "paid")
            }
            _ => Ok(false),
        }
    }
}

// ── Webhook handler ───────────────────────────────────────────────────────────

pub struct KelviqWebhookHandler;

impl KelviqWebhookHandler {
    /// Process an incoming Kelviq webhook event.
    ///
    /// Verifies `X-Kelviq-Secret` header with constant-time comparison.
    /// Handles:
    /// - `charge.completed` → `LedgerService::mark_paid()`
    /// - `charge.failed`    → `LedgerService::mark_failed()`
    /// - `charge.refunded`  → `LedgerService::mark_refunded()`
    pub async fn handle(
        db: &DatabaseConnection,
        raw_body: &str,
        secret_header: &str,
        webhook_secret: &str,
    ) -> Result<()> {
        // ── Verify shared secret ──────────────────────────────────────────────
        if !webhook_secret.is_empty() {
            // Constant-time comparison to prevent timing attacks.
            if !Self::constant_time_eq(secret_header.as_bytes(), webhook_secret.as_bytes()) {
                return Err(anyhow!("KelviqWebhook: signature verification failed"));
            }
        } else {
            tracing::warn!("KelviqWebhook: KELVIQ_WEBHOOK_SECRET not set — skipping verification");
        }

        let event: serde_json::Value = serde_json::from_str(raw_body)
            .context("KelviqWebhook: failed to parse event body")?;

        let event_type = event
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!(event_type, "KelviqWebhook: received event");

        match event_type {
            "charge.completed" | "charge.paid" => Self::on_charge_completed(db, &event).await?,
            "charge.failed" => Self::on_charge_failed(db, &event).await?,
            "charge.refunded" => Self::on_charge_refunded(db, &event).await?,
            other => {
                tracing::debug!(event_type = other, "KelviqWebhook: unhandled event type");
            }
        }

        Ok(())
    }

    /// Constant-time byte slice comparison (branchless).
    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        a.iter().zip(b.iter()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
    }

    fn extract_ledger_id(event: &serde_json::Value) -> Result<Uuid> {
        let raw = event
            .pointer("/data/external_reference")
            .or_else(|| event.pointer("/data/metadata/ledger_entry_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("KelviqWebhook: missing ledger_entry_id in event"))?;

        raw.parse::<Uuid>()
            .context("KelviqWebhook: ledger_entry_id is not a valid UUID")
    }

    fn extract_charge_id(event: &serde_json::Value) -> Option<String> {
        event
            .pointer("/data/id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    async fn on_charge_completed(db: &DatabaseConnection, event: &serde_json::Value) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(event)?;
        let charge_id = Self::extract_charge_id(event);

        crate::services::pm::ledger::PmLedgerService::mark_paid(
            db,
            ledger_entry_id,
            charge_id,
        )
        .await
        .context("KelviqWebhook: mark_paid failed")?;

        Ok(())
    }

    async fn on_charge_failed(db: &DatabaseConnection, event: &serde_json::Value) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(event)?;

        crate::services::pm::ledger::PmLedgerService::mark_failed(db, ledger_entry_id)
            .await
            .context("KelviqWebhook: mark_failed failed")?;

        Ok(())
    }

    async fn on_charge_refunded(db: &DatabaseConnection, event: &serde_json::Value) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(event)?;

        crate::services::pm::ledger::PmLedgerService::mark_refunded(
            db,
            ledger_entry_id,
            Some("kelviq_charge_refunded".into()),
        )
        .await
        .context("KelviqWebhook: mark_refunded failed")?;

        Ok(())
    }
}
