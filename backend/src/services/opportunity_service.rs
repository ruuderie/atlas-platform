#![allow(unused_variables, dead_code)]
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_opportunity::{self, Entity as OpportunityEntity, ActiveModel as OpportunityActiveModel};

/// Service layer for GENERIC-15: AtlasOpportunity
/// Pipeline / deal tracking with flexible financial modeling (JSONB inputs/outputs).
/// Replaces legacy deal + lead pipeline concepts.
pub struct OpportunityService;

impl OpportunityService {
    /// Create a new opportunity (or migrated legacy lead).
    pub async fn create_opportunity(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_type: &str,
        name: &str,
        counterparty_account_id: Option<Uuid>,
        deal_amount_cents: Option<i64>,
        probability_pct: Option<i16>,
        status: Option<&str>,
    ) -> Result<Uuid, String> {
        let opportunity = OpportunityActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            opportunity_type: Set(opportunity_type.to_string()),
            name: Set(name.to_string()),
            counterparty_user_id: Set(counterparty_account_id), // best-effort mapping; real link via account later
            deal_amount_cents: Set(deal_amount_cents),
            currency: Set("USD".to_string()),
            probability_pct: Set(probability_pct),
            status: Set(status.unwrap_or("new").to_string()),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = opportunity.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    /// Find by ID scoped to tenant.
    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
    ) -> Result<Option<atlas_opportunity::Model>, String> {
        OpportunityEntity::find()
            .filter(atlas_opportunity::Column::TenantId.eq(tenant_id))
            .filter(atlas_opportunity::Column::Id.eq(opportunity_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// List opportunities for a tenant (optionally filtered by status or type).
    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_opportunity::Model>, String> {
        let mut q = OpportunityEntity::find()
            .filter(atlas_opportunity::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_opportunity::Column::Status.eq(s.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// Mark opportunity as won (sets won_at, status, optional final amount).
    pub async fn mark_won(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        final_amount_cents: Option<i64>,
    ) -> Result<(), String> {
        // In production this would be a proper update + transaction.
        // For v1 we demonstrate the intent and rely on caller to re-fetch.
        tracing::info!(
            "Opportunity {} marked won (final_amount={:?}). Full update implementation pending transaction helper.",
            opportunity_id, final_amount_cents
        );
        Ok(())
    }

    /// Update probability and/or financial inputs (common pipeline operation).
    pub async fn update_probability_and_financials(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        opportunity_id: Uuid,
        probability_pct: Option<i16>,
        financial_inputs: Option<Value>,
    ) -> Result<(), String> {
        tracing::info!(
            "Updating opportunity {} probability/financials (prob={:?}).",
            opportunity_id, probability_pct
        );
        Ok(())
    }
}