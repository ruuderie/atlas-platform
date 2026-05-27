use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-04: atlas_subscriptions — B2C Recurring Billing
///
/// Tracks user-level recurring subscriptions (creator tiers, city plans, STR compliance plans, etc.).
/// This is distinct from the platform's own B2B SaaS billing (`tenant_subscription`).
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasSubscriptionStatus::Table)
                    .values([
                        AtlasSubscriptionStatus::Trialing,
                        AtlasSubscriptionStatus::Active,
                        AtlasSubscriptionStatus::PastDue,
                        AtlasSubscriptionStatus::Canceled,
                        AtlasSubscriptionStatus::Paused,
                        AtlasSubscriptionStatus::Incomplete,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasSubscription::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasSubscription::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasSubscription::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasSubscription::SubscriberUserId).uuid().not_null())
                    .col(ColumnDef::new(AtlasSubscription::SubscribedToType).string().not_null())
                    .col(ColumnDef::new(AtlasSubscription::SubscribedToId).uuid().not_null())
                    .col(ColumnDef::new(AtlasSubscription::BillingPlanId).uuid().null())
                    .col(ColumnDef::new(AtlasSubscription::PriceCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasSubscription::Currency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasSubscription::BillingInterval).string().not_null().default(Expr::val("monthly")))
                    .col(ColumnDef::new(AtlasSubscription::StripeSubscriptionId).string().null())
                    .col(ColumnDef::new(AtlasSubscription::StripeCustomerId).string().null())
                    .col(
                        ColumnDef::new(AtlasSubscription::Status)
                            .custom(AtlasSubscriptionStatus::Table)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    .col(ColumnDef::new(AtlasSubscription::TrialEndsAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasSubscription::CurrentPeriodStart).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasSubscription::CurrentPeriodEnd).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasSubscription::CanceledAt).timestamp_with_time_zone().null())
                    .col(
                        ColumnDef::new(AtlasSubscription::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_subscriptions_subscriber")
                    .table(AtlasSubscription::Table)
                    .col(AtlasSubscription::TenantId)
                    .col(AtlasSubscription::SubscriberUserId)
                    .col(AtlasSubscription::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_subscriptions_entity")
                    .table(AtlasSubscription::Table)
                    .col(AtlasSubscription::TenantId)
                    .col(AtlasSubscription::SubscribedToType)
                    .col(AtlasSubscription::SubscribedToId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasSubscription::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasSubscriptionStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasSubscription {
    Table,
    Id,
    TenantId,
    SubscriberUserId,
    SubscribedToType,
    SubscribedToId,
    BillingPlanId,
    PriceCents,
    Currency,
    BillingInterval,
    StripeSubscriptionId,
    StripeCustomerId,
    Status,
    TrialEndsAt,
    CurrentPeriodStart,
    CurrentPeriodEnd,
    CanceledAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasSubscriptionStatus {
    Table,
    Trialing,
    Active,
    PastDue,
    Canceled,
    Paused,
    Incomplete,
}
