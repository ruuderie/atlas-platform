use super::test_utils::*;
use uuid::Uuid;
use crate::traits::payment::{PaymentProvider, SubscriptionData, TransactionData};
use crate::services::billing::stripe_provider::StripeProvider;
use crate::services::billing::paddle_provider::PaddleProvider;
use crate::services::billing::zaprite_provider::ZapriteProvider;
use crate::services::billing::stablecoin_manager::StablecoinManager;

#[tokio::test]
async fn test_stripe_provider_abstraction() {
    let provider = StripeProvider::new("sk_test_123".to_string());
    let tenant_id = Uuid::new_v4();
    
    let sub = provider.create_subscription(tenant_id, "Pro Plan", 9900, "USD").await;
    assert!(sub.is_ok());
    assert_eq!(sub.unwrap().status, "active");

    let tx = provider.capture_payment(tenant_id, 15000, "USD").await;
    assert!(tx.is_ok());
    assert_eq!(tx.unwrap().amount, 15000);
}

#[tokio::test]
async fn test_paddle_provider_abstraction() {
    use wiremock::matchers::{method, path, header, body_json};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use serde_json::json;

    let mock_server = MockServer::start().await;
    let provider = PaddleProvider::with_base_url("api_key_123".to_string(), mock_server.uri());
    let tenant_id = Uuid::new_v4();

    let expected_payload = json!({
        "items": [{ "price_id": "pri_example", "quantity": 1 }]
    });

    Mock::given(method("POST"))
        .and(path("/subscriptions"))
        .and(header("authorization", "Bearer api_key_123"))
        .and(body_json(&expected_payload))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "sub_123"})))
        .expect(1)
        .mount(&mock_server)
        .await;
    
    // Test subscription
    let sub = provider.create_subscription(tenant_id, "pri_example", 4999, "USD").await;
    assert!(sub.is_ok());

    // Test payment capture
    let tx = provider.capture_payment(tenant_id, 4999, "USD").await;
    assert!(tx.is_ok());
    assert_eq!(tx.unwrap().status, "completed");
}

#[tokio::test]
async fn test_zaprite_provider_abstraction() {
    let provider = ZapriteProvider::new("api_key_123".to_string());
    let tenant_id = Uuid::new_v4();
    
    let tx = provider.capture_payment(tenant_id, 50000, "SATS").await;
    assert!(tx.is_ok());
    assert_eq!(tx.unwrap().currency, "SATS");
}

#[tokio::test]
async fn test_stablecoin_manager_routing() {
    let stripe_provider = StripeProvider::new("sk_test_123".to_string());
    let manager = StablecoinManager::new(Box::new(stripe_provider));
    
    let tenant_id = Uuid::new_v4();
    let sub = manager.create_subscription(tenant_id, "USDT Plan", 10000, "USDT").await;
    
    assert!(sub.is_ok());
    assert_eq!(sub.unwrap().status, "active");
}
