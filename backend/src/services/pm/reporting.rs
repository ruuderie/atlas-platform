//! Folio — Reporting Service
//!
//! Aggregation queries that build portable tenant reports and landlord analytics.
//! All queries run against existing tables — no new migrations.
//!
//! # Report types
//!
//! ## Tenant-facing (self-service via ReportRequest case)
//! - `rental_history`   — all leases the tenant has held, with dates and addresses
//! - `payment_history`  — all ledger entries (rent due, paid_at, on-time flag)
//! - `violation_history`— all compliance violations on the tenant's units
//! - `full_export`      — all three bundled (GDPR/CCPA right-to-portability)
//!
//! ## Landlord analytics (scoped by tenant_id)
//! - `landlord_overview` — revenue, occupancy, maintenance, late payment metrics
//!
//! ## Vendor analytics (scoped by service_provider_id)
//! - `vendor_overview` — work orders completed, revenue, avg response time

use anyhow::Result;
use chrono::{Datelike, NaiveDate, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Serialize;
use uuid::Uuid;

use crate::entities::{atlas_case, atlas_contract, atlas_ledger_entry};
use crate::types::pm::PmCaseType;

// ═══════════════════════════════════════════════════════════════════════════════
// Report request type — stored in atlas_cases.case_metadata
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    RentalHistory,
    PaymentHistory,
    ViolationHistory,
    FullExport,
}

impl std::fmt::Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::RentalHistory    => "rental_history",
            Self::PaymentHistory   => "payment_history",
            Self::ViolationHistory => "violation_history",
            Self::FullExport       => "full_export",
        })
    }
}

impl TryFrom<String> for ReportType {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "rental_history"    => Ok(Self::RentalHistory),
            "payment_history"   => Ok(Self::PaymentHistory),
            "violation_history" => Ok(Self::ViolationHistory),
            "full_export"       => Ok(Self::FullExport),
            other => Err(format!("unknown ReportType: '{other}'")),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report data structs
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize)]
pub struct LeaseHistoryEntry {
    pub contract_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub status: String,
    pub currency: String,
    pub monthly_rent_cents: Option<i64>,
    pub signed_at: Option<chrono::DateTime<Utc>>,
    pub terminated_at: Option<chrono::DateTime<Utc>>,
    pub termination_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PaymentHistoryEntry {
    pub ledger_id: Uuid,
    pub gross_amount_cents: i64,
    pub currency: String,
    pub due_date: Option<NaiveDate>,
    pub paid_at: Option<chrono::DateTime<Utc>>,
    /// True if payment was made on or before the due date.
    pub paid_on_time: bool,
    /// Days late (negative = paid early). None if not yet paid.
    pub days_late: Option<i64>,
    pub payment_rail: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ViolationHistoryEntry {
    pub case_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub category: String,
    pub subject: String,
    pub cure_status: String,
    pub cure_deadline: Option<NaiveDate>,
    pub filed_at: chrono::DateTime<Utc>,
    pub resolved_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TenantReport {
    pub report_type: String,
    pub generated_at: chrono::DateTime<Utc>,
    pub tenant_user_id: Uuid,
    pub rental_history: Vec<LeaseHistoryEntry>,
    pub payment_history: Vec<PaymentHistoryEntry>,
    pub violation_history: Vec<ViolationHistoryEntry>,
}

// ── Landlord analytics ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct LandlordOverview {
    pub generated_at: chrono::DateTime<Utc>,
    /// Total rent collected this calendar month (cents).
    pub revenue_this_month_cents: i64,
    /// Total rent collected this calendar year (cents).
    pub revenue_this_year_cents: i64,
    /// Count of active leases.
    pub active_leases: i64,
    /// Count of open maintenance cases.
    pub open_maintenance_cases: i64,
    /// Count of open violations.
    pub open_violations: i64,
    /// Count of payments past due_date but not yet paid.
    pub outstanding_payments: i64,
    /// Outstanding balance in cents.
    pub outstanding_balance_cents: i64,
    /// Percentage of payments made on time (last 12 months).
    pub on_time_payment_rate_pct: f64,
}

// ── Vendor analytics ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct VendorOverview {
    pub generated_at: chrono::DateTime<Utc>,
    pub service_provider_id: Uuid,
    pub completed_work_orders: i64,
    pub open_work_orders: i64,
    pub total_revenue_cents: i64,
    /// Average days between case creation and completion.
    pub avg_close_days: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ReportingService
// ═══════════════════════════════════════════════════════════════════════════════

pub struct ReportingService;

impl ReportingService {
    // ── Tenant report request ─────────────────────────────────────────────────

    /// Creates a `report_request` case in atlas_cases and returns the case ID.
    /// The case starts as `status = "pending"`. A background job (Phase 4) will
    /// pick it up, generate the report, attach the file, and set `status = "ready"`.
    ///
    /// For now, returns the fully-computed report synchronously (no background job).
    /// Phase 4 will move generation off-request.
    pub async fn request_report(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        requesting_user_id: Uuid,
        report_type: ReportType,
    ) -> Result<(Uuid, TenantReport)> {
        use crate::entities::atlas_case;

        let case_id = Uuid::new_v4();
        let now = Utc::now();

        // Generate the report synchronously
        let report = Self::generate_tenant_report(
            db, tenant_id, requesting_user_id, &report_type,
        ).await?;

        let metadata = serde_json::json!({
            "report_type":       report_type.to_string(),
            "requested_by_role": "tenant",
            "status":            "ready",
            "generated_at":      now.to_rfc3339(),
        });

        atlas_case::ActiveModel {
            id: Set(case_id),
            tenant_id: Set(tenant_id),
            case_type: Set(PmCaseType::ReportRequest.to_string()),
            reported_by_user_id: Set(Some(requesting_user_id)),
            subject: Set(format!("{} report", report_type)),
            status: Set("ready".to_string()),
            priority: Set("routine".to_string()),
            case_metadata: Set(Some(metadata)),
            created_at: Set(now),
            completed_at: Set(Some(now)),
            ..Default::default()
        }
        .insert(db)
        .await?;

        tracing::info!(case_id = %case_id, %tenant_id, user_id = %requesting_user_id,
            %report_type, "ReportingService: report generated");

        Ok((case_id, report))
    }

    /// List all past report requests by this tenant user.
    pub async fn list_report_requests(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        requesting_user_id: Uuid,
    ) -> Result<Vec<atlas_case::Model>> {
        Ok(atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ReportRequest.to_string()))
            .filter(atlas_case::Column::ReportedByUserId.eq(requesting_user_id))
            .order_by_desc(atlas_case::Column::CreatedAt)
            .all(db)
            .await?)
    }

    // ── Landlord analytics ────────────────────────────────────────────────────

    /// Compute landlord-wide KPIs from ledger entries and cases.
    pub async fn landlord_overview(
        db: &DatabaseConnection,
        tenant_id: Uuid,
    ) -> Result<LandlordOverview> {
        let now = Utc::now();
        let today = now.date_naive();
        let month_start = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .unwrap_or(today);
        let year_start = NaiveDate::from_ymd_opt(today.year(), 1, 1)
            .unwrap_or(today);

        // All ledger entries for this landlord tenant
        let entries = atlas_ledger_entry::Entity::find()
            .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
            .filter(atlas_ledger_entry::Column::BillableEntityType.eq("atlas_contract"))
            .all(db)
            .await?;

        let revenue_this_month_cents: i64 = entries.iter()
            .filter(|e| e.status == "paid")
            .filter(|e| e.paid_at.map(|p| p.date_naive() >= month_start).unwrap_or(false))
            .map(|e| e.net_amount_cents)
            .sum();

        let revenue_this_year_cents: i64 = entries.iter()
            .filter(|e| e.status == "paid")
            .filter(|e| e.paid_at.map(|p| p.date_naive() >= year_start).unwrap_or(false))
            .map(|e| e.net_amount_cents)
            .sum();

        let outstanding_entries: Vec<_> = entries.iter()
            .filter(|e| e.status == "pending" || e.status == "overdue")
            .collect();
        let outstanding_payments = outstanding_entries.len() as i64;
        let outstanding_balance_cents: i64 = outstanding_entries.iter()
            .map(|e| e.gross_amount_cents)
            .sum();

        // On-time rate: of paid entries with a due_date, how many were paid ≤ due_date?
        let paid_with_due: Vec<_> = entries.iter()
            .filter(|e| e.status == "paid" && e.due_date.is_some() && e.paid_at.is_some())
            .collect();
        let on_time_payment_rate_pct = if paid_with_due.is_empty() {
            100.0
        } else {
            let on_time = paid_with_due.iter()
                .filter(|e| {
                    let due = e.due_date.unwrap();
                    let paid = e.paid_at.unwrap().date_naive();
                    paid <= due
                })
                .count();
            (on_time as f64 / paid_with_due.len() as f64) * 100.0
        };

        // Active leases
        let active_leases = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::ContractType.eq("lease"))
            .filter(atlas_contract::Column::Status.eq("active"))
            .all(db)
            .await?
            .len() as i64;

        // Open maintenance + violations
        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::Status.eq("open"))
            .all(db)
            .await?;

        let open_maintenance_cases = cases.iter()
            .filter(|c| c.case_type == PmCaseType::Maintenance.to_string())
            .count() as i64;
        let open_violations = cases.iter()
            .filter(|c| c.case_type == PmCaseType::ComplianceViolation.to_string())
            .count() as i64;

        Ok(LandlordOverview {
            generated_at: now,
            revenue_this_month_cents,
            revenue_this_year_cents,
            active_leases,
            open_maintenance_cases,
            open_violations,
            outstanding_payments,
            outstanding_balance_cents,
            on_time_payment_rate_pct,
        })
    }

    // ── Vendor analytics ──────────────────────────────────────────────────────

    pub async fn vendor_overview(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        service_provider_id: Uuid,
    ) -> Result<VendorOverview> {
        let now = Utc::now();

        let all_cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::AssignedServiceProviderId.eq(service_provider_id))
            .all(db)
            .await?;

        let completed: Vec<_> = all_cases.iter()
            .filter(|c| c.status == "completed" && c.completed_at.is_some())
            .collect();
        let open = all_cases.iter().filter(|c| c.status == "open").count() as i64;

        let avg_close_days = if completed.is_empty() {
            None
        } else {
            let total_days: f64 = completed.iter()
                .filter_map(|c| {
                    let completed_at = c.completed_at?;
                    let days = (completed_at - c.created_at).num_hours() as f64 / 24.0;
                    Some(days)
                })
                .sum();
            Some(total_days / completed.len() as f64)
        };

        let total_revenue_cents: i64 = all_cases.iter()
            .filter_map(|c| c.actual_cost_cents)
            .sum();

        Ok(VendorOverview {
            generated_at: now,
            service_provider_id,
            completed_work_orders: completed.len() as i64,
            open_work_orders: open,
            total_revenue_cents,
            avg_close_days,
        })
    }

    // ── Private: generate tenant report ──────────────────────────────────────

    async fn generate_tenant_report(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
        report_type: &ReportType,
    ) -> Result<TenantReport> {
        let now = Utc::now();

        let rental_history = if matches!(
            report_type,
            ReportType::RentalHistory | ReportType::FullExport
        ) {
            Self::build_rental_history(db, tenant_id, user_id).await?
        } else {
            vec![]
        };

        let payment_history = if matches!(
            report_type,
            ReportType::PaymentHistory | ReportType::FullExport
        ) {
            Self::build_payment_history(db, tenant_id, user_id).await?
        } else {
            vec![]
        };

        let violation_history = if matches!(
            report_type,
            ReportType::ViolationHistory | ReportType::FullExport
        ) {
            Self::build_violation_history(db, tenant_id, user_id).await?
        } else {
            vec![]
        };

        Ok(TenantReport {
            report_type: report_type.to_string(),
            generated_at: now,
            tenant_user_id: user_id,
            rental_history,
            payment_history,
            violation_history,
        })
    }

    async fn build_rental_history(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<LeaseHistoryEntry>> {
        let leases = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::CounterpartyUserId.eq(user_id))
            .filter(atlas_contract::Column::ContractType.eq("lease"))
            .order_by_asc(atlas_contract::Column::StartDate)
            .all(db)
            .await?;

        Ok(leases.into_iter().map(|c| LeaseHistoryEntry {
            contract_id: c.id,
            asset_id: c.asset_id,
            start_date: c.start_date,
            end_date: c.end_date,
            status: c.status,
            currency: c.currency,
            monthly_rent_cents: c.recurring_amount_cents,
            signed_at: c.signed_at,
            terminated_at: c.terminated_at,
            termination_reason: c.termination_reason,
        }).collect())
    }

    async fn build_payment_history(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<PaymentHistoryEntry>> {
        let entries = atlas_ledger_entry::Entity::find()
            .filter(atlas_ledger_entry::Column::TenantId.eq(tenant_id))
            .filter(atlas_ledger_entry::Column::PayerUserId.eq(user_id))
            .order_by_asc(atlas_ledger_entry::Column::DueDate)
            .all(db)
            .await?;

        Ok(entries.into_iter().map(|e| {
            let (paid_on_time, days_late) = match (e.due_date, e.paid_at) {
                (Some(due), Some(paid)) => {
                    let paid_date = paid.date_naive();
                    let diff = (paid_date - due).num_days();
                    (diff <= 0, Some(diff))
                }
                _ => (false, None),
            };
            PaymentHistoryEntry {
                ledger_id: e.id,
                gross_amount_cents: e.gross_amount_cents,
                currency: e.currency,
                due_date: e.due_date,
                paid_at: e.paid_at,
                paid_on_time,
                days_late,
                payment_rail: e.payment_rail,
                status: e.status,
            }
        }).collect())
    }

    async fn build_violation_history(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        user_id: Uuid,
    ) -> Result<Vec<ViolationHistoryEntry>> {
        // Get asset IDs from this tenant's leases
        let asset_ids: Vec<Uuid> = atlas_contract::Entity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::CounterpartyUserId.eq(user_id))
            .filter(atlas_contract::Column::ContractType.eq("lease"))
            .all(db)
            .await?
            .into_iter()
            .filter_map(|c| c.asset_id)
            .collect();

        if asset_ids.is_empty() {
            return Ok(vec![]);
        }

        let cases = atlas_case::Entity::find()
            .filter(atlas_case::Column::TenantId.eq(tenant_id))
            .filter(atlas_case::Column::CaseType.eq(PmCaseType::ComplianceViolation.to_string()))
            .filter(atlas_case::Column::AssetId.is_in(asset_ids))
            .order_by_asc(atlas_case::Column::CreatedAt)
            .all(db)
            .await?;

        Ok(cases.into_iter().map(|c| {
            let meta = c.case_metadata.as_ref();
            ViolationHistoryEntry {
                case_id: c.id,
                asset_id: c.asset_id,
                category: meta.and_then(|m| m["category"].as_str()).unwrap_or("other").to_string(),
                subject: c.subject,
                cure_status: c.status,
                cure_deadline: meta
                    .and_then(|m| m["cure_deadline"].as_str())
                    .and_then(|s| s.parse().ok()),
                filed_at: c.created_at,
                resolved_at: meta
                    .and_then(|m| m["resolved_at"].as_str())
                    .and_then(|s| s.parse().ok()),
            }
        }).collect())
    }
}

