use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// GENERIC-08: atlas_ai_tasks — Async LLM / AI Processing Queue
///
/// Enforces the rule that expensive AI calls should never be done inline
/// in handlers or server functions. All AI work is queued here and processed
/// by background jobs.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasAiTasks::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasAiTasks::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasAiTasks::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasAiTasks::TaskType).string().not_null())
                    .col(ColumnDef::new(AtlasAiTasks::Model).string().null())
                    .col(ColumnDef::new(AtlasAiTasks::InputPayload).json_binary().not_null())
                    .col(ColumnDef::new(AtlasAiTasks::OutputPayload).json_binary().null())
                    .col(ColumnDef::new(AtlasAiTasks::SourceEntityType).string().null())
                    .col(ColumnDef::new(AtlasAiTasks::SourceEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasAiTasks::CallbackEntityType).string().null())
                    .col(ColumnDef::new(AtlasAiTasks::CallbackEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasAiTasks::CallbackField).string().null())
                    .col(ColumnDef::new(AtlasAiTasks::Status).string().not_null().default(Expr::val("queued")))
                    .col(ColumnDef::new(AtlasAiTasks::ErrorMessage).text().null())
                    .col(ColumnDef::new(AtlasAiTasks::RetryCount).integer().not_null().default(0))
                    .col(
                        ColumnDef::new(AtlasAiTasks::QueuedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AtlasAiTasks::StartedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasAiTasks::CompletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasAiTasks::InputTokens).integer().null())
                    .col(ColumnDef::new(AtlasAiTasks::OutputTokens).integer().null())
                    .col(ColumnDef::new(AtlasAiTasks::EstimatedCostMicroUsd).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ai_tasks_status")
                    .table(AtlasAiTasks::Table)
                    .col(AtlasAiTasks::TenantId)
                    .col(AtlasAiTasks::Status)
                    .col(AtlasAiTasks::QueuedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ai_tasks_entity")
                    .table(AtlasAiTasks::Table)
                    .col(AtlasAiTasks::TenantId)
                    .col(AtlasAiTasks::SourceEntityType)
                    .col(AtlasAiTasks::SourceEntityId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasAiTasks::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasAiTasks {
    Table,
    Id,
    TenantId,
    TaskType,
    Model,
    InputPayload,
    OutputPayload,
    SourceEntityType,
    SourceEntityId,
    CallbackEntityType,
    CallbackEntityId,
    CallbackField,
    Status,
    ErrorMessage,
    RetryCount,
    QueuedAt,
    StartedAt,
    CompletedAt,
    InputTokens,
    OutputTokens,
    EstimatedCostMicroUsd,
}
