//! Folio — Fair Housing Filter
//!
//! FHA-compliant applicant profile sanitizer for US/USVI applicants.
//! Non-removable service-layer invariant — not a feature flag.
//!
//! The hardcoded `PROTECTED_FILTER_KEYS` const that existed here has been
//! removed. The service now accepts `&dyn AntiDiscriminationLaw` and derives
//! both its query-validation list and profile-strip list directly from
//! `law.protected_fields()` — the canonical `FhaProtectedField` table in
//! `miami.rs`. One source of truth, no drift.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::types::pm::Jurisdiction;
use crate::services::pm::market::market_config::AntiDiscriminationLaw;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicantProfile {
    pub applicant_id: Uuid,
    pub tenant_id: Uuid,
    pub jurisdiction: Jurisdiction,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    // Protected fields — stripped for US/VI applicants
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub gender: Option<String>,
    pub race_ethnicity: Option<String>,
    pub national_origin: Option<String>,
    pub religion: Option<String>,
    pub disability_status: Option<String>,
    pub familial_status: Option<String>,
    // Financial screening — always retained
    pub annual_income_cents: Option<i64>,
    pub credit_score: Option<i32>,
    pub employment_status: Option<String>,
    pub employer_name: Option<String>,
    pub months_employed: Option<i32>,
    // BR-specific
    pub cpf: Option<String>,
    pub rg: Option<String>,
    pub serasa_score: Option<i32>,
    // ID document
    pub ssn_last4: Option<String>,
    pub government_id_type: Option<String>,
}

/// Post-filter profile: protected fields are structurally absent for US/VI applicants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedApplicantProfile {
    pub applicant_id: Uuid,
    pub tenant_id: Uuid,
    pub jurisdiction: Jurisdiction,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub annual_income_cents: Option<i64>,
    pub credit_score: Option<i32>,
    pub employment_status: Option<String>,
    pub employer_name: Option<String>,
    pub months_employed: Option<i32>,
    pub cpf: Option<String>,
    pub rg: Option<String>,
    pub serasa_score: Option<i32>,
    pub ssn_last4: Option<String>,
    pub government_id_type: Option<String>,
    /// True if FHA stripping was applied (US/VI jurisdiction).
    pub fha_filter_applied: bool,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct FairHousingFilter;

impl FairHousingFilter {
    /// Strip FHA-protected characteristics from a US/VI applicant profile.
    ///
    /// The `Jurisdiction` enum drives this check — no raw string comparison.
    /// Non-US jurisdictions (BR, DO, HT) pass through unmodified.
    pub fn sanitize(profile: ApplicantProfile) -> SanitizedApplicantProfile {
        let fha_applies = profile.jurisdiction.fha_applies();

        SanitizedApplicantProfile {
            applicant_id: profile.applicant_id,
            tenant_id: profile.tenant_id,
            jurisdiction: profile.jurisdiction,
            first_name: profile.first_name,
            last_name: profile.last_name,
            email: profile.email,
            phone: profile.phone,
            annual_income_cents: profile.annual_income_cents,
            credit_score: profile.credit_score,
            employment_status: profile.employment_status,
            employer_name: profile.employer_name,
            months_employed: profile.months_employed,
            cpf: profile.cpf,
            rg: profile.rg,
            serasa_score: profile.serasa_score,
            ssn_last4: profile.ssn_last4,
            government_id_type: profile.government_id_type,
            fha_filter_applied: fha_applies,
            // Protected fields are structurally absent from SanitizedApplicantProfile.
        }
    }

    /// Validate that a query filter object contains no protected characteristic filters.
    ///
    /// Derives the prohibited key list from `law.protected_fields()` — no
    /// hardcoded constant in this file. Checks both canonical `field_name` and
    /// any `also_aliases` for each protected class.
    ///
    /// Returns a list of human-readable violations. Non-US jurisdictions always
    /// return empty (no law provided = no restrictions).
    pub fn validate_query_filters(
        jurisdiction: &Jurisdiction,
        law: Option<&dyn AntiDiscriminationLaw>,
        filters: &serde_json::Value,
    ) -> Vec<String> {
        let law = match law {
            None => return vec![],
            Some(l) if !jurisdiction.fha_applies() => return vec![],
            Some(l) => l,
        };

        let mut violations = Vec::new();
        if let Some(obj) = filters.as_object() {
            for field in law.protected_fields() {
                // Check canonical name and all aliases
                let hits = std::iter::once(field.field_name)
                    .chain(field.also_aliases.iter().copied())
                    .filter(|key| obj.contains_key(*key))
                    .collect::<Vec<_>>();

                for key in hits {
                    violations.push(format!(
                        "Filter on '{}' is prohibited under {} ({} — {})",
                        key,
                        field.protected_class,
                        field.legal_basis,
                        law.statute().code,
                    ));
                }
            }
        }
        violations
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::pm::market::miami::FairHousingAct;

    fn make_profile(j: Jurisdiction) -> ApplicantProfile {
        ApplicantProfile {
            applicant_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            jurisdiction: j,
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: Some("jane@example.com".to_string()),
            phone: None,
            date_of_birth: Some(chrono::NaiveDate::from_ymd_opt(1985, 3, 15).unwrap()),
            gender: Some("F".to_string()),
            race_ethnicity: Some("Hispanic".to_string()),
            national_origin: Some("Cuba".to_string()),
            religion: None,
            disability_status: None,
            familial_status: Some("married_with_children".to_string()),
            annual_income_cents: Some(85_000_00),
            credit_score: Some(720),
            employment_status: Some("employed".to_string()),
            employer_name: None,
            months_employed: Some(36),
            cpf: None,
            rg: None,
            serasa_score: None,
            ssn_last4: Some("4242".to_string()),
            government_id_type: Some("ssn".to_string()),
        }
    }

    #[test]
    fn test_us_applicant_fha_applied() {
        let sanitized = FairHousingFilter::sanitize(make_profile(Jurisdiction::Us));
        assert!(sanitized.fha_filter_applied);
        assert_eq!(sanitized.credit_score, Some(720));
    }

    #[test]
    fn test_usvi_applicant_fha_applied() {
        let sanitized = FairHousingFilter::sanitize(make_profile(Jurisdiction::Vi));
        assert!(sanitized.fha_filter_applied);
    }

    #[test]
    fn test_brazil_not_filtered() {
        let sanitized = FairHousingFilter::sanitize(make_profile(Jurisdiction::Br));
        assert!(!sanitized.fha_filter_applied);
    }

    #[test]
    fn test_filter_validates_gender_for_us() {
        use serde_json::json;
        let fha = FairHousingAct;
        let violations = FairHousingFilter::validate_query_filters(
            &Jurisdiction::Us,
            Some(&fha),
            &json!({ "gender": "F", "credit_score_min": 650 }),
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("gender"));
        assert!(violations[0].contains("42 U.S.C. § 3604(b)"));
    }

    #[test]
    fn test_filter_catches_alias_dob() {
        use serde_json::json;
        let fha = FairHousingAct;
        // "dob" is an alias for date_of_birth — must be caught
        let violations = FairHousingFilter::validate_query_filters(
            &Jurisdiction::Us,
            Some(&fha),
            &json!({ "dob": "1985-03-15" }),
        );
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("dob"));
    }

    #[test]
    fn test_filter_no_violations_for_br() {
        use serde_json::json;
        let fha = FairHousingAct;
        // FHA doesn't apply to BR even if FairHousingAct is passed
        let violations = FairHousingFilter::validate_query_filters(
            &Jurisdiction::Br,
            Some(&fha),
            &json!({ "gender": "F" }),
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_filter_no_law_returns_empty() {
        use serde_json::json;
        let violations = FairHousingFilter::validate_query_filters(
            &Jurisdiction::Us,
            None,
            &json!({ "race": "any" }),
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_filter_permissible_fields_pass() {
        use serde_json::json;
        let fha = FairHousingAct;
        let violations = FairHousingFilter::validate_query_filters(
            &Jurisdiction::Us,
            Some(&fha),
            &json!({ "credit_score_min": 650, "income_min": 50_000 }),
        );
        assert!(violations.is_empty());
    }

    #[test]
    fn test_every_fha_field_has_legal_basis() {
        let fha = FairHousingAct;
        for field in fha.protected_fields() {
            assert!(
                !field.legal_basis.is_empty(),
                "FhaProtectedField '{}' is missing a legal_basis",
                field.field_name
            );
        }
    }
}
