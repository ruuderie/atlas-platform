//! Folio — Violation Service
//!
//! Manages compliance violations filed against tenants (LTR) and STR guests.
//! Stored in G-13 `atlas_cases` with `case_type = "compliance_violation"`.
//!
//! # Who does what
//! - **Landlord** files a violation against a lease/unit (`file_violation`)
//! - **Tenant** reads their own violations (`list_for_tenant`)
//! - **Both** can see the cure status and deadline
//!
//! # LTR vs STR violations
//!
//! | | LTR | STR |
//! |---|---|---|
//! | Linked to | `contract_id` (lease) | `reservation_id` (booking) |
//! | Cure window | 3–30 days | Usually `None` (immediate) |
//! | Categories | General compliance | Includes `UnauthorizedParty`, `OverOccupancy` |
//! | Charge back | Lease deposit | OTA damage claim / AirCover |
//!
//! # Cure window tracking
//! `case_metadata.cure_deadline` (ISO date) is the date by which the tenant
//! must remedy the violation. `case_metadata.cure_status` transitions:
//!   "open" → "cured" | "escalated" | "dismissed"
//!
//! # Type safety
//! `ViolationCategory` and `CureStatus` are enums — the compiler rejects
//! invalid strings before they reach the DB.

use anyhow::{anyhow, bail, Result};
use chrono::{NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::atlas_case;
use crate::types::pm::PmCaseType;

// ── Violation category ────────────────────────────────────────────────────────

/// The category of the lease/property violation.
/// Adding a new category here is the only way to file that type — raw strings
/// cannot reach the DB.
///
/// STR-specific categories (`UnauthorizedParty`, `OverOccupancy`) are used
/// when `reservation_id` is set on `FileViolationInput`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationCategory {
    // ── LTR & STR shared ─────────────────────────────────────────────
    Noise,
    UnauthorizedOccupant,
    UnauthorizedPet,
    UnauthorizedVehicle,
    PropertyDamage,
    LeaseBreach,
    Subletting,
    FailureToMaintain,
    IllegalActivity,
    Hoarding,
    SmokingInUnit,
    Other,
    // ── STR-specific ──────────────────────────────────────────────────
    /// Guest held a party / gathering exceeding the unit's occupancy rules.
    /// Use with `reservation_id` on STR bookings.
    UnauthorizedParty,
    /// More guests present than registered on the booking (`guest_count` exceeded).
    /// Use with `reservation_id` on STR bookings.
    OverOccupancy,
}

impl std::fmt::Display for ViolationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Noise                => "noise",
            Self::UnauthorizedOccupant => "unauthorized_occupant",
            Self::UnauthorizedPet      => "unauthorized_pet",
            Self::UnauthorizedVehicle  => "unauthorized_vehicle",
            Self::PropertyDamage       => "property_damage",
            Self::LeaseBreach          => "lease_breach",
            Self::Subletting           => "subletting",
            Self::FailureToMaintain    => "failure_to_maintain",
            Self::IllegalActivity      => "illegal_activity",
            Self::Hoarding             => "hoarding",
            Self::SmokingInUnit        => "smoking_in_unit",
            Self::Other                => "other",
            Self::UnauthorizedParty    => "unauthorized_party",
            Self::OverOccupancy        => "over_occupancy",
        })
    }
}

impl TryFrom<String> for ViolationCategory {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "noise"                 => Ok(Self::Noise),
            "unauthorized_occupant" => Ok(Self::UnauthorizedOccupant),
            "unauthorized_pet"      => Ok(Self::UnauthorizedPet),
            "unauthorized_vehicle"  => Ok(Self::UnauthorizedVehicle),
            "property_damage"       => Ok(Self::PropertyDamage),
            "lease_breach"          => Ok(Self::LeaseBreach),
            "subletting"            => Ok(Self::Subletting),
            "failure_to_maintain"   => Ok(Self::FailureToMaintain),
            "illegal_activity"      => Ok(Self::IllegalActivity),
            "hoarding"              => Ok(Self::Hoarding),
            "smoking_in_unit"       => Ok(Self::SmokingInUnit),
            "other"                 => Ok(Self::Other),
            "unauthorized_party"    => Ok(Self::UnauthorizedParty),
            "over_occupancy"        => Ok(Self::OverOccupancy),
            other => Err(format!("unknown ViolationCategory: '{other}'")),
        }
    }
}

// ── Cure status ───────────────────────────────────────────────────────────────

/// Lifecycle state of the cure window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CureStatus {
    /// Violation filed — tenant has until `cure_deadline` to remedy.
    Open,
    /// Tenant has remedied the violation within the cure window.
    Cured,
    /// Cure window expired without remedy — escalated (eviction / legal notice).
    Escalated,
    /// Landlord dismissed the violation (e.g. misunderstanding).
    Dismissed,
}

impl std::fmt::Display for CureStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Open       => "open",
            Self::Cured      => "cured",
            Self::Escalated  => "escalated",
            Self::Dismissed  => "dismissed",
        })
    }
}

impl TryFrom<String> for CureStatus {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "open"      => Ok(Self::Open),
            "cured"     => Ok(Self::Cured),
            "escalated" => Ok(Self::Escalated),
            "dismissed" => Ok(Self::Dismissed),
            other => Err(format!("unknown CureStatus: '{other}'")),
        }
    }
}

// ── Input / output types ──────────────────────────────────────────────────────

/// Landlord files a violation against a tenant's unit (LTR) or STR guest booking.
///
/// For LTR violations: set `contract_id`, leave `reservation_id` as `None`.
/// For STR violations: set `reservation_id`, leave `contract_id` as `None`.
/// `cure_days` is typically `None` for STR (guest is leaving imminently).
#[derive(Debug, Clone)]
pub struct FileViolationInput {
    /// The unit asset where the violation occurred.
    pub asset_id: Uuid,
    /// The active LTR lease this violation is associated with.
    pub contract_id: Option<Uuid>,
    /// The STR booking this violation is associated with.
    /// Set for STR guest violations; `None` for LTR violations.
    pub reservation_id: Option<Uuid>,
    pub category: ViolationCategory,
    /// Short subject line. e.g. "Unauthorized party — 15+ guests at 2am"
    pub subject: String,
    pub description: String,
    /// Days the tenant has to cure. Standard is 3–30 days (LTR).
    /// `None` = no cure period — typical for STR (guest checked out / OTA claim).
    pub cure_days: Option<u8>,
    /// Optional evidence: photo R2 keys, noise complaint refs, OTA report URLs, etc.
    pub evidence_notes: Option<String>,
}

/// Violation record returned from queries.
#[derive(Debug, Clone, Serialize)]
pub struct ViolationRecord {
    pub id: Uuid,
    pub asset_id: Option<Uuid>,
    /// Set for LTR violations.
    pub contract_id: Option<Uuid>,
    /// Set for STR violations — links to the specific booking.
    pub reservation_id: Option<Uuid>,
    pub category: String,
    pub subject: String,
    pub description: Option<String>,
    pub cure_status: CureStatus,
    pub cure_deadline: Option<NaiveDate>,
    pub filed_at: chrono::DateTime<Utc>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
    pub resolution_notes: Option<String>,
}

// ── ViolationService ──────────────────────────────────────────────────────────

pub struct ViolationService;

impl ViolationService {
    /// File a new violation against a tenant (landlord action).
    ///
    /// The cure deadline is computed from `cure_days` relative to today.
    /// Violations with no cure period are filed directly as `open` with no deadline.
    pub async fn file_violation(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filed_by_user_id: Uuid,
        input: FileViolationInput,
    ) -> Result<ViolationRecord> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let today = now.date_naive();

        let cure_deadline = input.cure_days.map(|days| {
            today + chrono::Duration::days(days as i64)
        });

        let metadata = serde_json::json!({
            "category":        input.category.to_string(),
            "cure_status":     CureStatus::Open.to_string(),
            "cure_deadline":   cure_deadline.map(|d| d.to_string()),
            "evidence_notes":  input.evidence_notes,
            "filed_by":        filed_by_user_id,
            "contract_id":     input.contract_id,
            "reservation_id":  input.reservation_id,
        });

        let case = atlas_case::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            case_type: Set(PmCaseType::ComplianceViolation.to_string()),
            asset_id: Set(Some(input.asset_id)),
            subject: Set(input.subject.clone()),
            description: Set(Some(input.description.clone())),
            status: Set(CureStatus::Open.to_string()),
            priority: Set("routine".to_string()),
            reported_by_user_id: Set(Some(filed_by_user_id)),
            case_metadata: Set(Some(metadata)),
            created_at: Set(now),
            ..Default::default()
        };
        case.insert(db).await?;

        tracing::info!(violation_id = %id, %tenant_id, asset_id = %input.asset_id,
            category = %input.category,
            reservation_id = ?input.reservation_id,
            "ViolationService: violation filed");

        Ok(ViolationRecord {
            id,
            asset_id: Some(input.asset_id),
            contract_id: input.contract_id,
            reservation_id: input.reservation_id,
            category: input.category.to_string(),
            subject: input.subject,
            description: Some(input.description),
            cure_status: CureStatus::Open,
            cure_deadline,
            filed_at: now,
            resolved_at: None,
            resolution_notes: None,
        })
    }

    /// List all violations filed against a specific tenant user (tenant self-view).
    /// Tenants can see their own violation history — read-only.
    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        tenant_user_id: Uuid,
    ) -> Result<Vec<ViolationRecord>> {
        // Violations are filed by the landlord (reported_by) against the tenant's unit.
        // The tenant sees violations on assets associated with their active leases.
        // We identify via `reported_by_user_id` pointing to the landlord filing —
        // and scope via tenant_id + the asset IDs on the tenant's contracts.
        // Simple approach: filter by tenant_id and the counterparty contracts for this user.
        use crate::entities::atlas_contract;
        use sea_orm::QuerySelect;

        let lease_asset_ids: Vec<Uuid> = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::CounterpartyUserId.eq(tenant_user_id))
            .filter(atlas_contract::Column::ContractType.eq("lease"))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|c| c.asset_id)
            .collect();

        if lease_asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ComplianceViolation.to_string()))
            .filter(atlas_case::Column::AssetId.is_in(lease_asset_ids))
            .order_by_desc(atlas_case::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(cases.into_iter().map(to_violation_record).collect())
    }

    /// List all violations on a specific asset (landlord view — unit or property).
    pub async fn list_for_asset(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<ViolationRecord>> {
        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ComplianceViolation.to_string()))
            .filter(atlas_case::Column::AssetId.eq(asset_id))
            .order_by_desc(atlas_case::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(cases.into_iter().map(to_violation_record).collect())
    }

    /// Transition the cure status on a violation.
    ///
    /// Valid transitions:
    /// - `Open` → `Cured` (tenant fixed it)
    /// - `Open` → `Escalated` (cure window expired, landlord escalates)
    /// - `Open` → `Dismissed` (landlord withdraws)
    /// - Any other transition is rejected.
    pub async fn update_cure_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        violation_id: Uuid,
        new_status: CureStatus,
        resolution_notes: Option<String>,
    ) -> Result<ViolationRecord> {
        let case = atlas_case::Entity::find_by_id(violation_id)
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ComplianceViolation.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("violation {violation_id} not found"))?;

        let current = CureStatus::try_from(case.status.clone())
            .unwrap_or(CureStatus::Open);

        // Validate the transition
        match (&current, &new_status) {
            (CureStatus::Open, CureStatus::Cured)
            | (CureStatus::Open, CureStatus::Escalated)
            | (CureStatus::Open, CureStatus::Dismissed) => {} // valid
            _ => bail!(
                "cannot transition violation from '{}' to '{}'",
                current, new_status
            ),
        }

        let now = Utc::now();
        let mut meta = case.case_metadata.clone().unwrap_or(serde_json::json!({}));
        meta["cure_status"] = serde_json::json!(new_status.to_string());
        meta["resolved_at"] = serde_json::json!(now.to_rfc3339());
        if let Some(n) = &resolution_notes {
            meta["resolution_notes"] = serde_json::json!(n);
        }

        let mut active: atlas_case::ActiveModel = case.into();
        active.status = Set(new_status.to_string());
        active.completed_at = Set(Some(now));
        active.case_metadata = Set(Some(meta));
        let updated = active.update(db).await?;

        tracing::info!(violation_id = %violation_id, %tenant_id,
            status = %new_status, "ViolationService: cure status updated");

        Ok(to_violation_record(updated))
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

pub fn to_violation_record(c: atlas_case::Model) -> ViolationRecord {
    let meta = c.case_metadata.as_ref();
    let cure_status = meta
        .and_then(|m| m["cure_status"].as_str())
        .and_then(|s| CureStatus::try_from(s.to_string()).ok())
        .unwrap_or(CureStatus::Open);
    let cure_deadline = meta
        .and_then(|m| m["cure_deadline"].as_str())
        .and_then(|s| s.parse().ok());
    let resolved_at = meta
        .and_then(|m| m["resolved_at"].as_str())
        .and_then(|s| s.parse::<chrono::DateTime<Utc>>().ok());
    let resolution_notes = meta
        .and_then(|m| m["resolution_notes"].as_str())
        .map(|s| s.to_string());
    let category = meta
        .and_then(|m| m["category"].as_str())
        .unwrap_or("other")
        .to_string();
    let contract_id = meta
        .and_then(|m| m["contract_id"].as_str())
        .and_then(|s| Uuid::parse_str(s).ok());
    let reservation_id = meta
        .and_then(|m| m["reservation_id"].as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    ViolationRecord {
        id: c.id,
        asset_id: c.asset_id,
        contract_id,
        reservation_id,
        category,
        subject: c.subject,
        description: c.description,
        cure_status,
        cure_deadline,
        filed_at: c.created_at,
        resolved_at,
        resolution_notes,
    }
}
