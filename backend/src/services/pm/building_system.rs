//! Folio — Building System service.
//!
//! Implements the Folio app's typed interface over G-10 `atlas_assets`
//! for `asset_type = "building_system"`.
//!
//! Building systems are infrastructure that belongs to the **property** (building)
//! itself, not to any individual unit:
//!   - Elevators, fire suppression, roof, common-area HVAC, generators, pools
//!
//! # Hierarchy
//!
//! ```text
//! atlas_assets (asset_type = "real_estate_property")    ← the building
//!   ├── atlas_assets (asset_type = "real_estate_unit")  ← Unit 1A
//!   │     └── atlas_assets (asset_type = "appliance")   ← appliance in unit
//!   ├── atlas_assets (asset_type = "building_system")   ← Elevator
//!   └── atlas_assets (asset_type = "building_system")   ← Roof
//! ```
//!
//! `parent_asset_id` → property asset (not a unit).
//!
//! # Regulatory linkage
//!
//! Systems requiring state/municipal certificates (elevators, fire suppression,
//! boilers) link their regulatory registrations via G-22 `atlas_record_relationships`
//! with `relationship_type = "regulatory_cert"`. No new schema required.
//!
//! # Lifecycle columns (platform G-10)
//!
//! - `scheduled_service_date` ← next inspection / maintenance due
//! - `expiry_date`            ← certificate / warranty / contract expiry
//! - `condition`              ← operational state
//! - `lifecycle_metadata`     ← BuildingSystemMetadata JSONB

use chrono::NaiveDate;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_asset;

// ── Building system type enum ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BuildingSystemType {
    // Vertical transportation
    Elevator,
    Escalator,
    // Life safety
    FireSuppression,
    FireAlarm,
    EmergencyLighting,
    // Mechanical
    CommonAreaHvac,
    Boiler,
    CoolingTower,
    Chiller,
    // Electrical
    Generator,
    ElectricalPanel,
    TransformerVault,
    // Water / plumbing
    RoofDrainSystem,
    SewerLift,
    BackflowPreventer,
    // Structure / envelope
    Roof,
    Facade,
    ParkingStructure,
    // Amenities
    Pool,
    Spa,
    GymEquipment,
    // Access / security
    SecuritySystem,
    AccessControl,
    Intercom,
    // Other
    Other,
}

// ── Certificate type ──────────────────────────────────────────────────────────

/// Which regulatory certificate governs this system, if any.
/// Used to guide the landlord to link the right G-16 registration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertificateType {
    StateElevatorCert,
    FireInspectionPermit,
    BoilerPressureVesselCert,
    PoolHealthPermit,
    BuildingPermit,
    Epa608Hvac,
    None,
}

// ── BuildingSystemMetadata ────────────────────────────────────────────────────
//
// Owned JSONB shape for asset_type = "building_system".
// Keys are stable public API — see asset_metadata_shapes.md.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSystemMetadata {
    /// Schema version — increment on any breaking key change.
    pub metadata_version: u32,
    pub system_type: BuildingSystemType,
    /// Make / brand (e.g. "Otis", "Carrier", "Generac"). Optional — some
    /// infrastructure (roof, parking structure) has no single manufacturer.
    pub make: Option<String>,
    pub model: Option<String>,
    pub serial_number: Option<String>,
    pub year_installed: Option<u16>,
    /// Which regulatory certificate governs this system.
    pub certificate_type: CertificateType,
    /// Issuing authority (e.g. "Miami-Dade County Fire Marshal").
    pub certificate_issuer: Option<String>,
    /// Certificate / permit number for reference.
    pub certificate_number: Option<String>,
    /// Maintenance contractor — FK to atlas_service_providers (not a DB FK).
    pub contractor_sp_id: Option<Uuid>,
    /// Maintenance contract end date (if on a service contract).
    pub service_contract_expiry: Option<NaiveDate>,
    /// Approximate replacement cost for capital planning (cents).
    pub replacement_cost_cents: Option<i64>,
    /// Useful life in years from install date (for capital reserve planning).
    pub useful_life_years: Option<u16>,
    /// Whether this system serves all units (true) or is a shared amenity (false).
    pub is_building_wide: Option<bool>,
    /// Free-form location within the building (e.g. "Roof level — south end").
    pub location_notes: Option<String>,
}

// ── Service input / output types ──────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateBuildingSystemInput {
    /// The property (building) asset this system belongs to.
    pub property_id: Uuid,
    /// Human-readable name e.g. "Main Elevator", "Roof — South Building".
    pub name: String,
    pub serial_number: Option<String>,
    /// Next inspection / maintenance due — written to `scheduled_service_date`.
    pub next_inspection_date: Option<NaiveDate>,
    /// Certificate or warranty expiry — written to `expiry_date`.
    pub cert_expiry_date: Option<NaiveDate>,
    pub condition: Option<String>,
    pub metadata: BuildingSystemMetadata,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBuildingSystemLifecycleInput {
    pub condition: Option<String>,
    pub next_inspection_date: Option<NaiveDate>,
    pub cert_expiry_date: Option<NaiveDate>,
    pub metadata_patch: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct BuildingSystemDetail {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub property_id: Option<Uuid>,
    pub name: String,
    pub serial_number: Option<String>,
    pub status: String,
    pub condition: Option<String>,
    pub scheduled_service_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// ── BuildingSystemService ─────────────────────────────────────────────────────

pub struct BuildingSystemService;

impl BuildingSystemService {
    /// Create a building system attached to a property asset.
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateBuildingSystemInput,
    ) -> Result<Uuid, anyhow::Error> {
        // Domain validation
        if input.name.trim().is_empty() {
            anyhow::bail!("name is required for building systems");
        }
        if let Some(years) = input.metadata.useful_life_years {
            if years == 0 {
                anyhow::bail!("useful_life_years must be > 0");
            }
        }

        let id = Uuid::new_v4();
        let meta_json = serde_json::to_value(&input.metadata)?;

        atlas_asset::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            parent_asset_id: Set(Some(input.property_id)),
            asset_type: Set("building_system".to_string()),
            name: Set(input.name),
            serial_or_folio_number: Set(input.serial_number),
            status: Set("active".to_string()),
            scheduled_service_date: Set(input.next_inspection_date),
            expiry_date: Set(input.cert_expiry_date),
            condition: Set(input.condition),
            lifecycle_metadata: Set(Some(meta_json)),
            ..Default::default()
        }
        .insert(db)
        .await?;

        Ok(id)
    }

    /// Update lifecycle state after an inspection or service event.
    pub async fn update_lifecycle(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        system_id: Uuid,
        input: UpdateBuildingSystemLifecycleInput,
    ) -> Result<(), anyhow::Error> {
        let existing = atlas_asset::Entity::find_by_id(system_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::AssetType.eq("building_system"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("building system {} not found", system_id))?;

        // Clone before consuming to avoid partial-move compile error.
        let existing_meta_clone = existing.lifecycle_metadata.clone();
        let updated_metadata = match (existing_meta_clone, input.metadata_patch) {
            (Some(mut m), Some(patch)) => {
                if let (Some(obj), Some(p)) = (m.as_object_mut(), patch.as_object()) {
                    for (k, v) in p {
                        obj.insert(k.clone(), v.clone());
                    }
                }
                Some(m)
            }
            (m, None) => m,
            (None, Some(patch)) => Some(patch),
        };

        let mut model: atlas_asset::ActiveModel = existing.into();
        if let Some(c) = input.condition {
            model.condition = Set(Some(c));
        }
        if let Some(d) = input.next_inspection_date {
            model.scheduled_service_date = Set(Some(d));
        }
        if let Some(e) = input.cert_expiry_date {
            model.expiry_date = Set(Some(e));
        }
        if let Some(m) = updated_metadata {
            model.lifecycle_metadata = Set(Some(m));
        }

        model.update(db).await?;
        Ok(())
    }

    /// List all building systems for a property.
    pub async fn list_for_property(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        property_id: Uuid,
    ) -> Result<Vec<BuildingSystemDetail>, anyhow::Error> {
        let rows = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::ParentAssetId.eq(property_id))
            .filter(atlas_asset::Column::AssetType.eq("building_system"))
            .all(db)
            .await?;
        Ok(rows.into_iter().map(to_detail).collect())
    }

    /// Get a single building system (tenant-scoped).
    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        system_id: Uuid,
    ) -> Result<BuildingSystemDetail, anyhow::Error> {
        let row = atlas_asset::Entity::find_by_id(system_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::AssetType.eq("building_system"))
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("building system {} not found", system_id))?;
        Ok(to_detail(row))
    }

    /// Combined lifecycle alert query — all asset types (appliances + building systems)
    /// with service due or expiry within the given horizon.
    ///
    /// This is the canonical platform-level alert query. Results are sorted:
    /// overdue first (negative days), then soonest upcoming.
    pub async fn get_lifecycle_alerts(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        horizon_days: i64,
    ) -> Result<Vec<LifecycleAlert>, anyhow::Error> {
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

        let mut alerts: Vec<LifecycleAlert> = rows
            .into_iter()
            .map(|a| {
                let earliest = earliest_date(a.scheduled_service_date, a.expiry_date);
                let days_until = earliest
                    .map(|d| (d - today).num_days())
                    .unwrap_or(i64::MAX);
                LifecycleAlert {
                    id: a.id,
                    name: a.name,
                    asset_type: a.asset_type,
                    parent_asset_id: a.parent_asset_id,
                    condition: a.condition,
                    scheduled_service_date: a.scheduled_service_date,
                    expiry_date: a.expiry_date,
                    days_until_alert: days_until,
                }
            })
            .collect();

        alerts.sort_by_key(|a| a.days_until_alert);
        Ok(alerts)
    }
}

/// Combined alert row — covers any asset_type (appliance, building_system, etc.)
#[derive(Debug, Serialize)]
pub struct LifecycleAlert {
    pub id: Uuid,
    pub name: String,
    /// Discriminator — "appliance" | "building_system" | any future type.
    pub asset_type: String,
    /// For appliances: unit_id. For building systems: property_id.
    pub parent_asset_id: Option<Uuid>,
    pub condition: Option<String>,
    pub scheduled_service_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    /// Days until alert fires. Negative = already overdue.
    pub days_until_alert: i64,
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn earliest_date(a: Option<NaiveDate>, b: Option<NaiveDate>) -> Option<NaiveDate> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

fn to_detail(a: atlas_asset::Model) -> BuildingSystemDetail {
    BuildingSystemDetail {
        id: a.id,
        tenant_id: a.tenant_id,
        property_id: a.parent_asset_id,
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
