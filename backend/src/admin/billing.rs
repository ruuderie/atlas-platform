use axum::{extract::{Path, State}, Json};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, QueryOrder, QuerySelect, PaginatorTrait};
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

// ── Tenant Subscription Detail ────────────────────────────────────────────────

/// Response shape for `GET /api/admin/billing/tenant/{id}/subscription`.
///
/// Aggregates the tenant's active `atlas_subscription`, its linked `billing_plan`
/// name, current-period end, status, and a seat count derived from active profiles.
#[derive(serde::Serialize)]
pub struct TenantSubscriptionDetail {
    pub subscription_id: Option<Uuid>,
    pub status: Option<String>,
    /// Price in cents, e.g. 526000 for $5,260/mo
    pub mrr_cents: Option<i64>,
    /// "month" | "year"
    pub billing_interval: Option<String>,
    pub current_period_end: Option<String>,
    pub grace_period_ends_at: Option<String>,
    pub is_billing_exempt: bool,
    pub billing_exemption_reason: Option<String>,
    /// Number of active user profiles billed to this tenant
    pub billable_seats: u64,
}

/// `GET /api/admin/billing/tenant/{tenant_id}/subscription`
pub async fn get_tenant_subscription(
    State(db): State<DatabaseConnection>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<TenantSubscriptionDetail>, (axum::http::StatusCode, String)> {
    use crate::entities::{atlas_subscription, profile};

    // Latest subscription for this tenant
    let sub = atlas_subscription::Entity::find()
        .filter(atlas_subscription::Column::TenantId.eq(tenant_id))
        .order_by_desc(atlas_subscription::Column::CreatedAt)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Count billable seats (active profiles)
    let billable_seats = profile::Entity::find()
        .filter(profile::Column::TenantId.eq(tenant_id))
        .count(&db)
        .await
        .unwrap_or(0);

    let detail = if let Some(s) = sub {
        TenantSubscriptionDetail {
            subscription_id: Some(s.id),
            status: Some(s.status.to_string()),
            mrr_cents: Some(s.price_cents),
            billing_interval: Some(s.billing_interval.clone()),
            current_period_end: s.current_period_end.map(|d| d.to_rfc3339()),
            grace_period_ends_at: s.grace_period_ends_at.map(|d| d.to_rfc3339()),
            is_billing_exempt: s.is_billing_exempt,
            billing_exemption_reason: s.billing_exemption_reason,
            billable_seats,
        }
    } else {
        TenantSubscriptionDetail {
            subscription_id: None,
            status: None,
            mrr_cents: None,
            billing_interval: None,
            current_period_end: None,
            grace_period_ends_at: None,
            is_billing_exempt: false,
            billing_exemption_reason: None,
            billable_seats,
        }
    };

    Ok(Json(detail))
}


#[derive(serde::Deserialize)]
pub struct BillingPlanPayload {
    pub name: String,
    /// Price in cents (e.g. 9900 = $99.00)
    pub price: i64,
    pub currency: Option<String>,
    /// Billing interval: "month" | "year"
    pub interval: String,
}

/// `POST /api/admin/billing/plans`
pub async fn create_billing_plan(
    State(db): State<DatabaseConnection>,
    Json(payload): Json<BillingPlanPayload>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    use sea_orm::{ActiveModelTrait, Set};
    use crate::entities::billing_plan::ActiveModel;

    let id = Uuid::new_v4();
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let plan = ActiveModel {
        id: Set(id),
        name: Set(payload.name),
        price: Set(payload.price),
        currency: Set(payload.currency.unwrap_or_else(|| "usd".to_string())),
        interval: Set(payload.interval),
        created_at: Set(Some(now)),
        updated_at: Set(Some(now)),
    };
    plan.insert(&db).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": id, "status": "created" })))
}

/// `PUT /api/admin/billing/plans/{id}`
pub async fn update_billing_plan(
    State(db): State<DatabaseConnection>,
    Path(plan_id): Path<Uuid>,
    Json(payload): Json<BillingPlanPayload>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    use sea_orm::{ActiveModelTrait, Set, IntoActiveModel};

    let plan = billing_plan::Entity::find_by_id(plan_id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    let mut active = plan.into_active_model();
    active.name = Set(payload.name);
    active.price = Set(payload.price);
    active.currency = Set(payload.currency.unwrap_or_else(|| "usd".to_string()));
    active.interval = Set(payload.interval);
    active.updated_at = Set(Some(chrono::Utc::now().into()));
    active.update(&db).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": plan_id, "status": "updated" })))
}

/// `DELETE /api/admin/billing/plans/{id}`
pub async fn delete_billing_plan(
    State(db): State<DatabaseConnection>,
    Path(plan_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, (axum::http::StatusCode, String)> {
    use sea_orm::ModelTrait;

    let plan = billing_plan::Entity::find_by_id(plan_id)
        .one(&db)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (axum::http::StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    plan.delete(&db).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
