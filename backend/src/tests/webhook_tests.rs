use sea_orm::{DatabaseConnection, Set, EntityTrait, ActiveModelTrait};
use uuid::Uuid;
use wiremock::matchers::{method, path, header, body_json};
use wiremock::{Mock, MockServer, ResponseTemplate};
use serde_json::json;

use crate::entities::webhook_endpoint;
use crate::services::webhook;
use crate::tests::api_tests::setup_test_app;

#[tokio::test]
async fn test_webhook_event_dispatch_to_wiremock() {
    let (_, db) = setup_test_app().await;
    

    let tenant_id = Uuid::new_v4();

    // 1. Start a local mock server
    let mock_server = MockServer::start().await;

    // 2. Set up a Mock to expect exactly 1 POST request matching specific criteria
    let payload = json!({
        "deal_id": "123",
        "value": 5000
    });

    Mock::given(method("POST"))
        .and(path("/webhook"))
        .and(header("X-Atlas-Event", "crm.deal.won"))
        // We will just verify it receives the json payload
        .and(body_json(&payload))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock_server)
        .await;

    // 3. Register the mock server URI as a webhook endpoint in the database
    let secret = "test_secret_key_123";
    webhook_endpoint::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        target_url: Set(format!("{}/webhook", mock_server.uri())),
        secret_key: Set(secret.to_string()),
        subscribed_events: Set(json!(["crm.deal.won", "listing.created"])),
        is_active: Set(true),
        created_at: Set(Some(chrono::Utc::now().into())),
        updated_at: Set(Some(chrono::Utc::now().into())),
        ..Default::default()
    }.insert(&db).await.unwrap();

    // 4. Dispatch the event
    webhook::dispatch_event(&db, tenant_id, "crm.deal.won", payload)
        .await
        .unwrap();

    // Give the async task a bit of time to make the HTTP request
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    // 5. The MockServer will automatically panic if it didn't receive exactly 1 request
    // matching all our criteria during its Drop trait when it goes out of scope here.
    // Or we can manually verify bindings if we wanted to assert the exact HMAC hash,
    // but the `body_json` and `header` ensures the wire protocol is respected.
}
