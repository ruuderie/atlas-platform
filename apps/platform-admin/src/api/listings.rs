use reqwest::StatusCode;
use crate::api::client::{api_url, create_client, with_credentials};
use crate::api::models::{ListingModel, ListingCreate, ListingUpdate, ListingWithAttributes};

pub async fn get_listings(directory_id: &str) -> Result<Vec<ListingModel>, String> {
    let client = create_client();
    let url = api_url(&format!("/listings?directory_id={}", directory_id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<ListingModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch listings: {}", res.status()))
    }
}

pub async fn get_listing_by_id(id: &str) -> Result<ListingModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/listings/{}", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<ListingModel>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch listing: {}", res.status()))
    }
}

pub async fn get_listing_with_attributes(id: &str) -> Result<ListingWithAttributes, String> {
    let client = create_client();
    let url = api_url(&format!("/api/listings/{}/with-attributes", id));
    let req = with_credentials(client.get(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<ListingWithAttributes>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to fetch listing with attributes: {}", res.status()))
    }
}

pub async fn create_listing(data: ListingCreate) -> Result<ListingModel, String> {
    let client = create_client();
    let url = api_url("/api/listings");
    let req = with_credentials(client.post(&url)).json(&data);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::CREATED || res.status() == StatusCode::OK {
        res.json::<ListingModel>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to create listing: {}", res.status()))
    }
}

pub async fn update_listing(id: &str, data: ListingUpdate) -> Result<ListingModel, String> {
    let client = create_client();
    let url = api_url(&format!("/api/listings/{}", id));
    let req = with_credentials(client.put(&url)).json(&data);
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<ListingModel>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to update listing: {}", res.status()))
    }
}

pub async fn delete_listing(id: &str) -> Result<(), String> {
    let client = create_client();
    let url = api_url(&format!("/api/listings/{}", id));
    let req = with_credentials(client.delete(&url));
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::NO_CONTENT || res.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(format!("Failed to delete listing: {}", res.status()))
    }
}

pub async fn search_listings(query: &str) -> Result<Vec<ListingModel>, String> {
    let client = create_client();
    let url = api_url(&format!("/listings/search?q={}", query));
    let req = with_credentials(client.get(&url));
    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<ListingModel>>().await { return Ok(data); }
        }
    }
    if crate::api::client::is_demo_mode() { Ok(vec![
        ListingModel { 
            id: "l1".into(), profile_id: "p1".into(), directory_id: "d1".into(), category_id: None, title: "Palantir Foundry Setup".into(), description: "Initialization guide".into(), listing_type: "Guide".into(), price: None, price_type: None, country: None, state: None, city: None, neighborhood: None, latitude: None, longitude: None, additional_info: serde_json::json!({}), status: crate::api::models::ListingStatus::Active, is_featured: false, is_based_on_template: false, based_on_template_id: None, is_ad_placement: false, is_active: true, created_at: "2026-03-18".into(), updated_at: "2026-03-18".into() 
        },
        ListingModel { 
            id: "l2".into(), profile_id: "p1".into(), directory_id: "d1".into(), category_id: None, title: "AWS Lambda Integrations".into(), description: "Connection details".into(), listing_type: "Guide".into(), price: None, price_type: None, country: None, state: None, city: None, neighborhood: None, latitude: None, longitude: None, additional_info: serde_json::json!({}), status: crate::api::models::ListingStatus::Active, is_featured: false, is_based_on_template: false, based_on_template_id: None, is_ad_placement: false, is_active: true, created_at: "2026-03-18".into(), updated_at: "2026-03-18".into() 
        },
    ]) } else { Err("Network Error: Backend unreachable".into()) }
}
