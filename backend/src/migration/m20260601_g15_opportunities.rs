use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

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
                    .table(AtlasOpportunity::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasOpportunity::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasOpportunity::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasOpportunity::OpportunityType).string().not_null())
                    .col(ColumnDef::new(AtlasOpportunity::Name).string().not_null())
                    .col(ColumnDef::new(AtlasOpportunity::AssetId).uuid().null())
                    .col(ColumnDef::new(AtlasOpportunity::CrmLeadId).uuid().null())
                    .col(ColumnDef::new(AtlasOpportunity::OwnerUserId).uuid().null())
                    .col(ColumnDef::new(AtlasOpportunity::CounterpartyUserId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasOpportunity::Status)
                            .custom(AtlasOpportunityStatus::Table)
                            .not_null()
                            .default(Expr::val("prospecting")),
                    )
                    .col(ColumnDef::new(AtlasOpportunity::DealAmountCents).big_integer().null())
                    .col(ColumnDef::new(AtlasOpportunity::Currency).char_len(3).not_null().default(Expr::val("USD")))
                    .col(ColumnDef::new(AtlasOpportunity::CloseDate).date().null())
                    .col(ColumnDef::new(AtlasOpportunity::ProbabilityPct).small_integer().null())
                    .col(ColumnDef::new(AtlasOpportunity::FinancialInputs).json_binary().null())
                    .col(ColumnDef::new(AtlasOpportunity::ComputedOutputs).json_binary().null())
                    .col(ColumnDef::new(AtlasOpportunity::Notes).text().null())
                    .col(ColumnDef::new(AtlasOpportunity::WonAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasOpportunity::LostAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasOpportunity::LostReason).text().null())
                    .col(
                        ColumnDef::new(AtlasOpportunity::CreatedAt)
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
                    .table(AtlasOpportunity::Table)
                    .col(AtlasOpportunity::TenantId)
                    .col(AtlasOpportunity::OpportunityType)
                    .col(AtlasOpportunity::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_opportunities_asset")
                    .table(AtlasOpportunity::Table)
                    .col(AtlasOpportunity::AssetId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_opportunities_lead")
                    .table(AtlasOpportunity::Table)
                    .col(AtlasOpportunity::CrmLeadId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasOpportunity::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasOpportunityStatus::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasOpportunity {
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
