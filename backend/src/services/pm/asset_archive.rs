//! Soft-archive (decommission) assets with Folio blocker policy.

use anyhow::{anyhow, Result};
use chrono::{Datelike, NaiveDate};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Machine codes returned in 409 blocker payloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveBlockerCode {
    ActiveChildren,
    ActiveLease,
    OpenWorkOrder,
    FutureReservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveBlocker {
    pub code: ArchiveBlockerCode,
    pub message: String,
    pub entity_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetireReason {
    Replaced,
    Failed,
    Sold,
    Other,
}

impl RetireReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Replaced => "replaced",
            Self::Failed => "failed",
            Self::Sold => "sold",
            Self::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "replaced" => Some(Self::Replaced),
            "failed" => Some(Self::Failed),
            "sold" => Some(Self::Sold),
            "other" => Some(Self::Other),
            _ => None,
        }
    }
}

/// Inclusive month range expansion for period payment batches.
/// Returns first-of-month dates from start..=end, or error if inverted/empty.
pub fn expand_month_range(
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<NaiveDate>, &'static str> {
    let start = NaiveDate::from_ymd_opt(start.year(), start.month(), 1)
        .ok_or("invalid start month")?;
    let end = NaiveDate::from_ymd_opt(end.year(), end.month(), 1).ok_or("invalid end month")?;
    if end < start {
        return Err("end month must be on or after start month");
    }
    let mut out = Vec::new();
    let mut cursor = start;
    // Cap to 120 months to prevent abuse
    for _ in 0..120 {
        out.push(cursor);
        if cursor == end {
            break;
        }
        let (y, m) = if cursor.month() == 12 {
            (cursor.year() + 1, 1)
        } else {
            (cursor.year(), cursor.month() + 1)
        };
        cursor = NaiveDate::from_ymd_opt(y, m, 1).ok_or("month overflow")?;
    }
    if out.last() != Some(&end) {
        return Err("range too large (max 120 months)");
    }
    Ok(out)
}

/// Closed vocabulary for per-asset alert prefs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetAlertType {
    PaymentOverdue,
    PaymentFailed,
    Vacancy,
    LeaseExpiring,
    MaintenanceOpen,
    InspectionDue,
    StrPermitExpiry,
    ViolationFiled,
}

impl AssetAlertType {
    pub const ALL: &'static [Self] = &[
        Self::PaymentOverdue,
        Self::PaymentFailed,
        Self::Vacancy,
        Self::LeaseExpiring,
        Self::MaintenanceOpen,
        Self::InspectionDue,
        Self::StrPermitExpiry,
        Self::ViolationFiled,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::PaymentOverdue => "payment_overdue",
            Self::PaymentFailed => "payment_failed",
            Self::Vacancy => "vacancy",
            Self::LeaseExpiring => "lease_expiring",
            Self::MaintenanceOpen => "maintenance_open",
            Self::InspectionDue => "inspection_due",
            Self::StrPermitExpiry => "str_permit_expiry",
            Self::ViolationFiled => "violation_filed",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.as_str() == s)
    }

    pub fn defaults() -> Vec<&'static str> {
        vec![
            Self::PaymentOverdue.as_str(),
            Self::Vacancy.as_str(),
            Self::ViolationFiled.as_str(),
        ]
    }
}

pub fn validate_alert_types(enabled: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(enabled.len());
    for id in enabled {
        match AssetAlertType::parse(id) {
            Some(t) => out.push(t.as_str().to_string()),
            None => return Err(format!("unknown alert type '{id}'")),
        }
    }
    out.sort();
    out.dedup();
    Ok(out)
}

pub struct AssetArchiveService;

impl AssetArchiveService {
    pub async fn collect_blockers(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Vec<ArchiveBlocker>> {
        use crate::entities::{atlas_asset, atlas_case, atlas_contract, atlas_reservation};

        let mut blockers = Vec::new();

        // Live (non-decommissioned) children
        let children = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::ParentAssetId.eq(asset_id))
            .filter(atlas_asset::Column::Status.ne("decommissioned"))
            .all(db)
            .await?;
        for c in children {
            blockers.push(ArchiveBlocker {
                code: ArchiveBlockerCode::ActiveChildren,
                message: format!("Active child unit: {}", c.name),
                entity_id: Some(c.id),
            });
        }

        let leases = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::AssetId.eq(asset_id))
            .filter(atlas_contract::Column::Status.eq("active"))
            .all(db)
            .await?;
        for l in leases {
            blockers.push(ArchiveBlocker {
                code: ArchiveBlockerCode::ActiveLease,
                message: "Active lease on this asset".into(),
                entity_id: Some(l.id),
            });
        }

        let open_cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::AssetId.eq(asset_id))
            .filter(atlas_case::Column::Status.eq("open"))
            .all(db)
            .await?;
        for c in open_cases {
            blockers.push(ArchiveBlocker {
                code: ArchiveBlockerCode::OpenWorkOrder,
                message: format!("Open case: {}", c.subject),
                entity_id: Some(c.id),
            });
        }

        let now = chrono::Utc::now();
        let future_res = atlas_reservation::Entity::find()
            .filter(atlas_reservation::Column::TenantId.eq(tenant_id))
            .filter(atlas_reservation::Column::ReservedAssetId.eq(asset_id))
            .filter(atlas_reservation::Column::StartsAt.gt(now))
            .filter(atlas_reservation::Column::Status.ne("cancelled"))
            .filter(atlas_reservation::Column::Status.ne("checked_out"))
            .all(db)
            .await?;
        for r in future_res {
            blockers.push(ArchiveBlocker {
                code: ArchiveBlockerCode::FutureReservation,
                message: "Future reservation on this asset".into(),
                entity_id: Some(r.id),
            });
        }

        Ok(blockers)
    }

    /// Soft-archive: set status = decommissioned when no blockers.
    pub async fn archive(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<(), anyhow::Error> {
        use crate::entities::atlas_asset;

        let blockers = Self::collect_blockers(db, tenant_id, asset_id).await?;
        if !blockers.is_empty() {
            return Err(anyhow!("archive_blocked:{} blockers", blockers.len()));
        }

        let row = atlas_asset::Entity::find_by_id(asset_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("asset not found"))?;

        let mut am: atlas_asset::ActiveModel = row.into();
        am.status = Set("decommissioned".to_string());
        am.update(db).await?;
        Ok(())
    }

    /// Retire a system or appliance → `inactive`, optional replace chain in lifecycle_metadata.
    pub async fn retire(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        asset_type: &str,
        reason: RetireReason,
        replaced_by_id: Option<Uuid>,
        notes: Option<String>,
    ) -> Result<(), anyhow::Error> {
        use crate::entities::atlas_asset;

        let row = atlas_asset::Entity::find_by_id(asset_id)
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::AssetType.eq(asset_type))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("{asset_type} not found"))?;

        if let Some(replacement_id) = replaced_by_id {
            let replacement = atlas_asset::Entity::find_by_id(replacement_id)
                .filter(atlas_asset::Column::TenantId.eq(tenant_id))
                .one(db)
                .await?
                .ok_or_else(|| anyhow!("replacement asset not found"))?;

            let mut repl_meta = replacement
                .lifecycle_metadata
                .clone()
                .unwrap_or_else(|| serde_json::json!({}));
            if let Some(obj) = repl_meta.as_object_mut() {
                obj.insert("replaces_id".into(), serde_json::json!(asset_id.to_string()));
            }
            let mut repl_am: atlas_asset::ActiveModel = replacement.into();
            repl_am.lifecycle_metadata = Set(Some(repl_meta));
            repl_am.update(db).await?;
        }

        let mut meta = row
            .lifecycle_metadata
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("retire_reason".into(), serde_json::json!(reason.as_str()));
            if let Some(n) = notes {
                obj.insert("retire_notes".into(), serde_json::json!(n));
            }
            if let Some(rid) = replaced_by_id {
                obj.insert("replaced_by_id".into(), serde_json::json!(rid.to_string()));
            }
        }

        let mut am: atlas_asset::ActiveModel = row.into();
        am.status = Set("inactive".to_string());
        am.lifecycle_metadata = Set(Some(meta));
        am.update(db).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn expand_jan_through_jun_is_six_months() {
        let start = NaiveDate::from_ymd_opt(2025, 1, 15).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 6, 28).unwrap();
        let months = expand_month_range(start, end).unwrap();
        assert_eq!(months.len(), 6);
        assert_eq!(months[0], NaiveDate::from_ymd_opt(2025, 1, 1).unwrap());
        assert_eq!(months[5], NaiveDate::from_ymd_opt(2025, 6, 1).unwrap());
    }

    #[test]
    fn expand_inverted_range_errors() {
        let start = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert!(expand_month_range(start, end).is_err());
    }

    #[test]
    fn alert_types_reject_unknown() {
        let err = validate_alert_types(&[String::from("not_a_real_alert")]).unwrap_err();
        assert!(err.contains("unknown"));
    }

    #[test]
    fn alert_types_accept_known() {
        let ok = validate_alert_types(&[
            String::from("vacancy"),
            String::from("payment_overdue"),
            String::from("vacancy"),
        ])
        .unwrap();
        assert_eq!(ok, vec!["payment_overdue", "vacancy"]);
    }

    #[test]
    fn retire_reason_parse() {
        assert_eq!(RetireReason::parse("replaced"), Some(RetireReason::Replaced));
        assert_eq!(RetireReason::parse("nope"), None);
    }

    #[test]
    fn expand_single_month_is_one() {
        let d = NaiveDate::from_ymd_opt(2025, 3, 10).unwrap();
        let months = expand_month_range(d, d).unwrap();
        assert_eq!(months.len(), 1);
        assert_eq!(months[0], NaiveDate::from_ymd_opt(2025, 3, 1).unwrap());
    }

    #[test]
    fn alert_defaults_are_known_types() {
        for id in AssetAlertType::defaults() {
            assert!(AssetAlertType::parse(id).is_some(), "unknown default {id}");
        }
    }
}
