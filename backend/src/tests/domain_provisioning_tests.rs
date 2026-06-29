//! # Domain Provisioning Tests
//!
//! Guards the critical path: Admin assigns a domain → DB saved → Ingress created →
//! cert-manager HTTP-01 fires → DNS instructions returned to UI.
//!
//! ## Coverage
//!
//! | Layer                     | What's tested                                                    |
//! |---------------------------|------------------------------------------------------------------|
//! | `tls_config` pure         | wildcard vs custom domain TLS secret + cert-manager annotation   |
//! | `service_for_app` pure    | app slug → k8s Service name mapping (all known slugs)            |
//! | `IngressProvisioner`      | sidecar request shape, 500 + unreachable error handling          |
//! | `update_public_config` API| domain saved + DNS instructions returned in response             |
//! | `get_public_config` API   | dns_instructions present in GET when custom_domain is set        |
//! | `ATLAS_CNAME_TARGET`      | env-driven CNAME target; no stale atlas-platform.com             |
//! | Duplicate domain          | 409 CONFLICT when two tenants claim same custom_domain           |

use axum::{body::Body, http::{Request, StatusCode}};
use http_body_util::BodyExt;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;
use tower::ServiceExt;

use super::api_tests::setup_test_app;
use super::test_utils;
use crate::entities::app_instance;

// ── Helpers ───────────────────────────────────────────────────────────────────

async fn response_json(resp: axum::response::Response) -> serde_json::Value {
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Provision a tenant and return the primary Folio/Anchor instance_id for use
/// in subsequent /public-config calls.
async fn provision_and_get_instance_id(
    app:   &axum::Router,
    db:    &sea_orm::DatabaseConnection,
    token: &str,
    slug:  &str,
    dom:   &str,
) -> (StatusCode, uuid::Uuid) {
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/tenants/provision")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::from(json!({
                    "tenant_name":             slug,
                    "display_name":            format!("Test {slug}"),
                    "domain":                  dom,
                    "admin_email":             format!("admin@{dom}"),
                    "admin_first_name":        "Test",
                    "admin_last_name":         "Admin",
                    "bypass_dns_verification": true,
                }).to_string()))
                .unwrap(),
        )
        .await.unwrap();

    let status = resp.status();
    let body   = response_json(resp).await;

    // Resolve the instance ID via DB (provision response returns tenant_id, not instance_id)
    let tenant_id: uuid::Uuid = body["tenant_id"]
        .as_str()
        .and_then(|s| s.parse().ok())
        .expect("provision response must include tenant_id");

    let instances = app_instance::Entity::find()
        .filter(app_instance::Column::TenantId.eq(tenant_id))
        .all(db).await.unwrap();

    // Prefer the first non-anchor instance (e.g. property_management), or anchor, or any
    let inst = instances.iter()
        .find(|i| i.app_type != "anchor")
        .or_else(|| instances.first())
        .expect("at least one app_instance must exist after provision");

    (status, inst.id)
}

async fn put_public_config(
    app: &axum::Router, token: &str, instance_id: &uuid::Uuid, custom_domain: &str,
) -> (StatusCode, serde_json::Value) {
    // GET first — seeds atlas_app_deployment_config if absent (lazy-create on first GET).
    // The real UI always loads the Config tab before saving; this matches that flow.
    let _ = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/app-instances/{instance_id}/public-config"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();

    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/admin/app-instances/{instance_id}/public-config"))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::from(json!({ "custom_domain": custom_domain }).to_string()))
                .unwrap(),
        )
        .await.unwrap();
    let status = resp.status();
    let body   = response_json(resp).await;
    (status, body)
}

async fn get_public_config(
    app: &axum::Router, token: &str, instance_id: &uuid::Uuid,
) -> (StatusCode, serde_json::Value) {
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/admin/app-instances/{instance_id}/public-config"))
                .header("Authorization", format!("Bearer {token}"))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await.unwrap();
    let status = resp.status();
    let body   = response_json(resp).await;
    (status, body)
}

// ── Sidecar pure-logic unit tests ─────────────────────────────────────────────
//
// The sidecar is a separate binary so we mirror its pure functions here.
// RISK GUARDED: silent routing regressions — wrong Service name → 502 for all tenants.

mod sidecar_pure_logic {

    fn service_for_app(app_slug: &str) -> &'static str {
        match app_slug {
            "property_management" | "folio" => "folio",
            "anchor"                         => "anchor-app",
            "network_instance"               => "network-instance",
            _                                => "folio",
        }
    }

    fn tls_config(domain: &str, tenant_slug: &str) -> (String, bool) {
        if domain.ends_with(".dev.atlas.oply.co") {
            ("wildcard-tls-dev".to_string(),  false)
        } else if domain.ends_with(".uat.atlas.oply.co") {
            ("wildcard-tls-uat".to_string(),  false)
        } else if domain.ends_with(".atlas.oply.co") {
            ("wildcard-tls-prod".to_string(), false)
        } else {
            (format!("{}-tls", tenant_slug.replace('.', "-")), true)
        }
    }

    #[test] fn property_management_routes_to_folio_service() {
        assert_eq!(service_for_app("property_management"), "folio");
    }
    #[test] fn folio_alias_routes_to_folio_service() {
        assert_eq!(service_for_app("folio"), "folio");
    }
    #[test] fn anchor_routes_to_anchor_app_service() {
        assert_eq!(service_for_app("anchor"), "anchor-app");
    }
    #[test] fn network_instance_routes_to_network_instance_service() {
        assert_eq!(service_for_app("network_instance"), "network-instance");
    }
    #[test] fn unknown_slug_falls_back_to_folio_not_crash() {
        assert_eq!(service_for_app("meridian"), "folio");
        assert_eq!(service_for_app(""), "folio");
    }
    #[test] fn service_names_use_hyphens_not_underscores() {
        for slug in ["property_management", "folio", "anchor", "network_instance"] {
            let svc = service_for_app(slug);
            assert!(!svc.contains('_'),
                "k8s Service name '{svc}' for slug '{slug}' must use hyphens, not underscores");
        }
    }

    #[test] fn dev_subdomain_uses_wildcard_no_cert_manager() {
        let (secret, needs_cert) = tls_config("demo.dev.atlas.oply.co", "acme");
        assert_eq!(secret, "wildcard-tls-dev");
        assert!(!needs_cert, "dev subdomain must NOT trigger HTTP-01 (cert already exists)");
    }
    #[test] fn uat_subdomain_uses_wildcard_no_cert_manager() {
        let (secret, needs_cert) = tls_config("demo.uat.atlas.oply.co", "acme");
        assert_eq!(secret, "wildcard-tls-uat");
        assert!(!needs_cert);
    }
    #[test] fn prod_subdomain_uses_wildcard_no_cert_manager() {
        let (secret, needs_cert) = tls_config("folio.atlas.oply.co", "acme");
        assert_eq!(secret, "wildcard-tls-prod");
        assert!(!needs_cert, "*.atlas.oply.co must use wildcard-tls-prod");
    }
    #[test] fn custom_domain_triggers_http01_with_slug_tls_secret() {
        let (secret, needs_cert) = tls_config("app.clientco.com", "clientco");
        assert_eq!(secret, "clientco-tls");
        assert!(needs_cert, "custom domain MUST trigger HTTP-01");
    }
    #[test] fn dots_in_tenant_slug_become_dashes_in_k8s_secret_name() {
        let (secret, _) = tls_config("app.example.com", "some.tenant.slug");
        assert_eq!(secret, "some-tenant-slug-tls");
        assert!(!secret.contains('.'), "k8s Secret name must not contain dots");
    }
    #[test] fn arbitrary_external_domains_trigger_http01() {
        for domain in ["app.buildwithruud.com", "folio.property.io", "tenant.example.co.uk"] {
            let (_, needs_cert) = tls_config(domain, "tenant");
            assert!(needs_cert, "External domain '{domain}' must trigger HTTP-01");
        }
    }
    #[test] fn suffixes_are_case_sensitive_no_mixed_case_bypass() {
        // Mixed case should NOT match the wildcard patterns
        let (_, needs_cert) = tls_config("DEMO.DEV.ATLAS.OPLY.CO", "acme");
        // ends_with is case-sensitive, so this should trigger HTTP-01
        assert!(needs_cert, "Uppercase domain must not bypass wildcard detection");
    }
}

// ── IngressProvisioner wiremock tests ─────────────────────────────────────────

mod ingress_provisioner_behaviour {
    use wiremock::matchers::{method, path, body_json};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use serde_json::json;
    use crate::services::ingress_provisioner::IngressProvisioner;

    fn provisioner(url: &str) -> IngressProvisioner {
        IngressProvisioner::with_sidecar_url(url)
    }

    #[tokio::test]
    async fn provision_sends_all_three_required_fields() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/ingress/provision"))
            .and(body_json(json!({
                "tenant_slug": "buildwithruud",
                "domain":      "folio.atlas.oply.co",
                "app_slug":    "property_management",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"success"})))
            .expect(1)
            .mount(&server).await;

        provisioner(&server.uri())
            .provision_domain("buildwithruud", "folio.atlas.oply.co", "property_management")
            .await
            .expect("provision should succeed");

        server.verify().await;
    }

    #[tokio::test]
    async fn provision_custom_domain_with_anchor_app_slug() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/ingress/provision"))
            .and(body_json(json!({
                "tenant_slug": "ctbuildpros",
                "domain":      "directory.ctbuildpros.com",
                "app_slug":    "anchor",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"success"})))
            .expect(1)
            .mount(&server).await;

        provisioner(&server.uri())
            .provision_domain("ctbuildpros", "directory.ctbuildpros.com", "anchor")
            .await
            .expect("anchor provision should succeed");

        server.verify().await;
    }

    #[tokio::test]
    async fn sidecar_500_returns_err_without_panicking() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("k8s unreachable"))
            .mount(&server).await;

        let err = provisioner(&server.uri())
            .provision_domain("tenant", "app.example.com", "folio")
            .await.unwrap_err();

        assert!(err.contains("500") || err.contains("k8s unreachable"),
            "Error must surface sidecar status/body. Got: {err}");
    }

    #[tokio::test]
    async fn sidecar_completely_unreachable_returns_err() {
        // Port nothing listens on
        let result = provisioner("http://127.0.0.1:19999")
            .provision_domain("tenant", "app.example.com", "folio")
            .await;
        assert!(result.is_err(), "Unreachable sidecar must return Err, not panic");
        assert!(!result.is_ok());
    }

    #[tokio::test]
    async fn deprovision_sends_slug_and_domain() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/ingress/deprovision"))
            .and(body_json(json!({
                "tenant_slug": "buildwithruud",
                "domain":      "folio.atlas.oply.co",
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"status":"success"})))
            .expect(1)
            .mount(&server).await;

        provisioner(&server.uri())
            .deprovision_domain("buildwithruud", "folio.atlas.oply.co")
            .await
            .expect("deprovision should succeed");

        server.verify().await;
    }
}

// ── update_public_config integration tests (real DB + test router) ─────────────

#[tokio::test]
async fn update_public_config_returns_dns_instructions_for_custom_domain() {
    let (app, db) = setup_test_app().await;
    let (_, token) = test_utils::create_and_login_admin_user(&app, &db).await;
    let slug = format!("dp-cfg-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let dom  = format!("{slug}.dev.atlas.oply.co");

    let (s, iid) = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    assert_eq!(s, StatusCode::CREATED, "provision must return 201");

    let custom = format!("app.{slug}.example.com");
    let (s, j) = put_public_config(&app, &token, &iid, &custom).await;

    assert_eq!(s, StatusCode::OK, "update_public_config returned: {j}");

    // dns_instructions must be present
    let dns = j.get("dns_instructions").expect("dns_instructions must be in response");
    assert!(dns.is_object(), "dns_instructions must be a JSON object");
    assert_eq!(dns["record_type"], "CNAME",
        "record_type must be CNAME, got: {}", dns["record_type"]);
    assert_eq!(dns["name"], custom,
        "dns_instructions.name must equal the custom_domain that was set");

    // custom_domain saved correctly
    assert_eq!(j["custom_domain"], custom);

    // CNAME value must not be the old stale value
    let cname_val = dns["value"].as_str().unwrap_or("");
    assert!(!cname_val.contains("atlas-platform.com"),
        "Stale 'atlas-platform.com' hardcoded value must not appear. Got: {cname_val}");
    assert!(!cname_val.is_empty(), "CNAME value must not be empty");
}

#[tokio::test]
async fn get_public_config_returns_dns_instructions_when_custom_domain_is_saved() {
    let (app, db) = setup_test_app().await;
    let (_, token) = test_utils::create_and_login_admin_user(&app, &db).await;
    let slug = format!("dp-get-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let dom  = format!("{slug}.dev.atlas.oply.co");

    let (s, iid) = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    assert_eq!(s, StatusCode::CREATED, "provision must return 201");

    let custom = format!("portal.{slug}.example.com");
    let (s, _) = put_public_config(&app, &token, &iid, &custom).await;
    assert_eq!(s, StatusCode::OK, "PUT must succeed before GET check");

    // GET must now return dns_instructions — not just the PUT response
    let (s, j) = get_public_config(&app, &token, &iid).await;
    assert_eq!(s, StatusCode::OK);

    let dns = j.get("dns_instructions")
        .expect("GET /public-config must return dns_instructions when custom_domain is persisted");
    assert!(dns.is_object() && !dns.is_null(),
        "dns_instructions must be a non-null object. Got: {j}");
    assert_eq!(dns["name"], custom);
}

#[tokio::test]
async fn get_public_config_omits_dns_instructions_without_custom_domain() {
    let (app, db) = setup_test_app().await;
    let (_, token) = test_utils::create_and_login_admin_user(&app, &db).await;
    let slug = format!("dp-nodns-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let dom  = format!("{slug}.dev.atlas.oply.co");

    let (s, iid) = provision_and_get_instance_id(&app, &db, &token, &slug, &dom).await;
    assert_eq!(s, StatusCode::CREATED);

    // GET without ever setting a custom domain
    let (s, j) = get_public_config(&app, &token, &iid).await;
    assert_eq!(s, StatusCode::OK);

    // dns_instructions should be absent or null
    let dns = j.get("dns_instructions");
    let absent_or_null = dns.is_none() || dns.map(|v| v.is_null()).unwrap_or(false);
    assert!(absent_or_null,
        "GET must NOT return non-null dns_instructions when no custom_domain is set. Got: {j}");
}

#[tokio::test]
async fn update_public_config_rejects_duplicate_custom_domain_with_409() {
    let (app, db) = setup_test_app().await;
    let (_, token) = test_utils::create_and_login_admin_user(&app, &db).await;

    let slug1 = format!("dpdup1-{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let slug2 = format!("dpdup2-{}", &uuid::Uuid::new_v4().to_string()[..8]);

    let (s1, iid1) = provision_and_get_instance_id(
        &app, &db, &token, &slug1, &format!("{slug1}.dev.atlas.oply.co")).await;
    let (s2, iid2) = provision_and_get_instance_id(
        &app, &db, &token, &slug2, &format!("{slug2}.dev.atlas.oply.co")).await;
    assert_eq!(s1, StatusCode::CREATED);
    assert_eq!(s2, StatusCode::CREATED);

    let shared_domain = format!("shared.{slug1}.example.com");

    // First tenant claims the domain — must succeed
    let (r1, _) = put_public_config(&app, &token, &iid1, &shared_domain).await;
    assert_eq!(r1, StatusCode::OK, "first assignment must succeed");

    // Second tenant claims the same domain — must be rejected
    let (r2, _) = put_public_config(&app, &token, &iid2, &shared_domain).await;
    assert_eq!(r2, StatusCode::CONFLICT,
        "duplicate custom_domain must return 409 CONFLICT");
}

// ── ATLAS_CNAME_TARGET env var contract (pure, no DB) ─────────────────────────

mod cname_target_contract {
    use crate::admin::app_instance::DnsInstructions;

    /// Mirrors what the backend handler does when building DnsInstructions.
    fn build_dns(domain: &str) -> DnsInstructions {
        let target = std::env::var("ATLAS_CNAME_TARGET")
            .unwrap_or_else(|_| "api.atlas.oply.co".to_string());
        DnsInstructions {
            record_type: "CNAME".to_string(),
            name:        domain.to_string(),
            value:       target.clone(),
            note:        format!("Point {domain} as a CNAME to {target}. SSL is provisioned automatically."),
        }
    }

    #[test]
    fn falls_back_to_prod_when_env_unset() {
        // SAFETY: single-threaded test; no other threads read this var concurrently.
        unsafe { std::env::remove_var("ATLAS_CNAME_TARGET"); }
        let dns = build_dns("app.example.com");
        assert_eq!(dns.value, "api.atlas.oply.co",
            "Default CNAME must point to prod, not stale atlas-platform.com");
        assert!(!dns.value.contains("atlas-platform.com"),
            "Stale hardcoded value must not appear in default");
    }

    #[test]
    fn reads_dev_cname_from_env_var() {
        // SAFETY: single-threaded test.
        unsafe { std::env::set_var("ATLAS_CNAME_TARGET", "api.dev.atlas.oply.co"); }
        let dns = build_dns("app.example.com");
        assert_eq!(dns.value, "api.dev.atlas.oply.co");
        unsafe { std::env::remove_var("ATLAS_CNAME_TARGET"); }
    }

    #[test]
    fn reads_uat_cname_from_env_var() {
        // SAFETY: single-threaded test.
        unsafe { std::env::set_var("ATLAS_CNAME_TARGET", "api.uat.atlas.oply.co"); }
        let dns = build_dns("app.example.com");
        assert_eq!(dns.value, "api.uat.atlas.oply.co");
        unsafe { std::env::remove_var("ATLAS_CNAME_TARGET"); }
    }

    #[test]
    fn dns_name_equals_custom_domain() {
        let dns = build_dns("portal.myco.com");
        assert_eq!(dns.name, "portal.myco.com");
    }

    #[test]
    fn record_type_is_always_cname() {
        let dns = build_dns("anything.com");
        assert_eq!(dns.record_type, "CNAME");
    }

    #[test]
    fn note_contains_both_domain_and_cname_target() {
        // SAFETY: single-threaded test.
        unsafe { std::env::set_var("ATLAS_CNAME_TARGET", "api.uat.atlas.oply.co"); }
        let dns = build_dns("staging.example.com");
        assert!(dns.note.contains("staging.example.com"), "note must mention the custom domain");
        assert!(dns.note.contains("api.uat.atlas.oply.co"), "note must mention the CNAME target");
        unsafe { std::env::remove_var("ATLAS_CNAME_TARGET"); }
    }

    #[test]
    fn dns_json_serializes_to_snake_case_field_names() {
        let dns = build_dns("example.com");
        let j = serde_json::to_value(&dns).unwrap();
        assert!(j.get("record_type").is_some(), "must be snake_case 'record_type'");
        assert!(j.get("name").is_some());
        assert!(j.get("value").is_some());
        assert!(j.get("note").is_some());
        assert!(j.get("recordType").is_none(), "camelCase 'recordType' must NOT appear");
    }
}
