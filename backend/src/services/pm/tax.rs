//! Folio — Tax Service (PM wrapper over G-17 `atlas_tax_events`)
//!
//! TDT (Tourist Development Tax) calculation for STR revenue.
//! Monthly reconciliation, OTA revenue ingestion hooks.
//!
//! # TDT rate lookup
//!
//! TDT rates are market-specific:
//!   - Miami-Dade: 7% (Miami-Dade County Ordinance, per `MiamiDadeMarket`)
//!   - USVI: 12.5% (USVI Hotel Room Tax, per `UsViMarket`)
//!   - Brazil: IRRF withheld at source by OTA; no TDT concept, rate = 0.0
//!
//! The rate is resolved by reading `folio_jurisdiction_code` from `tenant_setting`
//! and dispatching to `MarketRegistry::resolve(jurisdiction_code).tax_rates()`.
//!
//! # `atlas_tax_events` field map (G-17)
//!
//! | Field                  | Value for OTA revenue                        |
//! |------------------------|----------------------------------------------|
//! | `tax_type`             | `"TDT"` (Tourist Development Tax)            |
//! | `jurisdiction_code`    | from tenant setting or asset jurisdiction    |
//! | `source_entity_type`   | `"atlas_asset"` (the STR property)           |
//! | `source_entity_id`     | `asset_id`                                   |
//! | `gross_revenue_cents`  | OTA payout before OTA fees                   |
//! | `excluded_fees_cents`  | OTA platform fees (sourced from config)      |
//! | `taxable_revenue_cents`| gross - excluded_fees                        |
//! | `tax_rate`             | market TDT rate (e.g. 0.07 for Miami)        |
//! | `tax_amount_cents`     | taxable_revenue_cents × tax_rate             |
//! | `remitted_by`          | `"operator"` (landlord remits TDT monthly)   |
//! | `event_date`           | period_start (booking check-in date)         |

use anyhow::{Result, anyhow};
use chrono::NaiveDate;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

pub struct PmTaxService;

impl PmTaxService {
    /// Record an OTA revenue event for TDT calculation.
    ///
    /// # Arguments
    /// - `asset_id`              — The STR property that generated the revenue.
    /// - `gross_revenue_cents`   — OTA payout in smallest currency unit (before OTA fees).
    /// - `currency_code`         — ISO-4217, e.g. "USD", "BRL".
    /// - `period_start`          — Check-in date of the booking (TDT event date).
    /// - `ota_fee_cents`         — OTA platform fee already deducted (excluded from TDT base).
    ///                             If unknown, pass 0.
    /// - `source_integration_id` — FK to the atlas_external_integration row for this OTA.
    ///                             `None` for manual imports.
    ///
    /// Returns the `atlas_tax_events.id` of the created row.
    /// Simplified OTA revenue recording by reservation ID.
    ///
    /// Called from `ReservationService::confirm()` — records TDT at booking confirmation
    /// time without requiring the caller to know the asset_id or period_start.
    /// Uses `event_date = today` and zero excluded fees (gross = taxable).
    pub async fn record_ota_revenue_simple(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        reservation_id: Uuid,
        gross_revenue_cents: i64,
        _currency_code: &str, // atlas_tax_event has no currency column — stored in TDT rate resolution
        jurisdiction_code: &str,
    ) -> Result<Uuid> {
        use chrono::Utc;

        // Resolve TDT rate from market config.
        let tdt_rate = Self::resolve_tdt_rate(jurisdiction_code);
        let tdt_cents = (gross_revenue_cents as f64 * tdt_rate).round() as i64;
        let today = Utc::now().date_naive();
        let id = Uuid::new_v4();

        use sea_orm::{ActiveModelTrait, Set};

        let active = crate::entities::atlas_tax_event::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            tax_type: Set("tdt".to_string()),
            jurisdiction_code: Set(jurisdiction_code.to_string()),
            source_integration_id: Set(None),
            source_ledger_entry_id: Set(None),
            source_entity_type: Set(Some("atlas_reservations".to_string())),
            source_entity_id: Set(Some(reservation_id)),
            gross_revenue_cents: Set(gross_revenue_cents),
            excluded_fees_cents: Set(0),
            taxable_revenue_cents: Set(gross_revenue_cents),
            tax_rate: Set(tdt_rate),
            tax_amount_cents: Set(tdt_cents),
            remitted_by: Set("platform".to_string()),
            tax_filing_id: Set(None),
            event_date: Set(today),
            created_at: Set(Utc::now()),
        };

        active.insert(db).await.map_err(|e| {
            anyhow::anyhow!(
                "record_ota_revenue_simple: DB insert failed for reservation {reservation_id}: {e}"
            )
        })?;

        tracing::info!(
            %tenant_id, %reservation_id,
            tdt_cents, jurisdiction_code,
            "record_ota_revenue_simple: TDT obligation recorded"
        );

        Ok(id)
    }

    pub async fn record_ota_revenue(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        gross_revenue_cents: i64,
        currency_code: &str,
        period_start: NaiveDate,
    ) -> Result<Uuid> {
        Self::record_ota_revenue_full(
            db,
            tenant_id,
            asset_id,
            gross_revenue_cents,
            currency_code,
            period_start,
            0,    // ota_fee_cents defaults to 0 for backward compat
            None, // no integration ID
        )
        .await
    }

    /// Full OTA revenue recording with OTA fee deduction and integration source tracking.
    pub async fn record_ota_revenue_full(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        gross_revenue_cents: i64,
        currency_code: &str,
        period_start: NaiveDate,
        ota_fee_cents: i64,
        source_integration_id: Option<Uuid>,
    ) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

        // ── 1. Resolve jurisdiction code for this tenant ──────────────────────
        let jurisdiction_code = {
            let setting = crate::entities::tenant_setting::Entity::find()
                .filter(crate::entities::tenant_setting::Column::TenantId.eq(tenant_id))
                .filter(crate::entities::tenant_setting::Column::Key.eq("folio_jurisdiction_code"))
                .one(db)
                .await?;

            setting.map(|s| s.value).unwrap_or_else(|| "US".to_string())
        };

        // ── 2. Resolve TDT rate from market registry ──────────────────────────
        // MarketRegistry maps jurisdiction codes to market implementations.
        // Each market's tax_rates() returns the TDT rate for STR.
        let tdt_rate = Self::resolve_tdt_rate(&jurisdiction_code);

        // ── 3. Compute TDT ────────────────────────────────────────────────────
        let excluded_fees_cents = ota_fee_cents.max(0);
        let taxable_revenue_cents = (gross_revenue_cents - excluded_fees_cents).max(0);
        let tax_amount_cents = (taxable_revenue_cents as f64 * tdt_rate).round() as i64;

        // ── 4. Insert atlas_tax_events row ────────────────────────────────────
        let id = Uuid::new_v4();
        let now = Utc::now();

        let tax_event = crate::entities::atlas_tax_event::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            tax_type: Set("TDT".to_string()),
            jurisdiction_code: Set(jurisdiction_code.clone()),
            source_integration_id: Set(source_integration_id),
            source_ledger_entry_id: Set(None), // OTA revenue is not a PM ledger event
            source_entity_type: Set(Some("atlas_asset".to_string())),
            source_entity_id: Set(Some(asset_id)),
            gross_revenue_cents: Set(gross_revenue_cents),
            excluded_fees_cents: Set(excluded_fees_cents),
            taxable_revenue_cents: Set(taxable_revenue_cents),
            tax_rate: Set(tdt_rate),
            tax_amount_cents: Set(tax_amount_cents),
            remitted_by: Set("operator".to_string()),
            tax_filing_id: Set(None), // populated when operator files TDT return
            event_date: Set(period_start),
            created_at: Set(now),
        };

        tax_event.insert(db).await.map_err(|e| {
            anyhow!(
                "PmTaxService::record_ota_revenue_full: DB insert failed for tenant {tenant_id}: {e}"
            )
        })?;

        tracing::info!(
            tax_event_id = %id,
            %tenant_id,
            %asset_id,
            %jurisdiction_code,
            gross_revenue_cents,
            excluded_fees_cents,
            taxable_revenue_cents,
            tdt_rate,
            tax_amount_cents,
            currency_code,
            "PmTaxService: OTA revenue recorded with TDT"
        );

        Ok(id)
    }

    /// Resolve the Tourist Development Tax rate for a jurisdiction code string.
    ///
    /// Rates are sourced from each market's `tax_engine().str_tax_rate()`.
    /// Jurisdictions without a TDT (e.g. Brazil — IRRF withheld by OTA) return 0.0.
    ///
    /// | Jurisdiction | Rate  | Authority                         |
    /// |--------------|-------|-----------------------------------|
    /// | Us (Miami)   | 0.07  | Miami-Dade County Ordinance       |
    /// | Vi (USVI)    | 0.125 | USVI Hotel Room Tax               |
    /// | Br           | 0.00  | IRRF withheld at source by OTA    |
    pub fn resolve_tdt_rate(jurisdiction_code: &str) -> f64 {
        use crate::types::pm::Jurisdiction;
        use rust_decimal::prelude::ToPrimitive;

        let registry = crate::services::pm::market::market_config::MarketRegistry::build();

        let jurisdiction = match Jurisdiction::try_from(jurisdiction_code.to_string()) {
            Ok(j) => j,
            Err(_) => {
                tracing::warn!(
                    jurisdiction_code,
                    "PmTaxService: unknown jurisdiction for TDT rate — defaulting to 0.0"
                );
                return 0.0;
            }
        };

        match registry.resolve(&jurisdiction) {
            Ok(market) => market
                .tax_engine()
                .str_tax_rate()
                .map(|r| r.rate.to_f64().unwrap_or(0.0))
                .unwrap_or(0.0),
            Err(e) => {
                tracing::warn!(
                    jurisdiction_code,
                    "PmTaxService: no market config for jurisdiction: {e} — defaulting to 0.0"
                );
                0.0
            }
        }
    }
}
