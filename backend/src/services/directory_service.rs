use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, DbErr};
use crate::entities::{directory, prelude::*};
use crate::config::site_config::{SiteConfig, ModuleFlags};
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;
use anyhow::{Result, Context};

pub struct DirectoryService;

impl DirectoryService {
    pub async fn get_directory_config(
        db: &DatabaseConnection,
        directory_id: Uuid
    ) -> Result<SiteConfig> {
        // Find the directory first to ensure it exists
        let directory = Directory::find_by_id(directory_id)
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
            enabled_modules: ModuleFlags::from_bits_truncate(directory.enabled_modules.unwrap_or(0)),
            theme: directory.theme,
            custom_settings: directory.custom_settings
                .map(|v| serde_json::from_value(v).unwrap_or_default())
                .unwrap_or_default(),
            site_status: directory.status,
        };
        
        Ok(config)
    }
    
    pub async fn update_directory_config(
        db: &DatabaseConnection,
        directory_id: Uuid,
        config: SiteConfig
    ) -> Result<SiteConfig> {
        // Find the directory to update
        let directory = Directory::find_by_id(directory_id)
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
        directory.enabled_modules = sea_orm::Set(Some(config.enabled_modules.bits()));
        directory.theme = sea_orm::Set(config.theme);
        directory.custom_settings = sea_orm::Set(
            Some(serde_json::to_value(&config.custom_settings).unwrap_or_default())
        );
        directory.status = sea_orm::Set(config.site_status);
        
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
        let mut query = Directory::find();
        
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
    
    // Add more directory-related methods as needed
} 