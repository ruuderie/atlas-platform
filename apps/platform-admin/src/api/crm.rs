use super::client::{api_url, create_client, with_credentials};
use super::models::{
    AccountModel, CreateAccount, LeadModel, CreateLead, DealModel, CreateDeal, UserInfo,
    ContactModel, CreateContact, CrmNote, CrmActivity, CrmStatusOption
};
use reqwest::StatusCode;

pub async fn get_users() -> Result<Vec<UserInfo>, String> {
    let client = create_client();
    let url = api_url("/api/admin/users");
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<UserInfo>>().await { return Ok(data); }
        }
    }
    Err("Network Error: Backend unreachable".into())
}

pub async fn get_accounts(
    search: Option<&str>,
    page: u64,
    per_page: u64,
    status: Option<&str>,
    account_type: Option<&str>,
) -> Result<Vec<AccountModel>, String> {
    let client = create_client();
    let mut url = api_url("/api/admin/accounts");
    let mut qp = vec![
        format!("page={}", page),
        format!("per_page={}", per_page),
    ];
    if let Some(q) = search { if !q.is_empty() { qp.push(format!("search={}", urlencoding::encode(q))); } }
    if let Some(s) = status { if s != "all" { qp.push(format!("status={}", urlencoding::encode(s))); } }
    if let Some(t) = account_type { if t != "all" { qp.push(format!("account_type={}", urlencoding::encode(t))); } }
    if !qp.is_empty() { url = format!("{}?{}", url, qp.join("&")); }
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<AccountModel>>().await { return Ok(data); }
        }
    }
    Err("Network Error: Backend unreachable".into())
}


pub async fn create_account(data: CreateAccount) -> Result<AccountModel, String> {
    let client = create_client();
    let url = api_url("/api/admin/accounts");
    let req = with_credentials(client.post(&url).json(&data));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<AccountModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to create account".into())
    }
}

pub async fn get_leads(
    search: Option<&str>,
    page: u64,
    per_page: u64,
    stage: Option<&str>,
    source_prefix: Option<&str>,
) -> Result<Vec<LeadModel>, String> {
    let client = create_client();
    let mut url = api_url("/api/admin/leads");
    let mut qp = vec![
        format!("page={}", page),
        format!("per_page={}", per_page),
    ];
    if let Some(q) = search  { if !q.is_empty() { qp.push(format!("search={}", urlencoding::encode(q))); } }
    if let Some(s) = stage   { if s != "all"    { qp.push(format!("stage={}",  urlencoding::encode(s))); } }
    if let Some(p) = source_prefix { if !p.is_empty() { qp.push(format!("source_prefix={}", urlencoding::encode(p))); } }
    if !qp.is_empty() { url = format!("{}?{}", url, qp.join("&")); }
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<LeadModel>>().await { return Ok(data); }
        }
    }
    Err("Network Error: Backend unreachable".into())
}


pub async fn create_lead(data: CreateLead) -> Result<LeadModel, String> {
    let client = create_client();
    let url = api_url("/api/admin/leads");
    let req = with_credentials(client.post(&url).json(&data));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<LeadModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to create lead".into())
    }
}

pub async fn get_deals() -> Result<Vec<DealModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/deals");
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<DealModel>>().await { return Ok(data); }
        }
    }
    Err("Network Error: Backend unreachable".into())
}

pub async fn create_deal(data: CreateDeal) -> Result<DealModel, String> {
    let client = create_client();
    let url = api_url("/api/admin/deals");
    let req = with_credentials(client.post(&url));
    let res = req.json(&data).send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<DealModel>().await.map_err(|e| format!("Parse error: {e}"))
    } else {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!("Create deal failed (HTTP {}): {}", status.as_u16(), body))
    }
}

pub async fn get_user_by_id(id: &str) -> Result<UserInfo, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/users/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<UserInfo>().await.map_err(|e| e.to_string())

    } else {
        Err("Failed to fetch user".into())
    }
}

pub async fn get_account_by_id(id: &str) -> Result<AccountModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/accounts/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<AccountModel>().await.map_err(|e| e.to_string())

    } else {
        Err("Failed to fetch account".into())
    }
}

pub async fn get_lead_by_id(id: &str) -> Result<LeadModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/leads/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<LeadModel>().await.map_err(|e| e.to_string())

    } else {
        Err("Failed to fetch lead".into())
    }
}

pub async fn get_deal_by_id(id: &str) -> Result<DealModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/deals/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<DealModel>().await.map_err(|e| e.to_string())

    } else {
        Err("Failed to fetch deal".into())
    }
}

pub async fn get_contacts(
    search: Option<&str>,
    page: u64,
    per_page: u64,
    role: Option<&str>,
) -> Result<Vec<ContactModel>, String> {
    let client = create_client();
    let mut url = api_url("/api/admin/contacts");
    let mut qp = vec![
        format!("page={}", page),
        format!("per_page={}", per_page),
    ];
    if let Some(q) = search { if !q.is_empty() { qp.push(format!("search={}", urlencoding::encode(q))); } }
    if let Some(r) = role   { qp.push(format!("role={}", urlencoding::encode(r))); }
    if !qp.is_empty() { url = format!("{}?{}", url, qp.join("&")); }
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<ContactModel>>().await { return Ok(data); }
        }
    }
    Err("Network Error: Backend unreachable".into())
}


pub async fn get_contact_by_id(id: &str) -> Result<ContactModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/contacts/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<ContactModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to fetch contact".into())
    }
}

pub async fn create_contact(data: CreateContact) -> Result<ContactModel, String> {
    let client = create_client();
    let url = api_url("/api/admin/contacts");
    let req = with_credentials(client.post(&url).json(&data));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<ContactModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to create contact".into())
    }
}

pub async fn update_contact(id: &str, data: CreateContact) -> Result<ContactModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/contacts/{}", id));
    let req = with_credentials(client.put(&url).json(&data));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::OK {
        res.json::<ContactModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to update contact".into())
    }
}

pub async fn get_contact_notes(contact_id: &str) -> Result<Vec<CrmNote>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/crm/notes?entity_type=Contact&entity_id={}", contact_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<Vec<CrmNote>>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to fetch contact notes".into())
    }
}

pub async fn add_contact_note(contact_id: &str, content: &str) -> Result<CrmNote, String> {
    let client = create_client();
    let url = api_url("/api/crm/notes");
    let payload = serde_json::json!({
        "entity_type": "Contact",
        "entity_id": uuid::Uuid::parse_str(contact_id).unwrap_or_default(),
        "content": content
    });
    let req = with_credentials(client.post(&url).json(&payload));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<CrmNote>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to add contact note".into())
    }
}

pub async fn get_contact_activities(contact_id: &str) -> Result<Vec<CrmActivity>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/crm/activities?entity_type=Contact&entity_id={}", contact_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<Vec<CrmActivity>>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to fetch contact activities".into())
    }
}

pub async fn log_contact_activity(contact_id: &str, activity_type: &str, description: &str) -> Result<CrmActivity, String> {
    let client = create_client();
    let url = api_url("/api/crm/activities");
    let payload = serde_json::json!({
        "contact_id": uuid::Uuid::parse_str(contact_id).unwrap_or_default(),
        "activity_type": activity_type,
        "title": format!("Logged {}", activity_type),
        "description": description,
        "status": "Completed"
    });
    let req = with_credentials(client.post(&url).json(&payload));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<CrmActivity>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to log contact activity".into())
    }
}

pub async fn get_crm_status_options(object_type: &str) -> Result<Vec<CrmStatusOption>, String> {
    let client = create_client();
    let url = api_url(&format!("/api/crm/status-options?object_type={}", object_type));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<Vec<CrmStatusOption>>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to fetch status options".into())
    }
}

pub async fn convert_lead(id: &str) -> Result<ContactModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/crm/leads/{}/convert", id));
    let req = with_credentials(client.post(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK || res.status() == StatusCode::CREATED {
        res.json::<ContactModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to convert lead".into())
    }
}

pub async fn update_deal(id: &str, stage: &str, status: &str) -> Result<DealModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/deals/{}", id));
    let payload = serde_json::json!({ "stage": stage, "status": status });
    let req = with_credentials(client.put(&url).json(&payload));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<DealModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to update deal".into())
    }
}

pub async fn update_lead(id: &str, status: &str) -> Result<LeadModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/admin/leads/{}", id));
    let payload = serde_json::json!({
        "lead_status": status
    });
    let req = with_credentials(client.put(&url).json(&payload));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<LeadModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to update lead".into())
    }
}

// ── Call Logging ──────────────────────────────────────────────────────────────

/// Log a manual call entry as an Activity (type = Log).
/// Supports optional transcript file IDs (already uploaded to R2 + registered).
/// Calls `POST /api/crm/activities`.
pub async fn log_call_activity(
    lead_id: Option<&str>,
    contact_id: Option<&str>,
    account_id: Option<&str>,
    duration_min: u32,
    direction: &str,   // "inbound" | "outbound"
    outcome: &str,     // "connected" | "voicemail" | "no_answer"
    notes: &str,
    file_storage_paths: Vec<String>,
) -> Result<serde_json::Value, String> {
    let title = format!("Call — {} ({})", outcome, direction);
    let description = if notes.is_empty() { None } else { Some(notes) };

    // Build files array for the activity — each entry needs id + storage_path
    let files: Vec<serde_json::Value> = file_storage_paths
        .into_iter()
        .map(|path| {
            let name = path.split('/').last().unwrap_or("transcript").to_string();
            serde_json::json!({
                "id": uuid::Uuid::new_v4(),
                "name": name,
                "storage_path": path,
            })
        })
        .collect();

    let payload = serde_json::json!({
        "activity_type": "Log",
        "title": title,
        "description": description,
        "status": "Completed",
        "duration_minutes": duration_min,
        "direction": direction,
        "outcome": outcome,
        "lead_id": lead_id,
        "contact_id": contact_id,
        "account_id": account_id,
        "associated_entities": [],
        "files": files,
    });

    let client = create_client();
    let url = api_url("/api/crm/activities");
    let req = with_credentials(client.post(&url).json(&payload));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status().is_success() {
        res.json::<serde_json::Value>().await.map_err(|e| e.to_string())
    } else {
        Err(res.text().await.unwrap_or_default())
    }
}
