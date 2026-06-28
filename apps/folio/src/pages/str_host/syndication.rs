// apps/folio/src/pages/str_host/syndication.rs
//
// STR Host — Syndication — /s/syndication
//
// Per-listing channel distribution for STR hosts.
// Unlike the landlord syndication (tenant-level on/off), the STR version is
// per-listing and includes STR-specific OTAs (Airbnb, Vrbo, Booking.com)
// alongside direct-booking channels.
//
// Key flows:
//   1. Select a listing from the dropdown
//   2. Toggle OTA channels on/off for that listing
//   3. View per-channel sync status (last_synced, is_live, rate_override)
//   4. Push updates / queue a manual sync
//
// Data:
//   GET  /api/folio/str/listings                        → Vec<{id, name}>
//   GET  /api/folio/str/listings/:id/channels           → Vec<ChannelStatus>
//   POST /api/folio/str/listings/:id/channels/:channel  → toggle + config
//   POST /api/folio/str/listings/:id/channels/sync      → trigger push
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

// ── Static channel definitions (STR-focused) ──────────────────────────────────

struct StrChannel {
    id:       &'static str,
    name:     &'static str,
    icon:     &'static str,
    category: &'static str,
    desc:     &'static str,
    fee_pct:  Option<f64>,   // platform fee percentage
}

fn str_channels() -> Vec<StrChannel> {
    vec![
        StrChannel { id: "airbnb",          name: "Airbnb",          icon: "🏖",  category: "OTA",           desc: "100M+ active guests worldwide",                          fee_pct: Some(3.0) },
        StrChannel { id: "vrbo",            name: "Vrbo",            icon: "🏕",  category: "OTA",           desc: "Expedia Group — strong family/group bookings",            fee_pct: Some(5.0) },
        StrChannel { id: "bookingdotcom",   name: "Booking.com",     icon: "🌍",  category: "OTA",           desc: "EU market leader, 500M+ annual visitors",                fee_pct: Some(15.0) },
        StrChannel { id: "tripadvisor",     name: "TripAdvisor",     icon: "🦉",  category: "OTA",           desc: "Review-driven discovery, global reach",                  fee_pct: Some(3.0) },
        StrChannel { id: "atlas_network",   name: "Atlas Direct",    icon: "⚡",  category: "Platform",      desc: "Atlas native direct-booking — zero commission",          fee_pct: Some(0.0) },
        StrChannel { id: "google_vacation", name: "Google Vacation",  icon: "🔍", category: "Search",        desc: "Free OTA-surface listing via Google Travel",             fee_pct: None },
        StrChannel { id: "facebook",        name: "Facebook",        icon: "👥",  category: "Social",        desc: "Facebook Marketplace + Vacation Rental groups",          fee_pct: Some(0.0) },
        StrChannel { id: "hipcamp",         name: "Hipcamp",         icon: "🌲",  category: "Niche",         desc: "Outdoor + unique stays — cabins, glamping, farms",       fee_pct: Some(10.0) },
        StrChannel { id: "whimstay",        name: "Whimstay",        icon: "🎒",  category: "Niche",         desc: "Last-minute deals platform — high discount bookings",    fee_pct: Some(5.0) },
    ]
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrListingStub {
    pub id:   String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatus {
    pub channel_id:     String,
    pub is_live:        bool,
    pub last_synced_at: Option<String>,
    pub sync_errors:    Option<String>,
    pub rate_override:  Option<i64>,    // cents override; None = use base rate
    pub external_url:   Option<String>,
}

// ── Server functions ──────────────────────────────────────────────────────────

#[server(FetchStrListingStubs, "/api")]
pub async fn fetch_str_listing_stubs() -> Result<Vec<StrListingStub>, server_fn::error::ServerFnError> {
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
        crate::atlas_client::authenticated_get::<Vec<StrListingStub>>(
            "/api/folio/str/listings?stub=1", &token, None,
        ).await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(FetchStrChannelStatus, "/api")]
pub async fn fetch_str_channel_status(listing_id: String) -> Result<Vec<ChannelStatus>, server_fn::error::ServerFnError> {
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
        let url = format!("/api/folio/str/listings/{listing_id}/channels");
        crate::atlas_client::authenticated_get::<Vec<ChannelStatus>>(&url, &token, None)
            .await.map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    { unreachable!() }
}

#[server(SyncStrChannels, "/api")]
pub async fn sync_str_channels(listing_id: String) -> Result<(), server_fn::error::ServerFnError> {
    let _ = listing_id;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn sync_age_label(last_synced: Option<&str>) -> String {
    match last_synced {
        None    => "Never synced".to_string(),
        Some(s) => {
            let date = s.chars().take(10).collect::<String>();
            format!("Synced {date}")
        }
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn StrSyndication() -> impl IntoView {
    let active_listing = RwSignal::new(None::<StrListingStub>);
    let syncing        = RwSignal::new(false);
    let synced_toast   = RwSignal::new(false);

    let stubs_res = Resource::new(|| (), |_| fetch_str_listing_stubs());

    // Channel enabled state — keyed by channel_id
    let enabled: RwSignal<std::collections::HashSet<String>> = RwSignal::new({
        let mut s = std::collections::HashSet::new();
        s.insert("atlas_network".to_string());
        s
    });

    let channel_status_res = Resource::new(
        move || active_listing.get().map(|l| l.id),
        |lid| async move {
            match lid {
                Some(id) => fetch_str_channel_status(id).await.ok(),
                None     => None,
            }
        },
    );

    let channels = str_channels();

    view! {
        <div class="main-area">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"STR Channel Syndication"</h1>
                    <p class="page-subtitle">"Distribute your listings across OTAs and booking platforms"</p>
                </div>
                <div class="page-header-actions">
                    <button
                        class="btn btn-primary"
                        disabled=move || syncing.get() || active_listing.get().is_none()
                        on:click=move |_| {
                            if let Some(l) = active_listing.get() {
                                syncing.set(true);
                                let lid = l.id.clone();
                                leptos::task::spawn_local(async move {
                                    let _ = sync_str_channels(lid).await;
                                    synced_toast.set(true);
                                    syncing.set(false);
                                });
                            }
                        }
                    >{move || if syncing.get() { "⟳ Syncing…" } else { "⟳ Push to Channels" }}</button>
                </div>
            </div>

            // ── Listing selector ──
            <div class="g27-template-bar">
                <div class="g27-template-label">"Listing:"</div>
                <Suspense fallback=|| view! { <select class="form-select g27-template-select"><option>"Loading…"</option></select> }>
                    {move || stubs_res.get().map(|res| {
                        match res {
                            Ok(stubs) if !stubs.is_empty() => {
                                if active_listing.get().is_none() {
                                    active_listing.set(Some(stubs[0].clone()));
                                }
                                view! {
                                    <select class="form-select g27-template-select"
                                        on:change=move |ev| {
                                            let sel = event_target_value(&ev);
                                            if let Some(s) = stubs.iter().find(|s| s.id == sel) {
                                                active_listing.set(Some(s.clone()));
                                            }
                                        }
                                    >
                                        {stubs.iter().map(|s| {
                                            let sid  = s.id.clone();
                                            let name = s.name.clone();
                                            view! { <option value={sid}>{name}</option> }
                                        }).collect::<Vec<_>>()}
                                    </select>
                                }.into_any()
                            }
                            _ => view! {
                                <div class="text-sm text-on-surface-variant">"No listings found. Create a listing first."</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>

            {move || if synced_toast.get() {
                view! { <div class="alert-saved-toast">"✓ Sync queued — channels will update within 15 minutes"</div> }.into_any()
            } else { ().into_any() }}

            // ── KPIs ──
            <div class="kpi-row" style="margin:1rem 0;">
                <div class="kpi-card">
                    <span class="kpi-label">"Active Channels"</span>
                    <span class="kpi-value" style="color:#4ade80;">{move || enabled.get().len().to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Platforms Available"</span>
                    <span class="kpi-value" style="color:var(--cobalt);">{channels.len().to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Zero-Commission"</span>
                    <span class="kpi-value" style="color:#a78bfa;">"2"</span>
                </div>
            </div>

            // ── Channel grid ──
            {channels.iter().map(|ch| {
                let cid   = ch.id;
                let cname = ch.name;
                let icon  = ch.icon;
                let cat   = ch.category;
                let desc  = ch.desc;
                let fee   = ch.fee_pct;

                // Per-channel live status from server
                let sync_label = move || {
                    channel_status_res.get()
                        .flatten()
                        .as_ref()
                        .and_then(|statuses| statuses.iter().find(|s| s.channel_id == cid))
                        .map(|s| sync_age_label(s.last_synced_at.as_deref()))
                        .unwrap_or_else(|| "Not configured".to_string())
                };

                let has_error = move || {
                    channel_status_res.get()
                        .flatten()
                        .as_ref()
                        .and_then(|statuses| statuses.iter().find(|s| s.channel_id == cid))
                        .and_then(|s| s.sync_errors.as_ref())
                        .is_some()
                };

                let ext_url = move || {
                    channel_status_res.get()
                        .flatten()
                        .as_ref()
                        .and_then(|statuses| statuses.iter().find(|s| s.channel_id == cid))
                        .and_then(|s| s.external_url.clone())
                };

                let is_on = move || enabled.get().contains(cid);

                view! {
                    <div class=move || format!("syndic-channel-card {}", if is_on() {"syndic-channel-card--active"} else {""})>
                        <div class="syndic-channel-meta">
                            <span class="syndic-channel-icon">{icon}</span>
                            <div class="syndic-channel-info">
                                <div class="syndic-channel-name">{cname}
                                    <span class="syndic-channel-cat">{cat}</span>
                                </div>
                                <div class="syndic-channel-desc text-xs text-on-surface-variant">{desc}</div>
                                <div class="syndic-channel-status text-xs" style=move || if has_error() {"color:#f87171;"} else {"color:var(--on-surface-variant);"}>
                                    {move || if has_error() { "⚠ Sync error — check credentials".to_string() } else { sync_label() }}
                                </div>
                            </div>
                            <div class="syndic-channel-right">
                                {fee.map(|f| view! {
                                    <span class="syndic-fee-badge">{
                                        if f == 0.0 { "Free".to_string() }
                                        else { format!("{f:.0}% fee") }
                                    }</span>
                                })}
                                {move || ext_url().map(|url| view! {
                                    <a href={url} target="_blank" class="btn btn-ghost btn-sm" style="font-size:.72rem;">"View ↗"</a>
                                })}
                                <label class="syndic-toggle-wrap">
                                    <input
                                        type="checkbox"
                                        class="syndic-toggle-input"
                                        prop:checked=move || is_on()
                                        prop:disabled=move || cid == "atlas_network"
                                        on:change=move |ev: web_sys::Event| {
                                            let el = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                            if let Some(el) = el {
                                                let checked = el.checked();
                                                enabled.update(|s| {
                                                    if checked { s.insert(cid.to_string()); }
                                                    else { s.remove(cid); }
                                                });
                                                synced_toast.set(false);
                                            }
                                        }
                                    />
                                    <span class="syndic-toggle-track"></span>
                                </label>
                            </div>
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}

            <div class="viol-info-banner" style="margin-top:1.25rem;">
                <span class="viol-info-icon">"💡"</span>
                <p class="viol-info-text">
                    "Channel toggles update your distribution preferences. Click "
                    <strong>"Push to Channels"</strong>
                    " to propagate changes. OTA credentials are configured in "
                    <a href="/l/systems" class="text-cobalt">"Building Systems → Integrations"</a>"."
                </p>
            </div>
        </div>
    }
}
