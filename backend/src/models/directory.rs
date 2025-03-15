use chrono::{Utc, DateTime, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use sea_orm::ActiveValue::{Set, NotSet};
use crate::entities::directory;

#[derive(Debug, Deserialize, Serialize)]
pub struct DirectoryModel {
    pub id: Uuid,
    pub name: String,
    pub directory_type_id: Uuid,  // Added
    pub domain: String,
    pub description: String,  // Added
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub enabled_modules: u32,
    pub theme: Option<String>,
    pub custom_settings: Option<HashMap<String, serde_json::Value>>,
    pub site_status: String,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub logo: Option<String>,
    pub favicon: Option<String>,
    pub header_scripts: Option<String>,
    pub footer_scripts: Option<String>,
    pub google_analytics_id: Option<String>,
    pub google_site_verification: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub meta_title: Option<String>,
    pub page_title: Option<String>,
    pub page_description: Option<String>,
    pub page_keywords: Option<String>,
    pub canonical_url: Option<String>,
}

impl From<directory::Model> for DirectoryModel {
    fn from(model: directory::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            directory_type_id: model.directory_type_id,
            domain: model.domain,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
            enabled_modules: model.enabled_modules,
            theme: model.theme,
            custom_settings: model.custom_settings
                .map(|v| v.as_object()
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default()),
            site_status: model.site_status,
            subdomain: model.subdomain,
            custom_domain: model.custom_domain,
            logo: model.logo,
            favicon: model.favicon,
            header_scripts: model.header_scripts,
            footer_scripts: model.footer_scripts,
            google_analytics_id: model.google_analytics_id,
            google_site_verification: model.google_site_verification,
            meta_description: model.meta_description,
            meta_keywords: model.meta_keywords,
            meta_title: model.meta_title,
            page_title: model.page_title,
            page_description: model.page_description,
            page_keywords: model.page_keywords,
            canonical_url: model.canonical_url,
        }
    }
}

impl From<directory::ActiveModel> for DirectoryModel {
    fn from(input: directory::ActiveModel) -> Self {
        Self {
            id: input.id.unwrap(),
            name: input.name.unwrap(),
            directory_type_id: input.directory_type_id.unwrap(),
            domain: input.domain.unwrap(),
            description: input.description.unwrap(),
            created_at: input.created_at.unwrap(),
            updated_at: input.updated_at.unwrap(),
            enabled_modules: input.enabled_modules.unwrap(),
            theme: input.theme.unwrap(),
            custom_settings: input.custom_settings.unwrap()
                .map(|v| v.as_object()
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default()),
            site_status: input.site_status.unwrap(),
            subdomain: input.subdomain.unwrap(),
            custom_domain: input.custom_domain.unwrap(),
            logo: input.logo.unwrap(),
            favicon: input.favicon.unwrap(),
            header_scripts: input.header_scripts.unwrap(),
            footer_scripts: input.footer_scripts.unwrap(),
            google_analytics_id: input.google_analytics_id.unwrap(),
            google_site_verification: input.google_site_verification.unwrap(),
            meta_description: input.meta_description.unwrap(),
            meta_keywords: input.meta_keywords.unwrap(),
            meta_title: input.meta_title.unwrap(),
            page_title: input.page_title.unwrap(),
            page_description: input.page_description.unwrap(),
            page_keywords: input.page_keywords.unwrap(),
            canonical_url: input.canonical_url.unwrap(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateDirectory {
    pub name: String,
    pub directory_type_id: Uuid,
    pub domain: String,
    pub description: String,
}

impl From<CreateDirectory> for directory::ActiveModel {
    fn from(input: CreateDirectory) -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            name: Set(input.name),
            directory_type_id: Set(input.directory_type_id),
            domain: Set(input.domain),
            description: Set(input.description),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            custom_domain: NotSet,
            custom_settings: NotSet,
            enabled_modules: NotSet,
            theme: NotSet,
            site_status: NotSet,
            subdomain: NotSet,
            logo: NotSet,
            favicon: NotSet,
            header_scripts: NotSet,
            footer_scripts: NotSet,
            google_analytics_id: NotSet,
            google_site_verification: NotSet,
            meta_description: NotSet,
            meta_keywords: NotSet,
            meta_title: NotSet,
            page_title: NotSet,
            page_description: NotSet,
            page_keywords: NotSet,
            canonical_url: NotSet,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateDirectory {
    pub name: Option<String>,
    pub directory_type_id: Option<Uuid>,
    pub domain: Option<String>,
    pub description: Option<String>,
    pub enabled_modules: Option<u32>,
    pub theme: Option<String>,
    pub custom_settings: Option<HashMap<String, serde_json::Value>>,
    pub site_status: Option<String>,
    pub subdomain: Option<String>,
    pub custom_domain: Option<String>,
    pub logo: Option<String>,
    pub favicon: Option<String>,
    pub header_scripts: Option<String>,
    pub footer_scripts: Option<String>,
    pub google_analytics_id: Option<String>,
    pub google_site_verification: Option<String>,
    pub meta_description: Option<String>,
    pub meta_keywords: Option<String>,
    pub meta_title: Option<String>,
    pub page_title: Option<String>,
    pub page_description: Option<String>,
    pub page_keywords: Option<String>,
    pub canonical_url: Option<String>,
}
