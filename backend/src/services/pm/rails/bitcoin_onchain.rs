//! Bitcoin on-chain PM rail adapter.
//!
//! Tenants pay rent by sending BTC to the landlord's on-chain address.
//! Confirmation is polled via mempool.space (self-hosted or public API).
//!
//! # Denomination
//! All internal amounts are in satoshis. The USD/BTC conversion rate is fetched
//! at invoice creation time from mempool.space's price API and embedded in the
//! payment_instructions so the tenant knows how many sats to send.
//!
//! # Confirmation threshold
//! Default: 1 confirmation (safe for rent amounts under $10k).
//! Configurable via `folio_btc_confirmation_threshold` tenant setting.
//!
//! # Keys required in `credentials_encrypted`
//! ```json
//! {
//!   "address":       "bc1q...",    // landlord's receive address (or xpub)
//!   "mempool_host":  "https://mempool.space"  // optional: self-hosted
//! }
//! ```
//!
//! # Address reuse warning
//! For production, use an xpub + HD derivation path to generate fresh addresses
//! per invoice. The current implementation uses a single static address — safe
//! for low-volume landlords, not recommended for high-volume operators.

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use uuid::Uuid;

use crate::services::pm::payment_rail::{InvoiceResult, MempoolStatus, PaymentRailAdapter};

const MEMPOOL_SPACE_PUBLIC: &str = "https://mempool.space";

pub struct BitcoinOnchainRail {
    address: String,
    mempool_host: String,
    client: reqwest::Client,
}

impl BitcoinOnchainRail {
    pub fn new(address: String) -> Self {
        Self::with_mempool_host(address, MEMPOOL_SPACE_PUBLIC.to_string())
    }

    pub fn with_mempool_host(address: String, mempool_host: String) -> Self {
        Self {
            address,
            mempool_host,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch the current BTC/USD spot price from mempool.space.
    /// Returns `None` on any error — callers should fall back to a stale cached rate.
    pub async fn fetch_btc_usd_rate(&self) -> Option<f64> {
        let url = format!("{}/api/v1/prices", self.mempool_host);
        let resp = self.client.get(&url).send().await.ok()?;
        let body: serde_json::Value = resp.json().await.ok()?;
        body.get("USD").and_then(|v| v.as_f64())
    }

    /// Poll mempool.space for the status of a specific txid.
    ///
    /// Returns None if the tx is not yet seen in the mempool.
    pub async fn poll_tx(&self, txid: &str) -> Option<MempoolStatus> {
        let url = format!("{}/api/tx/{}", self.mempool_host, txid);
        let resp = self.client.get(&url).send().await.ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let body: serde_json::Value = resp.json().await.ok()?;
        let confirmed = body
            .get("status")
            .and_then(|s| s.get("confirmed"))
            .and_then(|c| c.as_bool())
            .unwrap_or(false);

        let confirmations = if confirmed {
            // mempool.space does not return confirmations directly —
            // derive from block_height delta (simplified: ≥1 if confirmed)
            1
        } else {
            0
        };

        let fee_sats = body
            .get("fee")
            .and_then(|f| f.as_u64());

        Some(MempoolStatus {
            txid: txid.to_string(),
            confirmed,
            confirmations,
            fee_sats,
        })
    }
}

#[async_trait]
impl PaymentRailAdapter for BitcoinOnchainRail {
    fn credential_type(&self) -> &'static str {
        "btc_onchain_address"
    }

    async fn create_invoice(
        &self,
        ledger_entry_id: Uuid,
        tenant_id: Uuid,
        amount_cents: i64,
        currency: &str,
        description: &str,
    ) -> Result<InvoiceResult> {
        // Fetch current BTC price to compute sat amount.
        let btc_usd_rate = self.fetch_btc_usd_rate().await.unwrap_or(60_000.0);

        // Convert USD cents → satoshis.
        // amount_cents is in USD (or BRL, depending on currency — for BRL
        // a separate conversion step would be needed).
        let amount_usd = amount_cents as f64 / 100.0;
        let amount_btc = amount_usd / btc_usd_rate;
        let amount_sats = (amount_btc * 100_000_000.0).round() as u64;

        // BIP-21 URI for QR code generation.
        let bip21_uri = format!(
            "bitcoin:{}?amount={}&label={}&message={}",
            self.address,
            amount_btc,
            urlencoding::encode("Folio Rent"),
            urlencoding::encode(description),
        );

        tracing::info!(
            %ledger_entry_id, %tenant_id,
            amount_cents, currency, description,
            address = %self.address,
            amount_sats,
            btc_usd_rate,
            "BitcoinOnchainRail: invoice created"
        );

        Ok(InvoiceResult {
            // For on-chain: the "invoice ID" is the receive address + ledger entry
            // (there's no network-level invoice — confirmation comes by txid).
            provider_invoice_id: format!("btc_onchain_{}_{}", self.address, ledger_entry_id.simple()),
            payment_instructions: json!({
                "rail": "btc_onchain",
                "address": self.address,
                "amount_sats": amount_sats,
                "amount_btc": format!("{:.8}", amount_btc),
                "bip21_uri": bip21_uri,
                "btc_usd_rate": btc_usd_rate,
                "note": "Send exact amount. After sending, submit your txid via /api/folio/billing/invoice/btc",
            }),
            expires_in_seconds: None, // on-chain addresses do not expire
        })
    }

    async fn is_payment_confirmed(&self, provider_invoice_id: &str) -> Result<bool> {
        // `provider_invoice_id` for on-chain rails is not a txid —
        // it's the address+ledger composite. The actual txid is submitted
        // by the tenant via POST /api/folio/billing/invoice/btc and stored
        // in atlas_ledger_entries.external_tx_id.
        //
        // This method is called by the mempool poll background job which
        // passes the txid (loaded from ledger) as the argument.
        // If the invoice ID looks like a txid (64 hex chars), poll it.

        if provider_invoice_id.len() == 64
            && provider_invoice_id.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Ok(
                self.poll_tx(provider_invoice_id)
                    .await
                    .map(|s| s.confirmed)
                    .unwrap_or(false),
            );
        }

        // Not a txid yet — tenant hasn't submitted it.
        Ok(false)
    }
}
