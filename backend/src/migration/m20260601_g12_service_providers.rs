use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-12: atlas_service_providers — Vendor / Contractor / Agent Registry
/// Supports tenant-scoped, platform-scoped, and marketplace providers.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasProviderScope::Table)
                    .values([
                        AtlasProviderScope::Tenant,
                        AtlasProviderScope::Platform,
                        AtlasProviderScope::Marketplace,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasProviderStatus::Table)
                    .values([
                        AtlasProviderStatus::Preferred,
                        AtlasProviderStatus::Active,
                        AtlasProviderStatus::Probationary,
                        AtlasProviderStatus::Suspended,
                        AtlasProviderStatus::Blocked,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasServiceProviders::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasServiceProviders::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::UserId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::Scope)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("tenant")),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::BusinessName)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::ServiceCategories)
                            .json_binary()
                            .not_null()
                            .default(Expr::val("[]")),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::RatingAvg)
                            .double()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::RatingCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::PreferredPaymentRail)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::BtcWalletAddress)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::StripeConnectId)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::IsInsured)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::IsBonded)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AtlasServiceProviders::ProfileMetadata)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasServiceProviders::Notes).text().null())
                    .col(
                        ColumnDef::new(AtlasServiceProviders::CreatedAt)
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
                    .name("idx_atlas_service_providers_scope")
                    .table(AtlasServiceProviders::Table)
                    .col(AtlasServiceProviders::TenantId)
                    .col(AtlasServiceProviders::Scope)
                    .col(AtlasServiceProviders::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_service_providers_categories")
                    .table(AtlasServiceProviders::Table)
                    .col(AtlasServiceProviders::ServiceCategories)
                    .to_owned(),
            )
            .await?;

        // Unique constraint on tenant + user
        manager
            .create_index(
                Index::create()
                    .name("uq_atlas_service_providers_tenant_user")
                    .table(AtlasServiceProviders::Table)
                    .col(AtlasServiceProviders::TenantId)
                    .col(AtlasServiceProviders::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasServiceProviders::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasProviderStatus::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasProviderScope::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasServiceProviders {
    Table,
    Id,
    TenantId,
    UserId,
    Scope,
    BusinessName,
    ServiceCategories,
    Status,
    RatingAvg,
    RatingCount,
    PreferredPaymentRail,
    BtcWalletAddress,
    StripeConnectId,
    IsInsured,
    IsBonded,
    ProfileMetadata,
    Notes,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasProviderScope {
    Table,
    Tenant,
    Platform,
    Marketplace,
}

#[derive(DeriveIden)]
enum AtlasProviderStatus {
    Table,
    Preferred,
    Active,
    Probationary,
    Suspended,
    Blocked,
}
