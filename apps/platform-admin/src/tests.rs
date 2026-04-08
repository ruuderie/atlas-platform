#![cfg(test)]
use wasm_bindgen_test::*;
use chrono::{TimeZone, Utc};
use serde_json::json;

use crate::pages::audit_logs::{format_datetime_diff, format_json_diff};
// Depending on module structure, we might test svg logic if it was extracted, 
// but it currently has unit tests in `svg_charts.rs` directly.

wasm_bindgen_test_configure!(run_in_browser);

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
    
    assert_eq!(grouped.len(), 2, "Should have exactly 2 distinct Tenant keys");
    
    let t1_group = grouped.get("T1").unwrap();
    assert_eq!(t1_group.0, "CT Build Pros");
    assert_eq!(t1_group.1.len(), 2, "CT Build Pros should contain exactly 2 FQDN apps");
    
    let t2_group = grouped.get("T2").unwrap();
    assert_eq!(t2_group.0, "BuildWithRuud");
    assert_eq!(t2_group.1.len(), 1, "BuildWithRuud should contain 1 app");
}
