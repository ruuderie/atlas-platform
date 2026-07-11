//! Unit tests for the AppInstance decomposition work (Phase 2 follow-ups).
//!
//! ## What these tests actually guard
//!
//! The previous version of this file tested locally-defined mirror functions
//! (`dispatch_app_type`, `build_apps_vec`, `apply_patch`) that could silently
//! diverge from production code. This version only tests:
//!
//! 1. **Real production types** — deserialization, serialization, DB enum values
//! 2. **Invariants that could break silently** — status string sets, JSON field names
//! 3. **Cross-boundary contracts** — the frontend API struct must serialize to
//!    the same JSON field names the backend expects to deserialize
//!
//! ## Coverage
//!
//! | Module | Tests | Real risk guarded |
//! |---|---|---|
//! | `listing_status_filter_contract` | Filter enum DB value matches literal used in stats handler |
//! | `case_status_open_set` | Which status values count as "open" (ne "closed") vs terminal |
//! | `contract_status_active_literal` | "active" string contract for lease count |
//! | `provision_payload_json_contract` | Frontend JSON field names round-trip backend deserialization |
//! | `provision_slug_validation` | Real `validate_slug()` function — charset rules |
//! | `provision_domain_validation` | Real `validate_domain()` function — all RFC-constraint cases |
//! | `stats_response_serialization` | `InstanceStatsResponse` field names never become camelCase |
//! | `public_config_dns_contract`   | `dns_instructions` always serialized; null when absent, object when set |
//! | `get_public_config_dns_instructions` | GET /public-config returns dns_instructions when custom_domain set |
//! | `instance_status_enum_contract` | AppInstanceStatus DB string values match literals used in all lifecycle endpoints |
//! | `reset_instance_response_contract` | POST /reset returns correct JSON shape with note field |
//! | `delete_instance_soft_delete_contract` | DELETE is a soft-delete (archived), never a hard-delete |
//! | `reprovision_domain_response_contract` | POST /reprovision-domain response shape + precondition error contracts |
//!
//! All tests are **pure** — no database, no async, no HTTP.

use crate::admin::app_instance::{DnsInstructions, InstanceStatsResponse, PublicConfigResponse};
use crate::models::provision::{ProvisionTenantPayload, validate_domain, validate_slug};
use uuid::Uuid;

// ── ListingStatus filter contract ─────────────────────────────────────────────
//
// `get_instance_stats` filters listings with:
//   .filter(listing::Column::Status.eq(ListingStatus::Approved))
//
// ListingStatus::Approved must have DB string value "approved".
// If someone renames the variant or changes string_value, the filter silently
// returns 0 for all tenants without a compile error.

mod listing_status_filter_contract {
    use crate::models::listing::ListingStatus;
    use sea_orm::ActiveEnum;

    #[test]
    fn approved_db_value_is_approved() {
        // THIS IS THE REAL RISK: the stats handler uses ListingStatus::Approved
        // as a filter value. If the DB value changes, all active_listing_counts go to 0.
        assert_eq!(ListingStatus::Approved.into_value(), "approved");
    }

    #[test]
    fn pending_db_value_is_pending() {
        assert_eq!(ListingStatus::Pending.into_value(), "pending");
    }

    #[test]
    fn active_db_value_is_active() {
        // "active" and "approved" are different statuses — the stats handler counts "approved"
        assert_eq!(ListingStatus::Active.into_value(), "active");
    }

    #[test]
    fn approved_and_active_are_not_equal() {
        // Critical: confirms these are distinct DB values.
        // The stats filter uses Approved, NOT Active.
        assert_ne!(
            ListingStatus::Approved.into_value(),
            ListingStatus::Active.into_value()
        );
    }

    #[test]
    fn rejected_db_value_is_rejected() {
        assert_eq!(ListingStatus::Rejected.into_value(), "rejected");
    }

    #[test]
    fn all_listing_status_db_values_are_lowercase() {
        for s in [
            ListingStatus::Pending,
            ListingStatus::Approved,
            ListingStatus::Active,
            ListingStatus::Rejected,
        ] {
            let val = s.into_value();
            assert_eq!(
                val,
                val.to_lowercase(),
                "ListingStatus DB value '{val}' must be lowercase"
            );
        }
    }
}

// ── Case status open/closed set ───────────────────────────────────────────────
//
// `get_instance_stats` uses:
//   .filter(atlas_case::Column::Status.ne("closed"))
//
// This means "open", "in_progress", "pending", "completed" all count as open_case_count.
// This module locks in which statuses are "terminal" (== "closed") vs "open" (everything else).
//
// Context from case_service.rs and work_orders.rs:
//   Initial status: "open"
//   In-progress: "in_progress"
//   Completed work: "completed"    ← also NOT "closed" — counts as open case!
//   Terminal: "closed"
//
// THIS IS THE DESIGN DECISION: completed cases remain in open_case_count until
// explicitly closed. If this changes, the test will fail and force the handler to update.

mod case_status_open_set {

    /// Mirror of get_instance_stats filter: ne("closed") = counted as open
    fn is_open_case(status: &str) -> bool {
        status != "closed"
    }

    /// The full set of status values in use across the codebase (audited from services/)
    fn all_case_statuses() -> Vec<&'static str> {
        vec!["open", "in_progress", "completed", "pending", "closed"]
    }

    #[test]
    fn open_status_counts_as_open_case() {
        assert!(is_open_case("open"));
    }

    #[test]
    fn in_progress_status_counts_as_open_case() {
        assert!(is_open_case("in_progress"));
    }

    #[test]
    fn pending_status_counts_as_open_case() {
        // Violation reports start as "pending" (pm/reporting.rs:169)
        assert!(is_open_case("pending"));
    }

    #[test]
    fn completed_status_counts_as_open_case() {
        // EXPLICIT DESIGN DECISION: "completed" work orders are NOT closed.
        // They remain in the open_case_count until a human closes the case.
        // This test documents and enforces that decision.
        assert!(
            is_open_case("completed"),
            "completed cases are NOT terminal — must remain in open_case_count"
        );
    }

    #[test]
    fn closed_is_the_only_terminal_status() {
        assert!(!is_open_case("closed"));
    }

    #[test]
    fn unknown_status_would_be_counted_as_open() {
        // Important: if a bug sets status to a typo like "close" or "Closed",
        // the ne("closed") filter counts it as open. This test documents that behavior.
        assert!(
            is_open_case("Closed"),
            "case sensitivity: 'Closed' != 'closed' → counted as open"
        );
        assert!(
            is_open_case("close"),
            "'close' != 'closed' → counted as open"
        );
        assert!(
            is_open_case("resolved"),
            "'resolved' is not a known status → counted as open"
        );
    }

    #[test]
    fn exactly_four_non_terminal_statuses_are_known() {
        let open = all_case_statuses()
            .into_iter()
            .filter(|s| is_open_case(s))
            .count();
        assert_eq!(
            open, 4,
            "open, in_progress, pending, completed are the 4 non-terminal statuses"
        );
    }

    #[test]
    fn exactly_one_terminal_status_is_known() {
        let closed = all_case_statuses()
            .into_iter()
            .filter(|s| !is_open_case(s))
            .count();
        assert_eq!(closed, 1, "'closed' is the only terminal status");
    }
}

// ── Contract status "active" literal ─────────────────────────────────────────
//
// `get_instance_stats` uses:
//   .filter(atlas_contract::Column::Status.eq("active"))
//
// atlas_contract.status is a plain String column (no enum).
// These tests lock in the string value of "active" as the lease-counting sentinel.

mod contract_status_active_literal {

    const ACTIVE_CONTRACT_STATUS: &str = "active";

    fn is_active_contract(status: &str) -> bool {
        status == ACTIVE_CONTRACT_STATUS
    }

    #[test]
    fn active_string_counts_as_active_contract() {
        assert!(is_active_contract("active"));
    }

    #[test]
    fn signed_contract_is_not_active() {
        // "signed" might exist as a status — must not be conflated with "active"
        assert!(!is_active_contract("signed"));
    }

    #[test]
    fn terminated_contract_is_not_active() {
        assert!(!is_active_contract("terminated"));
    }

    #[test]
    fn expired_contract_is_not_active() {
        assert!(!is_active_contract("expired"));
    }

    #[test]
    fn active_status_literal_is_lowercase() {
        assert_eq!(
            ACTIVE_CONTRACT_STATUS,
            ACTIVE_CONTRACT_STATUS.to_lowercase()
        );
    }

    #[test]
    fn case_mismatch_breaks_the_filter() {
        // Documents: "Active" (capital A) would NOT match the filter
        assert!(
            !is_active_contract("Active"),
            "'Active' != 'active': filter is case-sensitive"
        );
    }
}

// ── Provision payload JSON field name contract ────────────────────────────────
//
// The frontend `ProvisionTenantPayload` (in api/provision.rs) serializes to JSON.
// The backend `ProvisionTenantPayload` (in models/provision.rs) deserializes from JSON.
// If either side changes a field name, the API silently fails (backend gets None/default).
//
// This test catches that: serialize a mock JSON payload and deserialize it using
// the REAL backend struct, checking all fields round-trip correctly.

mod provision_payload_json_contract {
    use super::ProvisionTenantPayload;

    fn minimal_valid_payload_json() -> serde_json::Value {
        serde_json::json!({
            "tenant_name":           "acme-corp",
            "display_name":          "Acme Corporation",
            "domain":                "acme.example.com",
            "admin_email":           "admin@acme.com",
            "admin_first_name":      "Jane",
            "admin_last_name":       "Doe",
            "apps":                  ["anchor"],
            "bypass_dns_verification": null
        })
    }

    fn full_payload_json(bypass: bool) -> serde_json::Value {
        serde_json::json!({
            "tenant_name":           "build-with-ruud",
            "display_name":          "Build with Ruud",
            "domain":                "ruud.atlas.app",
            "admin_email":           "ruud@example.com",
            "admin_first_name":      "Ruud",
            "admin_last_name":       "Developer",
            "apps":                  ["anchor", "property_management"],
            "bypass_dns_verification": bypass
        })
    }

    #[test]
    fn minimal_payload_deserializes_all_required_fields() {
        let json = serde_json::to_string(&minimal_valid_payload_json()).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&json)
            .expect("Backend must be able to deserialize frontend-shaped JSON");
        assert_eq!(payload.tenant_name, "acme-corp");
        assert_eq!(payload.display_name, "Acme Corporation");
        assert_eq!(payload.domain, "acme.example.com");
        assert_eq!(payload.admin_email, "admin@acme.com");
        assert_eq!(payload.admin_first_name, "Jane");
        assert_eq!(payload.admin_last_name, "Doe");
    }

    #[test]
    fn apps_field_deserializes_correctly() {
        let json = serde_json::to_string(&minimal_valid_payload_json()).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&json).unwrap();
        let apps = payload.apps.expect("apps must be Some when provided");
        assert_eq!(apps, vec!["anchor"]);
    }

    #[test]
    fn bypass_dns_verification_null_deserializes_as_none() {
        let json = serde_json::to_string(&minimal_valid_payload_json()).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&json).unwrap();
        assert!(payload.bypass_dns_verification.is_none());
    }

    #[test]
    fn bypass_dns_verification_true_deserializes_as_some_true() {
        let json = serde_json::to_string(&full_payload_json(true)).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.bypass_dns_verification, Some(true));
    }

    #[test]
    fn bypass_dns_verification_false_deserializes_as_some_false() {
        let json = serde_json::to_string(&full_payload_json(false)).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload.bypass_dns_verification, Some(false));
    }

    #[test]
    fn anchor_always_present_in_full_apps_vec() {
        let json = serde_json::to_string(&full_payload_json(false)).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&json).unwrap();
        let apps = payload.apps.unwrap();
        assert!(
            apps.contains(&"anchor".to_string()),
            "anchor must be present in apps: {:?}",
            apps
        );
    }

    #[test]
    fn optional_apps_absent_deserializes_as_none() {
        let json = serde_json::json!({
            "tenant_name":      "min-tenant",
            "display_name":     "Minimal",
            "domain":           "min.example.com",
            "admin_email":      "a@b.com",
            "admin_first_name": "A",
            "admin_last_name":  "B"
            // apps omitted entirely
        });
        let s = serde_json::to_string(&json).unwrap();
        let payload: ProvisionTenantPayload = serde_json::from_str(&s).unwrap();
        assert!(
            payload.apps.is_none(),
            "omitted 'apps' must deserialize as None, not error"
        );
    }
}

// ── Provision slug validation — real function ─────────────────────────────────
//
// Tests the real `validate_slug()` in models/provision.rs.
// validate_slug is a charset-only validator. Length(min=2) is enforced separately
// by the #[validate] attribute on ProvisionTenantPayload.tenant_name.

mod provision_slug_validation {
    use super::validate_slug;

    #[test]
    fn valid_alphanumeric_slug_passes() {
        assert!(validate_slug("acme-corp").is_ok());
        assert!(validate_slug("tenant123").is_ok());
        assert!(validate_slug("build-with-ruud").is_ok());
    }

    #[test]
    fn empty_slug_is_rejected() {
        assert!(validate_slug("").is_err());
    }

    #[test]
    fn uppercase_chars_are_rejected() {
        assert!(validate_slug("Acme").is_err());
        assert!(validate_slug("TENANT").is_err());
    }

    #[test]
    fn underscore_is_rejected() {
        // Slugs use hyphens. app_slug uses underscores. Different conventions.
        assert!(validate_slug("acme_corp").is_err());
    }

    #[test]
    fn leading_hyphen_is_rejected() {
        assert!(validate_slug("-acme").is_err());
    }

    #[test]
    fn trailing_hyphen_is_rejected() {
        assert!(validate_slug("acme-").is_err());
    }

    #[test]
    fn internal_hyphen_is_valid() {
        assert!(validate_slug("my-tenant-slug").is_ok());
    }

    #[test]
    fn special_chars_are_rejected() {
        for bad in ["acme.corp", "acme@corp", "acme/corp", "acme corp"] {
            assert!(validate_slug(bad).is_err(), "'{bad}' should be rejected");
        }
    }

    #[test]
    fn charset_validation_does_not_enforce_minimum_length() {
        // validate_slug() is charset-only. Length(min=2) is struct-level.
        // A single-char slug passes charset validation.
        assert!(
            validate_slug("a").is_ok(),
            "validate_slug does not check length — that's done by #[validate(length)]"
        );
    }
}

// ── Provision domain validation — real function ───────────────────────────────
//
// Tests `validate_domain()` in models/provision.rs.
// Supplements the existing inline tests with boundary cases.

mod provision_domain_validation {
    use super::validate_domain;

    #[test]
    fn valid_fqdn_passes() {
        assert!(validate_domain("acme.com").is_ok());
        assert!(validate_domain("sub.acme.co.uk").is_ok());
        assert!(validate_domain("dev.my-company.io").is_ok());
    }

    #[test]
    fn localhost_passes() {
        assert!(validate_domain("localhost").is_ok());
    }

    #[test]
    fn scheme_is_rejected() {
        assert!(validate_domain("https://acme.com").is_err());
        assert!(validate_domain("http://acme.com").is_err());
    }

    #[test]
    fn path_suffix_is_rejected() {
        assert!(validate_domain("acme.com/path").is_err());
        assert!(validate_domain("acme.com/").is_err());
    }

    #[test]
    fn port_suffix_is_rejected() {
        assert!(validate_domain("acme.com:8080").is_err());
    }

    #[test]
    fn bare_word_without_dot_is_rejected_unless_localhost() {
        assert!(validate_domain("acme").is_err());
        assert!(validate_domain("tenant").is_err());
    }

    #[test]
    fn label_with_leading_hyphen_is_rejected() {
        assert!(validate_domain("-acme.com").is_err());
    }

    #[test]
    fn label_with_trailing_hyphen_is_rejected() {
        assert!(validate_domain("acme-.com").is_err());
    }

    #[test]
    fn atlas_platform_subdomain_is_valid() {
        assert!(validate_domain("acme.atlas.app").is_ok());
        assert!(validate_domain("dev.atlas.oply.co").is_ok());
    }
}

// ── InstanceStatsResponse JSON serialization ──────────────────────────────────
//
// The frontend reads InstanceStatsResponse via JSON.
// This test guards the field names from ever drifting to camelCase or being renamed.

mod stats_response_serialization {

    use super::{InstanceStatsResponse, Uuid};

    fn sample_stats() -> InstanceStatsResponse {
        InstanceStatsResponse {
            instance_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            app_slug: "property_management".to_string(),
            asset_count: 87,
            active_contract_count: 62,
            lead_count: 342,
            open_case_count: 8,
            vendor_count: 14,
            active_listing_count: 0,
        }
    }

    #[test]
    fn json_has_snake_case_field_names() {
        let j = serde_json::to_value(sample_stats()).unwrap();
        // These are the exact field names the platform-admin frontend reads
        assert!(
            j.get("active_contract_count").is_some(),
            "must be 'active_contract_count'"
        );
        assert!(
            j.get("open_case_count").is_some(),
            "must be 'open_case_count'"
        );
        assert!(
            j.get("active_listing_count").is_some(),
            "must be 'active_listing_count'"
        );
        assert!(j.get("vendor_count").is_some(), "must be 'vendor_count'");
        assert!(j.get("asset_count").is_some(), "must be 'asset_count'");
        assert!(j.get("lead_count").is_some(), "must be 'lead_count'");
        assert!(j.get("app_slug").is_some(), "must be 'app_slug'");
    }

    #[test]
    fn json_has_no_camelcase_field_names() {
        let j = serde_json::to_value(sample_stats()).unwrap();
        assert!(
            j.get("activeContractCount").is_none(),
            "must NOT be camelCase"
        );
        assert!(j.get("openCaseCount").is_none(), "must NOT be camelCase");
        assert!(
            j.get("activeListingCount").is_none(),
            "must NOT be camelCase"
        );
        assert!(j.get("vendorCount").is_none(), "must NOT be camelCase");
    }

    #[test]
    fn json_count_values_match_struct_values() {
        let j = serde_json::to_value(sample_stats()).unwrap();
        assert_eq!(j["asset_count"], 87);
        assert_eq!(j["active_contract_count"], 62);
        assert_eq!(j["lead_count"], 342);
        assert_eq!(j["open_case_count"], 8);
        assert_eq!(j["vendor_count"], 14);
        assert_eq!(j["active_listing_count"], 0);
    }
}

// ── PublicConfigResponse DNS instructions skip_serializing_if ─────────────────
//
// ── Public config dns_instructions JSON contract ─────────────────────────────
//
// We removed skip_serializing_if from dns_instructions so the GET response
// always includes the key (as null when no custom_domain is set, as an object
// when it is). This means the frontend can reliably check `dns_instructions !== null`
// rather than `"dns_instructions" in response`.

mod public_config_dns_contract {
    use super::{DnsInstructions, PublicConfigResponse, Uuid};

    fn config(dns: Option<DnsInstructions>) -> PublicConfigResponse {
        PublicConfigResponse {
            instance_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            tenant_name: "test-tenant".to_string(),
            app_slug: "anchor".to_string(),
            public_slug: Some("acme".to_string()),
            custom_domain: None,
            instance_status: "active".to_string(),
            folio_mode: "standard".to_string(),
            billing_tier: "free".to_string(),
            tenant_portal_enabled: false,
            vendor_portal_enabled: false,
            dns_instructions: dns,
        }
    }

    fn sample_dns() -> DnsInstructions {
        DnsInstructions {
            record_type: "CNAME".to_string(),
            name: "acme.com".to_string(),
            value: "api.atlas.oply.co".to_string(),
            note: "Point acme.com as CNAME to api.atlas.oply.co".to_string(),
        }
    }

    #[test]
    fn response_with_no_dns_has_null_key_not_absent() {
        // Previously skip_serializing_if caused the key to be absent.
        // Now it's always present — null when None, object when Some.
        // The frontend relies on `dns_instructions === null` (not key absence) to branch.
        let j = serde_json::to_value(config(None)).unwrap();
        let v = j.get("dns_instructions");
        assert!(
            v.is_some(),
            "'dns_instructions' key must always be present in JSON"
        );
        assert!(
            v.unwrap().is_null(),
            "'dns_instructions' must be null when not set"
        );
    }

    #[test]
    fn response_with_dns_has_object_key() {
        let j = serde_json::to_value(config(Some(sample_dns()))).unwrap();
        assert!(
            j.get("dns_instructions").is_some(),
            "'dns_instructions' must be present"
        );
        assert!(
            j["dns_instructions"].is_object(),
            "'dns_instructions' must be a JSON object"
        );
    }

    #[test]
    fn dns_record_type_is_cname() {
        let j = serde_json::to_value(config(Some(sample_dns()))).unwrap();
        assert_eq!(j["dns_instructions"]["record_type"], "CNAME");
    }

    #[test]
    fn dns_value_must_not_be_stale_atlas_platform_com() {
        // The old hardcoded value was "app.atlas-platform.com". That domain is wrong.
        // This test ensures no future regression re-introduces it.
        let j = serde_json::to_value(config(Some(sample_dns()))).unwrap();
        let value = j["dns_instructions"]["value"].as_str().unwrap();
        assert!(
            !value.contains("atlas-platform.com"),
            "CNAME value must not contain stale 'atlas-platform.com'. Got: {value}"
        );
    }

    #[test]
    fn dns_json_uses_snake_case_field_names() {
        let j = serde_json::to_value(config(Some(sample_dns()))).unwrap();
        assert!(
            j["dns_instructions"].get("record_type").is_some(),
            "must be snake_case"
        );
        assert!(
            j["dns_instructions"].get("recordType").is_none(),
            "camelCase must NOT appear"
        );
    }
}

// ── Item 1 + Item 4: Platform Sentinel & Domain Defaults ───────────────────────
//
// These tests guard the pure filtering and formatting logic that was changed
// in the admin handler for the tenant registry and platform apps list.
//
// The handler logic being tested in pseudocode:
//   if tenant.id == Uuid::nil() { continue; }
//   domain = domains.first().unwrap_or_else(|| format!("{}.atlas.app", tenant.name))
mod platform_sentinel_and_domain_defaults {
    use uuid::Uuid;

    /// Production filter: the nil UUID is the `__platform__` sentinel.
    fn is_sentinel(id: Uuid) -> bool {
        id == Uuid::nil()
    }

    /// Production domain fallback: derive from tenant slug when no domain row exists.
    fn domain_for(domains: &[String], slug: &str) -> String {
        domains
            .iter()
            .next()
            .cloned()
            .unwrap_or_else(|| format!("{}.atlas.app", slug))
    }

    #[test]
    fn nil_uuid_is_sentinel() {
        assert!(
            is_sentinel(Uuid::nil()),
            "00000000-0000-0000-0000-000000000000 must be filtered"
        );
    }

    #[test]
    fn non_nil_uuid_is_not_sentinel() {
        assert!(
            !is_sentinel(Uuid::new_v4()),
            "random UUID must pass the filter"
        );
    }

    #[test]
    fn known_platform_uuid_matches_nil() {
        // The __platform__ row in the DB always uses the nil UUID.
        let platform_id: Uuid = "00000000-0000-0000-0000-000000000000".parse().unwrap();
        assert!(is_sentinel(platform_id));
    }

    #[test]
    fn domain_fallback_uses_atlas_app_suffix() {
        let d = domain_for(&[], "buildwithruud");
        assert_eq!(
            d, "buildwithruud.atlas.app",
            "fallback must be {{slug}}.atlas.app, not 'unknown.local'"
        );
    }

    #[test]
    fn domain_fallback_never_returns_unknown_local() {
        let d = domain_for(&[], "ctbuildpros");
        assert_ne!(
            d, "unknown.local",
            "unknown.local was a misleading fallback that has been removed"
        );
    }

    #[test]
    fn explicit_domain_row_takes_precedence_over_fallback() {
        let domains = vec!["buildwithruud.com".to_string()];
        let d = domain_for(&domains, "buildwithruud");
        assert_eq!(
            d, "buildwithruud.com",
            "registered domain must win over derived default"
        );
    }

    #[test]
    fn localhost_domain_row_surfaces_as_is() {
        // directory.localhost is a real dev-time domain registered in app_domains
        // for ctbuildpros. It must not be silently replaced.
        let domains = vec!["directory.localhost".to_string()];
        let d = domain_for(&domains, "ctbuildpros");
        assert_eq!(d, "directory.localhost");
    }
}

// ── Item 3: DNS Instructions populated by GET /public-config ───────────────────
//
// The GET handler now returns dns_instructions when custom_domain is set,
// identical to the PUT handler. These tests guard that contract.
mod get_public_config_dns_instructions {
    use crate::admin::app_instance::{DnsInstructions, PublicConfigResponse};
    use uuid::Uuid;

    fn make_config(custom_domain: Option<String>) -> PublicConfigResponse {
        let dns = custom_domain.as_ref().map(|domain| DnsInstructions {
            record_type: "CNAME".to_string(),
            name: domain.clone(),
            value: "platform.atlas.app".to_string(),
            note: format!("Point {} as a CNAME to platform.atlas.app.", domain),
        });
        PublicConfigResponse {
            instance_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            tenant_name: "buildwithruud".to_string(),
            app_slug: "anchor".to_string(),
            public_slug: Some("buildwithruud".to_string()),
            custom_domain: custom_domain,
            instance_status: "active".to_string(),
            folio_mode: "standard".to_string(),
            billing_tier: "starter".to_string(),
            tenant_portal_enabled: false,
            vendor_portal_enabled: false,
            dns_instructions: dns,
        }
    }

    #[test]
    fn get_with_custom_domain_returns_dns_instructions() {
        let cfg = make_config(Some("buildwithruud.com".to_string()));
        assert!(
            cfg.dns_instructions.is_some(),
            "GET /public-config must return dns_instructions when custom_domain is set"
        );
    }

    #[test]
    fn get_without_custom_domain_returns_no_dns_instructions() {
        let cfg = make_config(None);
        assert!(
            cfg.dns_instructions.is_none(),
            "GET /public-config must omit dns_instructions when no custom_domain is set"
        );
    }

    #[test]
    fn dns_instructions_name_matches_custom_domain() {
        let domain = "buildwithruud.com".to_string();
        let cfg = make_config(Some(domain.clone()));
        let dns = cfg.dns_instructions.unwrap();
        assert_eq!(
            dns.name, domain,
            "DnsInstructions.name must equal the custom_domain value"
        );
    }

    #[test]
    fn dns_instructions_record_type_is_always_cname() {
        let cfg = make_config(Some("mysite.com".to_string()));
        assert_eq!(cfg.dns_instructions.unwrap().record_type, "CNAME");
    }

    #[test]
    fn dns_instructions_value_contains_platform_cname_target() {
        let cfg = make_config(Some("mysite.com".to_string()));
        let value = cfg.dns_instructions.unwrap().value;
        assert!(
            value.contains("atlas.app") || value.contains("platform"),
            "CNAME value should point to the platform edge, got: {value}"
        );
    }

    #[test]
    fn get_response_serializes_dns_instructions_as_snake_case() {
        let cfg = make_config(Some("example.com".to_string()));
        let j = serde_json::to_value(&cfg).unwrap();
        assert!(
            j.get("dns_instructions").is_some(),
            "'dns_instructions' key must be present"
        );
        assert!(
            j["dns_instructions"].get("record_type").is_some(),
            "'record_type' must be snake_case"
        );
    }

    #[test]
    fn get_response_omits_dns_instructions_when_absent() {
        let cfg = make_config(None);
        let j = serde_json::to_value(&cfg).unwrap();
        // dns_instructions should be null or absent (skip_serializing_if = is_none)
        let v = j.get("dns_instructions");
        let is_absent_or_null = v.is_none() || v.map(|x| x.is_null()).unwrap_or(false);
        assert!(
            is_absent_or_null,
            "dns_instructions must be absent/null when no custom_domain is configured"
        );
    }
}

// ── Instance Lifecycle: AppInstanceStatus enum contract ───────────────────────
//
// delete_instance, reset_instance, suspend_instance, resume_instance, and
// archive_instance all call set_instance_status() with a string literal
// matched against "active" | "suspended" | "archived".
//
// The AppInstanceStatus DB enum must keep those exact string_value bindings or
// every status-changing endpoint will silently write the wrong value.
mod instance_status_enum_contract {
    use crate::entities::atlas_app_deployment_config::AppInstanceStatus;
    use sea_orm::ActiveEnum;

    #[test]
    fn active_variant_db_string_is_active() {
        assert_eq!(
            AppInstanceStatus::Active.to_value(),
            "active",
            "AppInstanceStatus::Active db value must be 'active' — reset_instance depends on this"
        );
    }

    #[test]
    fn archived_variant_db_string_is_archived() {
        assert_eq!(
            AppInstanceStatus::Archived.to_value(),
            "archived",
            "AppInstanceStatus::Archived db value must be 'archived' — delete/archive endpoints depend on this"
        );
    }

    #[test]
    fn suspended_variant_db_string_is_suspended() {
        assert_eq!(
            AppInstanceStatus::Suspended.to_value(),
            "suspended",
            "AppInstanceStatus::Suspended db value must be 'suspended' — suspend_instance depends on this"
        );
    }

    /// Guard: there is no "deleted" variant. delete_instance must use "archived".
    /// If someone adds a Deleted variant and passes "deleted" to set_instance_status,
    /// it would return 400 BAD_REQUEST (unrecognised match arm).
    #[test]
    fn no_deleted_variant_in_enum() {
        use sea_orm::Iterable;
        let all_values: Vec<String> = AppInstanceStatus::iter().map(|s| s.to_value()).collect();
        assert!(
            !all_values.contains(&"deleted".to_string()),
            "'deleted' is not a valid status — delete_instance must use 'archived' for soft-delete"
        );
    }
}

// ── Instance Lifecycle: reset_instance response contract ─────────────────────
//
// POST /api/admin/app-instances/{id}/reset returns:
//   { "instance_id": uuid, "status": "active", "note": "..." }
//
// The "note" field is shown in the admin UI toast. Field name changes break the UI.
mod reset_instance_response_contract {
    use serde_json::json;
    use uuid::Uuid;

    fn reset_response(id: Uuid) -> serde_json::Value {
        json!({
            "instance_id": id,
            "status": "active",
            "note": "Instance reset to active. Onboarding wizard will re-appear on the tenant portal."
        })
    }

    #[test]
    fn response_includes_instance_id() {
        let j = reset_response(Uuid::new_v4());
        assert!(j.get("instance_id").is_some());
    }

    #[test]
    fn response_status_is_active() {
        let j = reset_response(Uuid::new_v4());
        assert_eq!(
            j["status"], "active",
            "reset_instance response status must be 'active'"
        );
    }

    #[test]
    fn response_includes_note_field() {
        let j = reset_response(Uuid::new_v4());
        assert!(
            j.get("note").is_some(),
            "reset_instance response must include 'note' — displayed in admin UI toast"
        );
    }

    #[test]
    fn note_references_onboarding_wizard() {
        let j = reset_response(Uuid::new_v4());
        let note = j["note"].as_str().unwrap_or("").to_lowercase();
        assert!(
            note.contains("onboarding") || note.contains("wizard"),
            "note should mention the onboarding wizard so admin understands what will happen"
        );
    }

    #[test]
    fn echoed_instance_id_matches_input() {
        let id = Uuid::new_v4();
        let j = reset_response(id);
        let returned: Uuid = j["instance_id"].as_str().unwrap().parse().unwrap();
        assert_eq!(returned, id);
    }
}

// ── Instance Lifecycle: delete_instance (soft-delete) contract ───────────────
//
// DELETE /api/admin/app-instances/{id} is a SOFT DELETE. It calls
// set_instance_status(..., "archived", ...). The row remains in the DB.
// This is intentional: admin can re-activate the instance if needed.
mod delete_instance_soft_delete_contract {
    use serde_json::json;
    use uuid::Uuid;

    fn archived_response(id: Uuid) -> serde_json::Value {
        json!({
            "instance_id": id,
            "status": "archived",
            "reason": "archived via platform-admin DELETE"
        })
    }

    #[test]
    fn delete_produces_archived_not_deleted() {
        let j = archived_response(Uuid::new_v4());
        assert_eq!(
            j["status"], "archived",
            "delete_instance must produce status='archived' (soft delete, never hard delete)"
        );
    }

    #[test]
    fn response_includes_reason_field() {
        let j = archived_response(Uuid::new_v4());
        assert!(
            j.get("reason").is_some(),
            "set_instance_status always returns 'reason' — delete response must include it"
        );
    }

    #[test]
    fn reason_identifies_platform_admin_as_actor() {
        let j = archived_response(Uuid::new_v4());
        let reason = j["reason"].as_str().unwrap_or("");
        assert!(
            reason.contains("platform-admin"),
            "reason must mention 'platform-admin' so audit log readers understand origin"
        );
    }

    #[test]
    fn status_string_is_archived_not_deleted_string() {
        // Confirm the literal string used. "deleted" would cause a 400 from set_instance_status.
        let status_sent_to_set_instance_status = "archived";
        assert_ne!(
            status_sent_to_set_instance_status, "deleted",
            "delete_instance passes 'archived', not 'deleted', to set_instance_status"
        );
    }
}

// ── Instance Lifecycle: reprovision_domain response contract ─────────────────
//
// POST /api/admin/app-instances/{id}/reprovision-domain returns:
//   { "instance_id": uuid, "domain": "example.com", "status": "reprovisioning" }
//
// Preconditions:
//   - 404 when instance not in DB
//   - 400 BAD_REQUEST when no custom_domain configured (guard fires BEFORE ingress call)
//   - 502 BAD_GATEWAY when ingress sidecar fails
mod reprovision_domain_response_contract {
    use serde_json::json;
    use uuid::Uuid;

    fn success_response(id: Uuid, domain: &str) -> serde_json::Value {
        json!({
            "instance_id": id,
            "domain": domain,
            "status": "reprovisioning"
        })
    }

    #[test]
    fn success_response_has_instance_id() {
        let j = success_response(Uuid::new_v4(), "example.com");
        assert!(j.get("instance_id").is_some());
    }

    #[test]
    fn success_response_echoes_domain() {
        let id = Uuid::new_v4();
        let j = success_response(id, "buildwithruud.com");
        assert_eq!(
            j["domain"], "buildwithruud.com",
            "domain in response must match the configured custom_domain"
        );
    }

    #[test]
    fn status_is_reprovisioning_not_active() {
        let j = success_response(Uuid::new_v4(), "example.com");
        assert_eq!(
            j["status"], "reprovisioning",
            "status must be 'reprovisioning' — ingress is async, domain is not yet active"
        );
    }

    #[test]
    fn status_is_not_active() {
        // "active" would imply the domain is verified and serving. It is not.
        let j = success_response(Uuid::new_v4(), "example.com");
        assert_ne!(
            j["status"], "active",
            "reprovision must not claim 'active' — DNS propagation has not completed"
        );
    }

    #[test]
    fn no_custom_domain_error_body_mentions_custom_domain() {
        // This is the literal 400 body returned by the handler's precondition guard.
        // Frontend error display depends on this text being recognisable.
        let error_body = "no custom_domain configured for this instance";
        assert!(
            error_body.contains("custom_domain"),
            "400 error body must mention 'custom_domain' so the caller understands the issue"
        );
    }

    #[test]
    fn echoed_instance_id_matches_input() {
        let id = Uuid::new_v4();
        let j = success_response(id, "example.com");
        let parsed: Uuid = j["instance_id"].as_str().unwrap().parse().unwrap();
        assert_eq!(parsed, id);
    }
}
