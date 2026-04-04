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
async fn test_crm_leads_lifecycle() {
    let (app, db) = setup_test_app().await;
    
    // Register User A
    let tenant = test_utils::create_test_tenant(&db).await;
    let mut username_a = format!("leaduser{}", Uuid::new_v4());
    let (status, login_res_a) = test_utils::register_test_user(&app, tenant.id, &mut username_a).await;

    assert_eq!(status, StatusCode::CREATED);
    
    let user_a_token = login_res_a["token"].as_str().unwrap().to_string();
    
    // Create an account and retrieve its ID for CRM mapping
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
    
    let account_body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let account_json: serde_json::Value = serde_json::from_slice(&account_body).unwrap();
    let account_id = account_json["id"].as_str().unwrap();

    // 1. Create a lead
    let lead_res = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/leads")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "account_id": account_id,
                    "name": "Johnny Test",
                    "first_name": "Johnny",
                    "last_name": "Test",
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(lead_res.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_crm_cases_and_notes() {
    let (app, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let mut username_a = format!("caseuser{}", Uuid::new_v4());
    let (_status, login_res_a) = test_utils::register_test_user(&app, tenant.id, &mut username_a).await;
    
    let user_a_token = login_res_a["token"].as_str().unwrap().to_string();
    
    // Create an account
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
    
    let account_body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let account_json: serde_json::Value = serde_json::from_slice(&account_body).unwrap();
    let account_id = account_json["id"].as_str().unwrap();

    // Create a Customer first
    let customer_res = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/customers")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "account_id": account_id,
                    "name": "Big Client Inc",
                    "customer_type": "BusinessEntity",
                    "attributes": {
                        "shipper": false,
                        "carrier": false,
                        "loan_seeker": false,
                        "loan_broker": false,
                        "software_vendor": false,
                        "tenant": false,
                        "software_development_client": false,
                        "salesforce_client": false,
                        "web3_client": false,
                        "bitcoiner": false,
                        "zk": false,
                        "lender": false,
                        "advertiser": false,
                        "gp": false,
                        "construction_contractor": false,
                        "construction_client": false,
                        "landlord": false
                    },
                    "status": "Active"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(customer_res.status(), StatusCode::CREATED);
    
    let cust_body = axum::body::to_bytes(customer_res.into_body(), usize::MAX).await.unwrap();
    let cust_json: serde_json::Value = serde_json::from_slice(&cust_body).unwrap();
    let customer_id = cust_json["id"].as_str().unwrap();

    // 1. Create a Case
    let case_res = app.clone()
        .oneshot(
            Request::builder()
                .header("Host", "localhost")
                .method("POST")
                .uri("/api/cases")
                .header("Authorization", format!("Bearer {}", user_a_token))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "customer_id": customer_id,
                    "title": "Bug in API",
                    "description": "Server responded with 500 error on checkout",
                    "priority": "High"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(case_res.status(), StatusCode::CREATED);
}
