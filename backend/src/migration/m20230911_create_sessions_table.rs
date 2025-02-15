use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Session::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Session::UserId).uuid().not_null())
                    .col(ColumnDef::new(Session::BearerToken).string().not_null())
                    .col(ColumnDef::new(Session::RefreshToken).string().not_null())
                    .col(ColumnDef::new(Session::TokenExpiration).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Session::RefreshTokenExpiration).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Session::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Session::LastAccessedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Session::IsAdmin).boolean().not_null())
                    .col(ColumnDef::new(Session::IsActive).boolean().not_null())
                    .col(ColumnDef::new(Session::IntegrityHash).string().not_null())
                    .col(ColumnDef::new(Session::LastModifiedDate).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_session_user_id")
                    .table(Session::Table)
                    .col(Session::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_session_bearer_token")
                    .table(Session::Table)
                    .col(Session::BearerToken)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Session::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Session {
    Table,
    Id,
    UserId,
    BearerToken,
    RefreshToken,
    TokenExpiration,
    RefreshTokenExpiration,
    CreatedAt,
    LastAccessedAt,
    IsAdmin,
    IsActive,
    IntegrityHash,
    LastModifiedDate,
}