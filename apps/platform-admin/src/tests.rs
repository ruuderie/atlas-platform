#![cfg(test)]
use chrono::{TimeZone, Utc};
use serde_json::json;
use wasm_bindgen_test::*;

use crate::pages::logs::{format_datetime_diff, format_json_diff};

wasm_bindgen_test_configure!(run_in_browser);

// ── Existing: datetime + JSON diff ────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_format_datetime_diff_wasm() {
    let dt = Utc.with_ymd_and_hms(2026, 4, 15, 12, 30, 45).unwrap();
    assert_eq!(format_datetime_diff(dt), "2026-04-15 12:30:45");
}

#[wasm_bindgen_test]
fn test_format_json_diff_wasm_with_data() {
    let val = Some(json!({
        "foo": "bar",
        "nested": { "val": 42 }
    }));
    let formatted = format_json_diff(&val);
    assert!(formatted.contains("\"foo\": \"bar\""));
    assert!(formatted.contains("\"val\": 42"));
}

#[wasm_bindgen_test]
fn test_format_json_diff_wasm_none() {
    assert_eq!(format_json_diff(&None), "None");
}

// ── Existing: group_apps_by_tenant ────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_group_apps_by_tenant_logic() {
    use crate::api::models::PlatformAppModel;
    use crate::utils::group_apps_by_tenant;

    let apps = vec![
        PlatformAppModel {
            tenant_id: "T1".to_string(),
            instance_id: "I1".to_string(),
            name: "CT Build Pros".to_string(),
            app_type: "Network".to_string(),
            domain: "ct-build-pros.oply.co".to_string(),
            site_status: "Active".to_string(),
            description: "".to_string(),
        },
        PlatformAppModel {
            tenant_id: "T1".to_string(),
            instance_id: "I2".to_string(),
            name: "CT Build Pros".to_string(),
            app_type: "Anchor".to_string(),
            domain: "directory.localhost".to_string(),
            site_status: "Active".to_string(),
            description: "".to_string(),
        },
        PlatformAppModel {
            tenant_id: "T2".to_string(),
            instance_id: "I3".to_string(),
            name: "BuildWithRuud".to_string(),
            app_type: "Anchor".to_string(),
            domain: "uat.buildwithruud.com".to_string(),
            site_status: "Active".to_string(),
            description: "".to_string(),
        },
    ];

    let grouped = group_apps_by_tenant(apps);
    assert_eq!(
        grouped.len(),
        2,
        "Should have exactly 2 distinct Tenant keys"
    );
    let t1_group = grouped.get("T1").unwrap();
    assert_eq!(t1_group.0, "CT Build Pros");
    assert_eq!(
        t1_group.1.len(),
        2,
        "CT Build Pros should contain exactly 2 FQDN apps"
    );
    let t2_group = grouped.get("T2").unwrap();
    assert_eq!(t2_group.0, "BuildWithRuud");
    assert_eq!(t2_group.1.len(), 1, "BuildWithRuud should contain 1 app");
}

// ── Billing plan model ─────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_billing_plan_model_fields() {
    use crate::api::models::BillingPlanModel;

    let plan = BillingPlanModel {
        id: "plan-001".to_string(),
        name: "Starter".to_string(),
        price: 4900,
        currency: "USD".to_string(),
        interval: "month".to_string(),
        created_at: Some("2026-01-01T00:00:00Z".to_string()),
    };

    assert_eq!(plan.id, "plan-001");
    assert_eq!(plan.name, "Starter");
    assert_eq!(plan.price, 4900, "price should be stored as integer cents");
    assert_eq!(plan.currency, "USD");
    assert_eq!(plan.interval, "month");
    assert!(plan.created_at.is_some());
}

#[wasm_bindgen_test]
fn test_billing_plan_price_to_dollars() {
    // Mirrors the fmt_mrr / price display logic in clients and billing pages.
    let starter_cents: i64 = 4900;
    assert_eq!(starter_cents / 100, 49, "4900 cents = $49");

    let free_cents: i64 = 0;
    assert_eq!(free_cents / 100, 0);

    let enterprise_cents: i64 = 99900;
    assert_eq!(enterprise_cents / 100, 999, "$999 enterprise tier");

    // Annual pricing: 12 months
    let annual_cents: i64 = 4900 * 12;
    assert_eq!(annual_cents / 100, 588, "Annual starter: $588/yr");
}

// ── MRR aggregation ───────────────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_mrr_sum_from_tenant_stats() {
    use crate::api::models::TenantStatModel;

    let make = |id: &str, name: &str, mrr: i64, status: &str| TenantStatModel {
        tenant_id: id.to_string(),
        name: name.to_string(),
        slug: name.to_lowercase().replace(' ', "-"),
        profile_count: 100,
        listing_count: 40,
        ad_purchase_count: 2,
        plan: Some("growth".to_string()),
        mrr_cents: Some(mrr),
        site_status: Some(status.to_string()),
        joined_at: Some("2025-11-01".to_string()),
        anchor_instance_id: Some(format!("inst-{}", id)),
    };

    let tenants = vec![
        make("T1", "Acme Corp", 7900, "active"),
        make("T2", "BuildWithRuud", 29900, "active"),
        make("T3", "Suspended Client", 0, "suspended"),
    ];

    let total_mrr: i64 = tenants.iter().filter_map(|t| t.mrr_cents).sum();
    assert_eq!(total_mrr, 37800, "Total MRR = T1 + T2 + T3 = 37800 cents");

    // Active-only MRR (mirrors billing dashboard filter)
    let active_mrr: i64 = tenants
        .iter()
        .filter(|t| t.site_status.as_deref() == Some("active"))
        .filter_map(|t| t.mrr_cents)
        .sum();
    assert_eq!(active_mrr, 37800, "Suspended has 0 MRR so total unchanged");

    // Format total as $k (mirrors billing dashboard KPI display)
    let mrr_k = total_mrr as f64 / 100_000.0; // cents → thousands of dollars
    assert!((mrr_k - 0.378).abs() < 0.001, "MRR = $0.378k");
}

// ── CRM-style tenant filtering ─────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_tenant_filter_by_status() {
    use crate::api::models::TenantStatModel;

    let make = |id: &str, name: &str, status: &str| TenantStatModel {
        tenant_id: id.to_string(),
        name: name.to_string(),
        slug: name.to_lowercase().replace(' ', "-"),
        profile_count: 10,
        listing_count: 5,
        ad_purchase_count: 0,
        plan: Some("starter".to_string()),
        mrr_cents: Some(4900),
        site_status: Some(status.to_string()),
        joined_at: None,
        anchor_instance_id: None,
    };

    let tenants = vec![
        make("T1", "Alpha Inc", "active"),
        make("T2", "Beta LLC", "suspended"),
        make("T3", "Gamma Co", "active"),
        make("T4", "Delta Ltd", "provisioning"),
    ];

    // Status filter
    let active: Vec<_> = tenants
        .iter()
        .filter(|t| t.site_status.as_deref() == Some("active"))
        .collect();
    assert_eq!(active.len(), 2, "2 active tenants");

    let suspended: Vec<_> = tenants
        .iter()
        .filter(|t| t.site_status.as_deref() == Some("suspended"))
        .collect();
    assert_eq!(suspended.len(), 1, "1 suspended tenant");

    // Name search (mirrors clients page search)
    let q = "alpha";
    let matched: Vec<_> = tenants
        .iter()
        .filter(|t| t.name.to_lowercase().contains(q))
        .collect();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].name, "Alpha Inc");

    // Plan filter
    let growth: Vec<_> = tenants
        .iter()
        .filter(|t| t.plan.as_deref() == Some("starter"))
        .collect();
    assert_eq!(
        growth.len(),
        4,
        "All 4 tenants on starter plan in this fixture"
    );
}

// ── PlatformAppSummary field coverage ─────────────────────────────────────────

#[wasm_bindgen_test]
fn test_platform_app_summary_all_fields() {
    use crate::api::models::PlatformAppSummary;

    // Standard client deployment
    let app = PlatformAppSummary {
        tenant_id: "T1".to_string(),
        instance_id: "inst-001".to_string(),
        name: "Folio App".to_string(),
        app_type: "folio".to_string(),
        domain: "folio.example.com".to_string(),
        mode: "standard".to_string(),
        site_status: "active".to_string(),
        description: "Residential PM platform".to_string(),
        platform_account_id: Some("acct-xyz".to_string()),
        purpose: None,
    };
    assert_eq!(app.mode, "standard");
    assert!(
        app.platform_account_id.is_some(),
        "CRM account cross-linked"
    );
    assert!(
        app.purpose.is_none(),
        "Not an internal instance — no purpose field"
    );

    // Internal operator demo instance
    let internal = PlatformAppSummary {
        tenant_id: "T0".to_string(),
        instance_id: "inst-demo".to_string(),
        name: "Demo Env".to_string(),
        app_type: "folio".to_string(),
        domain: "demo.atlas-platform.com".to_string(),
        mode: "internal_operator".to_string(),
        site_status: "active".to_string(),
        description: "Internal demo instance".to_string(),
        platform_account_id: None,
        purpose: Some("demo".to_string()),
    };
    assert_eq!(internal.mode, "internal_operator");
    assert_eq!(internal.purpose.as_deref(), Some("demo"));
    assert!(
        internal.platform_account_id.is_none(),
        "Internal instances not CRM-linked"
    );
}

// ── TenantStatModel slug field ─────────────────────────────────────────────────

#[wasm_bindgen_test]
fn test_tenant_stat_slug_field() {
    use crate::api::models::TenantStatModel;

    let t = TenantStatModel {
        tenant_id: "T1".to_string(),
        name: "Build With Ruud".to_string(),
        slug: "buildwithruud".to_string(),
        profile_count: 200,
        listing_count: 80,
        ad_purchase_count: 5,
        plan: Some("enterprise".to_string()),
        mrr_cents: Some(29900),
        site_status: Some("active".to_string()),
        joined_at: Some("2024-06-01".to_string()),
        anchor_instance_id: Some("inst-bwr".to_string()),
    };

    assert_eq!(
        t.slug, "buildwithruud",
        "slug must be the human-readable key shown in flags UI"
    );
    // The flags page uses slug (not tenant_id UUID) as the display + filter key
    assert_ne!(t.slug, t.tenant_id, "slug must differ from UUID tenant_id");
}
