use sea_orm_migration::prelude::*;

/// G26 forward migration — backfills schema gaps on live databases that had
/// the original m20260701_g26_catalog migration applied (renamed to
/// m20260803_g26_atlas_catalog via seaql_migrations at startup).
///
/// The original G26 migration stored `entry_type` as a plain VARCHAR.
/// The new G26 defines it as an `atlas_catalog_entry_type` Postgres enum,
/// adds an `available_count` GENERATED column, and installs updated_at triggers.
///
/// This migration is SAFE to run on any DB:
///   - IF NOT EXISTS guards prevent double-creation on fresh databases.
///   - EXCEPTION WHEN duplicate_object blocks catch any races.
///   - ALTER TABLE ... ADD COLUMN IF NOT EXISTS avoids adding duplicate columns.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── 1. Create atlas_catalog_entry_type enum (idempotent) ─────────────
        manager
            .get_connection()
            .execute_unprepared(
                "
                DO $$ BEGIN
                    CREATE TYPE atlas_catalog_entry_type AS ENUM (
                        'room_type',
                        'service_slot',
                        'package_tier',
                        'subscription_tier',
                        'coverage_option',
                        'add_on',
                        'equipment_unit'
                    );
                EXCEPTION WHEN duplicate_object THEN NULL;
                END $$;
                ",
            )
            .await?;

        // ── 2. Convert entry_type column from varchar → enum (idempotent) ────
        // Only converts if the column is still of type varchar/text.
        // Casting to the enum type will fail if any existing value is not in
        // the enum set — add USING with a safe fallback to 'room_type'.
        manager
            .get_connection()
            .execute_unprepared(
                "
                DO $$ BEGIN
                    IF EXISTS (
                        SELECT 1 FROM information_schema.columns
                        WHERE table_name = 'atlas_catalog_entries'
                          AND column_name = 'entry_type'
                          AND udt_name IN ('varchar', 'text', 'character varying')
                    ) THEN
                        ALTER TABLE atlas_catalog_entries
                            ALTER COLUMN entry_type TYPE atlas_catalog_entry_type
                            USING CASE
                                WHEN entry_type IN (
                                    'room_type','service_slot','package_tier',
                                    'subscription_tier','coverage_option','add_on','equipment_unit'
                                ) THEN entry_type::atlas_catalog_entry_type
                                ELSE 'room_type'::atlas_catalog_entry_type
                            END;
                    END IF;
                END $$;
                ",
            )
            .await?;

        // ── 3. Add available_count GENERATED column (idempotent) ─────────────
        manager
            .get_connection()
            .execute_unprepared(
                "
                ALTER TABLE atlas_catalog_entries
                    ADD COLUMN IF NOT EXISTS available_count INT
                        GENERATED ALWAYS AS (total_inventory - reserved_count) STORED;
                ",
            )
            .await?;

        // ── 4. Install update_updated_at_column() function (idempotent) ──────
        manager
            .get_connection()
            .execute_unprepared(
                "
                CREATE OR REPLACE FUNCTION update_updated_at_column()
                RETURNS TRIGGER AS $$
                BEGIN
                    NEW.updated_at = NOW();
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                ",
            )
            .await?;

        // ── 5. Attach trigger to atlas_catalog_entries (idempotent) ──────────
        manager
            .get_connection()
            .execute_unprepared(
                "
                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_catalog_entries
                        BEFORE UPDATE ON atlas_catalog_entries
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                EXCEPTION WHEN duplicate_object THEN NULL;
                END $$;
                ",
            )
            .await?;

        // ── 6. Attach trigger to atlas_catalog_rate_rules (idempotent) ───────
        manager
            .get_connection()
            .execute_unprepared(
                "
                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_catalog_rate_rules
                        BEFORE UPDATE ON atlas_catalog_rate_rules
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                EXCEPTION WHEN duplicate_object THEN NULL;
                END $$;
                ",
            )
            .await?;

        // ── 7. New indexes on atlas_catalog_entries (idempotent) ─────────────
        manager
            .get_connection()
            .execute_unprepared(
                "
                CREATE INDEX IF NOT EXISTS idx_catalog_entries_tenant
                    ON atlas_catalog_entries (tenant_id);
                CREATE INDEX IF NOT EXISTS idx_catalog_entries_entry_type
                    ON atlas_catalog_entries (entry_type);
                CREATE INDEX IF NOT EXISTS idx_catalog_entries_available
                    ON atlas_catalog_entries (tenant_id)
                    WHERE available_count > 0 AND NOT is_blocked;
                ",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Reverse: drop triggers, revert column type, drop enum, drop indexes.
        manager
            .get_connection()
            .execute_unprepared(
                "
                DROP TRIGGER IF EXISTS set_updated_at_catalog_entries   ON atlas_catalog_entries;
                DROP TRIGGER IF EXISTS set_updated_at_catalog_rate_rules ON atlas_catalog_rate_rules;
                DROP INDEX IF EXISTS idx_catalog_entries_tenant;
                DROP INDEX IF EXISTS idx_catalog_entries_entry_type;
                DROP INDEX IF EXISTS idx_catalog_entries_available;

                ALTER TABLE atlas_catalog_entries
                    ALTER COLUMN entry_type TYPE VARCHAR USING entry_type::VARCHAR;

                ALTER TABLE atlas_catalog_entries
                    DROP COLUMN IF EXISTS available_count;

                DROP TYPE IF EXISTS atlas_catalog_entry_type CASCADE;
                ",
            )
            .await?;
        Ok(())
    }
}
