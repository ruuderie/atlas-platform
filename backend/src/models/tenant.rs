use chrono::{Utc, DateTime};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use sea_orm::ActiveValue::Set;
use crate::entities::tenant;

#[derive(Debug, Deserialize, Serialize)]
pub struct TenantModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

impl From<tenant::Model> for TenantModel {
    fn from(model: tenant::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            description: model.description,
            created_at: model.created_at,
            updated_at: model.updated_at,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTenant {
    pub name: String,
    pub description: String,
}

impl From<CreateTenant> for tenant::ActiveModel {
    fn from(input: CreateTenant) -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            name: Set(input.name),
            description: Set(input.description),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTenant {
    pub name: Option<String>,
    pub description: Option<String>,
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
