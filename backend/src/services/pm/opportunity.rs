//! # G15 OpportunityService — Sales Pipeline & Deal Management
//!
//! Wraps the existing `atlas_opportunities` entity (G15 migration).
//! Uses the entity's `status` field as the stage discriminator,
//! and `opportunity_type` for the deal classification.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

use crate::{
    entities::atlas_opportunity,
    types::pm::{OpportunityStage, OpportunityType},
};

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateOpportunityPayload {
    pub name: String,
    pub opportunity_type: OpportunityType,
    /// Subject asset (G10). Kept for backward compat with existing entity.
    pub asset_id: Option<Uuid>,
    pub owner_user_id: Option<Uuid>,
    pub counterparty_user_id: Option<Uuid>,
    pub description: Option<String>,
    pub amount_cents: Option<i64>,
    pub currency: Option<String>,
    pub probability_pct: Option<i32>,
    pub close_date: Option<chrono::NaiveDate>,
    /// Arbitrary financial inputs (rate, term, LTV, etc.).
    pub financial_inputs: Option<serde_json::Value>,
    pub created_by_user_id: Option<Uuid>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct OpportunityFilter {
    pub stage: Option<OpportunityStage>,
    pub opportunity_type: Option<OpportunityType>,
    pub owner_user_id: Option<Uuid>,
    pub asset_id: Option<Uuid>,
    pub open_only: Option<bool>,
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct OpportunityService;

impl OpportunityService {
    pub async fn create(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        payload: CreateOpportunityPayload,
    ) -> Result<atlas_opportunity::Model> {
        let now = Utc::now();
        atlas_opportunity::ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            opportunity_type: Set(payload.opportunity_type.to_string()),
            name: Set(payload.name),
            asset_id: Set(payload.asset_id),
            crm_lead_id: Set(None),
            owner_user_id: Set(payload.owner_user_id),
            counterparty_user_id: Set(payload.counterparty_user_id),
            status: Set(OpportunityStage::Prospecting.to_string()),
            deal_amount_cents: Set(payload.amount_cents),
            currency: Set(payload.currency.unwrap_or_else(|| "USD".into())),
            close_date: Set(payload.close_date),
            probability_pct: Set(payload.probability_pct.map(|p| p as i16)),
            financial_inputs: Set(payload.financial_inputs),
            computed_outputs: Set(None),
            notes: Set(payload.description),
            won_at: Set(None),
            lost_at: Set(None),
            lost_reason: Set(None),
            created_at: Set(now),
        }
        .insert(db)
        .await
        .map_err(|e| anyhow!("create opportunity: {e:#}"))
    }

    pub async fn get(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<atlas_opportunity::Model> {
        atlas_opportunity::Entity::find_by_id(id)
            .filter(atlas_opportunity::Column::TenantId.eq(tenant_id))
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Opportunity {id} not found"))
    }

    pub async fn list(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        filter: OpportunityFilter,
    ) -> Result<Vec<atlas_opportunity::Model>> {
        let mut q = atlas_opportunity::Entity::find()
            .filter(atlas_opportunity::Column::TenantId.eq(tenant_id));

        if let Some(stage) = filter.stage {
            q = q.filter(atlas_opportunity::Column::Status.eq(stage.to_string()));
        }
        if let Some(opp_type) = filter.opportunity_type {
            q = q.filter(atlas_opportunity::Column::OpportunityType.eq(opp_type.to_string()));
        }
        if let Some(owner) = filter.owner_user_id {
            q = q.filter(atlas_opportunity::Column::OwnerUserId.eq(owner));
        }
        if let Some(asset) = filter.asset_id {
            q = q.filter(atlas_opportunity::Column::AssetId.eq(asset));
        }
        if filter.open_only.unwrap_or(false) {
            q = q.filter(
                atlas_opportunity::Column::Status.ne(OpportunityStage::ClosedWon.to_string()),
            );
            q = q.filter(
                atlas_opportunity::Column::Status.ne(OpportunityStage::ClosedLost.to_string()),
            );
        }

        Ok(q.order_by_desc(atlas_opportunity::Column::CreatedAt)
            .all(db)
            .await?)
    }

    // ── Stage machine ─────────────────────────────────────────────────────────

    pub async fn advance_stage(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        new_stage: OpportunityStage,
        won_amount_cents: Option<i64>,
        lost_reason: Option<String>,
    ) -> Result<atlas_opportunity::Model> {
        let opp = Self::get(db, tenant_id, id).await?;
        let current = OpportunityStage::try_from(opp.status.as_str())
            .map_err(|e| anyhow!("corrupt stage: {e}"))?;

        if matches!(
            current,
            OpportunityStage::ClosedWon | OpportunityStage::ClosedLost
        ) {
            return Err(anyhow!(
                "Opportunity {id} is already closed ({current}) — cannot advance"
            ));
        }

        let now = Utc::now();
        let mut active: atlas_opportunity::ActiveModel = opp.into();
        active.status = Set(new_stage.to_string());

        match &new_stage {
            OpportunityStage::ClosedWon => {
                active.won_at = Set(Some(now));
                if let Some(amt) = won_amount_cents {
                    active.deal_amount_cents = Set(Some(amt));
                }
                active.probability_pct = Set(Some(100));
            }
            OpportunityStage::ClosedLost => {
                active.lost_at = Set(Some(now));
                active.lost_reason = Set(lost_reason);
                active.probability_pct = Set(Some(0));
            }
            _ => {}
        }

        Ok(active.update(db).await?)
    }

    pub async fn update_forecast(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        id: Uuid,
        probability_pct: Option<i32>,
        close_date: Option<chrono::NaiveDate>,
        amount_cents: Option<i64>,
    ) -> Result<atlas_opportunity::Model> {
        let opp = Self::get(db, tenant_id, id).await?;
        let mut active: atlas_opportunity::ActiveModel = opp.into();
        if let Some(p) = probability_pct {
            active.probability_pct = Set(Some(p.max(0).min(100) as i16));
        }
        if let Some(d) = close_date {
            active.close_date = Set(Some(d));
        }
        if let Some(a) = amount_cents {
            active.deal_amount_cents = Set(Some(a));
        }
        Ok(active.update(db).await?)
    }
}
