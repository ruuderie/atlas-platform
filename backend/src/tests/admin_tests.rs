use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;

#[tokio::test]
async fn test_admin_user_management() {
    let (app, db) = setup_test_app().await;
    
    // Create admin user
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // Create a regular user
    let tenant = test_utils::create_test_tenant(&db).await;
    let mut username = format!("regularuser{}", Uuid::new_v4());
    let (status, login_res) = test_utils::register_test_user(&app, tenant.id, &mut username).await;

    
    assert_eq!(status, StatusCode::CREATED);
    
    let regular_user_id = login_res["user"]["id"].as_str().unwrap().to_string();
    
    // 1. List users
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/admin/users")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 2. Get specific user
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/admin/users/{}", regular_user_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 3. Toggle admin status
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri(format!("/api/admin/users/{}/toggle-admin", regular_user_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_listing_approvals() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // 1. Fetch pending listings
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/admin/listings/pending")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_admin_statistics() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // 1. Tenant Stats
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/admin/tenant-stats")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 2. Ad Purchases Stats
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri("/api/admin/ad-purchases/stats")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
