//! # G25 CommissionService — Commission Plan Application & Payout Split Computation
//!
//! Reads `atlas_commission_plans` + `atlas_commission_plan_splits` (G25 schema)
//! and computes how a transaction amount should be split between agents/brokers/platform.
//!
//! ## Computation models (split_basis column)
//!
//! | split_basis | Formula |
//! |-------------|---------|
//! | `flat_fee` | Fixed `cap_cents` regardless of transaction size |
//! | `gross_percentage` | `amount × split_rate / 100` |
//! | `net_percentage` | `(amount - deductions) × split_rate / 100` |
//! | `tiered` | Bracket lookup on plan `tiers` JSONB |
//! | `remainder` | The leftover after all other splits (use `is_remainder = true`) |
//!
//! The plan-level `cap_cents` and `minimum_cents` apply to the TOTAL commission.
//! Split-level `cap_cents` caps an individual recipient's earnings per transaction.

use anyhow::{anyhow, Result};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::entities::{atlas_commission_plan, atlas_commission_plan_split};

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct ComputedSplit {
    pub split_id: Uuid,
    pub recipient_type: String,
    pub recipient_account_id: Option<Uuid>,
    pub recipient_label: Option<String>,
    pub split_basis: String,
    pub gross_amount_cents: i64,
    /// After per-split cap enforcement.
    pub net_amount_cents: i64,
    pub capped: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommissionComputation {
    pub plan_id: Uuid,
    pub transaction_amount_cents: i64,
    pub splits: Vec<ComputedSplit>,
    pub total_commission_cents: i64,
    /// `transaction_amount_cents - total_commission_cents`
    pub platform_retain_cents: i64,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct CommissionService;

impl CommissionService {
    pub async fn get_plan(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        plan_id: Uuid,
    ) -> Result<atlas_commission_plan::Model> {
        atlas_commission_plan::Entity::find_by_id(plan_id)
            .filter(atlas_commission_plan::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Commission plan {plan_id} not found"))
    }

    pub async fn list_plans(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        active_only: bool,
    ) -> Result<Vec<atlas_commission_plan::Model>> {
        let mut q = atlas_commission_plan::Entity::find()
            .filter(atlas_commission_plan::Column::TenantId.eq(tenant_id));
        if active_only {
            q = q.filter(atlas_commission_plan::Column::IsActive.eq(true));
        }
        Ok(q.all(db).await?)
    }

    pub async fn get_splits(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        plan_id: Uuid,
    ) -> Result<Vec<atlas_commission_plan_split::Model>> {
        Ok(
            atlas_commission_plan_split::Entity::find()
                .filter(atlas_commission_plan_split::Column::TenantId.eq(tenant_id))
                .filter(atlas_commission_plan_split::Column::PlanId.eq(plan_id))
                .order_by_asc(atlas_commission_plan_split::Column::Priority)
                .all(db)
                .await?,
        )
    }

    // ── Commission computation ─────────────────────────────────────────────────

    /// Compute how `transaction_amount_cents` distributes under `plan_id`.
    ///
    /// `deduction_cents` — subtracted before net-percentage basis computation.
    pub async fn compute(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        plan_id: Uuid,
        transaction_amount_cents: i64,
        deduction_cents: i64,
    ) -> Result<CommissionComputation> {
        let plan = Self::get_plan(db, tenant_id, plan_id).await?;
        let splits = Self::get_splits(db, tenant_id, plan_id).await?;

        let net_base = (transaction_amount_cents - deduction_cents).max(0);

        let mut computed: Vec<ComputedSplit> = Vec::new();
        let mut total: i64 = 0;
        let mut running_remainder = transaction_amount_cents;

        for split in &splits {
            // split_rate is stored as NUMERIC (e.g. 3.0000 = 3%)
            let rate: f64 = split.split_rate.to_string().parse().unwrap_or(0.0);

            let gross: i64 = if split.is_remainder {
                running_remainder
            } else {
                match split.split_basis.as_str() {
                    "gross_percentage" => {
                        (transaction_amount_cents as f64 * rate / 100.0) as i64
                    }
                    "net_percentage" => (net_base as f64 * rate / 100.0) as i64,
                    "flat_fee" => split.cap_cents.unwrap_or(0),
                    "tiered" => Self::compute_tiered(transaction_amount_cents, &plan.tiers),
                    _ => (transaction_amount_cents as f64 * rate / 100.0) as i64,
                }
            };

            // Per-split cap
            let (net, capped) = match split.cap_cents {
                Some(cap) if !split.is_remainder && gross > cap => (cap, true),
                _ => (gross, false),
            };

            running_remainder = running_remainder.saturating_sub(net);
            total += net;

            computed.push(ComputedSplit {
                split_id: split.id,
                recipient_type: split.recipient_type.clone(),
                recipient_account_id: split.recipient_account_id,
                recipient_label: split.recipient_label.clone(),
                split_basis: split.split_basis.clone(),
                gross_amount_cents: gross,
                net_amount_cents: net,
                capped,
            });
        }

        // Plan-level minimum enforcement
        if let Some(min) = plan.minimum_cents {
            if total < min {
                total = min;
            }
        }

        // Plan-level cap enforcement
        if let Some(cap) = plan.cap_cents {
            if total > cap {
                total = cap;
            }
        }

        Ok(CommissionComputation {
            plan_id,
            transaction_amount_cents,
            splits: computed,
            total_commission_cents: total,
            platform_retain_cents: transaction_amount_cents.saturating_sub(total),
        })
    }

    // ── Tiered computation ─────────────────────────────────────────────────────

    /// Reads plan.tiers JSONB:
    /// ```json
    /// [{"up_to_cents": 500000, "rate_pct": 3.0}, {"up_to_cents": null, "rate_pct": 2.5}]
    /// ```
    fn compute_tiered(amount_cents: i64, tiers: &Option<serde_json::Value>) -> i64 {
        let Some(config) = tiers else { return 0 };
        let Some(arr) = config.as_array() else { return 0 };

        let mut remaining = amount_cents;
        let mut result: i64 = 0;
        let mut prev: i64 = 0;

        for tier in arr {
            let up_to = tier.get("up_to_cents").and_then(|v| v.as_i64());
            let rate = tier.get("rate_pct").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let bracket = match up_to {
                Some(limit) => (limit - prev).min(remaining).max(0),
                None => remaining,
            };

            result += (bracket as f64 * rate / 100.0) as i64;
            remaining -= bracket;

            if let Some(limit) = up_to {
                prev = limit;
            }
            if remaining <= 0 { break; }
        }

        result
    }
}
