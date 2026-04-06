use sea_orm::*;
use serde_json::Value;
use uuid::Uuid;
use crate::entities::global_search_index;

pub async fn upsert_search_index<C>(
    db: &C,
    entity_type: &str,
    entity_id: Uuid,
    tenant_id: Option<Uuid>,
    text_payload: &str,
    metadata: Value,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    let sql = r#"
        INSERT INTO global_search_index (id, entity_type, entity_id, tenant_id, searchable_text, metadata)
        VALUES ($1, $2, $3, $4, to_tsvector('english', $5), $6)
        ON CONFLICT (entity_type, entity_id)
        DO UPDATE SET 
            tenant_id = EXCLUDED.tenant_id,
            searchable_text = to_tsvector('english', $5),
            metadata = EXCLUDED.metadata,
            updated_at = CURRENT_TIMESTAMP
    "#;

    db.execute(Statement::from_sql_and_values(
        DbBackend::Postgres,
        sql,
        vec![
            Uuid::new_v4().into(),
            entity_type.into(),
            entity_id.into(),
            tenant_id.into(),
            text_payload.into(),
            metadata.into(),
        ],
    ))
    .await?;

    Ok(())
}

pub async fn remove_from_search_index<C>(
    db: &C,
    entity_type: &str,
    entity_id: Uuid,
) -> Result<(), DbErr>
where
    C: ConnectionTrait,
{
    global_search_index::Entity::delete_many()
        .filter(global_search_index::Column::EntityType.eq(entity_type))
        .filter(global_search_index::Column::EntityId.eq(entity_id))
        .exec(db)
        .await?;
    Ok(())
}
