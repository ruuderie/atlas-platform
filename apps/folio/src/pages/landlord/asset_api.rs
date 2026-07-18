//! Shared landlord asset detail / children server fns (hub + unit workspace).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetDetailDto {
    pub id: Uuid,
    pub parent_asset_id: Option<Uuid>,
    pub asset_type: String,
    pub name: String,
    pub status: String,
    pub city: Option<String>,
    pub state_province: Option<String>,
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
