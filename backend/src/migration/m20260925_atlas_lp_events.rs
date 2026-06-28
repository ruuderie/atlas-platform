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
                    .table(AtlasLpEvent::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(AtlasLpEvent::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(AtlasLpEvent::AppPageId).uuid().not_null())
                    .col(
                        ColumnDef::new(AtlasLpEvent::EventType)
                            .string()
                            .not_null()
                            .default("view"),
                    )
                    .col(ColumnDef::new(AtlasLpEvent::SessionId).string().not_null())
                    .col(ColumnDef::new(AtlasLpEvent::UtmSource).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::UtmMedium).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::UtmCampaign).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::UtmContent).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::UtmTerm).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::Viewport).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::Referrer).string().null())
                    .col(ColumnDef::new(AtlasLpEvent::CountryCode).string_len(2).null())
                    .col(
                        ColumnDef::new(AtlasLpEvent::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(AtlasLpEvent::Table, AtlasLpEvent::AppPageId)
                            .to(AppPage::Table, AppPage::Id)
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
                    .table(AtlasLpEvent::Table)
                    .col(AtlasLpEvent::AppPageId)
                    .col(AtlasLpEvent::EventType)
                    .col(AtlasLpEvent::CreatedAt)
                    .to_owned(),
            )
            .await?;

        // Index for UTM attribution breakdown
        manager
            .create_index(
                Index::create()
                    .name("idx_lp_events_utm_source")
                    .table(AtlasLpEvent::Table)
                    .col(AtlasLpEvent::UtmSource)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AtlasLpEvent::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AtlasLpEvent {
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
enum AppPage {
    Table,
    Id,
}
