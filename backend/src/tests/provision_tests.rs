use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use tower::ServiceExt;

use crate::entities::{app_domain, app_instance, account, tenant, user, user_account};
use crate::models::provision::validate_domain;
use super::api_tests::setup_test_app;

// ── validate_domain unit tests (no DB) ────────────────────────────────────────

#[test]
fn validate_domain_rejects_scheme() {
    assert!(validate_domain("https://acme.com").is_err());
    assert!(validate_domain("http://acme.com").is_err());
}

#[test]
fn validate_domain_rejects_path() {
    assert!(validate_domain("acme.com/path").is_err());
    assert!(validate_domain("acme.com/").is_err());
}

#[test]
fn validate_domain_rejects_port() {
    assert!(validate_domain("acme.com:8080").is_err());
}

#[test]
fn validate_domain_accepts_fqdn() {
    assert!(validate_domain("acme.com").is_ok());
    assert!(validate_domain("sub.acme.co.uk").is_ok());
    assert!(validate_domain("dev.my-company.io").is_ok());
}

#[test]
fn validate_domain_accepts_localhost() {
    assert!(validate_domain("localhost").is_ok());
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Unique domain per test to avoid conflicts when tests share a DB.
fn unique_domain() -> String {
    format!("test-{}.provisiontest.io", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
}

fn unique_slug() -> String {
    format!("test-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap())
}

async fn do_provision(
    app: &axum::Router,
    admin_token: &str,
    slug: &str,
    domain: &str,
) -> (StatusCode, serde_json::Value) {
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name": slug,
                    "display_name": format!("Test Tenant {}", slug),
                    "domain": domain,
                    "admin_email": format!("admin@{}", domain),
                    "admin_first_name": "Test",
                    "admin_last_name": "Admin",
                    "bypass_dns_verification": true
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, body)
}

// ── Integration tests ─────────────────────────────────────────────────────────

/// Provision creates all required rows atomically.
#[tokio::test]
async fn provision_creates_all_required_rows() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let domain = unique_domain();

    let (status, body) = do_provision(&app, &admin_token, &slug, &domain).await;
    assert_eq!(status, StatusCode::CREATED, "Expected 201, got: {:?}", body);

    let tenant_id: uuid::Uuid = body["tenant_id"].as_str()
        .and_then(|s| s.parse().ok())
        .expect("tenant_id in response");
    let account_id: uuid::Uuid = body["account_id"].as_str()
        .and_then(|s| s.parse().ok())
        .expect("account_id in response");

    // Assert: tenant row
    let t = tenant::Entity::find_by_id(tenant_id).one(&db).await.unwrap();
    assert!(t.is_some(), "tenant row should exist");
    assert_eq!(t.unwrap().name, slug);

    // Assert: account row
    let a = account::Entity::find_by_id(account_id).one(&db).await.unwrap();
    assert!(a.is_some(), "account row should exist");
    assert_eq!(a.unwrap().tenant_id, tenant_id);

    // Assert: app_instance row (anchor)
    let instances = app_instance::Entity::find()
        .filter(app_instance::Column::TenantId.eq(tenant_id))
        .all(&db).await.unwrap();
    assert!(!instances.is_empty(), "at least one app_instance should exist");
    assert!(instances.iter().any(|i| i.app_type == "anchor"), "anchor instance required");

    // Assert: app_domain row
    let anchor_instance = instances.iter().find(|i| i.app_type == "anchor").unwrap();
    let domains = app_domain::Entity::find()
        .filter(app_domain::Column::AppInstanceId.eq(anchor_instance.id))
        .all(&db).await.unwrap();
    assert!(!domains.is_empty(), "app_domain row should exist");
    assert_eq!(domains[0].domain_name, domain);

    // Assert: user row
    let users = user::Entity::find()
        .filter(user::Column::Email.eq(format!("admin@{}", domain)))
        .all(&db).await.unwrap();
    assert!(!users.is_empty(), "admin user should exist");
    let user_id = users[0].id;

    // Assert: user_account (Owner) row
    let ua = user_account::Entity::find()
        .filter(user_account::Column::UserId.eq(user_id))
        .filter(user_account::Column::AccountId.eq(account_id))
        .one(&db).await.unwrap();
    assert!(ua.is_some(), "user_account row should exist");
    assert_eq!(ua.unwrap().role, user_account::UserRole::Owner);

    // Assert: setup_url in response
    let setup_url = body["setup_url"].as_str().expect("setup_url in response");
    assert!(setup_url.contains(&domain), "setup_url should contain the domain");
    assert!(setup_url.contains("setup-passkey"), "setup_url should point to the passkey setup page");
}

/// Second provision call with same domain returns 409.
#[tokio::test]
async fn provision_is_idempotent_on_duplicate_domain() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let domain = unique_domain();

    // First call — should succeed
    let (status, body) = do_provision(&app, &admin_token, &unique_slug(), &domain).await;
    assert_eq!(status, StatusCode::CREATED, "First provision should succeed: {:?}", body);

    // Second call with same domain — should conflict
    let (status2, body2) = do_provision(&app, &admin_token, &unique_slug(), &domain).await;
    assert_eq!(status2, StatusCode::CONFLICT, "Duplicate domain should return 409: {:?}", body2);
}

/// Second provision call with same slug returns 409.
#[tokio::test]
async fn provision_rejects_duplicate_slug() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();

    let (status, _) = do_provision(&app, &admin_token, &slug, &unique_domain()).await;
    assert_eq!(status, StatusCode::CREATED);

    let (status2, body2) = do_provision(&app, &admin_token, &slug, &unique_domain()).await;
    assert_eq!(status2, StatusCode::CONFLICT, "Duplicate slug should return 409: {:?}", body2);
}

/// Non-SuperAdmin callers are rejected.
#[tokio::test]
async fn provision_rejected_without_super_admin_role() {
    let (app, db) = setup_test_app().await;

    // Create a regular tenant user (Owner role on a tenant, not PlatformSuperAdmin)
    let tenant = super::test_utils::create_test_tenant(&db).await;
    let mut username = format!("owner_{}", uuid::Uuid::new_v4());
    let (status, login_body) = super::test_utils::register_test_user(&app, tenant.id, &mut username).await;
    assert_eq!(status, StatusCode::CREATED);
    let owner_token = login_body["token"].as_str().unwrap().to_string();

    let (status, body) = do_provision(&app, &owner_token, &unique_slug(), &unique_domain()).await;
    assert_eq!(status, StatusCode::FORBIDDEN, "Owner role should be rejected: {:?}", body);

    // Unauthenticated
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name": unique_slug(),
                    "display_name": "Test",
                    "domain": unique_domain(),
                    "admin_email": "a@b.com",
                    "admin_first_name": "A",
                    "admin_last_name": "B"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "Unauthenticated should return 401");
}

/// Bad domain format returns 400.
#[tokio::test]
async fn provision_rejects_invalid_domain_format() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let bad_domains = vec![
        "https://acme.com",
        "acme.com/path",
        "acme.com:8080",
    ];

    for bad_domain in bad_domains {
        let response = app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/admin/tenants/provision")
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", admin_token))
                    .header("Host", "localhost")
                    .body(Body::from(json!({
                        "tenant_name": unique_slug(),
                        "display_name": "Test",
                        "domain": bad_domain,
                        "admin_email": "admin@test.com",
                        "admin_first_name": "A",
                        "admin_last_name": "B"
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(), StatusCode::BAD_REQUEST,
            "domain '{}' should return 400", bad_domain
        );
    }
}

/// Provisioned tenant should immediately pass the dynamic CORS check.
#[tokio::test]
async fn provision_adds_domain_to_cors_registry() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let domain = unique_domain();

    // 1. Try a request with the new origin before provisioning.
    let req_before = Request::builder()
        .method("OPTIONS")
        .uri("/health")
        .header("Origin", format!("https://{}", domain))
        .header("Access-Control-Request-Method", "GET")
        .header("Access-Control-Request-Headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let res_before = app.clone().oneshot(req_before).await.unwrap();
    let allow_origin_before = res_before.headers().get("access-control-allow-origin");
    assert!(
        allow_origin_before.is_none() || allow_origin_before.unwrap() != &format!("https://{}", domain),
        "Domain should not be CORS-allowed before provisioning"
    );

    // 2. Perform provisioning
    let (status, body) = do_provision(&app, &admin_token, &slug, &domain).await;
    assert_eq!(status, StatusCode::CREATED, "Expected 201, got: {:?}", body);

    // 3. Test that the origin is now CORS allowed!
    let req_after = Request::builder()
        .method("OPTIONS")
        .uri("/health")
        .header("Origin", format!("https://{}", domain))
        .header("Access-Control-Request-Method", "GET")
        .header("Access-Control-Request-Headers", "content-type")
        .body(Body::empty())
        .unwrap();
    let res_after = app.clone().oneshot(req_after).await.unwrap();
    
    let allow_origin_after = res_after.headers().get("access-control-allow-origin")
        .expect("CORS header Access-Control-Allow-Origin must be present after provisioning");
    assert_eq!(
        allow_origin_after,
        &format!("https://{}", domain),
        "Domain should be dynamically CORS-allowed after provisioning"
    );
}

/// Provision fails if DNS verification is not bypassed and is missing TXT record.
#[tokio::test]
async fn provision_fails_without_dns_bypass_and_missing_txt() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let domain = unique_domain();

    // Trigger without bypass_dns_verification (should attempt real DNS challenge and fail)
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name": slug,
                    "display_name": format!("Test Tenant {}", slug),
                    "domain": domain,
                    "admin_email": format!("admin@{}", domain),
                    "admin_first_name": "Test",
                    "admin_last_name": "Admin",
                    "bypass_dns_verification": false
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let msg = body["message"].as_str().unwrap_or("");
    assert!(msg.contains("DNS verification failed"), "Expected DNS verification error, got: {}", msg);
}

/// Folio-only tenant (no anchor) succeeds — domain binds to the folio instance.
#[tokio::test]
async fn provision_folio_only_without_anchor_succeeds() {
    let (app, db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let domain = unique_domain();

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name": slug,
                    "display_name": format!("Test Folio Tenant {}", slug),
                    "domain": domain,
                    "admin_email": format!("admin@{}", domain),
                    "admin_first_name": "Test",
                    "admin_last_name": "Admin",
                    "bypass_dns_verification": true,
                    "apps": ["property_management"]
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED, "Folio-only provision should succeed");

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let tenant_id: uuid::Uuid = body["tenant_id"].as_str()
        .and_then(|s| s.parse().ok())
        .expect("tenant_id in response");

    // Verify folio instance was created
    let instances = app_instance::Entity::find()
        .filter(app_instance::Column::TenantId.eq(tenant_id))
        .all(&db).await.unwrap();
    assert_eq!(instances.len(), 1, "only one instance should exist");
    assert_eq!(instances[0].app_type, "property_management", "folio instance should have canonical app_type");

    // Verify domain is bound to the folio instance (not anchor)
    let domains = app_domain::Entity::find()
        .filter(app_domain::Column::AppInstanceId.eq(instances[0].id))
        .all(&db).await.unwrap();
    assert!(!domains.is_empty(), "domain should be bound to the folio instance");
    assert_eq!(domains[0].domain_name, domain);
}

/// Unknown app type in the apps list is rejected with 400.
#[tokio::test]
async fn provision_rejects_unknown_app_type() {
    let (app, _db) = setup_test_app().await;
    let (_, admin_token) = super::test_utils::create_and_login_admin_user(&app, &_db).await;

    let slug = unique_slug();
    let domain = unique_domain();

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name": slug,
                    "display_name": "Bad App Test",
                    "domain": domain,
                    "admin_email": format!("admin@{}", domain),
                    "admin_first_name": "Test",
                    "admin_last_name": "Admin",
                    "bypass_dns_verification": true,
                    "apps": ["folio"]   // "folio" is the short alias, not the canonical ID
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST, "Unknown app type should return 400");
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    let msg = body["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("unknown app") && msg.contains("property_management"),
        "Error should mention the canonical valid value, got: {}",
        msg
    );
}
