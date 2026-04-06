use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionData {
    pub subscription_id: String,
    pub status: String,
    pub current_period_end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    pub transaction_id: String,
    pub amount: i64,
    pub currency: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub provider_tx_id: String,
    pub raw_body: Vec<u8>,
    pub signature: String,
}

#[async_trait]
pub trait PaymentProvider: Send + Sync {
    /// Create a subscription in the provider's system
    async fn create_subscription(
        &self, 
        tenant_id: Uuid, 
        plan_name: &str, 
        price_cents: i64, 
        currency: &str
    ) -> Result<SubscriptionData>;

    /// Capture a one-off payment
    async fn capture_payment(
        &self, 
        tenant_id: Uuid, 
        amount_cents: i64, 
        currency: &str
    ) -> Result<TransactionData>;

    /// Setup the tenant to receive payouts (e.g. Stripe Connect, BTCPay store routing)
    async fn setup_tenant_payout_route(&self, tenant_id: Uuid) -> Result<String>;

    /// Verify and process a webhook from the provider
    async fn process_webhook(&self, payload: &WebhookPayload) -> Result<()>;
}
