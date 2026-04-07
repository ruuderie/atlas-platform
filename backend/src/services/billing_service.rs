use crate::services::audit::AuditService;
use crate::entities::tenant_subscription;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use reqwest::StatusCode;

pub struct BillingService;

impl BillingService {
    /// Domain method to abstract subscription status changes and ensure they are audited
    pub async fn update_subscription_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        new_status: &str,
    ) -> Result<tenant_subscription::Model, (StatusCode, String)> {
        
        let subscription = tenant_subscription::Entity::find()
            .filter(tenant_subscription::Column::TenantId.eq(tenant_id))
            .one(db)
            .await
            .map_err(|e| {
                tracing::error!("Database query error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            })?;

        let subscription = match subscription {
            Some(s) => s,
            None => {
                return Err((StatusCode::NOT_FOUND, "Subscription not found for tenant".to_string()));
            }
        };

        if subscription.status == new_status {
            return Ok(subscription); // No-op if status is unchanged
        }

        let old_state = json!({
            "status": subscription.status.clone()
        });

        // Convert to active model for updating
        let mut active_sub: tenant_subscription::ActiveModel = subscription.into();
        active_sub.status = Set(new_status.to_string());
        active_sub.updated_at = Set(Some(Utc::now().into()));

        let updated_sub = active_sub.update(db).await.map_err(|e| {
            tracing::error!("Database update error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update subscription".to_string())
        })?;

        let new_state = json!({
            "status": updated_sub.status.clone()
        });

        // Audit the crucial billing status change
        AuditService::log_action(
            db.clone(),
            Some(tenant_id),
            None, // System triggered, or trace actor from axum logic in expanded parameters
            "billing.subscription.status_changed".to_string(),
            "TenantSubscription".to_string(),
            updated_sub.id,
            Some(old_state),
            Some(new_state),
            None,
        );

        Ok(updated_sub)
    }
}
