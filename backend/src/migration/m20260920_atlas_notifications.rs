/// atlas_notification + atlas_user_notification_pref — Notification inbox & channel prefs
///
/// ## Tables created
///
/// ### atlas_notification
/// Persistent in-app notification record. Written by NotificationService::dispatch().
/// Every notification is stored here regardless of which external channels were triggered.
/// - `notification_type`: lease_expiring | rent_due | maintenance_request | message_received |
///                        violation_filed | inspection_scheduled | payment_received |
///                        lead_assigned | scorecard_nudge | system | announcement
/// - `channels_attempted`: JSONB array of {channel, status, attempted_at} — delivery receipt log
/// - `entity_type` / `entity_id`: the Atlas record this notification is about (optional)
///
/// ### atlas_user_notification_pref
/// Per-user, per-tenant, per-channel opt-in preferences.
/// One row per (user_id, tenant_id, channel). UPSERT on conflict.
/// - `channel`: in_app | sms | email | telegram | whatsapp
/// - `config`: JSONB — channel-specific details:
///     telegram  → { "chat_id": "...", "scope": "personal" | "group" }
///     whatsapp  → { "phone": "+1...", "provider": "twilio" | "meta" }
///     sms       → { "phone": "+1..." }
///     email     → { "email": "..." }  (defaults to user.email if absent)
///     in_app    → {}  (always enabled, config unused)
/// - Tenant-level broadcast channels (e.g. a landlord group chat) are stored with
///   user_id = tenant_id (sentinel value) and scope = "broadcast".
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
-- ─────────────────────────────────────────────────────────────────────────────
-- atlas_notification: persistent in-app notification inbox
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS atlas_notification (
    id                  UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID        NOT NULL REFERENCES account(id) ON DELETE CASCADE,
    user_id             UUID        NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,

    -- Classification
    notification_type   TEXT        NOT NULL,
    title               TEXT        NOT NULL,
    body                TEXT        NOT NULL,
    priority            TEXT        NOT NULL DEFAULT 'normal'
                            CHECK (priority IN ('low', 'normal', 'high', 'urgent')),

    -- Linked entity (optional)
    entity_type         TEXT,
    entity_id           UUID,

    -- Extra structured data (action URL, image, etc.)
    metadata            JSONB,

    -- Delivery tracking: [{channel, status, attempted_at, error}]
    channels_attempted  JSONB       NOT NULL DEFAULT '[]'::jsonb,

    -- Read state
    read_at             TIMESTAMPTZ,

    -- Soft-delete (dismiss without purge)
    dismissed_at        TIMESTAMPTZ,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_atlas_notification_user
    ON atlas_notification (user_id, tenant_id, created_at DESC)
    WHERE dismissed_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_atlas_notification_unread
    ON atlas_notification (user_id, tenant_id)
    WHERE read_at IS NULL AND dismissed_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_atlas_notification_entity
    ON atlas_notification (entity_type, entity_id)
    WHERE entity_id IS NOT NULL;

-- ─────────────────────────────────────────────────────────────────────────────
-- atlas_user_notification_pref: per-user, per-tenant, per-channel opt-in
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS atlas_user_notification_pref (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID        NOT NULL REFERENCES "user"(id) ON DELETE CASCADE,
    tenant_id   UUID        NOT NULL REFERENCES account(id) ON DELETE CASCADE,

    -- Channel identifier
    channel     TEXT        NOT NULL
                    CHECK (channel IN ('in_app', 'sms', 'email', 'telegram', 'whatsapp')),

    -- Channel-specific config (see migration doc above for shape per channel)
    config      JSONB       NOT NULL DEFAULT '{}'::jsonb,

    -- Master on/off switch for this channel
    enabled     BOOLEAN     NOT NULL DEFAULT true,

    -- Notification types this pref applies to (empty = all types)
    -- e.g. ["rent_due", "lease_expiring"] to only get payment nudges via telegram
    applies_to  TEXT[]      NOT NULL DEFAULT ARRAY[]::TEXT[],

    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (user_id, tenant_id, channel)
);

CREATE INDEX IF NOT EXISTS idx_atlas_user_notif_pref_lookup
    ON atlas_user_notification_pref (user_id, tenant_id)
    WHERE enabled = true;

-- Trigger: auto-update updated_at
CREATE OR REPLACE FUNCTION atlas_user_notif_pref_set_updated_at()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_atlas_user_notif_pref_updated_at
    ON atlas_user_notification_pref;

CREATE TRIGGER trg_atlas_user_notif_pref_updated_at
    BEFORE UPDATE ON atlas_user_notification_pref
    FOR EACH ROW EXECUTE FUNCTION atlas_user_notif_pref_set_updated_at();
"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
DROP TRIGGER IF EXISTS trg_atlas_user_notif_pref_updated_at ON atlas_user_notification_pref;
DROP FUNCTION IF EXISTS atlas_user_notif_pref_set_updated_at();
DROP TABLE IF EXISTS atlas_user_notification_pref;
DROP TABLE IF EXISTS atlas_notification;
"#,
            )
            .await?;
        Ok(())
    }
}
