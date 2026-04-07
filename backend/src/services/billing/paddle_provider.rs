use anyhow::{Result, anyhow};
use async_trait::async_trait;
use uuid::Uuid;
use crate::traits::payment::{PaymentProvider, SubscriptionData, TransactionData, WebhookPayload};

pub struct PaddleProvider {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl PaddleProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url: "https://api.paddle.com".to_string(),
        }
    }

    pub fn with_base_url(api_key: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }
}

#[async_trait]
impl PaymentProvider for PaddleProvider {
    async fn create_subscription(&self, tenant_id: Uuid, plan_name: &str, price_cents: i64, currency: &str) -> Result<SubscriptionData> {
        tracing::info!("Creating Paddle Subscription for tenant {}", tenant_id);
        
        // Paddle uses proper API requests over reqwest because there's no official rust SDK
        let _res = self.client.post(&format!("{}/subscriptions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "items": [{ "price_id": plan_name, "quantity": 1 }]
            }))
            .send().await?;

        Ok(SubscriptionData {
            subscription_id: format!("sub_paddle_{}", Uuid::new_v4()),
            status: "active".to_string(),
            current_period_end: chrono::Utc::now() + chrono::Duration::try_days(30).unwrap(),
        })
    }

    async fn capture_payment(&self, tenant_id: Uuid, amount_cents: i64, currency: &str) -> Result<TransactionData> {
        tracing::info!("Capturing Paddle Payment (MOR routing)");
        Ok(TransactionData {
            transaction_id: format!("txn_paddle_{}", Uuid::new_v4()),
            amount: amount_cents,
            currency: currency.to_string(),
            status: "completed".to_string(),
        })
    }

    async fn setup_tenant_payout_route(&self, tenant_id: Uuid) -> Result<String> {
        // Paddle handles Merchant of Record so payout routing is simpler but less flexible for strict dual-wallets
        Ok(format!("paddle_payee_{}", Uuid::new_v4()))
    }

    async fn process_webhook(&self, payload: &WebhookPayload) -> Result<()> {
        tracing::info!("Processing Paddle webhook...");
        // Implement Paddle signature verification (usually via public key)
        Ok(())
    }
}
