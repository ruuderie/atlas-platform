pub mod routes;
pub mod setup;
pub mod billing;
pub mod analytics;
pub mod developer_console;
pub mod provision;
pub mod platform_products;   // Product launch engine: CRUD + deploy hooks
pub mod app_instance;        // App instance: public-config, lifecycle (suspend/archive)
pub mod product_variants;    // Product page variants: bulk-generate, AI localize, waitlist export
