use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::DatabaseConnection;
use crate::handlers::{admin, categories, 
    profiles, templates, contacts, customers,
    leads, deals, cases, files, listings, accounts};
use crate::admin::provision::provision_tenant;


pub fn admin_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    tracing::debug!("Setting up admin routes (legacy shim)");
    admin_routes_raw().with_state(db)
}

/// State-free admin route constructor — complies with the AtlasApp state-binding contract.
///
/// Returns all platform-admin–only Axum routes WITHOUT calling `.with_state(db)`.
/// State is applied exactly once at the `AtlasApp` boundary in `PlatformAdminApp::authenticated_router()`.
///
/// # Why this exists
/// The original `admin_routes(db)` called `.with_state(db)` internally.
/// Per the AtlasApp contract, state must NOT be applied inside route constructors that are
/// merged via `get_active_apps()` — Axum silently drops routes from pre-finalized sub-routers.
/// This function provides the state-free form that `PlatformAdminApp` can safely use.
pub fn admin_routes_raw() -> Router<DatabaseConnection> {
    tracing::debug!("Setting up admin routes");
    Router::new()
        .merge({
            Router::new()

                
                // User management
                .route("/api/admin/users", get(admin::list_users))
                .route("/api/admin/users/{user_id}", get(admin::get_user))
                .route("/api/admin/users/{user_id}", put(admin::update_user))
                .route("/api/admin/users/{user_id}", delete(admin::delete_user))
                .route("/api/admin/users/{user_id}/toggle-admin", post(admin::toggle_admin))
                .route("/api/admin/users/{user_id}/impersonate", post(admin::impersonate_user))
                // Account management
                .route("/api/admin/accounts", get(accounts::get_accounts).post(accounts::create_account))
                .route("/api/admin/accounts/{account_id}", get(accounts::get_account).put(accounts::update_account).delete(accounts::delete_account))
                // ── LEGACY CRM ROUTES (FULL HANDLER CUTOVER IN PROGRESS) ────────────────
                // All 6 legacy CRM handlers (customers, contacts, leads, deals, cases, activities)
                // have been updated to dual-write to the new unified Account/Contact/Opportunity/Case
                // services. These routes are now on a deprecation path.
                // New canonical surface: /api/admin/accounts, /api/opportunities, /api/cases, etc.
                // Customer management
                .route("/api/admin/customers", get(customers::get_customers).post(customers::create_customer))
                .route("/api/admin/customers/{customer_id}", get(customers::get_customer).put(customers::update_customer).delete(customers::delete_customer))
                // Lead management
                .route("/api/admin/leads", get(leads::get_leads).post(leads::create_lead))
                .route("/api/admin/leads/{lead_id}", get(leads::get_lead).put(leads::update_lead).delete(leads::delete_lead))
                // Deal management
                .route("/api/admin/deals", get(deals::get_deals).post(deals::create_deal))
                .route("/api/admin/deals/{deal_id}", get(deals::get_deal).put(deals::update_deal).delete(deals::delete_deal))
                .route("/api/admin/deals/{deal_id}/files/{file_id}", post(deals::add_file_to_deal))
                .route("/api/admin/deals/{deal_id}/files", get(deals::get_deal_files))
                //Contact management
                .route("/api/admin/contacts", get(contacts::get_contacts).post(contacts::create_contact))
                .route("/api/admin/contacts/{contact_id}", get(contacts::get_contact).put(contacts::update_contact).delete(contacts::delete_contact))
                // Case Management (legacy)
                .route("/api/admin/cases", get(cases::get_cases).post(cases::create_case))
                .route("/api/admin/cases/{case_id}", get(cases::get_case).put(cases::update_case).delete(cases::delete_case))

                // ── UNIFIED PLATFORM GENERICS CRM (new canonical surface) ───────────────
                // Powered by AccountService + ContactService + OpportunityService + CaseService + DocumentService
                // New unified opportunity pipeline (replaces legacy deals + leads in admin)
                // (handlers::opportunities would be added in follow-up; for now admin can use the service-backed paths)

                //File Management
                .route("/api/admin/files", post(files::create_file))
                .route("/api/admin/files/{id}", put(files::update_file))
                .route("/api/admin/files/{id}", get(files::get_file))
                .route("/api/admin/files/{id}/info", get(files::get_file_info))
                .route("/api/admin/files/{id}/thumbnail", get(files::get_file_thumbnail))
                .route("/api/admin/files/{id}", delete(files::delete_file))
                .route("/api/admin/files/user/{user_id}", get(files::get_user_files))
                .route("/api/admin/files/{file_id}/associate/{entity_type}/{entity_id}", post(files::associate_file).delete(files::disassociate_file))
                .route("/api/admin/files/associated/{entity_type}/{entity_id}", get(files::get_associated_files))
                // Network management
                .route("/api/admin/tenant-stats", get(admin::get_all_network_stats))
                .route("/api/admin/tenant-stats/{tenant_id}", get(admin::get_network_stats))
                .route("/api/admin/tenant/{tenant_id}/listings", get(admin::get_network_listings))
                .route("/api/admin/tenant/{tenant_id}/listings/{listing_id}", get(admin::get_listing))
                .route("/api/admin/platform/apps", get(admin::get_platform_apps))
                .route("/api/admin/platform/apps/{instance_id}/domains", get(admin::get_app_domains).post(admin::add_app_domain))
                .route("/api/admin/platform/apps/{instance_id}/domains/{domain_name}", delete(admin::remove_app_domain))
                // Deployment config mutations (keyed on tenant_id, not instance_id)
                .route("/api/admin/platform/apps/{tenant_id}/account", put(admin::link_deployment_account))
                .route("/api/admin/platform/apps/{tenant_id}/purpose", put(admin::set_deployment_purpose))
                // Provision a new tenant from scratch (tenant + user + account + app_instances + domain + setup link)
                .route("/api/admin/tenants/provision", post(crate::handlers::admin_provision::provision_tenant))
                // Re-provision existing tenant's app instances (calls provision() on all active AtlasApps)
                .route("/api/admin/platform/provision/{tenant_id}", post(provision_tenant))
                // G-06 Verification Queue
                .merge(crate::handlers::verification::authenticated_routes())
                // Tenant management API is handled via tenant::authenticated_routes

                // ── Admin Module Registry ─────────────────────────────────────────
                // GET  /api/admin/modules                            — tenant-scoped
                // POST /api/platform/tenants/{tenant_id}/modules     — platform-admin only
                .merge(crate::handlers::admin_modules::routes())

                // Listing management
                .route("/api/admin/listings", get(admin::get_network_listings).post(listings::create_listing))
                .route("/api/admin/listings/{id}", get(listings::get_listing_by_id).put(listings::update_listing).delete(listings::delete_listing))
                .route("/api/admin/listings/{id}/ab-tests", get(crate::handlers::ab_testing::get_listing_ab_tests_admin))

                .route("/api/admin/listings/pending", get(admin::list_pending_listings))
                .route("/api/admin/listings/{listing_id}/approve", post(admin::approve_listing))
                .route("/api/admin/listings/{listing_id}/reject", post(admin::reject_listing))

                // Ad purchase management
                .route("/api/admin/ad-purchases/stats", get(admin::get_ad_purchase_stats))
                .route("/api/admin/ad-purchases", get(admin::list_ad_purchases))
                .route("/api/admin/ad-purchases/{purchase_id}", get(admin::get_ad_purchase))
                .route("/api/admin/ad-purchases/active", get(admin::list_active_ad_purchases))
                .route("/api/admin/ad-purchases/{purchase_id}/cancel", post(admin::cancel_ad_purchase))

                // Category management
                .route("/api/admin/categories", get(categories::get_categories).post(categories::create_category))
                .route("/api/admin/categories/{category_id}", get(categories::get_category).put(categories::update_category).delete(categories::delete_category))

                // Profile management
                .route("/api/admin/profiles", get(profiles::get_profiles).post(profiles::create_profile))
                .route("/api/admin/profiles/{profile_id}", get(profiles::get_profile_by_id).put(profiles::update_profile).delete(profiles::delete_profile))

                // Template management
                .route("/api/admin/templates", get(templates::get_templates).post(templates::create_template))
                .route("/api/admin/templates/{template_id}", get(templates::get_template_by_id).put(templates::update_template).delete(templates::delete_template))

                // Statistics and reports
                .route("/admin/statistics/users", get(admin::get_user_statistics))
                .route("/admin/statistics/accounts", get(admin::get_account_statistics))
                .route("/admin/statistics/listings", get(admin::get_listing_statistics))
                .route("/admin/statistics/ad-purchases", get(admin::get_ad_purchase_statistics))
                .route("/admin/reports/activity", get(admin::get_activity_report))
                .route("/admin/reports/revenue", get(admin::get_revenue_report))
                // Analytics
                .route("/api/admin/analytics/business_kpis", get(crate::admin::analytics::get_business_kpis))
                .route("/api/admin/analytics/engagement", get(crate::admin::analytics::get_engagement))
                .route("/api/admin/analytics/trends", get(crate::admin::analytics::get_trends))
                .route("/api/admin/analytics/billing_summary", get(crate::admin::analytics::get_billing_summary))
                
                // Billing & Monetization
                .route("/api/admin/billing/plans", get(crate::admin::billing::list_billing_plans).post(crate::admin::billing::create_billing_plan))
                .route("/api/admin/billing/plans/{id}", put(crate::admin::billing::update_billing_plan).delete(crate::admin::billing::delete_billing_plan))
                .route("/api/admin/billing/transactions", get(crate::admin::billing::list_transactions))
                .route("/api/admin/billing/tenant/{tenant_id}", get(crate::admin::billing::get_tenant_ledger))
                .route("/api/admin/billing/tenant/{tenant_id}/subscription/{id}/exemption", post(crate::admin::billing::set_subscription_exemption))
                .route("/api/admin/billing/tenant/{tenant_id}/subscription/{id}/suspend", post(crate::admin::billing::suspend_subscription))
                .route("/api/admin/billing/tenant/{tenant_id}/subscription/{id}/reactivate", post(crate::admin::billing::reactivate_subscription))
                // AI Task logs
                .route("/api/admin/ai-tasks/{id}/logs", get(crate::admin::ai_tasks::get_task_logs))
                // Developer Console
                .route("/api/admin/developer/tenant/{tenant_id}/api-tokens", get(crate::admin::developer_console::list_api_tokens).post(crate::admin::developer_console::create_api_token))
                .route("/api/admin/developer/tenant/{tenant_id}/api-tokens/{token_id}", delete(crate::admin::developer_console::revoke_api_token))
                .route("/api/admin/developer/tenant/{tenant_id}/webhooks", get(crate::admin::developer_console::list_webhook_endpoints).post(crate::admin::developer_console::create_webhook_endpoint))
                .route("/api/admin/developer/tenant/{tenant_id}/webhooks/{endpoint_id}", delete(crate::admin::developer_console::delete_webhook_endpoint))
                .route("/api/admin/developer/tenant/{tenant_id}/webhook-deliveries", get(crate::admin::developer_console::list_webhook_deliveries))

                // ── Product Launch Engine (platform super-admin) ───────────────────
                // Platform product registry (Folio, Anchor, Network, Meridian)
                .merge(crate::admin::platform_products::routes_raw())
                // App instance public config + lifecycle (suspend / resume / archive)
                .merge(crate::admin::app_instance::routes_raw())
                // Product page variants: bulk-generate, AI localize, waitlist analytics + CSV export
                .merge(crate::admin::product_variants::routes_raw())
                // Feature flag registry: global rollout, plan gates, per-NI overrides, audit trail
                .merge(crate::admin::feature_flags::routes_raw())
                // Compliance management: G-16 regulatory permits + G-01 PostGIS geo-zones
                .merge(crate::admin::compliance::routes_raw())
                // AI Asynchronous Job Queue (G-08)
                .merge(crate::admin::ai_tasks::routes_raw())
                // User invitations (Platform Invite System)
                .merge(crate::admin::users::routes_raw())
                // Passkeys admin: super-admin can list/revoke all registered passkeys
                .merge(crate::admin::passkeys_admin::routes_raw())
                // A/B Test management: end a test (set status -> Ended)
                .route("/api/admin/ab-tests/{id}/end", axum::routing::post(crate::handlers::ab_testing::end_ab_test))
                // Admin R2 presigned upload (avatars, transcripts) — no SiteConfig dependency
                .merge(crate::admin::upload::routes())
                // Platform-generic syndication: offer catalog + active link management
                .merge(crate::handlers::syndication_admin::syndication_admin_routes())
                // G-19 Campaigns: list/create campaigns + enrollments
                .merge(crate::admin::campaigns::routes_raw())
                // G-07 ext — Platform support inbox: cross-tenant platform_support room management
                .merge(crate::admin::support_inbox::routes_raw())
                // Platform-admin Landing Page Builder (app-scoped acquisition pages)
                // Routes: /api/admin/landing-pages/* + /api/admin/utm-presets/*
                .merge(crate::handlers::landing_pages::landing_page_routes_raw())
        })
}
