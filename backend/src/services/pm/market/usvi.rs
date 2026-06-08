//! US Virgin Islands market configuration.
//!
//! FHA applies (USVI is a US territory).
//! USVI Hotel Room Tax: 12.5%.
//! No island-level STR ordinance configured yet — Phase 5.

use rust_decimal::Decimal;
use std::str::FromStr;
use crate::types::pm::{
    Jurisdiction, Currency, TaxRate, StatuteRef,
    GuaranteeType,
};
use crate::services::pm::market::market_config::{
    MarketConfig, TenancyLaw, AntiDiscriminationLaw, TaxEngine, CreditBureau,
};
// FairHousingAct and TransUnionBureau are stateless zero-size types —
// imported directly from miami.rs with no duplication.
use crate::services::pm::market::miami::{FairHousingAct, TransUnionBureau};

// ── Tax: USVI Hotel Room Tax ──────────────────────────────────────────────────

pub struct UsviHotelRoomTax;

impl TaxEngine for UsviHotelRoomTax {
    fn str_tax_rate(&self) -> Option<TaxRate> {
        Some(TaxRate {
            rate: Decimal::from_str("0.125").unwrap(),
            remittance_currency: Currency::Usd,
            ota_collects: true,
            label: "USVI Hotel Room Tax (12.5%)",
        })
    }

    fn rental_withholding_rate(&self) -> Option<TaxRate> { None }
    fn remittance_currency(&self) -> Currency { Currency::Usd }
}

// ── Tenancy Law: USVI Code Title 28 ──────────────────────────────────────────

pub struct UsviTenancyLaw;

static USVI_GUARANTEES: &[GuaranteeType] = &[GuaranteeType::Caucao, GuaranteeType::None];

impl TenancyLaw for UsviTenancyLaw {
    fn statute(&self) -> StatuteRef {
        StatuteRef {
            code: "V.I. Code tit. 28, § 311",
            name: "USVI Landlord-Tenant Relations",
            country: Jurisdiction::Vi,
            url: None,
        }
    }

    // No override of classify_expense — inherits None (no condomínio system in USVI).

    fn max_deposit_months(&self) -> Option<u8> { Some(2) } // V.I. Code tit. 28 § 311
    fn landlord_termination_notice_days(&self) -> u32 { 30 }
    fn tenant_vacate_notice_days(&self) -> u32 { 30 }
    fn allowed_guarantee_types(&self) -> &'static [GuaranteeType] { USVI_GUARANTEES }
}

// ── Root: UsViMarket ──────────────────────────────────────────────────────────

pub struct UsViMarket;

impl MarketConfig for UsViMarket {
    fn jurisdiction(&self) -> Jurisdiction { Jurisdiction::Vi }
    fn default_currency(&self) -> Currency { Currency::Usd }
    fn display_name(&self) -> &'static str { "US Virgin Islands" }

    fn tenancy_law(&self) -> Option<&dyn TenancyLaw> { Some(&UsviTenancyLaw) }

    // FHA applies in USVI — reuse Miami's implementation directly.
    fn anti_discrimination_law(&self) -> Option<&dyn AntiDiscriminationLaw> {
        Some(&FairHousingAct)
    }

    // No island-level STR ordinance — Phase 5.
    fn str_regulation(&self) -> Option<&dyn crate::services::pm::market::market_config::StrRegulation> { None }

    fn tax_engine(&self) -> &dyn TaxEngine { &UsviHotelRoomTax }
    fn credit_bureau(&self) -> &dyn CreditBureau { &TransUnionBureau }
}
