//! # Instance Lifecycle Integration Tests
//!
//! Guards the three new instance lifecycle endpoints added in the bootstrap_rbac
//! and instance management work:
//!
//! | Endpoint | Method | Handler |
//! |---|---|---|
//! | `/api/admin/app-instances/{id}/reset` | POST | `reset_instance` |
//! | `/api/admin/app-instances/{id}` | DELETE | `delete_instance` (soft) |
//! | `/api/admin/app-instances/{id}/reprovision-domain` | POST | `reprovision_domain` |
//!
//! ## Test strategy
//!
//! Each test provisions a fresh tenant (via the existing provision endpoint) to
//! get a real `instance_id`, then calls the lifecycle endpoint against it. This
//! tests the full HTTP → handler → DB round-trip using the CI test database.
//!
//! `reprovision_domain` calls the ingress sidecar, which is unavailable in CI.
//! Those tests assert the 400 precondition path (no custom_domain) and the
//! sidecar-failure path (502 BAD_GATEWAY), which do not require a live sidecar.

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

use crate::entities::{app_instance, atlas_app_deployment_config};
use super::api_tests::setup_test_app;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn unique_slug() -> String {
    format!("lctest-{}", Uuid::new_v4().to_string().split('-').next().unwrap())
}

fn unique_domain() -> String {
    format!("test-{}.lifecycle-test.io", Uuid::new_v4().to_string().split('-').next().unwrap())
}

async fn json_body(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Provision a tenant and return the primary (non-anchor) `instance_id`.
/// Replicates the pattern used in `domain_provisioning_tests.rs`.
async fn provision_and_get_instance_id(
    app:   &axum::Router,
    db:    &sea_orm::DatabaseConnection,
    token: &str,
    slug:  &str,
    dom:   &str,
) -> Uuid {
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name":            slug,
                    "display_name":           format!("LC Test {slug}"),
                    "domain":                 dom,
                    "admin_email":            format!("admin@{dom}"),
                    "admin_first_name":       "Test",
                    "admin_last_name":        "Admin",
                    "bypass_dns_verification": true,
                }).to_string()))
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED, "provision must succeed for lifecycle test setup");

    let body = json_body(resp).await;
    let tenant_id: Uuid = body["tenant_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .expect("provision response must include tenant_id");

    let instances = app_instance::Entity::find()
        .filter(app_instance::Column::TenantId.eq(tenant_id))
        .all(db)
        .await
        .expect("DB query for instances must succeed");

    // Prefer the non-anchor instance (property_management / network_instance), fall back to any.
    instances.iter()
        .find(|i| i.app_type != "anchor")
        .or_else(|| instances.first())
        .expect("at least one app_instance must exist after provision")
        .id
}

/// Seed atlas_app_deployment_config for the instance (the GET /public-config
/// endpoint lazy-creates it on first call).
async fn seed_deployment_config(app: &axum::Router, token: &str, instance_id: &Uuid) {
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/app-instances/{instance_id}/public-config"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();
}

// ── reset_instance tests ──────────────────────────────────────────────────────

/// POST /reset on a provisioned instance returns 200 with correct JSON.
#[tokio::test]
async fn reset_instance_returns_200_with_active_status() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    seed_deployment_config(&app, &token, &instance_id).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{instance_id}/reset"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "reset must return 200");
    let body = json_body(resp).await;
    assert_eq!(body["status"], "active", "reset response status must be 'active'");
    assert!(body.get("instance_id").is_some(), "reset response must include instance_id");
    assert!(body.get("note").is_some(), "reset response must include note field for UI toast");
}

/// POST /reset on a non-existent instance returns 404.
#[tokio::test]
async fn reset_instance_nonexistent_returns_404() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;
    let fake_id = Uuid::new_v4();

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{fake_id}/reset"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND,
        "reset on unknown instance must return 404");
}

/// POST /reset actually sets instance_status = 'active' in the DB.
#[tokio::test]
async fn reset_instance_sets_db_status_to_active() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    seed_deployment_config(&app, &token, &instance_id).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{instance_id}/reset"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify the DB row reflects "active"
    let row = atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
        .expect("DB query must succeed")
        .expect("deployment config row must exist after seed");

    assert_eq!(
        row.instance_status,
        atlas_app_deployment_config::AppInstanceStatus::Active,
        "DB must reflect Active status after reset"
    );
}

/// POST /reset requires auth — unauthenticated request returns 401.
#[tokio::test]
async fn reset_instance_requires_auth() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{instance_id}/reset"))
                .header("Host", "localhost")
                // No Authorization header
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert!(
        resp.status() == StatusCode::UNAUTHORIZED || resp.status() == StatusCode::FORBIDDEN,
        "reset without auth must return 401 or 403, got {}",
        resp.status()
    );
}

// ── delete_instance tests (soft-delete) ──────────────────────────────────────

/// DELETE /{id} returns 200 and sets status to 'archived' (soft delete).
#[tokio::test]
async fn delete_instance_soft_deletes_to_archived() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    seed_deployment_config(&app, &token, &instance_id).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/app-instances/{instance_id}"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK, "delete must return 200");
    let body = json_body(resp).await;
    assert_eq!(body["status"], "archived",
        "delete_instance is a soft delete — response status must be 'archived'");
}

/// DELETE /{id} verifies the DB row is 'archived', not removed.
#[tokio::test]
async fn delete_instance_row_still_exists_in_db() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    seed_deployment_config(&app, &token, &instance_id).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/app-instances/{instance_id}"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    // The DB row must still exist (soft delete)
    let row = atlas_app_deployment_config::Entity::find_by_id(instance_id)
        .one(&db)
        .await
        .expect("DB query must succeed");

    assert!(row.is_some(), "DB row must still exist after soft delete (row is archived, not removed)");
    assert_eq!(
        row.unwrap().instance_status,
        atlas_app_deployment_config::AppInstanceStatus::Archived,
        "DB status must be Archived after DELETE"
    );
}

/// DELETE /{id} on a non-existent instance returns 404.
#[tokio::test]
async fn delete_instance_nonexistent_returns_404() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;
    let fake_id = Uuid::new_v4();

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/app-instances/{fake_id}"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND,
        "delete of unknown instance must return 404");
}

// ── reprovision_domain tests ──────────────────────────────────────────────────

/// POST /reprovision-domain without a custom_domain set returns 400 BAD_REQUEST.
/// This is the most important test: it guards the precondition check that fires
/// BEFORE the ingress sidecar is contacted (so it works in CI without a sidecar).
#[tokio::test]
async fn reprovision_domain_without_custom_domain_returns_400() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    // Seed the deployment config row but do NOT set a custom_domain
    seed_deployment_config(&app, &token, &instance_id).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{instance_id}/reprovision-domain"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST,
        "reprovision-domain without a custom_domain must return 400, not 500");
}

/// POST /reprovision-domain with a custom_domain set attempts sidecar call.
/// In CI the sidecar is unavailable, so we expect either 200 (if mock is active)
/// or 502 BAD_GATEWAY (sidecar unreachable) — never a 4xx client error.
#[tokio::test]
async fn reprovision_domain_with_custom_domain_attempts_sidecar() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug        = unique_slug();
    let dom         = unique_domain();
    let custom_dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;

    // Seed deployment config, then set a custom_domain via PUT /public-config
    seed_deployment_config(&app, &token, &instance_id).await;
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/admin/app-instances/{instance_id}/public-config"))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::from(json!({ "custom_domain": custom_dom }).to_string()))
                .unwrap(),
        )
        .await.unwrap();

    // Now trigger reprovision
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{instance_id}/reprovision-domain"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    let status = resp.status();

    // 200 = sidecar mock responded (if INGRESS_SIDECAR_URL is set to a test double)
    // 502 = sidecar is unavailable in CI (expected in standard CI)
    // 400 = custom_domain precondition failed (MUST NOT happen — we set the domain)
    // 404 = instance not found (MUST NOT happen — we provisioned it)
    assert!(
        status == StatusCode::OK || status == StatusCode::BAD_GATEWAY,
        "reprovision-domain with a configured custom_domain must return 200 or 502 (sidecar result), got {status}"
    );
}

/// POST /reprovision-domain on a non-existent instance returns 404.
#[tokio::test]
async fn reprovision_domain_nonexistent_instance_returns_404() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;
    let fake_id = Uuid::new_v4();

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{fake_id}/reprovision-domain"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND,
        "reprovision-domain with unknown instance_id must return 404");
}

/// POST /reprovision-domain requires auth.
#[tokio::test]
async fn reprovision_domain_requires_auth() {
    let (app, db) = setup_test_app().await;
    let (_, token) = super::test_utils::create_and_login_admin_user(&app, &db).await;

    let slug = unique_slug();
    let dom  = unique_domain();
    let instance_id = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/app-instances/{instance_id}/reprovision-domain"))
                .header("Host", "localhost")
                // No Authorization header
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    assert!(
        resp.status() == StatusCode::UNAUTHORIZED || resp.status() == StatusCode::FORBIDDEN,
        "reprovision-domain without auth must return 401 or 403, got {}",
        resp.status()
    );
}
