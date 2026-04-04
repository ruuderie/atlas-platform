use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;

#[tokio::test]
async fn test_account_creation_and_listing() {
    let (app, db) = setup_test_app().await;
    
    // Register User A
    let tenant = test_utils::create_test_tenant(&db).await;
    let mut username_a = format!("accountuser{}", Uuid::new_v4());
    let (status, login_res_a) = test_utils::register_test_user(&app, tenant.id, &mut username_a).await;

    assert_eq!(status, StatusCode::CREATED);
    
    let user_a_token = login_res_a["token"].as_str().unwrap().to_string();
    
    // 1. Fetch my accounts
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/me/accounts")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 2. Create a new account
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/me/accounts")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "name": format!("Org {}", Uuid::new_v4()),
                    "tenant_id": tenant.id,
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_user_account_invitations() {
    let (app, db) = setup_test_app().await;
    let (admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // 1. Admin gets all accounts
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/admin/accounts")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
