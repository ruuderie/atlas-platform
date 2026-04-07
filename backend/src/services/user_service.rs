use crate::entities::user;
use crate::services::audit::AuditService;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter, Set};
use serde_json::json;
use crate::auth::hash_password;
use chrono::Utc;

pub struct UserService;

impl UserService {
    pub async fn update_email(
        db: &DatabaseConnection,
        current_user: user::Model,
        new_email: String,
    ) -> Result<user::Model, String> {
        // Check if new email already exists
        let existing_user = user::Entity::find()
            .filter(user::Column::Email.eq(&new_email))
            .one(db)
            .await
            .map_err(|_| "Database error checking email".to_string())?;

        if existing_user.is_some() {
            return Err("Email already in use".to_string());
        }

        let old_state = json!({"email": current_user.email});
        let new_state = json!({"email": new_email});

        let mut user_active: user::ActiveModel = current_user.clone().into();
        user_active.email = Set(new_email.clone());
        user_active.updated_at = Set(Utc::now());

        let updated_user = user_active.update(db).await.map_err(|_| {
            "Failed to update email".to_string()
        })?;

        // Dispatch background audit logging
        AuditService::log_action(
            db.clone(),
            None, // Might need to pass tenant_id
            Some(updated_user.id),
            "user.email.updated".to_string(),
            "User".to_string(),
            updated_user.id,
            Some(old_state),
            Some(new_state),
            None, // ip_address
        );

        Ok(updated_user)
    }

    pub async fn update_password(
        db: &DatabaseConnection,
        current_user: user::Model,
        new_password: String,
    ) -> Result<user::Model, String> {
        let hashed_password = hash_password(&new_password)
            .map_err(|_| "Error hashing new password".to_string())?;

        let old_state = json!({"password_hash_updated": false});
        let new_state = json!({"password_hash_updated": true});

        let mut user_active: user::ActiveModel = current_user.clone().into();
        user_active.password_hash = Set(hashed_password);
        user_active.updated_at = Set(Utc::now());

        let updated_user = user_active.update(db).await.map_err(|_| {
            "Failed to update password".to_string()
        })?;

        // Dispatch background audit logging
        AuditService::log_action(
            db.clone(),
            None,
            Some(updated_user.id),
            "user.password.updated".to_string(),
            "User".to_string(),
            updated_user.id,
            Some(old_state),
            Some(new_state),
            None,
        );

        Ok(updated_user)
    }
}
