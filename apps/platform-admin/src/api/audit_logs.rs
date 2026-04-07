use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};


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
) -> Result<Vec<AuditLogModel>, String> {
    let client = create_client();
    let mut url = "/api/audit-logs".to_string();
    
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
