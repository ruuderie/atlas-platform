use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create page_views table
        manager
            .create_table(
                Table::create()
                    .table(PageViews::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PageViews::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PageViews::TenantId).uuid().not_null())
                    .col(ColumnDef::new(PageViews::Path).string().not_null())
                    .col(ColumnDef::new(PageViews::UserAgent).string().null())
                    .col(
                        ColumnDef::new(PageViews::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create bitcoin_blocks table
        manager
            .create_table(
                Table::create()
                    .table(BitcoinBlocks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(BitcoinBlocks::Id).string().not_null().primary_key())
                    .col(ColumnDef::new(BitcoinBlocks::TenantId).uuid().not_null())
                    .col(ColumnDef::new(BitcoinBlocks::Height).big_integer().not_null())
                    .col(ColumnDef::new(BitcoinBlocks::Timestamp).big_integer().not_null())
                    .col(ColumnDef::new(BitcoinBlocks::TxCount).integer().not_null())
                    .col(ColumnDef::new(BitcoinBlocks::Size).integer().not_null())
                    .col(ColumnDef::new(BitcoinBlocks::Weight).integer().not_null())
                    .col(ColumnDef::new(BitcoinBlocks::Difficulty).double().not_null())
                    .col(
                        ColumnDef::new(BitcoinBlocks::FetchedAt)
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
                    .if_not_exists()
                    .name("idx_bitcoin_blocks_tenant_height")
                    .table(BitcoinBlocks::Table)
                    .col(BitcoinBlocks::TenantId)
                    .col(BitcoinBlocks::Height)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create tenant_background_jobs table
        manager
            .create_table(
                Table::create()
                    .table(TenantBackgroundJobs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TenantBackgroundJobs::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(TenantBackgroundJobs::TenantId).uuid().not_null())
                    .col(ColumnDef::new(TenantBackgroundJobs::JobType).string().not_null())
                    .col(ColumnDef::new(TenantBackgroundJobs::Config).json_binary().null())
                    .col(
                        ColumnDef::new(TenantBackgroundJobs::IntervalSeconds)
                            .integer()
                            .not_null()
                            .default(600),
                    )
                    .col(
                        ColumnDef::new(TenantBackgroundJobs::LastRun)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(TenantBackgroundJobs::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantBackgroundJobs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BitcoinBlocks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PageViews::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
enum PageViews {
    Table,
    Id,
    TenantId,
    Path,
    UserAgent,
    CreatedAt,
}

#[derive(Iden)]
enum BitcoinBlocks {
    Table,
    Id, // block hash string
    TenantId,
    Height,
    Timestamp,
    TxCount,
    Size,
    Weight,
    Difficulty,
    FetchedAt,
}

#[derive(Iden)]
pub enum TenantBackgroundJobs {
    Table,
    Id,
    TenantId,
    JobType,
    Config,
    IntervalSeconds,
    LastRun,
    IsActive,
}
