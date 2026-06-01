use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// G-27 gap fill: Context-Aware Display Rules engine.
///
/// Adds `atlas_scorecard_display_rules` — the "second axis" of G-27 configurability:
///   Axis 1 (existing): *what to measure*  → template + dimensions
///   Axis 2 (this):     *when to surface*  → display rules
///
/// A display rule is a condition → action pair that controls dimension visibility
/// in the session form. The frontend evaluates rules client-side against the
/// current entity field values (same pattern as Salesforce AppExchange G27SC).
///
/// # Rule evaluation contract
/// 1. Load all active rules for the template via `ScorecardService::get_display_rules`.
/// 2. For each dimension, collect all matching rules ordered by `priority ASC`.
/// 3. Apply conflict resolution: `Require` > `Hide` > `Show` (see `RuleAction::overrides`).
/// 4. Default (no matching rule): dimension is visible, not required.
///
/// # Tier gate
/// Display rules are a Professional+ feature. Starter tenants receive an empty
/// rule set — all dimensions render unconditionally. Gate is enforced in the
/// service layer via `tenant_setting` key `scorecard_display_rules_enabled`.
///
/// Spec: docs/architecture/g27/g27_appexchange_spec.md §11
///       docs/architecture/g27/g27_scorecards_spec.md §15 (added by gap fill)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // ── atlas_scorecard_display_rules ─────────────────────────────────────
        db.execute(sea_orm::Statement::from_string(
            backend,
            r#"
            CREATE TABLE IF NOT EXISTS atlas_scorecard_display_rules (
                id                  UUID        NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
                template_id         UUID        NOT NULL
                                        REFERENCES atlas_scorecard_templates(id) ON DELETE CASCADE,
                -- null = category-level rule (applies to all active dims in category_target)
                dimension_id        UUID        REFERENCES atlas_scorecard_dimensions(id) ON DELETE CASCADE,
                tenant_id           UUID        NOT NULL,

                -- Category-level target (used when dimension_id IS NULL)
                category_target     VARCHAR(50),

                -- ── Trigger axis ─────────────────────────────────────────────
                -- Stored as VARCHAR; service converts to/from TriggerCategory enum.
                -- 'record_state' | 'time_proximity' | 'activity_trigger' | 'score_gap'
                trigger_category    VARCHAR(30) NOT NULL,

                -- Which field on the subject entity to evaluate (record_state + time_proximity).
                -- e.g. 'stage', 'close_date', 'lead_status'
                field_reference     VARCHAR(255),

                -- Comparison operator.
                -- Stored as VARCHAR; service converts to/from RuleOperator enum.
                -- 'equals' | 'not_equals' | 'in' | 'not_in' | 'within_days' |
                -- 'overdue_days' | 'dimension_score_below' | 'dimension_score_above' |
                -- 'activity_type_is' | 'dimension_unrated'
                operator            VARCHAR(40) NOT NULL,

                -- Scalar comparison value (for 'equals', 'within_days', score thresholds).
                value               TEXT,

                -- List comparison values (for 'in', 'not_in', 'activity_type_is').
                -- Stored as JSONB array of strings: ["call", "demo"]
                value_list          JSONB,

                -- ── Action axis ───────────────────────────────────────────────
                -- Stored as VARCHAR; service converts to/from RuleAction enum.
                -- 'show' | 'hide' | 'require' | 'surface_as_nudge' |
                -- 'show_in_prep_mode' | 'show_alert_banner'
                action              VARCHAR(30) NOT NULL,

                -- For 'show_alert_banner': the message to render in the widget banner.
                alert_message       TEXT,

                -- ── Scope ─────────────────────────────────────────────────────
                -- Stored as VARCHAR; service converts to/from ModeScope enum.
                -- 'always' | 'post_activity' | 'pre_activity' | 'on_score_gap'
                mode_scope          VARCHAR(20) NOT NULL DEFAULT 'always',

                -- Lower number = higher priority. Conflict resolution uses priority ASC.
                -- Default 10 leaves room for urgent (1–9) and low-priority (11+) rules.
                priority            INT         NOT NULL DEFAULT 10,

                is_active           BOOLEAN     NOT NULL DEFAULT true,
                description         TEXT,
                created_by_user_id  UUID,
                created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            "#
            .to_owned(),
        ))
        .await?;

        // Primary read: all active rules for a template, ordered by priority.
        // Used by the session form on every render.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_display_rules_template_active \
             ON atlas_scorecard_display_rules (template_id, is_active, priority) \
             WHERE is_active = true;"
                .to_owned(),
        ))
        .await?;

        // Dimension-level lookup: rules that target a specific dimension.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_display_rules_dimension \
             ON atlas_scorecard_display_rules (dimension_id) \
             WHERE dimension_id IS NOT NULL;"
                .to_owned(),
        ))
        .await?;

        // Activity trigger lookup: rules that fire on a specific activity type.
        // Used by get_nudge_dimensions_for_activity to find matching rules efficiently.
        db.execute(sea_orm::Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_display_rules_activity_trigger \
             ON atlas_scorecard_display_rules (template_id, trigger_category) \
             WHERE trigger_category = 'activity_trigger' AND is_active = true;"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("atlas_scorecard_display_rules"))
                    .to_owned(),
            )
            .await
    }
}
