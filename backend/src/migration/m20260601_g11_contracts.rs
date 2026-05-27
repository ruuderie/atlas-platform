use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-11: atlas_contracts — Legal Agreement Registry
/// Covers leases, insurance policies, corporate rate agreements, SLAs, alliance agreements, etc.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasContractStatus::Table)
                    .values([
                        AtlasContractStatus::Draft,
                        AtlasContractStatus::PendingSignature,
                        AtlasContractStatus::Active,
                        AtlasContractStatus::Expired,
                        AtlasContractStatus::Terminated,
                        AtlasContractStatus::Renewed,
                        AtlasContractStatus::Suspended,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasContracts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasContracts::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasContracts::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasContracts::ContractType).string().not_null())
                    .col(ColumnDef::new(AtlasContracts::CounterpartyUserId).uuid().null())
                    .col(ColumnDef::new(AtlasContracts::AssetId).uuid().null()) // FK to atlas_assets
                    .col(ColumnDef::new(AtlasContracts::StartDate).date().not_null())
                    .col(ColumnDef::new(AtlasContracts::EndDate).date().null())
                    .col(ColumnDef::new(AtlasContracts::AutoRenew).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasContracts::RecurringAmountCents).big_integer().null())
                    .col(ColumnDef::new(AtlasContracts::Currency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasContracts::BillingInterval).string().not_null().default(Expr::val("monthly")))
                    .col(
                        ColumnDef::new(AtlasContracts::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("draft")),
                    )
                    .col(ColumnDef::new(AtlasContracts::SignedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasContracts::TerminatedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasContracts::TerminationReason).text().null())
                    .col(ColumnDef::new(AtlasContracts::TermsMetadata).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasContracts::CreatedAt)
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
                    .name("idx_atlas_contracts_tenant_type_status")
                    .table(AtlasContracts::Table)
                    .col(AtlasContracts::TenantId)
                    .col(AtlasContracts::ContractType)
                    .col(AtlasContracts::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contracts_asset")
                    .table(AtlasContracts::Table)
                    .col(AtlasContracts::AssetId)
                    .col(AtlasContracts::ContractType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contracts_counterparty")
                    .table(AtlasContracts::Table)
                    .col(AtlasContracts::CounterpartyUserId)
                    .col(AtlasContracts::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasContracts::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasContractStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasContracts {
    Table,
    Id,
    TenantId,
    ContractType,
    CounterpartyUserId,
    AssetId,
    StartDate,
    EndDate,
    AutoRenew,
    RecurringAmountCents,
    Currency,
    BillingInterval,
    Status,
    SignedAt,
    TerminatedAt,
    TerminationReason,
    TermsMetadata,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasContractStatus {
    Table,
    Draft,
    PendingSignature,
    Active,
    Expired,
    Terminated,
    Renewed,
    Suspended,
}
