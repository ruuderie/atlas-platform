//! Shared landlord asset detail / children server fns (hub + unit workspace).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveBlockerDto {
    pub code: String,
    pub message: String,
    pub entity_id: Option<Uuid>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ArchiveBlockedBody {
    error: String,
    blockers: Vec<ArchiveBlockerDto>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchiveOutcome {
    pub archived: bool,
    pub blockers: Vec<ArchiveBlockerDto>,
}

/// POST /api/folio/assets/{id}/archive — soft-archive; 409 returns blockers.
#[server(ArchiveFolioAsset, "/api")]
pub async fn archive_folio_asset(
    asset_id: String,
) -> Result<ArchiveOutcome, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let asset_id = Uuid::parse_str(asset_id.trim())
        .map_err(|_| ServerFnError::new("Invalid asset ID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;

    match crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &format!("/api/folio/assets/{asset_id}/archive"),
        &token,
        None,
        &serde_json::json!({}),
    )
    .await
    {
        Ok(_) => Ok(ArchiveOutcome {
            archived: true,
            blockers: vec![],
        }),
        Err(e) => {
            if e.contains("409") || e.contains("archive_blocked") {
                if let Some(start) = e.find('{') {
                    if let Ok(body) = serde_json::from_str::<ArchiveBlockedBody>(&e[start..]) {
                        return Ok(ArchiveOutcome {
                            archived: false,
                            blockers: body.blockers,
                        });
                    }
                }
            }
            Err(ServerFnError::new(format!("Archive failed: {e}")))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDetailDto {
    pub id: Uuid,
    #[serde(default)]
    pub portfolio_id: Option<Uuid>,
    pub parent_asset_id: Option<Uuid>,
    pub asset_type: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub address_line_1: Option<String>,
    #[serde(default)]
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    pub str_eligible: bool,
    #[serde(default)]
    pub str_listing_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetChildDto {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSummaryDto {
    pub id: Uuid,
    pub title: String,
    pub status: String,
    pub estimated_cost_cents: Option<i64>,
    pub actual_spent_cents: i64,
    pub child_count: usize,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetAssetForDispatch, "/api")]
pub async fn get_asset_for_dispatch(id: Uuid) -> Result<AssetDetailDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    crate::atlas_client::authenticated_get_with_headers(
        &format!("/api/folio/assets/{id}"),
        &token,
        None,
        proxy,
    )
    .await
    .map_err(ServerFnError::new)
}

#[server(GetAssetChildren, "/api")]
pub async fn get_asset_children(id: Uuid) -> Result<Vec<AssetChildDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    crate::atlas_client::authenticated_get_with_headers(
        &format!("/api/folio/assets/{id}/children"),
        &token,
        None,
        proxy,
    )
    .await
    .map_err(ServerFnError::new)
}

#[server(GetProjectsForAsset, "/api")]
pub async fn get_projects_for_asset(asset_id: Uuid) -> Result<Vec<ProjectSummaryDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    crate::atlas_client::authenticated_get_with_headers(
        &format!("/api/folio/projects?asset_id={asset_id}"),
        &token,
        None,
        proxy,
    )
    .await
    .map_err(ServerFnError::new)
}

#[derive(Serialize)]
struct CreateChildAssetBody {
    portfolio_id: Uuid,
    parent_asset_id: Option<Uuid>,
    property_type: String,
    name: String,
    address_line_1: String,
    address_line_2: Option<String>,
    city: String,
    state_province: String,
    postal_code: String,
    country_code: String,
    folio_number: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[derive(Deserialize)]
struct IdResp {
    id: Uuid,
}

/// Create a child asset under `parent_id` (unit or space).
#[server(CreateChildAsset, "/api")]
pub async fn create_child_asset(
    parent_id: Uuid,
    name: String,
    property_type: String,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    let parent: AssetDetailDto = crate::atlas_client::authenticated_get_with_headers(
        &format!("/api/folio/assets/{parent_id}"),
        &token,
        None,
        proxy.clone(),
    )
    .await
    .map_err(ServerFnError::new)?;
    let portfolio_id = parent
        .portfolio_id
        .ok_or_else(|| ServerFnError::new("Parent asset has no portfolio_id"))?;
    let body = CreateChildAssetBody {
        portfolio_id,
        parent_asset_id: Some(parent_id),
        property_type,
        name: name.trim().to_string(),
        address_line_1: parent
            .address_line_1
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| parent.name.clone()),
        address_line_2: parent.address_line_2.clone(),
        city: parent.city.clone().unwrap_or_default(),
        state_province: parent.state_province.clone().unwrap_or_default(),
        postal_code: parent.postal_code.clone().unwrap_or_default(),
        country_code: parent
            .country_code
            .clone()
            .unwrap_or_else(|| "US".into()),
        folio_number: None,
        latitude: None,
        longitude: None,
    };
    if body.name.is_empty() {
        return Err(ServerFnError::new("Name is required"));
    }
    let resp: IdResp = crate::atlas_client::authenticated_post_with_headers(
        "/api/folio/assets",
        &token,
        None,
        &body,
        proxy,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp.id)
}

#[derive(Serialize)]
struct CreateProjectBody {
    asset_id: Uuid,
    title: String,
    estimated_cost_cents: Option<i64>,
}

/// POST /api/folio/projects
#[server(CreateProjectForAsset, "/api")]
pub async fn create_project_for_asset(
    asset_id: Uuid,
    title: String,
    estimated_cost_cents: Option<i64>,
) -> Result<Uuid, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    let body = CreateProjectBody {
        asset_id,
        title: title.trim().to_string(),
        estimated_cost_cents,
    };
    if body.title.is_empty() {
        return Err(ServerFnError::new("Title is required"));
    }
    let resp: IdResp = crate::atlas_client::authenticated_post_with_headers(
        "/api/folio/projects",
        &token,
        None,
        &body,
        proxy,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(resp.id)
}
