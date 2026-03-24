use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::ListingAttributeModel;
use reqwest::StatusCode;
use serde_json::json;

pub async fn get_listing_attributes() -> Result<Vec<ListingAttributeModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/listing-attributes");

    let req = client.get(&url);
    let req = with_credentials(req);

    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<ListingAttributeModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch listing attributes: {}", res.status()))
    }
}
