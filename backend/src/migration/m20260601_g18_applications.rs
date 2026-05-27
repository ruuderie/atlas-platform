use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-18: atlas_applications — Structured Intake & Onboarding Workflows
/// Rental applications, employment, mortgage, agency onboarding, creator tiers, etc.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasApplication::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasApplication::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasApplication::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasApplication::ApplicationType).string().not_null())
                    .col(ColumnDef::new(AtlasApplication::ApplicantUserId).uuid().not_null())
                    .col(ColumnDef::new(AtlasApplication::TargetAssetId).uuid().null())
                    .col(ColumnDef::new(AtlasApplication::TargetOpportunityId).uuid().null())
                    .col(ColumnDef::new(AtlasApplication::TargetProgram).string().null())
                    .col(ColumnDef::new(AtlasApplication::Status).string().not_null().default(Expr::val("draft")))
                    .col(ColumnDef::new(AtlasApplication::PrimaryApplicationId).uuid().null())
                    .col(ColumnDef::new(AtlasApplication::MonthlyIncomeCents).big_integer().null())
                    .col(ColumnDef::new(AtlasApplication::IncomeCurrency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasApplication::NationalIdType).string().null())
                    .col(ColumnDef::new(AtlasApplication::NationalIdLast4).string_len(4).null())
                    .col(ColumnDef::new(AtlasApplication::ScreeningStatus).string().not_null().default(Expr::val("not_started")))
                    .col(ColumnDef::new(AtlasApplication::ScreeningProvider).string().null())
                    .col(ColumnDef::new(AtlasApplication::ScreeningPassed).boolean().null())
                    .col(ColumnDef::new(AtlasApplication::DisclosuresAccepted).json_binary().null())
                    .col(ColumnDef::new(AtlasApplication::ApplicationMetadata).json_binary().null())
                    .col(ColumnDef::new(AtlasApplication::SubmittedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasApplication::DecidedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasApplication::DecisionReason).text().null())
                    .col(ColumnDef::new(AtlasApplication::ResultingContractId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasApplication::CreatedAt)
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
                    .name("idx_atlas_applications_tenant_type_status")
                    .table(AtlasApplication::Table)
                    .col(AtlasApplication::TenantId)
                    .col(AtlasApplication::ApplicationType)
                    .col(AtlasApplication::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_applications_applicant")
                    .table(AtlasApplication::Table)
                    .col(AtlasApplication::ApplicantUserId)
                    .col(AtlasApplication::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_applications_asset")
                    .table(AtlasApplication::Table)
                    .col(AtlasApplication::TargetAssetId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_applications_primary")
                    .table(AtlasApplication::Table)
                    .col(AtlasApplication::PrimaryApplicationId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasApplication::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasApplication {
    Table,
    Id,
    TenantId,
    ApplicationType,
    ApplicantUserId,
    TargetAssetId,
    TargetOpportunityId,
    TargetProgram,
    Status,
    PrimaryApplicationId,
    MonthlyIncomeCents,
    IncomeCurrency,
    NationalIdType,
    NationalIdLast4,
    ScreeningStatus,
    ScreeningProvider,
    ScreeningPassed,
    DisclosuresAccepted,
    ApplicationMetadata,
    SubmittedAt,
    DecidedAt,
    DecisionReason,
    ResultingContractId,
    CreatedAt,
}
