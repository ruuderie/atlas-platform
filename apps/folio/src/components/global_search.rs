//! Cmd+K global finder for landlord shell.

use leptos::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::components::nav::{FolioRoute, NavIcon};

#[derive(Clone, Debug, PartialEq, Eq)]
struct SearchHit {
    group: String,
    label: String,
    path: String,
    keywords: String,
}

fn static_index() -> Vec<SearchHit> {
    vec![
        hit("Navigation", "Dashboard", FolioRoute::LandlordDashboard.path(), "home overview"),
        hit("Navigation", "Assets", FolioRoute::LandlordAssets.path(), "properties units buildings"),
        hit("Navigation", "Leases", FolioRoute::LandlordLeases.path(), "contracts rent"),
        hit("Navigation", "Maintenance", FolioRoute::LandlordMaintenance.path(), "work orders tickets"),
        hit("Navigation", "Deals", FolioRoute::LandlordDeals.path(), "wholesale creative finance"),
        hit("Navigation", "Map", FolioRoute::LandlordMap.path(), "portfolio map"),
        hit("Navigation", "Messages", FolioRoute::LandlordCommunications.path(), "inbox chat"),
        hit("Navigation", "Billing", FolioRoute::LandlordBilling.path(), "rent invoices money"),
        hit("Actions", "New lease", FolioRoute::LandlordLeaseCreate.path(), "create lease add contract"),
        hit(
            "Actions",
            "Add property",
            FolioRoute::LandlordAssetsCreate.path(),
            "create asset add building",
        ),
        hit(
            "Actions",
            "New work order",
            FolioRoute::LandlordMaintenanceNew.path(),
            "create maintenance ticket",
        ),
        hit("Setup", "Setup", FolioRoute::LandlordSetup.path(), "settings config admin"),
        hit("Setup", "Network", FolioRoute::LandlordTeam.path(), "team invites"),
        hit("Setup", "Referrals", FolioRoute::LandlordReferrals.path(), "friends family"),
        hit(
            "Setup",
            "Notifications",
            FolioRoute::LandlordNotifications.path(),
            "alerts prefs",
        ),
        hit("Setup", "STR Compliance", FolioRoute::LandlordStrCompliance.path(), "permits"),
        hit("Setup", "Syndication", FolioRoute::LandlordSyndication.path(), "channels listings"),
        hit("Setup", "Violations", FolioRoute::LandlordViolations.path(), "compliance"),
        hit("Setup", "Catalog", FolioRoute::LandlordCatalog.path(), "pricebook"),
        hit("Setup", "Campaigns", FolioRoute::LandlordCampaigns.path(), "outreach marketing"),
        hit("Setup", "Leads", FolioRoute::LandlordLeads.path(), "prospects crm"),
        hit("Setup", "Reservations", FolioRoute::LandlordReservations.path(), "str stays"),
        hit("Setup", "Ledger", FolioRoute::LandlordLedger.path(), "charges audit"),
        hit("Setup", "Ratings", FolioRoute::LandlordRatings.path(), "scorecards"),
        hit("Setup", "Inspections", FolioRoute::LandlordInspections.path(), "schedule"),
        hit("Setup", "Building systems", FolioRoute::LandlordSystems.path(), "elevator hvac"),
        hit("Setup", "Digital vault", FolioRoute::LandlordVault.path(), "documents files"),
        hit("Setup", "Vendors", FolioRoute::LandlordVendors.path(), "contractors"),
        hit("Setup", "Marketplace", FolioRoute::LandlordMarketplace.path(), "contractors trades"),
        hit("Setup", "Buyers", FolioRoute::LandlordBuyers.path(), "disposition crm"),
        hit("Setup", "Analytics", FolioRoute::LandlordMeridian.path(), "meridian kpi"),
        hit(
            "Setup",
            "Account billing",
            FolioRoute::LandlordAccountBilling.path(),
            "saas subscription",
        ),
        hit("Setup", "App settings", FolioRoute::Settings.path(), "preferences"),
    ]
}

fn hit(group: &str, label: &str, path: &str, keywords: &str) -> SearchHit {
    SearchHit {
        group: group.to_string(),
        label: label.to_string(),
        path: path.to_string(),
        keywords: keywords.to_string(),
    }
}

fn go_to(path: &str) {
    if let Some(w) = web_sys::window() {
        let _ = w.location().set_href(path);
    }
}

#[derive(Clone, Debug, Deserialize)]
struct AssetHit {
    id: Uuid,
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct LeaseHit {
    id: Uuid,
    status: String,
}

#[derive(Clone, Debug, Deserialize)]
struct WoHit {
    id: Uuid,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    description: Option<String>,
    status: String,
}

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GlobalSearchLive, "/api")]
async fn global_search_live(
    query: String,
) -> Result<Vec<(String, String, String)>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let q = query.trim().to_lowercase();
    if q.len() < 2 {
        return Ok(vec![]);
    }
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;

    let mut out = Vec::new();

    if let Ok(assets) =
        crate::atlas_client::authenticated_get::<Vec<AssetHit>>("/api/folio/assets", &token, None)
            .await
    {
        for a in assets
            .into_iter()
            .filter(|a| a.name.to_lowercase().contains(&q))
            .take(8)
        {
            out.push((
                "Assets".into(),
                a.name,
                FolioRoute::LandlordAssetDetail
                    .path()
                    .replace(":id", &a.id.to_string()),
            ));
        }
    }

    if let Ok(leases) =
        crate::atlas_client::authenticated_get::<Vec<LeaseHit>>("/api/folio/leases", &token, None)
            .await
    {
        for l in leases
            .into_iter()
            .filter(|l| l.status.to_lowercase().contains(&q) || l.id.to_string().contains(&q))
            .take(6)
        {
            let short = l.id.to_string();
            let short = &short[..8.min(short.len())];
            out.push((
                "Leases".into(),
                format!("Lease {short} · {}", l.status),
                FolioRoute::LandlordLeaseDetail
                    .path()
                    .replace(":id", &l.id.to_string()),
            ));
        }
    }

    if let Ok(wos) = crate::atlas_client::authenticated_get::<Vec<WoHit>>(
        "/api/folio/maintenance",
        &token,
        None,
    )
    .await
    {
        for w in wos
            .into_iter()
            .filter(|w| {
                w.subject
                    .as_deref()
                    .or(w.description.as_deref())
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q)
                    || w.status.to_lowercase().contains(&q)
                    || w.id.to_string().contains(&q)
            })
            .take(6)
        {
            let id_s = w.id.to_string();
            let short = &id_s[..8.min(id_s.len())];
            let label = w
                .subject
                .or(w.description)
                .unwrap_or_else(|| format!("WO {short}"));
            out.push((
                "Work orders".into(),
                format!("{label} · {}", w.status),
                FolioRoute::LandlordMaintenanceDetail
                    .path()
                    .replace(":id", &w.id.to_string()),
            ));
        }
    }

    Ok(out)
}

#[component]
pub fn GlobalSearch() -> impl IntoView {
    let open = RwSignal::new(false);
    let query = RwSignal::new(String::new());
    let selected = RwSignal::new(0usize);
    let live = Resource::new(
        move || query.get(),
        |q| async move {
            if q.trim().len() < 2 {
                return Ok::<Vec<(String, String, String)>, ServerFnError>(vec![]);
            }
            global_search_live(q).await
        },
    );

    #[cfg(feature = "hydrate")]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast;
        static BOUND: AtomicBool = AtomicBool::new(false);
        Effect::new(move |_| {
            if BOUND.swap(true, Ordering::SeqCst) {
                return;
            }
            let open_c = open;
            let query_c = query;
            let selected_c = selected;
            let handler = Closure::wrap(Box::new(move |ev: web_sys::KeyboardEvent| {
                let meta = ev.meta_key() || ev.ctrl_key();
                if meta && ev.key().eq_ignore_ascii_case("k") {
                    ev.prevent_default();
                    let next = !open_c.get_untracked();
                    open_c.set(next);
                    if next {
                        query_c.set(String::new());
                        selected_c.set(0);
                    }
                }
                if ev.key() == "Escape" && open_c.get_untracked() {
                    open_c.set(false);
                }
            }) as Box<dyn FnMut(_)>);
            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "keydown",
                    handler.as_ref().unchecked_ref(),
                );
            }
            handler.forget();
        });
    }

    let filtered = Memo::new(move |_| {
        let q = query.get().to_lowercase();
        let mut rows: Vec<SearchHit> = static_index()
            .into_iter()
            .filter(|h| {
                q.is_empty()
                    || h.label.to_lowercase().contains(&q)
                    || h.keywords.contains(&q)
                    || h.path.to_lowercase().contains(&q)
            })
            .collect();
        if let Some(Ok(live_rows)) = live.get() {
            for (group, label, path) in live_rows {
                rows.push(SearchHit {
                    group,
                    label,
                    path,
                    keywords: String::new(),
                });
            }
        }
        rows.truncate(24);
        rows
    });

    view! {
        <button
            type="button"
            class="folio-search-trigger press"
            title="Search (⌘K)"
            on:click=move |_| {
                open.set(true);
                query.set(String::new());
                selected.set(0);
            }
        >
            <span class="material-symbols-outlined">{NavIcon::Search.as_str()}</span>
            <span class="folio-search-trigger__label">"Search"</span>
            <kbd class="folio-search-trigger__kbd">"⌘K"</kbd>
        </button>

        <Show when=move || open.get()>
            <div
                class="folio-search-backdrop"
                on:click=move |_| open.set(false)
            >
                <div
                    class="folio-search-panel"
                    on:click=|ev| ev.stop_propagation()
                >
                    <div class="folio-search-input-wrap">
                        <span class="material-symbols-outlined">{NavIcon::Search.as_str()}</span>
                        <input
                            class="folio-search-input"
                            type="search"
                            placeholder="Search people, units, work orders, settings…"
                            autofocus
                            prop:value=move || query.get()
                            on:input=move |e| {
                                query.set(event_target_value(&e));
                                selected.set(0);
                            }
                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                let rows = filtered.get();
                                match ev.key().as_str() {
                                    "ArrowDown" => {
                                        ev.prevent_default();
                                        if !rows.is_empty() {
                                            selected.update(|i| *i = (*i + 1) % rows.len());
                                        }
                                    }
                                    "ArrowUp" => {
                                        ev.prevent_default();
                                        if !rows.is_empty() {
                                            selected.update(|i| {
                                                *i = if *i == 0 { rows.len() - 1 } else { *i - 1 };
                                            });
                                        }
                                    }
                                    "Enter" => {
                                        ev.prevent_default();
                                        if let Some(hit) = rows.get(selected.get()) {
                                            let path = hit.path.clone();
                                            open.set(false);
                                            go_to(&path);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        />
                    </div>
                    <ul class="folio-search-results">
                        {move || {
                            let rows = filtered.get();
                            if rows.is_empty() {
                                return view! {
                                    <li class="folio-search-empty">"No matches"</li>
                                }.into_any();
                            }
                            let sel = selected.get();
                            rows.into_iter().enumerate().map(|(idx, hit)| {
                                let path = hit.path.clone();
                                let active = idx == sel;
                                view! {
                                    <li>
                                        <button
                                            type="button"
                                            class=if active {
                                                "folio-search-row folio-search-row--active"
                                            } else {
                                                "folio-search-row"
                                            }
                                            on:click=move |_| {
                                                open.set(false);
                                                go_to(&path);
                                            }
                                        >
                                            <span class="folio-search-row__group">{hit.group.clone()}</span>
                                            <span class="folio-search-row__label">{hit.label.clone()}</span>
                                        </button>
                                    </li>
                                }
                            }).collect_view().into_any()
                        }}
                    </ul>
                </div>
            </div>
        </Show>
    }
}
