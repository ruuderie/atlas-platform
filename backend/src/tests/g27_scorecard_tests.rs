//! Integration tests for G-27 ScorecardService.
//! These are DB-backed tests (real schema via the test migrator).
//!
//! Coverage:
//!   - get_or_create: idempotency (double-call returns same ID)
//!   - open_session: creates a session linked to a scorecard
//!   - submit_entry: sparse entry per (session, dimension, contributor)
//!   - recompute_aggregates: smoke test — does not panic, returns Ok
//!   - find_similar: smoke test with no qualifying scorecards
//!
//! NOTE: These tests require a live test DB. They will be skipped gracefully if
//! the test DB is unavailable.

use uuid::Uuid;
use serde_json::json;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;
use crate::services::scorecard_service::ScorecardService;

/// Helper: insert a minimal scorecard template and one dimension directly via
/// raw SeaORM inserts so tests don't depend on a template API endpoint.
async fn create_test_template_and_dimension(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
) -> (Uuid, Uuid) {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::{
        atlas_scorecard_template::{self, ActiveModel as TemplateAM},
        atlas_scorecard_dimension::{self, ActiveModel as DimAM},
    };

    let template_id = Uuid::new_v4();
    TemplateAM {
        id: Set(template_id),
        tenant_id: Set(Some(tenant_id)),
        entity_type: Set("atlas_lead".to_owned()),
        name: Set("FMCSA Carrier Safety".to_owned()),
        description: Set(None),
        scoring_method: Set("weighted_average".to_owned()),
        scale_min: Set(rust_decimal::Decimal::ZERO),
        scale_max: Set(rust_decimal::Decimal::from(10)),
        min_entries_to_publish: Set(1), // low threshold for tests
        is_active: Set(true),
        version: Set(1),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    }
    .insert(db)
    .await
    .expect("template insert failed");

    let dim_id = Uuid::new_v4();
    DimAM {
        id: Set(dim_id),
        template_id: Set(template_id),
        tenant_id: Set(Some(tenant_id)),
        name: Set("Safety Score".to_owned()),
        description: Set(None),
        scale_type: Set("rating".to_owned()),
        scale_min: Set(Some(rust_decimal::Decimal::ZERO)),
        scale_max: Set(Some(rust_decimal::Decimal::from(10))),
        weight: Set(Some(rust_decimal::Decimal::from(1))),
        benchmark_tiers: Set(None),
        global_reference_value: Set(None),
        is_required: Set(false),
        display_order: Set(1),
        is_active: Set(true),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    }
    .insert(db)
    .await
    .expect("dimension insert failed");

    (template_id, dim_id)
}

#[tokio::test]
async fn test_scorecard_get_or_create_is_idempotent() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, _) = create_test_template_and_dimension(&db, tenant.id).await;

    let subject_id = Uuid::new_v4();

    // First call: creates
    let id_a = ScorecardService::get_or_create(
        &db,
        tenant.id,
        template_id,
        "atlas_lead",
        subject_id,
    )
    .await
    .expect("first get_or_create failed");

    // Second call: must return the same ID
    let id_b = ScorecardService::get_or_create(
        &db,
        tenant.id,
        template_id,
        "atlas_lead",
        subject_id,
    )
    .await
    .expect("second get_or_create failed");

    assert_eq!(id_a, id_b, "get_or_create must be idempotent");
}

#[tokio::test]
async fn test_scorecard_open_session_and_submit_entry() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    let subject_id = Uuid::new_v4();
    let rater_id = Uuid::new_v4();

    let scorecard_id = ScorecardService::get_or_create(
        &db,
        tenant.id,
        template_id,
        "atlas_lead",
        subject_id,
    )
    .await
    .expect("get_or_create failed");

    let session_id = ScorecardService::open_session(
        &db,
        scorecard_id,
        rater_id,
        tenant.id,
        chrono::Utc::now(),
        "call",
        None,
        None,
        Some(json!({"duration_days": 30})),
    )
    .await
    .expect("open_session failed");

    assert_ne!(session_id, Uuid::nil());

    // Submit a sparse entry (rating score on the safety dimension)
    ScorecardService::submit_entry(
        &db,
        session_id,
        scorecard_id,
        dim_id,
        rater_id,
        tenant.id,
        Some(rust_decimal::Decimal::from(8)),
        None, // no option_id (not a poll dimension)
        Some("Good safety record"),
        Some(json!({"duration_days": 30})),
        true, // is_verified
    )
    .await
    .expect("submit_entry failed");
}

#[tokio::test]
async fn test_scorecard_submit_entry_is_idempotent_per_contributor() {
    // The UNIQUE constraint on (session_id, dimension_id, contributor_user_id)
    // should prevent duplicate entries. A second submit for the same triple must fail.
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    let subject_id = Uuid::new_v4();
    let rater_id = Uuid::new_v4();

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", subject_id,
    ).await.unwrap();

    let session_id = ScorecardService::open_session(
        &db, scorecard_id, rater_id, tenant.id, chrono::Utc::now(),
        "review", None, None, None,
    ).await.unwrap();

    // First submit: OK
    ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, rater_id, tenant.id,
        Some(rust_decimal::Decimal::from(7)), None, None, None, true,
    ).await.expect("first submit must succeed");

    // Second submit for same (session, dimension, contributor): must fail with DB error
    let result = ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, rater_id, tenant.id,
        Some(rust_decimal::Decimal::from(9)), None, None, None, true,
    ).await;

    assert!(
        result.is_err(),
        "second submit for same (session, dimension, contributor) must fail (UNIQUE constraint)"
    );
}

#[tokio::test]
async fn test_recompute_aggregates_smoke() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    let subject_id = Uuid::new_v4();
    let rater_id = Uuid::new_v4();

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", subject_id,
    ).await.unwrap();

    let session_id = ScorecardService::open_session(
        &db, scorecard_id, rater_id, tenant.id, chrono::Utc::now(),
        "inspection", None, None, Some(json!({"duration_days": 90})),
    ).await.unwrap();

    ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, rater_id, tenant.id,
        Some(rust_decimal::Decimal::from(8)), None, None,
        Some(json!({"duration_days": 90})), true,
    ).await.unwrap();

    // recompute_aggregates should succeed without panicking
    ScorecardService::recompute_aggregates(&db, scorecard_id)
        .await
        .expect("recompute_aggregates must not fail");
}

#[tokio::test]
async fn test_find_similar_returns_empty_when_no_qualified_scorecards() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, _) = create_test_template_and_dimension(&db, tenant.id).await;

    // No scorecards exist yet — find_similar should return an empty vec, not an error
    let results = ScorecardService::find_similar(
        &db,
        tenant.id,
        template_id,
        vec![8.0, 7.0, 9.0],
        10,
        "medium",
    )
    .await
    .expect("find_similar must not fail on empty result");

    assert!(
        results.is_empty(),
        "find_similar must return empty vec when no scorecards meet confidence threshold"
    );
}
