use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use tower::ServiceExt;
use serde_json::json;
use std::env;
use uuid::Uuid;
use crate::{api, migration};
use hyper::body::Bytes;
use http_body_util::BodyExt;
use super::test_utils;

use fake::{Fake, faker::{
    company::en::CompanyName,
    internet::en::SafeEmail,
    phone_number::en::PhoneNumber,
}};
use crate::models::customer::Customer;
use crate::models::deal::DealModel;
use crate::models::contact::Contact;
async fn setup_test_app() -> (Router, DatabaseConnection) {
    let database_url = env::var("TEST_DATABASE_URL_LOCAL")
        .unwrap_or_else(|_| env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@127.0.0.1:5432/business_tenant_test".to_string()));

    let db = Database::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    migration::Migrator::fresh(&db)
        .await
        .expect("Failed to reset database");
    
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    let rate_limiter = crate::middleware::rate_limiter::RateLimiter::new();
    let app = api::create_router(db.clone())
        .layer(axum::Extension(db.clone()))
        .layer(axum::Extension(rate_limiter));
    (app, db)
}

#[tokio::test]
async fn test_crm_customers() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    
    // Test creating a customer
    let name = CompanyName().fake::<String>();
    let email = SafeEmail().fake::<String>();
    let phone = PhoneNumber().fake::<String>();

    // Note: Trying /admin/customers based on routes.rs
    let response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/customers")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": name,
                    "customer_type": "BusinessEntity",
                    "email": email,
                    "phone": phone,
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
                    }
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();
    
    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body_bytes).to_string();
    
    if status == StatusCode::NOT_FOUND {
        // Fallback to /api/admin/customers or /api/customers if /admin/customers is not found
        panic!("Route not found. Response: {}", body);
    }
    
    assert_eq!(status, StatusCode::CREATED, "Failed to create customer: {}", body);

    let customer: Customer = serde_json::from_str(&body).unwrap();
    assert_eq!(customer.name, name);
    assert_eq!(customer.email.unwrap(), email);

    // Test GET customer
    let get_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("GET")
                .uri(format!("/api/customers/{}", customer.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(get_response.status(), StatusCode::OK);

    // Test PUT customer
    let updated_name = format!("Updated {}", name);
    let put_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("PUT")
                .uri(format!("/api/customers/{}", customer.id))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": updated_name
                }).to_string()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(put_response.status(), StatusCode::OK);

    // Test DELETE customer
    let delete_response = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("DELETE")
                .uri(format!("/api/customers/{}", customer.id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_crm_deals() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    // Create a customer first
    let name = CompanyName().fake::<String>();
    let email = SafeEmail().fake::<String>();
    let phone = PhoneNumber().fake::<String>();

    let customer_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/customers")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": name,
                    "customer_type": "BusinessEntity",
                    "email": email,
                    "phone": phone,
                    "attributes": { "shipper": false, "carrier": false, "loan_seeker": false, "loan_broker": false, "software_vendor": false, "tenant": false, "software_development_client": false, "salesforce_client": false, "web3_client": false, "bitcoiner": false, "zk": false, "lender": false, "advertiser": false, "gp": false, "construction_contractor": false, "construction_client": false, "landlord": false }
                }).to_string())).unwrap()
        ).await.unwrap();
    
    let cust_bytes = axum::body::to_bytes(customer_res.into_body(), usize::MAX).await.unwrap();
    let customer: Customer = serde_json::from_slice(&cust_bytes).unwrap();

    // Test creating a deal
    let deal_name = "Major Contract".to_string();
    let deal_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/deals")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "customer_id": customer.id,
                    "name": deal_name,
                    "amount": 50000.0,
                    "status": "Open",
                    "stage": "Prospecting",
                    "close_date": null
                }).to_string())).unwrap()
        ).await.unwrap();
    
    let status = deal_res.status();
    let body_bytes = axum::body::to_bytes(deal_res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::CREATED, "Deal creation failed: {:?}", String::from_utf8_lossy(&body_bytes));

    let deal: DealModel = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(deal.name, deal_name);
    assert_eq!(deal.amount, 50000.0);
}

#[tokio::test]
async fn test_crm_contacts() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    // Create a customer first
    let name = CompanyName().fake::<String>();
    let customer_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/customers")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "name": name,
                    "customer_type": "BusinessEntity",
                    "attributes": { "shipper": false, "carrier": false, "loan_seeker": false, "loan_broker": false, "software_vendor": false, "tenant": false, "software_development_client": false, "salesforce_client": false, "web3_client": false, "bitcoiner": false, "zk": false, "lender": false, "advertiser": false, "gp": false, "construction_contractor": false, "construction_client": false, "landlord": false }
                }).to_string())).unwrap()
        ).await.unwrap();
    
    let cust_bytes = axum::body::to_bytes(customer_res.into_body(), usize::MAX).await.unwrap();
    let customer: Customer = serde_json::from_slice(&cust_bytes).unwrap();

    // Test creating a contact
    let contact_name = "John Doe".to_string();
    let contact_res = app.clone()
        .oneshot(
            Request::builder().header("Host", "localhost")
                .method("POST")
                .uri("/api/contacts")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .body(Body::from(json!({
                    "customer_id": customer.id,
                    "name": contact_name,
                    "first_name": "John",
                    "last_name": "Doe",
                    "email": "john@example.com"
                }).to_string())).unwrap()
        ).await.unwrap();
        
    let status = contact_res.status();
    let body_bytes = axum::body::to_bytes(contact_res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(status, StatusCode::CREATED, "Contact creation failed: {:?}", String::from_utf8_lossy(&body_bytes));

    let contact: Contact = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(contact.name, contact_name);
    assert_eq!(contact.first_name, Some("John".to_string()));
}
