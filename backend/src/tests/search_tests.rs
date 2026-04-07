use axum::{body::Body, http::{Request, StatusCode}};
use tower::ServiceExt;
use serde_json::Value as JsonValue;
use uuid::Uuid;
use chrono::Utc;
use sea_orm::{ConnectionTrait, Value};

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;

#[tokio::test]
async fn test_global_search_tenant_isolation_and_admin_bypass() {
    let (app, db) = setup_test_app().await;

    // Create Tenant Alpha and Tenant Beta
    let tenant_alpha = test_utils::create_test_tenant(&db).await;
    let tenant_beta = test_utils::create_test_tenant(&db).await;

    // Create an Admin user
    let (_admin_user, admin_jwt) = test_utils::create_and_login_admin_user(&app, &db).await;

    // Create a regular user (Non-Admin) in Tenant Alpha
    let mut reg_username = format!("reg{}", Uuid::new_v4());
    let (_, reg_response) = test_utils::register_test_user(&app, tenant_alpha.id, &mut reg_username).await;
    let reg_jwt = reg_response["token"].as_str().unwrap().to_string();

    // Manually insert search records. We use plain SQL to safely inject tsvector 
    // or we can try inserting via ActiveModel with a string if it maps safely. 
    // Wait, the DB column `searchable_text` might not auto-parse string to tsvector perfectly in seaorm 
    // for `custom("tsvector")` unless we use raw sql. To be safe, we insert using raw DB execution.
    let deal_1_id = Uuid::new_v4();
    let values_1: Vec<Value> = vec![Uuid::new_v4().into(), deal_1_id.into(), tenant_alpha.id.into(), Utc::now().into()];
    db.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        r#"
        INSERT INTO global_search_index (id, entity_type, entity_id, tenant_id, searchable_text, metadata, created_at, updated_at) 
        VALUES ($1, 'Deal', $2, $3, to_tsvector('english', 'Alpha Project Deal'), '{}', $4, $4)
        "#,
        values_1
    )).await.expect("Failed inserting test search index 1");

    let deal_2_id = Uuid::new_v4();
    let values_2: Vec<Value> = vec![Uuid::new_v4().into(), deal_2_id.into(), tenant_beta.id.into(), Utc::now().into()];
    db.execute(sea_orm::Statement::from_sql_and_values(
        sea_orm::DbBackend::Postgres,
        r#"
        INSERT INTO global_search_index (id, entity_type, entity_id, tenant_id, searchable_text, metadata, created_at, updated_at) 
        VALUES ($1, 'Deal', $2, $3, to_tsvector('english', 'Beta Secret Project'), '{}', $4, $4)
        "#,
        values_2
    )).await.expect("Failed inserting test search index 2");

    // 1. Test Regular User Searching with missing tenant_id (Should be FORBIDDEN)
    let req_no_tenant = Request::builder()
        .header("Host", "localhost")
        .method("GET")
        .uri("/api/v1/search?q=Project")
        .header("Authorization", format!("Bearer {}", reg_jwt))
        .body(Body::empty())
        .unwrap();

    let res_no_tenant = app.clone().oneshot(req_no_tenant).await.unwrap();
    let status = res_no_tenant.status();
    if status != StatusCode::FORBIDDEN {
        let b = axum::body::to_bytes(res_no_tenant.into_body(), usize::MAX).await.unwrap();
        panic!("Expected 403, got {}, body: {}", status, String::from_utf8_lossy(&b));
    }

    // 2. Test Regular User Searching WITH their own tenant_id (Should see Alpha)
    let req_alpha = Request::builder()
        .header("Host", "localhost")
        .method("GET")
        .uri(&format!("/api/v1/search?q=Project&tenant_id={}", tenant_alpha.id))
        .header("Authorization", format!("Bearer {}", reg_jwt))
        .body(Body::empty())
        .unwrap();

    let res_alpha = app.clone().oneshot(req_alpha).await.unwrap();
    assert_eq!(res_alpha.status(), StatusCode::OK);
    
    let bytes = axum::body::to_bytes(res_alpha.into_body(), usize::MAX).await.unwrap();
    let results: Vec<JsonValue> = serde_json::from_slice(&bytes).unwrap();
    // They should get 1 result (Alpha Project Deal)
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["tenant_id"].as_str().unwrap(), tenant_alpha.id.to_string());

    // 3. Test Admin User Searching WITHOUT tenant_id (Should see ALL)
    let req_admin = Request::builder()
        .header("Host", "localhost")
        .method("GET")
        .uri("/api/v1/search?q=Project")
        .header("Authorization", format!("Bearer {}", admin_jwt))
        .body(Body::empty())
        .unwrap();

    let res_admin = app.clone().oneshot(req_admin).await.unwrap();
    assert_eq!(res_admin.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(res_admin.into_body(), usize::MAX).await.unwrap();
    let results: Vec<JsonValue> = serde_json::from_slice(&bytes).unwrap();
    // They should get both 
    assert_eq!(results.len(), 2);
}
