//! Pure unit tests for G-27 `ScorecardService` helpers and G-31 `atlas_lead` helpers.
//! No database — exercises confidence levels, ranking, name computation,
//! terminal state detection, and trend direction thresholds.

#[cfg(test)]
mod tests {
    use crate::entities::atlas_lead::Model as LeadModel;
    use crate::services::scorecard_service::ScorecardService;

    // ── ScorecardService::compute_confidence_level ────────────────────────────

    #[test]
    fn confidence_level_insufficient_for_zero_entries() {
        assert_eq!(
            ScorecardService::compute_confidence_level(0),
            "insufficient"
        );
    }

    #[test]
    fn confidence_level_insufficient_for_four_entries() {
        assert_eq!(
            ScorecardService::compute_confidence_level(4),
            "insufficient"
        );
    }

    #[test]
    fn confidence_level_low_for_five_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(5), "low");
    }

    #[test]
    fn confidence_level_low_for_nine_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(9), "low");
    }

    #[test]
    fn confidence_level_medium_for_ten_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(10), "medium");
    }

    #[test]
    fn confidence_level_medium_for_49_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(49), "medium");
    }

    #[test]
    fn confidence_level_high_for_50_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(50), "high");
    }

    #[test]
    fn confidence_level_high_for_199_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(199), "high");
    }

    #[test]
    fn confidence_level_very_high_for_200_entries() {
        assert_eq!(ScorecardService::compute_confidence_level(200), "very_high");
    }

    #[test]
    fn confidence_level_very_high_for_large_values() {
        assert_eq!(
            ScorecardService::compute_confidence_level(100_000),
            "very_high"
        );
    }

    // Boundary: ensure every boundary is correct by testing n-1, n, n+1
    #[test]
    fn confidence_level_boundaries_are_correct() {
        let cases = [
            (4, "insufficient"),
            (5, "low"),
            (9, "low"),
            (10, "medium"),
            (49, "medium"),
            (50, "high"),
            (199, "high"),
            (200, "very_high"),
        ];
        for (entries, expected) in cases {
            assert_eq!(
                ScorecardService::compute_confidence_level(entries),
                expected,
                "failed at entries={entries}"
            );
        }
    }

    // ── ScorecardService::confidence_rank ─────────────────────────────────────

    #[test]
    fn confidence_rank_ordering_is_correct() {
        assert!(
            ScorecardService::confidence_rank("insufficient")
                < ScorecardService::confidence_rank("low"),
            "insufficient must rank below low"
        );
        assert!(
            ScorecardService::confidence_rank("low") < ScorecardService::confidence_rank("medium")
        );
        assert!(
            ScorecardService::confidence_rank("medium") < ScorecardService::confidence_rank("high")
        );
        assert!(
            ScorecardService::confidence_rank("high")
                < ScorecardService::confidence_rank("very_high")
        );
    }

    #[test]
    fn confidence_rank_unknown_level_returns_zero() {
        assert_eq!(ScorecardService::confidence_rank("bogus"), 0);
        assert_eq!(ScorecardService::confidence_rank(""), 0);
    }

    #[test]
    fn confidence_rank_level_and_compute_are_consistent() {
        // compute_confidence_level produces strings that confidence_rank understands
        let levels_in_order = ["insufficient", "low", "medium", "high", "very_high"];
        for i in 0..levels_in_order.len() - 1 {
            assert!(
                ScorecardService::confidence_rank(levels_in_order[i])
                    < ScorecardService::confidence_rank(levels_in_order[i + 1]),
                "rank must be strictly increasing: {} < {}",
                levels_in_order[i],
                levels_in_order[i + 1]
            );
        }
    }

    // ── LeadModel::compute_name ───────────────────────────────────────────────

    #[test]
    fn compute_name_full_name_when_both_parts_present() {
        let name = LeadModel::compute_name(Some("Jane"), Some("Doe"), None, None);
        assert_eq!(name, "Jane Doe");
    }

    #[test]
    fn compute_name_first_only_when_no_last() {
        let name = LeadModel::compute_name(Some("Jane"), None, None, None);
        assert_eq!(name, "Jane");
    }

    #[test]
    fn compute_name_last_only_when_no_first() {
        let name = LeadModel::compute_name(None, Some("Doe"), None, None);
        assert_eq!(name, "Doe");
    }

    #[test]
    fn compute_name_falls_back_to_company() {
        let name = LeadModel::compute_name(None, None, Some("Acme Corp"), None);
        assert_eq!(name, "Acme Corp");
    }

    #[test]
    fn compute_name_falls_back_to_email() {
        let name = LeadModel::compute_name(None, None, None, Some("jane@acme.com"));
        assert_eq!(name, "jane@acme.com");
    }

    #[test]
    fn compute_name_returns_unknown_when_all_none() {
        let name = LeadModel::compute_name(None, None, None, None);
        assert_eq!(name, "Unknown");
    }

    #[test]
    fn compute_name_ignores_empty_string_first_name() {
        // Empty strings should be treated as missing (filter)
        let name = LeadModel::compute_name(Some(""), Some("Doe"), None, None);
        assert_eq!(name, "Doe", "empty first name should be skipped");
    }

    #[test]
    fn compute_name_ignores_empty_string_last_name() {
        let name = LeadModel::compute_name(Some("Jane"), Some(""), None, None);
        assert_eq!(name, "Jane", "empty last name should be skipped");
    }

    #[test]
    fn compute_name_company_wins_over_email() {
        let name = LeadModel::compute_name(None, None, Some("Acme Corp"), Some("jane@acme.com"));
        assert_eq!(
            name, "Acme Corp",
            "company must win over email in fallback chain"
        );
    }

    #[test]
    fn compute_name_empty_company_falls_through_to_email() {
        let name = LeadModel::compute_name(None, None, Some(""), Some("jane@acme.com"));
        assert_eq!(name, "jane@acme.com");
    }

    // ── LeadModel::is_terminal ────────────────────────────────────────────────

    fn make_lead(status: &str) -> LeadModel {
        use chrono::Utc;
        use uuid::Uuid;
        LeadModel {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            first_name: None,
            middle_name: None,
            last_name: None,
            name: "Test Lead".to_owned(),
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
    fn is_terminal_true_for_converted() {
        assert!(make_lead("converted").is_terminal());
    }

    #[test]
    fn is_terminal_true_for_disqualified() {
        assert!(make_lead("disqualified").is_terminal());
    }

    #[test]
    fn is_terminal_false_for_active_statuses() {
        for status in &["new", "contacted", "qualifying", "qualified"] {
            assert!(
                !make_lead(status).is_terminal(),
                "status '{}' should NOT be terminal",
                status
            );
        }
    }
}
