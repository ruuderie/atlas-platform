use sea_orm_migration::prelude::extension::postgres::Type;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-15: atlas_opportunities — Deal / Pipeline Object
/// Used for wholesaling leads, insurance submissions, corporate contracts, agency onboarding, etc.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasOpportunityStatus::Table)
                    .values([
                        AtlasOpportunityStatus::Prospecting,
                        AtlasOpportunityStatus::Analysis,
                        AtlasOpportunityStatus::OfferSubmitted,
                        AtlasOpportunityStatus::Negotiating,
                        AtlasOpportunityStatus::UnderContract,
                        AtlasOpportunityStatus::ClosedWon,
                        AtlasOpportunityStatus::ClosedLost,
                        AtlasOpportunityStatus::OnHold,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasOpportunities::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasOpportunities::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::OpportunityType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(AtlasOpportunities::Name).string().not_null())
                    .col(ColumnDef::new(AtlasOpportunities::AssetId).uuid().null())
                    .col(ColumnDef::new(AtlasOpportunities::CrmLeadId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasOpportunities::OwnerUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::CounterpartyUserId)
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("prospecting")),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::DealAmountCents)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::Currency)
                            .char_len(3)
                            .not_null()
                            .default(Expr::val("USD")),
                    )
                    .col(ColumnDef::new(AtlasOpportunities::CloseDate).date().null())
                    .col(
                        ColumnDef::new(AtlasOpportunities::ProbabilityPct)
                            .small_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::FinancialInputs)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::ComputedOutputs)
                            .json_binary()
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasOpportunities::Notes).text().null())
                    .col(
                        ColumnDef::new(AtlasOpportunities::WonAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasOpportunities::LostAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(AtlasOpportunities::LostReason).text().null())
                    .col(
                        ColumnDef::new(AtlasOpportunities::CreatedAt)
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
                    .name("idx_atlas_opportunities_tenant_type_status")
                    .table(AtlasOpportunities::Table)
                    .col(AtlasOpportunities::TenantId)
                    .col(AtlasOpportunities::OpportunityType)
                    .col(AtlasOpportunities::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_opportunities_asset")
                    .table(AtlasOpportunities::Table)
                    .col(AtlasOpportunities::AssetId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_opportunities_lead")
                    .table(AtlasOpportunities::Table)
                    .col(AtlasOpportunities::CrmLeadId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasOpportunities::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasOpportunityStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasOpportunities {
    Table,
    Id,
    TenantId,
    OpportunityType,
    Name,
    AssetId,
    CrmLeadId,
    OwnerUserId,
    CounterpartyUserId,
    Status,
    DealAmountCents,
    Currency,
    CloseDate,
    ProbabilityPct,
    FinancialInputs,
    ComputedOutputs,
    Notes,
    WonAt,
    LostAt,
    LostReason,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasOpportunityStatus {
    Table,
    Prospecting,
    Analysis,
    OfferSubmitted,
    Negotiating,
    UnderContract,
    ClosedWon,
    ClosedLost,
    OnHold,
}
