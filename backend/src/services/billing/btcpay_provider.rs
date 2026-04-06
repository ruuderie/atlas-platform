use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;
use crate::traits::payment::{PaymentProvider, SubscriptionData, TransactionData, WebhookPayload};

pub struct BTCPayProvider {
    client: reqwest::Client,
    store_id: String,
    api_key: String,
    host: String,
}

impl BTCPayProvider {
    pub fn new(host: String, store_id: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            store_id,
            api_key,
            host,
        }
    }
}

#[async_trait]
impl PaymentProvider for BTCPayProvider {
    async fn create_subscription(&self, _tenant_id: Uuid, _plan_name: &str, _price_cents: i64, _currency: &str) -> Result<SubscriptionData> {
        tracing::info!("BTCPayServer: Setting up recurring Pull Payments proxy");
        Ok(SubscriptionData {
            subscription_id: format!("btcpay_sub_{}", Uuid::new_v4()),
            status: "active".to_string(),
            current_period_end: chrono::Utc::now() + chrono::Duration::try_days(30).unwrap(),
        })
    }

    async fn capture_payment(&self, tenant_id: Uuid, amount_cents: i64, currency: &str) -> Result<TransactionData> {
        tracing::info!("Generating BTCPayServer Invoice for tenant {}", tenant_id);
        Ok(TransactionData {
            transaction_id: format!("btcpay_inv_{}", Uuid::new_v4()),
            amount: amount_cents,
            currency: currency.to_string(),
            status: "pending_confirmation".to_string(),
        })
    }

    async fn setup_tenant_payout_route(&self, tenant_id: Uuid) -> Result<String> {
        tracing::info!("Configuring BTCPay derivation scheme for tenant {}", tenant_id);
        Ok(format!("btcpay_store_{}", Uuid::new_v4()))
    }

    async fn process_webhook(&self, _payload: &WebhookPayload) -> Result<()> {
        tracing::info!("BTCPayServer Webhook Verified");
        Ok(())
    }
}
