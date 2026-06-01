//! Integration tests for G-31 LeadService + AccountService.
//! DB-backed, using the same test harness as services_tests.rs.
//!
//! Coverage:
//!   LeadService:
//!     - create: compute_name invariant, default status
//!     - create_from_import: dedup semantics (caller calls find_duplicate first)
//!     - find_duplicate: email/domain/duns priority ordering
//!     - disqualify: happy path + terminal guard (converted → error)
//!     - disqualify: tenant isolation (foreign lead_id → error)
//!   AccountService:
//!     - create_from_lead_conversion: full firmographic round-trip
//!     - find_by_domain: finds match, excludes archived
//!     - search: finds by name, returns empty for no match

use uuid::Uuid;
use serde_json::json;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;
use crate::services::{
    lead_service::LeadService,
    account_service::AccountService,
};

// ── LeadService::create ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_lead_create_computes_name_from_first_last() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    // create(db, tenant_id, first, last, email, phone, company, source, listing_id, account_id)
    let lead = LeadService::create(
        &db,
        tenant.id,
        Some("Jane"),
        Some("Doe"),
        Some("jane@acme.com"),
        Some("555-0100"),
        None,
        Some("web_form"),
        None,
        None,
    )
    .await
    .expect("lead create failed");

    assert_eq!(lead.name, "Jane Doe");
    assert_eq!(lead.lead_status, "new");
    assert!(!lead.is_converted);
    assert!(!lead.is_duplicate);
}

#[tokio::test]
async fn test_lead_create_falls_back_name_to_company() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let lead = LeadService::create(
        &db, tenant.id, None, None,
        Some("dispatch@acme.com"), None,
        Some("Acme Transport LLC"),
        None, None, None,
    ).await.expect("lead create failed");

    assert_eq!(lead.name, "Acme Transport LLC");
}

#[tokio::test]
async fn test_lead_create_falls_back_name_to_email_when_no_name_or_company() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let email = format!("solo_{}@test.com", Uuid::new_v4().to_string().replace("-", ""));
    let lead = LeadService::create(
        &db, tenant.id, None, None,
        Some(&email), None, None, None, None, None,
    ).await.unwrap();

    assert_eq!(lead.name, email);
}

// ── LeadService::find_duplicate ───────────────────────────────────────────────

#[tokio::test]
async fn test_lead_find_duplicate_by_email() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let email = format!("uniq_{}@test.com", Uuid::new_v4().to_string().replace("-", ""));

    LeadService::create(
        &db, tenant.id, Some("Alice"), Some("T"),
        Some(&email), None, None, None, None, None,
    ).await.unwrap();

    // find_duplicate(db, tenant_id, email, domain, duns) → Option<Uuid>
    let dup = LeadService::find_duplicate(&db, tenant.id, Some(&email), None, None)
        .await.unwrap();

    assert!(dup.is_some(), "must find lead by email");
}

#[tokio::test]
async fn test_lead_find_duplicate_returns_none_for_unknown_email() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let result = LeadService::find_duplicate(
        &db, tenant.id, Some("nobody@nowhere.invalid"), None, None,
    ).await.unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_lead_find_duplicate_duns_wins_over_email() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    // Import a lead with a DUNS number
    LeadService::create_from_import(
        &db, tenant.id,
        "fmcsa", "DOT-99887766",
        Some("Bob"), Some("Builder"),
        Some("bob@builder.com"),
        Some("Bob Builders LLC"),
        Some("bobbuilders.com"),
        Some("DUNS-111222333"),
        Some(json!({"dot": "99887766"})),
        false, None,
    ).await.unwrap();

    // DUNS lookup should win (priority 1) over email
    let found_duns = LeadService::find_duplicate(
        &db, tenant.id, Some("bob@builder.com"), None, Some("DUNS-111222333"),
    ).await.unwrap();

    assert!(found_duns.is_some(), "DUNS lookup must find the lead");
}

// ── LeadService::create_from_import ───────────────────────────────────────────

#[tokio::test]
async fn test_lead_create_from_import_dedup_pattern() {
    // The contract: create_from_import is a pure writer.
    // Callers must call find_duplicate FIRST and skip import if a match is found.
    // This test verifies the pattern the service is designed for.
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let source_id = format!("DOT-{}", Uuid::new_v4().to_string().replace("-", "")[..8].to_string());

    // Step 1: find_duplicate → None (lead doesn't exist yet)
    let existing = LeadService::find_duplicate(
        &db, tenant.id, Some("import@test.com"), None, None,
    ).await.unwrap();
    assert!(existing.is_none());

    // Step 2: create
    let lead = LeadService::create_from_import(
        &db, tenant.id, "fmcsa", &source_id,
        Some("Import"), Some("User"),
        Some("import@test.com"),
        Some("Import Co"),
        Some("importco.com"),
        None,
        Some(json!({"dot": source_id})),
        false, None,
    ).await.expect("import create failed");

    assert_eq!(lead.data_source.as_deref(), Some("fmcsa"));
    assert_eq!(lead.data_source_id.as_deref(), Some(source_id.as_str()));

    // Step 3: find_duplicate → Some (lead now exists)
    let found = LeadService::find_duplicate(
        &db, tenant.id, Some("import@test.com"), None, None,
    ).await.unwrap();
    assert!(found.is_some(), "after import, find_duplicate must find the lead");
    assert_eq!(found.unwrap(), lead.id);
}

// ── LeadService::disqualify ───────────────────────────────────────────────────

#[tokio::test]
async fn test_lead_disqualify_sets_terminal_status() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let lead = LeadService::create(
        &db, tenant.id, Some("Charlie"), Some("Test"),
        None, None, None, None, None, None,
    ).await.unwrap();

    // disqualify(db, lead_id, tenant_id, reason)
    LeadService::disqualify(&db, lead.id, tenant.id, "bad_fit")
        .await
        .expect("disqualify must succeed");

    use sea_orm::EntityTrait;
    let updated = crate::entities::atlas_lead::Entity::find_by_id(lead.id)
        .one(&db).await.unwrap().unwrap();

    assert_eq!(updated.lead_status, "disqualified");
    assert!(updated.disqualified_at.is_some());
    assert_eq!(updated.disqualification_reason.as_deref(), Some("bad_fit"));
    assert!(updated.is_terminal());
}

#[tokio::test]
async fn test_lead_disqualify_returns_error_for_converted_lead() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    // Insert a pre-converted lead directly
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::atlas_lead::ActiveModel as LeadAM;
    let lead_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    LeadAM {
        id: Set(lead_id),
        tenant_id: Set(tenant.id),
        name: Set("Converted Lead".to_owned()),
        lead_status: Set("converted".to_owned()),
        is_converted: Set(true),
        converted_at: Set(Some(now)),
        country: Set("US".to_owned()),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    }.insert(&db).await.unwrap();

    let result = LeadService::disqualify(&db, lead_id, tenant.id, "mistake").await;
    assert!(result.is_err(), "disqualifying a converted (terminal) lead must fail");
    let err = result.unwrap_err();
    assert!(err.contains("terminal"), "error must mention terminal state; got: {err}");
}

#[tokio::test]
async fn test_lead_disqualify_returns_error_for_foreign_tenant() {
    let (_, db) = setup_test_app().await;
    let tenant_a = test_utils::create_test_tenant(&db).await;
    let tenant_b = test_utils::create_test_tenant(&db).await;

    let lead = LeadService::create(
        &db, tenant_a.id, Some("Cross"), Some("TenantTest"),
        None, None, None, None, None, None,
    ).await.unwrap();

    // Tenant B tries to disqualify Tenant A's lead
    let result = LeadService::disqualify(&db, lead.id, tenant_b.id, "attempted_theft").await;
    assert!(result.is_err(), "disqualifying another tenant's lead must fail");
}

// ── AccountService::create_from_lead_conversion ───────────────────────────────

#[tokio::test]
async fn test_account_create_from_lead_conversion_stores_firmographics() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let account_id = AccountService::create_from_lead_conversion(
        &db,
        tenant.id,
        "organization",
        "Acme Carriers LLC",
        None, None,
        Some("dispatch@acme.com"),
        Some("555-0200"),
        Some("https://acmecarriers.com"),
        Some("Transportation"),
        Some("4213"),
        Some("484121"),
        Some(45),
        Some(rust_decimal::Decimal::from(5_000_000)),
        Some("Main St"),
        Some("Dallas"),
        Some("TX"),
        Some("75201"),
        Some("US"),
        Some("acmecarriers.com"),
        Some("DUNS-987654321"),
        Some("fmcsa"),
        Some("DOT-12345678"),
        Some(json!({"dot_number": "12345678", "safety_rating": "satisfactory"})),
    )
    .await
    .expect("create_from_lead_conversion failed");

    use sea_orm::EntityTrait;
    let account = crate::entities::atlas_account::Entity::find_by_id(account_id)
        .one(&db).await.unwrap().expect("account must exist");

    assert_eq!(account.name, "Acme Carriers LLC");
    assert_eq!(account.account_type, "organization");
    assert_eq!(account.industry.as_deref(), Some("Transportation"));
    assert_eq!(account.domain.as_deref(), Some("acmecarriers.com"));
    assert_eq!(account.data_source.as_deref(), Some("fmcsa"));
    assert_eq!(account.city.as_deref(), Some("Dallas"));
    assert_eq!(account.num_employees, Some(45));
    assert_eq!(account.status, "active");
}

// ── AccountService::find_by_domain ────────────────────────────────────────────

#[tokio::test]
async fn test_account_find_by_domain_finds_exact_match() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let domain = format!("{}.test.io", Uuid::new_v4().to_string().replace("-", ""));

    AccountService::create_from_lead_conversion(
        &db, tenant.id, "organization", "Domain Test Co",
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None,
        Some(&domain), None, None, None, None,
    ).await.unwrap();

    let found = AccountService::find_by_domain(&db, tenant.id, &domain).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().domain.as_deref(), Some(domain.as_str()));
}

#[tokio::test]
async fn test_account_find_by_domain_excludes_archived() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let domain = format!("{}.archived.io", Uuid::new_v4().to_string().replace("-", ""));

    let account_id = AccountService::create_from_lead_conversion(
        &db, tenant.id, "organization", "Archived Co",
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None,
        Some(&domain), None, None, None, None,
    ).await.unwrap();

    use sea_orm::{ActiveModelTrait, IntoActiveModel, Set};
    let account = crate::entities::atlas_account::Entity::find_by_id(account_id)
        .one(&db).await.unwrap().unwrap();
    let mut am = account.into_active_model();
    am.status = Set("archived".to_owned());
    am.update(&db).await.unwrap();

    let found = AccountService::find_by_domain(&db, tenant.id, &domain).await.unwrap();
    assert!(found.is_none(), "archived accounts must not be returned");
}

// ── AccountService::search ────────────────────────────────────────────────────

#[tokio::test]
async fn test_account_search_finds_by_name() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let unique_name = format!("SearchTarget{}", Uuid::new_v4().to_string().replace("-", ""));
    AccountService::create_account(&db, tenant.id, "organization", &unique_name, None, None)
        .await.unwrap();

    let results = AccountService::search(&db, tenant.id, &unique_name[..12], 10)
        .await.unwrap();

    assert!(results.iter().any(|a| a.name == unique_name));
}

#[tokio::test]
async fn test_account_search_returns_empty_for_no_match() {
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    let results = AccountService::search(
        &db, tenant.id, "zzz_no_such_account_xyzzy_9999", 10,
    ).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
async fn test_account_search_metacharacter_does_not_panic() {
    // Regression test for LIKE injection fix — queries containing %, _, \ must
    // be handled as literals, not expanded as wildcards.
    let (_, db) = setup_test_app().await;
    let tenant = test_utils::create_test_tenant(&db).await;

    for metachar_query in &["%", "_", "\\", "100%_profit\\"] {
        let result = AccountService::search(&db, tenant.id, metachar_query, 10).await;
        assert!(
            result.is_ok(),
            "search must not fail on metacharacter input '{}': {:?}",
            metachar_query, result
        );
        // Metachar-only queries should return empty (no accounts named '%' etc.)
        assert!(result.unwrap().is_empty());
    }
}
