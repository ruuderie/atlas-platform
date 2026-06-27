// apps/folio/src/pages/landlord/map_portfolio.rs
//
// Map Portfolio page — /l/map
//
// Renders a Leaflet.js map with a pin for every portfolio property that has
// lat/lon set. Each pin has a popup with property name, address, status.
//
// Leaflet is loaded via CDN <script> / <link> tags injected into the page head.
// The map itself is initialised in a JS snippet executed after mount.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

// ── API types ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPin {
    pub id:             Uuid,
    pub name:           String,
    pub asset_type:     String,
    pub status:         String,
    pub latitude:       f64,
    pub longitude:      f64,
    pub address_line_1: Option<String>,
    pub city:           Option<String>,
    pub state_province: Option<String>,
    pub postal_code:    Option<String>,
}

// ── Server function ───────────────────────────────────────────────────────────

#[server(FetchMapPins, "/api")]
pub async fn fetch_map_pins() -> Result<Vec<MapPin>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = headers.get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find_map(|p| {
            let p = p.trim();
            p.strip_prefix("session=").map(|t| t.to_string())
        }))
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<MapPin>>(
        "/api/folio/assets/map",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Fetch map pins failed: {e}")))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn status_color(status: &str) -> &str {
    match status {
        "active" | "occupied"    => "#4ade80",
        "vacant"                 => "#fb923c",
        "maintenance"            => "#facc15",
        _                        => "#94a3b8",
    }
}

fn build_map_init_script(pins: &[MapPin]) -> String {
    // Build JSON pin array for the inline JS
    let pins_json = pins.iter().map(|p| {
        let addr = p.address_line_1.as_deref().unwrap_or("");
        let city = p.city.as_deref().unwrap_or("");
        let st   = p.state_province.as_deref().unwrap_or("");
        let zip  = p.postal_code.as_deref().unwrap_or("");
        let color = status_color(&p.status);
        format!(
            r#"{{"lat":{lat},"lng":{lng},"name":{name},"status":{status},"addr":{addr},"city":{city},"state":{state},"zip":{zip},"color":{color}}}"#,
            lat    = p.latitude,
            lng    = p.longitude,
            name   = serde_json::to_string(&p.name).unwrap_or_default(),
            status = serde_json::to_string(&p.status).unwrap_or_default(),
            addr   = serde_json::to_string(addr).unwrap_or_default(),
            city   = serde_json::to_string(city).unwrap_or_default(),
            state  = serde_json::to_string(st).unwrap_or_default(),
            zip    = serde_json::to_string(zip).unwrap_or_default(),
            color  = serde_json::to_string(color).unwrap_or_default(),
        )
    }).collect::<Vec<_>>().join(",");

    let center_lat = if pins.is_empty() { 39.5 } else {
        pins.iter().map(|p| p.latitude).sum::<f64>() / pins.len() as f64
    };
    let center_lng = if pins.is_empty() { -98.35 } else {
        pins.iter().map(|p| p.longitude).sum::<f64>() / pins.len() as f64
    };
    let zoom = if pins.is_empty() { 4 } else if pins.len() == 1 { 14 } else { 10 };

    format!(r#"
(function() {{
  if (window.__atlasMapInit) return;
  window.__atlasMapInit = true;

  var pins = [{pins_json}];

  function initMap() {{
    if (typeof L === 'undefined') {{
      setTimeout(initMap, 100);
      return;
    }}
    var el = document.getElementById('atlas-map');
    if (!el) return;
    if (el._leaflet_id) return;

    var map = L.map('atlas-map', {{
      center: [{center_lat}, {center_lng}],
      zoom: {zoom},
      zoomControl: true,
    }});

    L.tileLayer('https://{{s}}.basemaps.cartocdn.com/dark_all/{{z}}/{{x}}/{{y}}{{r}}.png', {{
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
      subdomains: 'abcd',
      maxZoom: 19
    }}).addTo(map);

    pins.forEach(function(p) {{
      var icon = L.divIcon({{
        className: '',
        html: '<div style="width:14px;height:14px;border-radius:50%;background:' + p.color +
              ';border:2px solid rgba(255,255,255,0.8);box-shadow:0 2px 6px rgba(0,0,0,0.5);"></div>',
        iconSize: [14, 14],
        iconAnchor: [7, 7],
      }});
      var addrLine = [p.addr, p.city, p.state, p.zip].filter(Boolean).join(', ');
      var popup = '<div style="font-family:Inter,sans-serif;min-width:180px;">' +
        '<div style="font-weight:700;font-size:0.95rem;color:#f1f5f9;margin-bottom:4px;">' + p.name + '</div>' +
        '<div style="font-size:0.8rem;color:#94a3b8;margin-bottom:6px;">' + addrLine + '</div>' +
        '<span style="display:inline-block;padding:2px 8px;border-radius:999px;font-size:0.7rem;font-weight:700;background:rgba(255,255,255,0.1);color:' + p.color + ';">' + p.status + '</span>' +
        '</div>';
      L.marker([p.lat, p.lng], {{ icon: icon }})
        .addTo(map)
        .bindPopup(popup, {{ className: 'atlas-map-popup' }});
    }});

    // Fit bounds if multiple pins
    if (pins.length > 1) {{
      var bounds = pins.map(function(p) {{ return [p.lat, p.lng]; }});
      map.fitBounds(bounds, {{ padding: [40, 40] }});
    }}
  }}

  if (document.readyState === 'loading') {{
    document.addEventListener('DOMContentLoaded', initMap);
  }} else {{
    initMap();
  }}
}})();
"#,
        pins_json  = pins_json,
        center_lat = center_lat,
        center_lng = center_lng,
        zoom       = zoom,
    )
}

// ── KPI strip ─────────────────────────────────────────────────────────────────

#[component]
fn MapKpiStrip(pins: Vec<MapPin>) -> impl IntoView {
    let total     = pins.len();
    let active    = pins.iter().filter(|p| p.status == "active" || p.status == "occupied").count();
    let vacant    = pins.iter().filter(|p| p.status == "vacant").count();
    let no_coords = pins.iter().filter(|p| p.latitude == 0.0 && p.longitude == 0.0).count();

    view! {
        <div class="map-kpi-strip">
            <div class="map-kpi-card">
                <span class="map-kpi-icon material-symbols-outlined">"location_on"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{total}</span>
                    <span class="map-kpi-label">"Mapped Properties"</span>
                </div>
            </div>
            <div class="map-kpi-card map-kpi-card--active">
                <span class="map-kpi-icon material-symbols-outlined">"check_circle"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{active}</span>
                    <span class="map-kpi-label">"Active / Occupied"</span>
                </div>
            </div>
            <div class="map-kpi-card map-kpi-card--vacant">
                <span class="map-kpi-icon material-symbols-outlined">"door_open"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{vacant}</span>
                    <span class="map-kpi-label">"Vacant"</span>
                </div>
            </div>
            <div class="map-kpi-card">
                <span class="map-kpi-icon material-symbols-outlined">"location_off"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{no_coords}</span>
                    <span class="map-kpi-label">"No Coordinates"</span>
                </div>
            </div>
        </div>
    }
}

// ── Property list sidebar ─────────────────────────────────────────────────────

#[component]
fn MapPinList(pins: Vec<MapPin>) -> impl IntoView {
    view! {
        <div class="map-pin-list">
            <div class="map-pin-list-header">"Properties"</div>
            {pins.into_iter().map(|p| {
                let addr = [
                    p.address_line_1.as_deref().unwrap_or(""),
                    p.city.as_deref().unwrap_or(""),
                ].iter().filter(|s| !s.is_empty()).cloned().collect::<Vec<_>>().join(", ");
                let color = status_color(&p.status).to_string();
                view! {
                    <div class="map-pin-item">
                        <div class="map-pin-dot" style=format!("background:{color};")></div>
                        <div class="map-pin-info">
                            <span class="map-pin-name">{p.name}</span>
                            <span class="map-pin-addr">{addr}</span>
                        </div>
                        <span class="map-pin-status"
                            style=format!("color:{color};"
                        )>{p.status.clone()}</span>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

// ── Main page ─────────────────────────────────────────────────────────────────

#[component]
pub fn MapPortfolio() -> impl IntoView {
    let pins = Resource::new(|| (), |_| fetch_map_pins());

    view! {
        // Leaflet CSS + JS via CDN
        <leptos_meta::Link
            rel="stylesheet"
            href="https://unpkg.com/leaflet@1.9.4/dist/leaflet.css"
            integrity="sha256-p4NxAoJBhIIN+hmNHrzRCf9tD/miZyoHS5obTRR9BMY="
            crossorigin="anonymous"
        />
        <leptos_meta::Script
            src="https://unpkg.com/leaflet@1.9.4/dist/leaflet.js"
            integrity="sha256-20nQCchB9co0qIjJZRGuk2/Z9VM+kNiyxNV/XN/WLs="
            crossorigin="anonymous"
        />

        <div class="map-page">
            <div class="map-header">
                <h1 class="map-title">"Portfolio Map"</h1>
                <p class="map-subtitle">"Geographic view of all properties with coordinates"</p>
            </div>

            <Suspense fallback=|| view! {
                <div class="map-kpi-strip map-kpi-strip--loading">
                    {(0..4).map(|_| view! { <div class="map-kpi-skel"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || pins.get().map(|res| match res {
                    Ok(data) => view! { <MapKpiStrip pins=data /> }.into_any(),
                    Err(_)   => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            <div class="map-body">
                // Sidebar list
                <Suspense fallback=|| view! { <div class="map-pin-list map-pin-list--loading"></div> }>
                    {move || pins.get().map(|res| match res {
                        Ok(data) => view! { <MapPinList pins=data /> }.into_any(),
                        Err(_)   => view! { <div></div> }.into_any(),
                    })}
                </Suspense>

                // Map container
                <div class="map-container">
                    <Suspense fallback=|| view! {
                        <div class="map-loading">
                            <span class="material-symbols-outlined map-spin">"public"</span>
                            <p>"Loading map…"</p>
                        </div>
                    }>
                        {move || pins.get().map(|res| match res {
                            Err(e) => view! {
                                <div class="map-error">
                                    <span class="material-symbols-outlined">"error"</span>
                                    <p>"Failed to load properties: " {e.to_string()}</p>
                                </div>
                            }.into_any(),
                            Ok(data) if data.is_empty() => view! {
                                <div class="map-empty">
                                    <span class="material-symbols-outlined map-empty-icon">"location_off"</span>
                                    <p class="map-empty-title">"No geo-coded properties"</p>
                                    <p class="map-empty-sub">
                                        "Add latitude and longitude when creating or editing properties."
                                    </p>
                                </div>
                            }.into_any(),
                            Ok(data) => {
                                let script = build_map_init_script(&data);
                                view! {
                                    <div id="atlas-map" class="map-leaflet"></div>
                                    <leptos_meta::Script>{script}</leptos_meta::Script>
                                }.into_any()
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
