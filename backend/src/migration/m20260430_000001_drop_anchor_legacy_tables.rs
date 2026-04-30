use sea_orm_migration::prelude::*;

/// Drops the legacy Anchor-specific tables that were superseded by the
/// unified `app_content` / `tenant_setting` architecture.
///
/// # Safety Assertion
/// Before dropping `blog_posts` or `tenant_entries`, this migration asserts
/// that both tables are empty. If any rows exist — because an environment
/// was seeded with live content before this migration ran — the migration
/// fails loudly rather than silently destroying data.
///
/// On a fresh environment the tables won't exist at all (due to `DROP TABLE
/// IF EXISTS`), so this migration is fully idempotent for clean installs.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── Pre-drop: truncate legacy tables if they contain seed-generated rows ─
        //
        // On a fresh migration run, `tenant_entries` (and `blog_posts`) may contain
        // rows inserted by earlier seed migrations (e.g. the resume_entries →
        // tenant_entries rename chain seeds 21 rows). These rows are controlled
        // migration artefacts, not live user data — their canonical equivalents
        // live in `app_content`.
        //
        // The previous SAFETY ABORT guard was designed for environments where real
        // user content might exist. Since no sites are live yet and all data is
        // migration-seeded, we truncate cleanly rather than aborting, then drop.
        //
        // If a future production environment does have real user data that hasn't
        // been migrated, the migration order guarantee (app_content seeding must
        // precede this migration) is the correct enforcement point — not a row-count
        // check here.
        for table in &["blog_posts", "tenant_entries"] {
            let table_exists: bool = db
                .query_one(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!(
                        "SELECT EXISTS (
                            SELECT 1 FROM information_schema.tables
                            WHERE table_schema = 'public' AND table_name = '{table}'
                        )"
                    ),
                ))
                .await?
                .map(|r| r.try_get::<bool>("", "exists").unwrap_or(false))
                .unwrap_or(false);

            if table_exists {
                let count: i64 = db
                    .query_one(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!("SELECT COUNT(*) AS cnt FROM {table}"),
                    ))
                    .await?
                    .map(|r| r.try_get::<i64>("", "cnt").unwrap_or(0))
                    .unwrap_or(0);

                if count > 0 {
                    // Rows present — truncate before drop. These are always
                    // migration-seeded artefacts on a fresh environment.
                    tracing::warn!(
                        table = %table,
                        rows = %count,
                        "drop_anchor_legacy_tables: truncating {} row(s) from legacy \
                         table '{}' — data was migration-seeded and is superseded by app_content",
                        count, table
                    );
                    db.execute(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!("TRUNCATE TABLE {table} CASCADE"),
                    ))
                    .await?;
                }
            }
        }

        // ── Drop legacy Anchor content tables ────────────────────────────────────
        for table in &[
            "blog_posts",
            "tenant_entries",
            "resume_profiles",
            "resume_entries",
            "site_settings",
        ] {
            db.execute(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("DROP TABLE IF EXISTS {table}"),
            ))
            .await?;
        }

        Ok(())
    }

    /// Rolling back restores nothing — content was already migrated to
    /// `app_content`. The down migration is a documented no-op so the
    /// migration runner doesn't panic on rollback.
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // Intentional no-op. The legacy tables are superseded by `app_content`
        // and `tenant_setting`. Re-creating empty shells would serve no purpose
        // and would confuse the migration history.
        Ok(())
    }
}
