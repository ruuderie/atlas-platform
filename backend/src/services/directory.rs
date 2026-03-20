use sea_orm::{DatabaseConnection, EntityTrait, QuerySelect, QueryFilter, ColumnTrait, DbErr, ActiveModelTrait};
use crate::config::site_config::{SiteConfig, ModuleFlags};
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;
use anyhow::{Result, Context};
use crate::entities::{self, directory};

pub struct DirectoryService;

impl DirectoryService {
    pub async fn get_directory_config(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<SiteConfig> {
        // Find the directory first to ensure it exists
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        // Now fetch the site configuration
        // This is a simplified example - adjust based on your actual data model
        let config = SiteConfig {
            directory_id,
            name: directory.name,
            domain: directory.domain,
            subdomain: directory.subdomain,
            custom_domain: directory.custom_domain,
            enabled_modules: ModuleFlags::from_bits_truncate(directory.enabled_modules as u32),
            theme: directory.theme,
            custom_settings: directory.custom_settings
                .map(|v| serde_json::from_value(v).unwrap_or_default())
                .unwrap_or_default(),
            site_status: Some(directory.site_status),
        };
        
        Ok(config)
    }
    
    pub async fn update_directory_config(
        db: &DatabaseConnection,
        directory_id: Uuid,
        config: SiteConfig
    ) -> Result<SiteConfig> {
        // Find the directory to update
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        // Convert to active model for updating
        let mut directory: directory::ActiveModel = directory.into();
        
        // Update fields from config
        directory.name = sea_orm::Set(config.name);
        directory.domain = sea_orm::Set(config.domain);
        directory.subdomain = sea_orm::Set(config.subdomain);
        directory.custom_domain = sea_orm::Set(config.custom_domain);
        directory.enabled_modules = sea_orm::Set(config.enabled_modules.bits() as i32);
        directory.theme = sea_orm::Set(config.theme);
        directory.custom_settings = sea_orm::Set(
            Some(serde_json::to_value(&config.custom_settings).unwrap_or_default())
        );
        directory.site_status = sea_orm::Set(config.site_status.unwrap_or_default());
        
        // Save the updated directory
        let updated_directory = directory.update(db)
            .await
            .context("Failed to update directory")?;
        
        // Return the updated config
        Self::get_directory_config(db, directory_id).await
    }
    
    pub async fn list_directories(
        db: &DatabaseConnection,
        limit: Option<u64>,
        offset: Option<u64>
    ) -> Result<Vec<directory::Model>> {
        let mut query = directory::Entity::find();
        
        if let Some(limit_val) = limit {
            query = query.limit(limit_val);
        }
        
        if let Some(offset_val) = offset {
            query = query.offset(offset_val);
        }
        
        let directories = query
            .all(db)
            .await
            .context("Failed to list directories")?;
            
        Ok(directories)
    }
    
    pub async fn get_site_config(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<SiteConfig> {
        Self::get_directory_config(db, directory_id).await
    }
    
    pub async fn get_enabled_modules(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<(u32, Vec<String>)> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        let modules = ModuleFlags::from_bits_truncate(directory.enabled_modules as u32);
        let module_names = Self::get_module_names(modules);
        
        Ok((directory.enabled_modules as u32, module_names))
    }
    
    pub async fn update_enabled_modules(
        db: &DatabaseConnection,
        directory_id: Uuid,
        enabled_modules: u32
    ) -> Result<(u32, Vec<String>)> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        let mut active_model: directory::ActiveModel = directory.clone().into();
        active_model.enabled_modules = sea_orm::Set(enabled_modules as i32);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        
        let updated_directory = active_model
            .update(db)
            .await
            .context("Failed to update directory modules")?;
        
        let modules = ModuleFlags::from_bits_truncate(updated_directory.enabled_modules as u32);
        let module_names = Self::get_module_names(modules);
        
        Ok((updated_directory.enabled_modules as u32, module_names))
    }
    
    pub async fn get_site_theme(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<Option<String>> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        Ok(directory.theme)
    }
    
    pub async fn update_site_theme(
        db: &DatabaseConnection,
        directory_id: Uuid,
        theme: Option<String>
    ) -> Result<Option<String>> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        let mut active_model: directory::ActiveModel = directory.clone().into();
        active_model.theme = sea_orm::Set(theme);
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        
        let updated_directory = active_model
            .update(db)
            .await
            .context("Failed to update directory theme")?;
        
        Ok(updated_directory.theme)
    }
    
    pub async fn get_custom_settings(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<Value> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        Ok(directory.custom_settings.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())))
    }
    
    pub async fn update_custom_settings(
        db: &DatabaseConnection,
        directory_id: Uuid,
        settings: Value
    ) -> Result<Value> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
        
        let mut active_model: directory::ActiveModel = directory.clone().into();
        active_model.custom_settings = sea_orm::Set(Some(settings));
        active_model.updated_at = sea_orm::Set(chrono::Utc::now());
        
        let updated_directory = active_model
            .update(db)
            .await
            .context("Failed to update directory custom settings")?;
        
        Ok(updated_directory.custom_settings.unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new())))
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
    
    pub async fn create_directory(
        db: &DatabaseConnection,
        input: crate::models::directory::CreateDirectory
    ) -> Result<directory::Model> {
        let new_directory = directory::ActiveModel {
            id: sea_orm::Set(Uuid::new_v4()),
            name: sea_orm::Set(input.name),
            description: sea_orm::Set(input.description),
            directory_type_id: sea_orm::Set(input.directory_type_id),
            domain: sea_orm::Set(input.domain),
            created_at: sea_orm::Set(chrono::Utc::now()),
            updated_at: sea_orm::Set(chrono::Utc::now()),
            custom_domain: sea_orm::NotSet,
            custom_settings: sea_orm::NotSet,
            enabled_modules: sea_orm::NotSet,
            theme: sea_orm::NotSet,
            site_status: sea_orm::NotSet,
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

        let directory = new_directory
            .insert(db)
            .await
            .context("Failed to create directory")?;
            
        Ok(directory)
    }
    
    pub async fn update_directory(
        db: &DatabaseConnection,
        directory_id: Uuid,
        input: crate::models::directory::UpdateDirectory
    ) -> Result<directory::Model> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;

        let mut active_directory: directory::ActiveModel = directory.clone().into();

        if let Some(name) = input.name {
            active_directory.name = sea_orm::Set(name);
        }
        if let Some(directory_type_id) = input.directory_type_id {
            active_directory.directory_type_id = sea_orm::Set(directory_type_id);
        }
        if let Some(domain) = input.domain {
            active_directory.domain = sea_orm::Set(domain);
        }
        if let Some(description) = input.description {
            active_directory.description = sea_orm::Set(description);
        }
        active_directory.updated_at = sea_orm::Set(chrono::Utc::now());

        let updated_directory = active_directory
            .update(db)
            .await
            .context("Failed to update directory")?;

        Ok(updated_directory)
    }
    
    pub async fn delete_directory(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<()> {
        directory::Entity::delete_by_id(directory_id)
            .exec(db)
            .await
            .context("Failed to delete directory")?;

        Ok(())
    }
    
    pub async fn get_directory_by_id(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<directory::Model> {
        let directory = directory::Entity::find_by_id(directory_id)
            .one(db)
            .await
            .context("Failed to query directory")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found"))?;
            
        Ok(directory)
    }
    
    pub async fn get_directory_by_domain(
        db: &DatabaseConnection,
        domain: &str
    ) -> Result<directory::Model> {
        let directory = directory::Entity::find()
            .filter(
                sea_orm::Condition::any()
                    .add(directory::Column::Domain.eq(domain))
                    .add(directory::Column::CustomDomain.eq(domain))
                    .add(directory::Column::Subdomain.eq(domain))
            )
            .one(db)
            .await
            .context("Failed to query directory by domain")?
            .ok_or_else(|| anyhow::anyhow!("Directory not found for domain"))?;
            
        Ok(directory)
    }
    
    pub async fn get_directories_by_type(
        db: &DatabaseConnection,
        directory_type_id: Uuid
    ) -> Result<Vec<directory::Model>> {
        let directories = directory::Entity::find()
            .filter(directory::Column::DirectoryTypeId.eq(directory_type_id))
            .all(db)
            .await
            .context("Failed to query directories by type")?;
            
        Ok(directories)
    }
    
    pub async fn get_directory_listings(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<Vec<crate::entities::listing::Model>> {
        // First check if directory exists
        let _directory = Self::get_directory_by_id(db, directory_id).await?;
        
        let listings = crate::entities::listing::Entity::find()
            .filter(crate::entities::listing::Column::DirectoryId.eq(directory_id))
            .all(db)
            .await
            .context("Failed to fetch listings")?;
            
        Ok(listings)
    }
} 