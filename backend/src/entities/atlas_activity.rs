use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// G-29: atlas_activity — Universal Polymorphic Activity Log.
///
/// The underlying Postgres table is `activity` (not renamed for backward compat).
///
/// Activities are the canonical record of "something that happened or was scheduled
/// between a team member and a platform entity." This covers:
///   - CRM: calls, emails, demos, follow-ups, meetings
///   - PM: inspections, maintenance visits, repair checks
///   - Insurance: adjuster site visits, coverage reviews, claim events
///   - Recruiting: screening calls, interviews, reference checks
///   - Pipeline: stage transitions, qualification events, deal reviews
///
/// Polymorphic subject:
///   `subject_entity_type` + `subject_entity_id` replace the legacy hard-coded FK
///   columns (lead_id, deal_id, customer_id, contact_id, case_id, account_id).
///   Legacy FK columns are kept until all handlers migrate.
///
/// `associated_entities` JSONB (from m20260523) stores the full list of entities
/// touched by this activity (e.g. a call that involves both a lead and a contact).
///
/// `activity_type` (legacy enum, keep for compat):
///   'Log' | 'Task' | 'Event'
///
/// `activity_category` (new platform discriminator):
///   'communication' | 'meeting' | 'task' | 'system_event' | 'pipeline_event'
///
/// `direction`: 'inbound' | 'outbound' | 'n_a'
///
/// `outcome`:
///   'connected' | 'voicemail' | 'no_answer' | 'bounced' |
///   'meeting_held' | 'no_show' | 'completed' | 'cancelled'
///
/// `status` (legacy):
///   'Open' | 'Pending' | 'Completed'
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "activity")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    // ── Polymorphic subject (G-29 canonical pattern) ──────────────────────────
    /// Primary entity this activity is about.
    pub subject_entity_type: Option<String>,
    pub subject_entity_id: Option<Uuid>,
    // ── Legacy hard-coded FK columns (kept for backward compat) ───────────────
    pub account_id: Option<Uuid>,
    pub deal_id: Option<Uuid>,
    pub customer_id: Option<Uuid>,
    pub lead_id: Option<Uuid>,
    pub contact_id: Option<Uuid>,
    pub case_id: Option<Uuid>,
    // ── Core fields ───────────────────────────────────────────────────────────
    /// Legacy type discriminator: 'Log' | 'Task' | 'Event'
    pub activity_type: String,
    pub title: String,
    pub description: Option<String>,
    /// 'Open' | 'Pending' | 'Completed'
    pub status: String,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub due_date: Option<DateTime<Utc>>,
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub completed_at: Option<DateTime<Utc>>,
    /// Multi-entity references: [{"entity_type":"lead","entity_id":"..."}]
    #[sea_orm(column_type = "Json")]
    pub associated_entities: Value,
    pub created_by: Uuid,
    pub assigned_to: Option<Uuid>,
    // ── G-29 platform columns ─────────────────────────────────────────────────
    /// 'communication' | 'meeting' | 'task' | 'system_event' | 'pipeline_event'
    pub activity_category: Option<String>,
    /// 'inbound' | 'outbound' | 'n_a'
    pub direction: Option<String>,
    /// Duration in seconds (calls, demos, meetings).
    pub duration_seconds: Option<i32>,
    /// Call/meeting/email outcome.
    pub outcome: Option<String>,
    /// When this activity is scheduled for (vs created_at = log time).
    #[sea_orm(column_type = "TimestampWithTimeZone", nullable)]
    pub scheduled_at: Option<DateTime<Utc>>,
    /// App-specific payload.
    /// call: {"recording_url":"...","transcript":"..."}
    /// email: {"subject":"...","body_preview":"...","message_id":"..."}
    /// meeting: {"location":"...","attendees":["..."]}
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub activity_metadata: Option<Value>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Resolve the primary subject entity, preferring the new polymorphic columns
    /// over the legacy FK columns.
    pub fn primary_subject(&self) -> Option<(&str, Uuid)> {
        if let (Some(t), Some(id)) = (&self.subject_entity_type, self.subject_entity_id) {
            return Some((t.as_str(), id));
        }
        // Fall back to legacy columns in CRM priority order
        if let Some(id) = self.lead_id {
            return Some(("lead", id));
        }
        if let Some(id) = self.contact_id {
            return Some(("contact", id));
        }
        if let Some(id) = self.customer_id {
            return Some(("customer", id));
        }
        if let Some(id) = self.deal_id {
            return Some(("deal", id));
        }
        if let Some(id) = self.case_id {
            return Some(("atlas_case", id));
        }
        if let Some(id) = self.account_id {
            return Some(("atlas_account", id));
        }
        None
    }

    /// True if this is a completed communication activity (call connected, meeting held).
    pub fn is_completed_communication(&self) -> bool {
        matches!(
            self.outcome.as_deref(),
            Some("connected" | "meeting_held" | "completed")
        )
    }
}
