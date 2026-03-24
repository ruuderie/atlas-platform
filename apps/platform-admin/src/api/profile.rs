use crate::api::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateEmailRequest {
    pub new_email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn update_email(new_email: String) -> Result<(), String> {
    let payload = UpdateEmailRequest { new_email };
    let client = create_client();
    let url = api_url("/api/profile/email");
    let req = client.put(&url).json(&payload);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.json::<ApiErrorResponse>().await.map(|e| e.message.unwrap_or("Failed".into())).unwrap_or("Failed".into()))
    }
}

pub async fn update_password(current_password: String, new_password: String) -> Result<(), String> {
    let payload = UpdatePasswordRequest { current_password, new_password };
    let client = create_client();
    let url = api_url("/api/profile/password");
    let req = client.put(&url).json(&payload);
    let req = with_credentials(req);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status().is_success() {
        Ok(())
    } else {
        Err(res.json::<ApiErrorResponse>().await.map(|e| e.message.unwrap_or("Failed".into())).unwrap_or("Failed".into()))
    }
}
