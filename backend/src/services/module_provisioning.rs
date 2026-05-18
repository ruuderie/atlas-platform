//! Module provisioning service.
//!
//! Provides `seed_default_modules()` — a shared, idempotent helper that inserts
//! the default `app_instance_module` rows for a given app instance.
//!
//! Called from `AtlasApp::provision()` implementations. Safe to call multiple
//! times on the same `app_instance_id` (uses `ON CONFLICT DO NOTHING`).

use sea_orm::{DatabaseConnection, Statement};
use sea_orm::ConnectionTrait;
use uuid::Uuid;
use crate::models::admin_module::AdminModuleType;

/// Seeds the default module set for a given app instance.
///
/// This is called by `AtlasApp::provision()` when a new tenant is onboarded.
/// Uses `ON CONFLICT (app_instance_id, module_type) DO NOTHING` so it is safe
/// to call multiple times — existing configuration is never overwritten.
///
/// # Arguments
/// * `db` — Active database connection
/// * `app_instance_id` — The UUID of the app instance being provisioned
/// * `modules` — Tuples of `(AdminModuleType, display_name, sort_order, is_fixed)`
///
/// # Errors
/// Returns a `String` error description on DB failure.
pub async fn seed_default_modules(
    db: &DatabaseConnection,
    app_instance_id: Uuid,
    modules: Vec<(AdminModuleType, &'static str, i32, bool)>,
) -> Result<(), String> {
    if modules.is_empty() {
        return Ok(());
    }

    // Build a single bulk INSERT for all modules.
    // ON CONFLICT DO NOTHING ensures idempotency.
    let mut rows: Vec<String> = Vec::with_capacity(modules.len());
    let mut param_idx = 1usize;
    let mut values: Vec<sea_orm::Value> = Vec::new();

    for (module_type, display_name, sort_order, is_fixed) in &modules {
        let type_str = module_type.to_string(); // SCREAMING_SNAKE_CASE via strum
        rows.push(format!(
            "(${}, ${}, ${}, ${}, ${})",
            param_idx, param_idx + 1, param_idx + 2, param_idx + 3, param_idx + 4
        ));
        values.push(app_instance_id.into());
        values.push(type_str.into());
        values.push((*display_name).into());
        values.push((*sort_order).into());
        values.push((*is_fixed).into());
        param_idx += 5;
    }

    let sql = format!(
        "INSERT INTO app_instance_module \
         (app_instance_id, module_type, display_name, sort_order, is_fixed) \
         VALUES {} \
         ON CONFLICT (app_instance_id, module_type) DO NOTHING",
        rows.join(", ")
    );

    db.execute(Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        &sql,
        values,
    ))
    .await
    .map_err(|e| format!("seed_default_modules failed for app_instance {app_instance_id}: {e}"))?;

    Ok(())
}

/// Resolves the `app_instance_id` for a given `(tenant_id, app_id)` pair.
///
/// Used by `AtlasApp::provision()` implementations that need the app instance
/// UUID to call `seed_default_modules()`.
pub async fn resolve_app_instance_id(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    app_id: &str,
) -> Result<Uuid, String> {
    use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

    let instance = crate::entities::app_instance::Entity::find()
        .filter(crate::entities::app_instance::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::app_instance::Column::AppType.eq(app_id))
        .one(db)
        .await
        .map_err(|e| format!("resolve_app_instance_id DB error: {e}"))?
        .ok_or_else(|| {
            format!("No app_instance found for tenant {tenant_id} / app '{app_id}'")
        })?;

    Ok(instance.id)
}
