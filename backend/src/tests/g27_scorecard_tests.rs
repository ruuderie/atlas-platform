//! Integration tests for G-27 ScorecardService.
//! These are DB-backed tests (real schema via the test migrator).
//!
//! Coverage:
//!   - get_or_create: idempotency (double-call returns same ID)
//!   - open_session: creates a session linked to a scorecard
//!   - submit_entry: sparse entry per (session, dimension, contributor)
//!   - recompute_aggregates: smoke test — does not panic, returns Ok
//!   - find_similar: smoke test with no qualifying scorecards
//!   ── Gap fill (this session) ──────────────────────────────────
//!   - verify_entry (confirmed=true):  sets is_verified=true
//!   - verify_entry (confirmed=false): deletes the entry
//!   - transcript_inferred source_type: accepted by submit_entry
//!   - is_inverted recompute: recompute_aggregates runs without panic on inverted dim
//!   - get_display_rules (Starter): returns [] when tenant_setting absent
//!   - get_display_rules (enabled):  returns active rules ordered by priority
//!   - get_nudge_dimensions_for_activity: returns matching nudge dimensions

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
        None, // app_instance_id
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
        "review", None, None, None, None,
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
        "inspection", None, None, Some("90-day review"), None,
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
        vec![true, true, true], // target_mask: all 3 dims present
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

// ── Gap-fill tests ─────────────────────────────────────────────────────────

/// Helper: create a template + inverted dimension (lower score = better).
async fn create_inverted_dimension(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
) -> (Uuid, Uuid) {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::{
        atlas_scorecard_template::ActiveModel as TemplateAM,
        atlas_scorecard_dimension::ActiveModel as DimAM,
    };
    use rust_decimal::Decimal;

    let template_id = Uuid::new_v4();
    TemplateAM {
        id: Set(template_id),
        tenant_id: Set(tenant_id),
        entity_type: Set("atlas_lead".to_owned()),
        name: Set("Inverted Dim Template".to_owned()),
        scoring_method: Set("weighted_mean".to_owned()),
        default_scale_min: Set(Decimal::ZERO),
        default_scale_max: Set(Decimal::from(100)),
        min_entries_to_publish: Set(1),
        is_published: Set(false),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("template insert failed");

    let dim_id = Uuid::new_v4();
    DimAM {
        id: Set(dim_id),
        template_id: Set(template_id),
        tenant_id: Set(tenant_id),
        slug: Set("churn_rate".to_owned()),
        name: Set("Churn Rate".to_owned()),
        scale_type: Set("absolute".to_owned()),
        scale_min: Set(Decimal::ZERO),
        scale_max: Set(Decimal::from(100)),
        weight: Set(Decimal::from(1)),
        benchmark_tiers: Set(json!([])),
        is_active: Set(true),
        is_community_ratable: Set(true),
        min_entries_to_show: Set(1),
        // ── Gap 2: is_inverted ──
        is_inverted: Set(true),
        ..Default::default()
    }
    .insert(db)
    .await
    .expect("inverted dimension insert failed");

    (template_id, dim_id)
}

/// Helper: insert a display rule directly for a given template + dimension.
async fn insert_display_rule(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    template_id: Uuid,
    dimension_id: Uuid,
    priority: i32,
    action: &str,
) -> Uuid {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::atlas_scorecard_display_rule::ActiveModel as RuleAM;

    let rule_id = Uuid::new_v4();
    RuleAM {
        id: Set(rule_id),
        template_id: Set(template_id),
        dimension_id: Set(Some(dimension_id)),
        tenant_id: Set(tenant_id),
        category_target: Set(None),
        trigger_category: Set("activity_trigger".to_owned()),
        field_reference: Set(None),
        operator: Set("activity_type_is".to_owned()),
        value: Set(None),
        value_list: Set(Some(json!(["call", "demo"]))),
        action: Set(action.to_owned()),
        alert_message: Set(None),
        mode_scope: Set("always".to_owned()),
        priority: Set(priority),
        is_active: Set(true),
        description: Set(None),
        created_by_user_id: Set(None),
        created_at: Set(chrono::Utc::now()),
    }
    .insert(db)
    .await
    .expect("display rule insert failed");

    rule_id
}

/// verify_entry(confirmed=true): is_verified must be set to true on the entry.
#[tokio::test]
async fn test_verify_entry_confirm() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", Uuid::new_v4(),
    ).await.unwrap();

    let rater_id = Uuid::new_v4();
    let session_id = ScorecardService::open_session(
        &db, scorecard_id, rater_id, tenant.id, chrono::Utc::now(),
        "call", None, None, None, None,
    ).await.unwrap();

    let entry_id = ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(7.0), None, "transcript_inferred", None, None,
    ).await.expect("submit_entry failed");

    // Confirm the entry
    ScorecardService::verify_entry(&db, entry_id, tenant.id, true)
        .await
        .expect("verify_entry(confirmed=true) must not fail");

    // Assert is_verified = true in DB
    use sea_orm::EntityTrait;
    let entry = crate::entities::atlas_scorecard_entry::Entity::find_by_id(entry_id)
        .one(&db)
        .await
        .unwrap()
        .expect("entry must still exist after confirm");

    assert!(entry.is_verified, "confirmed entry must have is_verified = true");
}

/// verify_entry(confirmed=false): the entry must be deleted.
#[tokio::test]
async fn test_verify_entry_reject() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", Uuid::new_v4(),
    ).await.unwrap();

    let rater_id = Uuid::new_v4();
    let session_id = ScorecardService::open_session(
        &db, scorecard_id, rater_id, tenant.id, chrono::Utc::now(),
        "call", None, None, None, None,
    ).await.unwrap();

    let entry_id = ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(6.0), None, "transcript_inferred", None, None,
    ).await.expect("submit_entry failed");

    // Reject the entry
    ScorecardService::verify_entry(&db, entry_id, tenant.id, false)
        .await
        .expect("verify_entry(confirmed=false) must not fail");

    // Assert entry no longer exists
    use sea_orm::EntityTrait;
    let entry = crate::entities::atlas_scorecard_entry::Entity::find_by_id(entry_id)
        .one(&db)
        .await
        .unwrap();

    assert!(entry.is_none(), "rejected entry must be deleted from DB");
}

/// submit_entry with source_type='transcript_inferred' must be accepted.
/// The entry must be inserted with is_verified=false.
#[tokio::test]
async fn test_submit_entry_transcript_inferred_source_type() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", Uuid::new_v4(),
    ).await.unwrap();

    let rater_id = Uuid::new_v4();
    let session_id = ScorecardService::open_session(
        &db, scorecard_id, rater_id, tenant.id, chrono::Utc::now(),
        "call", None, None, None, None,
    ).await.unwrap();

    let entry_id = ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(8.0), None,
        "transcript_inferred",   // ← Gap 5: must be a valid SourceType variant
        Some(json!({"call_id": "call_abc", "confidence": 0.91})),
        None,
    ).await.expect("transcript_inferred must be a valid source_type");

    // Entry must exist and be unverified
    use sea_orm::EntityTrait;
    let entry = crate::entities::atlas_scorecard_entry::Entity::find_by_id(entry_id)
        .one(&db)
        .await
        .unwrap()
        .expect("entry must exist");

    assert!(
        !entry.is_verified,
        "transcript_inferred entries must be inserted with is_verified=false"
    );
}

/// recompute_aggregates on a scorecard with an is_inverted dimension must not panic.
#[tokio::test]
async fn test_recompute_aggregates_inverted_dimension() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_inverted_dimension(&db, tenant.id).await;

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", Uuid::new_v4(),
    ).await.unwrap();

    let rater_id = Uuid::new_v4();
    let session_id = ScorecardService::open_session(
        &db, scorecard_id, rater_id, tenant.id, chrono::Utc::now(),
        "review", None, None, None, None,
    ).await.unwrap();

    // Score of 20 (low = good for inverted: churn_rate)
    ScorecardService::submit_entry(
        &db, session_id, scorecard_id, dim_id, tenant.id, rater_id,
        Some(20.0), None, "manual", None, None,
    ).await.unwrap();

    ScorecardService::recompute_aggregates(&db, scorecard_id)
        .await
        .expect("recompute_aggregates must not fail on an inverted dimension");
}

/// get_display_rules must return [] for a tenant without the feature setting.
#[tokio::test]
async fn test_get_display_rules_returns_empty_for_starter_tenant() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    // Insert a rule (it should NOT be returned for Starter tenants)
    insert_display_rule(&db, tenant.id, template_id, dim_id, 10, "surface_as_nudge").await;

    // No tenant_setting inserted → Starter tier
    let rules = ScorecardService::get_display_rules(&db, template_id, tenant.id)
        .await
        .expect("get_display_rules must not fail");

    assert!(
        rules.is_empty(),
        "Starter tenant (no scorecard_display_rules_enabled setting) must receive []"
    );
}

/// get_display_rules returns active rules ordered by priority for an enabled tenant.
#[tokio::test]
async fn test_get_display_rules_returns_active_rules_for_enabled_tenant() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;

    // Enable the feature for this tenant
    {
        use sea_orm::{ActiveModelTrait, Set};
        use crate::entities::tenant_setting::ActiveModel as SettingAM;
        SettingAM {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant.id),
            key: Set("scorecard_display_rules_enabled".to_owned()),
            value: Set("true".to_owned()),
            is_encrypted: Set(false),
            updated_at: Set(chrono::Utc::now()),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(&db)
        .await
        .expect("tenant_setting insert failed");
    }

    // Insert two rules with different priorities
    insert_display_rule(&db, tenant.id, template_id, dim_id, 5,  "require").await;
    insert_display_rule(&db, tenant.id, template_id, dim_id, 20, "surface_as_nudge").await;

    let rules = ScorecardService::get_display_rules(&db, template_id, tenant.id)
        .await
        .expect("get_display_rules must not fail");

    assert_eq!(rules.len(), 2, "must return all active rules");
    assert_eq!(rules[0].priority, 5,  "must be ordered by priority asc — first is priority 5");
    assert_eq!(rules[1].priority, 20, "must be ordered by priority asc — second is priority 20");
}

/// get_nudge_dimensions_for_activity returns matching nudge dimensions for a
/// matching activity type, and returns empty for a non-matching type.
#[tokio::test]
async fn test_get_nudge_dimensions_for_activity() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, dim_id) = create_test_template_and_dimension(&db, tenant.id).await;
    let entity_id = Uuid::new_v4();

    // Enable the feature
    {
        use sea_orm::{ActiveModelTrait, Set};
        use crate::entities::tenant_setting::ActiveModel as SettingAM;
        SettingAM {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant.id),
            key: Set("scorecard_display_rules_enabled".to_owned()),
            value: Set("true".to_owned()),
            is_encrypted: Set(false),
            updated_at: Set(chrono::Utc::now()),
            created_at: Set(chrono::Utc::now()),
        }
        .insert(&db)
        .await
        .expect("tenant_setting insert failed");
    }

    // Create scorecard for the entity
    let _ = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", entity_id,
    ).await.unwrap();

    // Rule fires on 'call' or 'demo'
    insert_display_rule(&db, tenant.id, template_id, dim_id, 10, "surface_as_nudge").await;

    // Matching activity type → must return 1 nudge dimension
    let nudges = ScorecardService::get_nudge_dimensions_for_activity(
        &db, tenant.id, template_id,
        "atlas_lead", entity_id,
        "call",   // matches value_list: ["call", "demo"]
    )
    .await
    .expect("get_nudge_dimensions_for_activity must not fail");

    assert_eq!(
        nudges.len(), 1,
        "must return 1 nudge dimension for a matching activity type"
    );
    assert_eq!(nudges[0].dimension_id, dim_id);
    assert_eq!(nudges[0].action, "surface_as_nudge");

    // Non-matching activity type → must return empty
    let nudges_none = ScorecardService::get_nudge_dimensions_for_activity(
        &db, tenant.id, template_id,
        "atlas_lead", entity_id,
        "email",  // NOT in value_list
    )
    .await
    .expect("get_nudge_dimensions_for_activity must not fail for non-matching type");

    assert!(
        nudges_none.is_empty(),
        "must return [] for an activity type not in any rule's value_list"
    );
}

// ── Phase A/B: deployments + auto-deploy ──────────────────────────────────────

async fn create_test_app_instance(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    app_type: &str,
) -> Uuid {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::app_instance::ActiveModel as InstanceAM;
    use chrono::Utc;

    let id = Uuid::new_v4();
    InstanceAM {
        id: Set(id),
        tenant_id: Set(tenant_id),
        app_type: Set(app_type.to_owned()),
        database_url: Set(None),
        data_seed_name: Set(None),
        settings: Set(None),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("app_instance insert failed");
    id
}

async fn insert_deployment(
    db: &sea_orm::DatabaseConnection,
    tenant_id: Uuid,
    template_id: Uuid,
    app_instance_id: Uuid,
    is_enabled: bool,
) {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::atlas_scorecard_template_deployment::ActiveModel as DepAM;
    use chrono::Utc;

    DepAM {
        id: Set(Uuid::new_v4()),
        template_id: Set(template_id),
        app_instance_id: Set(app_instance_id),
        tenant_id: Set(tenant_id),
        is_enabled: Set(is_enabled),
        trigger_event: Set("manual".to_owned()),
        trigger_context_entity_type: Set(None),
        created_at: Set(Utc::now()),
    }
    .insert(db)
    .await
    .expect("deployment insert failed");
}

#[tokio::test]
async fn test_templates_enabled_for_instance_filters_disabled() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, _) = create_test_template_and_dimension(&db, tenant.id).await;
    let instance_id = create_test_app_instance(&db, tenant.id, "property_management").await;

    insert_deployment(&db, tenant.id, template_id, instance_id, false).await;

    let enabled = ScorecardService::templates_enabled_for_instance(
        &db, tenant.id, instance_id, None,
    )
    .await
    .expect("query must succeed");

    assert!(
        enabled.is_empty(),
        "disabled deployment must not appear in enabled list"
    );

    // Enable and re-query
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use crate::entities::atlas_scorecard_template_deployment as deployments;
    let row = deployments::Entity::find()
        .filter(deployments::Column::TemplateId.eq(template_id))
        .filter(deployments::Column::AppInstanceId.eq(instance_id))
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    let mut am: deployments::ActiveModel = row.into();
    am.is_enabled = Set(true);
    am.update(&db).await.unwrap();

    let enabled = ScorecardService::templates_enabled_for_instance(
        &db, tenant.id, instance_id, None,
    )
    .await
    .expect("query must succeed");

    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].id, template_id);
}

#[tokio::test]
async fn test_templates_enabled_for_instance_wrong_tenant_empty() {
    let (_, db) = setup_test_app().await;
    let tenant_a = test_utils::create_test_tenant(&db).await;
    let tenant_b = test_utils::create_test_tenant(&db).await;
    let (template_id, _) = create_test_template_and_dimension(&db, tenant_a.id).await;
    let instance_a = create_test_app_instance(&db, tenant_a.id, "property_management").await;

    insert_deployment(&db, tenant_a.id, template_id, instance_a, true).await;

    // Query with tenant_b's id against tenant_a's instance → empty (tenant filter)
    let enabled = ScorecardService::templates_enabled_for_instance(
        &db, tenant_b.id, instance_a, None,
    )
    .await
    .expect("query must succeed");

    assert!(
        enabled.is_empty(),
        "wrong tenant must not see another tenant's deployments"
    );
}

#[tokio::test]
async fn test_templates_enabled_for_instance_wrong_instance_empty() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, _) = create_test_template_and_dimension(&db, tenant.id).await;
    let instance_a = create_test_app_instance(&db, tenant.id, "property_management").await;
    let instance_b = create_test_app_instance(&db, tenant.id, "property_management").await;

    insert_deployment(&db, tenant.id, template_id, instance_a, true).await;

    let enabled = ScorecardService::templates_enabled_for_instance(
        &db, tenant.id, instance_b, None,
    )
    .await
    .expect("query must succeed");

    assert!(
        enabled.is_empty(),
        "wrong instance must not see deployments for another instance"
    );
}

#[tokio::test]
async fn test_auto_deploy_on_seed_and_deploy_for_folio() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let instance_id = create_test_app_instance(&db, tenant.id, "property_management").await;

    crate::services::pm::scorecard_provisioner::seed_and_deploy_for_folio(&db, tenant.id)
        .await
        .expect("seed_and_deploy must succeed");

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::entities::atlas_scorecard_template_deployment as deployments;

    let deps = deployments::Entity::find()
        .filter(deployments::Column::TenantId.eq(tenant.id))
        .filter(deployments::Column::AppInstanceId.eq(instance_id))
        .filter(deployments::Column::IsEnabled.eq(true))
        .all(&db)
        .await
        .expect("list deployments");

    assert_eq!(
        deps.len(), 4,
        "auto-deploy must create 4 enabled deployments for seeded PM templates"
    );

    // Idempotent: second call must not duplicate
    crate::services::pm::scorecard_provisioner::seed_and_deploy_for_folio(&db, tenant.id)
        .await
        .expect("second seed_and_deploy must succeed");

    let deps2 = deployments::Entity::find()
        .filter(deployments::Column::TenantId.eq(tenant.id))
        .filter(deployments::Column::AppInstanceId.eq(instance_id))
        .all(&db)
        .await
        .expect("list deployments after re-seed");

    assert_eq!(deps2.len(), 4, "re-seed must remain idempotent");

    let enabled = ScorecardService::templates_enabled_for_instance(
        &db, tenant.id, instance_id, None,
    )
    .await
    .expect("list enabled");
    assert_eq!(enabled.len(), 4);
}

#[tokio::test]
async fn test_post_checkout_trigger_opens_session_and_submit_updates_aggregates() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let instance_id = create_test_app_instance(&db, tenant.id, "property_management").await;

    // Seed STR template + deploy with post_checkout (via provisioner defaults)
    crate::services::pm::scorecard_provisioner::seed_and_deploy_for_folio(&db, tenant.id)
        .await
        .expect("seed_and_deploy");

    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
    use crate::entities::atlas_scorecard_template as templates;
    use crate::entities::atlas_scorecard_dimension::ActiveModel as DimAM;
    use rust_decimal::Decimal;

    let str_template = templates::Entity::find()
        .filter(templates::Column::TenantId.eq(tenant.id))
        .filter(templates::Column::Name.eq("STR Property Assessment"))
        .one(&db)
        .await
        .unwrap()
        .expect("STR template");

    // Add a dimension so submit_entry + recompute have work to do
    let dim_id = Uuid::new_v4();
    DimAM {
        id: Set(dim_id),
        template_id: Set(str_template.id),
        tenant_id: Set(tenant.id),
        slug: Set("cleanliness".to_owned()),
        name: Set("Cleanliness".to_owned()),
        description: Set(None),
        category: Set(None),
        scale_type: Set("rating".to_owned()),
        scale_min: Set(Decimal::ZERO),
        scale_max: Set(Decimal::from(10)),
        weight: Set(Decimal::from(1)),
        unit_label: Set(None),
        benchmark_tiers: Set(json!([])),
        global_reference_value: Set(None),
        global_reference_label: Set(None),
        min_entries_to_show: Set(1),
        is_community_ratable: Set(true),
        is_active: Set(true),
        sort_order: Set(0),
        ..Default::default()
    }
    .insert(&db)
    .await
    .expect("dim insert");

    let asset_id = Uuid::new_v4();
    let reservation_id = Uuid::new_v4();
    let rater_id = Uuid::new_v4();

    let opened = crate::services::scorecard_triggers::on_str_checkout(
        &db,
        tenant.id,
        instance_id,
        reservation_id,
        asset_id,
        rater_id,
    )
    .await
    .expect("on_str_checkout");

    assert_eq!(
        opened.len(), 1,
        "STR Property Assessment deployment should open one post_checkout session"
    );
    assert_eq!(opened[0].template_id, str_template.id);

    use crate::entities::atlas_rating_session as sessions;
    let session = sessions::Entity::find_by_id(opened[0].session_id)
        .one(&db)
        .await
        .unwrap()
        .expect("session row");
    assert_eq!(session.context_entity_type.as_deref(), Some("atlas_reservation"));
    assert_eq!(session.context_entity_id, Some(reservation_id));
    assert_eq!(session.session_type, "stay");
    assert_eq!(session.app_instance_id, Some(instance_id));

    // Submit entry (manual is auto-verified) + recompute → aggregates update
    ScorecardService::submit_entry(
        &db,
        opened[0].session_id,
        opened[0].scorecard_id,
        dim_id,
        tenant.id,
        rater_id,
        Some(8.0),
        None,
        "manual",
        None,
        None,
    )
    .await
    .expect("submit_entry");

    ScorecardService::recompute_aggregates(&db, opened[0].scorecard_id)
        .await
        .expect("recompute");

    use crate::entities::atlas_scorecard as scorecards;
    use crate::entities::atlas_scorecard_entry as entries;
    let entry = entries::Entity::find()
        .filter(entries::Column::SessionId.eq(opened[0].session_id))
        .one(&db)
        .await
        .unwrap()
        .expect("entry");
    assert!(entry.is_verified, "manual entries must be auto-verified");

    let sc = scorecards::Entity::find_by_id(opened[0].scorecard_id)
        .one(&db)
        .await
        .unwrap()
        .expect("scorecard");
    assert!(
        sc.total_entries >= 1,
        "aggregates should reflect verified entry (entries={} sessions={})",
        sc.total_entries,
        sc.total_sessions
    );
}

#[tokio::test]
async fn test_case_resolved_trigger_opens_contractor_session() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let instance_id = create_test_app_instance(&db, tenant.id, "property_management").await;

    crate::services::pm::scorecard_provisioner::seed_and_deploy_for_folio(&db, tenant.id)
        .await
        .expect("seed_and_deploy");

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::entities::atlas_scorecard_template as templates;
    use crate::entities::atlas_rating_session as sessions;

    let contractor_template = templates::Entity::find()
        .filter(templates::Column::TenantId.eq(tenant.id))
        .filter(templates::Column::Name.eq("Contractor Performance"))
        .one(&db)
        .await
        .unwrap()
        .expect("Contractor Performance template");

    let case_id = Uuid::new_v4();
    let provider_id = Uuid::new_v4();
    let rater_id = Uuid::new_v4();

    let opened = crate::services::scorecard_triggers::on_case_resolved(
        &db,
        tenant.id,
        instance_id,
        case_id,
        provider_id,
        rater_id,
    )
    .await
    .expect("on_case_resolved");

    assert_eq!(
        opened.len(),
        1,
        "Contractor Performance deployment should open one case_resolved session"
    );
    assert_eq!(opened[0].template_id, contractor_template.id);

    let session = sessions::Entity::find_by_id(opened[0].session_id)
        .one(&db)
        .await
        .unwrap()
        .expect("session row");
    assert_eq!(session.context_entity_type.as_deref(), Some("atlas_case"));
    assert_eq!(session.context_entity_id, Some(case_id));
    assert_eq!(session.session_type, "job");
    assert_eq!(session.app_instance_id, Some(instance_id));
    assert_eq!(session.rater_user_id, rater_id);

    use crate::entities::atlas_scorecard as scorecards;
    let sc = scorecards::Entity::find_by_id(opened[0].scorecard_id)
        .one(&db)
        .await
        .unwrap()
        .expect("scorecard");
    assert_eq!(sc.subject_entity_type, "atlas_service_provider");
    assert_eq!(sc.subject_entity_id, provider_id);
}

#[tokio::test]
async fn test_open_session_allows_null_app_instance_id() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let (template_id, _) = create_test_template_and_dimension(&db, tenant.id).await;
    let rater_id = Uuid::new_v4();

    let scorecard_id = ScorecardService::get_or_create(
        &db, tenant.id, template_id, "atlas_lead", Uuid::new_v4(),
    )
    .await
    .unwrap();

    let session_id = ScorecardService::open_session(
        &db,
        scorecard_id,
        rater_id,
        tenant.id,
        chrono::Utc::now(),
        "call",
        None,
        None,
        None,
        None, // nullable for admin / legacy paths
    )
    .await
    .expect("open_session with null app_instance_id");

    use sea_orm::EntityTrait;
    use crate::entities::atlas_rating_session as sessions;
    let session = sessions::Entity::find_by_id(session_id)
        .one(&db)
        .await
        .unwrap()
        .expect("session");
    assert!(session.app_instance_id.is_none());
}

#[tokio::test]
async fn test_trigger_enqueues_evaluate_scorecard_nudge_job() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;
    let instance_id = create_test_app_instance(&db, tenant.id, "property_management").await;

    crate::services::pm::scorecard_provisioner::seed_and_deploy_for_folio(&db, tenant.id)
        .await
        .expect("seed_and_deploy");

    let opened = crate::services::scorecard_triggers::on_str_checkout(
        &db,
        tenant.id,
        instance_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    )
    .await
    .expect("on_str_checkout");
    assert!(!opened.is_empty());

    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
    use crate::entities::outbox_job;
    use crate::types::outbox::OutboxJobType;

    let jobs = outbox_job::Entity::find()
        .filter(outbox_job::Column::TenantId.eq(tenant.id))
        .filter(
            outbox_job::Column::JobType
                .eq(OutboxJobType::EvaluateScorecardNudge.to_string()),
        )
        .all(&db)
        .await
        .expect("outbox query");

    assert!(
        !jobs.is_empty(),
        "trigger must enqueue evaluate_scorecard_nudge"
    );
    let payload = &jobs[0].payload;
    let expected_session = opened[0].session_id.to_string();
    let expected_scorecard = opened[0].scorecard_id.to_string();
    assert_eq!(
        payload.get("session_id").and_then(|v| v.as_str()),
        Some(expected_session.as_str())
    );
    assert_eq!(
        payload.get("scorecard_id").and_then(|v| v.as_str()),
        Some(expected_scorecard.as_str())
    );
    assert_eq!(
        payload.get("activity_type").and_then(|v| v.as_str()),
        Some("post_checkout")
    );
}
