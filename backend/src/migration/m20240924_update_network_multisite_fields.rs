use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add new fields for multi-site management to the network table
        manager
            .alter_table(
                Table::alter()
                    .table(Network::Table)
                    .add_column(
                        ColumnDef::new(Network::EnabledModules)
                            .unsigned()
                            .not_null()
                            .default(0)
                    )
                    .add_column(
                        ColumnDef::new(Network::Theme)
                            .string()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Network::CustomSettings)
                            .json_binary()
                            .null()
                    )
                    .add_column(
                        ColumnDef::new(Network::SiteStatus)
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
                    .table(Network::Table)
                    .drop_column(Network::EnabledModules)
                    .drop_column(Network::Theme)
                    .drop_column(Network::CustomSettings)
                    .drop_column(Network::SiteStatus)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Network {
    Table,
    EnabledModules,
    Theme,
    CustomSettings,
    SiteStatus,
} 