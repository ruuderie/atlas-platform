use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserAppPermission::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserAppPermission::UserId).uuid().not_null())
                    .col(
                        ColumnDef::new(UserAppPermission::TenantId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserAppPermission::AppSlug)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserAppPermission::Permissions)
                            .json_binary()
                            .not_null()
                            .default("[]"),
                    )
                    .primary_key(
                        Index::create()
                            .name("pk_user_app_permission")
                            .col(UserAppPermission::UserId)
                            .col(UserAppPermission::TenantId)
                            .col(UserAppPermission::AppSlug),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_app_permission_user")
                            .from(UserAppPermission::Table, UserAppPermission::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_app_permission_tenant")
                            .from(UserAppPermission::Table, UserAppPermission::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserAppPermission::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum UserAppPermission {
    Table,
    UserId,
    TenantId,
    AppSlug,
    Permissions,
}

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
pub enum Tenant {
    Table,
    Id,
}
