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

/// POST /api/folio/assets/{id}/purge — hard-delete property tree. Confirm must be PURGE.
#[server(PurgeFolioAsset, "/api")]
pub async fn purge_folio_asset(asset_id: String) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let asset_id = Uuid::parse_str(asset_id.trim())
        .map_err(|_| ServerFnError::new("Invalid asset ID"))?;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;

    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &format!("/api/folio/assets/{asset_id}/purge"),
        &token,
        None,
        &serde_json::json!({ "confirm": "PURGE" }),
    )
    .await
    .map_err(|e| ServerFnError::new(format!("Purge failed: {e}")))?;
    Ok(())
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

/// Beds / baths / size / year / notes — `attributes.property_details`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PropertyDetailsDto {
    #[serde(default)]
    pub beds: Option<f64>,
    #[serde(default)]
    pub baths: Option<f64>,
    #[serde(default)]
    pub sqft: Option<i32>,
    #[serde(default)]
    pub year_built: Option<i32>,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Purchase / debt figures in cents — `attributes.capital`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CapitalDto {
    #[serde(default)]
    pub purchase_price_cents: Option<i64>,
    #[serde(default)]
    pub mortgage_balance_cents: Option<i64>,
    #[serde(default)]
    pub other_debt_cents: Option<i64>,
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
    /// Raw attributes JSON (may include `coordinates.{lat,lng}`).
    #[serde(default)]
    pub attributes: Option<serde_json::Value>,
    /// WGS-84 latitude — top-level if API sends it, else parsed from attributes.
    #[serde(default)]
    pub latitude: Option<f64>,
    /// WGS-84 longitude — top-level if API sends it, else parsed from attributes.
    #[serde(default)]
    pub longitude: Option<f64>,
    /// Parsed from `attributes.property_details` after load.
    #[serde(default)]
    pub property_details: Option<PropertyDetailsDto>,
    /// Parsed from `attributes.capital` after load.
    #[serde(default)]
    pub capital: Option<CapitalDto>,
}

impl AssetDetailDto {
    /// Fill lat/lng from `attributes.coordinates` when top-level fields are absent.
    pub fn with_coords_from_attributes(mut self) -> Self {
        if self.latitude.is_some() && self.longitude.is_some() {
            return self;
        }
        if let Some(coords) = self
            .attributes
            .as_ref()
            .and_then(|a| a.get("coordinates"))
        {
            if self.latitude.is_none() {
                self.latitude = coords.get("lat").and_then(|v| v.as_f64());
            }
            if self.longitude.is_none() {
                self.longitude = coords.get("lng").and_then(|v| v.as_f64());
            }
        }
        self
    }

    /// Parse `property_details` and `capital` from attributes (like coordinates).
    pub fn with_details_from_attributes(mut self) -> Self {
        let Some(attrs) = self.attributes.as_ref() else {
            return self;
        };
        if self.property_details.is_none() {
            self.property_details = attrs
                .get("property_details")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
        }
        if self.capital.is_none() {
            self.capital = attrs
                .get("capital")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
        }
        self
    }

    /// Coords + details + capital from attributes.
    pub fn with_attrs_parsed(self) -> Self {
        self.with_coords_from_attributes()
            .with_details_from_attributes()
    }

    /// Non-zero pin coordinates, if set.
    pub fn coords(&self) -> Option<(f64, f64)> {
        match (self.latitude, self.longitude) {
            (Some(lat), Some(lng)) if lat != 0.0 || lng != 0.0 => Some((lat, lng)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetCoordinatesDto {
    pub lat: f64,
    pub lng: f64,
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
    let detail: AssetDetailDto = crate::atlas_client::authenticated_get_with_headers(
        &format!("/api/folio/assets/{id}"),
        &token,
        None,
        proxy,
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(detail.with_attrs_parsed())
}

/// PUT /api/folio/assets/{id}/details — merge attributes.property_details.
#[server(PutAssetDetails, "/api")]
pub async fn put_asset_details(
    asset_id: Uuid,
    details: PropertyDetailsDto,
) -> Result<PropertyDetailsDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_put(
        &format!("/api/folio/assets/{asset_id}/details"),
        &token,
        None,
        &details,
    )
    .await
    .map_err(ServerFnError::new)
}

/// PUT /api/folio/assets/{id}/capital — merge attributes.capital.
#[server(PutAssetCapital, "/api")]
pub async fn put_asset_capital(
    asset_id: Uuid,
    capital: CapitalDto,
) -> Result<CapitalDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_put(
        &format!("/api/folio/assets/{asset_id}/capital"),
        &token,
        None,
        &capital,
    )
    .await
    .map_err(ServerFnError::new)
}

/// POST /api/folio/assets/{id}/geocode — Nominatim from stored address.
#[server(GeocodeAsset, "/api")]
pub async fn geocode_asset(asset_id: Uuid) -> Result<AssetCoordinatesDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let proxy = crate::atlas_client::folio_proxy_headers(&headers);
    crate::atlas_client::authenticated_post_with_headers(
        &format!("/api/folio/assets/{asset_id}/geocode"),
        &token,
        None,
        &serde_json::json!({}),
        proxy,
    )
    .await
    .map_err(ServerFnError::new)
}

/// PUT /api/folio/assets/{id}/coordinates
#[server(SetAssetCoordinates, "/api")]
pub async fn set_asset_coordinates(
    asset_id: Uuid,
    lat: f64,
    lng: f64,
) -> Result<AssetCoordinatesDto, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    #[derive(Serialize)]
    struct Body {
        lat: f64,
        lng: f64,
    }
    crate::atlas_client::authenticated_put(
        &format!("/api/folio/assets/{asset_id}/coordinates"),
        &token,
        None,
        &Body { lat, lng },
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
    .map_err(ServerFnError::new)
    .map(AssetDetailDto::with_attrs_parsed)?;
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
