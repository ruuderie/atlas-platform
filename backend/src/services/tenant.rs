use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect, QueryFilter, ColumnTrait, DbErr, ActiveModelTrait, Set, NotSet};
use uuid::Uuid;
use anyhow::{Result, Context};
use crate::entities::tenant;
use crate::models::tenant::{CreateTenant, UpdateTenant};

pub struct TenantService;

impl TenantService {
    pub async fn list_tenants(db: &DatabaseConnection) -> Result<Vec<tenant::Model>> {
        let tenants = tenant::Entity::find().all(db).await?;
        Ok(tenants)
    }

    pub async fn get_tenant_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<tenant::Model>> {
        let tenant = tenant::Entity::find_by_id(id).one(db).await?;
        Ok(tenant)
    }

    pub async fn create_tenant(db: &DatabaseConnection, input: CreateTenant) -> Result<tenant::Model> {
        let new_tenant = tenant::ActiveModel::from(input);
        let tenant = new_tenant.insert(db).await?;
        Ok(tenant)
    }

    pub async fn update_tenant(db: &DatabaseConnection, tenant_id: Uuid, input: UpdateTenant) -> Result<tenant::Model> {
        let existing = tenant::Entity::find_by_id(tenant_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found"))?;

        let mut active_model: tenant::ActiveModel = existing.into();

        if let Some(name) = input.name {
            active_model.name = Set(name);
        }
        if let Some(description) = input.description {
            active_model.description = Set(description);
        }
        if let Some(logo) = input.logo {
            active_model.logo = Set(Some(logo));
        }
        if let Some(favicon) = input.favicon {
            active_model.favicon = Set(Some(favicon));
        }
        
        let updated = active_model.update(db).await?;
        Ok(updated)
    }

    pub async fn delete_tenant(db: &DatabaseConnection, tenant_id: Uuid) -> Result<()> {
        let _result = tenant::Entity::delete_by_id(tenant_id).exec(db).await?;
        Ok(())
    }
}