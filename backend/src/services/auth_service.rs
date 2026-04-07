use crate::entities::{user, magic_link_token};
use crate::services::audit::AuditService;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set};
use serde_json::json;
use uuid::Uuid;
use chrono::{Duration, Utc};
use reqwest::StatusCode;

pub struct AuthService;

impl AuthService {
    pub async fn create_magic_link(
        db: &DatabaseConnection,
        req_email: &str,
    ) -> Result<magic_link_token::Model, (StatusCode, String)> {
        let user = user::Entity::find()
            .filter(user::Column::Email.eq(req_email))
            .one(db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            })?;

        let user = match user {
            Some(u) => u,
            None => {
                // Silently return an error structure that the handler interprets as 'ok' to prevent enum
                return Err((StatusCode::NOT_FOUND, "User not found".to_string()));
            }
        };

        // Generate token
        let token_string = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::minutes(15);

        let new_token = magic_link_token::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(user.id),
            token: Set(token_string.clone()),
            expires_at: Set(expires_at),
            is_used: Set(false),
            created_at: Set(Utc::now()),
        };

        let inserted_token = new_token.insert(db).await.map_err(|e| {
            tracing::error!("Failed to create token: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate magic link".to_string())
        })?;

        let new_state = json!({
            "token_id": inserted_token.id,
            "user_id": user.id,
            "expires_at": expires_at
        });

        // Audit the magic link generation
        AuditService::log_action(
            db.clone(),
            None, // Tenant ID unknown at this level without joining user, could expand if needed
            Some(user.id),
            "auth.magic_link.created".to_string(),
            "MagicLinkToken".to_string(),
            inserted_token.id,
            None,
            Some(new_state),
            None,
        );

        Ok(inserted_token)
    }

    pub async fn verify_magic_link(
        db: &DatabaseConnection,
        token_string: &str,
    ) -> Result<user::Model, (StatusCode, String)> {
        let token_record = magic_link_token::Entity::find()
            .filter(magic_link_token::Column::Token.eq(token_string))
            .filter(magic_link_token::Column::IsUsed.eq(false))
            .one(db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database query error".to_string())
            })?;

        let token_record = match token_record {
            Some(t) => t,
            None => return Err((StatusCode::BAD_REQUEST, "Invalid or expired magic link".to_string())),
        };

        if token_record.expires_at < Utc::now() {
            return Err((StatusCode::BAD_REQUEST, "Magic link has expired".to_string()));
        }

        let user_record = user::Entity::find_by_id(token_record.user_id)
            .one(db)
            .await
            .map_err(|e| {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "User query error".to_string())
            })?
            .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

        // Mark token as used
        let mut updated_token: magic_link_token::ActiveModel = token_record.clone().into();
        updated_token.is_used = Set(true);
        let consumed_token = updated_token.update(db).await.map_err(|e| {
            tracing::error!("Failed to update token: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to consume token".to_string())
        })?;

        let old_state = json!({"is_used": false});
        let new_state = json!({"is_used": true});

        AuditService::log_action(
            db.clone(),
            None, // Unknown at this domain level strictly
            Some(user_record.id),
            "auth.magic_link.consumed".to_string(),
            "MagicLinkToken".to_string(),
            consumed_token.id,
            Some(old_state),
            Some(new_state),
            None,
        );

        Ok(user_record)
    }
}
