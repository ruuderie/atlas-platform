use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::{AccountModel, CreateAccount, LeadModel, CreateLead, DealModel, CreateDeal, UserInfo};
use reqwest::StatusCode;

pub async fn get_users() -> Result<Vec<UserInfo>, String> {
    let client = create_client();
    let url = api_url("/api/users");
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<UserInfo>>().await { return Ok(data); }
        }
    }
    if crate::api::client::is_demo_mode() { Ok(vec![
        UserInfo { id: "u1".into(), first_name: "Alice".into(), last_name: "Smith".into(), email: "alice@foundry.local".into(), is_admin: true },
        UserInfo { id: "u2".into(), first_name: "Bob".into(), last_name: "Jones".into(), email: "bob@foundry.local".into(), is_admin: false },
    ]) } else { Err("Network Error: Backend unreachable".into()) }
}

pub async fn get_accounts() -> Result<Vec<AccountModel>, String> {
    let client = create_client();
    let url = api_url("/api/accounts");
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<AccountModel>>().await { return Ok(data); }
        }
    }
    if crate::api::client::is_demo_mode() { Ok(vec![
        AccountModel { id: "a1".into(), name: "Acme Corp".into() },
        AccountModel { id: "a2".into(), name: "Globex".into() },
    ]) } else { Err("Network Error: Backend unreachable".into()) }
}

pub async fn create_account(data: CreateAccount) -> Result<AccountModel, String> {
    let client = create_client();
    let url = api_url("/api/accounts");
    let req = with_credentials(client.post(&url).json(&data));
    let res = req.send().await.map_err(|e| e.to_string())?;

    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<AccountModel>().await.map_err(|e| e.to_string())
    } else {
        Err("Failed to create account".into())
    }
}

pub async fn get_leads() -> Result<Vec<LeadModel>, String> {
    let client = create_client();
    let url = api_url("/api/leads");
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<LeadModel>>().await { return Ok(data); }
        }
    }
    if crate::api::client::is_demo_mode() { Ok(vec![
        LeadModel { id: "l1".into(), name: "John Doe".into(), first_name: Some("John".into()), last_name: Some("Doe".into()), email: Some("john@acme.com".into()), status: Some("New".into()), is_converted: false },
        LeadModel { id: "l2".into(), name: "Jane Doe".into(), first_name: Some("Jane".into()), last_name: Some("Doe".into()), email: Some("jane@corp.com".into()), status: Some("Contacted".into()), is_converted: false },
    ]) } else { Err("Network Error: Backend unreachable".into()) }
}

pub async fn create_lead(data: CreateLead) -> Result<LeadModel, String> {
    let client = create_client();
    let url = api_url("/api/leads");
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
    let url = api_url("/api/deals");
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<DealModel>>().await { return Ok(data); }
        }
    }
    if crate::api::client::is_demo_mode() { Ok(vec![
        DealModel { id: "d1".into(), name: "Q3 License".into(), amount: 120000.0, stage: "Negotiation".into(), status: "Active".into(), customer_id: "a1".into() },
        DealModel { id: "d2".into(), name: "Annual Renewal".into(), amount: 45000.0, stage: "Closed Won".into(), status: "Won".into(), customer_id: "a2".into() },
    ]) } else { Err("Network Error: Backend unreachable".into()) }
}

pub async fn get_user_by_id(id: &str) -> Result<UserInfo, String> {
    let client = create_client();
    let url = api_url(&format!("/api/users/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<UserInfo>().await.map_err(|e| e.to_string())
    } else if crate::api::client::is_demo_mode() {
        Ok(UserInfo { id: id.to_string(), first_name: "Mock".to_string(), last_name: "User".to_string(), email: "mock@example.com".to_string(), is_admin: true })
    } else {
        Err("Failed to fetch user".into())
    }
}

pub async fn get_account_by_id(id: &str) -> Result<AccountModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/accounts/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<AccountModel>().await.map_err(|e| e.to_string())
    } else if crate::api::client::is_demo_mode() {
        Ok(AccountModel { id: id.to_string(), name: "Mock Account Corp".to_string() })
    } else {
        Err("Failed to fetch account".into())
    }
}

pub async fn get_lead_by_id(id: &str) -> Result<LeadModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/leads/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<LeadModel>().await.map_err(|e| e.to_string())
    } else if crate::api::client::is_demo_mode() {
        Ok(LeadModel { id: id.to_string(), name: "Mock Lead Company".to_string(), first_name: Some("John".to_string()), last_name: Some("Doe".to_string()), email: Some("john@example.com".to_string()), status: Some("Contacted".to_string()), is_converted: false })
    } else {
        Err("Failed to fetch lead".into())
    }
}

pub async fn get_deal_by_id(id: &str) -> Result<DealModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/deals/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    if res.status() == StatusCode::OK {
        res.json::<DealModel>().await.map_err(|e| e.to_string())
    } else if crate::api::client::is_demo_mode() {
        Ok(DealModel { id: id.to_string(), name: "Mock Deal Q4".to_string(), amount: 50000.0, stage: "Negotiation".to_string(), status: "Active".to_string(), customer_id: "A-101".to_string() })
    } else {
        Err("Failed to fetch deal".into())
    }
}
