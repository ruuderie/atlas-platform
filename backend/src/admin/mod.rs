pub mod routes;
pub mod setup;
pub mod billing;
pub mod analytics;
pub mod developer_console;
pub mod provision;
pub mod platform_products;   // Product launch engine: CRUD + deploy hooks
pub mod app_instance;        // App instance: public-config, lifecycle (suspend/archive)
pub mod product_variants;    // Product page variants: bulk-generate, AI localize, waitlist export
pub mod feature_flags;       // Feature flag registry: global rollout, plan gates, per-NI overrides, audit log
pub mod compliance;
pub mod ai_tasks;
pub mod users;
pub mod passkeys_admin;      // Super-admin passkey management: list + revoke all users' passkeys
pub mod upload;              // Admin-scoped R2 presigned upload URL (avatars, transcripts)




