//! Folio — Asset Service (PM wrapper over G-10 `atlas_assets`)
//!
//! Property/unit creation with folio number, asset code, G-27 auto-provisioning.
//!
//! Entity field map:
//!   `serial_or_folio_number` — county folio number OR generated asset code
//!   `address_line_1` / `address_line_2` — address fields (underscored)
//!   `attributes` — JSONB for property_type, coordinates, etc.
//!   `status` — required; defaults to "active"

use anyhow::{Result, anyhow};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::pm::scorecard_provisioner::get_pm_template;
use crate::services::scorecard_service::ScorecardService;
use crate::types::pm::PropertyType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUnitInput {
    pub portfolio_id: Uuid,
    pub parent_asset_id: Option<Uuid>,
    pub name: String,
    pub address_line_1: String,
    pub address_line_2: Option<String>,
    pub city: String,
    pub state_province: String,
    pub postal_code: String,
    pub country_code: String,
    pub property_type: PropertyType,
    /// County appraiser folio number (e.g. "01-4141-008-0010"). If provided,
    /// stored as-is in `serial_or_folio_number`. If None, the generated asset
    /// code (e.g. "US-FL-001") is stored there instead.
    pub folio_number: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub struct AssetService;

impl AssetService {
    /// Create a property unit in `atlas_assets`.
    ///
    /// `serial_or_folio_number` receives the county folio number if provided,
    /// otherwise the auto-generated asset code (`US-FL-001`).
    ///
    /// Phase 2: auto-provisions the correct G-27 scorecard via
    /// `property_type.scorecard_entity_type()`.
    pub async fn create_unit(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: CreateUnitInput,
    ) -> Result<Uuid> {
        use chrono::Utc;
        use sea_orm::{ActiveModelTrait, Set};

        // Generate asset code for tracking even when a folio number is provided.
        let asset_code = crate::services::pm::portfolio::PortfolioService::next_asset_code(
            db,
            tenant_id,
            &input.country_code,
            &input.state_province,
        )
        .await?
        .display();

        // serial_or_folio_number: prefer the official county folio, fall back to asset code.
        let serial_or_folio = input
            .folio_number
            .clone()
            .unwrap_or_else(|| asset_code.clone());

        let scorecard_entity_type = input.property_type.scorecard_entity_type();

        let id = Uuid::new_v4();
        let now = Utc::now();

        let attributes = serde_json::json!({
            "property_type": input.property_type.to_string(),
            "scorecard_entity_type": scorecard_entity_type.to_string(),
            "asset_code": asset_code,
            "folio_number": input.folio_number,
            "coordinates": { "lat": input.latitude, "lng": input.longitude },
        });

        let model = crate::entities::atlas_asset::ActiveModel {
            id: Set(id),
            tenant_id: Set(tenant_id),
            portfolio_id: Set(Some(input.portfolio_id)),
            parent_asset_id: Set(input.parent_asset_id),
            owner_user_id: Set(None),
            asset_type: Set(if input.parent_asset_id.is_some() {
                // G-10 discriminator for nested rentable units. `property_type`
                // (single_family / multi_family / …) remains in `attributes`.
                "real_estate_unit".to_string()
            } else {
                "real_estate_property".to_string()
            }),
            name: Set(input.name),
            serial_or_folio_number: Set(Some(serial_or_folio)),
            status: Set("active".to_string()),
            address_line_1: Set(Some(input.address_line_1)),
            address_line_2: Set(input.address_line_2),
            city: Set(Some(input.city)),
            state_province: Set(Some(input.state_province)),
            postal_code: Set(Some(input.postal_code)),
            country_code: Set(Some(input.country_code)),
            geo_point: Set(None),
            attributes: Set(Some(attributes)),
            created_at: Set(now),
            // G-10 lifecycle fields: not applicable to property/unit records.
            // Appliances set these via ApplianceService::create.
            ..Default::default()
        };
        model.insert(db).await?;

        tracing::info!(
            asset_id = %id, %tenant_id,
            scorecard_entity = %scorecard_entity_type,
            "AssetService: created PM unit"
        );

        // Phase 2: auto-provision G-27 scorecard for this asset.
        //
        // Template name is derived from property_type:
        //   STR  → "STR Property Assessment"
        //   else → "Rental Unit Quality"
        //
        // If FolioApp::provision() has not been called yet (dev env, test tenant),
        // this is non-fatal — we log a warning and continue. The scorecard can be
        // provisioned retroactively once the tenant runs through onboarding.
        let template_name = match scorecard_entity_type {
            crate::types::pm::ScorecardEntityType::StrProperty => "STR Property Assessment",
            _ => "Rental Unit Quality",
        };

        match get_pm_template(db, tenant_id, template_name).await {
            Ok(template) => {
                match ScorecardService::get_or_create(db, tenant_id, template.id, "atlas_asset", id)
                    .await
                {
                    Ok(scorecard_id) => {
                        tracing::info!(
                            asset_id = %id, %tenant_id,
                            %scorecard_id,
                            template = template_name,
                            "AssetService: G-27 scorecard provisioned"
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            asset_id = %id, %tenant_id,
                            "AssetService: G-27 scorecard creation failed (non-fatal): {e:#}"
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    asset_id = %id, %tenant_id,
                    template = template_name,
                    "AssetService: PM template not found — was FolioApp::provision() called? {e:#}"
                );
            }
        }

        Ok(id)
    }

    pub async fn list_units(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        portfolio_id: Option<Uuid>,
    ) -> Result<Vec<crate::entities::atlas_asset::Model>> {
        use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

        let mut query = crate::entities::atlas_asset::Entity::find()
            .filter(crate::entities::atlas_asset::Column::TenantId.eq(tenant_id))
            .filter(crate::entities::atlas_asset::Column::Status.eq("active"));

        if let Some(pid) = portfolio_id {
            query = query.filter(crate::entities::atlas_asset::Column::PortfolioId.eq(pid));
        }

        Ok(query.all(db).await?)
    }

    /// Filter assets to hired-PM grant scope (`None` = unrestricted).
    pub fn filter_by_asset_grants(
        assets: Vec<crate::entities::atlas_asset::Model>,
        grants: Option<&[Uuid]>,
    ) -> Vec<crate::entities::atlas_asset::Model> {
        let Some(grants) = grants else {
            return assets;
        };
        use crate::services::pm::management_delegation::ManagementDelegationService;
        use std::collections::HashMap;
        let parent_by_id: HashMap<Uuid, Option<Uuid>> = assets
            .iter()
            .map(|a| (a.id, a.parent_asset_id))
            .collect();
        assets
            .into_iter()
            .filter(|a| {
                ManagementDelegationService::asset_in_grant_scope(
                    a.id,
                    a.parent_asset_id,
                    grants,
                    &parent_by_id,
                )
            })
            .collect()
    }

    pub async fn get_unit(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<crate::entities::atlas_asset::Model> {
        use sea_orm::EntityTrait;

        let asset = crate::entities::atlas_asset::Entity::find_by_id(asset_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow!("Asset {asset_id} not found"))?;

        if asset.tenant_id != tenant_id {
            return Err(anyhow!("Asset {asset_id} not found for tenant {tenant_id}"));
        }

        Ok(asset)
    }

    /// Like [`get_unit`], but 404 when outside hired-PM grant scope.
    pub async fn get_unit_scoped(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::entities::atlas_asset::Model> {
        use crate::services::pm::management_delegation::ManagementDelegationService;

        let asset = Self::get_unit(db, tenant_id, asset_id).await?;
        let grants = ManagementDelegationService::accessible_asset_ids(db, user_id)
            .await
            .map_err(|e| anyhow!("asset grants: {e}"))?;
        if let Some(ref ids) = grants {
            if ids.contains(&asset.id) {
                return Ok(asset);
            }
            let mut cursor = asset.parent_asset_id;
            let mut guard = 0usize;
            while let Some(pid) = cursor {
                if ids.contains(&pid) {
                    return Ok(asset);
                }
                cursor = Self::get_unit(db, tenant_id, pid)
                    .await
                    .ok()
                    .and_then(|p| p.parent_asset_id);
                guard += 1;
                if guard > 32 {
                    break;
                }
            }
            return Err(anyhow!("Asset {asset_id} not found"));
        }
        Ok(asset)
    }

    /// Merge `attributes.coordinates.{lat,lng}` on an existing asset.
    pub async fn set_coordinates(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        lat: f64,
        lng: f64,
    ) -> Result<()> {
        use sea_orm::{ActiveModelTrait, EntityTrait, Set};

        if !(lat.is_finite() && lng.is_finite()) || (lat == 0.0 && lng == 0.0) {
            anyhow::bail!("invalid coordinates");
        }
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lng) {
            anyhow::bail!("coordinates out of range");
        }

        let asset = Self::get_unit(db, tenant_id, asset_id).await?;
        let mut attrs = asset
            .attributes
            .clone()
            .unwrap_or_else(|| serde_json::json!({}));
        if let Some(obj) = attrs.as_object_mut() {
            obj.insert(
                "coordinates".into(),
                serde_json::json!({ "lat": lat, "lng": lng }),
            );
        }
        let mut am: crate::entities::atlas_asset::ActiveModel = asset.into();
        am.attributes = Set(Some(attrs));
        am.update(db).await?;
        Ok(())
    }

    /// Nominatim geocode from the asset's stored address; persists coordinates.
    pub async fn geocode_from_address(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
    ) -> Result<(f64, f64)> {
        let asset = Self::get_unit(db, tenant_id, asset_id).await?;
        let q = [
            asset.address_line_1.clone().unwrap_or_default(),
            asset.city.clone().unwrap_or_default(),
            asset.state_province.clone().unwrap_or_default(),
            asset.postal_code.clone().unwrap_or_default(),
            asset.country_code.clone().unwrap_or_default(),
        ]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        if q.trim().is_empty() {
            anyhow::bail!("address required for geocode");
        }

        let (lat, lng) = nominatim_search(&q).await?;
        Self::set_coordinates(db, tenant_id, asset_id, lat, lng).await?;
        Ok((lat, lng))
    }

    /// Best-effort geocode after create when lat/lng were omitted.
    pub async fn maybe_geocode_new_asset(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        asset_id: Uuid,
        latitude: Option<f64>,
        longitude: Option<f64>,
    ) {
        if latitude.is_some() && longitude.is_some() {
            return;
        }
        if let Err(e) = Self::geocode_from_address(db, tenant_id, asset_id).await {
            tracing::warn!(%asset_id, %tenant_id, "AssetService: create-time geocode skipped: {e:#}");
        }
    }
}

async fn nominatim_search(query: &str) -> Result<(f64, f64)> {
    let client = reqwest::Client::builder()
        .user_agent("AtlasFolio/1.0 (property-ops; contact=dev@atlas.local)")
        .build()?;
    let url = reqwest::Url::parse_with_params(
        "https://nominatim.openstreetmap.org/search",
        &[("q", query), ("format", "json"), ("limit", "1")],
    )?;
    // Nominatim usage policy: max 1 req/s — brief politeness delay.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    let resp = client.get(url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("geocoder HTTP {}", resp.status());
    }
    let rows: Vec<serde_json::Value> = resp.json().await?;
    let first = rows
        .first()
        .ok_or_else(|| anyhow!("no geocode result"))?;
    let lat: f64 = first
        .get("lat")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing lat"))?
        .parse()?;
    let lng: f64 = first
        .get("lon")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("missing lon"))?
        .parse()?;
    Ok((lat, lng))
}
