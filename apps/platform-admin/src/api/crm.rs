use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::{AccountModel, CreateAccount, LeadModel, CreateLead, DealModel, CreateDeal, UserInfo};
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

pub async fn get_accounts() -> Result<Vec<AccountModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/accounts");
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

pub async fn get_leads() -> Result<Vec<LeadModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/leads");
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
