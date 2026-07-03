//! Waitlist endpoint — DB-backed integration tests.
//!
//! Uses the same `setup_test_app()` harness as the rest of the suite.
//! The Woodpecker CI pipeline provides a Postgres 15 service container
//! at `postgresql://postgres:postgres@database:5432/oplydbtest`.
//!
//! Tests:
//!   - Happy path: POST /api/pub/products/{slug}/waitlist → 201 + body
//!   - Response shape: position, status, product, market
//!   - Metadata storage: role + portfolio_size stored in lead_metadata JSONB
//!   - Variant path: POST /api/pub/products/{slug}/{variant}/waitlist
//!   - Minimal payload (email only): succeeds
//!   - Duplicate email: second call does not create a second lead
//!   - Missing email: 400 / 422 rejection

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::entities::atlas_lead;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

// ── helpers ───────────────────────────────────────────────────────────────────

/// Makes a POST request to the product-level waitlist endpoint and returns the
/// parsed JSON body + HTTP status.
async fn post_waitlist(
    slug: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let (app, _) = setup_test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/pub/products/{slug}/waitlist"))
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    let bytes  = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

/// Makes a POST to the variant-scoped waitlist endpoint.
async fn post_waitlist_variant(
    slug:    &str,
    variant: &str,
    payload: Value,
) -> (StatusCode, Value) {
    let (app, _) = setup_test_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/pub/products/{slug}/{variant}/waitlist"))
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    let bytes  = resp.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, body)
}

// ── Test: happy path ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_waitlist_returns_201_with_position() {
    let email = format!("wl_{}@test.com", Uuid::new_v4().simple());
    let (status, body) = post_waitlist(
        "folio",
        json!({ "email": email }),
    ).await;

    assert_eq!(status, StatusCode::CREATED,
        "expected 201; body: {body}");
    assert!(
        body.get("position").and_then(|v| v.as_u64()).is_some(),
        "response must include a numeric 'position'; got: {body}"
    );
    assert_eq!(
        body.get("status").and_then(|v| v.as_str()),
        Some("waiting"),
        "status field must be 'waiting'; got: {body}"
    );
}

// ── Test: response shape ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_waitlist_response_shape_has_required_fields() {
    let email = format!("shape_{}@test.com", Uuid::new_v4().simple());
    let (status, body) = post_waitlist(
        "folio",
        json!({
            "email":                email,
            "role":                 "Landlord",
            "portfolio_size_label": "6–20 units",
            "utm_source":           "social",
        }),
    ).await;

    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    // All four fields the marketing page relies on must be present
    assert!(body.get("position").is_some(), "missing 'position' in: {body}");
    assert!(body.get("status").is_some(),   "missing 'status' in: {body}");
    assert!(body.get("product").is_some(),  "missing 'product' in: {body}");
    assert!(body.get("market").is_some(),   "missing 'market' in: {body}");
}

// ── Test: role + portfolio_size stored in lead_metadata ──────────────────────

#[tokio::test]
async fn test_waitlist_stores_role_and_portfolio_size_in_metadata() {
    let email = format!("meta_{}@test.com", Uuid::new_v4().simple());
    let (app, db) = setup_test_app().await;

    let payload = json!({
        "email":                email,
        "role":                 "Property Manager",
        "portfolio_size_label": "21–100 units",
    });

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pub/products/folio/waitlist")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Verify the lead was persisted and metadata contains the expected fields
    let lead = atlas_lead::Entity::find()
        .filter(atlas_lead::Column::Email.eq(&email))
        .one(&db)
        .await
        .expect("DB query failed")
        .expect("lead should exist after successful waitlist submission");

    let meta: Value = serde_json::from_value(
        lead.lead_metadata.unwrap_or(Value::Null)
    ).unwrap_or(Value::Null);

    assert_eq!(
        meta.get("role").and_then(|v| v.as_str()),
        Some("Property Manager"),
        "lead_metadata.role mismatch: {meta}"
    );
    assert_eq!(
        meta.get("portfolio_size").and_then(|v| v.as_str()),
        Some("21–100 units"),
        "lead_metadata.portfolio_size mismatch: {meta}"
    );
}

// ── Test: variant-scoped endpoint ─────────────────────────────────────────────

#[tokio::test]
async fn test_waitlist_variant_endpoint_returns_201() {
    let email = format!("var_{}@test.com", Uuid::new_v4().simple());
    let (status, body) = post_waitlist_variant(
        "folio",
        "folio-home-br-pt",
        json!({ "email": email }),
    ).await;

    assert_eq!(status, StatusCode::CREATED,
        "variant waitlist should return 201; body: {body}");
    assert!(
        body.get("position").and_then(|v| v.as_u64()).is_some(),
        "position must be numeric; got: {body}"
    );
}

// ── Test: email-only payload ──────────────────────────────────────────────────

#[tokio::test]
async fn test_waitlist_minimal_email_only_payload_succeeds() {
    let email = format!("min_{}@test.com", Uuid::new_v4().simple());
    let (status, body) = post_waitlist(
        "folio",
        json!({ "email": email }),
    ).await;

    assert_eq!(status, StatusCode::CREATED,
        "email-only payload should succeed; body: {body}");
}

// ── Test: duplicate email dedup ───────────────────────────────────────────────

#[tokio::test]
async fn test_waitlist_duplicate_email_does_not_create_second_lead() {
    let (app1, db) = setup_test_app().await;
    let email = format!("dedup_{}@test.com", Uuid::new_v4().simple());

    // First submission
    let r1 = app1
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pub/products/folio/waitlist")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "email": email }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED, "first submission should be 201");

    // Second submission with same email
    let (app2, _) = setup_test_app().await;
    let r2 = app2
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pub/products/folio/waitlist")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "email": email }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    // Accept either 200 (already exists) or 201 — but NOT 5xx
    assert!(
        r2.status().is_success(),
        "duplicate email must not cause a server error; got {}", r2.status()
    );

    // Exactly one lead should exist for this email
    let count = atlas_lead::Entity::find()
        .filter(atlas_lead::Column::Email.eq(&email))
        .all(&db)
        .await
        .expect("DB query failed")
        .len();

    assert_eq!(count, 1,
        "duplicate submission must not create a second atlas_lead row");
}

// ── Test: missing email rejected ──────────────────────────────────────────────

#[tokio::test]
async fn test_waitlist_missing_email_is_rejected() {
    let (status, _) = post_waitlist(
        "folio",
        json!({ "role": "Landlord" }),  // no email
    ).await;

    assert!(
        status.is_client_error(),
        "payload without email should return 4xx; got {status}"
    );
}
