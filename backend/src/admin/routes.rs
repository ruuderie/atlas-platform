use axum::{
    routing::{get, post, put, delete},
    Router,
};
use sea_orm::DatabaseConnection;
use crate::handlers::{admin, categories, 
    profiles, templates, contacts, customers,
    leads, deals, cases, files, listings, accounts};


pub fn admin_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
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
                // Case Management
                .route("/api/admin/cases", get(cases::get_cases).post(cases::create_case))
                .route("/api/admin/cases/{case_id}", get(cases::get_case).put(cases::update_case).delete(cases::delete_case))

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
                // Tenant management API is handled via tenant::authenticated_routes
                // Tenant management API is handled via tenant::authenticated_routes

                // Listing management
                .route("/api/admin/listings", get(admin::get_network_listings).post(listings::create_listing))
                .route("/api/admin/listings/{id}", get(listings::get_listing_by_id).put(listings::update_listing).delete(listings::delete_listing))

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
                //.layer(axum::middleware::from_fn_with_state(db.clone(), auth_middleware))
                .with_state(db)
        })

}

