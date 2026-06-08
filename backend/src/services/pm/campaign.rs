//! # G19 CampaignService — Multi-Channel Campaign Management
//!
//! ## Scope
//!
//! Manages the four `atlas_campaign_*` tables that cover the **Awareness** and
//! **Acquisition** legs of the platform commerce chain:
//!
//! ```text
//! (Awareness)          (Acquisition)          (Conversion)
//! atlas_campaigns  →   atlas_campaign_enrollments  →  any platform entity
//!      ↓                        ↓
//! atlas_sequence_steps   atlas_campaign_events
//! ```
//!
//! ## Enum-driven type safety
//!
//! Every string field that has a bounded domain is represented as an enum in
//! `crate::types::pm`. The DB columns remain `VARCHAR` for forward-compat with
//! app-specific subtypes; the type boundary is enforced at the **payload layer**:
//!
//! | Field | Enum |
//! |-------|------|
//! | `campaign_type` | `CampaignType` |
//! | `status` | `CampaignStatus` |
//! | `goal_type` | `CampaignGoalType` |
//! | `enrollment.status` | `EnrollmentStatus` |
//! | `event.event_type` | `CampaignEventType` |
//! | `event.channel` | `CampaignChannel` |
//! | `sequence_step.step_type` | `SequenceStepType` |
//!
//! ## Counter maintenance
//!
//! `record_event()` uses a **match on `CampaignEventType`** to determine
//! exactly which counter column to increment — making it impossible to
//! silently increment the wrong counter by passing a misspelled string.
//!
//! ## External integration pattern
//!
//! External senders (Instantly.ai, Google Ads webhooks) write events via
//! `record_event()` after the webhook is parsed by the integration layer (G05).
//! This service never calls out to external APIs directly.

use anyhow::{anyhow, Result};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use uuid::Uuid;

use crate::{
    entities::{
        atlas_campaign, atlas_campaign_enrollment, atlas_campaign_event, atlas_sequence_step,
    },
    types::pm::{
        CampaignChannel, CampaignEventType, CampaignGoalType, CampaignStatus, CampaignType,
        EnrollmentStatus, SequenceStepType,
    },
};

// ── Input payload types ───────────────────────────────────────────────────────

/// Payload for creating a new campaign.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateCampaignPayload {
    /// NULL creates a root campaign. Set to an existing campaign ID to create a
    /// child (tactic) under a program or parent campaign.
    pub parent_campaign_id: Option<Uuid>,
    pub name: String,
    pub campaign_type: CampaignType,
    pub goal_type: Option<CampaignGoalType>,
    /// Entity type that a conversion creates (e.g. "atlas_applications").
    pub goal_entity_type: Option<String>,
    pub target_conversion_count: Option<i32>,
    pub budget_cents: Option<i64>,
    pub currency: Option<String>,
    pub attribution_window_days: Option<i32>,
    pub integration_id: Option<Uuid>,
    pub external_campaign_id: Option<String>,
    /// What this campaign is FOR — polymorphic subject.
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    pub starts_at: Option<chrono::DateTime<Utc>>,
    pub ends_at: Option<chrono::DateTime<Utc>>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub created_by_user_id: Option<Uuid>,
}

/// Payload for adding a sequence step to a campaign.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateSequenceStepPayload {
    pub campaign_id: Uuid,
    pub step_number: i32,
    pub step_type: SequenceStepType,
    pub subject_template: Option<String>,
    pub body_template: Option<String>,
    pub wait_days: Option<i32>,
    pub wait_hours: Option<i32>,
    pub send_time_preference: Option<String>,
    pub condition_type: Option<String>,
    pub condition_value: Option<serde_json::Value>,
    pub on_true_step: Option<i32>,
    pub on_false_step: Option<i32>,
    pub ab_variants: Option<serde_json::Value>,
    pub exit_on_reply: Option<bool>,
    pub exit_on_conversion: Option<bool>,
}

/// Payload for enrolling a contact in a campaign.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct EnrollContactPayload {
    pub campaign_id: Uuid,
    /// Set when enrolling an existing platform user.
    pub contact_user_id: Option<Uuid>,
    /// Set for external contacts not yet in the platform.
    pub contact_email: Option<String>,
    pub contact_name: Option<String>,
    pub contact_metadata: Option<serde_json::Value>,
    pub external_enrollment_id: Option<String>,
    /// When to execute the first step. Defaults to now.
    pub next_step_at: Option<chrono::DateTime<Utc>>,
}

/// Payload for recording a campaign event (from a webhook or internal action).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RecordEventPayload {
    pub enrollment_id: Uuid,
    pub event_type: CampaignEventType,
    pub channel: CampaignChannel,
    pub sequence_step_id: Option<Uuid>,
    pub link_clicked: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub metadata: Option<serde_json::Value>,
    /// Required when event_type is `Converted`.
    pub conversion_entity_type: Option<String>,
    pub conversion_entity_id: Option<Uuid>,
}

/// Filter for listing campaigns.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CampaignFilter {
    pub campaign_type: Option<CampaignType>,
    pub status: Option<CampaignStatus>,
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    /// Filter to only root campaigns (parent_campaign_id IS NULL).
    pub roots_only: Option<bool>,
    /// Filter to direct children of a specific campaign.
    pub parent_campaign_id: Option<Uuid>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct CampaignService;

impl CampaignService {
    // ── Campaign CRUD ─────────────────────────────────────────────────────────

    /// Create a new campaign in `Draft` status.
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateCampaignPayload,
    ) -> Result<atlas_campaign::Model> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let active = atlas_campaign::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            parent_campaign_id: Set(payload.parent_campaign_id),
            name: Set(payload.name),
            campaign_type: Set(payload.campaign_type.to_string()),
            status: Set(CampaignStatus::Draft.to_string()),
            audience_segment_id: Set(None),
            audience_filter: Set(None),
            goal_type: Set(payload.goal_type.map(|g| g.to_string())),
            goal_entity_type: Set(payload.goal_entity_type),
            target_conversion_count: Set(payload.target_conversion_count),
            budget_cents: Set(payload.budget_cents),
            currency: Set(Some(payload.currency.unwrap_or_else(|| "USD".to_string()))),
            spent_cents: Set(0),
            attribution_window_days: Set(payload.attribution_window_days.unwrap_or(30)),
            external_campaign_id: Set(payload.external_campaign_id),
            integration_id: Set(payload.integration_id),
            subject_entity_type: Set(payload.subject_entity_type),
            subject_entity_id: Set(payload.subject_entity_id),
            starts_at: Set(payload.starts_at),
            ends_at: Set(payload.ends_at),
            utm_source: Set(payload.utm_source),
            utm_medium: Set(payload.utm_medium),
            utm_campaign: Set(payload.utm_campaign),
            total_contacts: Set(0),
            total_opens: Set(0),
            total_clicks: Set(0),
            total_replies: Set(0),
            total_conversions: Set(0),
            created_by_user_id: Set(payload.created_by_user_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = active.insert(db).await?;

        tracing::info!(
            %tenant_id, campaign_id = %model.id, campaign_type = %model.campaign_type,
            "CampaignService::create: created '{}'", model.name
        );

        Ok(model)
    }

    /// Get a campaign by ID, verifying tenant ownership.
    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        campaign_id: Uuid,
    ) -> Result<atlas_campaign::Model> {
        atlas_campaign::Entity::find_by_id(campaign_id)
            .filter(atlas_campaign::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Campaign {campaign_id} not found for tenant {tenant_id}"))
    }

    /// List campaigns with optional filtering. Ordered newest-first.
    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filter: CampaignFilter,
    ) -> Result<Vec<atlas_campaign::Model>> {
        let mut q = atlas_campaign::Entity::find()
            .filter(atlas_campaign::Column::TenantId.eq(tenant_id));

        if let Some(ct) = filter.campaign_type {
            q = q.filter(atlas_campaign::Column::CampaignType.eq(ct.to_string()));
        }
        if let Some(st) = filter.status {
            q = q.filter(atlas_campaign::Column::Status.eq(st.to_string()));
        }
        if let Some(set) = filter.subject_entity_type {
            q = q.filter(atlas_campaign::Column::SubjectEntityType.eq(set));
        }
        if let Some(sei) = filter.subject_entity_id {
            q = q.filter(atlas_campaign::Column::SubjectEntityId.eq(sei));
        }
        if filter.roots_only == Some(true) {
            q = q.filter(atlas_campaign::Column::ParentCampaignId.is_null());
        }
        if let Some(pid) = filter.parent_campaign_id {
            q = q.filter(atlas_campaign::Column::ParentCampaignId.eq(pid));
        }

        Ok(q.order_by_desc(atlas_campaign::Column::CreatedAt).all(db).await?)
    }

    /// Transition campaign status. Enforces valid state machine transitions.
    ///
    /// Allowed transitions:
    /// ```text
    /// Draft      → Scheduled | Active
    /// Scheduled  → Active | Archived
    /// Active     → Paused | Completed | Archived
    /// Paused     → Active | Archived
    /// Completed  → Archived
    /// Archived   → (terminal — no transitions)
    /// ```
    pub async fn transition_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        campaign_id: Uuid,
        new_status: CampaignStatus,
    ) -> Result<atlas_campaign::Model> {
        let campaign = Self::get(db, tenant_id, campaign_id).await?;
        let current = CampaignStatus::try_from(campaign.status.as_str())
            .map_err(|e| anyhow!("Invalid stored status: {e}"))?;

        // Validate state machine.
        let allowed = match &current {
            CampaignStatus::Draft     => matches!(new_status, CampaignStatus::Scheduled | CampaignStatus::Active),
            CampaignStatus::Scheduled => matches!(new_status, CampaignStatus::Active | CampaignStatus::Archived),
            CampaignStatus::Active    => matches!(new_status, CampaignStatus::Paused | CampaignStatus::Completed | CampaignStatus::Archived),
            CampaignStatus::Paused    => matches!(new_status, CampaignStatus::Active | CampaignStatus::Archived),
            CampaignStatus::Completed => matches!(new_status, CampaignStatus::Archived),
            CampaignStatus::Archived  => false,
        };

        if !allowed {
            return Err(anyhow!(
                "Invalid transition: {current} → {new_status} for campaign {campaign_id}"
            ));
        }

        let mut active: atlas_campaign::ActiveModel = campaign.into();
        active.status = Set(new_status.to_string());
        active.updated_at = Set(Utc::now());
        Ok(active.update(db).await?)
    }

    // ── Sequence steps ────────────────────────────────────────────────────────

    /// Add a sequence step to a campaign. Steps are ordered by `step_number`.
    pub async fn add_sequence_step(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateSequenceStepPayload,
    ) -> Result<atlas_sequence_step::Model> {
        // Verify campaign ownership.
        Self::get(db, tenant_id, payload.campaign_id).await?;

        let active = atlas_sequence_step::ActiveModel {
            id: Set(Uuid::new_v4()),
            campaign_id: Set(payload.campaign_id),
            step_number: Set(payload.step_number),
            step_type: Set(payload.step_type.to_string()),
            subject_template: Set(payload.subject_template),
            body_template: Set(payload.body_template),
            wait_days: Set(payload.wait_days),
            wait_hours: Set(payload.wait_hours),
            send_time_preference: Set(payload.send_time_preference),
            condition_type: Set(payload.condition_type),
            condition_value: Set(payload.condition_value),
            on_true_step: Set(payload.on_true_step),
            on_false_step: Set(payload.on_false_step),
            ab_variants: Set(payload.ab_variants),
            exit_on_reply: Set(payload.exit_on_reply.unwrap_or(true)),
            exit_on_conversion: Set(payload.exit_on_conversion.unwrap_or(true)),
            created_at: Set(Utc::now()),
        };

        Ok(active.insert(db).await?)
    }

    /// List all sequence steps for a campaign, ordered by step_number.
    pub async fn list_steps(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        campaign_id: Uuid,
    ) -> Result<Vec<atlas_sequence_step::Model>> {
        Self::get(db, tenant_id, campaign_id).await?;

        Ok(atlas_sequence_step::Entity::find()
            .filter(atlas_sequence_step::Column::CampaignId.eq(campaign_id))
            .order_by_asc(atlas_sequence_step::Column::StepNumber)
            .all(db)
            .await?)
    }

    // ── Enrollment ────────────────────────────────────────────────────────────

    /// Enroll a contact in a campaign. Deduplication is caller's responsibility
    /// (check `find_enrollment_by_email` first if needed).
    pub async fn enroll(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: EnrollContactPayload,
    ) -> Result<atlas_campaign_enrollment::Model> {
        // Verify campaign belongs to this tenant and is enrollable.
        let campaign = Self::get(db, tenant_id, payload.campaign_id).await?;
        let status = CampaignStatus::try_from(campaign.status.as_str())
            .map_err(|e| anyhow!("Invalid stored campaign status: {e}"))?;

        if !matches!(status, CampaignStatus::Active | CampaignStatus::Scheduled) {
            return Err(anyhow!(
                "Cannot enroll in campaign {}: status is {status} (must be active or scheduled)",
                payload.campaign_id
            ));
        }

        let now = Utc::now();
        let id = Uuid::new_v4();

        let active = atlas_campaign_enrollment::ActiveModel {
            id: Set(id),
            campaign_id: Set(payload.campaign_id),
            tenant_id: Set(tenant_id),
            contact_user_id: Set(payload.contact_user_id),
            contact_email: Set(payload.contact_email),
            contact_name: Set(payload.contact_name),
            contact_metadata: Set(payload.contact_metadata),
            status: Set(EnrollmentStatus::Active.to_string()),
            current_step: Set(1),
            exit_reason: Set(None),
            exit_at: Set(None),
            converted_at: Set(None),
            conversion_entity_type: Set(None),
            conversion_entity_id: Set(None),
            external_enrollment_id: Set(payload.external_enrollment_id),
            enrolled_at: Set(now),
            next_step_at: Set(Some(payload.next_step_at.unwrap_or(now))),
        };

        let enrollment = active.insert(db).await?;

        // Increment total_contacts counter on the campaign.
        Self::increment_counter(db, payload.campaign_id, CampaignCounter::Contacts).await?;

        tracing::info!(
            %tenant_id, campaign_id = %payload.campaign_id, enrollment_id = %id,
            "CampaignService::enroll: contact enrolled"
        );

        Ok(enrollment)
    }

    /// Look up an enrollment by contact email within a campaign (for dedup).
    pub async fn find_enrollment_by_email(
        db: &DatabaseConnection,
        campaign_id: Uuid,
        email: &str,
    ) -> Result<Option<atlas_campaign_enrollment::Model>> {
        Ok(atlas_campaign_enrollment::Entity::find()
            .filter(atlas_campaign_enrollment::Column::CampaignId.eq(campaign_id))
            .filter(atlas_campaign_enrollment::Column::ContactEmail.eq(email))
            .one(db)
            .await?)
    }

    // ── Event recording ───────────────────────────────────────────────────────

    /// Record a campaign event and update all derived state via enum-driven
    /// match dispatch. This is the **single write path** for all campaign
    /// interaction data — webhooks from Instantly, Google Ads, etc. all route
    /// through here after being parsed by the G05 integration layer.
    ///
    /// Enum match guarantees:
    ///   - Exactly the right counter is incremented — no string-based dispatch
    ///   - `Converted` always exits the enrollment and writes conversion FK
    ///   - `Bounced` / `Unsubscribed` always exits the enrollment
    ///   - All other events only touch counters
    pub async fn record_event(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: RecordEventPayload,
    ) -> Result<atlas_campaign_event::Model> {
        let txn = db.begin().await?;

        // Load enrollment (verifies it exists).
        let enrollment = atlas_campaign_enrollment::Entity::find_by_id(payload.enrollment_id)
            .filter(atlas_campaign_enrollment::Column::TenantId.eq(tenant_id))
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("Enrollment {} not found", payload.enrollment_id))?;

        let campaign_id = enrollment.campaign_id;
        let now = Utc::now();

        // Write the event row.
        let event = atlas_campaign_event::ActiveModel {
            id: Set(Uuid::new_v4()),
            enrollment_id: Set(payload.enrollment_id),
            campaign_id: Set(campaign_id),
            tenant_id: Set(tenant_id),
            sequence_step_id: Set(payload.sequence_step_id),
            event_type: Set(payload.event_type.to_string()),
            channel: Set(payload.channel.to_string()),
            link_clicked: Set(payload.link_clicked),
            ip_address: Set(payload.ip_address),
            user_agent: Set(payload.user_agent),
            metadata: Set(payload.metadata),
            occurred_at: Set(now),
        }
        .insert(&txn)
        .await?;

        // ── Enum-driven dispatch ───────────────────────────────────────────────
        //
        // Match on CampaignEventType to determine:
        //   1. Which campaign-level counter to increment
        //   2. Whether the enrollment status must change
        //   3. Whether conversion tracking fields need writing
        //
        // Adding a new event type requires updating this match — the compiler
        // will enforce exhaustiveness, preventing silent counter misrouting.

        let maybe_counter: Option<CampaignCounter> = match &payload.event_type {
            CampaignEventType::Sent         => None, // not tracked in counters
            CampaignEventType::Delivered    => None,
            CampaignEventType::Opened       => Some(CampaignCounter::Opens),
            CampaignEventType::Clicked      => Some(CampaignCounter::Clicks),
            CampaignEventType::Replied      => Some(CampaignCounter::Replies),
            CampaignEventType::FormFill     => None,
            CampaignEventType::Converted    => Some(CampaignCounter::Conversions),
            CampaignEventType::Bounced      => None,
            CampaignEventType::Unsubscribed => None,
            CampaignEventType::SpamReported => None,
        };

        if let Some(counter) = maybe_counter {
            Self::increment_counter_in_txn(&txn, campaign_id, counter).await?;
        }

        // ── Enrollment state transitions ───────────────────────────────────────
        let terminal_status: Option<(EnrollmentStatus, &str)> = match &payload.event_type {
            CampaignEventType::Converted    => Some((EnrollmentStatus::Exited, "converted")),
            CampaignEventType::Replied      => Some((EnrollmentStatus::Exited, "replied")),
            CampaignEventType::Bounced      => Some((EnrollmentStatus::Bounced, "bounced")),
            CampaignEventType::Unsubscribed => Some((EnrollmentStatus::Unsubscribed, "unsubscribed")),
            CampaignEventType::SpamReported => Some((EnrollmentStatus::Exited, "spam_reported")),
            // All other events keep the enrollment active.
            _ => None,
        };

        if let Some((new_status, exit_reason)) = terminal_status {
            let mut active: atlas_campaign_enrollment::ActiveModel = enrollment.into();
            active.status = Set(new_status.to_string());
            active.exit_reason = Set(Some(exit_reason.to_string()));
            active.exit_at = Set(Some(now));

            // Write conversion FK only on Converted events.
            if matches!(payload.event_type, CampaignEventType::Converted) {
                active.converted_at = Set(Some(now));
                active.conversion_entity_type = Set(payload.conversion_entity_type);
                active.conversion_entity_id = Set(payload.conversion_entity_id);
            }

            active.update(&txn).await?;
        }

        txn.commit().await?;

        tracing::info!(
            %tenant_id, %campaign_id, enrollment_id = %payload.enrollment_id,
            event_type = %payload.event_type, channel = %payload.channel,
            "CampaignService::record_event: recorded"
        );

        Ok(event)
    }

    // ── List by subject entity ────────────────────────────────────────────────

    /// Find all campaigns tied to a specific platform entity (e.g. all campaigns
    /// promoting a specific asset, event, or opportunity).
    pub async fn find_by_subject(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<Vec<atlas_campaign::Model>> {
        Ok(atlas_campaign::Entity::find()
            .filter(atlas_campaign::Column::TenantId.eq(tenant_id))
            .filter(atlas_campaign::Column::SubjectEntityType.eq(entity_type))
            .filter(atlas_campaign::Column::SubjectEntityId.eq(entity_id))
            .order_by_desc(atlas_campaign::Column::CreatedAt)
            .all(db)
            .await?)
    }

    // ── Enrollment listing ────────────────────────────────────────────────────

    /// List enrollments for a campaign, optionally filtered by status.
    pub async fn list_enrollments(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        campaign_id: Uuid,
        status_filter: Option<EnrollmentStatus>,
    ) -> Result<Vec<atlas_campaign_enrollment::Model>> {
        // Verify campaign ownership.
        Self::get(db, tenant_id, campaign_id).await?;

        let mut q = atlas_campaign_enrollment::Entity::find()
            .filter(atlas_campaign_enrollment::Column::CampaignId.eq(campaign_id));

        if let Some(st) = status_filter {
            q = q.filter(atlas_campaign_enrollment::Column::Status.eq(st.to_string()));
        }

        Ok(q.order_by_asc(atlas_campaign_enrollment::Column::EnrolledAt)
            .all(db)
            .await?)
    }


    // ── Hierarchy ─────────────────────────────────────────────────────────────

    /// Return all direct children of a campaign (depth = 1).
    /// Use `CampaignFilter { parent_campaign_id: Some(id), .. }` with `list()`
    /// if you also need type/status filtering. This is the zero-filter shortcut.
    pub async fn find_children(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        parent_id: Uuid,
    ) -> Result<Vec<atlas_campaign::Model>> {
        // Verify the parent exists and belongs to this tenant.
        Self::get(db, tenant_id, parent_id).await?;

        Ok(atlas_campaign::Entity::find()
            .filter(atlas_campaign::Column::TenantId.eq(tenant_id))
            .filter(atlas_campaign::Column::ParentCampaignId.eq(parent_id))
            .order_by_asc(atlas_campaign::Column::CreatedAt)
            .all(db)
            .await?)
    }

    /// Roll up counter totals across the entire campaign subtree rooted at
    /// `root_id`. Uses a recursive CTE so it works at any depth.
    ///
    /// Returns `(total_contacts, total_opens, total_clicks, total_replies, total_conversions)`
    /// summed from the root campaign + all descendants.
    pub async fn get_hierarchy_stats(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        root_id: Uuid,
    ) -> Result<HierarchyStats> {
        use sea_orm::ConnectionTrait;
        use sea_orm::FromQueryResult;

        // Verify root belongs to this tenant.
        Self::get(db, tenant_id, root_id).await?;

        // Recursive CTE: walk the campaign tree top-down.
        let sql = format!(
            r#"
            WITH RECURSIVE campaign_tree AS (
                -- Anchor: the root campaign itself
                SELECT id, total_contacts, total_opens, total_clicks,
                       total_replies, total_conversions
                FROM atlas_campaigns
                WHERE id = '{root_id}' AND tenant_id = '{tenant_id}'

                UNION ALL

                -- Recursive step: direct children
                SELECT c.id, c.total_contacts, c.total_opens, c.total_clicks,
                       c.total_replies, c.total_conversions
                FROM atlas_campaigns c
                INNER JOIN campaign_tree ct ON c.parent_campaign_id = ct.id
                WHERE c.tenant_id = '{tenant_id}'
            )
            SELECT
                COALESCE(SUM(total_contacts),   0)::BIGINT AS contacts,
                COALESCE(SUM(total_opens),      0)::BIGINT AS opens,
                COALESCE(SUM(total_clicks),     0)::BIGINT AS clicks,
                COALESCE(SUM(total_replies),    0)::BIGINT AS replies,
                COALESCE(SUM(total_conversions),0)::BIGINT AS conversions
            FROM campaign_tree
            "#
        );

        #[derive(Debug, FromQueryResult)]
        struct Row {
            contacts:    i64,
            opens:       i64,
            clicks:      i64,
            replies:     i64,
            conversions: i64,
        }

        let row = Row::find_by_statement(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            sql,
        ))
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("hierarchy stats query returned no rows for {root_id}"))?;

        Ok(HierarchyStats {
            total_contacts:    row.contacts,
            total_opens:       row.opens,
            total_clicks:      row.clicks,
            total_replies:     row.replies,
            total_conversions: row.conversions,
        })
    }

    // ── Internal counter helpers ──────────────────────────────────────────────

    async fn increment_counter(
        db: &DatabaseConnection,
        campaign_id: Uuid,
        counter: CampaignCounter,
    ) -> Result<()> {
        use sea_orm::ConnectionTrait;
        let col = counter.column_name();
        db.execute_unprepared(&format!(
            "UPDATE atlas_campaigns SET {col} = {col} + 1, updated_at = NOW() WHERE id = '{campaign_id}'"
        ))
        .await
        .map_err(|e| anyhow!("increment_counter({col}) failed for {campaign_id}: {e:#}"))?;
        Ok(())
    }

    async fn increment_counter_in_txn(
        txn: &sea_orm::DatabaseTransaction,
        campaign_id: Uuid,
        counter: CampaignCounter,
    ) -> Result<()> {
        use sea_orm::ConnectionTrait;
        let col = counter.column_name();
        txn.execute_unprepared(&format!(
            "UPDATE atlas_campaigns SET {col} = {col} + 1, updated_at = NOW() WHERE id = '{campaign_id}'"
        ))
        .await
        .map_err(|e| anyhow!("increment_counter_in_txn({col}) failed for {campaign_id}: {e:#}"))?;
        Ok(())
    }
}

// ── Counter discriminant ──────────────────────────────────────────────────────
//
// Internal enum used by increment_counter* helpers to select the right column.
// Named by what the counter tracks, not by the DB column name — prevents the
// "which string goes here?" ambiguity that raw string dispatch would create.

enum CampaignCounter {
    Contacts,
    Opens,
    Clicks,
    Replies,
    Conversions,
}

impl CampaignCounter {
    fn column_name(&self) -> &'static str {
        match self {
            CampaignCounter::Contacts    => "total_contacts",
            CampaignCounter::Opens       => "total_opens",
            CampaignCounter::Clicks      => "total_clicks",
            CampaignCounter::Replies     => "total_replies",
            CampaignCounter::Conversions => "total_conversions",
        }
    }
}

// ── Public return types ───────────────────────────────────────────────────────

/// Roll-up counters for a campaign and all its descendants, returned by
/// `CampaignService::get_hierarchy_stats()`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HierarchyStats {
    /// Total contacts enrolled across root + all descendant campaigns.
    pub total_contacts: i64,
    /// Total email/message opens.
    pub total_opens: i64,
    /// Total link clicks.
    pub total_clicks: i64,
    /// Total replies.
    pub total_replies: i64,
    /// Total conversions (bookings, applications, sales, etc.).
    pub total_conversions: i64,
}
