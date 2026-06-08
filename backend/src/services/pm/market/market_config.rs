//! Folio — Market Configuration Trait System
//!
//! # Design Rationale
//!
//! Each market Folio operates in has distinct behaviour across five regulatory
//! axes. That behaviour lives in **one file per market**, not scattered across
//! service `match` statements.
//!
//! ## The Condomínio Resolution Chain
//!
//! The condomínio expense classifier used to live in `ConominioService` as a
//! hardcoded Brazilian keyword list. With `MarketConfig`, the call chain is:
//!
//! ```text
//! Handler
//!   → registry.resolve(&tenant_jurisdiction)   // &dyn MarketConfig
//!   → market.tenancy_law()                     // Option<&dyn TenancyLaw>
//!   → law.classify_expense("Fundo de reserva") // Option<ConominioExpenseCategory>
//! ```
//!
//! `ConominioService::classify_and_split` now accepts `&dyn TenancyLaw` and
//! delegates to it. The keyword lists live in `markets/brazil.rs::LeiDoInquilinato`,
//! not in the service. A Florida lease handler calling `classify_expense` on
//! `FloridaTenancyLaw` always gets `None` — correctly — because Florida has no
//! condomínio system. No `if jurisdiction == "BR"` needed anywhere.
//!
//! ## Why all returns are typed
//!
//! Previous version returned `&'static str` for currency, credit ID field,
//! jurisdiction label, and statute name. Each of those is a finite set with
//! known semantics — they are enums or structs, not strings.
//!
//! | Old return         | New return             | Why                              |
//! |--------------------|------------------------|----------------------------------|
//! | `&'static str`     | `Currency`             | DB boundary, payment rails       |
//! | `&'static str`     | `CreditIdField`        | Determines field read off profile |
//! | `&'static str`     | `SubJurisdiction`      | Structured label with `.label()` |
//! | `Decimal` + label  | `TaxRate`              | Carries OTA flag + compute method |
//! | `&'static str`     | `StatuteRef`           | Typed for audit logging + URL    |

use crate::types::pm::{
    Jurisdiction, Currency, CreditIdField, SubJurisdiction, TaxRate, StatuteRef,
    GuaranteeType, StrPermitCategory, ConominioExpenseCategory,
};

// ══════════════════════════════════════════════════════════════════════════════
// FhaProtectedField — shared type used by AntiDiscriminationLaw implementors
// ══════════════════════════════════════════════════════════════════════════════

/// A single FHA protected characteristic field definition.
///
/// # Why a struct, not two parallel &[&str] arrays
///
/// The old design had `prohibited_filter_keys: &[&str]` and
/// `strip_from_profile: &[&str]` as two separate arrays.
/// This meant:
///   - The same field name appeared in up to two places
///   - No indication of *why* a field is protected (which class, which statute)
///   - No way to test a single field's classification in isolation
///
/// `FhaProtectedField` co-locates:
///   - `field_name`: the struct/query key to check
///   - `protected_class`: human-readable class name for violation messages
///   - `legal_basis`: specific FHA section citation
///   - `strip_from_profile`: whether to omit from `SanitizedApplicantProfile`
///   - `also_aliases`: alternate query param names for the same class (e.g. "dob" for "date_of_birth")
#[derive(Debug, Clone, Copy)]
pub struct FhaProtectedField {
    /// The canonical field/query key name.
    pub field_name: &'static str,
    /// Plain-language protected class name.
    pub protected_class: &'static str,
    /// FHA statutory citation.
    pub legal_basis: &'static str,
    /// If true, this field is stripped from profiles returned to landlords.
    pub strip_from_profile: bool,
    /// Additional query-parameter aliases for this field (e.g. "dob", "sex").
    pub also_aliases: &'static [&'static str],
}
use std::fmt;

// ══════════════════════════════════════════════════════════════════════════════
// Sub-trait: TenancyLaw
// ══════════════════════════════════════════════════════════════════════════════

/// Behaviour governed by the local residential tenancy statute.
///
/// # Condomínio resolution
///
/// `classify_expense` is the method `ConominioService` delegates to.
/// For `LeiDoInquilinato` (BR) it returns the expense obligation.
/// For `FloridaTenancyLaw` (US) it always returns `None` — Florida has no
/// condomínio system. The service handles `None` as "no split applicable".
///
/// Implementors: `LeiDoInquilinato` (BR), `FloridaTenancyLaw` (US), `UsviTenancyLaw` (VI)
pub trait TenancyLaw: Send + Sync {
    /// Typed reference to the governing statute — used in audit logs and UI.
    fn statute(&self) -> StatuteRef;

    /// Classify a building expense description into tenant vs. landlord obligation.
    ///
    /// Returns `None` if:
    ///   (a) The description cannot be auto-classified (manual review needed), or
    ///   (b) This jurisdiction has no expense-classification system (e.g. Florida).
    ///
    /// **Default implementation returns `None`** — markets that do not have a
    /// condomínio-style split system inherit this and never incorrectly classify.
    fn classify_expense(&self, _description: &str) -> Option<ConominioExpenseCategory> {
        None
    }

    /// Default expense obligation when `classify_expense` returns `None`
    /// and the caller needs a fallback.
    ///
    /// Conservative: `DespesasOrdinarias` (charge tenant) only when the market's
    /// tenancy law says tenant bears unclassified expenses. Override to
    /// `DespesasExtraordinarias` if your jurisdiction defaults the other way.
    fn unclassified_expense_default(&self) -> Option<ConominioExpenseCategory> {
        None // No default — caller decides; safer than assuming
    }

    /// Maximum security deposit permitted, expressed as months of rent.
    /// `None` = no statutory cap.
    fn max_deposit_months(&self) -> Option<u8>;

    /// Minimum notice days the landlord must give before terminating.
    fn landlord_termination_notice_days(&self) -> u32;

    /// Minimum notice days the tenant must give to vacate.
    fn tenant_vacate_notice_days(&self) -> u32;

    /// Legal guarantee mechanisms available in this jurisdiction.
    fn allowed_guarantee_types(&self) -> &'static [GuaranteeType];
}

// ══════════════════════════════════════════════════════════════════════════════
// Sub-trait: AntiDiscriminationLaw
// ══════════════════════════════════════════════════════════════════════════════

/// Behaviour governed by housing anti-discrimination law.
///
/// # Single source of truth
///
/// Implementors expose one method — `protected_fields()` — which returns
/// the canonical `FhaProtectedField` table. The `FairHousingFilter` service
/// derives both its query-filter validation list and its profile-strip list
/// directly from this table.
///
/// Old design had `prohibited_filter_keys()` + `strip_from_profile()` as
/// two separate `&'static [&'static str]` methods that could drift out of sync.
///
/// Implementors: `FairHousingAct` (US + VI), future BR equivalent.
pub trait AntiDiscriminationLaw: Send + Sync {
    /// Typed statute reference.
    fn statute(&self) -> StatuteRef;

    /// The canonical protected field table for this jurisdiction.
    ///
    /// `FairHousingFilter` derives all query validation and profile sanitization
    /// by iterating this slice — no separate lists to maintain.
    fn protected_fields(&self) -> &'static [FhaProtectedField];
}

// ══════════════════════════════════════════════════════════════════════════════
// Sub-trait: StrRegulation
// ══════════════════════════════════════════════════════════════════════════════

/// Behaviour governed by local short-term rental ordinances.
///
/// Implementors: `MiamiDadeStrOrdinance`.
pub trait StrRegulation: Send + Sync {
    /// Typed sub-jurisdiction for compliance record storage.
    /// e.g. `SubJurisdiction { country: Us, state: "FL", county_city: "MIAMI-DADE" }`
    fn sub_jurisdiction(&self) -> SubJurisdiction;

    /// Permit categories available in this jurisdiction.
    fn permit_categories(&self) -> &'static [StrPermitCategory];

    /// Whether the owner must be present during guest stays for this category.
    fn owner_must_be_present(&self, category: &StrPermitCategory) -> bool;

    /// Maximum rental days per calendar year for this category.
    /// `None` = no cap.
    fn max_rental_days_per_year(&self, category: &StrPermitCategory) -> Option<u32>;

    /// Days before permit expiry to open a `compliance_violation` case.
    fn expiry_warning_days(&self) -> u32;
}

// ══════════════════════════════════════════════════════════════════════════════
// Sub-trait: TaxEngine
// ══════════════════════════════════════════════════════════════════════════════

/// Tax calculation rules for rental income.
///
/// Returns `TaxRate` structs — typed, not raw `Decimal` + `&'static str`.
/// `TaxRate::apply_to_cents(gross)` computes the tax amount inline.
///
/// Implementors: `FloridaTdt` (US), `UsviHotelRoomTax` (VI), `BrazilIrrf` (BR).
pub trait TaxEngine: Send + Sync {
    /// STR / transient accommodation tax rate.
    /// `None` = no STR-specific tax in this market.
    fn str_tax_rate(&self) -> Option<TaxRate>;

    /// Rental income withholding rate.
    /// `None` = not applicable.
    fn rental_withholding_rate(&self) -> Option<TaxRate>;

    /// The currency in which tax is remitted to the authority.
    fn remittance_currency(&self) -> Currency;
}

// ══════════════════════════════════════════════════════════════════════════════
// Sub-trait: CreditBureau
// ══════════════════════════════════════════════════════════════════════════════

/// Credit screening bureau configuration.
///
/// `applicant_id_field()` returns `CreditIdField` — not a string.
/// The service reads the correct field off `ApplicantProfile` via the enum:
///
/// ```rust,ignore
/// let id_value = match market.credit_bureau().applicant_id_field() {
///     CreditIdField::Cpf      => profile.cpf.as_deref(),
///     CreditIdField::SsnLast4 => profile.ssn_last4.as_deref(),
///     CreditIdField::Cnpj     => profile.cnpj.as_deref(),
/// };
/// ```
///
/// Implementors: `SerasaBureau` (BR), `TransUnionBureau` (US/VI).
pub trait CreditBureau: Send + Sync {
    /// Bureau display name.
    fn name(&self) -> &'static str;

    /// Typed identity field — determines which profile field is used for lookup.
    fn applicant_id_field(&self) -> CreditIdField;

    /// Minimum acceptable score for auto-approval.
    /// `None` = always require manual review.
    fn minimum_score_auto_approve(&self) -> Option<i32>;
}

// ══════════════════════════════════════════════════════════════════════════════
// Root trait: MarketConfig
// ══════════════════════════════════════════════════════════════════════════════

/// The root market configuration trait.
///
/// Each market that Folio operates in implements this trait exactly once.
/// Services accept `&dyn MarketConfig` — they are never aware of the concrete type.
///
/// # Typed returns — no &'static str
///
/// | Method                    | Return type       | Not                  |
/// |---------------------------|-------------------|----------------------|
/// | `default_currency()`      | `Currency`        | `&'static str`       |
/// | `tenancy_law()`           | `Option<&dyn TenancyLaw>` | `Option<bool>` |
/// | `tax_engine().str_tax_rate()` | `Option<TaxRate>` | `Option<Decimal>` |
/// | `credit_bureau().applicant_id_field()` | `CreditIdField` | `&'static str` |
/// | `str_regulation().sub_jurisdiction()` | `SubJurisdiction` | `&'static str` |
///
/// # Adding a new market
///
/// 1. `backend/src/markets/{country}.rs` — implement `MarketConfig` + sub-traits
/// 2. `markets/mod.rs` — `pub mod {country};`
/// 3. `MarketRegistry::build()` — push `Box::new({Country}Market)`
/// 4. Nothing else changes. No service edits required.
pub trait MarketConfig: Send + Sync {
    fn jurisdiction(&self) -> Jurisdiction;
    fn default_currency(&self) -> Currency;
    fn display_name(&self) -> &'static str;

    fn tenancy_law(&self) -> Option<&dyn TenancyLaw> { None }
    fn anti_discrimination_law(&self) -> Option<&dyn AntiDiscriminationLaw> { None }
    fn str_regulation(&self) -> Option<&dyn StrRegulation> { None }

    fn tax_engine(&self) -> &dyn TaxEngine;
    fn credit_bureau(&self) -> &dyn CreditBureau;
}

// ══════════════════════════════════════════════════════════════════════════════
// MarketRegistry
// ══════════════════════════════════════════════════════════════════════════════

pub struct MarketRegistry {
    markets: Vec<Box<dyn MarketConfig>>,
}

impl MarketRegistry {
    pub fn build() -> Self {
        use crate::services::pm::market::{brazil::BrazilMarket, miami::MiamiDadeMarket, usvi::UsViMarket};
        Self {
            markets: vec![
                Box::new(BrazilMarket),
                Box::new(MiamiDadeMarket),
                Box::new(UsViMarket),
            ],
        }
    }

    pub fn resolve(&self, jurisdiction: &Jurisdiction) -> anyhow::Result<&dyn MarketConfig> {
        self.markets
            .iter()
            .find(|m| &m.jurisdiction() == jurisdiction)
            .map(|m| m.as_ref())
            .ok_or_else(|| anyhow::anyhow!(
                "No MarketConfig registered for {:?}. Add it to MarketRegistry::build().",
                jurisdiction
            ))
    }
}

impl fmt::Debug for MarketRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.markets.iter().map(|m| m.display_name()).collect();
        write!(f, "MarketRegistry {{ markets: {:?} }}", names)
    }
}
