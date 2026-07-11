use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

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
                    .table(AtlasSubscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasSubscriptions::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::SubscriberUserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::SubscribedToType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::SubscribedToId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::BillingPlanId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::PriceCents)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::BillingInterval)
                            .string()
                            .not_null()
                            .default(Expr::val("monthly")),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::StripeSubscriptionId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::StripeCustomerId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::TrialEndsAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::CurrentPeriodStart)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::CurrentPeriodEnd)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::CanceledAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasSubscriptions::CreatedAt)
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
                    .table(AtlasSubscriptions::Table)
                    .col(AtlasSubscriptions::TenantId)
                    .col(AtlasSubscriptions::SubscriberUserId)
                    .col(AtlasSubscriptions::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_subscriptions_entity")
                    .table(AtlasSubscriptions::Table)
                    .col(AtlasSubscriptions::TenantId)
                    .col(AtlasSubscriptions::SubscribedToType)
                    .col(AtlasSubscriptions::SubscribedToId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasSubscriptions::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasSubscriptionStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasSubscriptions {
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
