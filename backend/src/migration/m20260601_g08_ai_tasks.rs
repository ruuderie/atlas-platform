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
                    .table(AtlasAiTask::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasAiTask::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AtlasAiTask::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AtlasAiTask::TaskType).string().not_null())
                    .col(ColumnDef::new(AtlasAiTask::Model).string().null())
                    .col(ColumnDef::new(AtlasAiTask::InputPayload).json_binary().not_null())
                    .col(ColumnDef::new(AtlasAiTask::OutputPayload).json_binary().null())
                    .col(ColumnDef::new(AtlasAiTask::SourceEntityType).string().null())
                    .col(ColumnDef::new(AtlasAiTask::SourceEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasAiTask::CallbackEntityType).string().null())
                    .col(ColumnDef::new(AtlasAiTask::CallbackEntityId).uuid().null())
                    .col(ColumnDef::new(AtlasAiTask::CallbackField).string().null())
                    .col(ColumnDef::new(AtlasAiTask::Status).string().not_null().default(Expr::val("queued")))
                    .col(ColumnDef::new(AtlasAiTask::ErrorMessage).text().null())
                    .col(ColumnDef::new(AtlasAiTask::RetryCount).integer().not_null().default(0))
                    .col(
                        ColumnDef::new(AtlasAiTask::QueuedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(AtlasAiTask::StartedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasAiTask::CompletedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(AtlasAiTask::InputTokens).integer().null())
                    .col(ColumnDef::new(AtlasAiTask::OutputTokens).integer().null())
                    .col(ColumnDef::new(AtlasAiTask::EstimatedCostMicroUsd).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ai_tasks_status")
                    .table(AtlasAiTask::Table)
                    .col(AtlasAiTask::TenantId)
                    .col(AtlasAiTask::Status)
                    .col(AtlasAiTask::QueuedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_atlas_ai_tasks_entity")
                    .table(AtlasAiTask::Table)
                    .col(AtlasAiTask::TenantId)
                    .col(AtlasAiTask::SourceEntityType)
                    .col(AtlasAiTask::SourceEntityId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasAiTask::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasAiTask {
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
