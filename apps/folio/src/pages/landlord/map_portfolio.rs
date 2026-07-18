//! Ops map — `/l/map`
//!
//! Leaflet pins for geo-coded assets with maintenance / STR layers.

use crate::components::nav::FolioRoute;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapPin {
    pub id: Uuid,
    pub name: String,
    pub asset_type: String,
    pub status: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address_line_1: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    #[serde(default)]
    pub open_wo_count: i64,
    #[serde(default)]
    pub next_wo_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub str_eligible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MapLayer {
    All,
    Maintenance,
    Str,
    Status,
}

impl MapLayer {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Maintenance => "Maintenance",
            Self::Str => "STR",
            Self::Status => "Status",
        }
    }

    fn from_query(s: &str) -> Self {
        match s {
            "maintenance" => Self::Maintenance,
            "str" => Self::Str,
            "status" => Self::Status,
            _ => Self::All,
        }
    }
}

#[server(FetchMapPins, "/api")]
pub async fn fetch_map_pins() -> Result<Vec<MapPin>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get::<Vec<MapPin>>("/api/folio/assets/map", &token, None)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Fetch map pins failed: {e}")))
}

fn pin_color(p: &MapPin, layer: MapLayer) -> &'static str {
    match layer {
        MapLayer::Maintenance if p.open_wo_count > 0 => "#ef4444",
        MapLayer::Str if p.str_eligible => "#a78bfa",
        _ => match p.status.as_str() {
            "active" | "occupied" => "#4ade80",
            "vacant" => "#fb923c",
            "maintenance" => "#facc15",
            _ => "#94a3b8",
        },
    }
}

fn filter_pins(pins: &[MapPin], layer: MapLayer) -> Vec<MapPin> {
    pins.iter()
        .filter(|p| match layer {
            MapLayer::All | MapLayer::Status => true,
            MapLayer::Maintenance => p.open_wo_count > 0,
            MapLayer::Str => p.str_eligible,
        })
        .cloned()
        .collect()
}

fn build_map_init_script(pins: &[MapPin], layer: MapLayer) -> String {
    let pins_json = pins
        .iter()
        .map(|p| {
            let addr = p.address_line_1.as_deref().unwrap_or("");
            let city = p.city.as_deref().unwrap_or("");
            let st = p.state_province.as_deref().unwrap_or("");
            let zip = p.postal_code.as_deref().unwrap_or("");
            let color = pin_color(p, layer);
            let href = FolioRoute::LandlordAssetDetail
                .path()
                .replace(":id", &p.id.to_string());
            let wo_new = format!(
                "{}?asset_id={}",
                FolioRoute::LandlordMaintenanceNew.path(),
                p.id
            );
            let next = p
                .next_wo_at
                .map(|d| d.format("%b %d").to_string())
                .unwrap_or_default();
            format!(
                r#"{{"lat":{lat},"lng":{lng},"name":{name},"status":{status},"addr":{addr},"city":{city},"state":{state},"zip":{zip},"color":{color},"href":{href},"woNew":{wo_new},"woCount":{wo},"nextWo":{next},"str":{str_el}}}"#,
                lat = p.latitude,
                lng = p.longitude,
                name = serde_json::to_string(&p.name).unwrap_or_default(),
                status = serde_json::to_string(&p.status).unwrap_or_default(),
                addr = serde_json::to_string(addr).unwrap_or_default(),
                city = serde_json::to_string(city).unwrap_or_default(),
                state = serde_json::to_string(st).unwrap_or_default(),
                zip = serde_json::to_string(zip).unwrap_or_default(),
                color = serde_json::to_string(color).unwrap_or_default(),
                href = serde_json::to_string(&href).unwrap_or_default(),
                wo_new = serde_json::to_string(&wo_new).unwrap_or_default(),
                wo = p.open_wo_count,
                next = serde_json::to_string(&next).unwrap_or_default(),
                str_el = if p.str_eligible { "true" } else { "false" },
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let center_lat = if pins.is_empty() {
        39.5
    } else {
        pins.iter().map(|p| p.latitude).sum::<f64>() / pins.len() as f64
    };
    let center_lng = if pins.is_empty() {
        -98.35
    } else {
        pins.iter().map(|p| p.longitude).sum::<f64>() / pins.len() as f64
    };
    let zoom = if pins.is_empty() {
        4
    } else if pins.len() == 1 {
        14
    } else {
        10
    };

    // Reset init flag so layer changes remount the map.
    format!(
        r#"
(function() {{
  window.__atlasMapInit = false;

  var pins = [{pins_json}];

  function initMap() {{
    if (typeof L === 'undefined') {{
      setTimeout(initMap, 100);
      return;
    }}
    var el = document.getElementById('atlas-map');
    if (!el) return;
    if (el._leaflet_id) {{
      el._leaflet_id = null;
      el.innerHTML = '';
    }}

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
      var badge = p.woCount > 0
        ? '<span style="position:absolute;top:-6px;right:-6px;min-width:16px;height:16px;padding:0 4px;border-radius:999px;background:#ef4444;color:#fff;font:700 9px/16px Inter,sans-serif;text-align:center;">' + p.woCount + '</span>'
        : '';
      var icon = L.divIcon({{
        className: '',
        html: '<div style="position:relative;width:14px;height:14px;border-radius:50%;background:' + p.color +
              ';border:2px solid rgba(255,255,255,0.8);box-shadow:0 2px 6px rgba(0,0,0,0.5);">' + badge + '</div>',
        iconSize: [14, 14],
        iconAnchor: [7, 7],
      }});
      var addrLine = [p.addr, p.city, p.state, p.zip].filter(Boolean).join(', ');
      var woLine = p.woCount > 0
        ? '<div style="font-size:0.75rem;color:#fca5a5;margin:6px 0;">' + p.woCount + ' open WO' + (p.nextWo ? ' · next ' + p.nextWo : '') + '</div>'
        : '';
      var popup = '<div style="font-family:Inter,sans-serif;min-width:200px;">' +
        '<div style="font-weight:700;font-size:0.95rem;color:#f1f5f9;margin-bottom:4px;">' + p.name + '</div>' +
        '<div style="font-size:0.8rem;color:#94a3b8;margin-bottom:6px;">' + addrLine + '</div>' +
        '<span style="display:inline-block;padding:2px 8px;border-radius:999px;font-size:0.7rem;font-weight:700;background:rgba(255,255,255,0.1);color:' + p.color + ';">' + p.status + '</span>' +
        (p.str ? ' <span style="display:inline-block;padding:2px 8px;border-radius:999px;font-size:0.7rem;font-weight:700;background:rgba(167,139,250,0.2);color:#a78bfa;">STR</span>' : '') +
        woLine +
        '<div style="display:flex;gap:8px;margin-top:8px;">' +
        '<a href="' + p.href + '" style="font-size:0.75rem;font-weight:700;color:#93c5fd;">Open building</a>' +
        '<a href="' + p.woNew + '" style="font-size:0.75rem;font-weight:700;color:#fca5a5;">Create WO</a>' +
        '</div></div>';
      L.marker([p.lat, p.lng], {{ icon: icon }})
        .addTo(map)
        .bindPopup(popup, {{ className: 'atlas-map-popup' }});
    }});

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
        pins_json = pins_json,
        center_lat = center_lat,
        center_lng = center_lng,
        zoom = zoom,
    )
}

#[component]
fn MapKpiStrip(pins: Vec<MapPin>) -> impl IntoView {
    let total = pins.len();
    let with_wo = pins.iter().filter(|p| p.open_wo_count > 0).count();
    let str_n = pins.iter().filter(|p| p.str_eligible).count();
    let vacant = pins.iter().filter(|p| p.status == "vacant").count();

    view! {
        <div class="map-kpi-strip">
            <div class="map-kpi-card">
                <span class="map-kpi-icon material-symbols-outlined">"location_on"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{total}</span>
                    <span class="map-kpi-label">"Mapped"</span>
                </div>
            </div>
            <div class="map-kpi-card map-kpi-card--vacant">
                <span class="map-kpi-icon material-symbols-outlined">"build"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{with_wo}</span>
                    <span class="map-kpi-label">"With open WO"</span>
                </div>
            </div>
            <div class="map-kpi-card map-kpi-card--active">
                <span class="map-kpi-icon material-symbols-outlined">"vacation"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{str_n}</span>
                    <span class="map-kpi-label">"STR-eligible"</span>
                </div>
            </div>
            <div class="map-kpi-card">
                <span class="map-kpi-icon material-symbols-outlined">"door_open"</span>
                <div class="map-kpi-body">
                    <span class="map-kpi-value">{vacant}</span>
                    <span class="map-kpi-label">"Vacant"</span>
                </div>
            </div>
        </div>
    }
}

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
                let color = pin_color(&p, MapLayer::All).to_string();
                let href = FolioRoute::LandlordAssetDetail
                    .path()
                    .replace(":id", &p.id.to_string());
                let wo = p.open_wo_count;
                view! {
                    <a class="map-pin-item press" href=href style="text-decoration:none;color:inherit;">
                        <div class="map-pin-dot" style=format!("background:{color};")></div>
                        <div class="map-pin-info">
                            <span class="map-pin-name">{p.name}</span>
                            <span class="map-pin-addr">{addr}</span>
                        </div>
                        <span class="map-pin-status" style=format!("color:{color};")>
                            {if wo > 0 { format!("{wo} WO") } else { p.status.clone() }}
                        </span>
                    </a>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}

#[component]
pub fn MapPortfolio() -> impl IntoView {
    let query = use_query_map();
    let layer = RwSignal::new(MapLayer::All);

    Effect::new(move |_| {
        if let Some(l) = query.get().get("layer") {
            layer.set(MapLayer::from_query(&l));
        }
    });

    let pins = Resource::new(|| (), |_| fetch_map_pins());

    view! {
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
                <h1 class="map-title">"Ops map"</h1>
                <p class="map-subtitle">"Properties, maintenance, and STR — filter layers below"</p>
            </div>

            <div class="map-layer-chips" role="tablist" aria-label="Map layers">
                {[MapLayer::All, MapLayer::Maintenance, MapLayer::Str, MapLayer::Status]
                    .into_iter()
                    .map(|l| {
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    if layer.get() == l {
                                        "map-layer-chip map-layer-chip--active"
                                    } else {
                                        "map-layer-chip"
                                    }
                                }
                                on:click=move |_| layer.set(l)
                            >
                                {l.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>

            <Suspense fallback=|| view! {
                <div class="map-kpi-strip map-kpi-strip--loading">
                    {(0..4).map(|_| view! { <div class="map-kpi-skel"></div> }).collect::<Vec<_>>()}
                </div>
            }>
                {move || pins.get().map(|res| match res {
                    Ok(data) => {
                        let filtered = filter_pins(&data, layer.get());
                        view! { <MapKpiStrip pins=filtered /> }.into_any()
                    }
                    Err(_) => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            <div class="map-body">
                <Suspense fallback=|| view! { <div class="map-pin-list map-pin-list--loading"></div> }>
                    {move || pins.get().map(|res| match res {
                        Ok(data) => {
                            let filtered = filter_pins(&data, layer.get());
                            view! { <MapPinList pins=filtered /> }.into_any()
                        }
                        Err(_) => view! { <div></div> }.into_any(),
                    })}
                </Suspense>

                <div class="map-container">
                    <Suspense fallback=|| view! {
                        <div class="map-loading">
                            <span class="material-symbols-outlined map-spin">"public"</span>
                            <p>"Loading map…"</p>
                        </div>
                    }>
                        {move || {
                            let l = layer.get();
                            pins.get().map(|res| match res {
                            Err(e) => view! {
                                <div class="map-error">
                                    <span class="material-symbols-outlined">"error"</span>
                                    <p>"Failed to load properties: " {e.to_string()}</p>
                                </div>
                            }.into_any(),
                            Ok(data) => {
                                let filtered = filter_pins(&data, l);
                                if filtered.is_empty() {
                                    view! {
                                        <div class="map-empty">
                                            <span class="material-symbols-outlined map-empty-icon">"location_off"</span>
                                            <p class="map-empty-title">"No pins in this layer"</p>
                                            <p class="map-empty-sub">
                                                "Try All, or add coordinates / open work orders on properties."
                                            </p>
                                        </div>
                                    }.into_any()
                                } else {
                                    let script = build_map_init_script(&filtered, l);
                                    view! {
                                        <div id="atlas-map" class="map-leaflet"></div>
                                        <leptos_meta::Script>{script}</leptos_meta::Script>
                                    }.into_any()
                                }
                            }
                        })}}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
