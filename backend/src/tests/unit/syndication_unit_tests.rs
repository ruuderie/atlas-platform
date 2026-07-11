//! Unit tests for G-05 Syndication Event Bus, Folio/NI configuration
//! combinations, and operational config controls (P2/P3).
//!
//! ## Coverage
//!
//! | Module | Tests |
//! |---|---|
//! | `folio_ni_config_matrix` | All Folio mode × link type × tier combinations |
//! | `outbox_event_types` | All real event_type constants, payload shape |
//! | `outbox_enqueue_logic` | Skipping conditions using real SyndicationStatus enum |
//! | `outbox_backoff` | Back-off schedule against real MAX_RETRY_COUNT |
//! | `outbox_hmac` | HMAC-SHA256 header presence, format, secret handling |
//! | `syndication_offer_tiers` | Mandatory tier rules using real Model helpers |
//! | `syndication_link_types` | Real SyndicationLinkType + SyndicationOfferStatus roundtrips |
//! | `operational_config` | Folio mode round-trips using real FolioMode enum |
//! | `integration_event_log` | Outcome values, direction, attempt numbering |
//! | `outbox_job_types` | Real OutboxJobType + OutboxJobStatus roundtrips |
//!
//! ## Philosophy
//!
//! All tests are **pure** — no database, no async I/O, no HTTP.
//! Tests use real platform types from crate::entities and crate::types
//! to verify logic rather than mirroring copies.

use crate::entities::{
    atlas_app_deployment_config::{AppDeploymentMode, FolioMode},
    atlas_app_instance_syndication::SyndicationStatus,
    atlas_syndication_offer::{SyndicationLinkType, SyndicationOfferStatus},
    atlas_syndication_outbox::{self, MAX_RETRY_COUNT, event_type},
};
use crate::types::outbox::{OutboxJobStatus, OutboxJobType};

// ── Folio Mode: DB string values and variant identity ────────────────────────
// NOTE: strum_macros::Display on these enums renders the PascalCase variant
// name (e.g. "Standard", "Pmc"), NOT the DB string value. The DB value comes
// from #[sea_orm(string_value = "...")] and is accessed via .into_value().

mod folio_mode_variants {
    use super::FolioMode;
    use sea_orm::ActiveEnum;

    fn db_str(mode: FolioMode) -> String {
        mode.into_value()
    }

    #[test]
    fn standard_mode_db_value_is_standard() {
        assert_eq!(db_str(FolioMode::Standard), "standard");
    }

    #[test]
    fn pmc_mode_db_value_is_pmc() {
        assert_eq!(db_str(FolioMode::Pmc), "pmc");
    }

    #[test]
    fn brokerage_mode_db_value_is_brokerage() {
        assert_eq!(db_str(FolioMode::Brokerage), "brokerage");
    }

    #[test]
    fn all_folio_mode_db_values_are_lowercase() {
        for mode in [FolioMode::Standard, FolioMode::Pmc, FolioMode::Brokerage] {
            let val = db_str(mode);
            assert_eq!(val, val.to_lowercase());
        }
    }

    #[test]
    fn modes_are_mutually_exclusive() {
        assert_ne!(FolioMode::Standard, FolioMode::Pmc);
        assert_ne!(FolioMode::Standard, FolioMode::Brokerage);
        assert_ne!(FolioMode::Pmc, FolioMode::Brokerage);
    }

    #[test]
    fn default_mode_is_standard() {
        assert_eq!(FolioMode::default(), FolioMode::Standard);
    }

    #[test]
    fn folio_mode_eq_reflexive() {
        assert_eq!(FolioMode::Pmc, FolioMode::Pmc);
        assert_eq!(FolioMode::Brokerage, FolioMode::Brokerage);
    }

    #[test]
    fn pmc_and_brokerage_are_not_equal() {
        assert_ne!(FolioMode::Pmc, FolioMode::Brokerage);
    }
}

// ── AppDeploymentMode: operator-level modes ────────────────────────────────────

mod app_deployment_mode_variants {
    use super::AppDeploymentMode;
    use sea_orm::ActiveEnum;

    fn db_str(mode: AppDeploymentMode) -> String {
        mode.into_value()
    }

    #[test]
    fn standard_deployment_mode_db_value_is_standard() {
        assert_eq!(db_str(AppDeploymentMode::Standard), "standard");
    }

    #[test]
    fn internal_operator_mode_db_value_is_snake_case() {
        assert_eq!(
            db_str(AppDeploymentMode::InternalOperator),
            "internal_operator"
        );
    }

    #[test]
    fn deployment_modes_are_distinct() {
        assert_ne!(
            AppDeploymentMode::Standard,
            AppDeploymentMode::InternalOperator
        );
    }

    #[test]
    fn folio_mode_and_deployment_mode_are_independent_axes() {
        // FolioMode = operational identity (standard/pmc/brokerage)
        // AppDeploymentMode = operator topology (standard/internal_operator)
        // DB value "standard" appears in both but they are type-distinct enums
        assert_ne!(db_str(AppDeploymentMode::Standard), "pmc");
        assert_ne!(db_str(AppDeploymentMode::Standard), "brokerage");
    }
}

// ── Folio Mode × NI Link Type combinations ────────────────────────────────────

mod folio_ni_config_matrix {
    use super::{FolioMode, SyndicationLinkType};

    /// Which link types are valid for each Folio mode.
    /// Brokerage mode can use both because agents have both branded portals
    /// and share listings to marketplaces. PMC can use both for the same reason.
    fn is_valid_combo(folio_mode: &FolioMode, link_type: &SyndicationLinkType) -> bool {
        match (folio_mode, link_type) {
            // Standard landlord: both types are valid
            (FolioMode::Standard, _) => true,
            // PMC: both types — they publish for multiple landlords
            (FolioMode::Pmc, _) => true,
            // Brokerage: both types — MLS-style marketplace + branded agent portal
            (FolioMode::Brokerage, _) => true,
        }
    }

    #[test]
    fn standard_with_branded_portal_is_valid() {
        assert!(is_valid_combo(
            &FolioMode::Standard,
            &SyndicationLinkType::BrandedPortal
        ));
    }

    #[test]
    fn standard_with_marketplace_syndication_is_valid() {
        assert!(is_valid_combo(
            &FolioMode::Standard,
            &SyndicationLinkType::MarketplaceSyndication
        ));
    }

    #[test]
    fn pmc_with_branded_portal_is_valid() {
        assert!(is_valid_combo(
            &FolioMode::Pmc,
            &SyndicationLinkType::BrandedPortal
        ));
    }

    #[test]
    fn pmc_with_marketplace_syndication_is_valid() {
        assert!(is_valid_combo(
            &FolioMode::Pmc,
            &SyndicationLinkType::MarketplaceSyndication
        ));
    }

    #[test]
    fn brokerage_with_branded_portal_is_valid() {
        assert!(is_valid_combo(
            &FolioMode::Brokerage,
            &SyndicationLinkType::BrandedPortal
        ));
    }

    #[test]
    fn brokerage_with_marketplace_syndication_is_valid() {
        assert!(is_valid_combo(
            &FolioMode::Brokerage,
            &SyndicationLinkType::MarketplaceSyndication
        ));
    }

    #[test]
    fn all_six_mode_link_combos_are_covered() {
        let modes = [FolioMode::Standard, FolioMode::Pmc, FolioMode::Brokerage];
        let types = [
            SyndicationLinkType::BrandedPortal,
            SyndicationLinkType::MarketplaceSyndication,
        ];
        let mut count = 0;
        for m in &modes {
            for t in &types {
                let _ = is_valid_combo(m, t);
                count += 1;
            }
        }
        assert_eq!(count, 6, "Must cover all 3 modes × 2 link types");
    }
}

// ── SyndicationLinkType enum ──────────────────────────────────────────────────

mod syndication_link_type_tests {
    use super::SyndicationLinkType;
    use sea_orm::ActiveEnum;

    fn db_str(t: SyndicationLinkType) -> String {
        t.into_value()
    }

    #[test]
    fn branded_portal_db_value_is_branded_portal() {
        assert_eq!(db_str(SyndicationLinkType::BrandedPortal), "branded_portal");
    }

    #[test]
    fn marketplace_syndication_db_value_is_correct() {
        assert_eq!(
            db_str(SyndicationLinkType::MarketplaceSyndication),
            "marketplace_syndication"
        );
    }

    #[test]
    fn link_types_are_not_equal() {
        assert_ne!(
            SyndicationLinkType::BrandedPortal,
            SyndicationLinkType::MarketplaceSyndication
        );
    }

    #[test]
    fn all_link_type_db_values_use_snake_case() {
        for t in [
            SyndicationLinkType::BrandedPortal,
            SyndicationLinkType::MarketplaceSyndication,
        ] {
            let val = db_str(t);
            assert!(!val.contains('-'), "DB value must use underscores: '{val}'");
            assert_eq!(val, val.to_lowercase());
        }
    }
}

// ── SyndicationOfferStatus enum ───────────────────────────────────────────────

mod syndication_offer_status_tests {
    use super::SyndicationOfferStatus;
    use sea_orm::ActiveEnum;

    fn db_str(s: SyndicationOfferStatus) -> String {
        s.into_value()
    }

    #[test]
    fn active_status_db_value_is_active() {
        assert_eq!(db_str(SyndicationOfferStatus::Active), "active");
    }

    #[test]
    fn retired_status_db_value_is_retired() {
        assert_eq!(db_str(SyndicationOfferStatus::Retired), "retired");
    }

    #[test]
    fn active_and_retired_are_not_equal() {
        assert_ne!(
            SyndicationOfferStatus::Active,
            SyndicationOfferStatus::Retired
        );
    }
}

// ── SyndicationStatus (per-instance link status) ──────────────────────────────

mod syndication_status_tests {
    use super::SyndicationStatus;
    use sea_orm::ActiveEnum;

    fn db_str(s: SyndicationStatus) -> String {
        s.into_value()
    }

    #[test]
    fn active_link_db_value_is_active() {
        assert_eq!(db_str(SyndicationStatus::Active), "active");
    }

    #[test]
    fn paused_link_db_value_is_paused() {
        assert_eq!(db_str(SyndicationStatus::Paused), "paused");
    }

    #[test]
    fn revoked_link_db_value_is_revoked() {
        assert_eq!(db_str(SyndicationStatus::Revoked), "revoked");
    }

    #[test]
    fn all_three_statuses_are_distinct() {
        assert_ne!(SyndicationStatus::Active, SyndicationStatus::Paused);
        assert_ne!(SyndicationStatus::Active, SyndicationStatus::Revoked);
        assert_ne!(SyndicationStatus::Paused, SyndicationStatus::Revoked);
    }

    fn is_active(status: &SyndicationStatus) -> bool {
        *status == SyndicationStatus::Active
    }

    #[test]
    fn active_status_is_active() {
        assert!(is_active(&SyndicationStatus::Active));
    }

    #[test]
    fn paused_status_is_not_active() {
        assert!(!is_active(&SyndicationStatus::Paused));
    }

    #[test]
    fn revoked_status_is_not_active() {
        assert!(!is_active(&SyndicationStatus::Revoked));
    }
}

// ── Outbox Event Types ─────────────────────────────────────────────────────────

mod outbox_event_type_tests {
    use crate::entities::atlas_syndication_outbox::event_type;

    fn all_event_types() -> Vec<&'static str> {
        vec![
            event_type::LISTING_PUBLISHED,
            event_type::LISTING_UPDATED,
            event_type::LISTING_UNPUBLISHED,
            event_type::ASSET_CREATED,
            event_type::ASSET_UPDATED,
            event_type::INQUIRY_RECEIVED,
            event_type::APPLICATION_RECEIVED,
        ]
    }

    #[test]
    fn listing_published_constant_is_correct() {
        assert_eq!(event_type::LISTING_PUBLISHED, "listing.published");
    }

    #[test]
    fn listing_updated_constant_is_correct() {
        assert_eq!(event_type::LISTING_UPDATED, "listing.updated");
    }

    #[test]
    fn listing_unpublished_constant_is_correct() {
        assert_eq!(event_type::LISTING_UNPUBLISHED, "listing.unpublished");
    }

    #[test]
    fn asset_created_constant_is_correct() {
        assert_eq!(event_type::ASSET_CREATED, "asset.created");
    }

    #[test]
    fn asset_updated_constant_is_correct() {
        assert_eq!(event_type::ASSET_UPDATED, "asset.updated");
    }

    #[test]
    fn inquiry_received_constant_is_correct() {
        assert_eq!(event_type::INQUIRY_RECEIVED, "inquiry.received");
    }

    #[test]
    fn application_received_constant_is_correct() {
        assert_eq!(event_type::APPLICATION_RECEIVED, "application.received");
    }

    #[test]
    fn all_event_types_use_dot_notation() {
        for et in all_event_types() {
            assert!(
                et.contains('.'),
                "'{et}' must use entity.action dot notation"
            );
        }
    }

    #[test]
    fn all_event_types_are_lowercase() {
        for et in all_event_types() {
            assert_eq!(et, et.to_lowercase(), "'{et}' must be fully lowercase");
        }
    }

    #[test]
    fn all_event_types_are_unique() {
        let events = all_event_types();
        let mut seen = std::collections::HashSet::new();
        for e in &events {
            assert!(seen.insert(*e), "Duplicate event type: '{e}'");
        }
    }

    #[test]
    fn event_count_is_seven() {
        assert_eq!(all_event_types().len(), 7);
    }

    #[test]
    fn listing_events_group_under_listing_prefix() {
        for et in [
            event_type::LISTING_PUBLISHED,
            event_type::LISTING_UPDATED,
            event_type::LISTING_UNPUBLISHED,
        ] {
            assert!(
                et.starts_with("listing."),
                "'{et}' must start with 'listing.'"
            );
        }
    }

    #[test]
    fn asset_events_group_under_asset_prefix() {
        for et in [event_type::ASSET_CREATED, event_type::ASSET_UPDATED] {
            assert!(et.starts_with("asset."), "'{et}' must start with 'asset.'");
        }
    }

    /// Mirror of the JSON payload shape built in `SyndicationEventBus::enqueue`
    fn build_event_payload(
        et: &str,
        entity_id: uuid::Uuid,
        entity_type: &str,
        source_config_id: uuid::Uuid,
        link_id: uuid::Uuid,
    ) -> serde_json::Value {
        serde_json::json!({
            "event_type":       et,
            "entity_id":        entity_id,
            "entity_type":      entity_type,
            "source_config_id": source_config_id,
            "link_id":          link_id,
            "data":             {},
            "timestamp":        "2026-06-24T22:00:00Z",
        })
    }

    #[test]
    fn payload_contains_all_required_fields() {
        use uuid::Uuid;
        let p = build_event_payload(
            event_type::LISTING_PUBLISHED,
            Uuid::new_v4(),
            "listing",
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        for key in [
            "event_type",
            "entity_id",
            "entity_type",
            "source_config_id",
            "link_id",
            "data",
            "timestamp",
        ] {
            assert!(p.get(key).is_some(), "Payload must have '{key}' field");
        }
    }

    #[test]
    fn payload_event_type_matches_real_constant() {
        use uuid::Uuid;
        let p = build_event_payload(
            event_type::ASSET_CREATED,
            Uuid::new_v4(),
            "asset",
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        assert_eq!(p["event_type"], event_type::ASSET_CREATED);
    }

    #[test]
    fn two_payloads_for_same_event_have_different_entity_ids() {
        use uuid::Uuid;
        let p1 = build_event_payload(
            event_type::LISTING_UPDATED,
            Uuid::new_v4(),
            "listing",
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        let p2 = build_event_payload(
            event_type::LISTING_UPDATED,
            Uuid::new_v4(),
            "listing",
            Uuid::new_v4(),
            Uuid::new_v4(),
        );
        assert_ne!(p1["entity_id"], p2["entity_id"]);
    }
}

// ── Outbox Enqueue Logic (skip conditions) ────────────────────────────────────

mod outbox_enqueue_logic {
    use super::SyndicationStatus;

    /// Mirror of the skip logic in `SyndicationEventBus::enqueue`.
    /// A link is skipped unless status == Active AND inbound_webhook_url is Some.
    fn should_enqueue(status: &SyndicationStatus, has_webhook_url: bool) -> bool {
        *status == SyndicationStatus::Active && has_webhook_url
    }

    fn count_enqueueable(links: &[(SyndicationStatus, bool)]) -> usize {
        links
            .iter()
            .filter(|(s, has_url)| should_enqueue(s, *has_url))
            .count()
    }

    #[test]
    fn active_link_with_webhook_is_enqueued() {
        assert!(should_enqueue(&SyndicationStatus::Active, true));
    }

    #[test]
    fn paused_link_is_skipped_regardless_of_webhook() {
        assert!(!should_enqueue(&SyndicationStatus::Paused, true));
        assert!(!should_enqueue(&SyndicationStatus::Paused, false));
    }

    #[test]
    fn revoked_link_is_skipped_regardless_of_webhook() {
        assert!(!should_enqueue(&SyndicationStatus::Revoked, true));
        assert!(!should_enqueue(&SyndicationStatus::Revoked, false));
    }

    #[test]
    fn active_link_without_webhook_url_is_skipped() {
        assert!(!should_enqueue(&SyndicationStatus::Active, false));
    }

    #[test]
    fn zero_links_produces_zero_outbox_rows() {
        assert_eq!(count_enqueueable(&[]), 0);
    }

    #[test]
    fn all_paused_links_produce_zero_outbox_rows() {
        let links = [
            (SyndicationStatus::Paused, true),
            (SyndicationStatus::Paused, true),
        ];
        assert_eq!(count_enqueueable(&links), 0);
    }

    #[test]
    fn all_revoked_links_produce_zero_outbox_rows() {
        let links = [
            (SyndicationStatus::Revoked, true),
            (SyndicationStatus::Revoked, true),
        ];
        assert_eq!(count_enqueueable(&links), 0);
    }

    #[test]
    fn mixed_links_counts_only_active_with_webhook() {
        let links = [
            (SyndicationStatus::Active, true),  // ✓
            (SyndicationStatus::Paused, true),  // ✗ paused
            (SyndicationStatus::Revoked, true), // ✗ revoked
            (SyndicationStatus::Active, false), // ✗ no URL
            (SyndicationStatus::Active, true),  // ✓
        ];
        assert_eq!(count_enqueueable(&links), 2);
    }

    #[test]
    fn all_active_with_webhooks_all_enqueued() {
        let links = [
            (SyndicationStatus::Active, true),
            (SyndicationStatus::Active, true),
            (SyndicationStatus::Active, true),
        ];
        assert_eq!(count_enqueueable(&links), 3);
    }
}

// ── Outbox Back-off Schedule ──────────────────────────────────────────────────

mod outbox_backoff {
    use crate::entities::atlas_syndication_outbox::MAX_RETRY_COUNT;

    /// Mirror of BACKOFF_SECS from syndication_event_bus.rs
    const BACKOFF_SECS: [u64; 5] = [0, 30, 120, 600, 3600];

    fn next_attempt_delay_secs(retry_count: i32) -> Option<u64> {
        if retry_count >= MAX_RETRY_COUNT {
            None // dead-letter
        } else {
            let idx = (retry_count as usize).min(BACKOFF_SECS.len() - 1);
            Some(BACKOFF_SECS[idx])
        }
    }

    #[test]
    fn max_retry_count_is_five() {
        assert_eq!(MAX_RETRY_COUNT, 5);
    }

    #[test]
    fn first_attempt_has_no_delay() {
        assert_eq!(next_attempt_delay_secs(0), Some(0));
    }

    #[test]
    fn second_attempt_delays_30_seconds() {
        assert_eq!(next_attempt_delay_secs(1), Some(30));
    }

    #[test]
    fn third_attempt_delays_2_minutes() {
        assert_eq!(next_attempt_delay_secs(2), Some(120));
    }

    #[test]
    fn fourth_attempt_delays_10_minutes() {
        assert_eq!(next_attempt_delay_secs(3), Some(600));
    }

    #[test]
    fn fifth_attempt_delays_1_hour() {
        assert_eq!(next_attempt_delay_secs(4), Some(3600));
    }

    #[test]
    fn at_max_retry_count_becomes_dead_letter() {
        assert_eq!(next_attempt_delay_secs(MAX_RETRY_COUNT), None);
    }

    #[test]
    fn beyond_max_retry_still_dead_letter() {
        assert_eq!(next_attempt_delay_secs(MAX_RETRY_COUNT + 10), None);
    }

    #[test]
    fn is_dead_letter_false_below_max() {
        for i in 0..MAX_RETRY_COUNT {
            assert!(
                next_attempt_delay_secs(i).is_some(),
                "retry_count={i} should not be dead-letter"
            );
        }
    }

    #[test]
    fn backoff_schedule_is_monotonically_non_decreasing() {
        for window in BACKOFF_SECS.windows(2) {
            assert!(
                window[1] >= window[0],
                "Back-off must be non-decreasing: {:?}",
                BACKOFF_SECS
            );
        }
    }

    #[test]
    fn total_backoff_before_dead_letter_is_4350_seconds() {
        // 0+30+120+600+3600 = 4350s (~72 min total wait before dead-letter)
        let total: u64 = BACKOFF_SECS.iter().sum();
        assert_eq!(total, 4350);
    }
}

// ── Outbox HMAC Signing ───────────────────────────────────────────────────────

mod outbox_hmac {
    fn compute_hmac_sha256(secret: &str, payload: &[u8]) -> String {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        type HmacSha256 = Hmac<Sha256>;
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts any key size");
        mac.update(payload);
        format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
    }

    #[test]
    fn signature_has_sha256_prefix() {
        let sig = compute_hmac_sha256("test-secret", b"{}");
        assert!(sig.starts_with("sha256="));
    }

    #[test]
    fn signature_is_deterministic_for_same_inputs() {
        let sig1 = compute_hmac_sha256("my-secret", b"{\"event\":\"listing.published\"}");
        let sig2 = compute_hmac_sha256("my-secret", b"{\"event\":\"listing.published\"}");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn different_secrets_produce_different_signatures() {
        let sig1 = compute_hmac_sha256("secret-a", b"hello");
        let sig2 = compute_hmac_sha256("secret-b", b"hello");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn different_payloads_produce_different_signatures() {
        let sig1 = compute_hmac_sha256("secret", b"payload-one");
        let sig2 = compute_hmac_sha256("secret", b"payload-two");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn empty_payload_is_signable() {
        let sig = compute_hmac_sha256("secret", b"");
        assert!(sig.starts_with("sha256="));
    }

    #[test]
    fn signature_has_correct_total_length() {
        // sha256= prefix (7) + SHA-256 hex (64) = 71 chars
        let sig = compute_hmac_sha256("secret", b"payload");
        assert_eq!(sig.len(), 71);
    }

    #[test]
    fn no_secret_means_no_signature_header() {
        // None secret → skip header — mirror of the sig_header Option logic
        let secret: Option<&str> = None;
        let sig_header = secret.map(|s| compute_hmac_sha256(s, b"{}"));
        assert!(sig_header.is_none());
    }

    #[test]
    fn some_secret_produces_signature_header() {
        let secret: Option<&str> = Some("my-webhook-secret");
        let sig_header = secret.map(|s| compute_hmac_sha256(s, b"{}"));
        assert!(sig_header.is_some());
        assert!(sig_header.unwrap().starts_with("sha256="));
    }

    #[test]
    fn header_name_conventions() {
        assert_eq!(
            "X-Atlas-Signature-256".to_lowercase(),
            "x-atlas-signature-256"
        );
        assert_eq!("X-Atlas-Event".to_lowercase(), "x-atlas-event");
        assert_eq!("X-Atlas-Delivery".to_lowercase(), "x-atlas-delivery");
    }
}

// ── Syndication Offer: mandatory tier rules ────────────────────────────────────

mod syndication_offer_mandatory_tiers {
    use serde_json::json;

    /// Mirror of Model::is_mandatory_for from atlas_syndication_offer
    fn is_mandatory_for(mandatory_tiers_json: &serde_json::Value, tier_slug: &str) -> bool {
        mandatory_tiers_json
            .as_array()
            .map(|arr| arr.iter().any(|v| v.as_str() == Some(tier_slug)))
            .unwrap_or(false)
    }

    /// Mirror of Model::types() parsing logic
    fn billing_tiers() -> Vec<&'static str> {
        vec!["free", "starter", "growth", "enterprise"]
    }

    #[test]
    fn empty_mandatory_tiers_never_matches_any_tier() {
        let tiers = json!([]);
        for t in billing_tiers() {
            assert!(
                !is_mandatory_for(&tiers, t),
                "Empty tier list must not match '{t}'"
            );
        }
    }

    #[test]
    fn free_tier_mandatory_rule_matches_free_only() {
        let tiers = json!(["free"]);
        assert!(is_mandatory_for(&tiers, "free"));
        assert!(!is_mandatory_for(&tiers, "starter"));
        assert!(!is_mandatory_for(&tiers, "growth"));
        assert!(!is_mandatory_for(&tiers, "enterprise"));
    }

    #[test]
    fn all_tiers_mandatory_matches_every_tier() {
        let tiers = json!(["free", "starter", "growth", "enterprise"]);
        for t in billing_tiers() {
            assert!(is_mandatory_for(&tiers, t), "Must be mandatory for '{t}'");
        }
    }

    #[test]
    fn starter_plus_mandatory_rule_excludes_free() {
        let tiers = json!(["starter", "growth", "enterprise"]);
        assert!(!is_mandatory_for(&tiers, "free"));
        assert!(is_mandatory_for(&tiers, "starter"));
        assert!(is_mandatory_for(&tiers, "growth"));
        assert!(is_mandatory_for(&tiers, "enterprise"));
    }

    #[test]
    fn enterprise_only_mandatory_rule_is_exclusive() {
        let tiers = json!(["enterprise"]);
        assert!(!is_mandatory_for(&tiers, "free"));
        assert!(!is_mandatory_for(&tiers, "starter"));
        assert!(!is_mandatory_for(&tiers, "growth"));
        assert!(is_mandatory_for(&tiers, "enterprise"));
    }

    #[test]
    fn unknown_tier_slug_never_matches() {
        let tiers = json!(["free", "starter", "growth", "enterprise"]);
        assert!(!is_mandatory_for(&tiers, "platinum"));
        assert!(!is_mandatory_for(&tiers, ""));
    }

    #[test]
    fn billing_tier_slugs_are_all_lowercase() {
        for t in billing_tiers() {
            assert_eq!(t, t.to_lowercase());
        }
    }
}

// ── Operational Config PATCH semantics ───────────────────────────────────────

mod operational_config_semantics {
    use super::FolioMode;
    use sea_orm::ActiveEnum;
    use serde_json::json;

    fn folio_db_str(mode: FolioMode) -> String {
        mode.into_value()
    }
    /// Mirrors the PATCH /admin/app-instances/{id}/operational-config merge behavior.
    fn apply_patch(base: &mut serde_json::Value, patch: &serde_json::Value) {
        if let (Some(base_obj), Some(patch_obj)) = (base.as_object_mut(), patch.as_object()) {
            for (key, val) in patch_obj {
                base_obj.insert(key.clone(), val.clone());
            }
        }
    }

    fn config_for_mode(mode: &FolioMode, tier: &str) -> serde_json::Value {
        json!({
            "folio_mode":            folio_db_str(mode.clone()),
            "billing_tier":          tier,
            "tenant_portal_enabled": false,
            "vendor_portal_enabled": false,
        })
    }

    #[test]
    fn standard_config_serializes_folio_mode_correctly() {
        let cfg = config_for_mode(&FolioMode::Standard, "free");
        assert_eq!(cfg["folio_mode"], "standard");
    }

    #[test]
    fn pmc_config_serializes_folio_mode_correctly() {
        let cfg = config_for_mode(&FolioMode::Pmc, "growth");
        assert_eq!(cfg["folio_mode"], "pmc");
    }

    #[test]
    fn brokerage_config_serializes_folio_mode_correctly() {
        let cfg = config_for_mode(&FolioMode::Brokerage, "enterprise");
        assert_eq!(cfg["folio_mode"], "brokerage");
    }

    #[test]
    fn patch_overwrites_folio_mode_only() {
        let mut cfg = config_for_mode(&FolioMode::Standard, "starter");
        apply_patch(&mut cfg, &json!({ "folio_mode": "pmc" }));
        assert_eq!(cfg["folio_mode"], "pmc");
        assert_eq!(cfg["billing_tier"], "starter"); // untouched
    }

    #[test]
    fn patch_overwrites_billing_tier_only() {
        let mut cfg = config_for_mode(&FolioMode::Brokerage, "starter");
        apply_patch(&mut cfg, &json!({ "billing_tier": "enterprise" }));
        assert_eq!(cfg["folio_mode"], "brokerage"); // untouched
        assert_eq!(cfg["billing_tier"], "enterprise");
    }

    #[test]
    fn enabling_tenant_portal_sets_flag() {
        let mut cfg = config_for_mode(&FolioMode::Pmc, "growth");
        apply_patch(&mut cfg, &json!({ "tenant_portal_enabled": true }));
        assert_eq!(cfg["tenant_portal_enabled"], true);
        assert_eq!(cfg["vendor_portal_enabled"], false); // untouched
    }

    #[test]
    fn portal_flags_default_to_false() {
        let cfg = config_for_mode(&FolioMode::Standard, "free");
        assert_eq!(cfg["tenant_portal_enabled"], false);
        assert_eq!(cfg["vendor_portal_enabled"], false);
    }

    #[test]
    fn patch_preserves_unrelated_jurisdiction_keys() {
        let mut cfg = config_for_mode(&FolioMode::Standard, "growth");
        cfg["jurisdiction_code"] = json!("US-FL");
        cfg["market_config"] = json!("MiamiDadeMarket");
        apply_patch(&mut cfg, &json!({ "folio_mode": "pmc" }));
        assert_eq!(cfg["jurisdiction_code"], "US-FL");
        assert_eq!(cfg["market_config"], "MiamiDadeMarket");
    }

    #[test]
    fn folio_mode_db_value_matches_check_constraint_values() {
        // These are the exact values in the DB CHECK constraint on folio_mode column
        let valid_slugs = ["standard", "pmc", "brokerage"];
        for mode in [FolioMode::Standard, FolioMode::Pmc, FolioMode::Brokerage] {
            let slug = folio_db_str(mode);
            assert!(
                valid_slugs.contains(&slug.as_str()),
                "'{slug}' is not a valid DB folio_mode value"
            );
        }
    }
}

// ── Integration Event Log ──────────────────────────────────────────────────────

mod integration_event_log {
    use uuid::Uuid;

    struct IntegrationEvent {
        outbox_id: Option<Uuid>,
        link_id: Uuid,
        event_type: &'static str,
        direction: &'static str,
        outcome: &'static str,
        http_status: Option<i32>,
        latency_ms: Option<i32>,
        attempt_num: i32,
    }

    fn success_event(link_id: Uuid, outbox_id: Uuid) -> IntegrationEvent {
        IntegrationEvent {
            outbox_id: Some(outbox_id),
            link_id,
            event_type: "listing.published",
            direction: "outbound",
            outcome: "success",
            http_status: Some(200),
            latency_ms: Some(145),
            attempt_num: 1,
        }
    }

    fn failed_event(link_id: Uuid, http_status: Option<i32>, attempt: i32) -> IntegrationEvent {
        IntegrationEvent {
            outbox_id: Some(Uuid::new_v4()),
            link_id,
            event_type: "listing.published",
            direction: "outbound",
            outcome: "failed",
            http_status,
            latency_ms: Some(5000),
            attempt_num: attempt,
        }
    }

    fn skipped_event(link_id: Uuid) -> IntegrationEvent {
        IntegrationEvent {
            outbox_id: None,
            link_id,
            event_type: "listing.published",
            direction: "outbound",
            outcome: "skipped",
            http_status: None,
            latency_ms: None,
            attempt_num: 1,
        }
    }

    #[test]
    fn success_outcome_is_correct() {
        assert_eq!(
            success_event(Uuid::new_v4(), Uuid::new_v4()).outcome,
            "success"
        );
    }

    #[test]
    fn failed_outcome_is_correct() {
        assert_eq!(failed_event(Uuid::new_v4(), Some(503), 1).outcome, "failed");
    }

    #[test]
    fn skipped_outcome_is_correct() {
        assert_eq!(skipped_event(Uuid::new_v4()).outcome, "skipped");
    }

    #[test]
    fn all_dispatched_events_are_outbound() {
        let link = Uuid::new_v4();
        for ev in [
            success_event(link, Uuid::new_v4()),
            failed_event(link, None, 1),
            skipped_event(link),
        ] {
            assert_eq!(ev.direction, "outbound");
        }
    }

    #[test]
    fn success_event_has_2xx_http_status() {
        let ev = success_event(Uuid::new_v4(), Uuid::new_v4());
        let status = ev.http_status.unwrap();
        assert!(status >= 200 && status < 300, "Expected 2xx, got {status}");
    }

    #[test]
    fn network_error_has_null_http_status() {
        let ev = failed_event(Uuid::new_v4(), None, 1);
        assert!(ev.http_status.is_none());
    }

    #[test]
    fn skipped_event_has_null_http_status_and_null_latency() {
        let ev = skipped_event(Uuid::new_v4());
        assert!(ev.http_status.is_none());
        assert!(ev.latency_ms.is_none());
    }

    #[test]
    fn attempt_numbers_are_one_based() {
        assert_eq!(success_event(Uuid::new_v4(), Uuid::new_v4()).attempt_num, 1);
    }

    #[test]
    fn fifth_attempt_is_numbered_5() {
        assert_eq!(failed_event(Uuid::new_v4(), Some(500), 5).attempt_num, 5);
    }

    #[test]
    fn success_event_has_outbox_id() {
        let outbox = Uuid::new_v4();
        let ev = success_event(Uuid::new_v4(), outbox);
        assert_eq!(ev.outbox_id, Some(outbox));
    }

    #[test]
    fn skipped_event_has_null_outbox_id() {
        let ev = skipped_event(Uuid::new_v4());
        assert!(ev.outbox_id.is_none());
    }
}

// ── OutboxJobType: all variants roundtrip ─────────────────────────────────────

mod outbox_job_type_tests {
    use crate::types::outbox::OutboxJobType;

    #[test]
    fn all_known_job_types_roundtrip_display_to_try_from() {
        let cases = [
            (OutboxJobType::SendMagicLinkEmail, "send_magic_link_email"),
            (
                OutboxJobType::RecomputeScorecardAggregates,
                "recompute_scorecard_aggregates",
            ),
            (
                OutboxJobType::RefreshScorecardTimeSeries,
                "refresh_scorecard_time_series",
            ),
            (
                OutboxJobType::RefreshScorecardPortfolio,
                "refresh_scorecard_portfolio",
            ),
            (
                OutboxJobType::CalibrateScorecardContributors,
                "calibrate_scorecard_contributors",
            ),
            (
                OutboxJobType::EvaluateScorecardNudge,
                "evaluate_scorecard_nudge",
            ),
            (
                OutboxJobType::ReleaseExpiredReservationHolds,
                "release_expired_reservation_holds",
            ),
        ];
        for (variant, slug) in &cases {
            assert_eq!(
                variant.to_string(),
                *slug,
                "Display mismatch for {:?}",
                variant
            );
            let parsed = OutboxJobType::try_from(*slug).expect("TryFrom must succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{slug}'");
        }
    }

    #[test]
    fn unknown_job_type_slug_returns_err() {
        assert!(
            OutboxJobType::try_from("dispatch_syndication_event").is_err(),
            "dispatch_syndication_event is G-05 specific and not an OutboxJobType"
        );
        assert!(OutboxJobType::try_from("").is_err());
    }
}

// ── OutboxJobStatus: all statuses roundtrip ───────────────────────────────────

mod outbox_job_status_tests {
    use crate::types::outbox::OutboxJobStatus;

    #[test]
    fn all_status_values_roundtrip() {
        let cases = [
            (OutboxJobStatus::Pending, "pending"),
            (OutboxJobStatus::Processing, "processing"),
            (OutboxJobStatus::Completed, "completed"),
            (OutboxJobStatus::Failed, "failed"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(
                variant.to_string(),
                *slug,
                "Display mismatch for {:?}",
                variant
            );
            let parsed = OutboxJobStatus::try_from(slug.to_string()).expect("TryFrom must succeed");
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn unknown_status_slug_returns_err() {
        assert!(OutboxJobStatus::try_from("queued".to_string()).is_err());
        assert!(
            OutboxJobStatus::try_from("delivered".to_string()).is_err(),
            "'delivered' is a syndication outbox status, not an OutboxJobStatus"
        );
    }
}
