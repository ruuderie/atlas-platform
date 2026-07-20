//! Ops map — `/l/map`
//!
//! Leaflet pins for geo-coded assets with maintenance / STR layers.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
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
    #[serde(default)]
    pub has_occupying_lease: bool,
}

impl MapPin {
    fn address_primary(&self) -> String {
        let street = self.address_line_1.as_deref().unwrap_or("").trim();
        if !street.is_empty() {
            return street.to_string();
        }
        self.name.clone()
    }

    fn address_secondary(&self) -> String {
        let street = self.address_line_1.as_deref().unwrap_or("").trim();
        let city_line = [
            self.city.as_deref().unwrap_or(""),
            self.state_province.as_deref().unwrap_or(""),
        ]
        .iter()
        .copied()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
        if !street.is_empty() && street != self.name {
            if city_line.is_empty() {
                self.name.clone()
            } else {
                format!("{} · {city_line}", self.name)
            }
        } else {
            city_line
        }
    }
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
        MapLayer::Str if p.str_eligible => "#6366f1",
        _ => {
            if p.has_occupying_lease {
                "#2563eb"
            } else {
                "#fb923c"
            }
        }
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

fn build_map_init_script(pins: &[MapPin], layer: MapLayer, focus_asset_id: Option<Uuid>) -> String {
    let pins_json = pins
        .iter()
        .map(|p| {
            let primary = p.address_primary();
            let secondary = p.address_secondary();
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
                r#"{{"id":{id},"lat":{lat},"lng":{lng},"primary":{primary},"secondary":{secondary},"occupied":{occ},"color":{color},"href":{href},"woNew":{wo_new},"woCount":{wo},"nextWo":{next},"str":{str_el}}}"#,
                id = serde_json::to_string(&p.id.to_string()).unwrap_or_default(),
                lat = p.latitude,
                lng = p.longitude,
                primary = serde_json::to_string(&primary).unwrap_or_default(),
                secondary = serde_json::to_string(&secondary).unwrap_or_default(),
                occ = if p.has_occupying_lease {
                    "true"
                } else {
                    "false"
                },
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

    let focused = focus_asset_id.and_then(|fid| pins.iter().find(|p| p.id == fid));
    let (center_lat, center_lng, zoom) = if let Some(p) = focused {
        (p.latitude, p.longitude, 15)
    } else if pins.is_empty() {
        (39.5, -98.35, 4)
    } else if pins.len() == 1 {
        (pins[0].latitude, pins[0].longitude, 14)
    } else {
        let lat = pins.iter().map(|p| p.latitude).sum::<f64>() / pins.len() as f64;
        let lng = pins.iter().map(|p| p.longitude).sum::<f64>() / pins.len() as f64;
        (lat, lng, 10)
    };
    let focus_id_js = focus_asset_id
        .map(|id| serde_json::to_string(&id.to_string()).unwrap_or_else(|_| "null".into()))
        .unwrap_or_else(|| "null".into());

    format!(
        r#"
(function() {{
  window.__atlasMapInit = false;

  var pins = [{pins_json}];
  var focusId = {focus_id_js};

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

    L.tileLayer('https://{{s}}.basemaps.cartocdn.com/light_all/{{z}}/{{x}}/{{y}}{{r}}.png', {{
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a> &copy; <a href="https://carto.com/">CARTO</a>',
      subdomains: 'abcd',
      maxZoom: 19
    }}).addTo(map);

    var focusMarker = null;
    pins.forEach(function(p) {{
      var badge = p.woCount > 0
        ? '<span class="atlas-map-pin-badge">' + p.woCount + '</span>'
        : '';
      var icon = L.divIcon({{
        className: 'atlas-map-pin',
        html: '<div class="atlas-map-pin__dot" style="background:' + p.color + ';">' + badge + '</div>',
        iconSize: [16, 16],
        iconAnchor: [8, 8],
      }});
      var statusLabel = p.occupied ? 'Occupied' : 'Vacant';
      var woLine = p.woCount > 0
        ? '<div class="atlas-map-popup__wo">' + p.woCount + ' open WO' + (p.nextWo ? ' · next ' + p.nextWo : '') + '</div>'
        : '';
      var popup = '<div class="atlas-map-popup__inner">' +
        '<div class="atlas-map-popup__title">' + p.primary + '</div>' +
        (p.secondary ? '<div class="atlas-map-popup__sub">' + p.secondary + '</div>' : '') +
        '<span class="atlas-map-popup__pill" style="color:' + p.color + ';">' + statusLabel + '</span>' +
        (p.str ? ' <span class="atlas-map-popup__pill atlas-map-popup__pill--str">STR</span>' : '') +
        woLine +
        '<div class="atlas-map-popup__actions">' +
        '<a href="' + p.href + '">Open building</a>' +
        '<a href="' + p.woNew + '">Create WO</a>' +
        '</div></div>';
      var marker = L.marker([p.lat, p.lng], {{ icon: icon }})
        .addTo(map)
        .bindPopup(popup, {{ className: 'atlas-map-popup' }});
      if (focusId && p.id === focusId) {{
        focusMarker = marker;
      }}
    }});

    if (focusMarker) {{
      focusMarker.openPopup();
    }} else if (pins.length > 1) {{
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
        focus_id_js = focus_id_js,
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
    let vacant = pins.iter().filter(|p| !p.has_occupying_lease).count();

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
            <div class="map-kpi-card map-kpi-card--str">
                <span class="map-kpi-icon material-symbols-outlined">"beach_access"</span>
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
fn MapLegend() -> impl IntoView {
    view! {
        <div class="map-legend" aria-label="Map legend">
            <span class="map-legend__item">
                <span class="map-legend__dot" style="background:#2563eb;"></span>
                "Occupied"
            </span>
            <span class="map-legend__item">
                <span class="map-legend__dot" style="background:#fb923c;"></span>
                "Vacant"
            </span>
            <span class="map-legend__item">
                <span class="map-legend__dot" style="background:#ef4444;"></span>
                "Open WO"
            </span>
            <span class="map-legend__item">
                <span class="map-legend__dot" style="background:#6366f1;"></span>
                "STR"
            </span>
        </div>
    }
}

#[component]
fn MapPinList(pins: Vec<MapPin>) -> impl IntoView {
    view! {
        <div class="map-pin-list">
            <div class="map-pin-list-header">"Properties"</div>
            {pins.into_iter().map(|p| {
                let primary = p.address_primary();
                let secondary = p.address_secondary();
                let color = pin_color(&p, MapLayer::All).to_string();
                let href = FolioRoute::LandlordAssetDetail
                    .path()
                    .replace(":id", &p.id.to_string());
                let wo = p.open_wo_count;
                let status = if p.has_occupying_lease {
                    "Occupied".to_string()
                } else {
                    "Vacant".to_string()
                };
                view! {
                    <a class="map-pin-item press" href=href style="text-decoration:none;color:inherit;">
                        <div class="map-pin-dot" style=format!("background:{color};")></div>
                        <div class="map-pin-info">
                            <span class="map-pin-name">{primary}</span>
                            <span class="map-pin-addr">{secondary}</span>
                        </div>
                        <span class="map-pin-status" style=format!("color:{color};")>
                            {if wo > 0 { format!("{wo} WO") } else { status }}
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
    let focus_asset_id = Memo::new(move |_| {
        query
            .get()
            .get("asset_id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });

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

        <div class="map-page landlord-list-page map-page--light">
            <PageHeader
                title=Signal::derive(|| "Ops map".to_string())
                subtitle=Signal::derive(|| {
                    "Properties, maintenance, and STR — filter layers below".to_string()
                })
            />

            <div class="folio-tab-bar" role="tablist" aria-label="Map layers">
                {[MapLayer::All, MapLayer::Maintenance, MapLayer::Str, MapLayer::Status]
                    .into_iter()
                    .map(|l| {
                        view! {
                            <button
                                type="button"
                                class=move || {
                                    if layer.get() == l {
                                        "folio-tab folio-tab--active"
                                    } else {
                                        "folio-tab"
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
                    // KPIs always from full pin set (not layer-filtered).
                    Ok(data) => view! { <MapKpiStrip pins=data /> }.into_any(),
                    Err(_) => view! { <div></div> }.into_any(),
                })}
            </Suspense>

            <MapLegend/>

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

                <div class="map-container map-container--light">
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
                                    let script =
                                        build_map_init_script(&filtered, l, focus_asset_id.get());
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
