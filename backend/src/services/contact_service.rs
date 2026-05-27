use sea_orm::{DatabaseConnection, EntityTrait, ActiveModelTrait, Set, QueryFilter, ColumnTrait};
use uuid::Uuid;
use chrono::Utc;
use crate::entities::atlas_contact::{self, Entity as ContactEntity, ActiveModel as ContactActiveModel};

/// Service layer for the unified Contact concept.
/// Every contact belongs to an Account.
pub struct ContactService;

impl ContactService {
    /// Create a new contact under an account.
    pub async fn create_contact(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        account_id: Uuid,
        first_name: Option<&str>,
        last_name: Option<&str>,
        email: Option<&str>,
        is_primary: bool,
    ) -> Result<Uuid, String> {
        let full_name = match (&first_name, &last_name) {
            (Some(f), Some(l)) => Some(format!("{} {}", f, l)),
            (Some(f), None) => Some(f.to_string()),
            (None, Some(l)) => Some(l.to_string()),
            _ => None,
        };

        let contact = ContactActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            account_id: Set(account_id),
            first_name: Set(first_name.map(|s| s.to_string())),
            last_name: Set(last_name.map(|s| s.to_string())),
            full_name: Set(full_name),
            email: Set(email.map(|s| s.to_string())),
            is_primary: Set(is_primary),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let result = contact.insert(db).await.map_err(|e| e.to_string())?;
        Ok(result.id)
    }

    /// List contacts for an account.
    pub async fn list_for_account(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        account_id: Uuid,
    ) -> Result<Vec<atlas_contact::Model>, String> {
        ContactEntity::find()
            .filter(atlas_contact::Column::TenantId.eq(tenant_id))
            .filter(atlas_contact::Column::AccountId.eq(account_id))
            .all(db)
            .await
            .map_err(|e| e.to_string())
    }

    /// Set a contact as the primary contact for its account (and unset others).
    pub async fn set_as_primary(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        contact_id: Uuid,
    ) -> Result<(), String> {
        // In a real implementation this would be a transaction + update many.
        // For now this is a stub showing intent.
        tracing::info!("TODO: Implement set_as_primary for contact {}", contact_id);
        Ok(())
    }
}
