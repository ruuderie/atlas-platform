use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::atlas_account::{self, Entity as AccountEntity, ActiveModel as AccountActiveModel};

/// Service layer for the unified Account concept (replaces legacy customer).
/// 
/// This is the foundation for B2B and B2C party management.
pub struct AccountService;

impl AccountService {
    /// Create a new account (organization or individual).
    pub async fn create_account(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        account_type: &str,
        name: &str,
        first_name: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<Uuid, String> {
        let account = AccountActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_type: Set(account_type.to_string()),
            name: Set(name.to_string()),
            first_name: Set(first_name.map(|s| s.to_string())),
            last_name: Set(last_name.map(|s| s.to_string())),
            status: Set("active".to_string()),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = account.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    /// Find an account by ID within a tenant.
    pub async fn find_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<atlas_account::Model>, String> {
        AccountEntity::find()
            .filter(atlas_account::Column::TenantId.eq(tenant_id))
            .filter(atlas_account::Column::Id.eq(account_id))
            .one(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// List accounts for a tenant.
    pub async fn list_for_tenant(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        limit: u64,
    ) -> Result<Vec<atlas_account::Model>, String> {
        AccountEntity::find()
            .filter(atlas_account::Column::TenantId.eq(tenant_id))
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// Find or create a default organization account for a tenant (useful during migration).
    pub async fn find_or_create_tenant_account(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        tenant_name: &str,
    ) -> Result<Uuid, String> {
        // In a real implementation we would have a better lookup (e.g. by name + type)
        let existing = Self::list_for_tenant(db, tenant_id, 5).await?;
        if let Some(acc) = existing.into_iter().find(|a| a.account_type == "organization") {
            return Ok(acc.id);
        }

        Self::create_account(db, tenant_id, "organization", tenant_name, None, None).await
    }
}
