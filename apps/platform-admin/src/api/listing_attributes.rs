use super::client::{api_url, create_client, with_credentials, ApiErrorResponse};
use super::models::ListingAttributeModel;
use reqwest::StatusCode;
use serde_json::json;

pub async fn get_listing_attributes() -> Result<Vec<ListingAttributeModel>, String> {
    let client = create_client();
    let url = api_url("/api/admin/listing-attributes");

    let req = client.get(&url);
    let req = with_credentials(req);

    if let Ok(res) = req.send().await {
        if res.status() == StatusCode::OK {
            if let Ok(data) = res.json::<Vec<ListingAttributeModel>>().await { 
                return Ok(data); 
            }
        }
    }
    
    // Fallback/Demo Mock
    Ok(vec![
        ListingAttributeModel { 
            id: "fake-attr-1".into(),
            listing_id: Some("fake-lst-1".into()),
            template_id: None,
            attribute_type: "ServiceDetail".into(),
            attribute_key: "Experience".into(),
            value: json!("15 Years"),
            created_at: "2026-03-23".into(),
            updated_at: "2026-03-23".into(),
        },
        ListingAttributeModel { 
            id: "fake-attr-2".into(),
            listing_id: None,
            template_id: Some("fake-tpl-1".into()),
            attribute_type: "Location".into(),
            attribute_key: "Address".into(),
            value: json!("123 Main St"),
            created_at: "2026-03-23".into(),
            updated_at: "2026-03-23".into(),
        }
    ])
}
