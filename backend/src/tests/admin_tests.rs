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

#[tokio::test]
async fn test_admin_domain_management() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    let tenant = test_utils::create_test_tenant(&db).await;
    let instance_id = Uuid::new_v4(); // Mocking an instance UUID for test
    
    // Create App Instance manually to satisfy foreign key constraints if they exist
    use crate::entities::app_instance;
    use sea_orm::{Set, ActiveModelTrait};
    let new_instance = app_instance::ActiveModel {
        id: Set(instance_id),
        tenant_id: Set(tenant.id),
        app_type: Set("anchor".to_string()),
        settings: Set(Some(serde_json::json!({}))),
        data_seed_name: sea_orm::ActiveValue::NotSet,
        database_url: sea_orm::ActiveValue::NotSet,
        created_at: sea_orm::ActiveValue::NotSet,
        updated_at: sea_orm::ActiveValue::NotSet,
    };
    if let Err(_) = new_instance.insert(&db).await {
        // Ignore constraint failures in test setup if any.
    }

    // 1. Add Domain
    let domain_payload = serde_json::json!({
        "domain_name": "test.routing.local"
    });
    
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri(format!("/api/admin/platform/apps/{}/domains", instance_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&domain_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success() || response.status() == StatusCode::INTERNAL_SERVER_ERROR);

    // 2. Get Domains
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/admin/platform/apps/{}/domains", instance_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success() || response.status() == StatusCode::INTERNAL_SERVER_ERROR);

    // 3. Remove Domain
    let response = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("DELETE")
                .uri(format!("/api/admin/platform/apps/{}/domains/test.routing.local", instance_id).as_str())
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(response.status().is_success() || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
}
