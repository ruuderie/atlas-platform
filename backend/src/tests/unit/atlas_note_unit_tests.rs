//! Pure unit tests for G-28 `atlas_note` entity helpers.
//! No database — all tests exercise in-memory model construction and logic only.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::entities::atlas_note::Model;
    use crate::types::note::NoteVisibility;

    // ── Helper: construct a minimal note ──────────────────────────────────────

    fn make_note(visibility: &str, is_pinned: bool, parent: Option<Uuid>) -> Model {
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
            is_private: false, // always false — legacy field not used in logic
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
        let note = make_note("internal", false, Some(Uuid::new_v4()));
        assert!(note.is_reply());
    }

    #[test]
    fn is_reply_false_when_no_parent() {
        let note = make_note("internal", false, None);
        assert!(!note.is_reply());
    }

    // ── visibility_typed ──────────────────────────────────────────────────────

    #[test]
    fn visibility_typed_returns_correct_variant_for_each_slug() {
        for (slug, expected) in &[
            ("public",   NoteVisibility::Public),
            ("internal", NoteVisibility::Internal),
            ("private",  NoteVisibility::Private),
        ] {
            let note = make_note(slug, false, None);
            assert_eq!(
                note.visibility_typed().unwrap(),
                *expected,
                "visibility_typed failed for '{}'",
                slug
            );
        }
    }

    #[test]
    fn visibility_typed_err_for_unknown_slug() {
        let note = make_note("confidential", false, None);
        assert!(note.visibility_typed().is_err());
    }

    #[test]
    fn public_note_is_externally_visible() {
        let note = make_note("public", false, None);
        assert!(note.visibility_typed().unwrap().is_external_visible());
    }

    #[test]
    fn internal_note_is_not_externally_visible() {
        let note = make_note("internal", false, None);
        assert!(!note.visibility_typed().unwrap().is_external_visible());
    }

    // ── note_type discriminator ───────────────────────────────────────────────

    #[test]
    fn new_general_note_type_typed_returns_general() {
        let note = Model::new_general("body", Uuid::new_v4(), "atlas_lead", Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(note.note_type_typed().to_string(), "general");
    }

    #[test]
    fn note_type_typed_known_slugs_roundtrip() {
        use crate::types::note::NoteType;
        let cases = [
            ("call_log",             NoteType::CallLog),
            ("site_visit",           NoteType::SiteVisit),
            ("inspection",           NoteType::Inspection),
            ("underwriting_comment", NoteType::UnderwritingComment),
            ("legal_memo",           NoteType::LegalMemo),
            ("compliance_note",      NoteType::ComplianceNote),
            ("coach_feedback",       NoteType::CoachFeedback),
        ];
        for (slug, expected) in &cases {
            let mut note = make_note("internal", false, None);
            note.note_type = slug.to_string();
            assert_eq!(note.note_type_typed(), *expected);
        }
    }

    #[test]
    fn note_type_typed_unknown_slug_produces_other_variant() {
        use crate::types::note::NoteType;
        let mut note = make_note("internal", false, None);
        note.note_type = "deal_memo".to_owned();
        assert!(matches!(note.note_type_typed(), NoteType::Other(ref s) if s == "deal_memo"));
    }

    // ── pinning ───────────────────────────────────────────────────────────────

    #[test]
    fn pinned_note_is_accessible() {
        let pinned = make_note("internal", true, None);
        assert!(pinned.is_pinned);

        let unpinned = make_note("internal", false, None);
        assert!(!unpinned.is_pinned);
    }
}
