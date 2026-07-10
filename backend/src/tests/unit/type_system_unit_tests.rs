//! Comprehensive unit tests for all new type-system modules.
//!
//! Covers `TryFrom<String>` / `Display` roundtrips, terminal-state logic,
//! `is_completed_interaction`, and typed JSONB helpers on entities.
//!
//! No database required — all tests are pure in-memory.

#[cfg(test)]
mod tests {
    // ── LeadStatus ────────────────────────────────────────────────────────────

    use crate::types::lead::{
        CompanyType, DataSource, LeadStatus, LocationType, OpportunityStatus, OpportunityType,
    };
    use crate::types::activity::{
        ActivityCategory, ActivityDirection, ActivityOutcome, ActivityStatus, ActivityType,
    };
    use crate::types::note::{NoteType, NoteVisibility};
    use crate::types::account::{AccountStatus, AccountType, TaxIdType};
    use crate::types::outbox::{OutboxJobStatus, OutboxJobType};
    use crate::types::scorecard::ScorecardEntityType;
    use crate::types::shared::MailingAddress;

    // ── LeadStatus: all variants round-trip through Display / TryFrom ─────────

    #[test]
    fn lead_status_display_roundtrip() {
        let variants = [
            (LeadStatus::New, "new"),
            (LeadStatus::Contacted, "contacted"),
            (LeadStatus::Qualifying, "qualifying"),
            (LeadStatus::Qualified, "qualified"),
            (LeadStatus::Disqualified, "disqualified"),
            (LeadStatus::Converted, "converted"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = LeadStatus::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn lead_status_unknown_slug_returns_err() {
        assert!(LeadStatus::try_from("pending".to_string()).is_err());
        assert!(LeadStatus::try_from("".to_string()).is_err());
        assert!(LeadStatus::try_from("CONVERTED".to_string()).is_err(), "must be lowercase");
    }

    #[test]
    fn lead_status_terminal_states() {
        assert!(LeadStatus::Converted.is_terminal());
        assert!(LeadStatus::Disqualified.is_terminal());

        assert!(!LeadStatus::New.is_terminal());
        assert!(!LeadStatus::Contacted.is_terminal());
        assert!(!LeadStatus::Qualifying.is_terminal());
        assert!(!LeadStatus::Qualified.is_terminal());
    }

    // ── DataSource: From (infallible) ─────────────────────────────────────────

    #[test]
    fn data_source_known_slugs_parse_correctly() {
        let cases = [
            ("fmcsa", DataSource::Fmcsa),
            ("business_leads_usa", DataSource::BusinessLeadsUsa),
            ("dot_registry", DataSource::DotRegistry),
            ("manual", DataSource::Manual),
            ("web_form", DataSource::WebForm),
            ("zoominfo", DataSource::Zoominfo),
            ("mwbe", DataSource::Mwbe),
            ("referral", DataSource::Referral),
        ];
        for (slug, expected) in &cases {
            let parsed = DataSource::from(*slug);
            assert_eq!(&parsed, expected, "DataSource::from failed for '{}'", slug);
            assert_eq!(parsed.to_string(), *slug, "Display mismatch for {:?}", expected);
        }
    }

    #[test]
    fn data_source_unknown_slug_becomes_other() {
        let ds = DataSource::from("custom_import_v3");
        assert!(
            matches!(ds, DataSource::Other(ref s) if s == "custom_import_v3"),
            "Expected Other variant"
        );
    }

    #[test]
    fn data_source_auto_provisions_scorecard() {
        assert!(DataSource::Fmcsa.auto_provisions_scorecard());
        assert!(DataSource::BusinessLeadsUsa.auto_provisions_scorecard());
        assert!(DataSource::DotRegistry.auto_provisions_scorecard());

        assert!(!DataSource::Manual.auto_provisions_scorecard());
        assert!(!DataSource::WebForm.auto_provisions_scorecard());
        assert!(!DataSource::Zoominfo.auto_provisions_scorecard());
    }

    // ── CompanyType ───────────────────────────────────────────────────────────

    #[test]
    fn company_type_roundtrip() {
        for (variant, slug) in &[
            (CompanyType::Public, "public"),
            (CompanyType::Private, "private"),
            (CompanyType::Government, "government"),
            (CompanyType::Nonprofit, "nonprofit"),
            (CompanyType::Individual, "individual"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            let parsed = CompanyType::try_from(slug.to_string()).unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    // ── LocationType ──────────────────────────────────────────────────────────

    #[test]
    fn location_type_roundtrip() {
        for (variant, slug) in &[
            (LocationType::Headquarters, "headquarters"),
            (LocationType::Branch, "branch"),
            (LocationType::Single, "single"),
            (LocationType::Franchise, "franchise"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            let parsed = LocationType::try_from(slug.to_string()).unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    // ── OpportunityType / OpportunityStatus ───────────────────────────────────

    #[test]
    fn opportunity_type_roundtrip() {
        let cases = [
            (OpportunityType::CrmLeadConversion, "crm_lead_conversion"),
            (OpportunityType::Manual, "manual"),
            (OpportunityType::Renewal, "renewal"),
            (OpportunityType::Upsell, "upsell"),
            (OpportunityType::Partner, "partner"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&OpportunityType::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn opportunity_status_closed_variants() {
        assert!(OpportunityStatus::ClosedWon.is_closed());
        assert!(OpportunityStatus::ClosedLost.is_closed());
        assert!(!OpportunityStatus::Prospecting.is_closed());
        assert!(!OpportunityStatus::Negotiation.is_closed());
    }

    // ── ActivityType / ActivityStatus ─────────────────────────────────────────

    #[test]
    fn activity_type_roundtrip() {
        for (variant, slug) in &[
            (ActivityType::Log, "Log"),
            (ActivityType::Task, "Task"),
            (ActivityType::Event, "Event"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&ActivityType::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn activity_status_is_done() {
        assert!(ActivityStatus::Completed.is_done());
        assert!(!ActivityStatus::Open.is_done());
        assert!(!ActivityStatus::Pending.is_done());
    }

    // ── ActivityCategory ──────────────────────────────────────────────────────

    #[test]
    fn activity_category_roundtrip() {
        let cases = [
            (ActivityCategory::Communication, "communication"),
            (ActivityCategory::Meeting, "meeting"),
            (ActivityCategory::Task, "task"),
            (ActivityCategory::SystemEvent, "system_event"),
            (ActivityCategory::PipelineEvent, "pipeline_event"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&ActivityCategory::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn activity_category_human_interaction_flag() {
        assert!(ActivityCategory::Communication.is_human_interaction());
        assert!(ActivityCategory::Meeting.is_human_interaction());
        assert!(!ActivityCategory::Task.is_human_interaction());
        assert!(!ActivityCategory::SystemEvent.is_human_interaction());
        assert!(!ActivityCategory::PipelineEvent.is_human_interaction());
    }

    // ── ActivityOutcome ───────────────────────────────────────────────────────

    #[test]
    fn activity_outcome_roundtrip() {
        let cases = [
            (ActivityOutcome::Connected, "connected"),
            (ActivityOutcome::Voicemail, "voicemail"),
            (ActivityOutcome::NoAnswer, "no_answer"),
            (ActivityOutcome::Bounced, "bounced"),
            (ActivityOutcome::MeetingHeld, "meeting_held"),
            (ActivityOutcome::NoShow, "no_show"),
            (ActivityOutcome::Completed, "completed"),
            (ActivityOutcome::Cancelled, "cancelled"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&ActivityOutcome::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn activity_outcome_completed_interaction_variants() {
        // Only three outcomes count as a substantive completed interaction
        assert!(ActivityOutcome::Connected.is_completed_interaction());
        assert!(ActivityOutcome::MeetingHeld.is_completed_interaction());
        assert!(ActivityOutcome::Completed.is_completed_interaction());

        assert!(!ActivityOutcome::Voicemail.is_completed_interaction());
        assert!(!ActivityOutcome::NoAnswer.is_completed_interaction());
        assert!(!ActivityOutcome::Bounced.is_completed_interaction());
        assert!(!ActivityOutcome::NoShow.is_completed_interaction());
        assert!(!ActivityOutcome::Cancelled.is_completed_interaction());
    }

    #[test]
    fn activity_outcome_unknown_returns_err() {
        assert!(ActivityOutcome::try_from("hung_up".to_string()).is_err());
    }

    // ── ActivityDirection ─────────────────────────────────────────────────────

    #[test]
    fn activity_direction_roundtrip() {
        for (variant, slug) in &[
            (ActivityDirection::Inbound, "inbound"),
            (ActivityDirection::Outbound, "outbound"),
            (ActivityDirection::Na, "n_a"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&ActivityDirection::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    // ── NoteVisibility ────────────────────────────────────────────────────────

    #[test]
    fn note_visibility_roundtrip() {
        for (variant, slug) in &[
            (NoteVisibility::Public, "public"),
            (NoteVisibility::Internal, "internal"),
            (NoteVisibility::Private, "private"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&NoteVisibility::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn note_visibility_external_visible() {
        assert!(NoteVisibility::Public.is_external_visible());
        assert!(!NoteVisibility::Internal.is_external_visible());
        assert!(!NoteVisibility::Private.is_external_visible());
    }

    #[test]
    fn note_visibility_unknown_returns_err() {
        assert!(NoteVisibility::try_from("confidential".to_string()).is_err());
    }

    // ── NoteType ──────────────────────────────────────────────────────────────

    #[test]
    fn note_type_known_slugs_roundtrip() {
        let cases = [
            (NoteType::General, "general"),
            (NoteType::CallLog, "call_log"),
            (NoteType::SiteVisit, "site_visit"),
            (NoteType::Inspection, "inspection"),
            (NoteType::UnderwritingComment, "underwriting_comment"),
            (NoteType::LegalMemo, "legal_memo"),
            (NoteType::ComplianceNote, "compliance_note"),
            (NoteType::CoachFeedback, "coach_feedback"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(variant.to_string(), *slug);
            assert!(matches!(NoteType::from(slug.to_string()), ref t if t.to_string() == *slug));
        }
    }

    #[test]
    fn note_type_unknown_becomes_other() {
        let nt = NoteType::from("deal_memo");
        assert!(matches!(nt, NoteType::Other(ref s) if s == "deal_memo"));
        assert_eq!(nt.to_string(), "deal_memo");
    }

    // ── AccountType / AccountStatus / TaxIdType ───────────────────────────────

    #[test]
    fn account_type_roundtrip() {
        for (variant, slug) in &[
            (AccountType::Individual, "individual"),
            (AccountType::Organization, "organization"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&AccountType::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn account_status_active_relationship() {
        assert!(AccountStatus::Active.is_active_relationship());
        assert!(AccountStatus::Prospect.is_active_relationship());
        assert!(!AccountStatus::Suspended.is_active_relationship());
        assert!(!AccountStatus::Archived.is_active_relationship());
    }

    #[test]
    fn tax_id_type_roundtrip() {
        for (variant, slug) in &[
            (TaxIdType::Ein, "ein"),
            (TaxIdType::Cnpj, "cnpj"),
            (TaxIdType::Cpf, "cpf"),
            (TaxIdType::Ssn, "ssn"),
            (TaxIdType::Tin, "tin"),
            (TaxIdType::Vat, "vat"),
            (TaxIdType::Usdot, "usdot"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&TaxIdType::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    // ── OutboxJobType / OutboxJobStatus ───────────────────────────────────────

    #[test]
    fn outbox_job_type_all_variants_roundtrip() {
        let cases = [
            (OutboxJobType::SendMagicLinkEmail, "send_magic_link_email"),
            (OutboxJobType::RecomputeScorecardAggregates, "recompute_scorecard_aggregates"),
            (OutboxJobType::RefreshScorecardTimeSeries, "refresh_scorecard_time_series"),
            (OutboxJobType::RefreshScorecardPortfolio, "refresh_scorecard_portfolio"),
            (OutboxJobType::CalibrateScorecardContributors, "calibrate_scorecard_contributors"),
            (OutboxJobType::EvaluateScorecardNudge, "evaluate_scorecard_nudge"),
            (OutboxJobType::ReleaseExpiredReservationHolds, "release_expired_reservation_holds"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = OutboxJobType::try_from(slug.to_string())
                .unwrap_or_else(|_| panic!("TryFrom failed for '{}'", slug));
            assert_eq!(&parsed, variant);
        }
    }

    #[test]
    fn outbox_job_type_unknown_returns_err() {
        assert!(OutboxJobType::try_from("bitcoin_sync".to_string()).is_err());
        assert!(OutboxJobType::try_from("".to_string()).is_err());
    }

    #[test]
    fn outbox_job_status_roundtrip() {
        for (variant, slug) in &[
            (OutboxJobStatus::Pending, "pending"),
            (OutboxJobStatus::Processing, "processing"),
            (OutboxJobStatus::Completed, "completed"),
            (OutboxJobStatus::Failed, "failed"),
        ] {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&OutboxJobStatus::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    // ── ScorecardEntityType ───────────────────────────────────────────────────

    #[test]
    fn scorecard_entity_type_platform_generics_roundtrip() {
        let cases = [
            (ScorecardEntityType::AtlasLead, "atlas_lead"),
            (ScorecardEntityType::AtlasOpportunity, "atlas_opportunity"),
            (ScorecardEntityType::AtlasAccount, "atlas_account"),
            (ScorecardEntityType::AtlasAsset, "atlas_asset"),
            (ScorecardEntityType::AtlasContact, "atlas_contact"),
            (ScorecardEntityType::AtlasPortfolio, "atlas_portfolio"),
            (ScorecardEntityType::AtlasCatalogEntry, "atlas_catalog_entry"),
            (ScorecardEntityType::AtlasServiceProvider, "atlas_service_provider"),
            (ScorecardEntityType::Tenant, "tenant"),
            (ScorecardEntityType::AppInstance, "app_instance"),
        ];
        for (variant, slug) in &cases {
            assert_eq!(variant.to_string(), *slug);
            assert_eq!(&ScorecardEntityType::try_from(slug.to_string()).unwrap(), variant);
        }
    }

    #[test]
    fn scorecard_entity_type_legacy_slugs_roundtrip() {
        assert_eq!(
            ScorecardEntityType::try_from("listing".to_string()).unwrap(),
            ScorecardEntityType::Listing
        );
        assert_eq!(
            ScorecardEntityType::try_from("profile".to_string()).unwrap(),
            ScorecardEntityType::Profile
        );
    }

    #[test]
    fn scorecard_entity_type_unknown_returns_err() {
        assert!(ScorecardEntityType::try_from("custom_entity".to_string()).is_err());
    }

    // ── MailingAddress ────────────────────────────────────────────────────────

    #[test]
    fn mailing_address_one_line_formats_correctly() {
        let addr = MailingAddress {
            street: Some("123 Main St".to_string()),
            city: Some("Austin".to_string()),
            state: Some("TX".to_string()),
            postal_code: Some("78701".to_string()),
            country: Some("US".to_string()),
            street2: None,
        };
        assert_eq!(addr.one_line().unwrap(), "123 Main St, Austin, TX, 78701, US");
    }

    #[test]
    fn mailing_address_empty_returns_none() {
        let addr = MailingAddress::default();
        assert!(addr.is_empty());
        assert!(addr.one_line().is_none());
    }

    #[test]
    fn mailing_address_serializes_and_deserializes() {
        let addr = MailingAddress {
            street: Some("456 Oak Ave".to_string()),
            city: Some("Portland".to_string()),
            state: Some("OR".to_string()),
            postal_code: Some("97201".to_string()),
            country: Some("US".to_string()),
            street2: None,
        };

        let json = serde_json::to_value(&addr).expect("should serialize");
        let deserialized: MailingAddress = serde_json::from_value(json).expect("should deserialize");

        assert_eq!(deserialized.street, addr.street);
        assert_eq!(deserialized.city, addr.city);
        assert_eq!(deserialized.postal_code, addr.postal_code);
    }

    // ── Entity helper: atlas_scorecard typed methods ──────────────────────────

    #[test]
    fn atlas_scorecard_dimension_vector_typed_parses_float_array() {
        use crate::entities::atlas_scorecard::Model as ScorecardModel;
        use serde_json::json;

        let raw_vec = vec![0.5f32, 0.75f32, 1.0f32];
        let json_val = serde_json::to_value(&raw_vec).unwrap();

        // Manually construct the minimum needed fields of a model for testing helpers
        // Note: this uses Default where possible to satisfy the struct constraint
        let sc = ScorecardModel {
            id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            template_id: uuid::Uuid::new_v4(),
            subject_entity_type: "atlas_lead".to_string(),
            subject_entity_id: uuid::Uuid::new_v4(),
            composite_score: None,
            confidence_level: "high".to_string(),
            total_contributors: 0,
            total_sessions: 0,
            total_entries: 50,
            dimension_vector: None,
            dimension_vector_v2: Some(json_val),
            has_data_mask: None,
            last_computed_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        let typed = sc.dimension_vector_v2_typed().unwrap().unwrap();
        assert_eq!(typed, raw_vec);
    }

    #[test]
    fn atlas_scorecard_has_data_mask_typed_parses_bool_array() {
        use crate::entities::atlas_scorecard::Model as ScorecardModel;

        let raw_mask = vec![true, false, true];
        let json_val = serde_json::to_value(&raw_mask).unwrap();

        let sc = ScorecardModel {
            id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            template_id: uuid::Uuid::new_v4(),
            subject_entity_type: "atlas_lead".to_string(),
            subject_entity_id: uuid::Uuid::new_v4(),
            composite_score: None,
            confidence_level: "high".to_string(),
            total_contributors: 0,
            total_sessions: 0,
            total_entries: 50,
            dimension_vector: None,
            dimension_vector_v2: None,
            has_data_mask: Some(json_val),
            last_computed_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        let typed = sc.has_data_mask_typed().unwrap().unwrap();
        assert_eq!(typed, raw_mask);
    }

    #[test]
    fn atlas_scorecard_subject_entity_type_typed_parses_known_slug() {
        use crate::entities::atlas_scorecard::Model as ScorecardModel;

        let sc = ScorecardModel {
            id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            template_id: uuid::Uuid::new_v4(),
            subject_entity_type: "atlas_lead".to_string(),
            subject_entity_id: uuid::Uuid::new_v4(),
            composite_score: None,
            confidence_level: "low".to_string(),
            total_contributors: 0,
            total_sessions: 0,
            total_entries: 5,
            dimension_vector: None,
            dimension_vector_v2: None,
            has_data_mask: None,
            last_computed_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        let typed = sc.subject_entity_type_typed().unwrap();
        assert_eq!(typed, ScorecardEntityType::AtlasLead);
    }

    // ── Entity helper: atlas_lead typed is_terminal ───────────────────────────
    //
    // These tests ensure the refactored is_terminal() still passes all the same
    // assertions as the original string-comparison implementation.

    fn make_lead_for_type_tests(status: &str) -> crate::entities::atlas_lead::Model {
        use chrono::Utc;
        use uuid::Uuid;
        crate::entities::atlas_lead::Model {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            first_name: None,
            middle_name: None,
            last_name: None,
            name: "Type Test Lead".to_owned(),
            title: None,
            email: None,
            email_verified: false,
            phone: None,
            phone_verified: false,
            fax: None,
            whatsapp: None,
            telegram: None,
            linkedin_url: None,
            twitter: None,
            instagram: None,
            facebook: None,
            avatar_url: None,
            company: None,
            company_dba: None,
            company_website: None,
            domain: None,
            industry: None,
            sub_industry: None,
            num_employees: None,
            annual_revenue: None,
            company_type: None,
            location_type: None,
            year_established: None,
            sic_code: None,
            naics_code: None,
            duns_number: None,
            credit_score_code: None,
            street_address: None,
            city: None,
            state: None,
            postal_code: None,
            country: "US".to_owned(),
            mailing_address: None,
            message: None,
            lead_status: status.to_owned(),
            source: None,
            data_source: None,
            data_source_id: None,
            lead_metadata: None,
            is_duplicate: false,
            duplicate_of_lead_id: None,
            listing_id: None,
            account_id: None,
            is_converted: false,
            converted_at: None,
            converted_account_id: None,
            converted_contact_id: None,
            converted_opportunity_id: None,
            disqualified_at: None,
            disqualification_reason: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn lead_model_is_terminal_delegates_to_lead_status_enum() {
        assert!(make_lead_for_type_tests("converted").is_terminal());
        assert!(make_lead_for_type_tests("disqualified").is_terminal());

        for status in &["new", "contacted", "qualifying", "qualified"] {
            assert!(
                !make_lead_for_type_tests(status).is_terminal(),
                "status '{}' should NOT be terminal",
                status
            );
        }
    }

    #[test]
    fn lead_model_lead_status_typed_returns_correct_enum() {
        let lead = make_lead_for_type_tests("qualifying");
        assert_eq!(lead.lead_status_typed().unwrap(), LeadStatus::Qualifying);
    }

    #[test]
    fn lead_model_lead_status_typed_err_for_unknown_value() {
        // Simulate a corrupt DB row with an unregistered status
        let lead = make_lead_for_type_tests("on_hold");
        assert!(lead.lead_status_typed().is_err());
    }

    #[test]
    fn lead_model_mailing_address_typed_returns_none_when_null() {
        let lead = make_lead_for_type_tests("new");
        let addr = lead.mailing_address_typed().unwrap();
        assert!(addr.is_none());
    }

    #[test]
    fn lead_model_mailing_address_typed_parses_valid_json() {
        use serde_json::json;

        let mut lead = make_lead_for_type_tests("new");
        lead.mailing_address = Some(json!({
            "street": "789 Elm St",
            "city": "Nashville",
            "state": "TN",
            "postal_code": "37201",
            "country": "US"
        }));

        let addr = lead.mailing_address_typed().unwrap().unwrap();
        assert_eq!(addr.city.as_deref(), Some("Nashville"));
        assert_eq!(addr.state.as_deref(), Some("TN"));
    }

    // ── NoteMetadata typed union ──────────────────────────────────────────────

    #[test]
    fn note_metadata_typed_parses_call_transcript() {
        use crate::types::note::NoteMetadata;
        use serde_json::json;

        let raw = json!({"url": "https://example.com/recording.mp3", "text": "Hello world"});
        let meta: NoteMetadata = serde_json::from_value(raw).unwrap();

        assert!(
            matches!(meta, NoteMetadata::CallTranscript(ref m) if m.url.contains("recording")),
            "Expected CallTranscript variant"
        );
    }

    #[test]
    fn note_metadata_typed_parses_rich_text() {
        use crate::types::note::NoteMetadata;
        use serde_json::json;

        let raw = json!({"delta": {"ops": [{"insert": "Hello\n"}]}});
        let meta: NoteMetadata = serde_json::from_value(raw).unwrap();

        assert!(matches!(meta, NoteMetadata::RichText(_)), "Expected RichText variant");
    }

    // ── ActivityMetadata typed union ──────────────────────────────────────────

    #[test]
    fn activity_metadata_typed_parses_email() {
        use crate::types::activity::ActivityMetadata;
        use serde_json::json;

        let raw = json!({"subject": "Follow up", "body_preview": "Hi there", "message_id": "abc123"});
        let meta: ActivityMetadata = serde_json::from_value(raw).unwrap();

        assert!(matches!(meta, ActivityMetadata::Email(_)), "Expected Email variant");
    }

    #[test]
    fn activity_metadata_typed_parses_call() {
        use crate::types::activity::ActivityMetadata;
        use serde_json::json;

        let raw = json!({"recording_url": "https://storage.example.com/call.mp3", "phone_number": "+1-555-1234"});
        let meta: ActivityMetadata = serde_json::from_value(raw).unwrap();

        assert!(matches!(meta, ActivityMetadata::Call(_)), "Expected Call variant");
    }
}

    // ── GTM types ─────────────────────────────────────────────────────────────

    use crate::types::gtm::{
        CopyStrategy, InjectAt, LaunchMode, LocalizationStatus, PixelType, PlanTier,
        ResolutionType,
    };

    #[test]
    fn launch_mode_display_roundtrip() {
        let variants = [
            (LaunchMode::Active,    "active"),
            (LaunchMode::Waitlist,  "waitlist"),
            (LaunchMode::PreOrder,  "pre_order"),
            (LaunchMode::PreLaunch, "pre_launch"),
            (LaunchMode::Draft,     "draft"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = LaunchMode::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn launch_mode_unknown_slug_returns_err() {
        assert!(LaunchMode::try_from("live".to_string()).is_err());
        assert!(LaunchMode::try_from("ACTIVE".to_string()).is_err(), "must be lowercase");
        assert!(LaunchMode::try_from("".to_string()).is_err());
    }

    #[test]
    fn launch_mode_allows_conversion() {
        assert!(LaunchMode::Active.allows_conversion());
        assert!(LaunchMode::Waitlist.allows_conversion());
        assert!(LaunchMode::PreOrder.allows_conversion());
        assert!(!LaunchMode::PreLaunch.allows_conversion());
        assert!(!LaunchMode::Draft.allows_conversion());
    }

    #[test]
    fn launch_mode_is_indexable() {
        assert!(LaunchMode::Active.is_indexable());
        assert!(LaunchMode::Waitlist.is_indexable());
        assert!(!LaunchMode::Draft.is_indexable());
        assert!(!LaunchMode::PreLaunch.is_indexable());
    }

    #[test]
    fn localization_status_display_roundtrip() {
        let variants = [
            (LocalizationStatus::Base,        "base"),
            (LocalizationStatus::AiLocalized, "ai_localized"),
            (LocalizationStatus::Manual,      "manual"),
            (LocalizationStatus::Pending,     "pending"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = LocalizationStatus::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn localization_status_unknown_slug_returns_err() {
        assert!(LocalizationStatus::try_from("ai".to_string()).is_err());
        assert!(LocalizationStatus::try_from("AI_LOCALIZED".to_string()).is_err());
    }

    #[test]
    fn copy_strategy_display_roundtrip() {
        let variants = [
            (CopyStrategy::Localized,   "localized"),
            (CopyStrategy::BaseCopy,    "base_copy"),
            (CopyStrategy::AiGenerated, "ai_generated"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = CopyStrategy::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn pixel_type_display_roundtrip() {
        let variants = [
            (PixelType::Gtm,      "gtm"),
            (PixelType::Ga4,      "ga4"),
            (PixelType::Meta,     "meta"),
            (PixelType::Linkedin, "linkedin"),
            (PixelType::Tiktok,   "tiktok"),
            (PixelType::Custom,   "custom"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = PixelType::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn pixel_type_needs_noscript_fallback() {
        assert!(PixelType::Meta.needs_noscript_fallback());
        assert!(PixelType::Linkedin.needs_noscript_fallback());
        assert!(!PixelType::Ga4.needs_noscript_fallback());
        assert!(!PixelType::Gtm.needs_noscript_fallback());
        assert!(!PixelType::Tiktok.needs_noscript_fallback());
        assert!(!PixelType::Custom.needs_noscript_fallback());
    }

    #[test]
    fn inject_at_display_roundtrip() {
        let variants = [
            (InjectAt::Head,      "head"),
            (InjectAt::BodyStart, "body_start"),
            (InjectAt::BodyEnd,   "body_end"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = InjectAt::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn plan_tier_display_roundtrip() {
        let variants = [
            (PlanTier::Starter,      "starter"),
            (PlanTier::Professional, "professional"),
            (PlanTier::Portfolio,    "portfolio"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = PlanTier::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn plan_tier_is_high_value() {
        assert!(PlanTier::Professional.is_high_value());
        assert!(PlanTier::Portfolio.is_high_value());
        assert!(!PlanTier::Starter.is_high_value());
    }

    #[test]
    fn resolution_type_display_roundtrip() {
        let variants = [
            (ResolutionType::Product,   "product"),
            (ResolutionType::Variant,   "variant"),
            (ResolutionType::TenantApp, "tenant_app"),
            (ResolutionType::NotFound,  "not_found"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = ResolutionType::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn pixel_type_unknown_slug_returns_err() {
        assert!(PixelType::try_from("facebook".to_string()).is_err());
        assert!(PixelType::try_from("GTM".to_string()).is_err(), "must be lowercase");
    }

    // ── AppId ──────────────────────────────────────────────────────────────────

    use crate::types::gtm::AppId;

    #[test]
    fn app_id_display_roundtrip() {
        let variants = [
            (AppId::PropertyManagement, "property_management"),
            (AppId::Anchor,             "anchor"),
            (AppId::NetworkInstance,    "network_instance"),
            (AppId::Meridian,           "meridian"),
            (AppId::CorePlatform,       "core_platform"),
        ];
        for (variant, slug) in &variants {
            assert_eq!(variant.to_string(), *slug, "Display mismatch for {:?}", variant);
            let parsed = AppId::try_from(slug.to_string()).expect("TryFrom should succeed");
            assert_eq!(&parsed, variant, "TryFrom roundtrip failed for '{}'", slug);
        }
    }

    #[test]
    fn app_id_unknown_slug_returns_err() {
        assert!(AppId::try_from("folio".to_string()).is_err(), "'folio' is a product slug, not an app_id");
        assert!(AppId::try_from("ANCHOR".to_string()).is_err(), "must be lowercase");
        assert!(AppId::try_from("".to_string()).is_err());
    }

    #[test]
    fn app_id_product_slug_mapping() {
        // Verifies the marketing slug → app_id relationship is correct.
        // If this mapping changes, update the migration backfill SQL too.
        assert_eq!(AppId::PropertyManagement.product_slug(), "folio");
        assert_eq!(AppId::Anchor.product_slug(),             "anchor");
        assert_eq!(AppId::NetworkInstance.product_slug(),    "network_instance");
        assert_eq!(AppId::Meridian.product_slug(),           "meridian");
    }

    #[test]
    fn app_id_all_db_values_matches_variants() {
        // Every variant must appear in all_db_values() — catch future drift.
        let all = AppId::all_db_values();
        for slug in all {
            assert!(
                AppId::try_from(*slug).is_ok(),
                "all_db_values() contains '{}' but TryFrom doesn't recognise it", slug
            );
        }
        assert_eq!(all.len(), 5, "Expected 5 AppId variants");
    }
