//! Integration tests for the Landing Page Builder API.
//!
//! Covers:
//!   - POST /api/pub/lp-events          (public ingest — no auth)
//!   - GET  /api/admin/landing-pages/{id}/analytics
//!   - GET  /api/admin/landing-pages/{id}/pixels
//!   - PUT  /api/admin/landing-pages/{id}/pixels/{type}
//!
//! All tests share one migrated DB (initialize_database is a no-op after the
//! first call). Each test creates its own app_page row with a fresh UUID so
//! tests are fully isolated from each other.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use http_body_util::BodyExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use super::api_tests::setup_test_app;
use crate::entities::{app_page, atlas_lp_event};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Insert a bare-minimum app_page row and return its id.
async fn seed_app_page(db: &sea_orm::DatabaseConnection, app_id: &str, slug: &str) -> Uuid {
    let id = Uuid::new_v4();
    app_page::ActiveModel {
        id: Set(id),
        tenant_id: Set(Uuid::nil()), // platform sentinel
        app_id: Set(app_id.to_string()),
        slug: Set(slug.to_string()),
        locale: Set("en".to_string()),
        title: Set("Test Page".to_string()),
        description: Set("Test description".to_string()),
        page_type: Set("landing".to_string()),
        hero_payload: Set(None),
        blocks_payload: Set(None),
        is_published: Set(true),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("Failed to seed app_page")
    .id
}

/// POST /api/pub/lp-events  (no auth required)
async fn post_lp_event(app: &axum::Router, body: Value) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/pub/lp-events")
                .header("Host", "localhost")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// GET /api/admin/landing-pages/{id}/analytics  (Bearer auth)
async fn get_analytics(app: &axum::Router, page_id: Uuid, token: &str) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/landing-pages/{}/analytics", page_id))
                .header("Host", "localhost")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// GET /api/admin/landing-pages/{id}/pixels  (Bearer auth)
async fn get_pixels(app: &axum::Router, page_id: Uuid, token: &str) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/landing-pages/{}/pixels", page_id))
                .header("Host", "localhost")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

/// PUT /api/admin/landing-pages/{id}/pixels/{type}  (Bearer auth)
async fn set_pixel(
    app: &axum::Router,
    page_id: Uuid,
    pixel_type: &str,
    enabled: bool,
    token: &str,
) -> (StatusCode, Value) {
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/admin/landing-pages/{}/pixels/{}",
                    page_id, pixel_type
                ))
                .header("Host", "localhost")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .body(Body::from(
                    json!({ "enabled": enabled, "snippet": null }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — POST /api/pub/lp-events
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_post_lp_event_rejects_invalid_event_type() {
    let (app, db) = setup_test_app().await;
    let page_id = seed_app_page(&db, "folio", "lp-reject-test").await;

    let (status, _) = post_lp_event(
        &app,
        json!({
            "app_page_id":  page_id,
            "event_type":   "pageview",        // ← wrong name
            "session_id":   Uuid::new_v4(),
        }),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unknown event_type must be rejected"
    );
}

#[tokio::test]
async fn test_post_lp_event_rejects_empty_event_type() {
    let (app, db) = setup_test_app().await;
    let page_id = seed_app_page(&db, "folio", "lp-empty-event-test").await;

    let (status, _) = post_lp_event(
        &app,
        json!({
            "app_page_id":  page_id,
            "event_type":   "",
            "session_id":   Uuid::new_v4(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_post_lp_event_accepts_view() {
    let (app, db) = setup_test_app().await;
    let page_id = seed_app_page(&db, "folio", "lp-view-test").await;

    let session = Uuid::new_v4();
    let (status, _) = post_lp_event(
        &app,
        json!({
            "app_page_id":   page_id,
            "event_type":    "view",
            "session_id":    session,
            "utm_source":    "google",
            "utm_medium":    "cpc",
            "utm_campaign":  "folio-launch",
        }),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::ACCEPTED,
        "valid view event should be accepted"
    );

    // Verify the row landed in the DB
    let rows = atlas_lp_event::Entity::find()
        .filter(atlas_lp_event::Column::AppPageId.eq(page_id))
        .filter(atlas_lp_event::Column::SessionId.eq(session.to_string()))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1, "exactly one event row should be inserted");
    assert_eq!(rows[0].event_type, "view");
    assert_eq!(rows[0].utm_source.as_deref(), Some("google"));
}

#[tokio::test]
async fn test_post_lp_event_accepts_lead_submitted() {
    let (app, db) = setup_test_app().await;
    let page_id = seed_app_page(&db, "folio", "lp-lead-test").await;

    let session = Uuid::new_v4();
    let (status, _) = post_lp_event(
        &app,
        json!({
            "app_page_id": page_id,
            "event_type":  "lead_submitted",
            "session_id":  session,
        }),
    )
    .await;

    assert_eq!(status, StatusCode::ACCEPTED);

    let rows = atlas_lp_event::Entity::find()
        .filter(atlas_lp_event::Column::AppPageId.eq(page_id))
        .filter(atlas_lp_event::Column::EventType.eq("lead_submitted"))
        .all(&db)
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn test_post_lp_event_accepts_cta_click() {
    let (app, db) = setup_test_app().await;
    let page_id = seed_app_page(&db, "folio", "lp-cta-test").await;

    let (status, _) = post_lp_event(
        &app,
        json!({
            "app_page_id": page_id,
            "event_type":  "cta_click",
            "session_id":  Uuid::new_v4(),
        }),
    )
    .await;

    assert_eq!(status, StatusCode::ACCEPTED);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — GET /api/admin/landing-pages/{id}/analytics
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_analytics_returns_zeros_for_empty_page() {
    let (app, db) = setup_test_app().await;
    let (_, token) = crate::tests::test_utils::create_and_login_admin_user(&app, &db).await;
    let page_id = seed_app_page(&db, "folio", "lp-analytics-empty").await;

    let (status, body) = get_analytics(&app, page_id, &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_views"], 0, "no events → 0 views");
    assert_eq!(body["total_leads"], 0, "no events → 0 leads");
    assert_eq!(body["cta_clicks"], 0, "no events → 0 CTA clicks");
    assert_eq!(
        body["conv_rate_pct"], 0.0,
        "0 views → conv rate 0.0, not NaN"
    );
}

#[tokio::test]
async fn test_analytics_counts_events_correctly() {
    let (app, db) = setup_test_app().await;
    let (_, token) = crate::tests::test_utils::create_and_login_admin_user(&app, &db).await;
    let page_id = seed_app_page(&db, "folio", "lp-analytics-counted").await;

    // Fire 4 views + 2 lead_submitted + 1 cta_click directly into the DB
    // (bypassing the HTTP ingest to keep the test fast and focused on aggregation)
    for (event_type, utm) in [
        ("view", "google"),
        ("view", "linkedin"),
        ("view", "google"),
        ("view", "email"),
        ("lead_submitted", "google"),
        ("lead_submitted", "linkedin"),
        ("cta_click", "google"),
    ] {
        atlas_lp_event::ActiveModel {
            id: Set(Uuid::new_v4()),
            app_page_id: Set(page_id),
            event_type: Set(event_type.to_string()),
            session_id: Set(Uuid::new_v4().to_string()),
            utm_source: Set(Some(utm.to_string())),
            utm_medium: Set(None),
            utm_campaign: Set(None),
            utm_content: Set(None),
            utm_term: Set(None),
            viewport: Set(None),
            referrer: Set(None),
            country_code: Set(None),
            created_at: Set(Utc::now()),
        }
        .insert(&db)
        .await
        .unwrap();
    }

    let (status, body) = get_analytics(&app, page_id, &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_views"], 4);
    assert_eq!(body["total_leads"], 2);
    assert_eq!(body["cta_clicks"], 1);

    // 2/4 * 100 = 50.0
    let rate = body["conv_rate_pct"].as_f64().unwrap();
    assert!((rate - 50.0).abs() < 0.01, "expected 50.0%, got {rate}");

    // Sources should be present
    let sources = body["sources"].as_array().unwrap();
    assert!(!sources.is_empty(), "should have UTM source breakdown");
    // google has 3 events (2 views + 1 lead) — must be first (sorted desc)
    assert_eq!(sources[0]["source"].as_str().unwrap(), "google");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — GET/PUT /api/admin/landing-pages/{id}/pixels
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_pixels_returns_all_disabled_for_new_page() {
    let (app, db) = setup_test_app().await;
    let (_, token) = crate::tests::test_utils::create_and_login_admin_user(&app, &db).await;
    let page_id = seed_app_page(&db, "folio", "lp-pixels-new").await;

    let (status, body) = get_pixels(&app, page_id, &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ga4"]["enabled"], false);
    assert_eq!(body["meta"]["enabled"], false);
    assert_eq!(body["linkedin"]["enabled"], false);
    assert_eq!(body["gtm"]["enabled"], false);
}

#[tokio::test]
async fn test_set_pixel_rejects_unknown_provider() {
    let (app, db) = setup_test_app().await;
    let (_, token) = crate::tests::test_utils::create_and_login_admin_user(&app, &db).await;
    let page_id = seed_app_page(&db, "folio", "lp-bad-pixel").await;

    let (status, _) = set_pixel(&app, page_id, "tiktok", true, &token).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "unknown pixel type must be rejected"
    );

    let (status, _) = set_pixel(&app, page_id, "GA4", true, &token).await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "wrong case must be rejected"
    );
}

#[tokio::test]
async fn test_set_pixel_enables_and_get_pixel_reads_back() {
    let (app, db) = setup_test_app().await;
    let (_, token) = crate::tests::test_utils::create_and_login_admin_user(&app, &db).await;
    let page_id = seed_app_page(&db, "folio", "lp-pixel-enable").await;

    // Enable ga4
    let (status, body) = set_pixel(&app, page_id, "ga4", true, &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["ga4"]["enabled"], true,
        "PUT response should reflect new state"
    );

    // GET pixels should now show ga4 enabled
    let (status, body) = get_pixels(&app, page_id, &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["ga4"]["enabled"], true,
        "ga4 should be persisted as enabled"
    );
    assert_eq!(
        body["meta"]["enabled"], false,
        "meta should remain disabled"
    );
}

#[tokio::test]
async fn test_set_pixel_preserves_previously_enabled_siblings() {
    let (app, db) = setup_test_app().await;
    let (_, token) = crate::tests::test_utils::create_and_login_admin_user(&app, &db).await;
    let page_id = seed_app_page(&db, "folio", "lp-pixel-siblings").await;

    // Enable ga4 first
    let (s, _) = set_pixel(&app, page_id, "ga4", true, &token).await;
    assert_eq!(s, StatusCode::OK);

    // Enable linkedin — must NOT wipe ga4
    let (s, body) = set_pixel(&app, page_id, "linkedin", true, &token).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(
        body["ga4"]["enabled"], true,
        "ga4 must survive linkedin toggle"
    );
    assert_eq!(
        body["linkedin"]["enabled"], true,
        "linkedin should now be enabled"
    );
    assert_eq!(
        body["meta"]["enabled"], false,
        "meta should remain untouched"
    );

    // Disable ga4 — linkedin must survive
    let (s, body) = set_pixel(&app, page_id, "ga4", false, &token).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(body["ga4"]["enabled"], false, "ga4 should now be disabled");
    assert_eq!(
        body["linkedin"]["enabled"], true,
        "linkedin must survive ga4 disable"
    );

    // Final GET confirms full state
    let (_, body) = get_pixels(&app, page_id, &token).await;
    assert_eq!(body["ga4"]["enabled"], false);
    assert_eq!(body["linkedin"]["enabled"], true);
    assert_eq!(body["meta"]["enabled"], false);
    assert_eq!(body["gtm"]["enabled"], false);
}
