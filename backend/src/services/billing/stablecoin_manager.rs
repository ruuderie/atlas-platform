use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::traits::payment::{PaymentProvider, SubscriptionData, TransactionData, WebhookPayload};

/// StablecoinManager orchestrates the routing of USD-pegged stablecoins (USDT/USDC).
/// In early versions, this just delegates back to Stripe Crypto / Paddle. 
/// In future, it can route natively to a smart-contract listener.
pub struct StablecoinManager {
    underlying_provider: Box<dyn PaymentProvider>,
}

impl StablecoinManager {
    pub fn new(underlying_provider: Box<dyn PaymentProvider>) -> Self {
        Self { underlying_provider }
    }
}

#[async_trait]
impl PaymentProvider for StablecoinManager {
    async fn create_subscription(&self, tenant_id: Uuid, plan_name: &str, price_cents: i64, currency: &str) -> Result<SubscriptionData> {
        tracing::info!("Delegating stablecoin subscription processing...");
        self.underlying_provider.create_subscription(tenant_id, plan_name, price_cents, currency).await
    }

    async fn capture_payment(&self, tenant_id: Uuid, amount_cents: i64, currency: &str) -> Result<TransactionData> {
        self.underlying_provider.capture_payment(tenant_id, amount_cents, currency).await
    }

    async fn setup_tenant_payout_route(&self, tenant_id: Uuid) -> Result<String> {
        self.underlying_provider.setup_tenant_payout_route(tenant_id).await
    }

    async fn process_webhook(&self, payload: &WebhookPayload) -> Result<()> {
        self.underlying_provider.process_webhook(payload).await
    }
}
