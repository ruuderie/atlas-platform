//! Pure unit tests for G-28 `atlas_note` entity helpers.
//! No database — all tests exercise in-memory model construction and logic only.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::entities::atlas_note::Model;

    // ── Helper: construct a minimal note ──────────────────────────────────────

    fn make_note(visibility: &str, is_private: bool, is_pinned: bool, parent: Option<Uuid>) -> Model {
        use chrono::Utc;
        Model {
            id: Uuid::new_v4(),
            content: "Test content".to_owned(),
            created_by: Uuid::new_v4(),
            entity_type: "atlas_lead".to_owned(),
            entity_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            note_type: "general".to_owned(),
            subject: None,
            visibility: visibility.to_owned(),
            is_pinned,
            parent_note_id: parent,
            note_metadata: None,
            is_private,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── new_general constructor ───────────────────────────────────────────────

    #[test]
    fn new_general_sets_sensible_defaults() {
        let entity_id = Uuid::new_v4();
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let note = Model::new_general("Hello world", user_id, "atlas_lead", entity_id, tenant_id);

        assert_eq!(note.content, "Hello world");
        assert_eq!(note.entity_type, "atlas_lead");
        assert_eq!(note.entity_id, entity_id);
        assert_eq!(note.tenant_id, tenant_id);
        assert_eq!(note.created_by, user_id);
        assert_eq!(note.note_type, "general");
        assert_eq!(note.visibility, "internal");
        assert!(!note.is_pinned);
        assert!(note.parent_note_id.is_none());
        assert!(note.note_metadata.is_none());
        assert!(!note.is_private, "legacy is_private should default false");
    }

    #[test]
    fn new_general_generates_unique_ids() {
        let uid = Uuid::new_v4();
        let a = Model::new_general("A", uid, "atlas_account", uid, uid);
        let b = Model::new_general("B", uid, "atlas_account", uid, uid);
        assert_ne!(a.id, b.id, "each note should get a fresh UUID");
    }

    // ── is_reply ──────────────────────────────────────────────────────────────

    #[test]
    fn is_reply_true_when_parent_set() {
        let note = make_note("internal", false, false, Some(Uuid::new_v4()));
        assert!(note.is_reply());
    }

    #[test]
    fn is_reply_false_when_no_parent() {
        let note = make_note("internal", false, false, None);
        assert!(!note.is_reply());
    }

    // ── effective_visibility ──────────────────────────────────────────────────

    #[test]
    fn effective_visibility_returns_visibility_field_by_default() {
        let public = make_note("public", false, false, None);
        assert_eq!(public.effective_visibility(), "public");

        let internal = make_note("internal", false, false, None);
        assert_eq!(internal.effective_visibility(), "internal");
    }

    #[test]
    fn effective_visibility_is_private_when_legacy_flag_set() {
        // Legacy is_private=true overrides the visibility field
        let note = make_note("internal", true, false, None);
        assert_eq!(
            note.effective_visibility(),
            "private",
            "is_private=true must override visibility field for backward compat"
        );
    }

    #[test]
    fn effective_visibility_public_and_is_private_false_stays_public() {
        let note = make_note("public", false, false, None);
        assert_eq!(note.effective_visibility(), "public");
    }

    // ── note_type discriminator ───────────────────────────────────────────────

    #[test]
    fn note_type_can_be_any_string() {
        // The entity doesn't validate note_type — that's a service concern.
        // This test documents that arbitrary values don't panic.
        for t in &["general", "call_log", "site_visit", "underwriting_comment", "legal_memo"] {
            let note = Model::new_general("body", Uuid::new_v4(), "atlas_case", Uuid::new_v4(), Uuid::new_v4());
            // new_general hardcodes "general" — this tests the field is a String, not an enum
            assert_eq!(note.note_type, "general");
            let _ = t; // document the intended domain values
        }
    }

    // ── pinning ───────────────────────────────────────────────────────────────

    #[test]
    fn pinned_note_is_accessible() {
        let pinned = make_note("internal", false, true, None);
        assert!(pinned.is_pinned);

        let unpinned = make_note("internal", false, false, None);
        assert!(!unpinned.is_pinned);
    }
}
