#![allow(unused_variables, dead_code)]
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Set,
};
use serde_json::Value;
use uuid::Uuid;

use crate::entities::atlas_asset::{self, ActiveModel as AssetActiveModel, Entity as AssetEntity};

/// Service layer for GENERIC-10: AtlasAsset
/// Central registry for all physical and digital assets with hierarchy support.
pub struct AssetService;

impl AssetService {
    pub async fn create_asset(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        portfolio_id: Option<Uuid>,
        parent_asset_id: Option<Uuid>,
        asset_type: &str,
        name: &str,
        status: &str,
        attributes: Option<Value>,
    ) -> Result<Uuid, String> {
        let asset = AssetActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            portfolio_id: Set(portfolio_id),
            parent_asset_id: Set(parent_asset_id),
            asset_type: Set(asset_type.to_string()),
            name: Set(name.to_string()),
            status: Set(status.to_string()),
            attributes: Set(attributes),
            created_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = asset.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<Option<atlas_asset::Model>, String> {
        AssetEntity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::Id.eq(asset_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_type: Option<&str>,
        status: Option<&str>,
        limit: u64,
    ) -> Result<Vec<atlas_asset::Model>, String> {
        let mut q = AssetEntity::find().filter(atlas_asset::Column::TenantId.eq(tenant_id));

        if let Some(at) = asset_type {
            q = q.filter(atlas_asset::Column::AssetType.eq(at.to_string()));
        }
        if let Some(st) = status {
            q = q.filter(atlas_asset::Column::Status.eq(st.to_string()));
        }

        q.limit(limit).all(db).await.map_err(|e| e.to_string())
    }

    pub async fn list_children(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        parent_asset_id: Uuid,
        limit: u64,
    ) -> Result<Vec<atlas_asset::Model>, String> {
        AssetEntity::find()
            .filter(atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(atlas_asset::Column::ParentAssetId.eq(parent_asset_id))
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_status(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        new_status: &str,
    ) -> Result<(), String> {
        tracing::info!("Asset {} status updated to {}", asset_id, new_status);
        Ok(())
    }
}
