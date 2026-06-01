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
        atlas_scorecard_template::ActiveModel as TemplateAM,
        atlas_scorecard_dimension::ActiveModel as DimAM,
    };
    use rust_decimal::Decimal;
    use serde_json::json;

    let template_id = Uuid::new_v4();
    TemplateAM {
        id: Set(template_id),
        // tenant_id is Uuid (non-optional) per the entity
        tenant_id: Set(tenant_id),
        entity_type: Set("atlas_lead".to_owned()),
        name: Set("FMCSA Carrier Safety".to_owned()),
        description: Set(None),
        // "weighted_mean" is the canonical value per the entity doc comment
        scoring_method: Set("weighted_mean".to_owned()),
        // entity uses default_scale_min / default_scale_max
        default_scale_min: Set(Decimal::ZERO),
        default_scale_max: Set(Decimal::from(10)),
        min_entries_to_publish: Set(1),
        // entity uses is_published (no is_active / version fields)
        is_published: Set(false),
        created_by_user_id: Set(None),
        // created_at / updated_at are managed by the set_updated_at_column trigger
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("template insert failed");

    let dim_id = Uuid::new_v4();
    DimAM {
        id: Set(dim_id),
        template_id: Set(template_id),
        // tenant_id is Uuid (non-optional) per the entity
        tenant_id: Set(tenant_id),
        // slug is required (non-optional String)
        slug: Set("safety_score".to_owned()),
        name: Set("Safety Score".to_owned()),
        description: Set(None),
        category: Set(None),
        scale_type: Set("rating".to_owned()),
        // scale_min / scale_max / weight are non-optional Decimal
        scale_min: Set(Decimal::ZERO),
        scale_max: Set(Decimal::from(10)),
        weight: Set(Decimal::from(1)),
        unit_label: Set(None),
        // benchmark_tiers is non-optional Value (JsonBinary column)
        benchmark_tiers: Set(json!([])),
        global_reference_value: Set(None),
        global_reference_label: Set(None),
        min_entries_to_show: Set(1),
        is_community_ratable: Set(true),
        is_active: Set(true),
        // sort_order replaces display_order; no is_required field on the entity
        sort_order: Set(0),
        // created_at / updated_at are trigger-managed — do not set manually
        ..Default::default()
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

    // open_session(db, scorecard_id, rater_user_id, tenant_id, occurred_at,
    //              session_type, context_entity_type, context_entity_id, session_label)
    let session_id = ScorecardService::open_session(
        &db,
        scorecard_id,
        rater_id,
        tenant.id,
        chrono::Utc::now(),
        "call",
        None,
        None,
        Some("Q1 qualification call"), // session_label: Option<&str>
    )
    .await
    .expect("open_session failed");

    assert_ne!(session_id, Uuid::nil());

    // submit_entry(db, session_id, scorecard_id, dimension_id, tenant_id,
    //              contributor_user_id, score: Option<f64>, option_id,
    //              source_type: &str, context: Option<Value>, note: Option<&str>)
    ScorecardService::submit_entry(
        &db,
        session_id,
        scorecard_id,
        dim_id,
        tenant.id,
        rater_id,
        Some(8.0_f64),                          // score: Option<f64>
        None,                                    // option_id: not a poll dimension
        "manual",                                // source_type: &str
        None,                                    // context: Option<Value>
        Some("Good safety record"),              // note: Option<&str>
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
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(7.0_f64), None, "manual", None, None,
    ).await.expect("first submit must succeed");

    // Second submit for same (session, dimension, contributor): must fail with DB error
    let result = ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(9.0_f64), None, "manual", None, None,
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
        "inspection", None, None, Some("90-day review"),
    ).await.unwrap();

    ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(8.0_f64), None, "inspection", Some(json!({"duration_days": 90})), None,
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
