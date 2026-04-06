use axum::{extract::{Path, State}, Json};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use crate::entities::{billing_plan, tenant_subscription, transaction};

pub async fn list_billing_plans(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<billing_plan::Model>>, (axum::http::StatusCode, String)> {
    let plans = billing_plan::Entity::find()
        .all(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(plans))
}

pub async fn list_transactions(
    State(db): State<DatabaseConnection>,
) -> Result<Json<Vec<transaction::Model>>, (axum::http::StatusCode, String)> {
    let txs = transaction::Entity::find()
        .all(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(txs))
}

pub async fn get_tenant_ledger(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<Vec<transaction::Model>>, (axum::http::StatusCode, String)> {
    let txs = transaction::Entity::find()
        .filter(transaction::Column::TenantId.eq(tenant_id))
        .all(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(txs))
}
