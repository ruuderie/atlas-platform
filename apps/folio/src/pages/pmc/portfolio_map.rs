// apps/folio/src/pages/pmc/portfolio_map.rs
//
// PMC Portfolio Map — /pmc/map
//
// Interactive map view of all managed properties across the PMC's client book.
// Reuses the same visual pattern as the landlord MapPortfolio (/l/map) but
// scoped to the property-manager's client portfolio rather than direct ownership.
//
// Data: GET /api/folio/pmc/properties?view=map
//   Returns [{asset_id, address, city, lat, lng, status, tenant_count, open_wo_count}]
//
// The map itself is rendered as a card grid in lat/lng sorted order (Leaflet/map
// WASM is not yet bundled — the geo-grid is the interim display until the map
// tile integration is wired).
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmcMapProperty {
    pub asset_id:        String,
    pub address:         String,
    pub city:            String,
    pub state:           String,
    pub lat:             Option<f64>,
    pub lng:             Option<f64>,
    pub status:          String,      // "occupied" | "vacant" | "maintenance" | "notice"
    pub tenant_count:    u32,
    pub open_wo_count:   u32,
    pub owner_name:      Option<String>,
    pub property_type:   String,      // "residential" | "commercial" | "mixed"
    pub unit_count:      u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PmcMapSummary {
    pub properties:      Vec<PmcMapProperty>,
    pub total_units:     u32,
    pub occupied_units:  u32,
    pub vacant_units:    u32,
    pub total_open_wo:   u32,
}

// ── Server function ───────────────────────────────────────────────────────────

#[server(FetchPmcMapProperties, "/api")]
pub async fn fetch_pmc_map_properties() -> Result<PmcMapSummary, server_fn::error::ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use axum::http::HeaderMap;
        use leptos_axum::extract;
        let headers = extract::<HeaderMap>().await.unwrap_or_default();
        let token = headers
            .get("cookie")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            }))
            .ok_or_else(|| server_fn::error::ServerFnError::new("No session"))?;
        crate::atlas_client::authenticated_get::<PmcMapSummary>(
            "/api/folio/pmc/properties?view=map", &token, None,
        ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn status_color(status: &str) -> &'static str {
    match status {
        "occupied"    => "#4ade80",
        "vacant"      => "#fbbf24",
        "maintenance" => "#f87171",
        "notice"      => "#a78bfa",
        _             => "#94a3b8",
    }
}

fn status_label(status: &str) -> &'static str {
    match status {
        "occupied"    => "Occupied",
        "vacant"      => "Vacant",
        "maintenance" => "Maintenance",
        "notice"      => "Notice",
        _             => "Unknown",
    }
}

fn type_icon(property_type: &str) -> &'static str {
    match property_type {
        "commercial" => "🏢",
        "mixed"      => "🏙",
        _            => "🏘",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── status_color ──────────────────────────────────────────────────────────

    #[test]
    fn status_color_occupied() {
        assert_eq!(status_color("occupied"), "#4ade80");
    }

    #[test]
    fn status_color_vacant() {
        assert_eq!(status_color("vacant"), "#fbbf24");
    }

    #[test]
    fn status_color_maintenance() {
        assert_eq!(status_color("maintenance"), "#f87171");
    }

    #[test]
    fn status_color_notice() {
        assert_eq!(status_color("notice"), "#a78bfa");
    }

    #[test]
    fn status_color_unknown_falls_back() {
        assert_eq!(status_color("delinquent"), "#94a3b8");
        assert_eq!(status_color(""), "#94a3b8");
    }

    #[test]
    fn status_color_returns_valid_hex() {
        for s in &["occupied", "vacant", "maintenance", "notice", "unknown"] {
            let c = status_color(s);
            assert!(c.starts_with('#'), "expected hex color, got {c:?} for status {s:?}");
            assert_eq!(c.len(), 7, "expected 6-char hex color, got {c:?}");
        }
    }

    // ── status_label ──────────────────────────────────────────────────────────

    #[test]
    fn status_label_all_known() {
        assert_eq!(status_label("occupied"),    "Occupied");
        assert_eq!(status_label("vacant"),      "Vacant");
        assert_eq!(status_label("maintenance"), "Maintenance");
        assert_eq!(status_label("notice"),      "Notice");
    }

    #[test]
    fn status_label_unknown_falls_back() {
        assert_eq!(status_label("delinquent"), "Unknown");
        assert_eq!(status_label(""),           "Unknown");
    }

    #[test]
    fn status_label_is_title_case() {
        for s in &["occupied", "vacant", "maintenance", "notice"] {
            let label = status_label(s);
            let first = label.chars().next().unwrap();
            assert!(first.is_uppercase(), "label {label:?} should start with uppercase");
        }
    }

    // ── type_icon ─────────────────────────────────────────────────────────────

    #[test]
    fn type_icon_commercial() {
        assert_eq!(type_icon("commercial"), "🏢");
    }

    #[test]
    fn type_icon_mixed() {
        assert_eq!(type_icon("mixed"), "🏙");
    }

    #[test]
    fn type_icon_residential_fallback() {
        // Both explicit "residential" and any unknown type get the residential icon
        assert_eq!(type_icon("residential"), "🏘");
        assert_eq!(type_icon("industrial"),  "🏘");
        assert_eq!(type_icon(""),            "🏘");
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PmcPortfolioMap() -> impl IntoView {
    let filter_status = RwSignal::new("all".to_string());
    let search_city   = RwSignal::new(String::new());

    let map_res = Resource::new(|| (), |_| fetch_pmc_map_properties());

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Portfolio Map"</h1>
                    <p class="page-subtitle">"Geographic view of managed properties"</p>
                </div>
            </div>

            <Suspense fallback=|| view! { <div class="doc-empty">"Loading portfolio…"</div> }>
                {move || map_res.get().map(|res| {
                    match res {
                        Ok(summary) => {
                            let occ_pct = if summary.total_units > 0 {
                                (summary.occupied_units as f64 / summary.total_units as f64 * 100.0) as u32
                            } else { 0 };

                            view! {
                                <div>
                                    // ── KPI strip ──
                                    <div class="kpi-row" style="margin-bottom:1.25rem;">
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Total Properties"</span>
                                            <span class="kpi-value" style="color:var(--cobalt)">{summary.properties.len().to_string()}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Occupancy"</span>
                                            <span class="kpi-value" style="color:#4ade80">{format!("{occ_pct}%")}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Vacant Units"</span>
                                            <span class="kpi-value" style="color:#fbbf24">{summary.vacant_units.to_string()}</span>
                                        </div>
                                        <div class="kpi-card">
                                            <span class="kpi-label">"Open WOs"</span>
                                            <span class="kpi-value" style="color:#f87171">{summary.total_open_wo.to_string()}</span>
                                        </div>
                                    </div>

                                    // ── Filter bar ──
                                    <div class="pmc-map-filters">
                                        <input
                                            type="text"
                                            class="form-input"
                                            style="max-width:14rem;"
                                            placeholder="Filter by city…"
                                            prop:value=move || search_city.get()
                                            on:input=move |ev| search_city.set(event_target_value(&ev))
                                        />
                                        <div class="pmc-map-status-filters">
                                            {["all", "occupied", "vacant", "maintenance", "notice"].iter().map(|s| {
                                                let s = *s;
                                                let color = if s == "all" { "#94a3b8" } else { status_color(s) };
                                                view! {
                                                    <button
                                                        class=move || format!("pmc-map-status-btn {}", if filter_status.get() == s {"pmc-map-status-btn--active"} else {""})
                                                        style=format!("--status-color:{color};")
                                                        on:click=move |_| filter_status.set(s.to_string())
                                                    >
                                                        {if s != "all" { view! { <span class="pmc-map-dot" style=format!("background:{color};")></span> }.into_any() } else { ().into_any() }}
                                                        {if s == "all" { "All" } else { status_label(s) }}
                                                    </button>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>

                                    // ── Map placeholder + grid ──
                                    <div class="pmc-map-container">
                                        <div class="pmc-map-tile-placeholder">
                                            <span class="pmc-map-tile-icon">"🗺"</span>
                                            <span class="pmc-map-tile-label">"Interactive map rendering requires geo tile integration"</span>
                                            <span class="text-xs text-on-surface-variant">"Showing property grid below sorted by location"</span>
                                        </div>
                                    </div>

                                    // ── Property grid ──
                                    <div class="owner-section">
                                        <div class="owner-section-title">"Properties"
                                            <span class="text-xs text-on-surface-variant" style="font-weight:400;margin-left:.5rem;">
                                                {move || {
                                                    let city_q = search_city.get().to_lowercase();
                                                    let status_q = filter_status.get();
                                                    let count = summary.properties.iter().filter(|p| {
                                                        (city_q.is_empty() || p.city.to_lowercase().contains(&city_q)) &&
                                                        (status_q == "all" || p.status == status_q)
                                                    }).count();
                                                    format!("({count} shown)")
                                                }}
                                            </span>
                                        </div>
                                        <div class="pmc-map-grid">
                                            {move || {
                                                let city_q = search_city.get().to_lowercase();
                                                let status_q = filter_status.get();
                                                summary.properties.iter()
                                                    .filter(|p| {
                                                        (city_q.is_empty() || p.city.to_lowercase().contains(&city_q)) &&
                                                        (status_q == "all" || p.status == status_q)
                                                    })
                                                    .map(|prop| {
                                                        let color = status_color(&prop.status);
                                                        let label = status_label(&prop.status);
                                                        let icon  = type_icon(&prop.property_type);
                                                        let city  = format!("{}, {}", prop.city, prop.state);
                                                        let addr  = prop.address.clone();
                                                        let owner = prop.owner_name.clone().unwrap_or_else(|| "—".to_string());
                                                        let units = prop.unit_count;
                                                        let tenants = prop.tenant_count;
                                                        let wos  = prop.open_wo_count;
                                                        let asset_href = format!("/pmc/clients/{}", prop.asset_id);
                                                        view! {
                                                            <div class="pmc-map-card">
                                                                <div class="pmc-map-card-header">
                                                                    <span class="pmc-map-card-icon">{icon}</span>
                                                                    <div class="pmc-map-card-addr">
                                                                        <div class="pmc-map-card-street">{addr}</div>
                                                                        <div class="pmc-map-card-city text-xs text-on-surface-variant">{city}</div>
                                                                    </div>
                                                                    <span class="ph-badge" style=format!("background:rgba(255,255,255,.06);color:{color};border:1px solid {color}30;font-size:.68rem;")>
                                                                        {label}
                                                                    </span>
                                                                </div>
                                                                <div class="pmc-map-card-meta">
                                                                    <span class="pmc-map-meta-chip">
                                                                        <span class="pmc-meta-icon">"👤"</span>{owner}
                                                                    </span>
                                                                    <span class="pmc-map-meta-chip">
                                                                        <span class="pmc-meta-icon">"🏠"</span>{units.to_string()} " units"
                                                                    </span>
                                                                    <span class="pmc-map-meta-chip">
                                                                        <span class="pmc-meta-icon">"👥"</span>{tenants.to_string()} " tenants"
                                                                    </span>
                                                                    {if wos > 0 {
                                                                        view! {
                                                                            <span class="pmc-map-meta-chip" style="color:#f87171;">
                                                                                <span class="pmc-meta-icon">"🔧"</span>{wos.to_string()} " open"
                                                                            </span>
                                                                        }.into_any()
                                                                    } else { ().into_any() }}
                                                                </div>
                                                                <div style="margin-top:.5rem;">
                                                                    <a href=asset_href class="btn btn-ghost btn-sm">"View Property →"</a>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect::<Vec<_>>()
                                            }}
                                        </div>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Err(_) => view! {
                            <div class="doc-empty">
                                <div>"Could not load portfolio map data. Please try again."</div>
                            </div>
                        }.into_any(),
                    }
                })}
            </Suspense>
        </div>
    }
}
