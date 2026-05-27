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
                    .table(AtlasContract::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasContract::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasContract::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasContract::ContractType).string().not_null())
                    .col(ColumnDef::new(AtlasContract::CounterpartyUserId).uuid().null())
                    .col(ColumnDef::new(AtlasContract::AssetId).uuid().null()) // FK to atlas_assets
                    .col(ColumnDef::new(AtlasContract::StartDate).date().not_null())
                    .col(ColumnDef::new(AtlasContract::EndDate).date().null())
                    .col(ColumnDef::new(AtlasContract::AutoRenew).boolean().not_null().default(false))
                    .col(ColumnDef::new(AtlasContract::RecurringAmountCents).big_integer().null())
                    .col(ColumnDef::new(AtlasContract::Currency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasContract::BillingInterval).string().not_null().default(Expr::val("monthly")))
                    .col(
                        ColumnDef::new(AtlasContract::Status)
                            .custom(AtlasContractStatus::Table)
                            .not_null()
                            .default(Expr::val("draft")),
                    )
                    .col(ColumnDef::new(AtlasContract::SignedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasContract::TerminatedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasContract::TerminationReason).text().null())
                    .col(ColumnDef::new(AtlasContract::TermsMetadata).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasContract::CreatedAt)
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
                    .table(AtlasContract::Table)
                    .col(AtlasContract::TenantId)
                    .col(AtlasContract::ContractType)
                    .col(AtlasContract::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contracts_asset")
                    .table(AtlasContract::Table)
                    .col(AtlasContract::AssetId)
                    .col(AtlasContract::ContractType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_contracts_counterparty")
                    .table(AtlasContract::Table)
                    .col(AtlasContract::CounterpartyUserId)
                    .col(AtlasContract::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasContract::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasContractStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasContract {
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
