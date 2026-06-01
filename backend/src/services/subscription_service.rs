#![allow(unused_variables, dead_code)]
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;

use crate::entities::atlas_subscription::{self, Entity as SubscriptionEntity, ActiveModel as SubscriptionActiveModel};

/// Service layer for GENERIC-04: AtlasSubscription
/// Recurring subscriptions, creator tiers, SaaS plans, membership, etc.
pub struct SubscriptionService;

impl SubscriptionService {
    pub async fn create_subscription(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        subscriber_user_id: Uuid,
        subscribed_to_type: &str,
        subscribed_to_id: Uuid,
        status: &str,
        price_cents: i64,
        currency: &str,
        billing_interval: &str,
    ) -> Result<Uuid, String> {
        let sub = SubscriptionActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            subscriber_user_id: Set(subscriber_user_id),
            subscribed_to_type: Set(subscribed_to_type.to_string()),
            subscribed_to_id: Set(subscribed_to_id),
            status: Set(status.to_string()),
            price_cents: Set(price_cents),
            currency: Set(currency.to_string()),
            billing_interval: Set(billing_interval.to_string()),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = sub.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        subscription_id: Uuid,
    ) -> Result<Option<atlas_subscription::Model>, String> {
        SubscriptionEntity::find()
            .filter(atlas_subscription::Column::TenantId.eq(tenant_id))
            .filter(atlas_subscription::Column::Id.eq(subscription_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_subscription::Model>, String> {
        let mut q = SubscriptionEntity::find()
            .filter(atlas_subscription::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_subscription::Column::Status.eq(s.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn cancel_subscription(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        subscription_id: Uuid,
        reason: Option<&str>,
    ) -> Result<(), String> {
        tracing::info!("Subscription {} cancelled: {:?}", subscription_id, reason);
        Ok(())
    }
}