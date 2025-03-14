use axum::{
    routing::{get, post, put, delete},
    Router,
    extract::{Extension, Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as JsonResponse},
};
use sea_orm::DatabaseConnection;
use crate::handlers::{admin, categories, 
    profiles, templates, contacts, customers,
    leads, deals, cases, files, directories};
use crate::auth::*;
use crate::middleware::*;
use axum::middleware::from_fn;

use uuid::Uuid;

pub fn admin_routes(db: DatabaseConnection) -> Router<DatabaseConnection> {
    tracing::debug!("Setting up admin routes");
    Router::new()
        .nest("/", {
            Router::new()

                
                // User management
                .route("/admin/users", get(admin::list_users))
                .route("/admin/users/:user_id", get(admin::get_user))
                .route("/admin/users/:user_id", put(admin::update_user))
                .route("/admin/users/:user_id", delete(admin::delete_user))
                .route("/admin/users/:user_id/toggle-admin", post(admin::toggle_admin))
                // Customer management
                .route("/admin/customers", get(customers::get_customers).post(customers::create_customer))
                .route("/admin/customers/:customer_id", get(customers::get_customer).put(customers::update_customer).delete(customers::delete_customer))
                // Lead management
                .route("/admin/leads", get(leads::get_leads).post(leads::create_lead))
                .route("/admin/leads/:lead_id", get(leads::get_lead).put(leads::update_lead).delete(leads::delete_lead))
                // Deal management
                .route("/admin/deals", get(deals::get_deals).post(deals::create_deal))
                .route("/admin/deals/:deal_id", get(deals::get_deal).put(deals::update_deal).delete(deals::delete_deal))
                .route("/admin/deals/:deal_id/files/:file_id", post(deals::add_file_to_deal))
                .route("/admin/deals/:deal_id/files", get(deals::get_deal_files))
                //Contact management
                .route("/admin/contacts", get(contacts::get_contacts).post(contacts::create_contact))
                .route("/admin/contacts/:contact_id", get(contacts::get_contact).put(contacts::update_contact).delete(contacts::delete_contact))
                // Case Management
                .route("/admin/cases", get(cases::get_cases).post(cases::create_case))
                .route("/admin/cases/:case_id", get(cases::get_case).put(cases::update_case).delete(cases::delete_case))

                //File Management
                .route("/admin/files", post(files::create_file))
                .route("/admin/files/:id", put(files::update_file))
                .route("/admin/files/:id", get(files::get_file))
                .route("/admin/files/:id/info", get(files::get_file_info))
                .route("/admin/files/:id/thumbnail", get(files::get_file_thumbnail))
                .route("/admin/files/:id", delete(files::delete_file))
                // Directory management
                .route("/admin/directory-stats", get(admin::get_all_directory_stats))
                .route("/admin/directory-stats/:directory_id", get(admin::get_directory_stats))
                .route("/admin/directory/:directory_id/listings", get(admin::get_directory_listings))
                .route("/admin/directory/:directory_id/listings/:listing_id", get(admin::get_listing))
                //ALL DIRECTORIES
                .route("/admin/directories", get(admin::get_directories))
                .route("/admin/directories/:directory_id", get(admin::get_directory))
                .route("/admin/directories/", post(admin::create_directory))
                .route("/admin/directories/:directory_id", put(admin::update_directory))
                .route("/admin/directories/:directory_id", delete(admin::delete_directory))
                .route("/admin/directories/type/:type_id", get(admin::get_directories_by_type))
                .route("/admin/directories/:directory_id/config", get(directories::get_site_config).put(directories::update_site_config))
                .route("/admin/directories/:directory_id/modules", get(directories::get_enabled_modules).put(directories::update_enabled_modules))
                .route("/admin/directories/:directory_id/theme", get(directories::get_site_theme).put(directories::update_site_theme))
                .route("/admin/directories/:directory_id/custom-settings", get(directories::get_custom_settings).put(directories::update_custom_settings))
                //DIRETORY TYPE
                .route("/admin/directory-types", get(admin::get_directory_types))
                .route("/admin/directory-types/:directory_type_id", get(admin::get_directory_type))
                //create directory type
                .route("/admin/directory-types", post(admin::create_directory_type))
                //update directory type
                .route("/admin/directory-types/:directory_type_id", put(admin::update_directory_type))
                //delete directory type
                .route("/admin/directory-types/:directory_type_id", delete(admin::delete_directory_type))

                // Listing management
                .route("/admin/listings/pending", get(admin::list_pending_listings))
                .route("/admin/listings/:listing_id/approve", post(admin::approve_listing))
                .route("/admin/listings/:listing_id/reject", post(admin::reject_listing))

                // Ad purchase management
                .route("/admin/ad-purchases/stats", get(admin::get_ad_purchase_stats))
                .route("/admin/ad-purchases", get(admin::list_ad_purchases))
                .route("/admin/ad-purchases/:purchase_id", get(admin::get_ad_purchase))
                .route("/admin/ad-purchases/active", get(admin::list_active_ad_purchases))
                .route("/admin/ad-purchases/:purchase_id/cancel", post(admin::cancel_ad_purchase))

                // Category management
                .route("/admin/categories", get(categories::get_categories).post(categories::create_category))
                .route("/admin/categories/:category_id", get(categories::get_category).put(categories::update_category).delete(categories::delete_category))

                // Profile management
                .route("/admin/profiles", get(profiles::get_profiles).post(profiles::create_profile))
                .route("/admin/profiles/:profile_id", get(profiles::get_profile_by_id).put(profiles::update_profile).delete(profiles::delete_profile))

                // Template management
                .route("/admin/templates", get(templates::get_templates).post(templates::create_template))
                .route("/admin/templates/:template_id", get(templates::get_template_by_id).put(templates::update_template).delete(templates::delete_template))

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

