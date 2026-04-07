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
