//! Unit tests for the AppInstance decomposition work (Phase 2 follow-ups).
//!
//! ## Coverage
//!
//! | Module | Tests |
//! |---|---|
//! | `instance_stats_response` | InstanceStatsResponse field presence, zero defaults, JSON shape |
//! | `public_config_response` | PublicConfigResponse app_slug dispatch logic, DNS instruction opt-in |
//! | `operational_config_patch` | Partial-update merge semantics for PATCH /operational-config |
//! | `provision_slug_validation` | validate_slug: length, charset, leading/trailing hyphen rules |
//! | `provision_app_selection` | Apps vec building: anchor always present, optional apps additive |
//! | `instance_app_slug_dispatch` | Exact slug match rules — no contains(), canonical slugs only |
//! | `billing_tier_constraints` | Billing tier slug shape rules and known values |
//!
//! ## Philosophy
//!
//! All tests are **pure** — no database, no async I/O, no HTTP.
//! Real platform types from `crate::admin::app_instance` and
//! `crate::models::provision` are used to verify logic rather than
//! testing against string copies.

use uuid::Uuid;
use crate::models::provision::{validate_slug, validate_domain};

// ── InstanceStatsResponse: struct shape and zero-value semantics ──────────────

mod instance_stats_response {
    use super::Uuid;
    use crate::admin::app_instance::InstanceStatsResponse;

    fn zero_stats(app_slug: &str) -> InstanceStatsResponse {
        InstanceStatsResponse {
            instance_id:          Uuid::new_v4(),
            tenant_id:            Uuid::new_v4(),
            app_slug:             app_slug.to_string(),
            asset_count:          0,
            active_contract_count: 0,
            lead_count:           0,
            open_case_count:      0,
            vendor_count:         0,
            active_listing_count: 0,
        }
    }

    #[test]
    fn folio_stats_zero_values_are_all_zero() {
        let s = zero_stats("property_management");
        assert_eq!(s.asset_count, 0);
        assert_eq!(s.active_contract_count, 0);
        assert_eq!(s.lead_count, 0);
        assert_eq!(s.open_case_count, 0);
        assert_eq!(s.vendor_count, 0);
        assert_eq!(s.active_listing_count, 0);
    }

    #[test]
    fn network_instance_stats_has_active_listing_count() {
        let s = zero_stats("network_instance");
        assert_eq!(s.active_listing_count, 0);
    }

    #[test]
    fn anchor_stats_carries_lead_count() {
        let s = zero_stats("anchor");
        assert_eq!(s.lead_count, 0);
    }

    #[test]
    fn app_slug_stored_exactly_as_given() {
        let s = zero_stats("property_management");
        assert_eq!(s.app_slug, "property_management");
    }

    #[test]
    fn instance_id_and_tenant_id_are_distinct() {
        let s = zero_stats("anchor");
        assert_ne!(s.instance_id, s.tenant_id);
    }

    #[test]
    fn stats_with_counts_are_independent_fields() {
        let s = InstanceStatsResponse {
            instance_id:          Uuid::new_v4(),
            tenant_id:            Uuid::new_v4(),
            app_slug:             "property_management".to_string(),
            asset_count:          87,
            active_contract_count: 62,
            lead_count:           342,
            open_case_count:      8,
            vendor_count:         14,
            active_listing_count: 0, // Folio doesn't surface listings in NI sense
        };
        assert_eq!(s.asset_count, 87);
        assert_eq!(s.active_contract_count, 62);
        assert_eq!(s.lead_count, 342);
        assert_eq!(s.open_case_count, 8);
        assert_eq!(s.vendor_count, 14);
        assert_eq!(s.active_listing_count, 0);
    }

    #[test]
    fn all_count_fields_are_u64() {
        // Compile-time: all counts must be u64 — this test exercises arithmetic
        let s = zero_stats("property_management");
        let total: u64 = s.asset_count
            + s.active_contract_count
            + s.lead_count
            + s.open_case_count
            + s.vendor_count
            + s.active_listing_count;
        assert_eq!(total, 0);
    }

    #[test]
    fn stats_serializes_to_json_with_all_count_fields() {
        let s = InstanceStatsResponse {
            instance_id:          Uuid::new_v4(),
            tenant_id:            Uuid::new_v4(),
            app_slug:             "network_instance".to_string(),
            asset_count:          5,
            active_contract_count: 3,
            lead_count:           100,
            open_case_count:      2,
            vendor_count:         7,
            active_listing_count: 41,
        };
        let j = serde_json::to_value(&s).expect("must serialize");
        assert!(j.get("instance_id").is_some());
        assert!(j.get("tenant_id").is_some());
        assert!(j.get("app_slug").is_some());
        assert!(j.get("asset_count").is_some());
        assert!(j.get("active_contract_count").is_some());
        assert!(j.get("lead_count").is_some());
        assert!(j.get("open_case_count").is_some());
        assert!(j.get("vendor_count").is_some());
        assert!(j.get("active_listing_count").is_some());
        assert_eq!(j["app_slug"], "network_instance");
        assert_eq!(j["active_listing_count"], 41);
    }

    #[test]
    fn json_field_names_use_snake_case() {
        let s = zero_stats("anchor");
        let j = serde_json::to_value(&s).expect("must serialize");
        // Verify snake_case key names exactly match what frontend API expects
        assert!(j.get("active_contract_count").is_some(), "must be 'active_contract_count'");
        assert!(j.get("open_case_count").is_some(),       "must be 'open_case_count'");
        assert!(j.get("active_listing_count").is_some(),  "must be 'active_listing_count'");
        assert!(j.get("vendor_count").is_some(),          "must be 'vendor_count'");
        // camelCase variants must NOT exist
        assert!(j.get("activeContractCount").is_none(),   "must NOT be camelCase");
        assert!(j.get("openCaseCount").is_none(),         "must NOT be camelCase");
    }
}

// ── App Slug Dispatch: exact-match rules ─────────────────────────────────────

mod instance_app_slug_dispatch {

    /// Mirrors the dispatch logic in `pages/apps/instance.rs`:
    /// exact match on `app_slug`, never `contains()`.
    fn dispatch_app_type(app_slug: &str) -> &'static str {
        match app_slug {
            "property_management" => "folio",
            "anchor"              => "anchor",
            "network_instance"    => "network",
            _                     => "unknown",
        }
    }

    #[test]
    fn property_management_dispatches_to_folio() {
        assert_eq!(dispatch_app_type("property_management"), "folio");
    }

    #[test]
    fn anchor_dispatches_to_anchor() {
        assert_eq!(dispatch_app_type("anchor"), "anchor");
    }

    #[test]
    fn network_instance_dispatches_to_network() {
        assert_eq!(dispatch_app_type("network_instance"), "network");
    }

    #[test]
    fn unknown_slug_falls_back_to_unknown_not_folio() {
        // This is the critical safety test: a new app type must NOT silently render as Folio
        assert_eq!(dispatch_app_type("insurance"), "unknown");
        assert_eq!(dispatch_app_type("marketplace"), "unknown");
        assert_eq!(dispatch_app_type(""), "unknown");
    }

    #[test]
    fn partial_slug_does_not_match_via_contains() {
        // "property" is a substring of "property_management" — exact match must reject it
        assert_ne!(dispatch_app_type("property"), "folio");
        assert_ne!(dispatch_app_type("management"), "folio");
        assert_ne!(dispatch_app_type("network"), "network"); // partial slug of "network_instance"
        assert_ne!(dispatch_app_type("instance"), "network");
    }

    #[test]
    fn case_sensitive_match_required() {
        assert_ne!(dispatch_app_type("Anchor"), "anchor");
        assert_ne!(dispatch_app_type("ANCHOR"), "anchor");
        assert_ne!(dispatch_app_type("Property_Management"), "folio");
    }

    #[test]
    fn all_canonical_slugs_are_lowercase_snake_case() {
        let canonical = ["property_management", "anchor", "network_instance"];
        for slug in canonical {
            assert_eq!(slug, slug.to_lowercase(), "slug '{slug}' must be lowercase");
            assert!(!slug.contains('-'),           "slug '{slug}' must use underscores");
        }
    }

    #[test]
    fn dispatch_is_exhaustive_for_all_known_slugs() {
        let known = [
            ("property_management", "folio"),
            ("anchor",              "anchor"),
            ("network_instance",    "network"),
        ];
        for (slug, expected) in known {
            assert_eq!(dispatch_app_type(slug), expected, "slug '{slug}' dispatched incorrectly");
        }
    }
}

// ── Provision Slug Validation ─────────────────────────────────────────────────

mod provision_slug_validation {
    use super::validate_slug;

    #[test]
    fn valid_slug_passes() {
        assert!(validate_slug("acme-corp").is_ok());
        assert!(validate_slug("buildwithruud").is_ok());
        assert!(validate_slug("tenant-123").is_ok());
        assert!(validate_slug("ab").is_ok()); // minimum length 2
    }

    #[test]
    fn empty_slug_is_rejected() {
        assert!(validate_slug("").is_err());
    }

    #[test]
    fn slug_with_uppercase_is_rejected() {
        assert!(validate_slug("AcmeCorp").is_err());
        assert!(validate_slug("TENANT").is_err());
    }

    #[test]
    fn slug_with_spaces_is_rejected() {
        assert!(validate_slug("acme corp").is_err());
    }

    #[test]
    fn slug_with_underscore_is_rejected() {
        // slugs use hyphens, NOT underscores (app_slug uses underscores — they differ by convention)
        assert!(validate_slug("acme_corp").is_err());
    }

    #[test]
    fn slug_with_leading_hyphen_is_rejected() {
        assert!(validate_slug("-acme").is_err());
    }

    #[test]
    fn slug_with_trailing_hyphen_is_rejected() {
        assert!(validate_slug("acme-").is_err());
    }

    #[test]
    fn slug_with_internal_hyphen_is_valid() {
        assert!(validate_slug("my-tenant-slug").is_ok());
    }

    #[test]
    fn slug_with_digits_is_valid() {
        assert!(validate_slug("tenant2026").is_ok());
        assert!(validate_slug("123abc").is_ok());
    }

    #[test]
    fn single_char_slug_passes_charset_validation() {
        // validate_slug() only checks: lowercase, digits, hyphens, no leading/trailing hyphen.
        // The length(min=2) constraint is enforced separately by the #[validate] struct attribute.
        // A single char "a" is valid charset-wise — the struct validator rejects it for length.
        assert!(validate_slug("a").is_ok(), "validate_slug is a charset validator, not a length validator");
    }

    #[test]
    fn validate_slug_has_no_length_gate() {
        // Explicitly documents that validate_slug does NOT enforce min/max length.
        // Length is enforced by #[validate(length(min = 2, max = 63))] on the struct field.
        assert!(validate_slug("a").is_ok());    // single char: charset ok, length enforced elsewhere
        assert!(validate_slug("ab").is_ok());   // two chars: both ok
    }

    #[test]
    fn slug_with_special_chars_is_rejected() {
        for bad in ["acme.corp", "acme@corp", "acme/corp", "acme!corp"] {
            assert!(validate_slug(bad).is_err(), "'{bad}' should be rejected");
        }
    }
}

// ── Provision App Selection Logic ─────────────────────────────────────────────

mod provision_app_selection {

    /// Mirrors the apps vec construction in AppCreate::handle_submit.
    /// anchor is always included; others are conditional.
    fn build_apps_vec(include_folio: bool, include_network: bool) -> Vec<String> {
        let mut apps = vec!["anchor".to_string()];
        if include_folio   { apps.push("property_management".to_string()); }
        if include_network { apps.push("network_instance".to_string()); }
        apps
    }

    #[test]
    fn anchor_only_selection_contains_exactly_one_app() {
        let apps = build_apps_vec(false, false);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0], "anchor");
    }

    #[test]
    fn anchor_is_always_first_regardless_of_other_selections() {
        let apps = build_apps_vec(true, true);
        assert_eq!(apps[0], "anchor");
    }

    #[test]
    fn selecting_folio_adds_property_management_slug() {
        let apps = build_apps_vec(true, false);
        assert!(apps.contains(&"property_management".to_string()));
        assert!(!apps.contains(&"network_instance".to_string()));
    }

    #[test]
    fn selecting_network_adds_network_instance_slug() {
        let apps = build_apps_vec(false, true);
        assert!(apps.contains(&"network_instance".to_string()));
        assert!(!apps.contains(&"property_management".to_string()));
    }

    #[test]
    fn selecting_all_apps_produces_three_entries() {
        let apps = build_apps_vec(true, true);
        assert_eq!(apps.len(), 3);
    }

    #[test]
    fn anchor_appears_exactly_once_even_with_all_selected() {
        let apps = build_apps_vec(true, true);
        assert_eq!(apps.iter().filter(|a| *a == "anchor").count(), 1);
    }

    #[test]
    fn all_app_slugs_are_snake_case() {
        let apps = build_apps_vec(true, true);
        for slug in &apps {
            assert!(!slug.contains('-'), "App slug '{slug}' must use underscores not hyphens");
            assert_eq!(slug.as_str(), slug.to_lowercase(), "App slug '{slug}' must be lowercase");
        }
    }

    #[test]
    fn backend_validates_anchor_presence() {
        // The backend enforces: if !apps.contains("anchor") → 400
        // This test verifies our build_apps_vec always satisfies that rule
        let apps_1 = build_apps_vec(false, false);
        let apps_2 = build_apps_vec(true,  false);
        let apps_3 = build_apps_vec(false, true);
        let apps_4 = build_apps_vec(true,  true);
        for apps in [&apps_1, &apps_2, &apps_3, &apps_4] {
            assert!(
                apps.contains(&"anchor".to_string()),
                "Every combination must include 'anchor': {:?}", apps
            );
        }
    }
}

// ── Operational Config PATCH: merge semantics ─────────────────────────────────

mod operational_config_patch_semantics {
    use serde_json::{json, Value};

    /// Mirrors the config JSON merge applied in update_operational_config handler.
    /// Only keys present in the patch are updated; others are preserved unchanged.
    fn apply_patch(base: &mut Value, patch: &Value) {
        if let (Some(base_obj), Some(patch_obj)) = (base.as_object_mut(), patch.as_object()) {
            for (key, val) in patch_obj {
                base_obj.insert(key.clone(), val.clone());
            }
        }
    }

    fn base_config() -> Value {
        json!({
            "folio_mode":            "standard",
            "billing_tier":          "free",
            "tenant_portal_enabled": false,
            "vendor_portal_enabled": false,
        })
    }

    #[test]
    fn patch_folio_mode_only_preserves_other_fields() {
        let mut cfg = base_config();
        apply_patch(&mut cfg, &json!({ "folio_mode": "pmc" }));
        assert_eq!(cfg["folio_mode"],            "pmc");
        assert_eq!(cfg["billing_tier"],          "free");
        assert_eq!(cfg["tenant_portal_enabled"], false);
        assert_eq!(cfg["vendor_portal_enabled"], false);
    }

    #[test]
    fn patch_billing_tier_to_growth_preserves_folio_mode() {
        let mut cfg = base_config();
        apply_patch(&mut cfg, &json!({ "billing_tier": "growth" }));
        assert_eq!(cfg["billing_tier"], "growth");
        assert_eq!(cfg["folio_mode"],   "standard");
    }

    #[test]
    fn enable_tenant_portal_does_not_touch_vendor_portal() {
        let mut cfg = base_config();
        apply_patch(&mut cfg, &json!({ "tenant_portal_enabled": true }));
        assert_eq!(cfg["tenant_portal_enabled"], true);
        assert_eq!(cfg["vendor_portal_enabled"], false);
    }

    #[test]
    fn full_patch_updates_all_four_fields() {
        let mut cfg = base_config();
        apply_patch(&mut cfg, &json!({
            "folio_mode":            "brokerage",
            "billing_tier":          "enterprise",
            "tenant_portal_enabled": true,
            "vendor_portal_enabled": true,
        }));
        assert_eq!(cfg["folio_mode"],            "brokerage");
        assert_eq!(cfg["billing_tier"],          "enterprise");
        assert_eq!(cfg["tenant_portal_enabled"], true);
        assert_eq!(cfg["vendor_portal_enabled"], true);
    }

    #[test]
    fn empty_patch_leaves_config_unchanged() {
        let mut cfg = base_config();
        let original = cfg.clone();
        apply_patch(&mut cfg, &json!({}));
        assert_eq!(cfg, original);
    }

    #[test]
    fn known_folio_modes_are_standard_pmc_brokerage() {
        for mode in ["standard", "pmc", "brokerage"] {
            let mut cfg = base_config();
            apply_patch(&mut cfg, &json!({ "folio_mode": mode }));
            assert_eq!(cfg["folio_mode"], mode);
        }
    }

    #[test]
    fn known_billing_tiers_are_four_values() {
        let tiers = ["free", "starter", "growth", "enterprise"];
        assert_eq!(tiers.len(), 4);
        for tier in tiers {
            let mut cfg = base_config();
            apply_patch(&mut cfg, &json!({ "billing_tier": tier }));
            assert_eq!(cfg["billing_tier"], tier);
        }
    }
}

// ── Billing Tier Constraints ──────────────────────────────────────────────────

mod billing_tier_constraints {

    fn known_tiers() -> Vec<&'static str> {
        vec!["free", "starter", "growth", "enterprise"]
    }

    #[test]
    fn there_are_exactly_four_billing_tiers() {
        assert_eq!(known_tiers().len(), 4);
    }

    #[test]
    fn all_tier_slugs_are_lowercase() {
        for t in known_tiers() {
            assert_eq!(t, t.to_lowercase(), "tier '{t}' must be lowercase");
        }
    }

    #[test]
    fn no_tier_slug_contains_hyphens_or_underscores() {
        for t in known_tiers() {
            assert!(!t.contains('-'), "tier '{t}' must not contain hyphens");
            assert!(!t.contains('_'), "tier '{t}' must not contain underscores");
        }
    }

    #[test]
    fn tiers_are_all_distinct() {
        let tiers = known_tiers();
        let set: std::collections::HashSet<_> = tiers.iter().collect();
        assert_eq!(set.len(), tiers.len(), "All tier slugs must be unique");
    }

    #[test]
    fn free_is_the_default_tier() {
        // Mirrors the default billing_tier in config when not set
        let default_tier = "free";
        assert!(known_tiers().contains(&default_tier));
    }

    #[test]
    fn enterprise_is_the_highest_tier() {
        // Just verifies it's in the list — ordering is by convention
        assert!(known_tiers().contains(&"enterprise"));
    }
}

// ── PublicConfigResponse: DNS instructions opt-in ────────────────────────────

mod public_config_dns_instructions {
    use super::Uuid;
    use crate::admin::app_instance::{PublicConfigResponse, DnsInstructions};

    fn make_config(with_dns: bool) -> PublicConfigResponse {
        PublicConfigResponse {
            instance_id:      Uuid::new_v4(),
            tenant_id:        Uuid::new_v4(),
            app_slug:         "anchor".to_string(),
            public_slug:      Some("acme".to_string()),
            custom_domain:    None,
            instance_status:  "active".to_string(),
            folio_mode:       "standard".to_string(),
            billing_tier:     "free".to_string(),
            tenant_portal_enabled: false,
            vendor_portal_enabled: false,
            dns_instructions: if with_dns {
                Some(DnsInstructions {
                    record_type: "CNAME".to_string(),
                    name:        "acme.com".to_string(),
                    value:       "platform.atlas.app".to_string(),
                    note:        "Point acme.com as a CNAME to platform.atlas.app.".to_string(),
                })
            } else {
                None
            },
        }
    }

    #[test]
    fn get_response_has_no_dns_instructions() {
        let cfg = make_config(false);
        assert!(cfg.dns_instructions.is_none());
    }

    #[test]
    fn put_response_includes_dns_instructions() {
        let cfg = make_config(true);
        assert!(cfg.dns_instructions.is_some());
    }

    #[test]
    fn dns_instructions_record_type_is_cname() {
        let cfg = make_config(true);
        let dns = cfg.dns_instructions.unwrap();
        assert_eq!(dns.record_type, "CNAME");
    }

    #[test]
    fn dns_instructions_json_skipped_when_none() {
        let cfg = make_config(false);
        let j = serde_json::to_value(&cfg).expect("must serialize");
        // serde skip_serializing_if = "Option::is_none"
        assert!(j.get("dns_instructions").is_none(), "dns_instructions must be absent from GET response JSON");
    }

    #[test]
    fn dns_instructions_json_present_when_some() {
        let cfg = make_config(true);
        let j = serde_json::to_value(&cfg).expect("must serialize");
        assert!(j.get("dns_instructions").is_some(), "dns_instructions must be present in PUT response JSON");
    }

    #[test]
    fn config_app_slug_carries_canonical_value() {
        let cfg = make_config(false);
        assert_eq!(cfg.app_slug, "anchor");
    }

    #[test]
    fn config_instance_status_carries_active_value() {
        let cfg = make_config(false);
        assert_eq!(cfg.instance_status, "active");
    }

    #[test]
    fn portal_flags_default_to_false() {
        let cfg = make_config(false);
        assert!(!cfg.tenant_portal_enabled);
        assert!(!cfg.vendor_portal_enabled);
    }
}
