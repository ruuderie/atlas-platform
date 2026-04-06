use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect, QueryFilter, ColumnTrait, DbErr, ActiveModelTrait};
use crate::config::site_config::{SiteConfig, ModuleFlags};
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;
use anyhow::{Result, Context};
use crate::entities::{self, network};

pub struct NetworkService;

impl NetworkService {
    pub async fn get_network_config(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<SiteConfig> {
        // Find the network first to ensure it exists
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        // Now fetch the site configuration
        // This is a simplified example - adjust based on your actual data model
        let config = SiteConfig {
            tenant_id,
            name: network.name,
            domain: network.domain,
            subdomain: network.subdomain,
            custom_domain: network.custom_domain,
            enabled_modules: ModuleFlags::from_bits_truncate(network.enabled_modules as u32),
            theme: network.theme,
            custom_settings: network.custom_settings
                .map(|v| serde_json::from_value(v).unwrap_or_default())
                .unwrap_or_default(),
            site_status: Some(network.site_status),
        };
        
        Ok(config)
    }
    
    pub async fn update_network_config(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        config: SiteConfig
    ) -> Result<SiteConfig> {
        // Find the network to update
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        // Convert to active model for updating
        let mut network: network::ActiveModel = network.into();
        
        // Update fields from config
        network.name = sea_orm::Set(config.name);
        network.domain = sea_orm::Set(config.domain);
        network.subdomain = sea_orm::Set(config.subdomain);
        network.custom_domain = sea_orm::Set(config.custom_domain);
        network.enabled_modules = sea_orm::Set(config.enabled_modules.bits() as i32);
        network.theme = sea_orm::Set(config.theme);
        network.custom_settings = sea_orm::Set(
            Some(serde_json::to_value(&config.custom_settings).unwrap_or_default())
        );
        network.site_status = sea_orm::Set(config.site_status.unwrap_or_default());
        
        // Save the updated network
        let updated_network = network.update(db)
            .await
            .context("Failed to update network")?;
        
        // Return the updated config
        Self::get_network_config(db, tenant_id).await
    }
    
    pub async fn list_networks(
        db: &DatabaseConnection,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> Result<Vec<network::Model>> {
        let mut query = network::Entity::find();
        
        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }
        
        if let Some(offset_val) = offset {
            query = query.offset(offset_val);
        }
        
        let networks = query
            .all(db)
            .await
            .context("Failed to list networks")?;
            
        Ok(networks)
    }
    
    pub async fn get_site_config(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<SiteConfig> {
        Self::get_network_config(db, tenant_id).await
    }
    
    pub async fn get_enabled_modules(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<(u32, Vec<String>)> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        let modules = ModuleFlags::from_bits_truncate(network.enabled_modules as u32);
        let module_names = Self::get_module_names(modules);
        
        Ok((network.enabled_modules as u32, module_names))
    }
    
    pub async fn update_enabled_modules(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        enabled_modules: u32
    ) -> Result<(u32, Vec<String>)> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        let mut active_model: network::ActiveModel = network.clone().into();
        active_model.enabled_modules = sea_orm::Set(enabled_modules as i32);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        
        let updated_network = active_model
            .update(db)
            .await
            .context("Failed to update network modules")?;
        
        let modules = ModuleFlags::from_bits_truncate(updated_network.enabled_modules as u32);
        let module_names = Self::get_module_names(modules);
        
        Ok((updated_network.enabled_modules as u32, module_names))
    }
    
    pub async fn get_site_theme(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<Option<String>> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        Ok(network.theme)
    }
    
    pub async fn update_site_theme(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        theme: Option<String>
    ) -> Result<Option<String>> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        let mut active_model: network::ActiveModel = network.clone().into();
        active_model.theme = sea_orm::Set(theme);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        
        let updated_network = active_model
            .update(db)
            .await
            .context("Failed to update network theme")?;
        
        Ok(updated_network.theme)
    }
    
    pub async fn get_custom_settings(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<Value> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        Ok(network.custom_settings.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())))
    }
    
    pub async fn update_custom_settings(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        settings: Value
    ) -> Result<Value> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
        
        let mut active_model: network::ActiveModel = network.clone().into();
        active_model.custom_settings = sea_orm::Set(Some(settings));
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        
        let updated_network = active_model
            .update(db)
            .await
            .context("Failed to update network custom settings")?;
        
        Ok(updated_network.custom_settings.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())))
    }
    
    // Helper function to get module names from flags
    fn get_module_names(flags: ModuleFlags) -> Vec<String> {
        let mut modules = Vec::new();
        
        if flags.contains(ModuleFlags::LISTINGS) { modules.push("listings".to_string()); }
        if flags.contains(ModuleFlags::PROFILES) { modules.push("profiles".to_string()); }
        if flags.contains(ModuleFlags::MESSAGING) { modules.push("messaging".to_string()); }
        if flags.contains(ModuleFlags::PAYMENTS) { modules.push("payments".to_string()); }
        if flags.contains(ModuleFlags::ANALYTICS) { modules.push("analytics".to_string()); }
        if flags.contains(ModuleFlags::REVIEWS) { modules.push("reviews".to_string()); }
        if flags.contains(ModuleFlags::EVENTS) { modules.push("events".to_string()); }
        if flags.contains(ModuleFlags::CUSTOM_FIELDS) { modules.push("custom_fields".to_string()); }
        
        modules
    }
    
    pub async fn create_network(
        db: &DatabaseConnection,
        input: crate::models::network::CreateNetwork
    ) -> Result<network::Model> {
        let mut custom_settings = serde_json::Map::new();
        if let Some(strategy) = &input.deployment_strategy {
            custom_settings.insert("deployment_strategy".to_string(), serde_json::Value::String(strategy.clone()));
            if strategy == "dedicated" {
                tracing::info!("🚀 [ORCHESTRATION HOOK] Provisioning dedicated container for domain: {}", input.domain);
            } else {
                tracing::info!("♻️ [ORCHESTRATION HOOK] Using shared multi-tenant infrastructure for domain: {}", input.domain);
            }
        }

        let new_network = network::ActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            name: sea_orm::Set(input.name),
            description: sea_orm::Set(input.description),
            tenant_id: sea_orm::Set(input.tenant_id),
            domain: sea_orm::Set(input.domain),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
            custom_domain: sea_orm::NotSet,
            custom_settings: sea_orm::Set(Some(serde_json::Value::Object(custom_settings))),
            enabled_modules: sea_orm::Set(0),
            theme: sea_orm::NotSet,
            site_status: sea_orm::Set("Active".to_string()),
            subdomain: sea_orm::NotSet,
            logo: sea_orm::NotSet,
            favicon: sea_orm::NotSet,
            header_scripts: sea_orm::NotSet,
            footer_scripts: sea_orm::NotSet,
            google_analytics_id: sea_orm::NotSet,
            google_site_verification: sea_orm::NotSet,
            meta_description: sea_orm::NotSet,
            meta_keywords: sea_orm::NotSet,
            meta_title: sea_orm::NotSet,
            page_title: sea_orm::NotSet,
            page_description: sea_orm::NotSet,
            page_keywords: sea_orm::NotSet,
            canonical_url: sea_orm::NotSet,
        };

        let network = new_network
            .insert(db)
            .await
            .context("Failed to create network")?;
            
        Ok(network)
    }
    
    pub async fn update_network(
        db: &DatabaseConnection,
        tenant_id: Uuid,
        input: crate::models::network::UpdateNetwork
    ) -> Result<network::Model> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;

        let mut active_network: network::ActiveModel = network.clone().into();

        if let Some(name) = input.name {
            active_network.name = sea_orm::Set(name);
        }
        if let Some(tenant_id) = input.tenant_id {
            active_network.tenant_id = sea_orm::Set(tenant_id);
        }
        if let Some(domain) = input.domain {
            active_network.domain = sea_orm::Set(domain);
        }
        if let Some(description) = input.description {
            active_network.description = sea_orm::Set(description);
        }
        active_network.updated_at = sea_orm::Set(chrono::Utc::now());

        let updated_network = active_network
            .update(db)
            .await
            .context("Failed to update network")?;

        Ok(updated_network)
    }
    
    pub async fn delete_network(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<()> {
        network::Entity::delete_by_id(tenant_id)
            .exec(db)
            .await
            .context("Failed to delete network")?;

        Ok(())
    }
    
    pub async fn get_network_by_id(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<network::Model> {
        let network = network::Entity::find_by_id(tenant_id)
            .one(db)
            .await
            .context("Failed to query network")?
            .ok_or_else(|| anyhow::anyhow!("Network not found"))?;
            
        Ok(network)
    }
    
    pub async fn get_network_by_domain(
        db: &DatabaseConnection,
        domain: &str
    ) -> Result<network::Model> {
        let network = network::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(network::Column::Domain.eq(domain))
                    .add(network::Column::CustomDomain.eq(domain))
                    .add(network::Column::Subdomain.eq(domain))
            )
            .one(db)
            .await
            .context("Failed to query network by domain")?
            .ok_or_else(|| anyhow::anyhow!("Network not found for domain"))?;
            
        Ok(network)
    }
    
    pub async fn get_networks_by_type(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<Vec<network::Model>> {
        let networks = network::Entity::find()
            .filter(network::Column::NetworkTypeId.eq(tenant_id))
            .all(db)
            .await
            .context("Failed to query networks by type")?;
            
        Ok(networks)
    }
    
    pub async fn get_network_listings(
        db: &DatabaseConnection,
        tenant_id: Uuid
    ) -> Result<Vec<crate::entities::listing::Model>> {
        // First check if network exists
        let _network = Self::get_network_by_id(db, tenant_id).await?;
        
        let listings = crate::entities::listing::Entity::find()
            .filter(crate::entities::listing::Column::TenantId.eq(tenant_id))
            .all(db)
            .await
            .context("Failed to fetch listings")?;
            
        Ok(listings)
    }
} 