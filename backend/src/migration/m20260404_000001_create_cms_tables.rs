use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create app_pages table
        manager
            .create_table(
                Table::create()
                    .table(AppPages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppPages::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AppPages::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AppPages::Slug).string().not_null())
                    .col(ColumnDef::new(AppPages::Title).string().not_null())
                    .col(ColumnDef::new(AppPages::Description).text().not_null())
                    .col(ColumnDef::new(AppPages::PageType).string().not_null().default("standard".to_string()))
                    .col(ColumnDef::new(AppPages::HeroPayload).json_binary())
                    .col(ColumnDef::new(AppPages::BlocksPayload).json_binary())
                    .col(ColumnDef::new(AppPages::IsPublished).boolean().not_null().default(true))
                    .col(
                        ColumnDef::new(AppPages::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AppPages::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for app_pages.tenant_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-app_pages-tenant_id")
                    .from(AppPages::Table, AppPages::TenantId)
                    .to(Tenant::Table, Tenant::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Create app_menus table
        manager
            .create_table(
                Table::create()
                    .table(AppMenus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppMenus::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AppMenus::TenantId).uuid().not_null())
                    .col(ColumnDef::new(AppMenus::MenuType).string().not_null().default("header".to_string()))
                    .col(ColumnDef::new(AppMenus::Label).string().not_null())
                    .col(ColumnDef::new(AppMenus::Href).string())
                    .col(ColumnDef::new(AppMenus::ParentId).uuid())
                    .col(ColumnDef::new(AppMenus::DisplayOrder).integer().not_null().default(0))
                    .col(ColumnDef::new(AppMenus::IsVisible).boolean().not_null().default(true))
                    .col(
                        ColumnDef::new(AppMenus::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(AppMenus::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for app_menus.tenant_id
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-app_menus-tenant_id")
                    .from(AppMenus::Table, AppMenus::TenantId)
                    .to(Tenant::Table, Tenant::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        // Add foreign key constraint for app_menus.parent_id (self-referencing)
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk-app_menus-parent_id")
                    .from(AppMenus::Table, AppMenus::ParentId)
                    .to(AppMenus::Table, AppMenus::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppMenus::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(AppPages::Table).to_owned())
            .await?;
            
        Ok(())
    }
}

#[derive(Iden)]
pub enum AppPages {
    Table,
    Id,
    TenantId,
    Slug,
    Title,
    Description,
    PageType,
    HeroPayload,
    BlocksPayload,
    IsPublished,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum AppMenus {
    Table,
    Id,
    TenantId,
    MenuType,
    Label,
    Href,
    ParentId,
    DisplayOrder,
    IsVisible,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum Tenant {
    Table,
    Id,
}
