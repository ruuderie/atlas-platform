//! m20260925_atlas_lp_events — Landing Page Funnel Analytics
//!
//! Creates `atlas_lp_events` which records every view and lead-capture event
//! on platform-admin landing pages. One row per event.

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AtlasLpEvents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AtlasLpEvents::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AtlasLpEvents::AppPageId).uuid().not_null())
                    .col(
                        ColumnDef::new(AtlasLpEvents::EventType)
                            .string()
                            .not_null()
                            .default("view"),
                    )
                    .col(ColumnDef::new(AtlasLpEvents::SessionId).string().not_null())
                    .col(ColumnDef::new(AtlasLpEvents::UtmSource).string().null())
                    .col(ColumnDef::new(AtlasLpEvents::UtmMedium).string().null())
                    .col(ColumnDef::new(AtlasLpEvents::UtmCampaign).string().null())
                    .col(ColumnDef::new(AtlasLpEvents::UtmContent).string().null())
                    .col(ColumnDef::new(AtlasLpEvents::UtmTerm).string().null())
                    .col(ColumnDef::new(AtlasLpEvents::Viewport).string().null())
                    .col(ColumnDef::new(AtlasLpEvents::Referrer).string().null())
                    .col(
                        ColumnDef::new(AtlasLpEvents::CountryCode)
                            .string_len(2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AtlasLpEvents::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AtlasLpEvents::Table, AtlasLpEvents::AppPageId)
                            .to(AppPages::Table, AppPages::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for per-page aggregation queries
        manager
            .create_index(
                Index::create()
                    .name("idx_lp_events_page_type_ts")
                    .table(AtlasLpEvents::Table)
                    .col(AtlasLpEvents::AppPageId)
                    .col(AtlasLpEvents::EventType)
                    .col(AtlasLpEvents::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Index for UTM attribution breakdown
        manager
            .create_index(
                Index::create()
                    .name("idx_lp_events_utm_source")
                    .table(AtlasLpEvents::Table)
                    .col(AtlasLpEvents::UtmSource)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasLpEvents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasLpEvents {
    Table,
    Id,
    AppPageId,
    EventType,
    SessionId,
    UtmSource,
    UtmMedium,
    UtmCampaign,
    UtmContent,
    UtmTerm,
    Viewport,
    Referrer,
    CountryCode,
    CreatedAt,
}

#[derive(DeriveIden)]
enum AppPages {
    Table,
    Id,
}
