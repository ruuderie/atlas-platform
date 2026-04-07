use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create telemetry_events table
        manager
            .create_table(
                Table::create()
                    .table(TelemetryEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TelemetryEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TelemetryEvents::TenantId).uuid().not_null())
                    .col(ColumnDef::new(TelemetryEvents::EventSource).string().not_null())
                    .col(ColumnDef::new(TelemetryEvents::EventType).string().not_null())
                    .col(ColumnDef::new(TelemetryEvents::EventPayload).json_binary().null())
                    .col(
                        ColumnDef::new(TelemetryEvents::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TelemetryEvents::Processed)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Create platform_metrics_daily table
        manager
            .create_table(
                Table::create()
                    .table(PlatformMetricsDaily::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PlatformMetricsDaily::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PlatformMetricsDaily::Date).date().not_null())
                    .col(ColumnDef::new(PlatformMetricsDaily::TenantId).uuid().not_null())
                    .col(ColumnDef::new(PlatformMetricsDaily::MetricSource).string().not_null())
                    .col(ColumnDef::new(PlatformMetricsDaily::MetricKey).string().not_null())
                    .col(ColumnDef::new(PlatformMetricsDaily::MetricValue).float().not_null()) // float to allow averages or precise counts (money, index)
                    .to_owned(),
            )
            .await?;

        // Add Unique index to support ON CONFLICT DO UPDATE upserts
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_platform_metrics_daily_unique_key")
                    .table(PlatformMetricsDaily::Table)
                    .col(PlatformMetricsDaily::Date)
                    .col(PlatformMetricsDaily::TenantId)
                    .col(PlatformMetricsDaily::MetricSource)
                    .col(PlatformMetricsDaily::MetricKey)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PlatformMetricsDaily::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TelemetryEvents::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum TelemetryEvents {
    Table,
    Id,
    TenantId,
    EventSource,
    EventType,
    EventPayload,
    Timestamp,
    Processed,
}

#[derive(Iden)]
enum PlatformMetricsDaily {
    Table,
    Id,
    Date,
    TenantId,
    MetricSource,
    MetricKey,
    MetricValue,
}
