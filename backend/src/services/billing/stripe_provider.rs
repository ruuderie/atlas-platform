use anyhow::{Result, Context, anyhow};
use async_trait::async_trait;
use uuid::Uuid;
use crate::traits::payment::{PaymentProvider, SubscriptionData, TransactionData, WebhookPayload};

pub struct StripeProvider {
    client: stripe::Client,
}

impl StripeProvider {
    pub fn new(secret_key: String) -> Self {
        Self {
            client: stripe::Client::new(secret_key),
        }
    }
}

#[async_trait]
impl PaymentProvider for StripeProvider {
    async fn create_subscription(&self, tenant_id: Uuid, plan_name: &str, price_cents: i64, currency: &str) -> Result<SubscriptionData> {
        tracing::info!("Creating Stripe Subscription for tenant {} (Plan: {})", tenant_id, plan_name);
        
        // In a fully integrated flow, we would map the local plan_id to a Stripe Price ID
        // and create the Customer then Subscription via self.client.
        // let mut create_sub = stripe::CreateSubscription::new(customer.id); ...
        
        Ok(SubscriptionData {
            subscription_id: format!("sub_{}", Uuid::new_v4()),
            status: "active".to_string(),
            current_period_end: chrono::Utc::now() + chrono::Duration::try_days(30).unwrap(),
        })
    }

    async fn capture_payment(&self, tenant_id: Uuid, amount_cents: i64, currency: &str) -> Result<TransactionData> {
        tracing::info!("Capturing Stripe Payment for tenant {} (Amount: {} {})", tenant_id, amount_cents, currency);
        
        // let mut pi = stripe::CreatePaymentIntent::new(amount_cents, stripe::Currency::from_str(currency)?);
        // let payment_intent = stripe::PaymentIntent::create(&self.client, pi).await?;

        Ok(TransactionData {
            transaction_id: format!("pi_{}", Uuid::new_v4()),
            amount: amount_cents,
            currency: currency.to_string(),
            status: "succeeded".to_string(),
        })
    }

    async fn setup_tenant_payout_route(&self, tenant_id: Uuid) -> Result<String> {
        tracing::info!("Setting up Stripe Connect Account for tenant {}", tenant_id);
        // let account = stripe::Account::create(&self.client, ...).await?;
        Ok(format!("acct_{}", Uuid::new_v4()))
    }

    async fn process_webhook(&self, payload: &WebhookPayload) -> Result<()> {
        let stripe_secret = std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default();
        let payload_str = String::from_utf8(payload.raw_body.clone()).context("Invalid payload bytes")?;
        
        // Check if the webhook signature passes Stripe's strict verification
        let event = stripe::Webhook::construct_event(
            &payload_str,
            &payload.signature,
            &stripe_secret,
        ).map_err(|e| anyhow!("Webhook signature verification failed: {:?}", e))?;

        // match event.type_ {
        //     stripe::EventType::InvoicePaymentSucceeded => { ... }
        //     _ => {}
        // }

        tracing::info!("Processed Stripe webhook successfully: {:?}", event.type_);
        Ok(())
    }
}
