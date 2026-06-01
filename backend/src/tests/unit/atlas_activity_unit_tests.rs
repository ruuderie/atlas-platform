//! Pure unit tests for G-29 `atlas_activity` entity helpers.
//! No database — all tests exercise in-memory model construction and logic only.

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use serde_json::json;
    use crate::entities::atlas_activity::Model;

    // ── Helper: construct a minimal activity ──────────────────────────────────

    fn make_activity(
        subject_type: Option<&str>,
        subject_id: Option<Uuid>,
        lead_id: Option<Uuid>,
        contact_id: Option<Uuid>,
        deal_id: Option<Uuid>,
        outcome: Option<&str>,
    ) -> Model {
        use chrono::Utc;
        Model {
            id: Uuid::new_v4(),
            tenant_id: Some(Uuid::new_v4()),
            subject_entity_type: subject_type.map(str::to_owned),
            subject_entity_id: subject_id,
            account_id: None,
            deal_id,
            customer_id: None,
            lead_id,
            contact_id,
            case_id: None,
            activity_type: "Log".to_owned(),
            title: "Test activity".to_owned(),
            description: None,
            status: "Open".to_owned(),
            due_date: None,
            completed_at: None,
            associated_entities: json!([]),
            created_by: Uuid::new_v4(),
            assigned_to: None,
            activity_category: None,
            direction: None,
            duration_seconds: None,
            outcome: outcome.map(str::to_owned),
            scheduled_at: None,
            activity_metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    // ── primary_subject resolution ────────────────────────────────────────────

    #[test]
    fn primary_subject_returns_polymorphic_columns_first() {
        let id = Uuid::new_v4();
        let activity = make_activity(Some("atlas_lead"), Some(id), Some(Uuid::new_v4()), None, None, None);
        let (t, resolved_id) = activity.primary_subject().expect("should have subject");
        assert_eq!(t, "atlas_lead");
        assert_eq!(resolved_id, id, "polymorphic columns should win over legacy FKs");
    }

    #[test]
    fn primary_subject_falls_back_to_lead_id() {
        let lead = Uuid::new_v4();
        let activity = make_activity(None, None, Some(lead), None, None, None);
        let (t, id) = activity.primary_subject().expect("should have subject via lead_id");
        assert_eq!(t, "lead");
        assert_eq!(id, lead);
    }

    #[test]
    fn primary_subject_lead_wins_over_contact_in_legacy_priority() {
        let lead = Uuid::new_v4();
        let contact = Uuid::new_v4();
        // Both set — lead_id should win (CRM priority order: lead > contact > customer > deal)
        let activity = make_activity(None, None, Some(lead), Some(contact), None, None);
        let (t, id) = activity.primary_subject().expect("should have subject");
        assert_eq!(t, "lead");
        assert_eq!(id, lead);
    }

    #[test]
    fn primary_subject_contact_wins_when_no_lead() {
        let contact = Uuid::new_v4();
        let activity = make_activity(None, None, None, Some(contact), None, None);
        let (t, id) = activity.primary_subject().unwrap();
        assert_eq!(t, "contact");
        assert_eq!(id, contact);
    }

    #[test]
    fn primary_subject_deal_is_last_legacy_resort() {
        let deal = Uuid::new_v4();
        let activity = make_activity(None, None, None, None, Some(deal), None);
        let (t, id) = activity.primary_subject().unwrap();
        assert_eq!(t, "deal");
        assert_eq!(id, deal);
    }

    #[test]
    fn primary_subject_returns_none_when_no_subject_set() {
        let activity = make_activity(None, None, None, None, None, None);
        assert!(activity.primary_subject().is_none());
    }

    // ── is_completed_communication ────────────────────────────────────────────

    #[test]
    fn connected_call_is_completed_communication() {
        let a = make_activity(None, None, None, None, None, Some("connected"));
        assert!(a.is_completed_communication());
    }

    #[test]
    fn meeting_held_is_completed_communication() {
        let a = make_activity(None, None, None, None, None, Some("meeting_held"));
        assert!(a.is_completed_communication());
    }

    #[test]
    fn completed_outcome_is_completed_communication() {
        let a = make_activity(None, None, None, None, None, Some("completed"));
        assert!(a.is_completed_communication());
    }

    #[test]
    fn voicemail_is_not_completed_communication() {
        let a = make_activity(None, None, None, None, None, Some("voicemail"));
        assert!(!a.is_completed_communication());
    }

    #[test]
    fn no_answer_is_not_completed_communication() {
        let a = make_activity(None, None, None, None, None, Some("no_answer"));
        assert!(!a.is_completed_communication());
    }

    #[test]
    fn no_show_is_not_completed_communication() {
        let a = make_activity(None, None, None, None, None, Some("no_show"));
        assert!(!a.is_completed_communication());
    }

    #[test]
    fn none_outcome_is_not_completed_communication() {
        let a = make_activity(None, None, None, None, None, None);
        assert!(!a.is_completed_communication());
    }

    // ── Polymorphic subject type exhaustion ───────────────────────────────────

    #[test]
    fn polymorphic_subject_type_can_reference_any_entity() {
        // The field is a String — this documents intended entity_type values
        for (entity_type, _) in &[
            ("atlas_lead", "lead entity"),
            ("atlas_account", "account entity"),
            ("atlas_contact", "contact entity"),
            ("atlas_opportunity", "opportunity entity"),
            ("atlas_case", "case entity"),
            ("atlas_asset", "asset entity"),
        ] {
            let id = Uuid::new_v4();
            let a = make_activity(Some(entity_type), Some(id), None, None, None, None);
            let (t, resolved_id) = a.primary_subject().unwrap();
            assert_eq!(t, *entity_type);
            assert_eq!(resolved_id, id);
        }
    }
}
