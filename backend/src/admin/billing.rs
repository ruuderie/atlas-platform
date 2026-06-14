use axum::{extract::{Path, State}, Json};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use uuid::Uuid;
use crate::entities::{billing_plan, transaction};

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

#[derive(serde::Deserialize)]
pub struct ExemptionPayload {
    pub is_exempt: bool,
    pub reason: Option<String>,
}

pub async fn set_subscription_exemption(
    State(db): State<DatabaseConnection>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ExemptionPayload>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    use crate::services::subscription_service::SubscriptionService;
    SubscriptionService::toggle_billing_exemption(&db, tenant_id, id, payload.is_exempt, payload.reason)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e))?;
    Ok(Json(serde_json::json!({ "status": "success" })))
}

pub async fn suspend_subscription(
    State(db): State<DatabaseConnection>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    use crate::entities::atlas_subscription::{Entity as SubscriptionEntity, ActiveModel as SubscriptionActiveModel, SubscriptionStatus};
    use sea_orm::{ActiveModelTrait, Set};
    let sub = SubscriptionEntity::find()
        .filter(crate::entities::atlas_subscription::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_subscription::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(s) = sub {
        let mut active: SubscriptionActiveModel = s.into();
        active.status = Set(SubscriptionStatus::Suspended);
        active.update(&db).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(serde_json::json!({ "status": "suspended" })))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "Subscription not found".to_string()))
    }
}

pub async fn reactivate_subscription(
    State(db): State<DatabaseConnection>,
    Path((tenant_id, id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    use crate::entities::atlas_subscription::{Entity as SubscriptionEntity, ActiveModel as SubscriptionActiveModel, SubscriptionStatus};
    use sea_orm::{ActiveModelTrait, Set};
    let sub = SubscriptionEntity::find()
        .filter(crate::entities::atlas_subscription::Column::TenantId.eq(tenant_id))
        .filter(crate::entities::atlas_subscription::Column::Id.eq(id))
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(s) = sub {
        let mut active: SubscriptionActiveModel = s.into();
        active.status = Set(SubscriptionStatus::Active);
        active.grace_period_ends_at = Set(None); // Reset grace clock
        active.update(&db).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        Ok(Json(serde_json::json!({ "status": "active" })))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "Subscription not found".to_string()))
    }
}
