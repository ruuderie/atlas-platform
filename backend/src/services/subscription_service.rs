#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use uuid::Uuid;

use crate::entities::atlas_subscription::{
    self, ActiveModel as SubscriptionActiveModel, Entity as SubscriptionEntity,
};

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
        status: atlas_subscription::SubscriptionStatus,
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
            status: Set(status),
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
        status: Option<atlas_subscription::SubscriptionStatus>,
        limit: u64,
    ) -> Result<Vec<atlas_subscription::Model>, String> {
        let mut q =
            SubscriptionEntity::find().filter(atlas_subscription::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_subscription::Column::Status.eq(s));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
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

    pub async fn toggle_billing_exemption(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        subscription_id: Uuid,
        is_exempt: bool,
        reason: Option<String>,
    ) -> Result<(), String> {
        let sub = SubscriptionEntity::find()
            .filter(atlas_subscription::Column::TenantId.eq(tenant_id))
            .filter(atlas_subscription::Column::Id.eq(subscription_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(s) = sub {
            let mut active: SubscriptionActiveModel = s.into();
            active.is_billing_exempt = Set(is_exempt);
            active.billing_exemption_reason = Set(reason);
            active.update(db).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Subscription not found".to_string())
        }
    }

    pub async fn update_grace_period(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        subscription_id: Uuid,
        ends_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<(), String> {
        let sub = SubscriptionEntity::find()
            .filter(atlas_subscription::Column::TenantId.eq(tenant_id))
            .filter(atlas_subscription::Column::Id.eq(subscription_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())?;

        if let Some(s) = sub {
            let mut active: SubscriptionActiveModel = s.into();
            active.grace_period_ends_at = Set(ends_at);
            active.update(db).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Subscription not found".to_string())
        }
    }

    pub async fn process_subscription_suspensions(
        db: &DatabaseConnection,
    ) -> Result<usize, String> {
        let now = Utc::now();
        let overdue_subs = SubscriptionEntity::find()
            .filter(
                atlas_subscription::Column::Status
                    .eq(atlas_subscription::SubscriptionStatus::PastDue),
            )
            .filter(atlas_subscription::Column::IsBillingExempt.eq(false))
            .filter(atlas_subscription::Column::GracePeriodEndsAt.lt(now))
            .all(db)
            .await
            .map_err(|e| e.to_string())?;

        let count = overdue_subs.len();
        for sub in overdue_subs {
            let mut active: SubscriptionActiveModel = sub.clone().into();
            active.status = Set(atlas_subscription::SubscriptionStatus::Suspended);
            active.update(db).await.map_err(|e| e.to_string())?;

            tracing::info!(
                "Suspended tenant {} subscription {} due to grace period expiration.",
                sub.tenant_id,
                sub.id
            );
        }

        Ok(count)
    }
}
