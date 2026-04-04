use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TenantSetting::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TenantSetting::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TenantSetting::TenantId).uuid().not_null())
                    .col(ColumnDef::new(TenantSetting::Key).string().not_null())
                    .col(ColumnDef::new(TenantSetting::Value).text().not_null())
                    .col(ColumnDef::new(TenantSetting::IsEncrypted).boolean().not_null().default(false))
                    .col(ColumnDef::new(TenantSetting::UpdatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(TenantSetting::CreatedAt).timestamp_with_time_zone().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tenant_setting_tenant_id")
                            .from(TenantSetting::Table, TenantSetting::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_tenant_setting_key")
                    .table(TenantSetting::Table)
                    .col(TenantSetting::TenantId)
                    .col(TenantSetting::Key)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantSetting::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TenantSetting {
    Table,
    Id,
    TenantId,
    Key,
    Value,
    IsEncrypted,
    UpdatedAt,
    CreatedAt,
}

#[derive(Iden)]
enum Tenant {
    Table,
    Id,
}
