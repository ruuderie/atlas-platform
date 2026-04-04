use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use serde_json::json;
use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;
use uuid::Uuid;

#[tokio::test]
async fn test_magic_link_flow() {
    let (app, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    // Create user
    let mut username = format!("testuser{}", Uuid::new_v4());
    let (status, json_body) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    let email = json_body["user"]["email"]
        .as_str()
        .expect("No email returned in register response")
        .to_string();

    // 1. Request Magic Link for correct email
    let req_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": email,
                    "tenant_id": tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(req_res.status(), StatusCode::OK);

    // Read the token straight from DB
    use crate::entities::magic_link_token;
    use sea_orm::{EntityTrait, QueryOrder, ActiveModelTrait, Set};
    use chrono::{Utc, Duration};
    
    let token_model = magic_link_token::Entity::find()
        .order_by_desc(magic_link_token::Column::CreatedAt)
        .one(&db)
        .await
        .unwrap()
        .expect("No token created");

    // 2. Verify Magic Link successfully
    let ver_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": token_model.token,
                    "tenant_id": tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(ver_res.status(), StatusCode::OK);

    // 3. Test Expiration logic
    // Create an expired token manually
    let expired_token_str = format!("expired_{}", Uuid::new_v4());
    magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        token: Set(expired_token_str.clone()),
        expires_at: Set(Utc::now() - Duration::minutes(30)), // Expired 30 mins ago
        is_used: Set(false),
        created_at: Set(Utc::now() - Duration::hours(1)),
    }.insert(&db).await.unwrap();

    let ver_expired_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": expired_token_str,
                    "tenant_id": tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(ver_expired_res.status(), StatusCode::UNAUTHORIZED, "Expired tokens should be rejected");

    // 4. Test Tenant Isolation
    let other_tenant = test_utils::create_test_tenant(&db).await;
    
    // Create valid token for Tenant 1
    let isolated_token_str = format!("iso_{}", Uuid::new_v4());
    magic_link_token::ActiveModel {
        id: Set(Uuid::new_v4()),
        user_id: Set(token_model.user_id),
        token: Set(isolated_token_str.clone()),
        expires_at: Set(Utc::now() + Duration::minutes(30)), // Valid
        is_used: Set(false),
        created_at: Set(Utc::now()),
    }.insert(&db).await.unwrap();

    // Try consuming it inside Tenant 2
    let cross_tenant_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": isolated_token_str,
                    "tenant_id": other_tenant.id
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
        
    assert_eq!(cross_tenant_res.status(), StatusCode::UNAUTHORIZED, "Token mapped to Tenant A should not authenticate Tenant B");
}
