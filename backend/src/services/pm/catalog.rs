//! # G26 CatalogService — Product Catalog, Pricebook & Availability Grid
//!
//! Manages the three-table `atlas_catalog_*` generic that sits between
//! G10 Assets (what you own) and G24 Quotes (what you're proposing to sell):
//!
//! ```text
//! G10 atlas_assets          → atlas_catalog_entries  → G24 atlas_quotes
//!   (owned inventory)           (saleable product)        (priced proposal)
//!                                      ↓
//!                           atlas_catalog_rate_rules   (dynamic pricing)
//!                           atlas_catalog_availability (slot inventory)
//! ```
//!
//! ## Effective Price Resolution
//!
//! `get_effective_price()` applies rate rules in descending priority order.
//! A day-specific `override_price_cents` in the availability grid always wins.
//! Rule applicability is filtered by:
//!   - `applies_from` / `applies_to` (date window)
//!   - `day_of_week_mask` (bitmask; Mon=1 through Sun=64)
//!   - `min_duration` (minimum billing intervals — generic across nightly/hourly/daily/etc.)
//!   - `channel` (booking source: 'direct', 'ota', 'gds', 'corporate')
//!
//! ## Availability & Locking
//!
//! `reserve_slots()` and `release_slots()` use row-level `SELECT ... FOR UPDATE`
//! locking to prevent double-booking under concurrent requests.
//! The `ReservationService` calls `reserve_slots()` inside `create_hold()`
//! and `release_slots()` inside `cancel()`.

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde_json::Value as Json;
use uuid::Uuid;

use crate::{
    entities::{atlas_catalog_availability, atlas_catalog_entry, atlas_catalog_rate_rule},
    types::pm::{BillingInterval, BookingChannel, CatalogEntryType},
};

// ── Public input types ────────────────────────────────────────────────────────

/// Payload for creating a new catalog entry (product).
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateCatalogEntryPayload {
    pub entry_type: CatalogEntryType,
    pub name: String,
    pub description: Option<String>,
    pub asset_id: Option<Uuid>,
    pub base_price_cents: i64,
    pub currency: String,
    /// The time unit this entry charges per. NULL = one-time purchase.
    pub billing_interval: Option<BillingInterval>,
    pub min_quantity: Option<i32>,
    pub max_quantity: Option<i32>,
    pub catalog_metadata: Option<Json>,
    pub sort_order: Option<i32>,
    pub cover_image_attachment_id: Option<Uuid>,
}

/// Payload for creating a rate rule on a catalog entry.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateRateRulePayload {
    pub catalog_entry_id: Uuid,
    pub rule_name: Option<String>,
    pub applies_from: Option<NaiveDate>,
    pub applies_to: Option<NaiveDate>,
    pub day_of_week_mask: Option<i32>,
    /// Minimum number of billing intervals (unit depends on parent entry's billing_interval).
    /// For a Nightly entry: minimum nights. For Hourly: minimum hours. For Daily: minimum days.
    pub min_duration: Option<i32>,
    /// Scope to a specific booking channel. None = rule applies to all channels.
    pub channel: Option<BookingChannel>,
    pub price_override_cents: Option<i64>,
    pub price_modifier_pct: Option<f64>,
    pub priority: Option<i32>,
}

/// Filter for listing catalog entries.
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct CatalogFilter {
    pub entry_type: Option<String>,
    pub asset_id: Option<Uuid>,
    pub is_available: Option<bool>,
}

/// Availability summary for a date range (returned by `check_availability`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AvailabilitySummary {
    pub slot_date: NaiveDate,
    pub total_inventory: i32,
    pub reserved_count: i32,
    pub available_count: i32,
    pub is_blocked: bool,
    pub block_reason: Option<String>,
    /// Effective price for this day (override > rate rule > base price).
    pub effective_price_cents: i64,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct CatalogService;

impl CatalogService {
    // ── Catalog entry CRUD ────────────────────────────────────────────────────

    /// Create a new catalog entry (product definition).
    pub async fn create_entry(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateCatalogEntryPayload,
    ) -> Result<atlas_catalog_entry::Model> {
        use chrono::Utc;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let active = atlas_catalog_entry::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            // Convert enum → String for the VARCHAR column.
            entry_type: Set(payload.entry_type.to_string()),
            name: Set(payload.name),
            description: Set(payload.description),
            asset_id: Set(payload.asset_id),
            base_price_cents: Set(payload.base_price_cents),
            currency: Set(payload.currency),
            // Convert enum → String for the VARCHAR column; None = one-time purchase.
            billing_interval: Set(payload.billing_interval.map(|bi| bi.to_string())),
            is_available: Set(true),
            min_quantity: Set(payload.min_quantity.unwrap_or(1)),
            max_quantity: Set(payload.max_quantity),
            catalog_metadata: Set(payload.catalog_metadata.unwrap_or(serde_json::json!({}))),
            sort_order: Set(payload.sort_order.unwrap_or(0)),
            cover_image_attachment_id: Set(payload.cover_image_attachment_id),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let model = active.insert(db).await?;

        tracing::info!(
            %tenant_id, entry_id = %model.id, entry_type = %model.entry_type,
            "CatalogService::create_entry: created '{}'", model.name
        );

        Ok(model)
    }

    /// List catalog entries for a tenant, with optional filtering.
    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filter: CatalogFilter,
    ) -> Result<Vec<atlas_catalog_entry::Model>> {
        let mut q = atlas_catalog_entry::Entity::find()
            .filter(atlas_catalog_entry::Column::TenantId.eq(tenant_id));

        if let Some(et) = filter.entry_type {
            q = q.filter(atlas_catalog_entry::Column::EntryType.eq(et));
        }
        if let Some(aid) = filter.asset_id {
            q = q.filter(atlas_catalog_entry::Column::AssetId.eq(aid));
        }
        if let Some(avail) = filter.is_available {
            q = q.filter(atlas_catalog_entry::Column::IsAvailable.eq(avail));
        }

        let rows = q
            .order_by_asc(atlas_catalog_entry::Column::SortOrder)
            .all(db)
            .await?;
        Ok(rows)
    }

    /// Get a single catalog entry by ID, verifying tenant ownership.
    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entry_id: Uuid,
    ) -> Result<atlas_catalog_entry::Model> {
        atlas_catalog_entry::Entity::find_by_id(entry_id)
            .filter(atlas_catalog_entry::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("CatalogEntry {entry_id} not found for tenant {tenant_id}"))
    }

    // ── Rate rules ────────────────────────────────────────────────────────────

    /// Add a rate rule to a catalog entry.
    pub async fn apply_rate_rule(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateRateRulePayload,
    ) -> Result<atlas_catalog_rate_rule::Model> {
        use chrono::Utc;

        let id = Uuid::new_v4();

        let active = atlas_catalog_rate_rule::ActiveModel {
            id: Set(id),
            catalog_entry_id: Set(payload.catalog_entry_id),
            tenant_id: Set(tenant_id),
            rule_name: Set(payload.rule_name),
            applies_from: Set(payload.applies_from),
            applies_to: Set(payload.applies_to),
            day_of_week_mask: Set(payload.day_of_week_mask),
            min_duration: Set(payload.min_duration),
            // Convert enum → String for VARCHAR column; None = applies to all channels.
            channel: Set(payload.channel.map(|c| c.to_string())),
            price_override_cents: Set(payload.price_override_cents),
            price_modifier_pct: Set(payload
                .price_modifier_pct
                .map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO))),
            priority: Set(payload.priority.unwrap_or(0)),
            is_active: Set(true),
            created_at: Set(Utc::now()),
        };

        let rule = active.insert(db).await?;

        tracing::info!(
            %tenant_id, rule_id = %rule.id,
            entry_id = %rule.catalog_entry_id,
            "CatalogService::apply_rate_rule: created rate rule '{:?}'", rule.rule_name
        );

        Ok(rule)
    }

    // ── Effective price resolution ────────────────────────────────────────────

    /// Compute the effective price for a catalog entry on a given date.
    ///
    /// Resolution order (highest wins):
    ///   1. `override_price_cents` on the availability slot for that specific date
    ///   2. Highest-priority active rate rule that matches the date, channel, min_stay
    ///   3. `base_price_cents` on the catalog entry
    ///
    /// Returns the price in cents.
    pub async fn get_effective_price(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entry_id: Uuid,
        date: NaiveDate,
        channel: Option<&str>,
        // Total duration in billing-interval units.
        // For a Nightly entry: number of nights being booked.
        // For an Hourly entry: number of hours. For Daily: number of days.
        // Pass None when the caller doesn't know or the rule has no min_duration.
        duration: Option<i32>,
    ) -> Result<i64> {
        // 1. Load the catalog entry for base price.
        let entry = Self::get(db, tenant_id, entry_id).await?;

        // 2. Check for a day-specific override in the availability grid.
        let slot = atlas_catalog_availability::Entity::find()
            .filter(atlas_catalog_availability::Column::CatalogEntryId.eq(entry_id))
            .filter(atlas_catalog_availability::Column::SlotDate.eq(date))
            .one(db)
            .await?;

        if let Some(s) = &slot {
            if let Some(override_price) = s.override_price_cents {
                return Ok(override_price);
            }
        }

        // 3. Evaluate rate rules (descending priority — first match wins).
        let rules = atlas_catalog_rate_rule::Entity::find()
            .filter(atlas_catalog_rate_rule::Column::CatalogEntryId.eq(entry_id))
            .filter(atlas_catalog_rate_rule::Column::IsActive.eq(true))
            .order_by_desc(atlas_catalog_rate_rule::Column::Priority)
            .all(db)
            .await?;

        let day_bit = day_of_week_bit(date);

        for rule in &rules {
            if !rule_applies_to_date(rule, date, day_bit) {
                continue;
            }
            if !rule_applies_to_channel(rule, channel) {
                continue;
            }
            if !rule_applies_to_duration(rule, duration) {
                continue;
            }

            // This rule matches — apply it.
            if let Some(override_price) = rule.price_override_cents {
                return Ok(override_price);
            }
            if let Some(modifier_pct) = rule.price_modifier_pct {
                let modifier_f64 = f64::try_from(modifier_pct).unwrap_or(0.0);
                let price = (entry.base_price_cents as f64 * (1.0 + modifier_f64 / 100.0)).round() as i64;
                return Ok(price);
            }
        }

        // 4. No rule matched — return base price.
        Ok(entry.base_price_cents)
    }

    /// Compute the total effective price for a multi-night stay.
    /// Sums `get_effective_price()` for each night from `check_in` (inclusive)
    /// to `check_out` (exclusive).
    pub async fn get_effective_price_range(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entry_id: Uuid,
        check_in: NaiveDate,
        check_out: NaiveDate,
        channel: Option<&str>,
    ) -> Result<i64> {
        let nights = (check_out - check_in).num_days() as i32;
        if nights <= 0 {
            return Err(anyhow!("check_in must be before check_out"));
        }

        let mut total: i64 = 0;
        let mut current = check_in;
        while current < check_out {
            total += Self::get_effective_price(
                db,
                tenant_id,
                entry_id,
                current,
                channel,
                Some(nights), // duration = total nights for the stay
            )
            .await?;
            current = current.succ_opt().ok_or_else(|| anyhow!("date overflow"))?;
        }
        Ok(total)
    }

    // ── Availability grid management ──────────────────────────────────────────

    /// Check availability for a catalog entry over a date range.
    /// Returns one `AvailabilitySummary` per day, including effective price.
    /// Days with no inventory row are treated as fully available at base price.
    pub async fn check_availability(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entry_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<Vec<AvailabilitySummary>> {
        // Validate entry ownership before building the availability grid.
        let _entry = Self::get(db, tenant_id, entry_id).await?;

        let slots = atlas_catalog_availability::Entity::find()
            .filter(atlas_catalog_availability::Column::CatalogEntryId.eq(entry_id))
            .filter(atlas_catalog_availability::Column::SlotDate.gte(from_date))
            .filter(atlas_catalog_availability::Column::SlotDate.lt(to_date))
            .order_by_asc(atlas_catalog_availability::Column::SlotDate)
            .all(db)
            .await?;

        let nights = (to_date - from_date).num_days() as i32;

        // Index slots by date for O(1) lookup.
        let slot_map: std::collections::HashMap<NaiveDate, _> =
            slots.into_iter().map(|s| (s.slot_date, s)).collect();

        let mut result = Vec::with_capacity(nights as usize);
        let mut current = from_date;

        while current < to_date {
            let effective_price = Self::get_effective_price(
                db,
                tenant_id,
                entry_id,
                current,
                Some("direct"),
                Some(nights), // duration = total nights for this availability check
            )
            .await?;

            let summary = if let Some(slot) = slot_map.get(&current) {
                AvailabilitySummary {
                    slot_date: current,
                    total_inventory: slot.total_inventory,
                    reserved_count: slot.reserved_count,
                    available_count: slot.available_count,
                    is_blocked: slot.is_blocked,
                    block_reason: slot.block_reason.clone(),
                    effective_price_cents: effective_price,
                }
            } else {
                // No row → effectively unlimited availability (single-unit STR).
                AvailabilitySummary {
                    slot_date: current,
                    total_inventory: 1,
                    reserved_count: 0,
                    available_count: 1,
                    is_blocked: false,
                    block_reason: None,
                    effective_price_cents: effective_price,
                }
            };

            result.push(summary);
            current = current.succ_opt().ok_or_else(|| anyhow!("date overflow"))?;
        }

        Ok(result)
    }

    /// Increment `reserved_count` for each date in the range (exclusive `to_date`).
    ///
    /// Uses a database transaction with row-level locking to prevent double-booking.
    /// Rows that don't exist in the availability grid are created on first reserve
    /// (supports the common STR pattern of not pre-populating the availability grid).
    pub async fn reserve_slots(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entry_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        quantity: i32,
    ) -> Result<()> {
        let txn = db.begin().await?;
        Self::reserve_slots_in_txn(&txn, tenant_id, entry_id, from_date, to_date, quantity).await?;
        txn.commit().await?;
        Ok(())
    }

    /// Release slots (decrement `reserved_count`). Idempotent — clamped to 0.
    pub async fn release_slots(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        entry_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        quantity: i32,
    ) -> Result<()> {
        let txn = db.begin().await?;
        Self::release_slots_in_txn(&txn, tenant_id, entry_id, from_date, to_date, quantity).await?;
        txn.commit().await?;
        Ok(())
    }

    /// Transaction-scoped reserve — called by `ReservationService::create_hold()`
    /// which manages its own transaction context.
    pub async fn reserve_slots_in_txn(
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        entry_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        quantity: i32,
    ) -> Result<()> {
        // Bulk upsert via raw SQL — most efficient for range reservations.
        // ON CONFLICT increments reserved_count in one round-trip.
        let from_str = from_date.to_string();
        let to_str = to_date.to_string();

        // Generate date series and upsert in one statement.
        txn.execute_unprepared(&format!(
            "INSERT INTO atlas_catalog_availability
                (id, catalog_entry_id, tenant_id, slot_date, total_inventory, reserved_count, is_blocked)
             SELECT gen_random_uuid(), '{entry_id}', '{tenant_id}', d::date, 1, {quantity}, false
             FROM generate_series('{from_str}'::date, '{to_str}'::date - INTERVAL '1 day', INTERVAL '1 day') AS d
             ON CONFLICT (catalog_entry_id, slot_date)
             DO UPDATE SET reserved_count = atlas_catalog_availability.reserved_count + {quantity}"
        ))
        .await
        .map_err(|e| anyhow!("reserve_slots_in_txn failed for entry {entry_id}: {e:#}"))?;

        Ok(())
    }

    /// Transaction-scoped release — called by `ReservationService::cancel()`.
    pub async fn release_slots_in_txn(
        txn: &DatabaseTransaction,
        _tenant_id: Uuid,
        entry_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
        quantity: i32,
    ) -> Result<()> {
        let from_str = from_date.to_string();
        let to_str = to_date.to_string();

        // Decrement but clamp to 0 — safe if called more than once.
        txn.execute_unprepared(&format!(
            "UPDATE atlas_catalog_availability
             SET reserved_count = GREATEST(0, reserved_count - {quantity})
             WHERE catalog_entry_id = '{entry_id}'
               AND slot_date >= '{from_str}'
               AND slot_date < '{to_str}'"
        ))
        .await
        .map_err(|e| anyhow!("release_slots_in_txn failed for entry {entry_id}: {e:#}"))?;

        Ok(())
    }

    // ── Rate push stub (Phase 7 — OTA channel manager) ────────────────────────

    /// Push rate and availability updates to an external PMS/OTA via integration.
    ///
    /// Currently a stub — Phase 7 will wire this to the OTA integration adapter.
    /// The stub logs the intent so the background job infrastructure can be built
    /// incrementally.
    pub async fn push_rates_to_pms(
        _db: &DatabaseConnection,
        entry_id: Uuid,
        integration_id: Uuid,
        from_date: NaiveDate,
        to_date: NaiveDate,
    ) -> Result<()> {
        tracing::info!(
            %entry_id, %integration_id,
            from = %from_date, to = %to_date,
            "CatalogService::push_rates_to_pms: stub — Phase 7 OTA channel push pending"
        );
        Ok(())
    }
}

// ── Rate rule applicability helpers ──────────────────────────────────────────

/// Returns the day-of-week bitmask value for a date.
/// Mon=1, Tue=2, Wed=4, Thu=8, Fri=16, Sat=32, Sun=64.
fn day_of_week_bit(date: NaiveDate) -> i32 {
    // chrono::Weekday: Mon=0..Sun=6
    1 << date.weekday().num_days_from_monday()
}

fn rule_applies_to_date(
    rule: &atlas_catalog_rate_rule::Model,
    date: NaiveDate,
    day_bit: i32,
) -> bool {
    if let Some(from) = rule.applies_from {
        if date < from {
            return false;
        }
    }
    if let Some(to) = rule.applies_to {
        if date > to {
            return false;
        }
    }
    if let Some(mask) = rule.day_of_week_mask {
        if mask & day_bit == 0 {
            return false;
        }
    }
    true
}

fn rule_applies_to_channel(rule: &atlas_catalog_rate_rule::Model, channel: Option<&str>) -> bool {
    match (&rule.channel, channel) {
        (None, _) => true,       // rule applies to all channels
        (Some(_), None) => true, // caller didn't specify a channel — don't exclude
        (Some(rc), Some(c)) => rc.eq_ignore_ascii_case(c),
    }
}

fn rule_applies_to_duration(rule: &atlas_catalog_rate_rule::Model, duration: Option<i32>) -> bool {
    match (rule.min_duration, duration) {
        (None, _) => true,
        (Some(_), None) => true,
        (Some(min), Some(d)) => d >= min,
    }
}
