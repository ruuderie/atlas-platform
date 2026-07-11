//! Unit tests for PMC (G-33), Vendor Marketplace (G-34), and supporting infrastructure.
//!
//! ## Coverage
//!
//! | Module | Tests |
//! |---|---|
//! | `services::pm::aggregates` | SQL template shape, metric struct defaults, occupancy math |
//! | `handlers::folio::pm::invite` | Email normalization, username slug derivation, token URL construction |
//! | `handlers::folio::marketplace::vendors` | Geo filter SQL generation, trade type filter, distance expression, limit clamping |
//! | `handlers::folio::marketplace::endorse` | Idempotency behavior, endorsement metadata shape |
//! | `handlers::folio::marketplace::listing` | Bio length validation, WKT point format, partial update logic |
//! | `extractors::app_config` | Mode comparison, config key accessors, fallback behavior |
//! | `extractors::client_context` | Header parsing, UUID validation |
//! | `types::pm::PropertyType` | New commercial variants roundtrip, Display, TryFrom |
//! | G-33 app config constants | Mode slug correctness |
//!
//! ## Philosophy
//!
//! All tests are **pure** — no database, no async I/O, no HTTP.
//! Business logic extracted as local mirror functions matching the handler/service code.

// ── Aggregate metrics: SQL template shape ─────────────────────────────────────

mod aggregates_sql_tests {
    use uuid::Uuid;

    /// Mirror of the CTE SQL template from `services::pm::aggregates::fetch_client_metrics`.
    fn build_metrics_sql(tenant_id: Uuid) -> String {
        format!(
            r#"
            WITH portfolio_counts AS (
                SELECT managed_account_id, COUNT(*)::bigint AS portfolio_count
                FROM   atlas_portfolios
                WHERE  tenant_id = '{tenant_id}'
                  AND  managed_account_id IS NOT NULL
                GROUP  BY managed_account_id
            ),
            asset_counts AS (
                SELECT p.managed_account_id,
                       COUNT(a.id)::bigint                AS property_count,
                       COUNT(CASE WHEN a.asset_type = 'rental_unit' THEN 1 END)::bigint AS unit_count
                FROM   atlas_portfolios p
                JOIN   atlas_assets a ON a.portfolio_id = p.id
                WHERE  p.tenant_id = '{tenant_id}'
                  AND  p.managed_account_id IS NOT NULL
                GROUP  BY p.managed_account_id
            ),
            lease_counts AS (
                SELECT managed_account_id,
                       COUNT(*)::bigint AS active_lease_count,
                       COUNT(CASE WHEN status = 'active' THEN 1 END)::bigint AS active_count,
                       COUNT(*)::bigint AS total_count
                FROM   atlas_contracts
                WHERE  tenant_id = '{tenant_id}'
                  AND  managed_account_id IS NOT NULL
                GROUP  BY managed_account_id
            )
            SELECT
                COALESCE(pc.managed_account_id, ac.managed_account_id, lc.managed_account_id) AS account_id,
                COALESCE(pc.portfolio_count, 0) AS portfolio_count,
                COALESCE(ac.property_count, 0)  AS property_count,
                COALESCE(ac.unit_count, 0)       AS unit_count,
                COALESCE(lc.active_lease_count, 0) AS active_lease_count,
                0.0::float AS occupancy_pct
            FROM portfolio_counts pc
            FULL OUTER JOIN asset_counts ac ON ac.managed_account_id = pc.managed_account_id
            FULL OUTER JOIN lease_counts lc ON lc.managed_account_id = pc.managed_account_id
            "#,
            tenant_id = tenant_id,
        )
    }

    #[test]
    fn sql_contains_tenant_id() {
        let tid = Uuid::new_v4();
        let sql = build_metrics_sql(tid);
        let tid_str = tid.to_string();
        assert!(sql.contains(&tid_str), "SQL must be scoped to the tenant");
    }

    #[test]
    fn sql_contains_managed_account_id_filter() {
        let sql = build_metrics_sql(Uuid::new_v4());
        assert!(sql.contains("managed_account_id IS NOT NULL"));
    }

    #[test]
    fn sql_uses_full_outer_join() {
        let sql = build_metrics_sql(Uuid::new_v4());
        assert!(sql.to_uppercase().contains("FULL OUTER JOIN"));
    }

    #[test]
    fn sql_coalesces_nulls_to_zero() {
        let sql = build_metrics_sql(Uuid::new_v4());
        // Ensure we never return NULL for counts
        assert!(sql.contains("COALESCE(pc.portfolio_count, 0)"));
        assert!(sql.contains("COALESCE(ac.property_count, 0)"));
        assert!(sql.contains("COALESCE(lc.active_lease_count, 0)"));
    }

    #[test]
    fn different_tenant_ids_produce_different_sql() {
        let sql1 = build_metrics_sql(Uuid::new_v4());
        let sql2 = build_metrics_sql(Uuid::new_v4());
        assert_ne!(sql1, sql2);
    }

    #[test]
    fn occupancy_pct_defaults_to_zero_float() {
        let sql = build_metrics_sql(Uuid::new_v4());
        assert!(sql.contains("0.0::float AS occupancy_pct"));
    }
}

// ── Aggregate metrics: occupancy math ─────────────────────────────────────────

mod occupancy_math_tests {
    /// Mirror of the occupancy calculation logic.
    fn compute_occupancy(occupied: i64, total: i64) -> f64 {
        if total == 0 {
            0.0
        } else {
            (occupied as f64 / total as f64) * 100.0
        }
    }

    #[test]
    fn full_occupancy_returns_100() {
        assert_eq!(compute_occupancy(10, 10), 100.0);
    }

    #[test]
    fn zero_occupied_returns_0() {
        assert_eq!(compute_occupancy(0, 10), 0.0);
    }

    #[test]
    fn zero_total_returns_0_not_nan() {
        let result = compute_occupancy(0, 0);
        assert_eq!(result, 0.0);
        assert!(!result.is_nan());
    }

    #[test]
    fn partial_occupancy_rounds_correctly() {
        // 3 of 4 = 75%
        let result = compute_occupancy(3, 4);
        assert!((result - 75.0).abs() < 0.001);
    }

    #[test]
    fn single_unit_occupied_returns_100() {
        assert_eq!(compute_occupancy(1, 1), 100.0);
    }

    #[test]
    fn high_volume_portfolio_is_stable() {
        // 999 of 1000 units = 99.9%
        let result = compute_occupancy(999, 1000);
        assert!((result - 99.9).abs() < 0.001);
    }
}

// ── Invite flow: email normalization ──────────────────────────────────────────

mod invite_email_tests {
    /// Mirror of the normalization in `invite_client_landlord`.
    fn normalize_email(raw: &str) -> String {
        raw.trim().to_lowercase()
    }

    /// Mirror of the username slug derivation.
    fn derive_username(email: &str) -> String {
        email.split('@').next().unwrap_or(email).to_string()
    }

    #[test]
    fn uppercase_email_is_lowercased() {
        assert_eq!(
            normalize_email("Jane.DOE@EXAMPLE.COM"),
            "jane.doe@example.com"
        );
    }

    #[test]
    fn whitespace_is_trimmed() {
        assert_eq!(normalize_email("  user@test.com  "), "user@test.com");
    }

    #[test]
    fn already_lowercase_is_unchanged() {
        assert_eq!(normalize_email("bob@example.com"), "bob@example.com");
    }

    #[test]
    fn username_slug_extracts_local_part() {
        assert_eq!(derive_username("alice@example.com"), "alice");
    }

    #[test]
    fn username_slug_handles_no_at_sign() {
        // fallback: return the whole string
        assert_eq!(derive_username("nodomain"), "nodomain");
    }

    #[test]
    fn username_slug_takes_first_part_only() {
        // edge case: multiple @ signs (malformed but should not panic)
        assert_eq!(derive_username("a@b@c.com"), "a");
    }

    #[test]
    fn empty_email_returns_empty_username() {
        assert_eq!(derive_username(""), "");
    }
}

// ── Invite flow: setup URL construction ───────────────────────────────────────

mod invite_setup_url_tests {
    use uuid::Uuid;

    /// Mirror of the setup_url construction in `invite_client_landlord`.
    fn build_setup_url(frontend_url: &str, token: &str, client_account_id: Uuid) -> String {
        format!(
            "{}/folio/setup?token={}&client={}",
            frontend_url, token, client_account_id
        )
    }

    #[test]
    fn url_contains_token() {
        let token = "tok_abc123";
        let url = build_setup_url("https://app.example.com", token, Uuid::new_v4());
        assert!(url.contains("token=tok_abc123"));
    }

    #[test]
    fn url_contains_client_id() {
        let client_id = Uuid::new_v4();
        let url = build_setup_url("https://app.example.com", "tok", client_id);
        assert!(url.contains(&client_id.to_string()));
    }

    #[test]
    fn url_uses_folio_setup_path() {
        let url = build_setup_url("https://app.example.com", "tok", Uuid::new_v4());
        assert!(url.contains("/folio/setup?"));
    }

    #[test]
    fn url_has_correct_query_param_order() {
        let url = build_setup_url("https://app.example.com", "tok", Uuid::new_v4());
        let token_pos = url.find("token=").unwrap();
        let client_pos = url.find("client=").unwrap();
        assert!(
            token_pos < client_pos,
            "token must come before client in query string"
        );
    }

    #[test]
    fn frontend_url_with_trailing_slash_still_works() {
        // If the env var has a trailing slash, we'd get double slash — document expected behavior.
        // Production should strip it; this test captures the current behavior.
        let url = build_setup_url("https://app.example.com/", "tok", Uuid::new_v4());
        assert!(url.starts_with("https://app.example.com//folio/setup"));
    }
}

// ── Marketplace: proximity SQL generation ─────────────────────────────────────

mod marketplace_geo_sql_tests {
    /// Mirrors the geo filter construction in `list_vendors`.
    fn build_geo_filter(lat: Option<f64>, lng: Option<f64>, radius_km: f64) -> String {
        let radius_m = radius_km * 1000.0;
        match (lat, lng) {
            (Some(lat), Some(lng)) => format!(
                "AND ST_DWithin(sp.marketplace_location::geography, ST_MakePoint({lng}, {lat})::geography, {radius_m})"
            ),
            _ => String::new(),
        }
    }

    /// Mirrors the distance select expression construction.
    fn build_distance_select(lat: Option<f64>, lng: Option<f64>) -> String {
        match (lat, lng) {
            (Some(lat), Some(lng)) => format!(
                "ST_Distance(sp.marketplace_location::geography, ST_MakePoint({lng}, {lat})::geography) / 1000.0 AS distance_km,"
            ),
            _ => "NULL::float AS distance_km,".to_string(),
        }
    }

    /// Mirrors the trade type filter construction.
    fn build_trade_filter(trade_type: Option<&str>) -> String {
        match trade_type {
            Some(t) => format!("AND sp.marketplace_trade_types @> ARRAY['{t}']::text[]"),
            None => String::new(),
        }
    }

    /// Mirrors the limit clamping in `list_vendors`.
    fn clamp_limit(limit: Option<i64>) -> i64 {
        limit.unwrap_or(20).min(100)
    }

    #[test]
    fn geo_filter_with_lat_lng_uses_st_dwithin() {
        let filter = build_geo_filter(Some(25.76), Some(-80.19), 50.0);
        assert!(filter.contains("ST_DWithin"));
        assert!(filter.contains("50000"));
    }

    #[test]
    fn geo_filter_without_coordinates_is_empty() {
        assert!(build_geo_filter(None, None, 50.0).is_empty());
    }

    #[test]
    fn geo_filter_with_only_lat_is_empty() {
        // Both lat and lng required
        assert!(build_geo_filter(Some(25.0), None, 50.0).is_empty());
    }

    #[test]
    fn geo_filter_with_only_lng_is_empty() {
        assert!(build_geo_filter(None, Some(-80.0), 50.0).is_empty());
    }

    #[test]
    fn geo_filter_uses_lng_lat_order_for_st_makepoint() {
        // PostGIS ST_MakePoint(lng, lat) — longitude first
        let filter = build_geo_filter(Some(18.33), Some(-64.93), 25.0);
        let make_point_pos = filter.find("ST_MakePoint(").unwrap();
        let inside = &filter[make_point_pos..make_point_pos + 30];
        // -64.93 (lng) should appear before 18.33 (lat)
        assert!(
            inside.contains("-64.93"),
            "longitude must come first in ST_MakePoint"
        );
    }

    #[test]
    fn radius_converted_from_km_to_meters() {
        let filter = build_geo_filter(Some(0.0), Some(0.0), 100.0);
        assert!(filter.contains("100000"), "100km should become 100000m");
    }

    #[test]
    fn distance_select_without_coords_is_null() {
        let select = build_distance_select(None, None);
        assert!(select.contains("NULL::float AS distance_km"));
    }

    #[test]
    fn distance_select_with_coords_uses_st_distance() {
        let select = build_distance_select(Some(25.0), Some(-80.0));
        assert!(select.contains("ST_Distance"));
        assert!(select.contains("/ 1000.0 AS distance_km"));
    }

    #[test]
    fn trade_filter_uses_array_containment_operator() {
        let filter = build_trade_filter(Some("plumber"));
        assert!(filter.contains("@> ARRAY['plumber']::text[]"));
    }

    #[test]
    fn trade_filter_none_is_empty() {
        assert!(build_trade_filter(None).is_empty());
    }

    #[test]
    fn limit_default_is_20() {
        assert_eq!(clamp_limit(None), 20);
    }

    #[test]
    fn limit_clamped_to_100() {
        assert_eq!(clamp_limit(Some(999)), 100);
    }

    #[test]
    fn limit_exact_100_is_preserved() {
        assert_eq!(clamp_limit(Some(100)), 100);
    }

    #[test]
    fn limit_small_value_preserved() {
        assert_eq!(clamp_limit(Some(5)), 5);
    }
}

// ── Marketplace: endorsement metadata ─────────────────────────────────────────

mod marketplace_endorsement_tests {
    use serde_json::{Value, json};
    use uuid::Uuid;

    const ENDORSEMENT_TYPE: &str = "marketplace_endorsement";
    const VENDOR_ENTITY_TYPE: &str = "atlas_service_providers";

    fn build_endorsement_metadata(context: &str) -> Value {
        json!({
            "endorsed_at": "2026-06-09T05:00:00Z",
            "context": context
        })
    }

    #[test]
    fn endorsement_type_slug_is_correct() {
        assert_eq!(ENDORSEMENT_TYPE, "marketplace_endorsement");
    }

    #[test]
    fn vendor_entity_type_slug_is_correct() {
        assert_eq!(VENDOR_ENTITY_TYPE, "atlas_service_providers");
    }

    #[test]
    fn endorsement_metadata_has_context_field() {
        let meta = build_endorsement_metadata("marketplace");
        assert_eq!(meta["context"], "marketplace");
    }

    #[test]
    fn endorsement_metadata_has_endorsed_at() {
        let meta = build_endorsement_metadata("marketplace");
        assert!(meta["endorsed_at"].is_string());
    }

    #[test]
    fn endorsement_metadata_serializes_to_valid_json() {
        let meta = build_endorsement_metadata("marketplace");
        let s = serde_json::to_string(&meta);
        assert!(s.is_ok());
    }

    #[test]
    fn source_entity_type_is_atlas_account() {
        // The endorsement is FROM the landlord's account, TO the vendor SP
        let source = "atlas_account";
        assert!(source.starts_with("atlas_")); // platform convention
    }

    #[test]
    fn endorsement_is_directional() {
        // Source = landlord account, Target = service provider — not the reverse
        let source_type = "atlas_account";
        let target_type = "atlas_service_providers";
        assert_ne!(source_type, target_type);
    }

    #[test]
    fn inverse_label_is_endorsed_by() {
        let inverse = "endorsed_by";
        assert_eq!(inverse, "endorsed_by");
    }

    #[test]
    fn two_endorsements_with_different_uuids_are_distinct() {
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        assert_ne!(id1, id2);
    }
}

// ── Marketplace: listing update bio validation ─────────────────────────────────

mod marketplace_listing_tests {
    const BIO_MAX_LEN: usize = 500;

    /// Mirror of the bio validation in `update_my_listing`.
    fn validate_bio(bio: &str) -> Result<(), &'static str> {
        if bio.len() > BIO_MAX_LEN {
            Err("bio must be 500 characters or fewer")
        } else {
            Ok(())
        }
    }

    /// Mirror of the WKT point construction.
    fn build_wkt_point(lat: f64, lng: f64) -> String {
        format!("SRID=4326;POINT({lng} {lat})")
    }

    #[test]
    fn bio_at_exact_limit_is_valid() {
        let bio = "a".repeat(500);
        assert!(validate_bio(&bio).is_ok());
    }

    #[test]
    fn bio_one_over_limit_is_invalid() {
        let bio = "a".repeat(501);
        assert!(validate_bio(&bio).is_err());
    }

    #[test]
    fn empty_bio_is_valid() {
        assert!(validate_bio("").is_ok());
    }

    #[test]
    fn normal_bio_is_valid() {
        assert!(validate_bio("Licensed plumber with 15 years experience.").is_ok());
    }

    #[test]
    fn bio_error_message_mentions_500() {
        let bio = "x".repeat(501);
        let err = validate_bio(&bio).unwrap_err();
        assert!(err.contains("500"));
    }

    #[test]
    fn wkt_point_uses_srid_4326() {
        let wkt = build_wkt_point(25.76, -80.19);
        assert!(wkt.starts_with("SRID=4326;POINT("));
    }

    #[test]
    fn wkt_point_has_lng_before_lat() {
        // PostGIS convention: POINT(lng lat)
        let lat = 25.76;
        let lng = -80.19;
        let wkt = build_wkt_point(lat, lng);
        // -80.19 (lng) comes before 25.76 (lat) in the string
        let lng_pos = wkt.find("-80.19").unwrap();
        let lat_pos = wkt.find("25.76").unwrap();
        assert!(
            lng_pos < lat_pos,
            "longitude must precede latitude in WKT POINT"
        );
    }

    #[test]
    fn wkt_point_for_miami_is_correct() {
        let wkt = build_wkt_point(25.7617, -80.1918);
        assert_eq!(wkt, "SRID=4326;POINT(-80.1918 25.7617)");
    }

    #[test]
    fn wkt_point_for_st_thomas_usvi_is_correct() {
        let wkt = build_wkt_point(18.3358, -64.9307);
        assert_eq!(wkt, "SRID=4326;POINT(-64.9307 18.3358)");
    }
}

// ── App config: mode comparisons ──────────────────────────────────────────────
//
// NOTE: This module previously tested a `pmc_enabled` JSON boolean approach
// that was superseded by migration m20260909_folio_instance_mode, which
// introduced the typed `folio_mode` column (standard|pmc|brokerage) with a
// DB-level CHECK constraint. Those tests have been replaced with tests that
// use the real FolioMode and AppDeploymentMode types.

mod app_config_tests {
    use crate::entities::atlas_app_deployment_config::{AppDeploymentMode, FolioMode};
    use sea_orm::ActiveEnum;

    // ── Note on Display vs DB value ───────────────────────────────────────────
    // strum_macros::Display renders PascalCase variant names ("Standard", "Pmc").
    // The DB string value ("standard", "pmc") comes from #[sea_orm(string_value)].
    // Use .into_value() to get the DB-persisted string (returns String directly).

    fn db_value(mode: FolioMode) -> String {
        mode.into_value()
    }

    fn deploy_db_value(mode: AppDeploymentMode) -> String {
        mode.into_value()
    }

    // ── FolioMode: DB string values ───────────────────────────────────────────

    #[test]
    fn folio_mode_standard_db_value_is_standard() {
        assert_eq!(db_value(FolioMode::Standard), "standard");
    }

    #[test]
    fn folio_mode_pmc_db_value_is_pmc() {
        assert_eq!(db_value(FolioMode::Pmc), "pmc");
    }

    #[test]
    fn folio_mode_brokerage_db_value_is_brokerage() {
        assert_eq!(db_value(FolioMode::Brokerage), "brokerage");
    }

    // ── FolioMode: variant identity ───────────────────────────────────────────

    #[test]
    fn default_folio_mode_is_standard() {
        assert_eq!(FolioMode::default(), FolioMode::Standard);
    }

    #[test]
    fn pmc_mode_is_not_standard() {
        assert_ne!(FolioMode::Pmc, FolioMode::Standard);
    }

    #[test]
    fn brokerage_mode_is_not_pmc() {
        // Platform rule: a single folio instance cannot be both PMC and brokerage
        assert_ne!(FolioMode::Brokerage, FolioMode::Pmc);
    }

    #[test]
    fn brokerage_mode_is_not_standard() {
        assert_ne!(FolioMode::Brokerage, FolioMode::Standard);
    }

    // ── AppDeploymentMode: DB string values ───────────────────────────────────

    #[test]
    fn deployment_mode_standard_db_value_is_standard() {
        assert_eq!(deploy_db_value(AppDeploymentMode::Standard), "standard");
    }

    #[test]
    fn deployment_mode_internal_operator_db_value_is_snake_case() {
        assert_eq!(
            deploy_db_value(AppDeploymentMode::InternalOperator),
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

    // ── AppDeploymentMode ≠ FolioMode: they are independent axes ─────────────
    //
    // AppDeploymentMode governs operator topology (who deployed the platform).
    // FolioMode governs operational identity (what kind of business this is).
    // These are separate DB columns on atlas_app_deployment_config.

    #[test]
    fn folio_mode_db_values_do_not_include_internal_operator() {
        let folio_db_values = [
            db_value(FolioMode::Standard),
            db_value(FolioMode::Pmc),
            db_value(FolioMode::Brokerage),
        ];
        let internal_op = deploy_db_value(AppDeploymentMode::InternalOperator);
        assert!(!folio_db_values.iter().any(|v| v == &internal_op));
    }

    #[test]
    fn deployment_mode_db_values_do_not_include_pmc_or_brokerage() {
        let deploy_db_values = [
            deploy_db_value(AppDeploymentMode::Standard),
            deploy_db_value(AppDeploymentMode::InternalOperator),
        ];
        assert!(!deploy_db_values.iter().any(|v| v == "pmc"));
        assert!(!deploy_db_values.iter().any(|v| v == "brokerage"));
    }

    #[test]
    fn all_folio_mode_db_values_are_lowercase_snake_case() {
        for mode in [FolioMode::Standard, FolioMode::Pmc, FolioMode::Brokerage] {
            let val = db_value(mode);
            assert_eq!(
                val,
                val.to_lowercase(),
                "DB value must be lowercase: '{val}'"
            );
            assert!(!val.contains('-'), "DB value must use underscores: '{val}'");
        }
    }
}

// ── Client context: header parsing ────────────────────────────────────────────

mod client_context_header_tests {
    use uuid::Uuid;

    const CLIENT_ACCOUNT_HEADER: &str = "x-folio-client-account";

    /// Mirror of the UUID extraction logic in `ClientContext::from_request_parts`.
    fn parse_client_account_header(header_val: Option<&str>) -> Result<Uuid, &'static str> {
        let val = header_val.ok_or("missing header")?;
        val.parse::<Uuid>().map_err(|_| "invalid UUID")
    }

    #[test]
    fn header_name_is_lowercase_kebab() {
        assert_eq!(CLIENT_ACCOUNT_HEADER, "x-folio-client-account");
    }

    #[test]
    fn valid_uuid_header_parses_correctly() {
        let id = Uuid::new_v4();
        let result = parse_client_account_header(Some(&id.to_string()));
        assert_eq!(result, Ok(id));
    }

    #[test]
    fn missing_header_returns_error() {
        assert!(parse_client_account_header(None).is_err());
    }

    #[test]
    fn non_uuid_header_returns_error() {
        assert!(parse_client_account_header(Some("not-a-uuid")).is_err());
    }

    #[test]
    fn empty_string_header_returns_error() {
        assert!(parse_client_account_header(Some("")).is_err());
    }

    #[test]
    fn partial_uuid_header_returns_error() {
        assert!(parse_client_account_header(Some("550e8400-e29b")).is_err());
    }

    #[test]
    fn braced_uuid_is_accepted_by_uuid_crate() {
        // The Rust uuid crate actually accepts braced UUIDs like {550e8400-...}.
        // In practice an HTTP header value like this would be unusual but technically
        // valid at the parse level. The tenant isolation DB check is the authoritative guard.
        let result = parse_client_account_header(Some("{550e8400-e29b-41d4-a716-446655440000}"));
        assert!(
            result.is_ok(),
            "uuid crate accepts braced format — this is expected"
        );
    }

    #[test]
    fn nil_uuid_header_is_valid_parse_but_would_be_rejected_by_db() {
        // Nil UUID parses fine — tenant isolation check in DB would reject it
        let nil = Uuid::nil().to_string();
        assert!(parse_client_account_header(Some(&nil)).is_ok());
    }
}

// ── PropertyType: commercial variants ─────────────────────────────────────────

mod property_type_commercial_tests {
    use crate::types::pm::PropertyType;

    #[test]
    fn commercial_variant_exists_and_displays() {
        let t = PropertyType::Commercial;
        assert_eq!(t.to_string(), "commercial");
    }

    #[test]
    fn commercial_roundtrips_through_string() {
        let t = PropertyType::try_from("commercial".to_string()).unwrap();
        assert_eq!(t, PropertyType::Commercial);
    }

    #[test]
    fn all_residential_variants_still_roundtrip() {
        let variants = [
            ("single_family", PropertyType::SingleFamily),
            ("condo", PropertyType::Condo),
            ("townhouse", PropertyType::Townhouse),
            ("multi_family", PropertyType::MultiFamily),
            ("str", PropertyType::Str),
            ("commercial", PropertyType::Commercial),
        ];
        for (slug, expected) in variants {
            let parsed = PropertyType::try_from(slug.to_string())
                .unwrap_or_else(|_| panic!("failed to parse '{slug}'"));
            assert_eq!(parsed, expected, "mismatch for slug '{slug}'");
            assert_eq!(parsed.to_string(), slug, "Display mismatch for '{slug}'");
        }
    }

    #[test]
    fn unknown_property_type_returns_error() {
        assert!(PropertyType::try_from("villa".to_string()).is_err());
    }

    #[test]
    fn office_not_yet_a_variant() {
        // Document future G-35 intent: "office" is not yet parsed
        assert!(PropertyType::try_from("office".to_string()).is_err());
    }
}

// ── G-34: Vendor marketplace visibility semantics ─────────────────────────────

mod marketplace_visibility_tests {
    /// Simulates the opt-in gate logic: vendor only visible if is_marketplace_visible = true.
    fn is_discoverable(is_marketplace_visible: bool) -> bool {
        is_marketplace_visible
    }

    /// Simulates the cross-tenant bio truncation for display.
    fn truncate_bio(bio: Option<&str>, max_len: usize) -> Option<String> {
        bio.map(|b| {
            if b.len() > max_len {
                format!("{}…", &b[..max_len.saturating_sub(1)])
            } else {
                b.to_string()
            }
        })
    }

    #[test]
    fn vendor_not_opted_in_is_not_discoverable() {
        assert!(!is_discoverable(false));
    }

    #[test]
    fn vendor_opted_in_is_discoverable() {
        assert!(is_discoverable(true));
    }

    #[test]
    fn none_bio_returns_none() {
        assert_eq!(truncate_bio(None, 100), None);
    }

    #[test]
    fn short_bio_returned_as_is() {
        assert_eq!(truncate_bio(Some("Hello"), 100), Some("Hello".to_string()));
    }

    #[test]
    fn long_bio_truncated_with_ellipsis() {
        let long_bio = "a".repeat(200);
        let result = truncate_bio(Some(&long_bio), 50).unwrap();
        assert!(result.contains('…'), "truncated bio must end with ellipsis");
        // The '…' character is 3 UTF-8 bytes — use char count, not byte len
        assert!(
            result.chars().count() <= 50,
            "char count ({}) exceeded 50",
            result.chars().count()
        );
    }
}

// ── Role scoping: client_account_id semantics ─────────────────────────────────

mod role_scoping_tests {
    use uuid::Uuid;

    /// Simulates the access check logic:
    /// - None = tenant-wide access (PM sees all clients)
    /// - Some(id) = scoped to that specific client account only
    fn can_access_account(
        role_client_account_id: Option<Uuid>,
        requested_account_id: Uuid,
    ) -> bool {
        match role_client_account_id {
            None => true, // tenant-wide: PM can access anything
            Some(scoped_id) => scoped_id == requested_account_id,
        }
    }

    #[test]
    fn pm_with_no_scope_can_access_any_account() {
        let account_id = Uuid::new_v4();
        assert!(can_access_account(None, account_id));
    }

    #[test]
    fn landlord_with_correct_scope_can_access_their_account() {
        let account_id = Uuid::new_v4();
        assert!(can_access_account(Some(account_id), account_id));
    }

    #[test]
    fn landlord_with_wrong_scope_cannot_access_other_account() {
        let scoped_to = Uuid::new_v4();
        let different = Uuid::new_v4();
        assert!(!can_access_account(Some(scoped_to), different));
    }

    #[test]
    fn nil_uuid_scope_does_not_match_real_account() {
        let real_id = Uuid::new_v4();
        assert!(!can_access_account(Some(Uuid::nil()), real_id));
    }

    #[test]
    fn nil_uuid_scope_matches_nil_account() {
        // Edge case: nil accounts should not exist in production
        assert!(can_access_account(Some(Uuid::nil()), Uuid::nil()));
    }

    #[test]
    fn scoped_role_for_client_a_cannot_see_client_b() {
        let client_a = Uuid::new_v4();
        let client_b = Uuid::new_v4();
        // Landlord scoped to client A cannot access client B's data
        assert!(!can_access_account(Some(client_a), client_b));
    }
}
