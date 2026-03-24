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
    let url = api_url(&format!("/api/admin/listings/{}", id));
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
    let url = api_url(&format!("/api/admin/listings/{}/with-attributes", id));
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
    let url = api_url("/api/admin/listings");
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
    let url = api_url(&format!("/api/admin/listings/{}", id));
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
    let url = api_url(&format!("/api/admin/listings/{}", id));
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
    
    let res = req.send().await.map_err(|e| e.to_string())?;
    
    if res.status() == StatusCode::OK {
        res.json::<Vec<ListingModel>>().await.map_err(|e| e.to_string())
    } else {
        Err(format!("Failed to search listings: {}", res.status()))
    }
}
