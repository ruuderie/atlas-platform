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
                    .table(AtlasTaxEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasTaxEvent::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasTaxEvent::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::TaxType).string().not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::JurisdictionCode).string().not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::SourceIntegrationId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvent::SourceLedgerEntryId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvent::SourceEntityType).string().null())
                    .col(ColumnDef::new(AtlasTaxEvent::SourceEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvent::GrossRevenueCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::ExcludedFeesCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxEvent::TaxableRevenueCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::TaxRate).decimal_len(6, 5).not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::TaxAmountCents).big_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxEvent::RemittedBy).string().not_null().default(Expr::val("host")))
                    .col(ColumnDef::new(AtlasTaxEvent::TaxFilingId).uuid().null())
                    .col(ColumnDef::new(AtlasTaxEvent::EventDate).date().not_null())
                    .col(
                        ColumnDef::new(AtlasTaxEvent::CreatedAt)
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
                    .table(AtlasTaxEvent::Table)
                    .col(AtlasTaxEvent::TenantId)
                    .col(AtlasTaxEvent::TaxType)
                    .col(AtlasTaxEvent::JurisdictionCode)
                    .col(AtlasTaxEvent::EventDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_tax_events_filing")
                    .table(AtlasTaxEvent::Table)
                    .col(AtlasTaxEvent::TaxFilingId)
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
                    .table(AtlasTaxFiling::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasTaxFiling::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasTaxFiling::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasTaxFiling::TaxType).string().not_null())
                    .col(ColumnDef::new(AtlasTaxFiling::JurisdictionCode).string().not_null())
                    .col(ColumnDef::new(AtlasTaxFiling::PeriodYear).small_integer().not_null())
                    .col(ColumnDef::new(AtlasTaxFiling::PeriodMonth).small_integer().null())
                    .col(ColumnDef::new(AtlasTaxFiling::PeriodQuarter).small_integer().null())
                    .col(ColumnDef::new(AtlasTaxFiling::TotalTaxableRevenueCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxFiling::TotalTaxOwedCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxFiling::PlatformRemittedCents).big_integer().not_null().default(0))
                    .col(ColumnDef::new(AtlasTaxFiling::HostOwedCents).big_integer().not_null().default(0))
                    .col(
                        ColumnDef::new(AtlasTaxFiling::Status)
                            .custom(AtlasTaxFilingStatus::Table)
                            .not_null()
                            .default(Expr::val("draft")),
                    )
                    .col(ColumnDef::new(AtlasTaxFiling::DueDate).date().null())
                    .col(ColumnDef::new(AtlasTaxFiling::FiledAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasTaxFiling::ConfirmationNumber).string().null())
                    .col(ColumnDef::new(AtlasTaxFiling::FilingDocumentId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasTaxFiling::CreatedAt)
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
                    .table(AtlasTaxFiling::Table)
                    .col(AtlasTaxFiling::TenantId)
                    .col(AtlasTaxFiling::TaxType)
                    .col(AtlasTaxFiling::JurisdictionCode)
                    .col(AtlasTaxFiling::PeriodYear)
                    .col(AtlasTaxFiling::PeriodMonth)
                    .col(AtlasTaxFiling::PeriodQuarter)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasTaxFiling::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AtlasTaxEvent::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasTaxFilingStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasTaxEvent {
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
enum AtlasTaxFiling {
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
