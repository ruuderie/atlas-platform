use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

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
                    .table(AtlasServiceProvider::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasServiceProvider::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasServiceProvider::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasServiceProvider::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(AtlasServiceProvider::Scope)
                            .custom(AtlasProviderScope::Table)
                            .not_null()
                            .default(Expr::val("tenant")),
                    )
                    .col(ColumnDef::new(AtlasServiceProvider::BusinessName).string().null())
                    .col(ColumnDef::new(AtlasServiceProvider::ServiceCategories).json_binary().not_null().default(Expr::val("[]")))
                    .col(
                        ColumnDef::new(AtlasServiceProvider::Status)
                            .custom(AtlasProviderStatus::Table)
                            .not_null()
                            .default(Expr::val("active")),
                    )
                    .col(ColumnDef::new(AtlasServiceProvider::RatingAvg).decimal_len(3, 2).null())
                    .col(ColumnDef::new(AtlasServiceProvider::RatingCount).integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasServiceProvider::PreferredPaymentRail).string().null())
                    .col(ColumnDef::new(AtlasServiceProvider::BtcWalletAddress).string().null())
                    .col(ColumnDef::new(AtlasServiceProvider::StripeConnectId).string().null())
                    .col(ColumnDef::new(AtlasServiceProvider::IsInsured).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasServiceProvider::IsBonded).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasServiceProvider::ProfileMetadata).json_binary().null())
                    .col(ColumnDef::new(AtlasServiceProvider::Notes).text().null())
                    .col(
                        ColumnDef::new(AtlasServiceProvider::CreatedAt)
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
                    .table(AtlasServiceProvider::Table)
                    .col(AtlasServiceProvider::TenantId)
                    .col(AtlasServiceProvider::Scope)
                    .col(AtlasServiceProvider::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_service_providers_categories")
                    .table(AtlasServiceProvider::Table)
                    .col(AtlasServiceProvider::ServiceCategories)
                    .to_owned(),
            )
            .await?;

        // Unique constraint on tenant + user
        manager
            .create_index(
                Index::create()
                    .name("uq_atlas_service_providers_tenant_user")
                    .table(AtlasServiceProvider::Table)
                    .col(AtlasServiceProvider::TenantId)
                    .col(AtlasServiceProvider::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasServiceProvider::Table).to_owned())
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
enum AtlasServiceProvider {
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
