//! # G-19 Direct Mail Campaign Helpers
//!
//! Service functions for managing mail drops and offer codes within campaigns.
//! Companion to `campaign.rs` for direct mail specific operations.

use anyhow::{Result, anyhow};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use uuid::Uuid;

use crate::{
    entities::{atlas_campaign_mail_drop, atlas_campaign_offer_code},
    services::pm::campaign::CampaignService,
};

// ── Mail Drop Management ─────────────────────────────────────────────────────

/// Payload for creating a mail drop.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateMailDropPayload {
    pub drop_name: String,
    pub creative_variant: Option<String>,
    pub utm_content: Option<String>,
    pub piece_count: i32,
    pub unit_cost_cents: Option<i64>,
    pub provider_job_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Create a new mail drop under a campaign.
pub async fn create_mail_drop(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    campaign_id: Uuid,
    payload: CreateMailDropPayload,
) -> Result<atlas_campaign_mail_drop::Model> {
    // Verify campaign ownership
    CampaignService::get(db, tenant_id, campaign_id).await?;

    let active = atlas_campaign_mail_drop::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        campaign_id: Set(campaign_id),
        drop_name: Set(payload.drop_name),
        creative_variant: Set(payload.creative_variant),
        utm_content: Set(payload.utm_content),
        piece_count: Set(payload.piece_count),
        unit_cost_cents: Set(payload.unit_cost_cents),
        provider_job_id: Set(payload.provider_job_id),
        status: Set("draft".to_string()),
        mailed_at: Set(None),
        metadata: Set(payload.metadata),
        created_at: Set(Utc::now()),
        updated_at: Set(Utc::now()),
    };

    let drop = active.insert(db).await?;

    tracing::info!(
        %tenant_id, %campaign_id, drop_id = %drop.id,
        "create_mail_drop: created '{}'", drop.drop_name
    );

    Ok(drop)
}

/// List all mail drops for a campaign.
pub async fn list_mail_drops(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    campaign_id: Uuid,
) -> Result<Vec<atlas_campaign_mail_drop::Model>> {
    // Verify campaign ownership
    CampaignService::get(db, tenant_id, campaign_id).await?;

    Ok(atlas_campaign_mail_drop::Entity::find()
        .filter(atlas_campaign_mail_drop::Column::CampaignId.eq(campaign_id))
        .all(db)
        .await?)
}

// ── Offer Code Management ────────────────────────────────────────────────────

/// Payload for creating an offer code.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateOfferCodePayload {
    pub code: String,
    pub mail_drop_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

/// Create a new offer code under a campaign.
pub async fn create_offer_code(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    campaign_id: Uuid,
    payload: CreateOfferCodePayload,
) -> Result<atlas_campaign_offer_code::Model> {
    // Verify campaign ownership
    CampaignService::get(db, tenant_id, campaign_id).await?;

    // If mail_drop_id is provided, verify it belongs to this campaign
    if let Some(drop_id) = payload.mail_drop_id {
        let drop = atlas_campaign_mail_drop::Entity::find_by_id(drop_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Mail drop {} not found", drop_id))?;
        
        if drop.campaign_id != campaign_id {
            return Err(anyhow!("Mail drop {} does not belong to campaign {}", drop_id, campaign_id));
        }
    }

    let active = atlas_campaign_offer_code::ActiveModel {
        id: Set(Uuid::new_v4()),
        tenant_id: Set(tenant_id),
        campaign_id: Set(campaign_id),
        mail_drop_id: Set(payload.mail_drop_id),
        code: Set(payload.code.clone()),
        is_active: Set(payload.is_active.unwrap_or(true)),
        redemption_count: Set(0),
        created_at: Set(Utc::now()),
    };

    let offer_code = active.insert(db).await?;

    tracing::info!(
        %tenant_id, %campaign_id, offer_code_id = %offer_code.id,
        "create_offer_code: created '{}'", offer_code.code
    );

    Ok(offer_code)
}

/// Find an offer code by its code string (case-insensitive).
pub async fn find_offer_code_by_code(
    db: &DatabaseConnection,
    code: &str,
) -> Result<Option<atlas_campaign_offer_code::Model>> {
    use sea_orm::{FromQueryResult, Statement};
    
    // Case-insensitive exact match using direct SQL
    let models = atlas_campaign_offer_code::Model::find_by_statement(
        Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            "SELECT * FROM atlas_campaign_offer_codes WHERE LOWER(code) = LOWER($1) LIMIT 1",
            [code.into()]
        )
    )
    .all(db)
    .await?;
    
    Ok(models.into_iter().next())
}

/// Increment the redemption count for an offer code.
pub async fn increment_redemption_count(
    db: &DatabaseConnection,
    offer_code_id: Uuid,
) -> Result<atlas_campaign_offer_code::Model> {
    let offer_code = atlas_campaign_offer_code::Entity::find_by_id(offer_code_id)
        .one(db)
        .await?
        .ok_or_else(|| anyhow!("Offer code {} not found", offer_code_id))?;

    let mut active: atlas_campaign_offer_code::ActiveModel = offer_code.into();
    active.redemption_count = Set(active.redemption_count.unwrap() + 1);
    
    let updated = active.update(db).await?;

    tracing::info!(
        offer_code_id = %offer_code_id,
        new_count = updated.redemption_count,
        "increment_redemption_count: updated"
    );

    Ok(updated)
}

/// List all offer codes for a campaign.
pub async fn list_offer_codes(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    campaign_id: Uuid,
) -> Result<Vec<atlas_campaign_offer_code::Model>> {
    // Verify campaign ownership
    CampaignService::get(db, tenant_id, campaign_id).await?;

    Ok(atlas_campaign_offer_code::Entity::find()
        .filter(atlas_campaign_offer_code::Column::CampaignId.eq(campaign_id))
        .all(db)
        .await?)
}