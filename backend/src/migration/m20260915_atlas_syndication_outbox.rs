use sea_orm_migration::prelude::*;

/// G-05 Syndication Event Bus — outbox + integration event log.
///
/// # Two tables
///
/// ## `atlas_syndication_outbox`
/// Transactional outbox pattern for all outbound syndication events.
/// Written inside the same DB transaction that changes a listing/asset/link.
/// A background worker polls this table, calls the NI webhook URL,
/// and marks the row `delivered` or increments `retry_count`.
///
/// Dead-letter: after `retry_count >= 5` the row is moved to `failed`
/// so platform ops can inspect and replay.
///
/// ## `atlas_integration_events`
/// Immutable append-only ledger of all dispatched events and their outcomes.
/// Written by the worker after each delivery attempt. Used for:
///   - Audit trails (G-05 compliance)
///   - Debug / replay tooling in platform-admin
///   - Webhook delivery reports shown to operators
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS atlas_integration_events;

                -- ── atlas_syndication_outbox ────────────────────────────────────────
                CREATE TABLE IF NOT EXISTS atlas_syndication_outbox (
                    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

                    -- Which syndication link this event targets
                    link_id             UUID        NOT NULL
                                        REFERENCES atlas_app_instance_syndication(id)
                                        ON DELETE CASCADE,

                    -- Source app instance that fired the event
                    source_config_id    UUID        NOT NULL
                                        REFERENCES atlas_app_deployment_config(id)
                                        ON DELETE CASCADE,

                    -- Event semantics
                    -- 'listing.published'  | 'listing.updated'   | 'listing.unpublished'
                    -- 'asset.created'      | 'asset.updated'
                    -- 'inquiry.received'   | 'application.received'
                    event_type          TEXT        NOT NULL,

                    -- JSON payload to POST to the NI webhook
                    -- Shape: { event_type, entity_id, entity_type, tenant_id, data: {...} }
                    payload             JSONB       NOT NULL DEFAULT '{}',

                    -- Delivery lifecycle
                    -- 'pending' | 'processing' | 'delivered' | 'failed'
                    status              TEXT        NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'processing', 'delivered', 'failed')),

                    -- How many dispatch attempts have been made
                    retry_count         INTEGER     NOT NULL DEFAULT 0,

                    -- Last HTTP status code received from NI (NULL if never attempted)
                    last_http_status    INTEGER,

                    -- Last error message (truncated to 1 KB)
                    last_error          TEXT,

                    -- Scheduled next attempt (allows exponential back-off)
                    next_attempt_at     TIMESTAMPTZ NOT NULL DEFAULT now(),

                    -- Timestamps
                    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
                    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_syndication_outbox_status_next
                    ON atlas_syndication_outbox (status, next_attempt_at)
                    WHERE status IN ('pending', 'processing');

                CREATE INDEX IF NOT EXISTS idx_syndication_outbox_link
                    ON atlas_syndication_outbox (link_id);

                CREATE INDEX IF NOT EXISTS idx_syndication_outbox_source
                    ON atlas_syndication_outbox (source_config_id);

                DO $$ BEGIN
                    CREATE TRIGGER set_updated_at_syndication_outbox
                        BEFORE UPDATE ON atlas_syndication_outbox
                        FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
                EXCEPTION WHEN duplicate_object THEN NULL;
                END $$;

                -- ── atlas_integration_events ───────────────────────────────────────────
                CREATE TABLE IF NOT EXISTS atlas_integration_events (
                    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),

                    -- Which outbox row this delivery attempt corresponds to
                    outbox_id           UUID
                                        REFERENCES atlas_syndication_outbox(id)
                                        ON DELETE SET NULL,

                    -- Denormalised for query performance (events survive outbox cleanup)
                    link_id             UUID        NOT NULL
                                        REFERENCES atlas_app_instance_syndication(id)
                                        ON DELETE CASCADE,

                    source_config_id    UUID        NOT NULL
                                        REFERENCES atlas_app_deployment_config(id)
                                        ON DELETE CASCADE,

                    -- Event type mirrors outbox.event_type
                    event_type          TEXT        NOT NULL,

                    -- Direction from source app's perspective
                    -- 'outbound' = source → NI   |   'inbound' = NI → source
                    direction           TEXT        NOT NULL DEFAULT 'outbound'
                        CHECK (direction IN ('outbound', 'inbound')),

                    -- Delivery outcome for this attempt
                    -- 'success' | 'failed' | 'skipped'
                    outcome             TEXT        NOT NULL
                        CHECK (outcome IN ('success', 'failed', 'skipped')),

                    -- HTTP response status (NULL for inbound events)
                    http_status         INTEGER,

                    -- Abbreviated response body / error message (max 2 KB)
                    response_body       TEXT,

                    -- Latency in milliseconds
                    latency_ms          INTEGER,

                    -- Attempt number (1-based) — mirrors outbox.retry_count + 1 at time of attempt
                    attempt_number      INTEGER     NOT NULL DEFAULT 1,

                    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
                );

                CREATE INDEX IF NOT EXISTS idx_integration_events_link
                    ON atlas_integration_events (link_id, created_at DESC);

                CREATE INDEX IF NOT EXISTS idx_integration_events_source
                    ON atlas_integration_events (source_config_id, created_at DESC);

                CREATE INDEX IF NOT EXISTS idx_integration_events_outbox
                    ON atlas_integration_events (outbox_id)
                    WHERE outbox_id IS NOT NULL;",
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "DROP TRIGGER IF EXISTS set_updated_at_syndication_outbox
                     ON atlas_syndication_outbox;
                 DROP TABLE IF EXISTS atlas_integration_events;
                 DROP TABLE IF EXISTS atlas_syndication_outbox;",
            )
            .await?;

        Ok(())
    }
}
