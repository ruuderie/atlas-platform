use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(WebauthnChallenge::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(WebauthnChallenge::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(WebauthnChallenge::Challenge).json_binary().not_null())
                    .col(ColumnDef::new(WebauthnChallenge::ChallengeType).string().not_null())
                    .col(ColumnDef::new(WebauthnChallenge::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(WebauthnChallenge::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_webauthn_challenge_expires_at")
                    .table(WebauthnChallenge::Table)
                    .col(WebauthnChallenge::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WebauthnChallenge::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum WebauthnChallenge {
    Table,
    Id,
    Challenge,
    ChallengeType,
    ExpiresAt,
    CreatedAt,
}
