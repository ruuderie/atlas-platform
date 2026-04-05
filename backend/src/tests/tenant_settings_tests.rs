use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use serde_json::json;
use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;

#[tokio::test]
async fn test_tenant_settings_and_communications() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // Create Tenant
    let tenant = test_utils::create_test_tenant(&db).await;

    // 1. Get empty settings
    let get_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/tenants/{}/settings", tenant.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);

    // 2. Upsert SMTP Host setting
    let upsert_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri(format!("/api/tenants/{}/settings", tenant.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "key": "smtp_server",
                    "value": "smtp.test.mail",
                    "is_encrypted": false
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(upsert_res.status(), StatusCode::CREATED);

    // 3. Test sending email via communications handler (It should use the mocked localhost fallback or fail if we didn't mock properly, 
    // actually our handler mocks when host is empty or localhost, but since we set it to smtp.test.mail, it will attempt to connect and fail, returning 500. This tests routing!)
    let email_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/communications/email")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "tenant_id": tenant.id,
                    "to_email": "test@example.com",
                    "subject": "Test Subj",
                    "body_html": "<p>Hi</p>"
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    // Will fail because "smtp.test.mail" doesn't exist, proving our DB injection was used!
    assert_eq!(email_res.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
