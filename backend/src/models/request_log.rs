use chrono::{Utc, DateTime, Duration};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use sea_orm::prelude::*;
use serde_json::Value;
use sea_orm::{IntoActiveModel, Set};
use sea_orm::sea_query::StringLen;
use crate::entities::listing;
use std::str::FromStr;
use axum::http::request::Parts;
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum RequestStatus {
    #[sea_orm(string_value = "success")]
    Success,
    #[sea_orm(string_value = "failure")]
    Failure,
}
// Add this to your models.rs file or create it if it doesn't exist

pub struct RequestInfo {
    pub method: String,
    pub uri: String,
    pub status_code: i32,
    pub user_id: Option<Uuid>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub request_type: RequestType,
    pub created_at: DateTime<Utc>,
    pub request_status: RequestStatus,
    pub failure_reason: Option<String>,
}

impl RequestInfo {
    pub fn from_parts(parts: &Parts) -> Self {
        let ip_address = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let user_agent = parts
            .headers
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        RequestInfo {
            method: parts.method.to_string(),
            uri: parts.uri.to_string(),
            status_code: 200, // Default to 200, update later if needed
            user_id: None, // This should be set elsewhere, e.g., after user authentication
            ip_address,
            user_agent,
            request_type: RequestType::API, // Default to API, update if needed
            created_at: Utc::now(),
            request_status: RequestStatus::Success, // Default to Success, update if needed
            failure_reason: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub path: String,
    pub method: String,
    pub status_code: i32,
    pub request_type: RequestType,
    pub created_at: DateTime<Utc>,
    pub request_status: RequestStatus,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(32))")]
pub enum RequestType {
    #[sea_orm(string_value = "login")]
    Login,
    #[sea_orm(string_value = "api")]
    API,
}
