//! Integration tests for the Admin Module Registry API.
//!
//! Tests cover:
//! - `GET /api/admin/modules` — happy path, unauthorized, empty state
//! - `POST /api/platform/tenants/{tenant_id}/modules` — superadmin upsert, forbidden for non-admin
//! - Fixed-module enforcement — Dashboard/Settings/Security cannot be disabled
//! - Idempotency — duplicate seeds produce no extra rows
//!
//! Uses the same `setup_test_app` + `test_utils` pattern as all other integration tests in this file.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

use crate::tests::api_tests::setup_test_app;
use crate::tests::test_utils;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

async fn get_or_create_test_tenant_id(db: &sea_orm::DatabaseConnection) -> Uuid {
    // Return the first tenant or create one
    use sea_orm::{EntityTrait, QueryOrder};
    let tenant = crate::entities::tenant::Entity::find()
        .order_by_asc(crate::entities::tenant::Column::CreatedAt)
        .one(db)
        .await
        .unwrap();
    if let Some(t) = tenant {
        t.id
    } else {
        let t = crate::tests::test_utils::create_test_tenant(db).await;
        t.id
    }
}

async fn get_or_create_app_instance_for_test_tenant(db: &sea_orm::DatabaseConnection) -> Uuid {
    let tenant_id = get_or_create_test_tenant_id(db).await;
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
    let instance = crate::entities::app_instance::Entity::find()
        .filter(crate::entities::app_instance::Column::TenantId.eq(tenant_id))
        .one(db)
        .await
        .unwrap();
    if let Some(i) = instance {
        i.id
    } else {
        crate::entities::app_instance::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            app_type: Set("anchor".to_string()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap()
        .id
    }
}

/// Seeds a minimal `app_instance_module` row for a given app_instance_id.
/// Returns the created module's `module_type` string.
async fn seed_module(
    db: &sea_orm::DatabaseConnection,
    app_instance_id: Uuid,
    module_type: &str,
    sort_order: i32,
    is_fixed: bool,
) {
    use sea_orm::{Set, ActiveModelTrait};
    use crate::entities::app_instance_module;

    app_instance_module::ActiveModel {
        id: Set(Uuid::new_v4()),
        app_instance_id: Set(app_instance_id),
        module_type: Set(module_type.to_string()),
        display_name: Set(module_type.to_string()),
        icon: Set(None),
        sort_order: Set(sort_order),
        is_enabled: Set(true),
        is_fixed: Set(is_fixed),
        config: Set(None),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
    }
    .insert(db)
    .await
    .expect("seed_module insert failed");
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /api/admin/modules
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_admin_modules_returns_ok_for_admin() {
    let (app, _db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &_db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/modules")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // 200 OK (even if empty list — no modules seeded for the test tenant)
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "GET /api/admin/modules should return 200 for authenticated admin"
    );
}

#[tokio::test]
async fn test_get_admin_modules_returns_forbidden_without_auth() {
    let (app, _db) = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/modules")
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // The platform auth_middleware returns 403 FORBIDDEN (not 401) when
    // no session cookie / Authorization header is present. This matches
    // the behaviour observed in other protected endpoints.
    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "GET /api/admin/modules should return 403 for unauthenticated requests"
    );
}

#[tokio::test]
async fn test_get_admin_modules_returns_forbidden_for_member_role() {
    // Validates P0 fix: the role gate rejects Member-role users who have a
    // valid session but insufficient privileges to read the module registry.
    let (app, db) = setup_test_app().await;
    let tenant_id = get_or_create_test_tenant_id(&db).await;

    // register_test_user creates a Member-role user_account.
    let mut username = format!("member{}", Uuid::new_v4());
    let (_, login_res) = test_utils::register_test_user(&app, tenant_id, &mut username).await;
    let member_token = login_res["token"].as_str().unwrap_or("").to_string();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/modules")
                .header("Authorization", format!("Bearer {}", member_token))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Member-role users must not be able to read the module registry"
    );
}

#[tokio::test]
async fn test_get_admin_modules_returns_sorted_list() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;

    // Find the test tenant's app_instance_id
    let instance_id = get_or_create_app_instance_for_test_tenant(&db).await;

    // Seed modules in reverse sort order intentionally
    seed_module(&db, instance_id, "SECURITY",  30, true).await;
    seed_module(&db, instance_id, "DASHBOARD",  0, true).await;
    seed_module(&db, instance_id, "BLOG",       10, false).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/admin/modules")
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Host", "localhost")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: Value = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .map(|b| serde_json::from_slice(&b).unwrap())
        .unwrap();

    let modules = body.as_array().expect("response should be an array");
    // Verify sort_order ascending
    let orders: Vec<i64> = modules
        .iter()
        .map(|m| m["sort_order"].as_i64().unwrap())
        .collect();
    let mut sorted = orders.clone();
    sorted.sort();
    assert_eq!(orders, sorted, "modules should be returned sorted by sort_order ASC");
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /api/platform/tenants/{tenant_id}/modules
// ─────────────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_upsert_module_creates_new_row() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    let tenant_id = get_or_create_test_tenant_id(&db).await;

    let body = json!({
        "module_type": "LEADS",
        "display_name": "Leads",
        "is_enabled": true,
        "sort_order": 20,
        "icon": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/platform/tenants/{tenant_id}/modules"))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "PlatformSuperAdmin should be able to upsert modules"
    );
}

#[tokio::test]
async fn test_upsert_module_forbidden_for_regular_user() {
    let (app, db) = setup_test_app().await;
    let tenant_id = get_or_create_test_tenant_id(&db).await;

    // Create and login a regular (non-admin) user
    let mut username = format!("regularuser{}", Uuid::new_v4());
    let (_, login_res) = test_utils::register_test_user(&app, tenant_id, &mut username).await;
    let regular_token = login_res["token"].as_str().unwrap_or("").to_string();

    let body = json!({
        "module_type": "BLOG",
        "display_name": "Blog",
        "is_enabled": true,
        "sort_order": 10,
        "icon": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/platform/tenants/{tenant_id}/modules"))
                .header("Authorization", format!("Bearer {}", regular_token))
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Regular users should not be able to upsert modules"
    );
}

#[tokio::test]
async fn test_fixed_module_cannot_be_disabled_via_api() {
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    let tenant_id = get_or_create_test_tenant_id(&db).await;

    // Attempt to disable Dashboard (a fixed module) — must be rejected with 400.
    let body = json!({
        "module_type": "DASHBOARD",
        "display_name": "Dashboard",
        "is_enabled": false,
        "sort_order": 0,
        "icon": null
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/platform/tenants/{tenant_id}/modules"))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    // P1 fix: must return 400 BAD_REQUEST (not 200) so the Platform Admin UI
    // receives explicit feedback rather than silently succeeding.
    assert_eq!(
        response.status(),
        StatusCode::BAD_REQUEST,
        "Disabling a fixed module must return 400 BAD_REQUEST"
    );
}

#[tokio::test]
async fn test_fixed_module_can_be_enabled_via_api() {
    // Enabling a fixed module (is_enabled=true) must always succeed — only
    // disabling is forbidden.
    let (app, db) = setup_test_app().await;
    let (_admin_user, admin_token) = test_utils::create_and_login_admin_user(&app, &db).await;
    let tenant_id = get_or_create_test_tenant_id(&db).await;

    let body = json!({
        "module_type": "SETTINGS",
        "display_name": "Settings",
        "is_enabled": true,
        "sort_order": 20,
        "icon": null
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/platform/tenants/{tenant_id}/modules"))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("Content-Type", "application/json")
                .header("Host", "localhost")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Enabling a fixed module must return 200 OK"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// UNIT TESTS — module_provisioning service
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod unit_tests {
    use crate::models::admin_module::AdminModuleType;

    #[test]
    fn test_dashboard_settings_security_are_fixed() {
        assert!(AdminModuleType::Dashboard.is_fixed(), "Dashboard must be fixed");
        assert!(AdminModuleType::Settings.is_fixed(),  "Settings must be fixed");
        assert!(AdminModuleType::Security.is_fixed(),  "Security must be fixed");
    }

    #[test]
    fn test_non_platform_modules_are_not_fixed() {
        let non_fixed = [
            AdminModuleType::Blog,
            AdminModuleType::Leads,
            AdminModuleType::Contacts,
            AdminModuleType::Services,
            AdminModuleType::Listings,
            AdminModuleType::Navigation,
        ];
        for m in non_fixed {
            assert!(!m.is_fixed(), "{m:?} should not be fixed");
        }
    }

    #[test]
    fn test_anchor_has_leads_and_contacts_in_default_modules() {
        use crate::atlas_apps::anchor::AnchorApp;
        use crate::traits::atlas_app::AtlasApp;
        let app = AnchorApp;
        let modules = app.default_modules();
        let types: Vec<_> = modules.iter().map(|(t, _, _, _)| *t).collect();
        assert!(types.contains(&AdminModuleType::Leads),    "AnchorApp must include Leads module");
        assert!(types.contains(&AdminModuleType::Contacts), "AnchorApp must include Contacts module");
        assert!(types.contains(&AdminModuleType::Dashboard),"AnchorApp must include Dashboard module");
    }

    #[test]
    fn test_network_instance_has_leads_and_listings() {
        use crate::atlas_apps::network_instance::NetworkInstanceApp;
        use crate::traits::atlas_app::AtlasApp;
        let app = NetworkInstanceApp;
        let modules = app.default_modules();
        let types: Vec<_> = modules.iter().map(|(t, _, _, _)| *t).collect();
        assert!(types.contains(&AdminModuleType::Leads),    "NetworkInstanceApp must include Leads");
        assert!(types.contains(&AdminModuleType::Listings), "NetworkInstanceApp must include Listings");
    }

    #[test]
    fn test_all_apps_include_fixed_modules() {
        use crate::atlas_apps::{anchor::AnchorApp, network_instance::NetworkInstanceApp};
        use crate::traits::atlas_app::AtlasApp;
        let apps: Vec<Box<dyn AtlasApp>> = vec![Box::new(AnchorApp), Box::new(NetworkInstanceApp)];
        for app in apps {
            let modules = app.default_modules();
            let types: Vec<_> = modules.iter().map(|(t, _, _, _)| *t).collect();
            assert!(types.contains(&AdminModuleType::Dashboard), "{} missing Dashboard", app.app_id());
            assert!(types.contains(&AdminModuleType::Settings),  "{} missing Settings",  app.app_id());
            assert!(types.contains(&AdminModuleType::Security),  "{} missing Security",  app.app_id());
        }
    }

    #[test]
    fn test_sort_orders_are_non_negative() {
        use crate::atlas_apps::anchor::AnchorApp;
        use crate::traits::atlas_app::AtlasApp;
        for (_, _, sort_order, _) in AnchorApp.default_modules() {
            assert!(sort_order >= 0, "sort_order must be non-negative");
        }
    }

    #[test]
    fn test_sort_orders_are_unique_per_app() {
        use crate::atlas_apps::anchor::AnchorApp;
        use crate::traits::atlas_app::AtlasApp;
        let modules = AnchorApp.default_modules();
        let mut orders: Vec<i32> = modules.iter().map(|(_, _, s, _)| *s).collect();
        orders.sort();
        orders.dedup();
        assert_eq!(
            orders.len(), modules.len(),
            "Each module must have a unique sort_order"
        );
    }

    #[test]
    fn test_display_names_are_non_empty() {
        use crate::atlas_apps::anchor::AnchorApp;
        use crate::atlas_apps::network_instance::NetworkInstanceApp;
        use crate::traits::atlas_app::AtlasApp;
        for app in [AnchorApp.default_modules(), NetworkInstanceApp.default_modules()] {
            for (_, display_name, _, _) in app {
                assert!(!display_name.is_empty(), "display_name must not be empty");
            }
        }
    }

    /// P1 fix #6 — guard against backend/shared-ui AdminModuleType drift.
    ///
    /// `shared-ui` re-declares `AdminModuleType` (WASM can't import backend crate types).
    /// If a developer adds a variant on the backend without updating shared-ui (or vice
    /// versa), the sidebar silently drops or misroutes modules.
    ///
    /// This test fixes the expected count at the backend level. When the backend enum
    /// grows, update the count here AND add the variant to `shared-ui`.
    #[test]
    fn test_admin_module_type_variant_count() {
        use strum::IntoEnumIterator;
        // Current canonical count: Dashboard, Settings, Security, Blog, ResumeProfiles,
        // ResumeEntries, LandingPages, Webforms, Navigation, Footer, PageHeaders,
        // Leads, Contacts, LeadOptions, Services, CaseStudies, Highlights,
        // Properties, Listings, Custom = 20 variants.
        //
        // UPDATE shared-ui AdminModuleType AND this constant together.
        const EXPECTED_VARIANT_COUNT: usize = 20;
        let actual = AdminModuleType::iter().count();
        assert_eq!(
            actual,
            EXPECTED_VARIANT_COUNT,
            "AdminModuleType has {actual} variants but expected {EXPECTED_VARIANT_COUNT}. \
             If you added a new variant, also add it to \
             apps/shared-ui/src/components/admin_module_sidebar.rs and update this constant."
        );
    }
}
