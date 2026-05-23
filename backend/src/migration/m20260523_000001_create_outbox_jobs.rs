use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OutboxJob::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OutboxJob::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OutboxJob::TenantId).uuid().not_null())
                    .col(ColumnDef::new(OutboxJob::JobType).string().not_null())
                    .col(ColumnDef::new(OutboxJob::Payload).json_binary().not_null())
                    .col(ColumnDef::new(OutboxJob::Status).string().not_null())
                    .col(ColumnDef::new(OutboxJob::Attempts).integer().not_null().default(0))
                    .col(ColumnDef::new(OutboxJob::ErrorMessage).text())
                    .col(ColumnDef::new(OutboxJob::LockedBy).string())
                    .col(ColumnDef::new(OutboxJob::LockedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(OutboxJob::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(OutboxJob::RunAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create index on status + run_at for extremely efficient FIFO queue checks
        manager
            .create_index(
                Index::create()
                    .name("idx_outbox_jobs_status_run_at")
                    .table(OutboxJob::Table)
                    .col(OutboxJob::Status)
                    .col(OutboxJob::RunAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OutboxJob::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OutboxJob {
    Table,
    Id,
    TenantId,
    JobType,
    Payload,
    Status,
    Attempts,
    ErrorMessage,
    LockedBy,
    LockedAt,
    CreatedAt,
    RunAt,
}
