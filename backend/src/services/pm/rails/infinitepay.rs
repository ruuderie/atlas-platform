//! InfinitePay PIX rail adapter — Brazil market.
//!
//! InfinitePay is a Brazilian acquirer and PIX facilitator popular with
//! small-to-mid landlords. Tenants pay rent via PIX — funds settle in the
//! landlord's InfinitePay account in seconds via the Banco Central rails.
//!
//! # Flow
//! 1. `create_invoice()` → POST to InfinitePay Charges API → returns a PIX
//!    QR code + copy-paste key.
//! 2. Returns `{ "qr_code": "...", "pix_key": "...", "expiry_seconds": 3600 }`.
//! 3. InfinitePay fires a webhook on `charge.paid` → handler marks ledger paid.
//!
//! # Keys required in `credentials_encrypted`
//! ```json
//! {
//!   "api_key": "ak_live_..."
//! }
//! ```
//!
//! # Jurisdiction
//! Active when `folio_jurisdiction_code = 'BR'`. The billing handler checks this
//! setting before constructing the adapter.
//!
//! # Webhook verification
//! InfinitePay signs webhook payloads with HMAC-SHA256 using the webhook secret.
//! Signature is in the `X-InfinitePay-Signature` header as `sha256=<hex>`.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use sea_orm::DatabaseConnection;
use serde_json::json;
use sha2::Sha256;
use uuid::Uuid;

use crate::services::pm::payment_rail::{InvoiceResult, PaymentRailAdapter};

const INFINITEPAY_API_BASE: &str = "https://api.infinitepay.io/v2";

pub struct InfinitePayRail {
    api_key: String,
    client: reqwest::Client,
}

impl InfinitePayRail {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PaymentRailAdapter for InfinitePayRail {
    fn credential_type(&self) -> &'static str {
        "pix_key"
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
            "amount": amount_cents,
            "currency": if currency.is_empty() { "BRL" } else { currency },
            "payment_method": "pix",
            "description": description,
            "external_id": ledger_entry_id.to_string(),
            "metadata": {
                "ledger_entry_id": ledger_entry_id.to_string(),
                "tenant_id": tenant_id.to_string(),
            }
        });

        let resp = self
            .client
            .post(format!("{INFINITEPAY_API_BASE}/charges"))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("InfinitePayRail: HTTP request to /charges failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(anyhow!(
                "InfinitePayRail: /charges returned {status}: {text}"
            ));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .context("InfinitePayRail: failed to parse /charges response")?;

        let provider_invoice_id = data
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("InfinitePayRail: response missing 'id'"))?
            .to_string();

        let qr_code = data
            .pointer("/pix/qr_code")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let pix_key = data
            .pointer("/pix/key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        tracing::info!(
            %ledger_entry_id, %tenant_id,
            amount_cents, currency,
            %provider_invoice_id,
            "InfinitePayRail: PIX charge created"
        );

        Ok(InvoiceResult {
            provider_invoice_id,
            payment_instructions: json!({
                "rail": "pix",
                "qr_code": qr_code,
                "pix_key": pix_key,
                "expiry_seconds": 3600,
                "amount_brl_centavos": amount_cents,
            }),
            expires_in_seconds: Some(3600),
        })
    }

    async fn is_payment_confirmed(&self, provider_invoice_id: &str) -> Result<bool> {
        let url = format!("{INFINITEPAY_API_BASE}/charges/{provider_invoice_id}");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let body: serde_json::Value = r.json().await.unwrap_or_default();
                let status = body.get("status").and_then(|s| s.as_str()).unwrap_or("");
                Ok(status == "paid")
            }
            Ok(r) => {
                tracing::warn!(
                    invoice_id = %provider_invoice_id,
                    status = %r.status(),
                    "InfinitePayRail: unexpected status polling charge"
                );
                Ok(false)
            }
            Err(e) => {
                tracing::warn!(
                    invoice_id = %provider_invoice_id,
                    "InfinitePayRail: polling error (non-fatal): {e:#}"
                );
                Ok(false)
            }
        }
    }
}

// ── Webhook handler ───────────────────────────────────────────────────────────

pub struct InfinitePayWebhookHandler;

impl InfinitePayWebhookHandler {
    /// Process an incoming InfinitePay webhook event.
    ///
    /// Verifies `X-InfinitePay-Signature: sha256=<hex>` with HMAC-SHA256.
    /// Handles:
    /// - `charge.paid`    → `LedgerService::mark_paid()`
    /// - `charge.failed`  → `LedgerService::mark_failed()`
    /// - `charge.refunded`→ `LedgerService::mark_refunded()`
    pub async fn handle(
        db: &DatabaseConnection,
        raw_body: &str,
        signature_header: &str,
        webhook_secret: &str,
    ) -> Result<()> {
        // ── Verify HMAC-SHA256 signature ──────────────────────────────────────
        if !webhook_secret.is_empty() {
            Self::verify_signature(raw_body, signature_header, webhook_secret)?;
        } else {
            tracing::warn!("InfinitePayWebhook: INFINITEPAY_WEBHOOK_SECRET not set — skipping verification");
        }

        let event: serde_json::Value = serde_json::from_str(raw_body)
            .context("InfinitePayWebhook: failed to parse event body")?;

        let event_type = event
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!(event_type, "InfinitePayWebhook: received event");

        match event_type {
            "charge.paid" => Self::on_charge_paid(db, &event).await?,
            "charge.failed" => Self::on_charge_failed(db, &event).await?,
            "charge.refunded" => Self::on_charge_refunded(db, &event).await?,
            other => {
                tracing::debug!(event_type = other, "InfinitePayWebhook: unhandled event type");
            }
        }

        Ok(())
    }

    fn verify_signature(
        payload: &str,
        signature_header: &str,
        secret: &str,
    ) -> Result<()> {
        // Header format: "sha256=<hex>"
        let hex_sig = signature_header
            .strip_prefix("sha256=")
            .ok_or_else(|| anyhow!("InfinitePayWebhook: malformed signature header"))?;

        let expected = hex::decode(hex_sig)
            .context("InfinitePayWebhook: signature is not valid hex")?;

        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .context("InfinitePayWebhook: invalid HMAC key")?;
        mac.update(payload.as_bytes());

        mac.verify_slice(&expected)
            .map_err(|_| anyhow!("InfinitePayWebhook: signature verification failed"))?;

        Ok(())
    }

    fn extract_ledger_id(event: &serde_json::Value) -> Result<Uuid> {
        // InfinitePay stores `external_id` at the top level and also in metadata.
        let raw = event
            .pointer("/data/object/external_id")
            .or_else(|| event.pointer("/data/object/metadata/ledger_entry_id"))
            .or_else(|| event.get("external_id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("InfinitePayWebhook: missing ledger_entry_id in event"))?;

        raw.parse::<Uuid>()
            .context("InfinitePayWebhook: ledger_entry_id is not a valid UUID")
    }

    fn extract_charge_id(event: &serde_json::Value) -> Option<String> {
        event
            .pointer("/data/object/id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    async fn on_charge_paid(db: &DatabaseConnection, event: &serde_json::Value) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(event)?;
        let charge_id = Self::extract_charge_id(event);

        crate::services::pm::ledger::PmLedgerService::mark_paid(
            db,
            ledger_entry_id,
            charge_id,
        )
        .await
        .context("InfinitePayWebhook: mark_paid failed")?;

        Ok(())
    }

    async fn on_charge_failed(db: &DatabaseConnection, event: &serde_json::Value) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(event)?;

        crate::services::pm::ledger::PmLedgerService::mark_failed(db, ledger_entry_id)
            .await
            .context("InfinitePayWebhook: mark_failed failed")?;

        Ok(())
    }

    async fn on_charge_refunded(db: &DatabaseConnection, event: &serde_json::Value) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(event)?;

        crate::services::pm::ledger::PmLedgerService::mark_refunded(
            db,
            ledger_entry_id,
            Some("infinitepay_charge_refunded".into()),
        )
        .await
        .context("InfinitePayWebhook: mark_refunded failed")?;

        Ok(())
    }
}
