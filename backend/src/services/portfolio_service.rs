#![allow(unused_variables, dead_code)]
use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait, QuerySelect};
use uuid::Uuid;
use chrono::Utc;
use serde_json::Value;

use crate::entities::atlas_portfolio::{self, Entity as PortfolioEntity, ActiveModel as PortfolioActiveModel};

/// Service layer for GENERIC-09: AtlasPortfolio
/// Groups assets for reporting, billing, access control, etc.
pub struct PortfolioService;

impl PortfolioService {
    pub async fn create_portfolio(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        owner_user_id: Uuid,
        portfolio_type: &str,
        name: &str,
        description: Option<&str>,
        metadata: Option<Value>,
    ) -> Result<Uuid, String> {
        let portfolio = PortfolioActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            owner_user_id: Set(owner_user_id),
            portfolio_type: Set(portfolio_type.to_string()),
            name: Set(name.to_string()),
            description: Set(description.map(|s| s.to_string())),
            managed_account_id: sea_orm::ActiveValue::NotSet,
            metadata: Set(metadata),
            created_at: Set(Utc::now()),
        };

        let result = portfolio.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        portfolio_id: Uuid,
    ) -> Result<Option<atlas_portfolio::Model>, String> {
        PortfolioEntity::find()
            .filter(atlas_portfolio::Column::TenantId.eq(tenant_id))
            .filter(atlas_portfolio::Column::Id.eq(portfolio_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        portfolio_type: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_portfolio::Model>, String> {
        let mut q = PortfolioEntity::find()
            .filter(atlas_portfolio::Column::TenantId.eq(tenant_id));

        if let Some(pt) = portfolio_type {
            q = q.filter(atlas_portfolio::Column::PortfolioType.eq(pt.to_string()));
        }

        q.limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_metadata(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        portfolio_id: Uuid,
        metadata: Value,
    ) -> Result<(), String> {
        tracing::info!("Updating metadata for portfolio {}", portfolio_id);
        Ok(())
    }
}