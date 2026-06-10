//! Folio — Appliance service.
//!
//! Implements the Folio app's typed interface over G-10 `atlas_assets`
//! for `asset_type = "appliance"`. The DB schema is generic; this module
//! owns the Folio-specific contract for `lifecycle_metadata`.
//!
//! # Architecture note
//!
//! The three indexed platform columns are written by this service:
//! - `scheduled_service_date` ← computed from install_date + service_interval_days
//! - `expiry_date`            ← warranty_expiry_date input
//! - `condition`              ← condition input
//!
//! All other appliance-specific fields live in `lifecycle_metadata` JSONB.
//! See: `docs/architecture/asset_metadata_shapes.md` — "appliance" shape.

use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_asset;

// ── Appliance type enum ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceType {
    Refrigerator,
    Washer,
    Dryer,
    WasherDryerCombo,
    WaterHeater,
    Boiler,
    HvacUnit,
    Dishwasher,
    OvenRange,
    GarbageDisposal,
    PoolPump,
    GarageDoorOpener,
    AirHandler,
    WaterSoftener,
    Other,
}

impl std::fmt::Display for ApplianceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "other".to_string());
        write!(f, "{}", s)
    }
}

// ── Fuel type enum ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FuelType {
    Electric,
    Gas,
    Propane,
    Solar,
    HeatPump,
}

// ── ApplianceMetadata — lifecycle_metadata shape for asset_type = "appliance" ──
//
// This struct owns the JSONB shape. Keys are stable public API.
// Increment `metadata_version` on any breaking key change.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplianceMetadata {
    /// Schema version. Increment when keys are renamed or removed.
    pub metadata_version: u32,
    pub appliance_type: ApplianceType,
    pub make: String,
    pub model: String,
    pub year_manufactured: Option<u16>,
    pub fuel_type: Option<FuelType>,
    /// Installer service provider ID (FK to atlas_service_providers — not a DB FK,
    /// enforced at the service layer only).
    pub installer_sp_id: Option<Uuid>,
    pub warranty_provider: Option<String>,
    pub warranty_contact: Option<String>,
    /// Original purchase price in the tenant's base currency (cents).
    pub purchase_price_cents: Option<i64>,
    /// Current market replacement cost (cents). Updated annually.
    pub replacement_cost_cents: Option<i64>,
    /// How often this appliance should be serviced. Used to compute
    /// `scheduled_service_date` on create and after each service log.
    pub service_interval_days: Option<i32>,
}

// ── Service input / output types ──────────────────────────────────────────────

/// Input for creating an appliance linked to a unit asset.
#[derive(Debug, Deserialize)]
pub struct CreateApplianceInput {
    /// The unit (parent asset) this appliance belongs to.
    pub unit_id: Uuid,
    /// Human-readable name e.g. "Kitchen Refrigerator" or "Boiler - Unit 3B".
    pub name: String,
    /// Serial number. Written to both `serial_or_folio_number` and
    /// `lifecycle_metadata.serial_number` for searchability.
    pub serial_number: Option<String>,
    /// Warranty expiry date — written to the indexed `expiry_date` column.
    pub warranty_expiry_date: Option<NaiveDate>,
    /// Date the appliance was installed — used to compute next service date.
    pub install_date: Option<NaiveDate>,
    /// Current operational condition.
    pub condition: Option<String>,
    /// Typed appliance-specific fields.
    pub metadata: ApplianceMetadata,
}

/// Input for updating lifecycle fields after a service event.
#[derive(Debug, Deserialize)]
pub struct UpdateApplianceLifecycleInput {
    /// New condition after service (optional).
    pub condition: Option<String>,
    /// Override the next service date. If None, recomputed from interval.
    pub next_service_date: Option<NaiveDate>,
    /// Updated warranty expiry (e.g. extended warranty was added).
    pub expiry_date: Option<NaiveDate>,
    /// Updated metadata fields (merged with existing — non-null fields win).
    pub metadata_patch: Option<serde_json::Value>,
}

/// Full appliance response including lifecycle fields.
#[derive(Debug, Serialize)]
pub struct ApplianceDetail {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub unit_id: Option<Uuid>,
    pub name: String,
    pub serial_number: Option<String>,
    pub status: String,
    pub condition: Option<String>,
    pub scheduled_service_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Alert row — assets with service due or expiry within the alert horizon.
#[derive(Debug, Serialize)]
pub struct AssetLifecycleAlert {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub unit_id: Option<Uuid>,
    pub condition: Option<String>,
    pub scheduled_service_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    /// Days until the earliest of scheduled_service_date / expiry_date.
    /// Negative = already overdue.
    pub days_until_alert: i64,
}

// ── ApplianceService ──────────────────────────────────────────────────────────

pub struct ApplianceService;

impl ApplianceService {
    /// Create an appliance linked to a unit, writing all lifecycle columns.
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateApplianceInput,
    ) -> Result<Uuid, anyhow::Error> {
        // Domain validation (Folio-specific — DB enforces nothing beyond JSONB)
        if input.metadata.make.trim().is_empty() {
            anyhow::bail!("make is required for appliances");
        }
        if input.metadata.model.trim().is_empty() {
            anyhow::bail!("model is required for appliances");
        }
        if let Some(interval) = input.metadata.service_interval_days {
            if interval <= 0 {
                anyhow::bail!("service_interval_days must be > 0");
            }
        }

        // Compute scheduled_service_date from install_date + interval
        let scheduled_service_date = compute_next_service_date(
            input.install_date,
            input.metadata.service_interval_days,
        );

        let id = Uuid::new_v4();
        let meta_json = serde_json::to_value(&input.metadata)?;

        atlas_asset::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            parent_asset_id: Set(Some(input.unit_id)),
            asset_type: Set("appliance".to_string()),
            name: Set(input.name),
            serial_or_folio_number: Set(input.serial_number),
            status: Set("active".to_string()),
            scheduled_service_date: Set(scheduled_service_date),
            expiry_date: Set(input.warranty_expiry_date),
            condition: Set(input.condition),
            lifecycle_metadata: Set(Some(meta_json)),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(id)
    }

    /// Update lifecycle fields after a service event or manual edit.
    pub async fn update_lifecycle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        appliance_id: Uuid,
        input: UpdateApplianceLifecycleInput,
    ) -> Result<(), anyhow::Error> {
        let existing = atlas_asset::Entity::find_by_id(appliance_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::AssetType.eq("appliance"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("appliance {} not found", appliance_id))?;

        // Extract and merge lifecycle_metadata before consuming the model.
        // Cloning existing_meta avoids a partial-move compile error when we
        // later call existing.into() to get the ActiveModel.
        let existing_meta_clone = existing.lifecycle_metadata.clone();
        let updated_metadata = match (existing_meta_clone, input.metadata_patch) {
            (Some(mut existing_meta), Some(patch)) => {
                if let (Some(obj), Some(patch_obj)) =
                    (existing_meta.as_object_mut(), patch.as_object())
                {
                    for (k, v) in patch_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
                Some(existing_meta)
            }
            (existing_meta, None) => existing_meta,
            (None, Some(patch)) => Some(patch),
        };

        let mut model: atlas_asset::ActiveModel = existing.into();
        if let Some(c) = input.condition {
            model.condition = Set(Some(c));
        }
        if let Some(d) = input.next_service_date {
            model.scheduled_service_date = Set(Some(d));
        }
        if let Some(e) = input.expiry_date {
            model.expiry_date = Set(Some(e));
        }
        if let Some(m) = updated_metadata {
            model.lifecycle_metadata = Set(Some(m));
        }

        model.update(db).await?;
        Ok(())
    }

    /// List all appliances for a given unit.
    pub async fn list_for_unit(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        unit_id: Uuid,
    ) -> Result<Vec<ApplianceDetail>, anyhow::Error> {
        let rows = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::ParentAssetId.eq(unit_id))
            .filter(atlas_asset::Column::AssetType.eq("appliance"))
            .all(db)
            .await?;

        Ok(rows.into_iter().map(to_detail).collect())
    }

    /// Get a single appliance by ID (tenant-scoped).
    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        appliance_id: Uuid,
    ) -> Result<ApplianceDetail, anyhow::Error> {
        let row = atlas_asset::Entity::find_by_id(appliance_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::AssetType.eq("appliance"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("appliance {} not found", appliance_id))?;

        Ok(to_detail(row))
    }

    /// Platform-level alert query — all assets (any asset_type) with service due
    /// or expiry within the given horizon. Not appliance-specific.
    pub async fn get_lifecycle_alerts(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        horizon_days: i64,
    ) -> Result<Vec<AssetLifecycleAlert>, anyhow::Error> {
        use sea_orm::Condition;

        let today = chrono::Utc::now().date_naive();
        let cutoff = today + chrono::Duration::days(horizon_days);

        let rows = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(
                Condition::any()
                    .add(atlas_asset::Column::ScheduledServiceDate.lte(cutoff))
                    .add(atlas_asset::Column::ExpiryDate.lte(cutoff)),
            )
            .all(db)
            .await?;

        let mut alerts: Vec<AssetLifecycleAlert> = rows
            .into_iter()
            .map(|a| {
                let earliest = earliest_date(a.scheduled_service_date, a.expiry_date);
                let days_until = earliest
                    .map(|d| (d - today).num_days())
                    .unwrap_or(i64::MAX);
                AssetLifecycleAlert {
                    id: a.id,
                    name: a.name,
                    asset_type: a.asset_type,
                    unit_id: a.parent_asset_id,
                    condition: a.condition,
                    scheduled_service_date: a.scheduled_service_date,
                    expiry_date: a.expiry_date,
                    days_until_alert: days_until,
                }
            })
            .collect();

        // Sort: overdue first, then soonest
        alerts.sort_by_key(|a| a.days_until_alert);
        Ok(alerts)
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn compute_next_service_date(
    install_date: Option<NaiveDate>,
    interval_days: Option<i32>,
) -> Option<NaiveDate> {
    match (install_date, interval_days) {
        (Some(d), Some(i)) if i > 0 => Some(d + chrono::Duration::days(i as i64)),
        _ => None,
    }
}

fn earliest_date(a: Option<NaiveDate>, b: Option<NaiveDate>) -> Option<NaiveDate> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

fn to_detail(a: atlas_asset::Model) -> ApplianceDetail {
    ApplianceDetail {
        id: a.id,
        tenant_id: a.tenant_id,
        unit_id: a.parent_asset_id,
        name: a.name,
        serial_number: a.serial_or_folio_number,
        status: a.status,
        condition: a.condition,
        scheduled_service_date: a.scheduled_service_date,
        expiry_date: a.expiry_date,
        metadata: a.lifecycle_metadata,
        created_at: a.created_at,
    }
}
