use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

/// G-28: atlas_note — Universal Polymorphic Note.
///
/// The underlying Postgres table is `notes` (not renamed for backward compat).
/// Any platform entity can have notes by setting `entity_type` + `entity_id`.
///
/// Supported `entity_type` values (non-exhaustive):
///   'atlas_asset' | 'atlas_account' | 'atlas_contact' | 'atlas_lead' |
///   'atlas_opportunity' | 'atlas_case' | 'atlas_contract' | 'atlas_application' |
///   'atlas_service_provider' | 'atlas_portfolio' | 'deal' | 'customer'
///
/// `note_type` discriminator (app-defined, non-exhaustive):
///   'general' | 'call_log' | 'site_visit' | 'inspection' |
///   'underwriting_comment' | 'legal_memo' | 'compliance_note' | 'coach_feedback'
///
/// Threading: `parent_note_id` points to another note in the same entity thread.
/// Top-level notes have `parent_note_id = NULL`.
///
/// `visibility`:
///   'public'   — visible to all parties including external (e.g. tenant portal)
///   'internal' — visible only to internal users (default)
///   'private'  — visible only to the creating user
///
/// `is_private` is kept for backward compatibility; `visibility` is canonical.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "notes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    /// Rich text or plain text note body.
    pub content: String,
    pub created_by: Uuid,
    /// Polymorphic entity type discriminator.
    pub entity_type: String,
    pub entity_id: Uuid,
    pub tenant_id: Uuid,
    // ── G-28 platform columns ─────────────────────────────────────────────────
    /// Application-defined note type. Defaults to 'general'.
    pub note_type: String,
    /// Short heading shown in feed list views (optional — content is the body).
    pub subject: Option<String>,
    /// 'public' | 'internal' | 'private'
    pub visibility: String,
    /// Pinned notes are surfaced first in the entity note feed.
    pub is_pinned: bool,
    /// Self-referential FK for note threads. None = top-level note.
    pub parent_note_id: Option<Uuid>,
    /// App-specific payload.
    /// Rich text: {"delta": {...}} | Call transcript: {"url": "...", "text": "..."}
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub note_metadata: Option<Value>,
    // ── Legacy compat ─────────────────────────────────────────────────────────
    /// Kept for backward compat. Use `visibility = 'private'` going forward.
    pub is_private: bool,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTime<Utc>,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    /// Create a new top-level note with sensible defaults.
    pub fn new_general(
        content: impl Into<String>,
        created_by: Uuid,
        entity_type: impl Into<String>,
        entity_id: Uuid,
        tenant_id: Uuid,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            content: content.into(),
            created_by,
            entity_type: entity_type.into(),
            entity_id,
            tenant_id,
            note_type: "general".to_owned(),
            subject: None,
            visibility: "internal".to_owned(),
            is_pinned: false,
            parent_note_id: None,
            note_metadata: None,
            is_private: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// True if this note is a thread reply.
    pub fn is_reply(&self) -> bool {
        self.parent_note_id.is_some()
    }

    /// Resolved visibility — canonical check that handles legacy `is_private`.
    pub fn effective_visibility(&self) -> &str {
        if self.is_private {
            return "private";
        }
        &self.visibility
    }
}
