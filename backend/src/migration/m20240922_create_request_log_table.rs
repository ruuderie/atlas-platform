use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RequestLog::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RequestLog::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RequestLog::UserId).uuid().null())
                    .col(ColumnDef::new(RequestLog::IpAddress).string().not_null())
                    .col(ColumnDef::new(RequestLog::UserAgent).string())
                    .col(ColumnDef::new(RequestLog::Path).string().not_null())
                    .col(ColumnDef::new(RequestLog::Method).string().not_null())
                    .col(ColumnDef::new(RequestLog::StatusCode).integer().not_null())
                    .col(ColumnDef::new(RequestLog::RequestType).string().not_null())
                    .col(
                        ColumnDef::new(RequestLog::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(RequestLog::RequestStatus)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(RequestLog::FailureReason).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-request_log-user_id")
                            .from(RequestLog::Table, RequestLog::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RequestLog::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum RequestLog {
    Table,
    Id,
    UserId,
    IpAddress,
    UserAgent,
    Path,
    Method,
    StatusCode,
    RequestType,
    CreatedAt,
    RequestStatus,
    FailureReason,
}

#[derive(Iden)]
enum User {
    Table,
    Id,
}
