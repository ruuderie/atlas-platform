use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::client::{api_url, create_client, with_credentials};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogModel {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub action_type: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub old_state: Option<serde_json::Value>,
    pub new_state: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn get_audit_logs(
    tenant_id: Option<Uuid>,
    actor_id: Option<Uuid>,
    entity_id: Option<Uuid>,
    date_from: Option<&str>,
    date_to: Option<&str>,
) -> Result<Vec<AuditLogModel>, String> {
    let client = create_client();
    let mut url = "/api/admin/audit-logs".to_string();

    let mut params = Vec::new();
    if let Some(t) = tenant_id {
        params.push(format!("tenant_id={}", t));
    }
    if let Some(a) = actor_id {
        params.push(format!("actor_id={}", a));
    }
    if let Some(e) = entity_id {
        params.push(format!("entity_id={}", e));
    }
    if let Some(from) = date_from.filter(|s| !s.is_empty()) {
        params.push(format!("date_from={}", from));
    }
    if let Some(to) = date_to.filter(|s| !s.is_empty()) {
        params.push(format!("date_to={}", to));
    }

    if !params.is_empty() {
        url = format!("{}?{}", url, params.join("&"));
    }

    let full_url = api_url(&url);
    let req = client.get(&full_url);
    let req = with_credentials(req);

    let response = req.send().await.map_err(|e| e.to_string())?;

    if response.status().is_success() {
        let logs: Vec<AuditLogModel> = response.json().await.map_err(|e| e.to_string())?;
        Ok(logs)
    } else {
        Err(format!("Error fetching audit logs: {}", response.status()))
    }
}

/// Build the CSV export download URL with the same filters as the list endpoint.
pub fn audit_logs_export_url(
    tenant_id: Option<Uuid>,
    actor_id: Option<Uuid>,
    date_from: &str,
    date_to: &str,
) -> String {
    let mut params = Vec::new();
    if let Some(t) = tenant_id {
        params.push(format!("tenant_id={}", t));
    }
    if let Some(a) = actor_id {
        params.push(format!("actor_id={}", a));
    }
    if !date_from.is_empty() {
        params.push(format!("date_from={}", date_from));
    }
    if !date_to.is_empty() {
        params.push(format!("date_to={}", date_to));
    }
    if params.is_empty() {
        "/api/admin/audit-logs/export".to_string()
    } else {
        format!("/api/admin/audit-logs/export?{}", params.join("&"))
    }
}
