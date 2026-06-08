//! # Stripe Connect PM Rail Adapter
//!
//! Handles rent collection via Stripe Connect Express/Standard accounts.
//!
//! ## Flow
//!
//! 1. `create_invoice()` creates a PaymentIntent against the platform's secret key
//!    with `transfer_data.destination` pointing to the landlord's Connect account.
//!    The `metadata` hash embeds `ledger_entry_id` + `tenant_id` so the webhook
//!    handler can do idempotent reconciliation without a database lookup by ID.
//!
//! 2. Returns `InvoiceResult { client_secret, provider_invoice_id }` for the
//!    frontend to complete payment with Stripe.js (`stripe.confirmCardPayment()`).
//!
//! 3. Stripe fires `payment_intent.succeeded` →
//!    `StripeConnectWebhookHandler::handle()` → `LedgerService::mark_paid()`.
//!
//! ## Required keys in `atlas_payment_credentials.credentials_encrypted`
//!
//! ```json
//! {
//!   "secret_key":  "sk_live_...",
//!   "account_id":  "acct_..."    // landlord's Stripe Connect account ID
//! }
//! ```
//!
//! ## Application fee
//!
//! Set `STRIPE_PLATFORM_FEE_BPS` env var (basis points, e.g. `200` = 2%).
//! If unset, defaults to 0 (no platform fee taken).

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::json;
use stripe::{
    Client, CreatePaymentIntent, CreatePaymentIntentTransferData, Currency, PaymentIntent,
};
use uuid::Uuid;

use crate::services::pm::payment_rail::{InvoiceResult, PaymentRailAdapter};

// ── Rail adapter ──────────────────────────────────────────────────────────────

pub struct StripeConnectRail {
    /// Platform-level secret key (not the Connect account key).
    secret_key: String,
    /// Landlord's Stripe Connect account ID (`acct_...`).
    connect_account_id: String,
}

impl StripeConnectRail {
    pub fn new(secret_key: String, connect_account_id: String) -> Self {
        Self {
            secret_key,
            connect_account_id,
        }
    }

    /// Read the optional platform fee in basis points from the environment.
    /// 200 BPS = 2%. Defaults to 0 if not set or invalid.
    fn platform_fee_bps() -> u64 {
        std::env::var("STRIPE_PLATFORM_FEE_BPS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0)
    }

    /// Compute the application fee amount in cents.
    fn application_fee_cents(amount_cents: i64) -> Option<i64> {
        let bps = Self::platform_fee_bps();
        if bps == 0 {
            None
        } else {
            Some((amount_cents * bps as i64) / 10_000)
        }
    }
}

#[async_trait]
impl PaymentRailAdapter for StripeConnectRail {
    fn credential_type(&self) -> &'static str {
        "stripe_connect_express"
    }

    /// Create a PaymentIntent with transfer_data.destination set to the
    /// landlord's Connect account. Embeds `ledger_entry_id` in metadata
    /// for idempotent webhook reconciliation.
    async fn create_invoice(
        &self,
        ledger_entry_id: Uuid,
        tenant_id: Uuid,
        amount_cents: i64,
        currency: &str,
        description: &str,
    ) -> Result<InvoiceResult> {
        let client = Client::new(&self.secret_key);

        let stripe_currency = currency
            .to_lowercase()
            .parse::<Currency>()
            .map_err(|_| anyhow!("unsupported currency: {currency}"))?;

        let mut params = CreatePaymentIntent::new(amount_cents, stripe_currency);
        params.description = Some(description);
        params.transfer_data = Some(CreatePaymentIntentTransferData {
            destination: self.connect_account_id.clone(),
            amount: None, // use full amount; fee deducted via application_fee_amount
        });

        if let Some(fee_cents) = Self::application_fee_cents(amount_cents) {
            params.application_fee_amount = Some(fee_cents);
        }

        // Embed ledger + tenant IDs in metadata for webhook reconciliation.
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("ledger_entry_id".to_string(), ledger_entry_id.to_string());
        metadata.insert("tenant_id".to_string(), tenant_id.to_string());
        params.metadata = Some(metadata);

        let intent = PaymentIntent::create(&client, params)
            .await
            .context("Stripe PaymentIntent::create failed")?;

        let client_secret = intent.client_secret.clone().ok_or_else(|| {
            anyhow!("Stripe did not return a client_secret for PaymentIntent {}", intent.id)
        })?;

        tracing::info!(
            %ledger_entry_id,
            %tenant_id,
            amount_cents,
            currency,
            payment_intent_id = %intent.id,
            connect_account = %self.connect_account_id,
            "Stripe Connect PaymentIntent created"
        );

        Ok(InvoiceResult {
            provider_invoice_id: intent.id.to_string(),
            payment_instructions: json!({
                "client_secret": client_secret,
                "payment_intent_id": intent.id.to_string(),
                "rail": "stripe_connect",
                "publishable_key_hint": "Use your platform publishable key with Stripe.js",
            }),
            expires_in_seconds: None,
        })
    }

    /// Stripe confirmations arrive via webhook, not polling.
    /// Returns `Ok(false)` — reconciliation is event-driven.
    async fn is_payment_confirmed(&self, _provider_invoice_id: &str) -> Result<bool> {
        Ok(false)
    }
}

// ── Webhook handler ───────────────────────────────────────────────────────────

/// Processes incoming Stripe webhook events and reconciles the ledger.
///
/// ## Handled events
///
/// | Event | Action |
/// |---|---|
/// | `payment_intent.succeeded` | `LedgerService::mark_paid()` |
/// | `payment_intent.payment_failed` | `LedgerService::mark_failed()` |
/// | `charge.refunded` | `LedgerService::mark_refunded()` |
///
/// ## Idempotency
///
/// All three handlers are idempotent — calling them twice on the same
/// `ledger_entry_id` is safe because the service checks current status before writing.
pub struct StripeConnectWebhookHandler;

impl StripeConnectWebhookHandler {
    /// Verify the Stripe webhook signature and dispatch to the appropriate handler.
    ///
    /// `webhook_secret` must be the endpoint secret from the Stripe dashboard
    /// (or `STRIPE_WEBHOOK_SECRET` env var). Returns `Ok(())` for unhandled events.
    pub async fn handle(
        db: &sea_orm::DatabaseConnection,
        raw_body: &str,
        stripe_signature: &str,
        webhook_secret: &str,
    ) -> Result<()> {
        let event = stripe::Webhook::construct_event(raw_body, stripe_signature, webhook_secret)
            .map_err(|e| anyhow!("Stripe webhook signature verification failed: {e:?}"))?;

        tracing::info!(event_type = ?event.type_, event_id = %event.id, "Stripe webhook received");

        match event.type_ {
            stripe::EventType::PaymentIntentSucceeded => {
                if let stripe::EventObject::PaymentIntent(intent) = event.data.object {
                    Self::on_payment_succeeded(db, intent).await?;
                }
            }
            stripe::EventType::PaymentIntentPaymentFailed => {
                if let stripe::EventObject::PaymentIntent(intent) = event.data.object {
                    Self::on_payment_failed(db, intent).await?;
                }
            }
            stripe::EventType::ChargeRefunded => {
                if let stripe::EventObject::Charge(charge) = event.data.object {
                    Self::on_charge_refunded(db, charge).await?;
                }
            }
            _ => {
                tracing::debug!(event_type = ?event.type_, "Stripe webhook: unhandled event type");
            }
        }

        Ok(())
    }

    // ── Event handlers ────────────────────────────────────────────────────────

    async fn on_payment_succeeded(
        db: &sea_orm::DatabaseConnection,
        intent: stripe::PaymentIntent,
    ) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(&intent.metadata)?;
        let provider_invoice_id = intent.id.to_string();

        tracing::info!(
            %ledger_entry_id,
            %provider_invoice_id,
            "Stripe payment_intent.succeeded → marking ledger entry paid"
        );

        crate::services::pm::ledger::PmLedgerService::mark_paid(
            db,
            ledger_entry_id,
            Some(provider_invoice_id),
        )
        .await
        .context("LedgerService::mark_paid failed after payment_intent.succeeded")?;

        Ok(())
    }

    async fn on_payment_failed(
        db: &sea_orm::DatabaseConnection,
        intent: stripe::PaymentIntent,
    ) -> Result<()> {
        let ledger_entry_id = Self::extract_ledger_id(&intent.metadata)?;

        tracing::warn!(
            %ledger_entry_id,
            failure_message = intent.last_payment_error.as_ref()
                .and_then(|e| e.message.as_deref())
                .unwrap_or("unknown"),
            "Stripe payment_intent.payment_failed → marking ledger entry failed"
        );

        crate::services::pm::ledger::PmLedgerService::mark_failed(db, ledger_entry_id)
            .await
            .context("LedgerService::mark_failed failed after payment_intent.payment_failed")?;

        Ok(())
    }

    async fn on_charge_refunded(
        db: &sea_orm::DatabaseConnection,
        charge: stripe::Charge,
    ) -> Result<()> {
        // The charge metadata mirrors the PaymentIntent metadata (Stripe copies it).
        let ledger_entry_id = Self::extract_ledger_id(&charge.metadata)?;

        tracing::info!(
            %ledger_entry_id,
            "Stripe charge.refunded → marking ledger entry refunded"
        );

        crate::services::pm::ledger::PmLedgerService::mark_refunded(
            db,
            ledger_entry_id,
            Some("stripe_charge_refunded".into()),
        )
        .await
        .context("LedgerService::mark_refunded failed after charge.refunded")?;

        Ok(())
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn extract_ledger_id(
        metadata: &std::collections::HashMap<String, String>,
    ) -> Result<uuid::Uuid> {
        let raw = metadata.get("ledger_entry_id").ok_or_else(|| {
            anyhow!("Stripe webhook metadata missing 'ledger_entry_id' — cannot reconcile")
        })?;
        raw.parse::<uuid::Uuid>()
            .context("'ledger_entry_id' in Stripe metadata is not a valid UUID")
    }
}
