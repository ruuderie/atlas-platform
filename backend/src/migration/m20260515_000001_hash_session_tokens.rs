use sea_orm_migration::prelude::*;

/// Adds SHA-256 hash columns for bearer and refresh tokens to the session table.
///
/// # Security Rationale
///
/// Previously, `bearer_token` and `refresh_token` were stored as raw JWTs in
/// the `session` table. A database read compromise (SQL injection, direct DB
/// access) would yield immediately usable credentials valid for up to 7 days.
///
/// This migration adds `bearer_token_hash` and `refresh_token_hash` columns
/// (SHA-256 hex, 64 chars) as a **non-breaking additive change**:
/// - The existing plaintext columns are kept temporarily for backward compatibility.
/// - New sessions write both the hash (for lookup) and the plaintext (for old code).
/// - Lookups are migrated to use the hash columns.
/// - The plaintext columns will be dropped in a follow-up migration once all
///   deployed pods are running the new code.
///
/// # Implementation Note
///
/// `SHA-256(JWT)` is a one-way transformation. Even with the hash, an attacker
/// cannot reconstruct the JWT (JWTs are not short enough for rainbow table attacks).
/// The hash is used only for DB lookup — the actual token is sent to the client
/// and compared in the application layer.
pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260515_000001_hash_session_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add bearer_token_hash column
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Session::BearerTokenHash)
                            .string_len(64)
                            .null()
                            .comment("SHA-256(bearer_token) hex digest — used for DB lookup"),
                    )
                    .to_owned(),
            )
            .await?;

        // Add refresh_token_hash column
        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Session::RefreshTokenHash)
                            .string_len(64)
                            .null()
                            .comment("SHA-256(refresh_token) hex digest — used for DB lookup"),
                    )
                    .to_owned(),
            )
            .await?;

        // Backfill hashes for all existing sessions using PostgreSQL's SHA-256.
        // encode(sha256(bearer_token::bytea), 'hex') is stable and doesn't require
        // any Rust-side iteration — runs entirely in the DB for performance.
        manager
            .get_connection()
            .execute_unprepared(
                "UPDATE session
                 SET bearer_token_hash  = encode(sha256(bearer_token::bytea),  'hex'),
                     refresh_token_hash = encode(sha256(refresh_token::bytea), 'hex')
                 WHERE bearer_token_hash IS NULL",
            )
            .await?;

        // Add unique index on bearer_token_hash for O(1) session lookup.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_session_bearer_token_hash")
                    .table(Session::Table)
                    .col(Session::BearerTokenHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Add index on refresh_token_hash for refresh flow.
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_session_refresh_token_hash")
                    .table(Session::Table)
                    .col(Session::RefreshTokenHash)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_session_bearer_token_hash")
                    .table(Session::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_session_refresh_token_hash")
                    .table(Session::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::BearerTokenHash)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Session::Table)
                    .drop_column(Session::RefreshTokenHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Session {
    Table,
    BearerTokenHash,
    RefreshTokenHash,
}
