#![allow(dead_code, unused_imports)]
use sea_orm::DatabaseConnection;
use uuid::Uuid;

// ── Category helpers shared across all industry seed packs ───────────────────

/// Inserts a network_type if it doesn't already exist, returning its ID.
pub(super) async fn ensure_network_type(
    db: &DatabaseConnection,
    name: &str,
    description: &str,
) -> Result<Uuid, String> {
    use sea_orm::Statement;
    use sea_orm::ConnectionTrait;

    let row = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!(
            "INSERT INTO network_type (id, name, description, is_active, created_at, updated_at)
             VALUES (gen_random_uuid(), '{name}', '{description}', true, NOW(), NOW())
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id"
        ),
    ))
    .await
    .map_err(|e| format!("ensure_network_type error: {e}"))?
    .ok_or_else(|| "ensure_network_type: no row returned".to_string())?;

    let id: Uuid = row.try_get("", "id").map_err(|e| format!("uuid parse: {e}"))?;
    Ok(id)
}

/// Inserts a top-level category under a network_type if it doesn't exist, returning its ID.
pub(super) async fn ensure_category(
    db: &DatabaseConnection,
    network_type_id: Uuid,
    name: &str,
    description: &str,
) -> Result<Uuid, String> {
    use sea_orm::Statement;
    use sea_orm::ConnectionTrait;

    let row = db.query_one(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!(
            "INSERT INTO category (id, network_type_id, name, description, is_custom, is_active, created_at, updated_at)
             VALUES (gen_random_uuid(), '{network_type_id}', '{name}', '{description}', false, true, NOW(), NOW())
             ON CONFLICT (network_type_id, name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id"
        ),
    ))
    .await
    .map_err(|e| format!("ensure_category('{name}') error: {e}"))?
    .ok_or_else(|| format!("ensure_category('{name}'): no row returned"))?;

    let id: Uuid = row.try_get("", "id").map_err(|e| format!("uuid parse: {e}"))?;
    Ok(id)
}

/// Inserts a sub-category under a parent, idempotent.
pub(super) async fn ensure_subcategory(
    db: &DatabaseConnection,
    network_type_id: Uuid,
    parent_id: Uuid,
    name: &str,
) -> Result<(), String> {
    use sea_orm::Statement;
    use sea_orm::ConnectionTrait;

    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!(
            "INSERT INTO category (id, network_type_id, parent_category_id, name, description, is_custom, is_active, created_at, updated_at)
             VALUES (gen_random_uuid(), '{network_type_id}', '{parent_id}', '{name}', 'Subcategory of {name}', false, true, NOW(), NOW())
             ON CONFLICT (network_type_id, name) DO NOTHING"
        ),
    ))
    .await
    .map_err(|e| format!("ensure_subcategory('{name}') error: {e}"))?;

    Ok(())
}

/// Records a seed application timestamp in tenant_setting, allowing re-application
/// while maintaining a full history. Each call appends a new ISO 8601 timestamp.
pub(super) async fn record_seed_application(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    seed_id: &str,
) -> Result<(), String> {
    use sea_orm::Statement;
    use sea_orm::ConnectionTrait;

    let key = format!("seed_applied:{seed_id}");
    let now = chrono::Utc::now().to_rfc3339();

    db.execute(Statement::from_string(
        sea_orm::DatabaseBackend::Postgres,
        format!(
            "INSERT INTO tenant_setting (id, tenant_id, key, value, is_encrypted, updated_at, created_at)
             VALUES (gen_random_uuid(), '{tenant_id}', '{key}', '{now}', false, NOW(), NOW())
             ON CONFLICT (tenant_id, key) DO UPDATE
               SET value = EXCLUDED.value,
                   updated_at = NOW()"
        ),
    ))
    .await
    .map_err(|e| format!("record_seed_application error: {e}"))?;

    Ok(())
}
