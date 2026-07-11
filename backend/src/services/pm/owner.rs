//! Folio — Owner Service
//!
//! Read-only visibility for beneficial property owners who have delegated
//! day-to-day management to a PMC. **No write operations.** All queries
//! are scoped to assets / contracts where the owner's user account is
//! linked via `atlas_record_relationships` (G-22, relationship_type =
//! `"owner_user"`) to the asset.
//!
//! # Scoping strategy
//!
//! When a PMC onboards an owner-client, the PM operator records the link:
//!   source_entity_type = "atlas_user"
//!   source_entity_id   = owner_user.id
//!   target_entity_type = "atlas_asset"
//!   target_entity_id   = asset.id
//!   relationship_type  = "owner_user"
//!
//! All owner queries filter assets by this relationship. This gives the owner
//! exactly the assets they own — nothing more, nothing less.
//!
//! # Authorization
//! The `require_owner` middleware (folio.rs) ensures `FolioRole::Owner` before
//! any of these handlers are reached. The service never checks role itself.

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use uuid::Uuid;

use crate::entities::{
    atlas_asset, atlas_case, atlas_contract, atlas_ledger_entry, atlas_record_relationship,
};
use crate::types::pm::PmCaseType;

// ── Constants ─────────────────────────────────────────────────────────────────

pub const OWNER_ASSET_RELATIONSHIP: &str = "owner_user";

// ═══════════════════════════════════════════════════════════════════════════════
// Output types
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct OwnerPortfolioSummary {
    pub generated_at: chrono::DateTime<Utc>,
    pub owner_user_id: Uuid,
    pub total_properties: usize,
    pub occupied_units: usize,
    pub vacant_units: usize,
    pub occupancy_pct: f64,
    pub revenue_this_month_cents: i64,
    pub revenue_ytd_cents: i64,
    pub outstanding_balance_cents: i64,
    pub outstanding_payments: usize,
    pub on_time_payment_rate_pct: f64,
    pub active_leases: usize,
    pub open_maintenance_cases: usize,
    pub open_violations: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnerPropertySummary {
    pub asset_id: Uuid,
    pub asset_name: String,
    pub asset_type: String,
    pub address_line_1: Option<String>,
    pub active_leases: usize,
    pub open_maintenance: usize,
    pub open_violations: usize,
    pub revenue_this_month_cents: i64,
    pub outstanding_balance_cents: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnerLeaseEntry {
    pub contract_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub monthly_rent_cents: Option<i64>,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnerMaintenanceSummary {
    pub case_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub subject: String,
    pub priority: String,
    pub status: String,
    pub created_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OwnerInspectionEntry {
    pub case_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub subject: String,
    pub status: String,
    pub scheduled_at: Option<chrono::DateTime<Utc>>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// OwnerService
// ═══════════════════════════════════════════════════════════════════════════════

pub struct OwnerService;

impl OwnerService {
    // ── Asset resolution ──────────────────────────────────────────────────────

    /// Returns the IDs of all assets linked to this owner via G-22 `owner_user` relationship.
    pub async fn owned_asset_ids(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<Vec<Uuid>> {
        let rels = atlas_record_relationship::Entity::find()
            .filter(atlas_record_relationship::Column::TenantId.eq(tenant_id))
            .filter(atlas_record_relationship::Column::SourceEntityType.eq("atlas_user"))
            .filter(atlas_record_relationship::Column::SourceEntityId.eq(owner_user_id))
            .filter(atlas_record_relationship::Column::TargetEntityType.eq("atlas_asset"))
            .filter(
                atlas_record_relationship::Column::RelationshipType.eq(OWNER_ASSET_RELATIONSHIP),
            )
            .all(db)
            .await?;

        Ok(rels.into_iter().map(|r| r.target_entity_id).collect())
    }

    // ── Portfolio overview ────────────────────────────────────────────────────

    /// Top-level KPI dashboard for the owner.
    pub async fn portfolio_summary(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<OwnerPortfolioSummary> {
        let now = Utc::now();
        let today = now.date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);
        let year_start = NaiveDate::from_ymd_opt(today.year(), 1, 1).unwrap_or(today);

        let asset_ids = Self::owned_asset_ids(db, tenant_id, owner_user_id).await?;
        let total_properties = asset_ids.len();

        if asset_ids.is_empty() {
            return Ok(OwnerPortfolioSummary {
                generated_at: now,
                owner_user_id,
                total_properties: 0,
                occupied_units: 0,
                vacant_units: 0,
                occupancy_pct: 0.0,
                revenue_this_month_cents: 0,
                revenue_ytd_cents: 0,
                outstanding_balance_cents: 0,
                outstanding_payments: 0,
                on_time_payment_rate_pct: 100.0,
                active_leases: 0,
                open_maintenance_cases: 0,
                open_violations: 0,
            });
        }

        // Active leases on owned assets
        let leases = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::ContractType.eq("lease"))
            .filter(atlas_contract::Column::Status.eq("active"))
            .filter(atlas_contract::Column::AssetId.is_in(asset_ids.clone()))
            .all(db)
            .await?;

        let active_leases = leases.len();
        let occupied_units = leases
            .iter()
            .filter_map(|c| c.asset_id)
            .collect::<std::collections::HashSet<_>>()
            .len();
        let vacant_units = total_properties.saturating_sub(occupied_units);
        let occupancy_pct = if total_properties > 0 {
            (occupied_units as f64 / total_properties as f64) * 100.0
        } else {
            0.0
        };

        // Ledger entries for owned assets' leases
        let contract_ids: Vec<Uuid> = leases.iter().map(|c| c.id).collect();
        let entries = if contract_ids.is_empty() {
            vec![]
        } else {
            atlas_ledger_entry::Entity::find()
                .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
                .filter(atlas_ledger_entry::Column::BillableEntityId.is_in(contract_ids))
                .all(db)
                .await?
        };

        let revenue_this_month_cents: i64 = entries
            .iter()
            .filter(|e| e.status == "paid")
            .filter(|e| {
                e.paid_at
                    .map(|p| p.date_naive() >= month_start)
                    .unwrap_or(false)
            })
            .map(|e| e.net_amount_cents)
            .sum();

        let revenue_ytd_cents: i64 = entries
            .iter()
            .filter(|e| e.status == "paid")
            .filter(|e| {
                e.paid_at
                    .map(|p| p.date_naive() >= year_start)
                    .unwrap_or(false)
            })
            .map(|e| e.net_amount_cents)
            .sum();

        let outstanding: Vec<_> = entries
            .iter()
            .filter(|e| e.status == "pending" || e.status == "overdue")
            .collect();
        let outstanding_payments = outstanding.len();
        let outstanding_balance_cents: i64 = outstanding.iter().map(|e| e.gross_amount_cents).sum();

        let paid_with_due: Vec<_> = entries
            .iter()
            .filter(|e| e.status == "paid" && e.due_date.is_some() && e.paid_at.is_some())
            .collect();
        let on_time_payment_rate_pct = if paid_with_due.is_empty() {
            100.0
        } else {
            let on_time = paid_with_due
                .iter()
                .filter(|e| e.paid_at.unwrap().date_naive() <= e.due_date.unwrap())
                .count();
            (on_time as f64 / paid_with_due.len() as f64) * 100.0
        };

        // Open cases on owned assets
        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::Status.eq("open"))
            .filter(atlas_case::Column::AssetId.is_in(asset_ids))
            .all(db)
            .await?;

        let open_maintenance_cases = cases
            .iter()
            .filter(|c| c.case_type == PmCaseType::Maintenance.to_string())
            .count();
        let open_violations = cases
            .iter()
            .filter(|c| c.case_type == PmCaseType::ComplianceViolation.to_string())
            .count();

        Ok(OwnerPortfolioSummary {
            generated_at: now,
            owner_user_id,
            total_properties,
            occupied_units,
            vacant_units,
            occupancy_pct,
            revenue_this_month_cents,
            revenue_ytd_cents,
            outstanding_balance_cents,
            outstanding_payments,
            on_time_payment_rate_pct,
            active_leases,
            open_maintenance_cases,
            open_violations,
        })
    }

    // ── Per-property drilldown ────────────────────────────────────────────────

    /// List summary stats for each owned property.
    pub async fn list_properties(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<Vec<OwnerPropertySummary>> {
        let asset_ids = Self::owned_asset_ids(db, tenant_id, owner_user_id).await?;
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let assets = atlas_asset::Entity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::Id.is_in(asset_ids.clone()))
            .all(db)
            .await?;

        let today = Utc::now().date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap_or(today);

        let mut summaries = Vec::with_capacity(assets.len());

        for asset in assets {
            let asset_leases = atlas_contract::Entity::find()
                .filter(atlas_contract::Column::TenantId.eq(tenant_id))
                .filter(atlas_contract::Column::AssetId.eq(asset.id))
                .filter(atlas_contract::Column::ContractType.eq("lease"))
                .filter(atlas_contract::Column::Status.eq("active"))
                .all(db)
                .await?;

            let contract_ids: Vec<Uuid> = asset_leases.iter().map(|c| c.id).collect();

            let (revenue_this_month_cents, outstanding_balance_cents) = if contract_ids.is_empty() {
                (0i64, 0i64)
            } else {
                let entries = atlas_ledger_entry::Entity::find()
                    .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
                    .filter(atlas_ledger_entry::Column::BillableEntityId.is_in(contract_ids))
                    .all(db)
                    .await?;

                let rev: i64 = entries
                    .iter()
                    .filter(|e| e.status == "paid")
                    .filter(|e| {
                        e.paid_at
                            .map(|p| p.date_naive() >= month_start)
                            .unwrap_or(false)
                    })
                    .map(|e| e.net_amount_cents)
                    .sum();
                let outstanding: i64 = entries
                    .iter()
                    .filter(|e| e.status == "pending" || e.status == "overdue")
                    .map(|e| e.gross_amount_cents)
                    .sum();
                (rev, outstanding)
            };

            let cases = atlas_case::Entity::find()
                .filter(atlas_case::Column::TenantId.eq(tenant_id))
                .filter(atlas_case::Column::AssetId.eq(asset.id))
                .filter(atlas_case::Column::Status.eq("open"))
                .all(db)
                .await?;

            summaries.push(OwnerPropertySummary {
                asset_id: asset.id,
                asset_name: asset.name,
                asset_type: asset.asset_type,
                address_line_1: asset.address_line_1,
                active_leases: asset_leases.len(),
                open_maintenance: cases
                    .iter()
                    .filter(|c| c.case_type == PmCaseType::Maintenance.to_string())
                    .count(),
                open_violations: cases
                    .iter()
                    .filter(|c| c.case_type == PmCaseType::ComplianceViolation.to_string())
                    .count(),
                revenue_this_month_cents,
                outstanding_balance_cents,
            });
        }

        Ok(summaries)
    }

    /// Active leases across all owned assets.
    pub async fn list_leases(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<Vec<OwnerLeaseEntry>> {
        let asset_ids = Self::owned_asset_ids(db, tenant_id, owner_user_id).await?;
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let leases = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::ContractType.eq("lease"))
            .filter(atlas_contract::Column::AssetId.is_in(asset_ids))
            .order_by_desc(atlas_contract::Column::StartDate)
            .all(db)
            .await?;

        Ok(leases
            .into_iter()
            .map(|c| OwnerLeaseEntry {
                contract_id: c.id,
                asset_id: c.asset_id,
                start_date: c.start_date,
                end_date: c.end_date,
                status: c.status,
                monthly_rent_cents: c.recurring_amount_cents,
                currency: c.currency,
            })
            .collect())
    }

    /// Open maintenance cases on owned assets.
    pub async fn list_maintenance(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<Vec<OwnerMaintenanceSummary>> {
        let asset_ids = Self::owned_asset_ids(db, tenant_id, owner_user_id).await?;
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::Maintenance.to_string()))
            .filter(atlas_case::Column::AssetId.is_in(asset_ids))
            .order_by_desc(atlas_case::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(cases
            .into_iter()
            .map(|c| OwnerMaintenanceSummary {
                case_id: c.id,
                asset_id: c.asset_id,
                subject: c.subject,
                priority: c.priority,
                status: c.status,
                created_at: c.created_at,
                completed_at: c.completed_at,
            })
            .collect())
    }

    /// Scheduled and completed inspections on owned assets.
    pub async fn list_inspections(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
    ) -> Result<Vec<OwnerInspectionEntry>> {
        let asset_ids = Self::owned_asset_ids(db, tenant_id, owner_user_id).await?;
        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ScheduledInspection.to_string()))
            .filter(atlas_case::Column::AssetId.is_in(asset_ids))
            .order_by_desc(atlas_case::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(cases
            .into_iter()
            .map(|c| {
                let scheduled_at = c
                    .case_metadata
                    .as_ref()
                    .and_then(|m| m["scheduled_date"].as_str())
                    .and_then(|s| s.parse().ok());
                OwnerInspectionEntry {
                    case_id: c.id,
                    asset_id: c.asset_id,
                    subject: c.subject,
                    status: c.status,
                    scheduled_at,
                    completed_at: c.completed_at,
                }
            })
            .collect())
    }

    // ── Owner-asset link management (PMC-only write operation) ────────────────

    /// PMC operator links an owner user account to an asset they own.
    /// This is the only write operation — called during PMC client onboarding.
    /// Returns the relationship ID.
    pub async fn link_owner_to_asset(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        pm_user_id: Uuid,
        owner_user_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Uuid> {
        use crate::entities::atlas_record_relationship;
        use sea_orm::{ActiveModelTrait, Set};

        let rel_id = Uuid::new_v4();
        let now = Utc::now();

        atlas_record_relationship::ActiveModel {
            id: Set(rel_id),
            tenant_id: Set(tenant_id),
            source_entity_type: Set("atlas_user".to_string()),
            source_entity_id: Set(owner_user_id),
            target_entity_type: Set("atlas_asset".to_string()),
            target_entity_id: Set(asset_id),
            relationship_type: Set(OWNER_ASSET_RELATIONSHIP.to_string()),
            relationship_metadata: Set(Some(serde_json::json!({
                "linked_by": pm_user_id,
                "linked_at": now.to_rfc3339(),
            }))),
            created_at: Set(now),
            ..Default::default()
        }
        .insert(db)
        .await?;

        tracing::info!(
            rel_id = %rel_id, %tenant_id, %owner_user_id, %asset_id,
            "OwnerService: owner linked to asset"
        );

        Ok(rel_id)
    }
}
