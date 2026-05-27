use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-13: atlas_cases — Salesforce-Style Case / Work Item Object
/// The most reusable domain object. Covers maintenance, claims, tasks, support tickets, compliance violations, etc.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasCasePriority::Table)
                    .values([
                        AtlasCasePriority::Critical,
                        AtlasCasePriority::High,
                        AtlasCasePriority::Medium,
                        AtlasCasePriority::Low,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(AtlasCaseStatus::Table)
                    .values([
                        AtlasCaseStatus::New,
                        AtlasCaseStatus::Open,
                        AtlasCaseStatus::Assigned,
                        AtlasCaseStatus::InProgress,
                        AtlasCaseStatus::PendingCustomer,
                        AtlasCaseStatus::PendingParts,
                        AtlasCaseStatus::PendingApproval,
                        AtlasCaseStatus::Resolved,
                        AtlasCaseStatus::Closed,
                        AtlasCaseStatus::Cancelled,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AtlasCases::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasCases::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasCases::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasCases::CaseType).string().not_null())
                    .col(ColumnDef::new(AtlasCases::ReportedByUserId).uuid().null())
                    .col(ColumnDef::new(AtlasCases::AssetId).uuid().null()) // FK to atlas_assets
                    .col(ColumnDef::new(AtlasCases::ContractId).uuid().null()) // FK to atlas_contracts
                    .col(ColumnDef::new(AtlasCases::AssignedServiceProviderId).uuid().null())
                    .col(ColumnDef::new(AtlasCases::AssignedUserId).uuid().null())
                    .col(
                        ColumnDef::new(AtlasCases::Priority)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("medium")),
                    )
                    .col(
                        ColumnDef::new(AtlasCases::Status)
                            .string_len(30)
                            .not_null()
                            .default(Expr::val("new")),
                    )
                    .col(ColumnDef::new(AtlasCases::Subject).string().not_null())
                    .col(ColumnDef::new(AtlasCases::Description).text().null())
                    .col(ColumnDef::new(AtlasCases::ScheduledAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasCases::CompletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasCases::EstimatedCostCents).big_integer().null())
                    .col(ColumnDef::new(AtlasCases::ActualCostCents).big_integer().null())
                    .col(ColumnDef::new(AtlasCases::LedgerEntryId).uuid().null())
                    .col(ColumnDef::new(AtlasCases::PrimaryAttachmentId).uuid().null())
                    .col(ColumnDef::new(AtlasCases::WsRoomId).uuid().null())
                    .col(ColumnDef::new(AtlasCases::CaseMetadata).json_binary().null())
                    .col(
                        ColumnDef::new(AtlasCases::CreatedAt)
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
                    .name("idx_atlas_cases_tenant_type_status")
                    .table(AtlasCases::Table)
                    .col(AtlasCases::TenantId)
                    .col(AtlasCases::CaseType)
                    .col(AtlasCases::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_cases_asset")
                    .table(AtlasCases::Table)
                    .col(AtlasCases::AssetId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_cases_provider")
                    .table(AtlasCases::Table)
                    .col(AtlasCases::AssignedServiceProviderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_cases_priority_active")
                    .table(AtlasCases::Table)
                    .col(AtlasCases::TenantId)
                    .col(AtlasCases::Priority)
                    .col(AtlasCases::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasCases::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasCaseStatus::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(AtlasCasePriority::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasCases {
    Table,
    Id,
    TenantId,
    CaseType,
    ReportedByUserId,
    AssetId,
    ContractId,
    AssignedServiceProviderId,
    AssignedUserId,
    Priority,
    Status,
    Subject,
    Description,
    ScheduledAt,
    CompletedAt,
    EstimatedCostCents,
    ActualCostCents,
    LedgerEntryId,
    PrimaryAttachmentId,
    WsRoomId,
    CaseMetadata,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AtlasCasePriority {
    Table,
    Critical,
    High,
    Medium,
    Low,
}

#[derive(DeriveIden)]
enum AtlasCaseStatus {
    Table,
    New,
    Open,
    Assigned,
    InProgress,
    PendingCustomer,
    PendingParts,
    PendingApproval,
    Resolved,
    Closed,
    Cancelled,
}
