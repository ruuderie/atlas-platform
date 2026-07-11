//! Miami-Dade market configuration.
//!
//! FHA applies. TDT 7%. STR Ordinance 2023-89.

use crate::services::pm::market::market_config::{
    AntiDiscriminationLaw, CreditBureau, FhaProtectedField, MarketConfig, StrRegulation, TaxEngine,
    TenancyLaw,
};
use crate::types::pm::{
    CreditIdField, Currency, GuaranteeType, Jurisdiction, StatuteRef, StrPermitCategory,
    SubJurisdiction, TaxRate,
};
use rust_decimal::Decimal;
use std::str::FromStr;

// ── Law: Fair Housing Act ─────────────────────────────────────────────────────

pub struct FairHousingAct;

/// Canonical FHA protected characteristics table.
///
/// Replaces the two parallel arrays (`FHA_PROHIBITED_FILTER_KEYS` +
/// `FHA_STRIP_FROM_PROFILE`) that existed before. Each row captures:
///   - the canonical field name
///   - which protected class it belongs to (42 U.S.C. § 3604)
///   - the specific FHA section
///   - whether it must be stripped from profiles returned to landlords
///   - common query-parameter aliases that also trigger a violation
///
/// `FairHousingFilter` iterates this single table for both query validation
/// and profile sanitization — no parallel arrays that can drift out of sync.
static FHA_PROTECTED_FIELDS: &[FhaProtectedField] = &[
    FhaProtectedField {
        field_name: "date_of_birth",
        protected_class: "Familial Status / Age",
        legal_basis: "42 U.S.C. § 3604(b)",
        strip_from_profile: true,
        also_aliases: &["dob"],
    },
    FhaProtectedField {
        field_name: "gender",
        protected_class: "Sex",
        legal_basis: "42 U.S.C. § 3604(b)",
        strip_from_profile: true,
        also_aliases: &["sex"],
    },
    FhaProtectedField {
        field_name: "race_ethnicity",
        protected_class: "Race / Color / National Origin",
        legal_basis: "42 U.S.C. § 3604(a)",
        strip_from_profile: true,
        also_aliases: &["race", "ethnicity"],
    },
    FhaProtectedField {
        field_name: "national_origin",
        protected_class: "National Origin",
        legal_basis: "42 U.S.C. § 3604(b)",
        strip_from_profile: true,
        also_aliases: &[],
    },
    FhaProtectedField {
        field_name: "religion",
        protected_class: "Religion",
        legal_basis: "42 U.S.C. § 3604(b)",
        strip_from_profile: true,
        also_aliases: &[],
    },
    FhaProtectedField {
        field_name: "disability_status",
        protected_class: "Handicap / Disability",
        legal_basis: "42 U.S.C. § 3604(f)",
        strip_from_profile: true,
        also_aliases: &["disability"],
    },
    FhaProtectedField {
        field_name: "familial_status",
        protected_class: "Familial Status",
        legal_basis: "42 U.S.C. § 3604(b)",
        strip_from_profile: true,
        also_aliases: &["family_status"],
    },
];

impl AntiDiscriminationLaw for FairHousingAct {
    fn statute(&self) -> StatuteRef {
        StatuteRef {
            code: "42 U.S.C. § 3604",
            name: "Fair Housing Act",
            country: Jurisdiction::Us,
            url: Some(
                "https://www.hud.gov/program_offices/fair_housing_equal_opp/fair_housing_act_overview",
            ),
        }
    }

    fn protected_fields(&self) -> &'static [FhaProtectedField] {
        FHA_PROTECTED_FIELDS
    }
}

// ── STR: Miami-Dade Ordinance 2023-89 ────────────────────────────────────────

pub struct MiamiDadeStrOrdinance;

static MIAMI_PERMIT_CATEGORIES: &[StrPermitCategory] = &[
    StrPermitCategory::PrincipalResidence,
    StrPermitCategory::Hosted,
    StrPermitCategory::NonHosted,
];

impl StrRegulation for MiamiDadeStrOrdinance {
    fn sub_jurisdiction(&self) -> SubJurisdiction {
        SubJurisdiction {
            country: Jurisdiction::Us,
            state: Some("FL"),
            county_city: Some("MIAMI-DADE"),
        }
    }

    fn permit_categories(&self) -> &'static [StrPermitCategory] {
        MIAMI_PERMIT_CATEGORIES
    }

    fn owner_must_be_present(&self, category: &StrPermitCategory) -> bool {
        matches!(category, StrPermitCategory::Hosted)
    }

    fn max_rental_days_per_year(&self, category: &StrPermitCategory) -> Option<u32> {
        match category {
            StrPermitCategory::PrincipalResidence => Some(120),
            StrPermitCategory::Hosted => None,
            StrPermitCategory::NonHosted => Some(90),
        }
    }

    fn expiry_warning_days(&self) -> u32 {
        30
    }
}

// ── Tax: Florida TDT ──────────────────────────────────────────────────────────

pub struct FloridaTdt;

impl TaxEngine for FloridaTdt {
    fn str_tax_rate(&self) -> Option<TaxRate> {
        Some(TaxRate {
            rate: Decimal::from_str("0.07").unwrap(),
            remittance_currency: Currency::Usd,
            ota_collects: true,
            label: "TDT — Miami-Dade Tourist Development Tax (7%)",
        })
    }

    fn rental_withholding_rate(&self) -> Option<TaxRate> {
        None
    }
    fn remittance_currency(&self) -> Currency {
        Currency::Usd
    }
}

// ── Credit: TransUnion ────────────────────────────────────────────────────────

/// Re-used as-is by `UsViMarket`.
pub struct TransUnionBureau;

impl CreditBureau for TransUnionBureau {
    fn name(&self) -> &'static str {
        "TransUnion"
    }
    fn applicant_id_field(&self) -> CreditIdField {
        CreditIdField::SsnLast4
    }
    fn minimum_score_auto_approve(&self) -> Option<i32> {
        Some(650)
    }
}

// ── Tenancy Law: Florida § 83 ─────────────────────────────────────────────────

pub struct FloridaTenancyLaw;

static US_GUARANTEES: &[GuaranteeType] = &[GuaranteeType::Caucao, GuaranteeType::None];

impl TenancyLaw for FloridaTenancyLaw {
    fn statute(&self) -> StatuteRef {
        StatuteRef {
            code: "Fla. Stat. § 83",
            name: "Florida Residential Landlord and Tenant Act",
            country: Jurisdiction::Us,
            url: Some(
                "http://www.leg.state.fl.us/statutes/index.cfm?App_mode=Display_Statute&URL=0000-0099/0083/0083.html",
            ),
        }
    }

    fn max_deposit_months(&self) -> Option<u8> {
        None
    }
    fn landlord_termination_notice_days(&self) -> u32 {
        15
    }
    fn tenant_vacate_notice_days(&self) -> u32 {
        30
    }
    fn allowed_guarantee_types(&self) -> &'static [GuaranteeType] {
        US_GUARANTEES
    }
}

// ── Root: MiamiDadeMarket ─────────────────────────────────────────────────────

pub struct MiamiDadeMarket;

impl MarketConfig for MiamiDadeMarket {
    fn jurisdiction(&self) -> Jurisdiction {
        Jurisdiction::Us
    }
    fn default_currency(&self) -> Currency {
        Currency::Usd
    }
    fn display_name(&self) -> &'static str {
        "Miami-Dade, FL (USA)"
    }

    fn tenancy_law(&self) -> Option<&dyn TenancyLaw> {
        Some(&FloridaTenancyLaw)
    }
    fn anti_discrimination_law(&self) -> Option<&dyn AntiDiscriminationLaw> {
        Some(&FairHousingAct)
    }
    fn str_regulation(&self) -> Option<&dyn StrRegulation> {
        Some(&MiamiDadeStrOrdinance)
    }
    fn tax_engine(&self) -> &dyn TaxEngine {
        &FloridaTdt
    }
    fn credit_bureau(&self) -> &dyn CreditBureau {
        &TransUnionBureau
    }
}
