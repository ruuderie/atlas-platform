pub mod ai_tasks;
pub mod analytics;
pub mod app_instance; // App instance: public-config, lifecycle (suspend/archive)
pub mod billing;
pub mod campaigns; // G-19 — Campaign registry: list, detail, create, status update, enrollments
pub mod compliance;
pub mod developer_console;
pub mod feature_flags; // Feature flag registry: global rollout, plan gates, per-NI overrides, audit log
pub mod passkeys_admin; // Super-admin passkey management: list + revoke all users' passkeys
pub mod platform_products; // Product launch engine: CRUD + deploy hooks
pub mod product_variants; // Product page variants: bulk-generate, AI localize, waitlist export
pub mod provision;
pub mod routes;
pub mod setup;
pub mod support_inbox;
pub mod upload; // Admin-scoped R2 presigned upload URL (avatars, transcripts)
pub mod users; // G-07 — Platform support inbox: cross-tenant platform_support room management
