use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add new fields for multi-site management to the directory table
        manager
            .alter_table(
                Table::alter()
                    .table(Directory::Table)
                    .add_column(
                        ColumnDef::new(Directory::EnabledModules)
                            .unsigned()
                            .not_null()
                            .default(0)
                    )
                    .add_column(
                        ColumnDef::new(Directory::Theme)
                            .string()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Directory::CustomSettings)
                            .json_binary()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Directory::SiteStatus)
                            .string()
                            .not_null()
                            .default("active")
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the added columns
        manager
            .alter_table(
                Table::alter()
                    .table(Directory::Table)
                    .drop_column(Directory::EnabledModules)
                    .drop_column(Directory::Theme)
                    .drop_column(Directory::CustomSettings)
                    .drop_column(Directory::SiteStatus)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Directory {
    Table,
    EnabledModules,
    Theme,
    CustomSettings,
    SiteStatus,
} 