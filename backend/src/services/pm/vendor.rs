//! Folio — Vendor Service (PM wrapper over G-12 `atlas_service_providers`)
//!
//! Contractor onboarding, emergency availability, G-27 scorecard auto-provisioning.
//!
//! # Entity field map (`atlas_service_providers`)
//!   `scope`              → "tenant" (hired by this operator) or "platform"
//!   `business_name`      → contractor business name
//!   `service_categories` → JSONB array of trade categories
//!   `status`             → "active" | "inactive" | "suspended"
//!   `profile_metadata`   → JSONB for trade_type, license, emergency flag, rate
//!   `user_id`            → required — link to atlas user account
//!   `is_insured`/`is_bonded` → defaults false

use anyhow::{anyhow, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::types::pm::TradeType;
use crate::services::pm::scorecard_provisioner::get_pm_template;
use crate::services::scorecard_service::ScorecardService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVendorInput {
    pub user_id: Uuid,
    pub business_name: String,
    pub trade_type: TradeType,
    pub license_number: Option<String>,
    pub license_state: Option<String>,
    pub is_emergency_available: bool,
    pub hourly_rate_cents: Option<i64>,
    pub is_insured: bool,
    pub is_bonded: bool,
}

pub struct VendorService;

impl VendorService {
    /// Onboard a contractor/vendor in `atlas_service_providers`.
    ///
    /// PM-specific fields (trade_type, license, emergency flag) live in
    /// `profile_metadata` JSONB. `service_categories` holds the trade type
    /// as a JSON array for cross-platform querying.
    ///
    /// Phase 2: auto-provisions the "Contractor Performance" G-27 scorecard.
    pub async fn onboard(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateVendorInput,
    ) -> Result<Uuid> {
        use sea_orm::{Set, ActiveModelTrait};
        use chrono::Utc;

        let id = Uuid::new_v4();
        let now = Utc::now();

        let metadata = serde_json::json!({
            "trade_type": input.trade_type.to_string(),
            "is_emergency_available": input.is_emergency_available,
            "hourly_rate_cents": input.hourly_rate_cents,
            "license_number": input.license_number,
            "license_state": input.license_state,
        });

        let service_categories = serde_json::json!([input.trade_type.to_string()]);

        let model = crate::entities::atlas_service_provider::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            user_id: Set(input.user_id),
            scope: Set("tenant".to_string()),
            business_name: Set(Some(input.business_name)),
            service_categories: Set(service_categories),
            status: Set("active".to_string()),
            rating_avg: Set(None),
            rating_count: Set(0),
            is_insured: Set(input.is_insured),
            is_bonded: Set(input.is_bonded),
            profile_metadata: Set(Some(metadata)),
            created_at: Set(now),
            ..Default::default()
        };
        model.insert(db).await?;

        tracing::info!(
            vendor_id = %id, %tenant_id,
            trade = %input.trade_type,
            "VendorService: vendor onboarded"
        );

        // Phase 2: auto-provision the "Contractor Performance" G-27 scorecard.
        //
        // Every vendor gets a scorecard on creation so landlords can immediately
        // start rating work quality after the first job. Non-fatal on template miss.
        match get_pm_template(db, tenant_id, "Contractor Performance").await {
            Ok(template) => {
                match ScorecardService::get_or_create(
                    db,
                    tenant_id,
                    template.id,
                    "atlas_service_provider",
                    id,
                ).await {
                    Ok(scorecard_id) => {
                        tracing::info!(
                            vendor_id = %id, %tenant_id,
                            %scorecard_id,
                            "VendorService: Contractor Performance G-27 scorecard provisioned"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            vendor_id = %id, %tenant_id,
                            "VendorService: G-27 scorecard creation failed (non-fatal): {e:#}"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    vendor_id = %id, %tenant_id,
                    "VendorService: 'Contractor Performance' template not found — was FolioApp::provision() called? {e:#}"
                );
            }
        }

        Ok(id)
    }

    /// Toggle emergency availability on a vendor.
    ///
    /// Updates `profile_metadata.is_emergency_available` in JSONB.
    /// No updated_at column on this entity — omitted.
    pub async fn set_emergency_available(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        vendor_id: Uuid,
        available: bool,
    ) -> Result<()> {
        use sea_orm::{EntityTrait, IntoActiveModel, ActiveModelTrait, Set};

        let vendor = crate::entities::atlas_service_provider::Entity::find_by_id(vendor_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Vendor {vendor_id} not found"))?;

        if vendor.tenant_id != tenant_id {
            return Err(anyhow!("Vendor {vendor_id} not found for tenant {tenant_id}"));
        }

        let mut meta = vendor.profile_metadata.clone()
            .unwrap_or_else(|| serde_json::json!({}));
        meta["is_emergency_available"] = serde_json::json!(available);

        let mut active = vendor.into_active_model();
        active.profile_metadata = Set(Some(meta));
        active.update(db).await?;

        Ok(())
    }
}
