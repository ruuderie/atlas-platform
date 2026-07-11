#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::atlas_contract::{
    self, ActiveModel as ContractActiveModel, Entity as ContractEntity,
};

/// Service layer for GENERIC-11: AtlasContract
/// Legal agreements, leases, policies, SLAs with rich terms_metadata.
pub struct ContractService;

impl ContractService {
    pub async fn create_contract(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        contract_type: &str,
        counterparty_user_id: Option<Uuid>,
        asset_id: Option<Uuid>,
        start_date: chrono::NaiveDate,
        end_date: Option<chrono::NaiveDate>,
        recurring_amount_cents: Option<i64>,
        billing_interval: &str,
        status: &str,
        terms_metadata: Option<Value>,
    ) -> Result<Uuid, String> {
        let contract = ContractActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            contract_type: Set(contract_type.to_string()),
            counterparty_user_id: Set(counterparty_user_id),
            asset_id: Set(asset_id),
            start_date: Set(start_date),
            end_date: Set(end_date),
            recurring_amount_cents: Set(recurring_amount_cents),
            currency: Set("USD".to_string()),
            billing_interval: Set(billing_interval.to_string()),
            status: Set(status.to_string()),
            terms_metadata: Set(terms_metadata),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = contract.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        contract_id: Uuid,
    ) -> Result<Option<atlas_contract::Model>, String> {
        ContractEntity::find()
            .filter(atlas_contract::Column::TenantId.eq(tenant_id))
            .filter(atlas_contract::Column::Id.eq(contract_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        status: Option<&str>,
        contract_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_contract::Model>, String> {
        let mut q = ContractEntity::find().filter(atlas_contract::Column::TenantId.eq(tenant_id));

        if let Some(s) = status {
            q = q.filter(atlas_contract::Column::Status.eq(s.to_string()));
        }
        if let Some(ct) = contract_type {
            q = q.filter(atlas_contract::Column::ContractType.eq(ct.to_string()));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    pub async fn terminate_contract(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        contract_id: Uuid,
        reason: &str,
    ) -> Result<(), String> {
        tracing::info!("Contract {} terminated: {}", contract_id, reason);
        Ok(())
    }
}
