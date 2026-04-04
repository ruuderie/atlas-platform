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
    let (status, _) = test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    
    let email = format!("{}@example.com", username); // Usually how test_utils generates it or passing it in.
    // Wait, test_utils::register_test_user does `let mut email = format!("{}@test.com", username);` internally.
    let generated_email = format!("{}@test.com", username);

    // 1. Request Magic Link
    let req_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/request")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "email": generated_email
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(req_res.status(), StatusCode::OK);

    // Testing 'verify_magic_link' via HTTP isn't fully possible here because we don't know the generated token without reading the DB.
    // We'll read the token straight from DB!
    use crate::entities::magic_link_token;
    use sea_orm::{EntityTrait, QueryOrder};
    let token_model = magic_link_token::Entity::find()
        .order_by_desc(magic_link_token::Column::CreatedAt)
        .one(&db)
        .await
        .unwrap()
        .expect("No token created");

    // 2. Verify Magic Link
    let ver_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/auth/magic-link/verify")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "token": token_model.token
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    assert_eq!(ver_res.status(), StatusCode::OK);
    
    let body_bytes = axum::body::to_bytes(ver_res.into_body(), usize::MAX).await.unwrap();
    let _: crate::models::session::SessionResponse = serde_json::from_slice(&body_bytes).unwrap_or_else(|_| panic!("Failed parsing session response"));
}
