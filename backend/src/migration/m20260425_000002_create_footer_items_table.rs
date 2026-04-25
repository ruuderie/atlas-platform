use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(FooterItems::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FooterItems::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FooterItems::Label).string().not_null())
                    .col(ColumnDef::new(FooterItems::Href).string().null())
                    .col(ColumnDef::new(FooterItems::DisplayOrder).integer().not_null().default(0))
                    .col(ColumnDef::new(FooterItems::IsVisible).boolean().not_null().default(true))
                    .to_owned(),
            )
            .await?;

        // Seed some default footer items for UAT
        manager.get_connection().execute_unprepared(
            "INSERT INTO footer_items (label, href, display_order, is_visible) VALUES 
            ('Terms', '/terms', 10, true),
            ('Privacy', '/privacy', 20, true)
            ON CONFLICT DO NOTHING;"
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FooterItems::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum FooterItems {
    Table,
    Id,
    Label,
    Href,
    DisplayOrder,
    IsVisible,
}
