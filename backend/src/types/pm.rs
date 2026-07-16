//! Canonical Rust types for the Folio Property Management App.
//!
//! # Rule — same as types/scorecard.rs
//! These enums are the **source of truth** for all PM domain discriminator strings.
//! DB VARCHAR columns are derived from them via `Display`. Never write raw string
//! literals like `"maintenance"` or `"atlas_asset"` in service code.
//!
//! # Boundary contract
//! - Entity models keep `String` at the SeaORM DB boundary.
//! - Services call `TryFrom<String>` immediately after reading from DB.
//! - Services call `.to_string()` (via `Display`) immediately before writing to DB.
//! - JSON serde is driven entirely by `#[serde(rename_all = "snake_case")]`.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Jurisdiction ──────────────────────────────────────────────────────────────

/// Operating jurisdiction for a Folio tenant.
///
/// Determines: applicable tenancy law, payment rails, tax calculation engine,
/// Fair Housing filtering (US/VI only), and STR compliance rules.
///
/// Stored as VARCHAR in `tenant_setting(key='folio_jurisdiction_code')`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Jurisdiction {
    /// United States (Florida, etc.)
    Us,
    /// Brazil (Lei do Inquilinato — Law 8.245/91)
    Br,
    /// Dominican Republic
    Do,
    /// Haiti
    Ht,
    /// US Virgin Islands (FHA applies, TDT separate from FL)
    Vi,
}

impl Jurisdiction {
    /// Returns true if the Fair Housing Act applies in this jurisdiction.
    pub fn fha_applies(&self) -> bool {
        matches!(self, Self::Us | Self::Vi)
    }

    /// Returns true if Lei do Inquilinato applies (Brazil tenancy law).
    pub fn lei_do_inquilinato_applies(&self) -> bool {
        matches!(self, Self::Br)
    }

    /// Returns the default currency code for this jurisdiction.
    pub fn default_currency(&self) -> &'static str {
        match self {
            Self::Us | Self::Vi => "USD",
            Self::Br => "BRL",
            Self::Do => "DOP",
            Self::Ht => "HTG",
        }
    }
}

impl fmt::Display for Jurisdiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Us => "US",
            Self::Br => "BR",
            Self::Do => "DO",
            Self::Ht => "HT",
            Self::Vi => "VI",
        })
    }
}

impl TryFrom<String> for Jurisdiction {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "US" => Ok(Self::Us),
            "BR" => Ok(Self::Br),
            "DO" => Ok(Self::Do),
            "HT" => Ok(Self::Ht),
            "VI" => Ok(Self::Vi),
            other => Err(format!("unknown Folio Jurisdiction: '{other}'")),
        }
    }
}

// ── Currency ──────────────────────────────────────────────────────────────────

/// ISO 4217 currency codes for the markets Folio operates in.
///
/// Used as the typed return of `MarketConfig::default_currency()` and
/// `TaxEngine::remittance_currency()` — replaces all `&'static str` currency fields.
///
/// Stored as VARCHAR in `atlas_contracts.currency_code`,
/// `atlas_ledger_entries.currency_code`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    /// US Dollar
    Usd,
    /// Brazilian Real
    Brl,
    /// Dominican Peso
    Dop,
    /// Haitian Gourde
    Htg,
}

impl Currency {
    /// Number of minor units (cents) per major unit.
    pub fn minor_units(&self) -> u32 {
        match self {
            Self::Usd | Self::Brl | Self::Dop | Self::Htg => 100,
        }
    }

    /// ISO 4217 alpha code.
    pub fn iso_code(&self) -> &'static str {
        match self {
            Self::Usd => "USD",
            Self::Brl => "BRL",
            Self::Dop => "DOP",
            Self::Htg => "HTG",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.iso_code())
    }
}

impl TryFrom<String> for Currency {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Self::Usd),
            "BRL" => Ok(Self::Brl),
            "DOP" => Ok(Self::Dop),
            "HTG" => Ok(Self::Htg),
            other => Err(format!("unknown Currency: '{other}'")),
        }
    }
}

// ── CreditIdField ─────────────────────────────────────────────────────────────

/// The applicant identity field a credit bureau requires for lookup.
///
/// Returned by `CreditBureau::applicant_id_field()` — replaces `&'static str`.
/// Determines which field the service reads off `ApplicantProfile`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreditIdField {
    /// Brazilian CPF (Cadastro de Pessoas Físicas) — 11 digits.
    Cpf,
    /// US Social Security Number — last 4 digits.
    SsnLast4,
    /// Brazilian CNPJ — for legal entity applicants.
    Cnpj,
}

impl fmt::Display for CreditIdField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cpf => "cpf",
            Self::SsnLast4 => "ssn_last4",
            Self::Cnpj => "cnpj",
        })
    }
}

impl TryFrom<String> for CreditIdField {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "cpf" => Ok(Self::Cpf),
            "ssn_last4" => Ok(Self::SsnLast4),
            "cnpj" => Ok(Self::Cnpj),
            other => Err(format!("unknown CreditIdField: '{other}'")),
        }
    }
}

// ── SubJurisdiction ───────────────────────────────────────────────────────────

/// A sub-national jurisdiction reference for regulatory records.
///
/// More granular than `Jurisdiction` (which is country-level).
/// Used in `StrRegulation::jurisdiction_label()` — replaces `&'static str`.
/// Stored in `atlas_regulatory_registrations.registration_metadata -> jurisdiction`.
///
/// Examples:
///   `SubJurisdiction { country: Us, state: Some("FL"), county_city: Some("MIAMI-DADE") }`
///   `SubJurisdiction { country: Br, state: Some("SP"), county_city: None }`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubJurisdiction {
    pub country: Jurisdiction,
    /// US state abbreviation or BR estado abbreviation.
    pub state: Option<&'static str>,
    /// County or municipality name.
    pub county_city: Option<&'static str>,
}

impl SubJurisdiction {
    /// Formats as `US-FL-MIAMI-DADE`, `BR-SP`, or `BR`.
    pub fn label(&self) -> String {
        let mut parts = vec![self.country.to_string()];
        if let Some(st) = self.state {
            parts.push(st.to_string());
        }
        if let Some(cc) = self.county_city {
            parts.push(cc.to_string());
        }
        parts.join("-")
    }
}

impl fmt::Display for SubJurisdiction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label())
    }
}

// ── TaxRate ───────────────────────────────────────────────────────────────────

/// A typed tax rate — replaces bare `Decimal` + `&'static str` label pairs
/// in `TaxEngine` trait returns.
///
/// Carries: the rate itself, the currency it's remitted in, whether an OTA
/// collects it, and a human-readable label for ledger entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxRate {
    /// The rate as a decimal fraction, e.g. `dec!(0.07)` for 7%.
    pub rate: rust_decimal::Decimal,
    /// Currency in which the tax is remitted.
    pub remittance_currency: Currency,
    /// Whether major OTA platforms collect and remit on behalf of the host.
    pub ota_collects: bool,
    /// Human-readable label for ledger entries and UI.
    pub label: &'static str,
}

impl TaxRate {
    /// Compute tax amount from a gross revenue amount in minor units (cents).
    pub fn apply_to_cents(&self, gross_cents: i64) -> i64 {
        use rust_decimal::prelude::ToPrimitive;
        (rust_decimal::Decimal::from(gross_cents) * self.rate)
            .to_i64()
            .unwrap_or(0)
    }
}

// ── StatuteRef ────────────────────────────────────────────────────────────────

/// A typed reference to a legal statute — replaces `&'static str` statute names.
///
/// Used in `TenancyLaw::statute()` for audit logging and UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatuteRef {
    /// Short code: "Lei 8.245/91", "42 U.S.C. § 3604", "Fla. Stat. § 83"
    pub code: &'static str,
    /// Full name: "Lei do Inquilinato", "Fair Housing Act"
    pub name: &'static str,
    /// Country jurisdiction.
    pub country: Jurisdiction,
    /// Official URL (optional — for audit trail links).
    pub url: Option<&'static str>,
}

impl fmt::Display for StatuteRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.code)
    }
}

// ── Property type ─────────────────────────────────────────────────────────────

/// The type of real estate asset.
///
/// Stored as VARCHAR in `atlas_assets.asset_type`.
/// Used by G-27 scorecard auto-provisioning to select the correct template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    /// Detached single-family home.
    SingleFamily,
    /// Condominium unit.
    Condo,
    /// Townhouse / rowhouse.
    Townhouse,
    /// Multi-family building (duplex, triplex, apartment complex).
    MultiFamily,
    /// Short-term rental unit (Airbnb / VRBO).
    Str,
    /// Commercial / mixed-use.
    Commercial,
}

impl PropertyType {
    /// Returns the G-27 scorecard entity_type for this property type.
    pub fn scorecard_entity_type(&self) -> ScorecardEntityType {
        match self {
            Self::Str => ScorecardEntityType::StrProperty,
            _ => ScorecardEntityType::RentalUnit,
        }
    }
}

impl fmt::Display for PropertyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SingleFamily => "single_family",
            Self::Condo => "condo",
            Self::Townhouse => "townhouse",
            Self::MultiFamily => "multi_family",
            Self::Str => "str",
            Self::Commercial => "commercial",
        })
    }
}

impl TryFrom<String> for PropertyType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "single_family" => Ok(Self::SingleFamily),
            "condo" => Ok(Self::Condo),
            "townhouse" => Ok(Self::Townhouse),
            "multi_family" => Ok(Self::MultiFamily),
            "str" => Ok(Self::Str),
            "commercial" => Ok(Self::Commercial),
            other => Err(format!("unknown PropertyType: '{other}'")),
        }
    }
}

// ── Case type ─────────────────────────────────────────────────────────────────

/// PM-specific case types stored in `atlas_cases.case_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PmCaseType {
    /// Tenant-reported maintenance request.
    Maintenance,
    /// STR/LTR compliance violation or expiry.
    ComplianceViolation,
    /// Applicant screening or dispute.
    ApplicationReview,
    /// Lease renewal negotiation.
    LeaseRenewal,
    /// Eviction or move-out process case.
    MoveOut,
    /// Landlord-initiated proactive inspection. Linked to an asset (appliance or
    /// building system). Completion rolls forward `atlas_assets.scheduled_service_date`.
    ScheduledInspection,
    /// Tenant-initiated request for a portable report (rental history, payment
    /// history, full profile export). The generated file is attached to the case
    /// via `case_metadata.download_attachment_id` when ready.
    ReportRequest,
    /// Renovation / cap-ex container. Child maintenance WOs link via G-22
    /// `child_work_order`. Budget on parent; actual = Σ children.
    RenovationProject,
}

impl fmt::Display for PmCaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Maintenance => "maintenance",
            Self::ComplianceViolation => "compliance_violation",
            Self::ApplicationReview => "application_review",
            Self::LeaseRenewal => "lease_renewal",
            Self::MoveOut => "move_out",
            Self::ScheduledInspection => "scheduled_inspection",
            Self::ReportRequest => "report_request",
            Self::RenovationProject => "renovation_project",
        })
    }
}

impl TryFrom<String> for PmCaseType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "maintenance" => Ok(Self::Maintenance),
            "compliance_violation" => Ok(Self::ComplianceViolation),
            "application_review" => Ok(Self::ApplicationReview),
            "lease_renewal" => Ok(Self::LeaseRenewal),
            "move_out" => Ok(Self::MoveOut),
            "scheduled_inspection" => Ok(Self::ScheduledInspection),
            "report_request" => Ok(Self::ReportRequest),
            "renovation_project" => Ok(Self::RenovationProject),
            other => Err(format!("unknown PmCaseType: '{other}'")),
        }
    }
}

/// Folio G-22 relationship_type vocabulary (write-boundary enum).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PmRelationshipType {
    /// Parent renovation_project case → child maintenance case.
    ChildWorkOrder,
    /// Asset → preferred service provider.
    DefaultContractor,
}

impl fmt::Display for PmRelationshipType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ChildWorkOrder => "child_work_order",
            Self::DefaultContractor => "default_contractor",
        })
    }
}

impl TryFrom<String> for PmRelationshipType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "child_work_order" => Ok(Self::ChildWorkOrder),
            "default_contractor" => Ok(Self::DefaultContractor),
            other => Err(format!("unknown PmRelationshipType: '{other}'")),
        }
    }
}

/// Composed project timeline event kinds (Folio read model).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectTimelineKind {
    WorkOrder,
    Milestone,
    Expense,
    G27Scored,
    G27Pending,
    ProjectOpened,
}

/// G-27 project rollup coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectG27Coverage {
    None,
    Pending,
    Partial,
    Complete,
}

// ── Entity type (for generic entity_id references) ────────────────────────────

/// Atlas generic entity type discriminator — used in `entity_type` VARCHAR columns
/// across `atlas_cases`, `atlas_contracts`, `atlas_regulatory_registrations`, etc.
///
/// PM services always reference exactly these entity types.
/// Using this enum makes it impossible to accidentally write `"atlas_Asset"` or
/// `"AtlasAsset"` — the compiler enforces the correct casing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AtlasEntityType {
    AtlasAsset,
    AtlasContact,
    AtlasOpportunity,
    AtlasServiceProvider,
    AtlasLead,
    AtlasAccount,
}

impl fmt::Display for AtlasEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AtlasAsset => "atlas_asset",
            Self::AtlasContact => "atlas_contact",
            Self::AtlasOpportunity => "atlas_opportunity",
            Self::AtlasServiceProvider => "atlas_service_provider",
            Self::AtlasLead => "atlas_lead",
            Self::AtlasAccount => "atlas_account",
        })
    }
}

impl TryFrom<String> for AtlasEntityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "atlas_asset" => Ok(Self::AtlasAsset),
            "atlas_contact" => Ok(Self::AtlasContact),
            "atlas_opportunity" => Ok(Self::AtlasOpportunity),
            "atlas_service_provider" => Ok(Self::AtlasServiceProvider),
            "atlas_lead" => Ok(Self::AtlasLead),
            "atlas_account" => Ok(Self::AtlasAccount),
            other => Err(format!("unknown AtlasEntityType: '{other}'")),
        }
    }
}

// ── Contract type ─────────────────────────────────────────────────────────────

/// PM-specific contract types stored in `atlas_contracts.contract_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PmContractType {
    /// Long-term residential lease.
    Lease,
    /// Short-term rental booking.
    StrBooking,
    /// Vendor/contractor service agreement.
    VendorService,
    /// Property management agreement (landlord ↔ PM company).
    ManagementAgreement,
    /// Wholesaling purchase contract (equitable interest / P&S).
    WholesalePurchase,
    /// Assignment of wholesale purchase contract to cash buyer.
    Assignment,
    /// Subject-to acquisition (deed; underlying loan stays in seller name).
    SubjectToPurchase,
    /// Purchase option (no deed yet).
    PurchaseOption,
    /// Lease-option / rent-to-own with tenant-buyer.
    LeaseOption,
    /// Wraparound / AITD owner-finance sale.
    Wrap,
    /// Land contract / contract for deed.
    LandContract,
}

impl fmt::Display for PmContractType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Lease => "lease",
            Self::StrBooking => "str_booking",
            Self::VendorService => "vendor_service",
            Self::ManagementAgreement => "management_agreement",
            Self::WholesalePurchase => "wholesale_purchase",
            Self::Assignment => "assignment",
            Self::SubjectToPurchase => "subject_to_purchase",
            Self::PurchaseOption => "purchase_option",
            Self::LeaseOption => "lease_option",
            Self::Wrap => "wrap",
            Self::LandContract => "land_contract",
        })
    }
}

impl TryFrom<String> for PmContractType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "lease" => Ok(Self::Lease),
            "str_booking" => Ok(Self::StrBooking),
            "vendor_service" => Ok(Self::VendorService),
            "management_agreement" => Ok(Self::ManagementAgreement),
            "wholesale_purchase" => Ok(Self::WholesalePurchase),
            "assignment" => Ok(Self::Assignment),
            "subject_to_purchase" => Ok(Self::SubjectToPurchase),
            "purchase_option" => Ok(Self::PurchaseOption),
            "lease_option" => Ok(Self::LeaseOption),
            "wrap" => Ok(Self::Wrap),
            "land_contract" => Ok(Self::LandContract),
            other => Err(format!("unknown PmContractType: '{other}'")),
        }
    }
}

// ── Brazilian lease guarantee type ────────────────────────────────────────────

/// Guarantee mechanism for a Brazilian residential lease (Lei do Inquilinato Art. 37).
///
/// Stored as VARCHAR in `atlas_contracts.terms_metadata -> guarantee_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuaranteeType {
    /// Fiador — personal guarantor who co-signs.
    Fiador,
    /// Seguro Fiança — rental insurance policy.
    SeguroFianca,
    /// Caução — cash or asset deposit (max 3 months rent).
    Caucao,
    /// Título de Capitalização.
    TituloCapitalizacao,
    /// No guarantee required.
    None,
}

impl fmt::Display for GuaranteeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Fiador => "fiador",
            Self::SeguroFianca => "seguro_fianca",
            Self::Caucao => "caucao",
            Self::TituloCapitalizacao => "titulo_capitalizacao",
            Self::None => "none",
        })
    }
}

impl TryFrom<String> for GuaranteeType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "fiador" => Ok(Self::Fiador),
            "seguro_fianca" => Ok(Self::SeguroFianca),
            "caucao" => Ok(Self::Caucao),
            "titulo_capitalizacao" => Ok(Self::TituloCapitalizacao),
            "none" => Ok(Self::None),
            other => Err(format!("unknown GuaranteeType: '{other}'")),
        }
    }
}

// ── Maintenance category ──────────────────────────────────────────────────────

/// Category of a maintenance request.
///
/// Stored in `atlas_cases.case_metadata -> category`.
/// Used for vendor dispatch routing — plumbing tickets are routed only to
/// vendors with `trade_type = TradeType::Plumber`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceCategory {
    Plumbing,
    Electrical,
    Hvac,
    Structural,
    Pest,
    Appliance,
    Roofing,
    General,
}

impl MaintenanceCategory {
    /// Returns the matching vendor `TradeType` for dispatch routing.
    pub fn preferred_trade(&self) -> TradeType {
        match self {
            Self::Plumbing => TradeType::Plumber,
            Self::Electrical => TradeType::Electrician,
            Self::Hvac => TradeType::Hvac,
            Self::Structural => TradeType::GeneralContractor,
            Self::Roofing => TradeType::Roofer,
            _ => TradeType::General,
        }
    }
}

impl fmt::Display for MaintenanceCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Plumbing => "plumbing",
            Self::Electrical => "electrical",
            Self::Hvac => "hvac",
            Self::Structural => "structural",
            Self::Pest => "pest",
            Self::Appliance => "appliance",
            Self::Roofing => "roofing",
            Self::General => "general",
        })
    }
}

impl TryFrom<String> for MaintenanceCategory {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "plumbing" => Ok(Self::Plumbing),
            "electrical" => Ok(Self::Electrical),
            "hvac" => Ok(Self::Hvac),
            "structural" => Ok(Self::Structural),
            "pest" => Ok(Self::Pest),
            "appliance" => Ok(Self::Appliance),
            "roofing" => Ok(Self::Roofing),
            "general" => Ok(Self::General),
            other => Err(format!("unknown MaintenanceCategory: '{other}'")),
        }
    }
}

// ── Vendor trade type ─────────────────────────────────────────────────────────

/// A contractor's primary trade.
///
/// Stored in `atlas_service_providers.provider_metadata -> trade_type`.
/// Drives vendor dispatch routing in `MaintenanceService::dispatch_vendor()`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeType {
    Plumber,
    Electrician,
    Hvac,
    GeneralContractor,
    Roofer,
    Painter,
    Landscaper,
    Cleaner, // STR turnover cleaning
    Inspector,
    General,
}

impl fmt::Display for TradeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Plumber => "plumber",
            Self::Electrician => "electrician",
            Self::Hvac => "hvac",
            Self::GeneralContractor => "general_contractor",
            Self::Roofer => "roofer",
            Self::Painter => "painter",
            Self::Landscaper => "landscaper",
            Self::Cleaner => "cleaner",
            Self::Inspector => "inspector",
            Self::General => "general",
        })
    }
}

impl TryFrom<String> for TradeType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "plumber" => Ok(Self::Plumber),
            "electrician" => Ok(Self::Electrician),
            "hvac" => Ok(Self::Hvac),
            "general_contractor" => Ok(Self::GeneralContractor),
            "roofer" => Ok(Self::Roofer),
            "painter" => Ok(Self::Painter),
            "landscaper" => Ok(Self::Landscaper),
            "cleaner" => Ok(Self::Cleaner),
            "inspector" => Ok(Self::Inspector),
            "general" => Ok(Self::General),
            other => Err(format!("unknown TradeType: '{other}'")),
        }
    }
}

// ── Deal Ops track ────────────────────────────────────────────────────────────

/// Folio Deal Ops rail: wholesaling (ugly houses) vs creative finance (pretty houses).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DealTrack {
    Wholesale,
    CreativeFinance,
}

impl fmt::Display for DealTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Wholesale => "wholesale",
            Self::CreativeFinance => "creative_finance",
        })
    }
}

impl TryFrom<String> for DealTrack {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "wholesale" => Ok(Self::Wholesale),
            "creative_finance" => Ok(Self::CreativeFinance),
            other => Err(format!("unknown DealTrack: '{other}'")),
        }
    }
}

// ── Wholesale pipeline stage ──────────────────────────────────────────────────

/// Stage in the wholesale acquisition pipeline (Ron LeGrand 14-step).
///
/// Stored as VARCHAR in `atlas_opportunities.status` (no separate `stage` column).
/// Terminal: `AssignedOrClosed`, `Dead`, `ConvertedToCf`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WholesaleStage {
    New,
    Prescreened,
    OfferOut,
    UnderContract,
    TitleClear,
    Marketing,
    AssignedOrClosed,
    Dead,
    ConvertedToCf,
    /// Legacy alias — prefer `Prescreened`.
    Qualified,
    /// Legacy alias — prefer `OfferOut`.
    Negotiating,
    /// Legacy alias — prefer `AssignedOrClosed`.
    Closed,
}

impl WholesaleStage {
    /// Terminal stages cannot be transitioned out of.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::AssignedOrClosed | Self::Dead | Self::ConvertedToCf | Self::Closed
        )
    }

    /// Canonical stage for UI columns (collapses legacy aliases).
    pub fn canonical(&self) -> Self {
        match self {
            Self::Qualified => Self::Prescreened,
            Self::Negotiating => Self::OfferOut,
            Self::Closed => Self::AssignedOrClosed,
            other => other.clone(),
        }
    }
}

impl fmt::Display for WholesaleStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::New => "new",
            Self::Prescreened => "prescreened",
            Self::OfferOut => "offer_out",
            Self::UnderContract => "under_contract",
            Self::TitleClear => "title_clear",
            Self::Marketing => "marketing",
            Self::AssignedOrClosed => "assigned_or_closed",
            Self::Dead => "dead",
            Self::ConvertedToCf => "converted_to_cf",
            Self::Qualified => "qualified",
            Self::Negotiating => "negotiating",
            Self::Closed => "closed",
        })
    }
}

impl TryFrom<String> for WholesaleStage {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "new" => Ok(Self::New),
            "prescreened" => Ok(Self::Prescreened),
            "offer_out" => Ok(Self::OfferOut),
            "under_contract" => Ok(Self::UnderContract),
            "title_clear" => Ok(Self::TitleClear),
            "marketing" => Ok(Self::Marketing),
            "assigned_or_closed" => Ok(Self::AssignedOrClosed),
            "dead" => Ok(Self::Dead),
            "converted_to_cf" => Ok(Self::ConvertedToCf),
            "qualified" => Ok(Self::Qualified),
            "negotiating" => Ok(Self::Negotiating),
            "closed" => Ok(Self::Closed),
            // UI alias historically used for "new"
            "lead" => Ok(Self::New),
            other => Err(format!("unknown WholesaleStage: '{other}'")),
        }
    }
}

/// Wholesale cash-buyer disposition stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WholesaleBuyerStage {
    BuyerLead,
    PrescreenPass,
    DepositHeld,
    Assigned,
    Closed,
    Disqualified,
}

impl WholesaleBuyerStage {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Closed | Self::Disqualified)
    }
}

impl fmt::Display for WholesaleBuyerStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BuyerLead => "buyer_lead",
            Self::PrescreenPass => "prescreen_pass",
            Self::DepositHeld => "deposit_held",
            Self::Assigned => "assigned",
            Self::Closed => "closed",
            Self::Disqualified => "disqualified",
        })
    }
}

impl TryFrom<String> for WholesaleBuyerStage {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "buyer_lead" => Ok(Self::BuyerLead),
            "prescreen_pass" => Ok(Self::PrescreenPass),
            "deposit_held" => Ok(Self::DepositHeld),
            "assigned" => Ok(Self::Assigned),
            "closed" => Ok(Self::Closed),
            "disqualified" => Ok(Self::Disqualified),
            other => Err(format!("unknown WholesaleBuyerStage: '{other}'")),
        }
    }
}

/// Creative finance acquisition stages (Pretty House).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreativeFinanceAcquireStage {
    New,
    Prescreened,
    OfferStructured,
    CyaClosing,
    OwnedOrOptioned,
    Dead,
}

impl CreativeFinanceAcquireStage {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::OwnedOrOptioned | Self::Dead)
    }
}

impl fmt::Display for CreativeFinanceAcquireStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::New => "new",
            Self::Prescreened => "prescreened",
            Self::OfferStructured => "offer_structured",
            Self::CyaClosing => "cya_closing",
            Self::OwnedOrOptioned => "owned_or_optioned",
            Self::Dead => "dead",
        })
    }
}

impl TryFrom<String> for CreativeFinanceAcquireStage {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "new" => Ok(Self::New),
            "prescreened" => Ok(Self::Prescreened),
            "offer_structured" => Ok(Self::OfferStructured),
            "cya_closing" => Ok(Self::CyaClosing),
            "owned_or_optioned" => Ok(Self::OwnedOrOptioned),
            "dead" => Ok(Self::Dead),
            other => Err(format!("unknown CreativeFinanceAcquireStage: '{other}'")),
        }
    }
}

/// Creative finance disposition (tenant-buyer) stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreativeFinanceDisposeStage {
    BuyerLead,
    PrescreenPass,
    AraDeposit,
    Installed,
    ExerciseCashout,
    Disqualified,
}

impl CreativeFinanceDisposeStage {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Installed | Self::ExerciseCashout | Self::Disqualified
        )
    }
}

impl fmt::Display for CreativeFinanceDisposeStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BuyerLead => "buyer_lead",
            Self::PrescreenPass => "prescreen_pass",
            Self::AraDeposit => "ara_deposit",
            Self::Installed => "installed",
            Self::ExerciseCashout => "exercise_cashout",
            Self::Disqualified => "disqualified",
        })
    }
}

impl TryFrom<String> for CreativeFinanceDisposeStage {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "buyer_lead" => Ok(Self::BuyerLead),
            "prescreen_pass" => Ok(Self::PrescreenPass),
            "ara_deposit" => Ok(Self::AraDeposit),
            "installed" => Ok(Self::Installed),
            "exercise_cashout" => Ok(Self::ExerciseCashout),
            "disqualified" => Ok(Self::Disqualified),
            other => Err(format!("unknown CreativeFinanceDisposeStage: '{other}'")),
        }
    }
}

/// Pretty House buy structures (six) + wholesale all-cash MAO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcquisitionStructure {
    AllCashMao,
    SubjectToFreeEquity,
    SubjectToCashEquity,
    SubjectToDeferredEquity,
    SubjectToSellerSecond,
    SellerFinanceWrap,
    PurchaseOption,
}

impl fmt::Display for AcquisitionStructure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AllCashMao => "all_cash_mao",
            Self::SubjectToFreeEquity => "subject_to_free_equity",
            Self::SubjectToCashEquity => "subject_to_cash_equity",
            Self::SubjectToDeferredEquity => "subject_to_deferred_equity",
            Self::SubjectToSellerSecond => "subject_to_seller_second",
            Self::SellerFinanceWrap => "seller_finance_wrap",
            Self::PurchaseOption => "purchase_option",
        })
    }
}

impl TryFrom<String> for AcquisitionStructure {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "all_cash_mao" => Ok(Self::AllCashMao),
            "subject_to_free_equity" => Ok(Self::SubjectToFreeEquity),
            "subject_to_cash_equity" => Ok(Self::SubjectToCashEquity),
            "subject_to_deferred_equity" => Ok(Self::SubjectToDeferredEquity),
            "subject_to_seller_second" => Ok(Self::SubjectToSellerSecond),
            "seller_finance_wrap" => Ok(Self::SellerFinanceWrap),
            "purchase_option" => Ok(Self::PurchaseOption),
            other => Err(format!("unknown AcquisitionStructure: '{other}'")),
        }
    }
}

/// Planned exit after acquisition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitMode {
    WholesaleAssignment,
    SimultaneousClose,
    LeaseOption,
    OwnerFinanceWrap,
    LandContract,
    RetailCash,
}

impl fmt::Display for ExitMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::WholesaleAssignment => "wholesale_assignment",
            Self::SimultaneousClose => "simultaneous_close",
            Self::LeaseOption => "lease_option",
            Self::OwnerFinanceWrap => "owner_finance_wrap",
            Self::LandContract => "land_contract",
            Self::RetailCash => "retail_cash",
        })
    }
}

impl TryFrom<String> for ExitMode {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "wholesale_assignment" => Ok(Self::WholesaleAssignment),
            "simultaneous_close" => Ok(Self::SimultaneousClose),
            "lease_option" => Ok(Self::LeaseOption),
            "owner_finance_wrap" => Ok(Self::OwnerFinanceWrap),
            "land_contract" => Ok(Self::LandContract),
            "retail_cash" => Ok(Self::RetailCash),
            other => Err(format!("unknown ExitMode: '{other}'")),
        }
    }
}

/// Buyer fit matrix (wholesale cash vs creative-finance tenant-buyer).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuyerFit {
    /// Wholesale: can raise cash and close quickly.
    CashReady,
    /// Wholesale: interested but needs more time.
    NeedsTime,
    /// CF: strong money + credit — conventional path.
    Conventional,
    /// CF: money + weak credit — lease-option / OF.
    LeaseOptionOrOf,
    /// CF: little money + strong credit — loan only / nurture.
    LoanOnly,
    Disqualify,
}

impl fmt::Display for BuyerFit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CashReady => "cash_ready",
            Self::NeedsTime => "needs_time",
            Self::Conventional => "conventional",
            Self::LeaseOptionOrOf => "lease_option_or_of",
            Self::LoanOnly => "loan_only",
            Self::Disqualify => "disqualify",
        })
    }
}

impl TryFrom<String> for BuyerFit {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "cash_ready" => Ok(Self::CashReady),
            "needs_time" => Ok(Self::NeedsTime),
            "conventional" => Ok(Self::Conventional),
            "lease_option_or_of" => Ok(Self::LeaseOptionOrOf),
            "loan_only" => Ok(Self::LoanOnly),
            "disqualify" => Ok(Self::Disqualify),
            other => Err(format!("unknown BuyerFit: '{other}'")),
        }
    }
}

/// Application type for G-18 `atlas_applications.application_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PmApplicationType {
    Rental,
    TenantBuyer,
    CashBuyer,
}

impl fmt::Display for PmApplicationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rental => "rental",
            Self::TenantBuyer => "tenant_buyer",
            Self::CashBuyer => "cash_buyer",
        })
    }
}

impl TryFrom<String> for PmApplicationType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "rental" => Ok(Self::Rental),
            "tenant_buyer" => Ok(Self::TenantBuyer),
            "cash_buyer" => Ok(Self::CashBuyer),
            other => Err(format!("unknown PmApplicationType: '{other}'")),
        }
    }
}

// ── Wholesale seller motivation ───────────────────────────────────────────────

/// Why a seller is motivated to sell quickly.
///
/// Stored in `atlas_opportunities.financial_inputs -> motivation`.
/// Used by the Lead Quality Assessment G-27 scorecard as a dimension signal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SellerMotivation {
    Divorce,
    Probate,
    Foreclosure,
    RelocationJob,
    FinancialDistress,
    Inheritance,
    Downsizing,
    TiredLandlord,
    Other,
}

impl fmt::Display for SellerMotivation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Divorce => "divorce",
            Self::Probate => "probate",
            Self::Foreclosure => "foreclosure",
            Self::RelocationJob => "relocation_job",
            Self::FinancialDistress => "financial_distress",
            Self::Inheritance => "inheritance",
            Self::Downsizing => "downsizing",
            Self::TiredLandlord => "tired_landlord",
            Self::Other => "other",
        })
    }
}

impl TryFrom<String> for SellerMotivation {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "divorce" => Ok(Self::Divorce),
            "probate" => Ok(Self::Probate),
            "foreclosure" => Ok(Self::Foreclosure),
            "relocation_job" => Ok(Self::RelocationJob),
            "financial_distress" => Ok(Self::FinancialDistress),
            "inheritance" => Ok(Self::Inheritance),
            "downsizing" => Ok(Self::Downsizing),
            "tired_landlord" => Ok(Self::TiredLandlord),
            "other" => Ok(Self::Other),
            other => Err(format!("unknown SellerMotivation: '{other}'")),
        }
    }
}

// ── Opportunity type ──────────────────────────────────────────────────────────

/// PM opportunity types stored in `atlas_opportunities.opportunity_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PmOpportunityType {
    /// Wholesale acquisition lead (ugly-house / MAO track).
    WholesaleLead,
    /// Cash investor buyer for a wholesale disposition.
    WholesaleBuyer,
    /// Pretty House / creative-finance acquisition.
    CreativeFinanceAcquisition,
    /// Tenant-buyer disposition for creative finance.
    CreativeFinanceDisposition,
    /// New tenant application for an available unit.
    LeaseApplication,
    /// Lease renewal negotiation.
    LeaseRenewal,
}

impl fmt::Display for PmOpportunityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::WholesaleLead => "wholesale_lead",
            Self::WholesaleBuyer => "wholesale_buyer",
            Self::CreativeFinanceAcquisition => "creative_finance_acquisition",
            Self::CreativeFinanceDisposition => "creative_finance_disposition",
            Self::LeaseApplication => "lease_application",
            Self::LeaseRenewal => "lease_renewal",
        })
    }
}

impl TryFrom<String> for PmOpportunityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "wholesale_lead" => Ok(Self::WholesaleLead),
            "wholesale_buyer" => Ok(Self::WholesaleBuyer),
            "creative_finance_acquisition" => Ok(Self::CreativeFinanceAcquisition),
            "creative_finance_disposition" => Ok(Self::CreativeFinanceDisposition),
            "lease_application" => Ok(Self::LeaseApplication),
            "lease_renewal" => Ok(Self::LeaseRenewal),
            other => Err(format!("unknown PmOpportunityType: '{other}'")),
        }
    }
}

// ── STR permit category ───────────────────────────────────────────────────────

/// Miami-Dade STR permit category.
///
/// Stored in `atlas_regulatory_registrations.registration_metadata -> permit_category`.
/// Determines zoning eligibility and platform fee structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrPermitCategory {
    /// Owner's primary residence — most permissive.
    PrincipalResidence,
    /// Owner present during guest stays.
    Hosted,
    /// Owner absent during guest stays — most restricted.
    NonHosted,
}

impl fmt::Display for StrPermitCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PrincipalResidence => "principal_residence",
            Self::Hosted => "hosted",
            Self::NonHosted => "non_hosted",
        })
    }
}

impl TryFrom<String> for StrPermitCategory {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "principal_residence" => Ok(Self::PrincipalResidence),
            "hosted" => Ok(Self::Hosted),
            "non_hosted" => Ok(Self::NonHosted),
            other => Err(format!("unknown StrPermitCategory: '{other}'")),
        }
    }
}

// ── Regulatory registration type ──────────────────────────────────────────────

/// PM registration types stored in `atlas_regulatory_registrations.registration_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PmRegistrationType {
    /// Miami-Dade STR operating permit.
    StrPermit,
    /// Contractor trade license.
    ContractorLicense,
    /// Business/rental license.
    BusinessLicense,
    /// Certificate of occupancy.
    CertificateOfOccupancy,
}

impl fmt::Display for PmRegistrationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StrPermit => "str_permit",
            Self::ContractorLicense => "contractor_license",
            Self::BusinessLicense => "business_license",
            Self::CertificateOfOccupancy => "certificate_of_occupancy",
        })
    }
}

impl TryFrom<String> for PmRegistrationType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "str_permit" => Ok(Self::StrPermit),
            "contractor_license" => Ok(Self::ContractorLicense),
            "business_license" => Ok(Self::BusinessLicense),
            "certificate_of_occupancy" => Ok(Self::CertificateOfOccupancy),
            other => Err(format!("unknown PmRegistrationType: '{other}'")),
        }
    }
}

// ── G-27 scorecard entity types (PM-specific) ─────────────────────────────────

/// G-27 entity types for the four canonical PM scorecard templates.
///
/// Stored as VARCHAR in `atlas_scorecards.entity_type` and
/// `atlas_scorecard_templates.entity_type`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScorecardEntityType {
    /// STR Property Assessment — rates the physical property for STR suitability.
    StrProperty,
    /// Rental Unit Quality — rates LTR unit quality (cleanliness, amenities, responsiveness).
    RentalUnit,
    /// Contractor Performance — rates vendor quality, timeliness, workmanship.
    Contractor,
    /// Lead Quality Assessment — rates wholesale lead viability (private per operator).
    WholesaleLead,
}

impl fmt::Display for ScorecardEntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StrProperty => "str_property",
            Self::RentalUnit => "rental_unit",
            Self::Contractor => "contractor",
            Self::WholesaleLead => "wholesale_lead",
        })
    }
}

impl TryFrom<String> for ScorecardEntityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "str_property" => Ok(Self::StrProperty),
            "rental_unit" => Ok(Self::RentalUnit),
            "contractor" => Ok(Self::Contractor),
            "wholesale_lead" => Ok(Self::WholesaleLead),
            other => Err(format!("unknown ScorecardEntityType: '{other}'")),
        }
    }
}

// ── Folio role ────────────────────────────────────────────────────────────────

/// The PM-context role of a user within the Folio application.
///
/// Stored as VARCHAR in `user_account.folio_role` (added by
/// `m20260810_add_folio_role_to_user_account`). Determines which Folio
/// frontend namespace the user is routed to and which backend endpoints
/// they are permitted to call.
///
/// - `Landlord`        — single property manager / PM operator. Full PM suite.
/// - `Tenant`          — renter or STR guest. Own lease, payments, maintenance.
/// - `Vendor`          — contractor. Assigned work orders and invoices.
/// - `PropertyManager` — PMC operator. Cross-client view of multiple landlord books.
///                       Only valid when app config has `"pmc_enabled": true`.
/// - `PropertyOwnerLite` — free-tier property owner. Tracks property value and vendors.
///                       No leases, no billing. Upgrade path → Landlord via Stripe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FolioRole {
    #[default]
    Landlord,
    Tenant,
    /// STR booking guest — short-term rental visitor.
    /// Linked to a booking (atlas_bookings), not a lease.
    /// Onboarding: select/confirm dates → profile → house rules.
    /// Distinct from Tenant (LTR applicant/renter). Home path: `/g`.
    StrGuest,
    Vendor,
    PropertyManager,
    Owner,
    /// STR co-host — manages bookings, messaging, cleaning for specific STR assets
    /// they've been delegated. Access is asset-scoped via `atlas_user_asset_access`.
    Cohost,
    // NOTE: StrHost is NOT a separate role. STR is an asset-level trait:
    //   atlas_assets.str_eligible = true.
    // A Landlord whose portfolio contains str_eligible assets gets the STR sections
    // in their dashboard dynamically (has_str_assets in SessionInfo).
    Agent,
    Broker,
    /// Free-tier property owner. Same `/dashboard` shell as Landlord but with
    /// Landlord-only modules gated behind `is_lite()`. Tracks property value,
    /// linked vendors, and submits G-27 reviews via invite.
    ///
    /// Upgrade path: `POST /api/folio/upgrade-role` → Stripe → `role_slug = 'landlord'`.
    /// G-10 asset + value history carry forward with zero re-entry on upgrade.
    PropertyOwnerLite,
}

impl FolioRole {
    /// Returns the home path for this role's frontend namespace.
    /// Used by the frontend router to redirect after login.
    pub fn home_path(&self) -> &'static str {
        match self {
            Self::Landlord => "/dashboard",
            Self::Tenant => "/my-home",
            Self::StrGuest => "/g",
            Self::Vendor => "/work-orders",
            Self::PropertyManager => "/pm",
            Self::Owner => "/owner",
            Self::Cohost => "/ch",
            Self::Agent => "/a",
            Self::Broker => "/b",
            Self::PropertyOwnerLite => "/dashboard",
        }
    }

    /// Returns true if this role has cross-client (PMC) capabilities.
    pub fn is_pmc(&self) -> bool {
        matches!(self, Self::PropertyManager)
    }

    /// Returns true if this is a read-only beneficial owner.
    pub fn is_owner(&self) -> bool {
        matches!(self, Self::Owner)
    }

    /// Returns true if this role operates in the brokerage namespace.
    pub fn is_brokerage(&self) -> bool {
        matches!(self, Self::Agent | Self::Broker)
    }

    /// Returns true if this is the free-tier Property Owner Lite role.
    ///
    /// Used by the frontend to gate Landlord-only modules (leases, rent collection,
    /// maintenance dispatch, campaigns) and show the upgrade banner.
    pub fn is_lite(&self) -> bool {
        matches!(self, Self::PropertyOwnerLite)
    }
}

impl fmt::Display for FolioRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Landlord => "landlord",
            Self::Tenant => "tenant",
            Self::StrGuest => "str_guest",
            Self::Vendor => "vendor",
            Self::PropertyManager => "property_manager",
            Self::Owner => "owner",
            Self::Cohost => "cohost",
            Self::Agent => "agent",
            Self::Broker => "broker",
            Self::PropertyOwnerLite => "property_owner_lite",
        })
    }
}

impl TryFrom<String> for FolioRole {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "landlord" => Ok(Self::Landlord),
            "tenant" => Ok(Self::Tenant),
            "str_guest" => Ok(Self::StrGuest),
            "vendor" => Ok(Self::Vendor),
            "property_manager" => Ok(Self::PropertyManager),
            "owner" => Ok(Self::Owner),
            "cohost" => Ok(Self::Cohost),
            "agent" => Ok(Self::Agent),
            "broker" => Ok(Self::Broker),
            "property_owner_lite" => Ok(Self::PropertyOwnerLite),
            // str_host was removed — it is an asset trait, not a role.
            // Invites with app_role="str_host" are rejected at provision validation.
            other => Err(format!(
                "unknown FolioRole: '{other}' (hint: use 'landlord' — STR capability is enabled per-asset)"
            )),
        }
    }
}

impl TryFrom<&str> for FolioRole {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        FolioRole::try_from(s.to_string())
    }
}

// ── PropertyValueSource ───────────────────────────────────────────────────────

/// Source type for a property value history entry.
///
/// Stored as VARCHAR in `atlas_asset_value_history.source`.
/// Each variant renders with a distinct marker style on the value history chart.
/// `source_ref TEXT` on the same row carries the external URL, document ID,
/// appraiser name, or AVM report ID for traceability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyValueSource {
    /// Owner self-reported estimate.
    ManualEntry,
    /// The original purchase price — baseline anchor for appreciation tracking.
    PurchasePrice,
    /// Zillow automated valuation model estimate.
    ZillowAvm,
    /// County tax assessment value.
    CountyRecord,
    /// Licensed appraiser report (full appraisal).
    CertifiedAppraisal,
    /// Bank or lender appraisal from a mortgage or refinance.
    BankAppraisal,
    /// Real estate agent comparative market analysis.
    AgentCma,
}

impl fmt::Display for PropertyValueSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ManualEntry => "manual",
            Self::PurchasePrice => "purchase_price",
            Self::ZillowAvm => "zillow_avm",
            Self::CountyRecord => "county_record",
            Self::CertifiedAppraisal => "certified_appraisal",
            Self::BankAppraisal => "bank_appraisal",
            Self::AgentCma => "agent_cma",
        })
    }
}

impl TryFrom<String> for PropertyValueSource {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "manual" => Ok(Self::ManualEntry),
            "purchase_price" => Ok(Self::PurchasePrice),
            "zillow_avm" => Ok(Self::ZillowAvm),
            "county_record" => Ok(Self::CountyRecord),
            "certified_appraisal" => Ok(Self::CertifiedAppraisal),
            "bank_appraisal" => Ok(Self::BankAppraisal),
            "agent_cma" => Ok(Self::AgentCma),
            other => Err(format!("unknown PropertyValueSource: '{other}'")),
        }
    }
}

// ── InvitePurpose ─────────────────────────────────────────────────────────────

/// Discriminator stored in `platform_invite.invite_purpose`.
///
/// Distinguishes a standard onboarding invite from a vendor-initiated review
/// request. The `context_entity_id` column on the same row carries the
/// relevant entity FK (e.g. the `atlas_service_providers.id` for a review invite).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvitePurpose {
    /// Standard user onboarding invite (default behaviour, existing rows).
    Onboarding,
    /// Vendor-initiated G-27 review request sent to a property owner.
    /// `context_entity_id` = the `atlas_service_providers.id` of the vendor.
    ReviewRequest,
    /// G-36 NetworkInvite peer growth invite.
    NetworkReferral,
}

impl fmt::Display for InvitePurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Onboarding => "onboarding",
            Self::ReviewRequest => "review_request",
            Self::NetworkReferral => "network_referral",
        })
    }
}

impl TryFrom<String> for InvitePurpose {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "onboarding" => Ok(Self::Onboarding),
            "review_request" => Ok(Self::ReviewRequest),
            "network_referral" => Ok(Self::NetworkReferral),
            other => Err(format!("unknown InvitePurpose: '{other}'")),
        }
    }
}

// ── Template scope ────────────────────────────────────────────────────────────

/// G-27 template scope — controls cross-tenant benchmark aggregation.
///
/// Stored as VARCHAR in `atlas_scorecard_templates.template_scope`.
/// Added by migration `m20260801_pm_g27_template_scope`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateScope {
    /// Canonical cross-tenant template. Eligible for benchmark pool aggregation.
    Platform,
    /// Private per-landlord template. Excluded from cross-tenant pool.
    Tenant,
}

impl fmt::Display for TemplateScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Platform => "platform",
            Self::Tenant => "tenant",
        })
    }
}

impl TryFrom<String> for TemplateScope {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "platform" => Ok(Self::Platform),
            "tenant" => Ok(Self::Tenant),
            other => Err(format!("unknown TemplateScope: '{other}'")),
        }
    }
}

// ── Condomínio expense category ───────────────────────────────────────────────

/// Classification of a Brazilian condomínio expense per Lei do Inquilinato Art. 22–23.
///
/// Stored in `atlas_ledger_splits.metadata -> condominio_split -> category`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConominioExpenseCategory {
    /// Tenant obligation — day-to-day operating expenses (Art. 23).
    DespesasOrdinarias,
    /// Landlord obligation — capital/structural expenses (Art. 22).
    DespesasExtraordinarias,
}

impl fmt::Display for ConominioExpenseCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::DespesasOrdinarias => "despesas_ordinarias",
            Self::DespesasExtraordinarias => "despesas_extraordinarias",
        })
    }
}

impl TryFrom<String> for ConominioExpenseCategory {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "despesas_ordinarias" => Ok(Self::DespesasOrdinarias),
            "despesas_extraordinarias" => Ok(Self::DespesasExtraordinarias),
            other => Err(format!("unknown ConominioExpenseCategory: '{other}'")),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// UNIT TESTS
// ══════════════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jurisdiction_fha_applies() {
        assert!(Jurisdiction::Us.fha_applies());
        assert!(Jurisdiction::Vi.fha_applies());
        assert!(!Jurisdiction::Br.fha_applies());
        assert!(!Jurisdiction::Do.fha_applies());
        assert!(!Jurisdiction::Ht.fha_applies());
    }

    #[test]
    fn test_jurisdiction_lei_inquilinato() {
        assert!(Jurisdiction::Br.lei_do_inquilinato_applies());
        assert!(!Jurisdiction::Us.lei_do_inquilinato_applies());
    }

    #[test]
    fn test_jurisdiction_default_currency() {
        assert_eq!(Jurisdiction::Us.default_currency(), "USD");
        assert_eq!(Jurisdiction::Vi.default_currency(), "USD");
        assert_eq!(Jurisdiction::Br.default_currency(), "BRL");
        assert_eq!(Jurisdiction::Do.default_currency(), "DOP");
    }

    #[test]
    fn test_jurisdiction_roundtrip() {
        for (s, expected) in &[
            ("US", Jurisdiction::Us),
            ("BR", Jurisdiction::Br),
            ("VI", Jurisdiction::Vi),
        ] {
            let parsed = Jurisdiction::try_from(s.to_string()).unwrap();
            assert_eq!(&parsed, expected);
            assert_eq!(parsed.to_string().to_uppercase(), s.to_uppercase());
        }
    }

    #[test]
    fn test_property_type_scorecard_entity() {
        assert_eq!(
            PropertyType::Str.scorecard_entity_type(),
            ScorecardEntityType::StrProperty
        );
        assert_eq!(
            PropertyType::Condo.scorecard_entity_type(),
            ScorecardEntityType::RentalUnit
        );
        assert_eq!(
            PropertyType::SingleFamily.scorecard_entity_type(),
            ScorecardEntityType::RentalUnit
        );
    }

    #[test]
    fn test_maintenance_preferred_trade() {
        assert_eq!(
            MaintenanceCategory::Plumbing.preferred_trade(),
            TradeType::Plumber
        );
        assert_eq!(
            MaintenanceCategory::Electrical.preferred_trade(),
            TradeType::Electrician
        );
        assert_eq!(
            MaintenanceCategory::Pest.preferred_trade(),
            TradeType::General
        );
    }

    #[test]
    fn test_wholesale_stage_terminal() {
        assert!(WholesaleStage::Closed.is_terminal());
        assert!(WholesaleStage::Dead.is_terminal());
        assert!(!WholesaleStage::Qualified.is_terminal());
        assert!(!WholesaleStage::UnderContract.is_terminal());
    }

    #[test]
    fn test_all_enums_display_parse_roundtrip() {
        // Spot-check every enum: Display → TryFrom should roundtrip.
        macro_rules! check_roundtrip {
            ($val:expr) => {{
                let s = $val.to_string();
                let parsed = std::convert::TryFrom::try_from(s.clone())
                    .expect(&format!("roundtrip failed for '{s}'"));
                assert_eq!($val, parsed);
            }};
        }

        check_roundtrip!(PmCaseType::Maintenance);
        check_roundtrip!(PmCaseType::ComplianceViolation);
        check_roundtrip!(PmCaseType::RenovationProject);
        check_roundtrip!(PmRelationshipType::ChildWorkOrder);
        check_roundtrip!(AtlasEntityType::AtlasAsset);
        check_roundtrip!(AtlasEntityType::AtlasServiceProvider);
        check_roundtrip!(PmContractType::Lease);
        check_roundtrip!(PmContractType::StrBooking);
        check_roundtrip!(GuaranteeType::Fiador);
        check_roundtrip!(GuaranteeType::TituloCapitalizacao);
        check_roundtrip!(TradeType::GeneralContractor);
        check_roundtrip!(WholesaleStage::UnderContract);
        check_roundtrip!(WholesaleStage::AssignedOrClosed);
        check_roundtrip!(SellerMotivation::TiredLandlord);
        check_roundtrip!(PmOpportunityType::WholesaleLead);
        check_roundtrip!(PmOpportunityType::CreativeFinanceAcquisition);
        check_roundtrip!(DealTrack::CreativeFinance);
        check_roundtrip!(AcquisitionStructure::SubjectToFreeEquity);
        check_roundtrip!(ExitMode::LeaseOption);
        check_roundtrip!(BuyerFit::LeaseOptionOrOf);
        check_roundtrip!(PmContractType::LeaseOption);
        check_roundtrip!(PmApplicationType::TenantBuyer);
        check_roundtrip!(StrPermitCategory::PrincipalResidence);
        check_roundtrip!(PmRegistrationType::StrPermit);
        check_roundtrip!(ScorecardEntityType::Contractor);
        check_roundtrip!(TemplateScope::Platform);
        check_roundtrip!(ConominioExpenseCategory::DespesasOrdinarias);
    }

    #[test]
    fn test_unknown_variants_return_err() {
        assert!(Jurisdiction::try_from("XX".to_string()).is_err());
        assert!(PropertyType::try_from("penthouse".to_string()).is_err());
        assert!(PmCaseType::try_from("unknown_case".to_string()).is_err());
        assert!(WholesaleStage::try_from("won".to_string()).is_err());
        assert!(TradeType::try_from("wizard".to_string()).is_err());
    }
}

// ── OtaPlatform ───────────────────────────────────────────────────────────────

/// The OTA channel a reservation originated from.
///
/// Stored in `atlas_reservations.reservation_metadata["ota_platform"]`.
/// The `Other` variant handles any channel not yet explicitly modelled.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OtaPlatform {
    Airbnb,
    Vrbo,
    BookingCom,
    Direct,
    Other(String),
}

impl fmt::Display for OtaPlatform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OtaPlatform::Airbnb => write!(f, "airbnb"),
            OtaPlatform::Vrbo => write!(f, "vrbo"),
            OtaPlatform::BookingCom => write!(f, "booking_com"),
            OtaPlatform::Direct => write!(f, "direct"),
            OtaPlatform::Other(s) => write!(f, "{s}"),
        }
    }
}

impl TryFrom<String> for OtaPlatform {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "airbnb" => Ok(OtaPlatform::Airbnb),
            "vrbo" => Ok(OtaPlatform::Vrbo),
            "booking_com" | "booking.com" => Ok(OtaPlatform::BookingCom),
            "direct" => Ok(OtaPlatform::Direct),
            other => Ok(OtaPlatform::Other(other.to_string())),
        }
    }
}

// ── G26 Catalog enums ─────────────────────────────────────────────────────────
//
// These replace raw string fields throughout the catalog layer.
// The database stores VARCHAR (for forward-compat with app-specific subtypes);
// the service layer converts via TryFrom<String> / Display at the boundary.

// ── BillingInterval ───────────────────────────────────────────────────────────
//
// The time unit in which a catalog entry charges. Also determines what the
// `min_duration` field on a rate rule means:
//   Nightly   → min_duration = minimum consecutive nights
//   Hourly    → min_duration = minimum hours
//   Daily     → min_duration = minimum days (equipment rental)
//   Weekly    → min_duration = minimum weeks
//   Monthly   → min_duration = minimum months (subscription cycles)
//   Annually  → min_duration = minimum years
//   PerUnit   → min_duration = minimum quantity (tickets, seats, passes)
//
// A NULL billing_interval means a one-time, non-recurring purchase.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingInterval {
    Nightly,
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Annually,
    PerUnit,
}

impl fmt::Display for BillingInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BillingInterval::Nightly => write!(f, "nightly"),
            BillingInterval::Hourly => write!(f, "hourly"),
            BillingInterval::Daily => write!(f, "daily"),
            BillingInterval::Weekly => write!(f, "weekly"),
            BillingInterval::Monthly => write!(f, "monthly"),
            BillingInterval::Annually => write!(f, "annually"),
            BillingInterval::PerUnit => write!(f, "per_unit"),
        }
    }
}

impl TryFrom<String> for BillingInterval {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "nightly" => Ok(BillingInterval::Nightly),
            "hourly" => Ok(BillingInterval::Hourly),
            "daily" => Ok(BillingInterval::Daily),
            "weekly" => Ok(BillingInterval::Weekly),
            "monthly" => Ok(BillingInterval::Monthly),
            "annually" => Ok(BillingInterval::Annually),
            "per_unit" => Ok(BillingInterval::PerUnit),
            other => Err(format!("unknown BillingInterval: {other}")),
        }
    }
}

impl TryFrom<&str> for BillingInterval {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        BillingInterval::try_from(s.to_string())
    }
}

// ── CatalogEntryType ──────────────────────────────────────────────────────────
//
// Discriminates the product category stored in `atlas_catalog_entries.entry_type`.
// Must match the values in the `atlas_catalog_entry_type` PostgreSQL enum.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogEntryType {
    /// Hotel or STR room category (King Suite, Studio, etc.)
    RoomType,
    /// Time-bounded service slot (cleaning appointment, HVAC visit, etc.)
    ServiceSlot,
    /// Travel bundle tier (Economy / Standard / Premium)
    PackageTier,
    /// SaaS or creator plan (Free / Pro / Enterprise)
    SubscriptionTier,
    /// Insurance coverage product (Basic / Plus / Comprehensive)
    CoverageOption,
    /// Ancillary upsell item (parking, breakfast, early check-in, etc.)
    AddOn,
    /// Rentable physical unit (car, bike, tool, equipment)
    EquipmentUnit,
}

impl fmt::Display for CatalogEntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CatalogEntryType::RoomType => write!(f, "room_type"),
            CatalogEntryType::ServiceSlot => write!(f, "service_slot"),
            CatalogEntryType::PackageTier => write!(f, "package_tier"),
            CatalogEntryType::SubscriptionTier => write!(f, "subscription_tier"),
            CatalogEntryType::CoverageOption => write!(f, "coverage_option"),
            CatalogEntryType::AddOn => write!(f, "add_on"),
            CatalogEntryType::EquipmentUnit => write!(f, "equipment_unit"),
        }
    }
}

impl TryFrom<String> for CatalogEntryType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "room_type" => Ok(CatalogEntryType::RoomType),
            "service_slot" => Ok(CatalogEntryType::ServiceSlot),
            "package_tier" => Ok(CatalogEntryType::PackageTier),
            "subscription_tier" => Ok(CatalogEntryType::SubscriptionTier),
            "coverage_option" => Ok(CatalogEntryType::CoverageOption),
            "add_on" => Ok(CatalogEntryType::AddOn),
            "equipment_unit" => Ok(CatalogEntryType::EquipmentUnit),
            other => Err(format!("unknown CatalogEntryType: {other}")),
        }
    }
}

impl TryFrom<&str> for CatalogEntryType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CatalogEntryType::try_from(s.to_string())
    }
}

// ── BookingChannel ────────────────────────────────────────────────────────────
//
// The distribution channel through which a booking originates.
// Used in `atlas_catalog_rate_rules.channel` to scope a rate rule to a
// specific channel (e.g. OTA discount, direct booking premium, corporate rate).
// A NULL channel on a rule means it applies to all channels.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BookingChannel {
    /// Guest books directly through the operator's own website or app.
    Direct,
    /// Guest books through a third-party OTA (Airbnb, Vrbo, Booking.com, etc.)
    Ota,
    /// Corporate booking via a GDS (Sabre, Amadeus, Travelport).
    Gds,
    /// Pre-negotiated corporate rate with a specific company.
    Corporate,
    /// Group booking (conference, wedding block, etc.)
    Group,
}

impl fmt::Display for BookingChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BookingChannel::Direct => write!(f, "direct"),
            BookingChannel::Ota => write!(f, "ota"),
            BookingChannel::Gds => write!(f, "gds"),
            BookingChannel::Corporate => write!(f, "corporate"),
            BookingChannel::Group => write!(f, "group"),
        }
    }
}

impl TryFrom<String> for BookingChannel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "direct" => Ok(BookingChannel::Direct),
            "ota" => Ok(BookingChannel::Ota),
            "gds" => Ok(BookingChannel::Gds),
            "corporate" => Ok(BookingChannel::Corporate),
            "group" => Ok(BookingChannel::Group),
            other => Err(format!("unknown BookingChannel: {other}")),
        }
    }
}

impl TryFrom<&str> for BookingChannel {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        BookingChannel::try_from(s.to_string())
    }
}

// ── G19 Campaign enums ────────────────────────────────────────────────────────
//
// All seven enum types follow the same pattern as G26 (BillingInterval, etc.):
//   - DB column stays VARCHAR for forward-compat
//   - Service input types use the enum (enforced at payload boundary)
//   - DB write: enum.to_string()
//   - DB read: CampaignType::try_from(row.campaign_type)?
//   - JSON: serde rename_all = "snake_case"

// ── CampaignType ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignType {
    /// Multi-step cold email sequence (Instantly.ai, Lemlist, Apollo)
    ColdEmail,
    /// Physical direct mail (postcards, letters, flyers via USPS/Lob/PostGrid)
    DirectMail,
    /// Pay-per-click ads (Google Ads, Meta Ads, LinkedIn Ads)
    Ppc,
    /// Organic or paid social media posts
    Social,
    /// Conference, webinar, open house, meetup
    EventBased,
    /// SMS broadcast or drip
    Sms,
    /// Blog, SEO, lead magnet, content download
    Content,
    /// Referral / partner program
    Referral,
    /// Pixel-based retargeting (Facebook Pixel, Google RLSA)
    Retargeting,
}

impl fmt::Display for CampaignType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignType::ColdEmail => write!(f, "cold_email"),
            CampaignType::DirectMail => write!(f, "direct_mail"),
            CampaignType::Ppc => write!(f, "ppc"),
            CampaignType::Social => write!(f, "social"),
            CampaignType::EventBased => write!(f, "event_based"),
            CampaignType::Sms => write!(f, "sms"),
            CampaignType::Content => write!(f, "content"),
            CampaignType::Referral => write!(f, "referral"),
            CampaignType::Retargeting => write!(f, "retargeting"),
        }
    }
}

impl TryFrom<String> for CampaignType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "cold_email" => Ok(CampaignType::ColdEmail),
            "direct_mail" => Ok(CampaignType::DirectMail),
            "ppc" => Ok(CampaignType::Ppc),
            "social" => Ok(CampaignType::Social),
            "event_based" => Ok(CampaignType::EventBased),
            "sms" => Ok(CampaignType::Sms),
            "content" => Ok(CampaignType::Content),
            "referral" => Ok(CampaignType::Referral),
            "retargeting" => Ok(CampaignType::Retargeting),
            other => Err(format!("unknown CampaignType: {other}")),
        }
    }
}

impl TryFrom<&str> for CampaignType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CampaignType::try_from(s.to_string())
    }
}

// ── CampaignStatus ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Active,
    Paused,
    Completed,
    Archived,
}

impl fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignStatus::Draft => write!(f, "draft"),
            CampaignStatus::Scheduled => write!(f, "scheduled"),
            CampaignStatus::Active => write!(f, "active"),
            CampaignStatus::Paused => write!(f, "paused"),
            CampaignStatus::Completed => write!(f, "completed"),
            CampaignStatus::Archived => write!(f, "archived"),
        }
    }
}

impl TryFrom<String> for CampaignStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(CampaignStatus::Draft),
            "scheduled" => Ok(CampaignStatus::Scheduled),
            "active" => Ok(CampaignStatus::Active),
            "paused" => Ok(CampaignStatus::Paused),
            "completed" => Ok(CampaignStatus::Completed),
            "archived" => Ok(CampaignStatus::Archived),
            other => Err(format!("unknown CampaignStatus: {other}")),
        }
    }
}

impl TryFrom<&str> for CampaignStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CampaignStatus::try_from(s.to_string())
    }
}

/// Build unique human-readable campaign id: `{app_id}_{slug(name)}`.
pub fn campaign_global_name(app_id: &str, name: &str) -> String {
    let app = app_id
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    let slug = name
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let slug = slug
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    let app = app
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    format!("{app}_{slug}")
}

// ── G-37 Ambassador enums ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbassadorPartnerType {
    Referral,
    Influencer,
    Affiliate,
    Recruiter,
}

impl fmt::Display for AmbassadorPartnerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Referral => write!(f, "referral"),
            Self::Influencer => write!(f, "influencer"),
            Self::Affiliate => write!(f, "affiliate"),
            Self::Recruiter => write!(f, "recruiter"),
        }
    }
}

impl TryFrom<&str> for AmbassadorPartnerType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "referral" => Ok(Self::Referral),
            "influencer" => Ok(Self::Influencer),
            "affiliate" => Ok(Self::Affiliate),
            "recruiter" => Ok(Self::Recruiter),
            other => Err(format!("unknown AmbassadorPartnerType: {other}")),
        }
    }
}

impl TryFrom<String> for AmbassadorPartnerType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbassadorStatus {
    Active,
    Disabled,
}

impl fmt::Display for AmbassadorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Disabled => write!(f, "disabled"),
        }
    }
}

impl TryFrom<&str> for AmbassadorStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "disabled" => Ok(Self::Disabled),
            other => Err(format!("unknown AmbassadorStatus: {other}")),
        }
    }
}

impl TryFrom<String> for AmbassadorStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferAudience {
    Landlord,
    Vendor,
}

impl fmt::Display for ReferAudience {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Landlord => write!(f, "landlord"),
            Self::Vendor => write!(f, "vendor"),
        }
    }
}

impl TryFrom<&str> for ReferAudience {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "landlord" => Ok(Self::Landlord),
            "vendor" => Ok(Self::Vendor),
            other => Err(format!("unknown ReferAudience: {other}")),
        }
    }
}

impl TryFrom<String> for ReferAudience {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

/// Invite-out channel for F&F referrals (portal + platform-admin).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferralInviteChannel {
    Sms,
    Email,
}

impl fmt::Display for ReferralInviteChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sms => write!(f, "sms"),
            Self::Email => write!(f, "email"),
        }
    }
}

impl TryFrom<&str> for ReferralInviteChannel {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "sms" | "text" => Ok(Self::Sms),
            "email" | "mail" => Ok(Self::Email),
            other => Err(format!("unknown ReferralInviteChannel: {other}")),
        }
    }
}

impl TryFrom<String> for ReferralInviteChannel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbassadorFulfillmentKind {
    BusinessCards,
    Merch,
}

impl fmt::Display for AmbassadorFulfillmentKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BusinessCards => write!(f, "business_cards"),
            Self::Merch => write!(f, "merch"),
        }
    }
}

impl TryFrom<&str> for AmbassadorFulfillmentKind {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "business_cards" => Ok(Self::BusinessCards),
            "merch" => Ok(Self::Merch),
            other => Err(format!("unknown AmbassadorFulfillmentKind: {other}")),
        }
    }
}

impl TryFrom<String> for AmbassadorFulfillmentKind {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbassadorFulfillmentStatus {
    Requested,
    Cancelled,
    Fulfilled,
}

impl fmt::Display for AmbassadorFulfillmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Requested => write!(f, "requested"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Fulfilled => write!(f, "fulfilled"),
        }
    }
}

impl TryFrom<&str> for AmbassadorFulfillmentStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "requested" => Ok(Self::Requested),
            "cancelled" => Ok(Self::Cancelled),
            "fulfilled" => Ok(Self::Fulfilled),
            other => Err(format!("unknown AmbassadorFulfillmentStatus: {other}")),
        }
    }
}

impl TryFrom<String> for AmbassadorFulfillmentStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
    }
}

// ── EnrollmentStatus ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnrollmentStatus {
    /// Actively progressing through the sequence steps.
    Active,
    /// Manually paused by an operator.
    Paused,
    /// Completed all steps without exiting early.
    Completed,
    /// Triggered an exit condition (replied, converted, unsubscribed, manually removed).
    Exited,
    /// Email hard-bounced — contact removed from sequence.
    Bounced,
    /// Contact opted out.
    Unsubscribed,
}

impl fmt::Display for EnrollmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnrollmentStatus::Active => write!(f, "active"),
            EnrollmentStatus::Paused => write!(f, "paused"),
            EnrollmentStatus::Completed => write!(f, "completed"),
            EnrollmentStatus::Exited => write!(f, "exited"),
            EnrollmentStatus::Bounced => write!(f, "bounced"),
            EnrollmentStatus::Unsubscribed => write!(f, "unsubscribed"),
        }
    }
}

impl TryFrom<String> for EnrollmentStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "active" => Ok(EnrollmentStatus::Active),
            "paused" => Ok(EnrollmentStatus::Paused),
            "completed" => Ok(EnrollmentStatus::Completed),
            "exited" => Ok(EnrollmentStatus::Exited),
            "bounced" => Ok(EnrollmentStatus::Bounced),
            "unsubscribed" => Ok(EnrollmentStatus::Unsubscribed),
            other => Err(format!("unknown EnrollmentStatus: {other}")),
        }
    }
}

impl TryFrom<&str> for EnrollmentStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        EnrollmentStatus::try_from(s.to_string())
    }
}

// ── CampaignGoalType ──────────────────────────────────────────────────────────
//
// What a successful conversion creates. Determines `goal_entity_type` context
// and drives the routing of `conversion_entity_type` on enrollments.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignGoalType {
    /// Prospect fills in a form — creates an atlas_contact or atlas_lead record.
    LeadCapture,
    /// Prospect makes a reservation or purchase — creates an atlas_reservations record.
    Booking,
    /// Prospect submits an application — creates an atlas_applications record.
    Application,
    /// Prospect completes a purchase — creates an atlas_contracts / atlas_quotes record.
    Sale,
    /// Prospect registers for an event — creates an atlas_event_registrations record.
    Registration,
    /// Prospect enrolls in a subscription — creates an atlas_subscriptions record.
    Subscription,
    /// Prospect creates an account / signs up (G-36 program outcomes).
    Signup,
    /// Prospect finishes an onboarding wizard (G-36 program outcomes).
    OnboardingComplete,
}

impl fmt::Display for CampaignGoalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignGoalType::LeadCapture => write!(f, "lead_capture"),
            CampaignGoalType::Booking => write!(f, "booking"),
            CampaignGoalType::Application => write!(f, "application"),
            CampaignGoalType::Sale => write!(f, "sale"),
            CampaignGoalType::Registration => write!(f, "registration"),
            CampaignGoalType::Subscription => write!(f, "subscription"),
            CampaignGoalType::Signup => write!(f, "signup"),
            CampaignGoalType::OnboardingComplete => write!(f, "onboarding_complete"),
        }
    }
}

impl TryFrom<String> for CampaignGoalType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "lead_capture" => Ok(CampaignGoalType::LeadCapture),
            "booking" => Ok(CampaignGoalType::Booking),
            "application" => Ok(CampaignGoalType::Application),
            "sale" => Ok(CampaignGoalType::Sale),
            "registration" => Ok(CampaignGoalType::Registration),
            "subscription" => Ok(CampaignGoalType::Subscription),
            "signup" => Ok(CampaignGoalType::Signup),
            "onboarding_complete" => Ok(CampaignGoalType::OnboardingComplete),
            other => Err(format!("unknown CampaignGoalType: {other}")),
        }
    }
}

impl TryFrom<&str> for CampaignGoalType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CampaignGoalType::try_from(s.to_string())
    }
}

// ── SequenceStepType ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceStepType {
    /// Send an email (subject_template + body_template).
    Email,
    /// Send an SMS (body_template).
    Sms,
    /// Wait a fixed duration before next step.
    Wait,
    /// Branch on a condition (on_true_step / on_false_step).
    Condition,
    /// Assign a manual task to the sequence owner.
    Task,
    /// Send a LinkedIn connection request or message.
    Linkedin,
}

impl fmt::Display for SequenceStepType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SequenceStepType::Email => write!(f, "email"),
            SequenceStepType::Sms => write!(f, "sms"),
            SequenceStepType::Wait => write!(f, "wait"),
            SequenceStepType::Condition => write!(f, "condition"),
            SequenceStepType::Task => write!(f, "task"),
            SequenceStepType::Linkedin => write!(f, "linkedin"),
        }
    }
}

impl TryFrom<String> for SequenceStepType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "email" => Ok(SequenceStepType::Email),
            "sms" => Ok(SequenceStepType::Sms),
            "wait" => Ok(SequenceStepType::Wait),
            "condition" => Ok(SequenceStepType::Condition),
            "task" => Ok(SequenceStepType::Task),
            "linkedin" => Ok(SequenceStepType::Linkedin),
            other => Err(format!("unknown SequenceStepType: {other}")),
        }
    }
}

impl TryFrom<&str> for SequenceStepType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        SequenceStepType::try_from(s.to_string())
    }
}

// ── CampaignEventType ─────────────────────────────────────────────────────────
//
// The discrete event types recorded in `atlas_campaign_events`.
// `record_event()` in CampaignService matches on this enum to determine
// which counter to increment on the parent `atlas_campaigns` row.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignEventType {
    Sent,
    Delivered,
    Opened,
    Clicked,
    Replied,
    Bounced,
    Unsubscribed,
    SpamReported,
    /// Terminal event — triggers enrollment status → Exited + conversion tracking.
    Converted,
    /// A form on a landing page was filled out.
    FormFill,
    /// Direct-mail piece submitted to provider / mail house.
    MailSubmitted,
    /// Direct-mail piece marked delivered (USPS/Lob webhook).
    MailDelivered,
    /// Direct-mail piece returned undeliverable.
    MailReturned,
    /// Monetary cost incurred (increments campaign spent_cents).
    CostIncurred,
}

impl fmt::Display for CampaignEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignEventType::Sent => write!(f, "sent"),
            CampaignEventType::Delivered => write!(f, "delivered"),
            CampaignEventType::Opened => write!(f, "opened"),
            CampaignEventType::Clicked => write!(f, "clicked"),
            CampaignEventType::Replied => write!(f, "replied"),
            CampaignEventType::Bounced => write!(f, "bounced"),
            CampaignEventType::Unsubscribed => write!(f, "unsubscribed"),
            CampaignEventType::SpamReported => write!(f, "spam_reported"),
            CampaignEventType::Converted => write!(f, "converted"),
            CampaignEventType::FormFill => write!(f, "form_fill"),
            CampaignEventType::MailSubmitted => write!(f, "mail_submitted"),
            CampaignEventType::MailDelivered => write!(f, "mail_delivered"),
            CampaignEventType::MailReturned => write!(f, "mail_returned"),
            CampaignEventType::CostIncurred => write!(f, "cost_incurred"),
        }
    }
}

impl TryFrom<String> for CampaignEventType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "sent" => Ok(CampaignEventType::Sent),
            "delivered" => Ok(CampaignEventType::Delivered),
            "opened" => Ok(CampaignEventType::Opened),
            "clicked" => Ok(CampaignEventType::Clicked),
            "replied" => Ok(CampaignEventType::Replied),
            "bounced" => Ok(CampaignEventType::Bounced),
            "unsubscribed" => Ok(CampaignEventType::Unsubscribed),
            "spam_reported" => Ok(CampaignEventType::SpamReported),
            "converted" => Ok(CampaignEventType::Converted),
            "form_fill" => Ok(CampaignEventType::FormFill),
            "mail_submitted" => Ok(CampaignEventType::MailSubmitted),
            "mail_delivered" => Ok(CampaignEventType::MailDelivered),
            "mail_returned" => Ok(CampaignEventType::MailReturned),
            "cost_incurred" => Ok(CampaignEventType::CostIncurred),
            other => Err(format!("unknown CampaignEventType: {other}")),
        }
    }
}

impl TryFrom<&str> for CampaignEventType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CampaignEventType::try_from(s.to_string())
    }
}

// ── CampaignChannel ───────────────────────────────────────────────────────────
//
// The delivery channel for a campaign event. Different from `BookingChannel` —
// this describes HOW an event was delivered, not through which booking source
// the reservation arrived.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CampaignChannel {
    Email,
    Sms,
    /// Paid click on a Google/Meta/LinkedIn ad.
    PpcClick,
    /// Engagement from a social media post.
    Social,
    /// Interaction at or from a live event (open house, webinar, conference).
    Event,
    /// Referral link click.
    Referral,
    /// LinkedIn direct message or connection request.
    Linkedin,
    /// Physical direct mail (postcard, letter, flyer).
    DirectMail,
}

impl fmt::Display for CampaignChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CampaignChannel::Email => write!(f, "email"),
            CampaignChannel::Sms => write!(f, "sms"),
            CampaignChannel::PpcClick => write!(f, "ppc_click"),
            CampaignChannel::Social => write!(f, "social"),
            CampaignChannel::Event => write!(f, "event"),
            CampaignChannel::Referral => write!(f, "referral"),
            CampaignChannel::Linkedin => write!(f, "linkedin"),
            CampaignChannel::DirectMail => write!(f, "direct_mail"),
        }
    }
}

impl TryFrom<String> for CampaignChannel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "email" => Ok(CampaignChannel::Email),
            "sms" => Ok(CampaignChannel::Sms),
            "ppc_click" => Ok(CampaignChannel::PpcClick),
            "social" => Ok(CampaignChannel::Social),
            "event" => Ok(CampaignChannel::Event),
            "referral" => Ok(CampaignChannel::Referral),
            "linkedin" => Ok(CampaignChannel::Linkedin),
            "direct_mail" | "postcard" | "mail" => Ok(CampaignChannel::DirectMail),
            other => Err(format!("unknown CampaignChannel: {other}")),
        }
    }
}

impl TryFrom<&str> for CampaignChannel {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CampaignChannel::try_from(s.to_string())
    }
}

// ── G20 Attribution enums ─────────────────────────────────────────────────────

// ── AttributionChannel ────────────────────────────────────────────────────────
//
// The marketing channel that produced a touchpoint. Distinct from CampaignChannel
// (which describes HOW a campaign event was delivered) — this describes WHERE
// the visitor came from, using the standard channel taxonomy shared by GA4,
// HubSpot, and Bizible/Marketo Measure.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionChannel {
    /// Google/Bing organic search result.
    OrganicSearch,
    /// Google Ads, Bing Ads — CPC/CPM clicks.
    PaidSearch,
    /// Meta Ads, LinkedIn Ads, TikTok Ads, Twitter Ads.
    PaidSocial,
    /// Unpaid post on LinkedIn, Instagram, Twitter, Facebook, etc.
    OrganicSocial,
    /// Instantly.ai / cold outreach sequence email.
    ColdEmail,
    /// Referral link from a partner or affiliate.
    Referral,
    /// Event-driven touchpoint (open house, webinar, conference check-in).
    Event,
    /// Direct URL visit with no referrer.
    Direct,
    /// SMS campaign message.
    Sms,
    /// Blog post, SEO article, lead magnet download.
    Content,
    /// Affiliate program click.
    Affiliate,
    /// Physical direct mail (postcard / letter / flyer QR or typed URL).
    DirectMail,
}

impl fmt::Display for AttributionChannel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributionChannel::OrganicSearch => write!(f, "organic_search"),
            AttributionChannel::PaidSearch => write!(f, "paid_search"),
            AttributionChannel::PaidSocial => write!(f, "paid_social"),
            AttributionChannel::OrganicSocial => write!(f, "organic_social"),
            AttributionChannel::ColdEmail => write!(f, "cold_email"),
            AttributionChannel::Referral => write!(f, "referral"),
            AttributionChannel::Event => write!(f, "event"),
            AttributionChannel::Direct => write!(f, "direct"),
            AttributionChannel::Sms => write!(f, "sms"),
            AttributionChannel::Content => write!(f, "content"),
            AttributionChannel::Affiliate => write!(f, "affiliate"),
            AttributionChannel::DirectMail => write!(f, "direct_mail"),
        }
    }
}

impl TryFrom<String> for AttributionChannel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "organic_search" => Ok(AttributionChannel::OrganicSearch),
            "paid_search" => Ok(AttributionChannel::PaidSearch),
            "paid_social" => Ok(AttributionChannel::PaidSocial),
            "organic_social" => Ok(AttributionChannel::OrganicSocial),
            "cold_email" => Ok(AttributionChannel::ColdEmail),
            "referral" => Ok(AttributionChannel::Referral),
            "event" => Ok(AttributionChannel::Event),
            "direct" => Ok(AttributionChannel::Direct),
            "sms" => Ok(AttributionChannel::Sms),
            "content" => Ok(AttributionChannel::Content),
            "affiliate" => Ok(AttributionChannel::Affiliate),
            "direct_mail" | "postcard" | "mail" => Ok(AttributionChannel::DirectMail),
            other => Err(format!("unknown AttributionChannel: {other}")),
        }
    }
}

impl TryFrom<&str> for AttributionChannel {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        AttributionChannel::try_from(s.to_string())
    }
}

// ── AttributionModel ──────────────────────────────────────────────────────────
//
// The model used to distribute credit across all touchpoints in a conversion
// path. `record_conversion()` in AttributionService matches on this enum to
// determine the credit-distribution algorithm — no string-based dispatch.

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionModel {
    /// 100% credit to the first interaction in the path.
    FirstTouch,
    /// 100% credit to the last interaction before conversion.
    LastTouch,
    /// Equal credit distributed across all touchpoints.
    Linear,
    /// More credit to touchpoints closer in time to the conversion.
    TimeDecay,
    /// 40% first touch, 40% last touch, 20% distributed to middle touches.
    PositionBased,
}

impl fmt::Display for AttributionModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AttributionModel::FirstTouch => write!(f, "first_touch"),
            AttributionModel::LastTouch => write!(f, "last_touch"),
            AttributionModel::Linear => write!(f, "linear"),
            AttributionModel::TimeDecay => write!(f, "time_decay"),
            AttributionModel::PositionBased => write!(f, "position_based"),
        }
    }
}

impl TryFrom<String> for AttributionModel {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "first_touch" => Ok(AttributionModel::FirstTouch),
            "last_touch" => Ok(AttributionModel::LastTouch),
            "linear" => Ok(AttributionModel::Linear),
            "time_decay" => Ok(AttributionModel::TimeDecay),
            "position_based" => Ok(AttributionModel::PositionBased),
            other => Err(format!("unknown AttributionModel: {other}")),
        }
    }
}

impl TryFrom<&str> for AttributionModel {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        AttributionModel::try_from(s.to_string())
    }
}

// ── G21 Event enums ───────────────────────────────────────────────────────────

// ── EventType ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Real estate open house (PM app primary use case).
    OpenHouse,
    /// Online webinar / live stream.
    Webinar,
    /// Multi-day conference with sessions.
    Conference,
    /// Informal community meetup.
    Meetup,
    /// Professional training session or CE seminar (AgentLink, ClaimSwift).
    Training,
    /// Creator-hosted live experience (Famtasm).
    LiveExperience,
    /// Brand activation / pop-up event (Clipping Marketplace).
    BrandActivation,
    /// Hotel meeting or conference room booking (Direct Booking).
    VenueBooking,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::OpenHouse => write!(f, "open_house"),
            EventType::Webinar => write!(f, "webinar"),
            EventType::Conference => write!(f, "conference"),
            EventType::Meetup => write!(f, "meetup"),
            EventType::Training => write!(f, "training"),
            EventType::LiveExperience => write!(f, "live_experience"),
            EventType::BrandActivation => write!(f, "brand_activation"),
            EventType::VenueBooking => write!(f, "venue_booking"),
        }
    }
}

impl TryFrom<String> for EventType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "open_house" => Ok(EventType::OpenHouse),
            "webinar" => Ok(EventType::Webinar),
            "conference" => Ok(EventType::Conference),
            "meetup" => Ok(EventType::Meetup),
            "training" => Ok(EventType::Training),
            "live_experience" => Ok(EventType::LiveExperience),
            "brand_activation" => Ok(EventType::BrandActivation),
            "venue_booking" => Ok(EventType::VenueBooking),
            other => Err(format!("unknown EventType: {other}")),
        }
    }
}

impl TryFrom<&str> for EventType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        EventType::try_from(s.to_string())
    }
}

// ── EventStatus ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Draft,
    Published,
    /// Registration is open.
    Active,
    /// Registration closed but event has not yet occurred.
    RegistrationClosed,
    /// Event is in progress.
    InProgress,
    Completed,
    Cancelled,
}

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventStatus::Draft => write!(f, "draft"),
            EventStatus::Published => write!(f, "published"),
            EventStatus::Active => write!(f, "active"),
            EventStatus::RegistrationClosed => write!(f, "registration_closed"),
            EventStatus::InProgress => write!(f, "in_progress"),
            EventStatus::Completed => write!(f, "completed"),
            EventStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl TryFrom<String> for EventStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(EventStatus::Draft),
            "published" => Ok(EventStatus::Published),
            "active" => Ok(EventStatus::Active),
            "registration_closed" => Ok(EventStatus::RegistrationClosed),
            "in_progress" => Ok(EventStatus::InProgress),
            "completed" => Ok(EventStatus::Completed),
            "cancelled" => Ok(EventStatus::Cancelled),
            other => Err(format!("unknown EventStatus: {other}")),
        }
    }
}

impl TryFrom<&str> for EventStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        EventStatus::try_from(s.to_string())
    }
}

// ── RegistrationStatus ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationStatus {
    /// Awaiting payment confirmation (for paid tickets).
    PendingPayment,
    Confirmed,
    /// Event is at capacity; attendee is on the waitlist.
    Waitlisted,
    Cancelled,
    /// Attendee was scanned in at the event.
    CheckedIn,
    /// Registered but did not attend.
    NoShow,
}

impl fmt::Display for RegistrationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegistrationStatus::PendingPayment => write!(f, "pending_payment"),
            RegistrationStatus::Confirmed => write!(f, "confirmed"),
            RegistrationStatus::Waitlisted => write!(f, "waitlisted"),
            RegistrationStatus::Cancelled => write!(f, "cancelled"),
            RegistrationStatus::CheckedIn => write!(f, "checked_in"),
            RegistrationStatus::NoShow => write!(f, "no_show"),
        }
    }
}

impl TryFrom<String> for RegistrationStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "pending_payment" => Ok(RegistrationStatus::PendingPayment),
            "confirmed" => Ok(RegistrationStatus::Confirmed),
            "waitlisted" => Ok(RegistrationStatus::Waitlisted),
            "cancelled" => Ok(RegistrationStatus::Cancelled),
            "checked_in" => Ok(RegistrationStatus::CheckedIn),
            "no_show" => Ok(RegistrationStatus::NoShow),
            other => Err(format!("unknown RegistrationStatus: {other}")),
        }
    }
}

impl TryFrom<&str> for RegistrationStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        RegistrationStatus::try_from(s.to_string())
    }
}

// ── G24 Quote enums ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteStatus {
    /// Editable, not yet sent to the prospect.
    Draft,
    /// Delivered to the prospect; awaiting response.
    Sent,
    /// Prospect accepted — triggers reservation creation.
    Accepted,
    Rejected,
    /// Validity window has passed with no response.
    Expired,
    /// Quote was converted to a booking/contract.
    Converted,
    /// Superseded by a newer revision.
    Superseded,
}

impl fmt::Display for QuoteStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteStatus::Draft => write!(f, "draft"),
            QuoteStatus::Sent => write!(f, "sent"),
            QuoteStatus::Accepted => write!(f, "accepted"),
            QuoteStatus::Rejected => write!(f, "rejected"),
            QuoteStatus::Expired => write!(f, "expired"),
            QuoteStatus::Converted => write!(f, "converted"),
            QuoteStatus::Superseded => write!(f, "superseded"),
        }
    }
}

impl TryFrom<String> for QuoteStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(QuoteStatus::Draft),
            "sent" => Ok(QuoteStatus::Sent),
            "accepted" => Ok(QuoteStatus::Accepted),
            "rejected" => Ok(QuoteStatus::Rejected),
            "expired" => Ok(QuoteStatus::Expired),
            "converted" => Ok(QuoteStatus::Converted),
            "superseded" => Ok(QuoteStatus::Superseded),
            other => Err(format!("unknown QuoteStatus: {other}")),
        }
    }
}

impl TryFrom<&str> for QuoteStatus {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        QuoteStatus::try_from(s.to_string())
    }
}

/// The type of a line item within a quote.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteLineItemType {
    /// A catalog product (linked to G26 atlas_catalog_entries).
    Product,
    /// A billable service.
    Service,
    /// A flat fee (setup, admin, processing).
    Fee,
    /// A tax line (linked to G17 atlas_tax).
    Tax,
    /// A monetary discount (negative amount).
    Discount,
    /// A percentage-based discount.
    PercentageDiscount,
    /// A custom one-off line.
    Custom,
}

impl fmt::Display for QuoteLineItemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteLineItemType::Product => write!(f, "product"),
            QuoteLineItemType::Service => write!(f, "service"),
            QuoteLineItemType::Fee => write!(f, "fee"),
            QuoteLineItemType::Tax => write!(f, "tax"),
            QuoteLineItemType::Discount => write!(f, "discount"),
            QuoteLineItemType::PercentageDiscount => write!(f, "percentage_discount"),
            QuoteLineItemType::Custom => write!(f, "custom"),
        }
    }
}

impl TryFrom<String> for QuoteLineItemType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "product" => Ok(QuoteLineItemType::Product),
            "service" => Ok(QuoteLineItemType::Service),
            "fee" => Ok(QuoteLineItemType::Fee),
            "tax" => Ok(QuoteLineItemType::Tax),
            "discount" => Ok(QuoteLineItemType::Discount),
            "percentage_discount" => Ok(QuoteLineItemType::PercentageDiscount),
            "custom" => Ok(QuoteLineItemType::Custom),
            other => Err(format!("unknown QuoteLineItemType: {other}")),
        }
    }
}

impl TryFrom<&str> for QuoteLineItemType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        QuoteLineItemType::try_from(s.to_string())
    }
}

// ── G15 Opportunity enums ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityStage {
    /// First contact / awareness.
    Prospecting,
    /// Needs analysis / discovery call completed.
    Qualified,
    /// Proposal sent or demo scheduled.
    Proposal,
    /// Commercial negotiation in progress.
    Negotiation,
    /// Deal closed and won.
    ClosedWon,
    /// Deal closed and lost.
    ClosedLost,
    /// Deal on hold pending external event.
    OnHold,
}

impl fmt::Display for OpportunityStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpportunityStage::Prospecting => write!(f, "prospecting"),
            OpportunityStage::Qualified => write!(f, "qualified"),
            OpportunityStage::Proposal => write!(f, "proposal"),
            OpportunityStage::Negotiation => write!(f, "negotiation"),
            OpportunityStage::ClosedWon => write!(f, "closed_won"),
            OpportunityStage::ClosedLost => write!(f, "closed_lost"),
            OpportunityStage::OnHold => write!(f, "on_hold"),
        }
    }
}

impl TryFrom<String> for OpportunityStage {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "prospecting" => Ok(OpportunityStage::Prospecting),
            "qualified" => Ok(OpportunityStage::Qualified),
            "proposal" => Ok(OpportunityStage::Proposal),
            "negotiation" => Ok(OpportunityStage::Negotiation),
            "closed_won" => Ok(OpportunityStage::ClosedWon),
            "closed_lost" => Ok(OpportunityStage::ClosedLost),
            "on_hold" => Ok(OpportunityStage::OnHold),
            other => Err(format!("unknown OpportunityStage: {other}")),
        }
    }
}

impl TryFrom<&str> for OpportunityStage {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        OpportunityStage::try_from(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpportunityType {
    /// New customer acquisition.
    NewBusiness,
    /// Expansion within an existing account.
    Upsell,
    /// Cross-sell of a different product.
    CrossSell,
    /// Contract or subscription renewal.
    Renewal,
    /// Win-back of a churned customer.
    Winback,
}

impl fmt::Display for OpportunityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpportunityType::NewBusiness => write!(f, "new_business"),
            OpportunityType::Upsell => write!(f, "upsell"),
            OpportunityType::CrossSell => write!(f, "cross_sell"),
            OpportunityType::Renewal => write!(f, "renewal"),
            OpportunityType::Winback => write!(f, "winback"),
        }
    }
}

impl TryFrom<String> for OpportunityType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "new_business" => Ok(OpportunityType::NewBusiness),
            "upsell" => Ok(OpportunityType::Upsell),
            "cross_sell" => Ok(OpportunityType::CrossSell),
            "renewal" => Ok(OpportunityType::Renewal),
            "winback" => Ok(OpportunityType::Winback),
            other => Err(format!("unknown OpportunityType: {other}")),
        }
    }
}

impl TryFrom<&str> for OpportunityType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        OpportunityType::try_from(s.to_string())
    }
}

// ── G25 Commission enums ──────────────────────────────────────────────────────

/// How commission is calculated — the basis for the rate.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommissionBasis {
    /// Fixed flat fee per transaction, regardless of size.
    FlatFee,
    /// Percentage of gross transaction value.
    GrossPercentage,
    /// Percentage of net transaction value (after deductions).
    NetPercentage,
    /// Tiered rate based on transaction volume brackets.
    Tiered,
    /// Percentage only applied above a minimum threshold.
    SplitAboveThreshold,
}

impl fmt::Display for CommissionBasis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommissionBasis::FlatFee => write!(f, "flat_fee"),
            CommissionBasis::GrossPercentage => write!(f, "gross_percentage"),
            CommissionBasis::NetPercentage => write!(f, "net_percentage"),
            CommissionBasis::Tiered => write!(f, "tiered"),
            CommissionBasis::SplitAboveThreshold => write!(f, "split_above_threshold"),
        }
    }
}

impl TryFrom<String> for CommissionBasis {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "flat_fee" => Ok(CommissionBasis::FlatFee),
            "gross_percentage" => Ok(CommissionBasis::GrossPercentage),
            "net_percentage" => Ok(CommissionBasis::NetPercentage),
            "tiered" => Ok(CommissionBasis::Tiered),
            "split_above_threshold" => Ok(CommissionBasis::SplitAboveThreshold),
            other => Err(format!("unknown CommissionBasis: {other}")),
        }
    }
}

impl TryFrom<&str> for CommissionBasis {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CommissionBasis::try_from(s.to_string())
    }
}

/// Whether a commission plan has an earnings cap.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommissionCapType {
    /// No cap — unlimited earnings.
    None,
    /// Cap applied per individual transaction.
    PerTransaction,
    /// Cap applied per calendar month.
    Monthly,
    /// Cap applied per calendar year.
    Annual,
}

impl fmt::Display for CommissionCapType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommissionCapType::None => write!(f, "none"),
            CommissionCapType::PerTransaction => write!(f, "per_transaction"),
            CommissionCapType::Monthly => write!(f, "monthly"),
            CommissionCapType::Annual => write!(f, "annual"),
        }
    }
}

impl TryFrom<String> for CommissionCapType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "none" => Ok(CommissionCapType::None),
            "per_transaction" => Ok(CommissionCapType::PerTransaction),
            "monthly" => Ok(CommissionCapType::Monthly),
            "annual" => Ok(CommissionCapType::Annual),
            other => Err(format!("unknown CommissionCapType: {other}")),
        }
    }
}

impl TryFrom<&str> for CommissionCapType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        CommissionCapType::try_from(s.to_string())
    }
}

// ── G-36 atlas_programs ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramKind {
    NetworkInvite,
    Referral,
    ReviewRequest,
    WaitlistAccess,
    LeadCapture,
    PartnerShare,
}

impl fmt::Display for ProgramKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NetworkInvite => "network_invite",
            Self::Referral => "referral",
            Self::ReviewRequest => "review_request",
            Self::WaitlistAccess => "waitlist_access",
            Self::LeadCapture => "lead_capture",
            Self::PartnerShare => "partner_share",
        })
    }
}

impl TryFrom<&str> for ProgramKind {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "network_invite" => Ok(Self::NetworkInvite),
            "referral" => Ok(Self::Referral),
            "review_request" => Ok(Self::ReviewRequest),
            "waitlist_access" => Ok(Self::WaitlistAccess),
            "lead_capture" => Ok(Self::LeadCapture),
            "partner_share" => Ok(Self::PartnerShare),
            other => Err(format!("unknown ProgramKind: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramOutcomeType {
    Signup,
    WizardComplete,
    FormSubmit,
    ReviewSubmitted,
    FirstJobLogged,
    SubscriptionActivated,
}

impl fmt::Display for ProgramOutcomeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Signup => "signup",
            Self::WizardComplete => "wizard_complete",
            Self::FormSubmit => "form_submit",
            Self::ReviewSubmitted => "review_submitted",
            Self::FirstJobLogged => "first_job_logged",
            Self::SubscriptionActivated => "subscription_activated",
        })
    }
}

impl TryFrom<&str> for ProgramOutcomeType {
    type Error = String;
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "signup" => Ok(Self::Signup),
            "wizard_complete" => Ok(Self::WizardComplete),
            "form_submit" => Ok(Self::FormSubmit),
            "review_submitted" => Ok(Self::ReviewSubmitted),
            "first_job_logged" => Ok(Self::FirstJobLogged),
            "subscription_activated" => Ok(Self::SubscriptionActivated),
            other => Err(format!("unknown ProgramOutcomeType: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramActionStatus {
    Created,
    Sent,
    Opened,
    Accepted,
    OutcomeComplete,
    Expired,
    Revoked,
}

impl fmt::Display for ProgramActionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Created => "created",
            Self::Sent => "sent",
            Self::Opened => "opened",
            Self::Accepted => "accepted",
            Self::OutcomeComplete => "outcome_complete",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRewardBeneficiary {
    Actor,
    Target,
}

impl fmt::Display for ProgramRewardBeneficiary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Actor => "actor",
            Self::Target => "target",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRewardType {
    SubscriptionCreditDays,
    FeatureUnlock,
    None,
}

impl fmt::Display for ProgramRewardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::SubscriptionCreditDays => "subscription_credit_days",
            Self::FeatureUnlock => "feature_unlock",
            Self::None => "none",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramRewardGrantStatus {
    Pending,
    Granted,
    Applied,
    Revoked,
}

impl fmt::Display for ProgramRewardGrantStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pending => "pending",
            Self::Granted => "granted",
            Self::Applied => "applied",
            Self::Revoked => "revoked",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramOutcomeStatus {
    Pending,
    Completed,
    Failed,
}

impl fmt::Display for ProgramOutcomeStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
        })
    }
}
