use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-17: atlas_tax_events + atlas_tax_filings
/// Revenue tax tracking and periodic filing/remittance ledger.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Tax Events
        manager
            .create_table(
                Table::create()
                    .table(AtlasTaxEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasTaxEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasTaxEvents::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::TaxType).string().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::JurisdictionCode).string().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::SourceIntegrationId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvents::SourceLedgerEntryId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvents::SourceEntityType).string().null())
                    .col(ColumnDef::new(AtlasTaxEvents::SourceEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvents::GrossRevenueCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::ExcludedFeesCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxEvents::TaxableRevenueCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::TaxRate).double().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::TaxAmountCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxEvents::RemittedBy).string().not_null().default(Expr::val("host")))
                    .col(ColumnDef::new(AtlasTaxEvents::TaxFilingId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvents::EventDate).date().not_null())
                    .col(
                        ColumnDef::new(AtlasTaxEvents::CreatedAt)
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
                    .name("idx_atlas_tax_events_jurisdiction")
                    .table(AtlasTaxEvents::Table)
                    .col(AtlasTaxEvents::TenantId)
                    .col(AtlasTaxEvents::TaxType)
                    .col(AtlasTaxEvents::JurisdictionCode)
                    .col(AtlasTaxEvents::EventDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_tax_events_filing")
                    .table(AtlasTaxEvents::Table)
                    .col(AtlasTaxEvents::TaxFilingId)
                    .to_owned(),
            )
            .await?;

        // Tax Filings
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasTaxFilingStatus::Table)
                    .values([
                        AtlasTaxFilingStatus::Draft,
                        AtlasTaxFilingStatus::Filed,
                        AtlasTaxFilingStatus::Amended,
                        AtlasTaxFilingStatus::Accepted,
                        AtlasTaxFilingStatus::Disputed,
                        AtlasTaxFilingStatus::Overdue,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasTaxFilings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasTaxFilings::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasTaxFilings::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasTaxFilings::TaxType).string().not_null())
                    .col(ColumnDef::new(AtlasTaxFilings::JurisdictionCode).string().not_null())
                    .col(ColumnDef::new(AtlasTaxFilings::PeriodYear).small_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxFilings::PeriodMonth).small_integer().null())
                    .col(ColumnDef::new(AtlasTaxFilings::PeriodQuarter).small_integer().null())
                    .col(ColumnDef::new(AtlasTaxFilings::TotalTaxableRevenueCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxFilings::TotalTaxOwedCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxFilings::PlatformRemittedCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxFilings::HostOwedCents).big_integer().not_null().default(0))
                    .col(
                        ColumnDef::new(AtlasTaxFilings::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("draft")),
                    )
                    .col(ColumnDef::new(AtlasTaxFilings::DueDate).date().null())
                    .col(ColumnDef::new(AtlasTaxFilings::FiledAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasTaxFilings::ConfirmationNumber).string().null())
                    .col(ColumnDef::new(AtlasTaxFilings::FilingDocumentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasTaxFilings::CreatedAt)
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
                    .name("uq_atlas_tax_filings_period")
                    .table(AtlasTaxFilings::Table)
                    .col(AtlasTaxFilings::TenantId)
                    .col(AtlasTaxFilings::TaxType)
                    .col(AtlasTaxFilings::JurisdictionCode)
                    .col(AtlasTaxFilings::PeriodYear)
                    .col(AtlasTaxFilings::PeriodMonth)
                    .col(AtlasTaxFilings::PeriodQuarter)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasTaxFilings::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasTaxEvents::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasTaxFilingStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasTaxEvents {
    Table,
    Id,
    TenantId,
    TaxType,
    JurisdictionCode,
    SourceIntegrationId,
    SourceLedgerEntryId,
    SourceEntityType,
    SourceEntityId,
    GrossRevenueCents,
    ExcludedFeesCents,
    TaxableRevenueCents,
    TaxRate,
    TaxAmountCents,
    RemittedBy,
    TaxFilingId,
    EventDate,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasTaxFilings {
    Table,
    Id,
    TenantId,
    TaxType,
    JurisdictionCode,
    PeriodYear,
    PeriodMonth,
    PeriodQuarter,
    TotalTaxableRevenueCents,
    TotalTaxOwedCents,
    PlatformRemittedCents,
    HostOwedCents,
    Status,
    DueDate,
    FiledAt,
    ConfirmationNumber,
    FilingDocumentId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasTaxFilingStatus {
    Table,
    Draft,
    Filed,
    Amended,
    Accepted,
    Disputed,
    Overdue,
}
