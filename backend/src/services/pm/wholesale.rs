//! Folio — Wholesale Service (PM wrapper over G-15 `atlas_opportunities`)
//!
//! MAO calculator, wholesale lead pipeline, Kanban stage transitions.
//!
//! # MAO Formula
//! `MAO = (ARV × multiplier) − estimated_repairs − wholesale_fee`
//!
//! The 70% rule is the standard US investor metric. Operators may override
//! the multiplier (0.65–0.75 range) via `folio_mao_multiplier` tenant setting.
//!
//! # Entity field map (`atlas_opportunities`)
//!   `opportunity_type`  → PmOpportunityType::WholesaleLead
//!   `status`            → WholesaleStage (stored here — no separate `stage` column)
//!   `deal_amount_cents` → MAO result
//!   `currency`          → ISO 4217 (not `currency_code`)
//!   `owner_user_id`     → assigned analyst (not `assigned_to_user_id`)
//!   `financial_inputs`  → JSONB: ARV, repairs, motivation, multiplier
//!   `computed_outputs`  → JSONB: MAO, is_viable

use anyhow::{anyhow, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use std::str::FromStr;

use crate::types::pm::{WholesaleStage, SellerMotivation, PmOpportunityType};
use crate::services::pm::scorecard_provisioner::get_pm_template;
use crate::services::scorecard_service::ScorecardService;

// ── MAO result type ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaoResult {
    pub arv_cents: i64,
    pub repair_cents: i64,
    pub wholesale_fee_cents: i64,
    pub multiplier: Decimal,
    /// Maximum Allowable Offer in cents. Negative = deal not viable.
    pub mao_cents: i64,
    pub is_viable: bool,
    pub currency: String,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct WholesaleService;

impl WholesaleService {
    /// Calculate the Maximum Allowable Offer.
    ///
    /// `MAO = (ARV × multiplier) − repairs − wholesale_fee`
    /// Default multiplier: 0.70.
    pub fn calculate_mao(
        arv_cents: i64,
        repair_cents: i64,
        wholesale_fee_cents: i64,
        multiplier: Option<Decimal>,
        currency: Option<String>,
    ) -> MaoResult {
        let m = multiplier.unwrap_or_else(|| Decimal::from_str("0.70").unwrap());
        let mao = (Decimal::from(arv_cents) * m)
            - Decimal::from(repair_cents)
            - Decimal::from(wholesale_fee_cents);
        let mao_cents = mao.to_i64().unwrap_or(i64::MIN);

        MaoResult {
            arv_cents,
            repair_cents,
            wholesale_fee_cents,
            multiplier: m,
            mao_cents,
            is_viable: mao_cents > 0,
            currency: currency.unwrap_or_else(|| "USD".to_string()),
        }
    }

    /// Create a new wholesale lead in `atlas_opportunities`.
    ///
    /// Stage is stored in `status` (the entity has no separate `stage` column).
    /// Financial details live in `financial_inputs` / `computed_outputs` JSONB.
    pub async fn create_lead(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        address: &str,
        arv_cents: i64,
        repair_cents: i64,
        motivation: SellerMotivation,
        owner_user_id: Option<Uuid>,
    ) -> Result<Uuid> {
        use sea_orm::{Set, ActiveModelTrait};
        use chrono::Utc;

        let id = Uuid::new_v4();
        let now = Utc::now();
        let mao = Self::calculate_mao(arv_cents, repair_cents, 5_000_00, None, None);

        let financial_inputs = serde_json::json!({
            "address": address,
            "arv_cents": arv_cents,
            "repair_cents": repair_cents,
            "motivation": motivation.to_string(),
            "multiplier": "0.70",
        });

        let computed_outputs = serde_json::json!({
            "mao_cents": mao.mao_cents,
            "is_viable": mao.is_viable,
        });

        let model = crate::entities::atlas_opportunity::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            opportunity_type: Set(PmOpportunityType::WholesaleLead.to_string()),
            name: Set(format!("Wholesale Lead — {address}")),
            // `status` stores the WholesaleStage — entity has no separate `stage` column
            status: Set(WholesaleStage::New.to_string()),
            deal_amount_cents: Set(Some(mao.mao_cents.max(0))),
            currency: Set("USD".to_string()),
            owner_user_id: Set(owner_user_id),
            financial_inputs: Set(Some(financial_inputs)),
            computed_outputs: Set(Some(computed_outputs)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        tracing::info!(
            opportunity_id = %id, %tenant_id,
            %address,
            mao_viable = mao.is_viable,
            "WholesaleService: wholesale lead created"
        );

        // Phase 2: auto-provision the "Lead Quality Assessment" G-27 scorecard.
        //
        // Scope = tenant (private per operator — excluded from cross-tenant pool).
        // Operators rate their leads on motivation strength, ARV confidence,
        // repair accuracy, and negotiation leverage. Non-fatal on template miss.
        match get_pm_template(db, tenant_id, "Lead Quality Assessment").await {
            Ok(template) => {
                match ScorecardService::get_or_create(
                    db,
                    tenant_id,
                    template.id,
                    "atlas_opportunity",
                    id,
                ).await {
                    Ok(scorecard_id) => {
                        tracing::info!(
                            opportunity_id = %id, %tenant_id,
                            %scorecard_id,
                            "WholesaleService: Lead Quality G-27 scorecard provisioned"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            opportunity_id = %id, %tenant_id,
                            "WholesaleService: G-27 scorecard creation failed (non-fatal): {e:#}"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    opportunity_id = %id, %tenant_id,
                    "WholesaleService: 'Lead Quality Assessment' template not found — was FolioApp::provision() called? {e:#}"
                );
            }
        }

        Ok(id)
    }

    /// Qualify a lead — advances `status` from `New` → `Qualified`.
    /// Returns an error if already in a terminal stage.
    pub async fn qualify_lead(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
    ) -> Result<()> {
        Self::advance_stage(db, tenant_id, opportunity_id, WholesaleStage::Qualified).await
    }

    /// Advance a lead's Kanban stage (stored in `atlas_opportunities.status`).
    /// Terminal stages (`Closed`/`Dead`) cannot be changed.
    pub async fn advance_stage(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        new_stage: WholesaleStage,
    ) -> Result<()> {
        use sea_orm::{EntityTrait, ActiveModelTrait, Set, IntoActiveModel};

        let opp = crate::entities::atlas_opportunity::Entity::find_by_id(opportunity_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Opportunity {opportunity_id} not found"))?;

        if opp.tenant_id != tenant_id {
            return Err(anyhow!("Opportunity {opportunity_id} not found for tenant {tenant_id}"));
        }

        // `status` holds the stage string — parse via the WholesaleStage enum
        let current = WholesaleStage::try_from(opp.status.clone())
            .map_err(|e| anyhow!("Invalid stage in DB: {e}"))?;

        if current.is_terminal() {
            return Err(anyhow!(
                "Cannot advance a {} lead — terminal stage",
                opp.status
            ));
        }

        let mut active = opp.into_active_model();
        active.status = Set(new_stage.to_string());
        active.update(db).await?;

        tracing::info!(%opportunity_id, %tenant_id, stage = %new_stage, "WholesaleService: stage advanced");
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mao_standard_deal() {
        // $150,000 ARV, $20,000 repairs, $5,000 fee → MAO = $80,000
        let r = WholesaleService::calculate_mao(150_000_00, 20_000_00, 5_000_00, None, None);
        assert_eq!(r.mao_cents, 80_000_00);
        assert!(r.is_viable);
    }

    #[test]
    fn test_mao_not_viable() {
        let r = WholesaleService::calculate_mao(100_000_00, 80_000_00, 5_000_00, None, None);
        assert!(r.mao_cents < 0);
        assert!(!r.is_viable);
    }

    #[test]
    fn test_mao_custom_multiplier() {
        let r = WholesaleService::calculate_mao(
            200_000_00, 15_000_00, 7_500_00,
            Some(Decimal::from_str("0.65").unwrap()),
            Some("USD".to_string()),
        );
        assert_eq!(r.mao_cents, 107_500_00);
        assert_eq!(r.multiplier, Decimal::from_str("0.65").unwrap());
    }

    #[test]
    fn test_stage_terminal_guard() {
        assert!(WholesaleStage::Closed.is_terminal());
        assert!(WholesaleStage::Dead.is_terminal());
        assert!(!WholesaleStage::UnderContract.is_terminal());
    }

    #[test]
    fn test_stage_roundtrip() {
        let s = WholesaleStage::UnderContract;
        let parsed = WholesaleStage::try_from(s.to_string()).unwrap();
        assert_eq!(s, parsed);
    }
}
