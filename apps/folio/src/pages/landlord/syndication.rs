// apps/folio/src/pages/landlord/syndication.rs
//
// Syndication — /l/syndication
//
// Manages which listing networks the landlord syndicates their properties to.
// Wraps the platform's multi-channel listing push system.
// Uses /api/folio/catalog for the listing source-of-truth.
// Syndication channel configs are tenant-level (operator-side) settings.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Static channel definitions ─────────────────────────────────────────────

struct Channel {
    id: &'static str,
    name: &'static str,
    icon: &'static str,
    category: &'static str,
    desc: &'static str,
}

fn all_channels() -> Vec<Channel> {
    vec![
        Channel {
            id: "zillow",
            name: "Zillow",
            icon: "🏡",
            category: "Major Portals",
            desc: "Largest US rental marketplace — 35M+ monthly users",
        },
        Channel {
            id: "apartments",
            name: "Apartments.com",
            icon: "🏢",
            category: "Major Portals",
            desc: "Top apartment search with CoStar backing",
        },
        Channel {
            id: "realtor",
            name: "Realtor.com",
            icon: "🔑",
            category: "Major Portals",
            desc: "NAR-affiliated portal for LTR and sale listings",
        },
        Channel {
            id: "facebook",
            name: "Facebook Marketplace",
            icon: "👥",
            category: "Social",
            desc: "Largest social real estate marketplace",
        },
        Channel {
            id: "craigslist",
            name: "Craigslist",
            icon: "📋",
            category: "Classifieds",
            desc: "High-intent free listings — strong local reach",
        },
        Channel {
            id: "hotpads",
            name: "HotPads",
            icon: "🗺",
            category: "Map Search",
            desc: "Map-first rental search, Zillow Group",
        },
        Channel {
            id: "trulia",
            name: "Trulia",
            icon: "🌐",
            category: "Map Search",
            desc: "Zillow Group portal — neighborhood-focused search",
        },
        Channel {
            id: "airbnb",
            name: "Airbnb",
            icon: "🏖",
            category: "STR Platforms",
            desc: "Global STR platform — 100M+ active guests",
        },
        Channel {
            id: "vrbo",
            name: "Vrbo",
            icon: "🏕",
            category: "STR Platforms",
            desc: "Expedia Group STR platform — family stays",
        },
        Channel {
            id: "bookingdotcom",
            name: "Booking.com",
            icon: "🌍",
            category: "STR Platforms",
            desc: "European leader, strong international demand",
        },
        Channel {
            id: "atlas_network",
            name: "Atlas Network",
            icon: "⚡",
            category: "Platform",
            desc: "Atlas Platform native listing network",
        },
    ]
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn LandlordSyndication() -> impl IntoView {
    // Local toggle state for each channel (in production persisted to backend)
    let enabled_channels: RwSignal<std::collections::HashSet<&'static str>> = RwSignal::new({
        let mut s = std::collections::HashSet::new();
        s.insert("atlas_network"); // Atlas always on
        s
    });

    let channels = all_channels();

    view! {
        <div class="main-area">

            <div class="page-header">
                <div>
                    <h1 class="page-title">"Syndication"</h1>
                    <p class="page-subtitle">"Choose which listing networks receive your active property inventory"</p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-primary btn-sm"
                        on:click=move |_| {
                            // In production: save channel selections to tenant settings
                        }
                    >
                        "Save Preferences"
                    </button>
                </div>
            </div>

            // ── Overview KPIs ──
            <div class="kpi-row" style="margin-bottom:1.25rem;">
                <div class="kpi-card">
                    <span class="kpi-label">"Active Channels"</span>
                    <span class="kpi-value" style="color:var(--green)">
                        {move || enabled_channels.get().len().to_string()}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Available Networks"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">
                        {channels.len().to_string()}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Est. Monthly Reach"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">"140M+"</span>
                </div>
            </div>

            // ── Atlas Network always-on notice ──
            <div class="syndic-notice">
                <span class="syndic-notice-icon">"⚡"</span>
                <span>"Atlas Network syndication is always active for your listings."</span>
            </div>

            // ── Channel groups ──
            {
                let mut categories: Vec<(&'static str, Vec<&Channel>)> = Vec::new();
                let channels_ref = all_channels();
                for ch in channels_ref.iter() {
                    if let Some(pos) = categories.iter().position(|(c, _)| *c == ch.category) {
                        categories[pos].1.push(ch);
                    } else {
                        categories.push((ch.category, vec![ch]));
                    }
                }
                let categories_static: Vec<_> = vec![
                    ("Major Portals",  vec!["zillow", "apartments", "realtor"]),
                    ("Social",         vec!["facebook"]),
                    ("Classifieds",    vec!["craigslist"]),
                    ("Map Search",     vec!["hotpads", "trulia"]),
                    ("STR Platforms",  vec!["airbnb", "vrbo", "bookingdotcom"]),
                    ("Platform",       vec!["atlas_network"]),
                ];

                let all_chans = all_channels();
                categories_static.into_iter().map(|(cat_name, ids)| {
                    let chans: Vec<_> = ids.into_iter().filter_map(|id| {
                        all_chans.iter().find(|c| c.id == id)
                    }).collect();

                    view! {
                        <div class="syndic-group">
                            <div class="syndic-group-title">{cat_name}</div>
                            <div class="syndic-channel-grid">
                                {chans.into_iter().map(|ch| {
                                    let ch_id    = ch.id;
                                    let ch_name  = ch.name;
                                    let ch_icon  = ch.icon;
                                    let ch_desc  = ch.desc;
                                    let is_atlas = ch_id == "atlas_network";

                                    view! {
                                        <div class=move || format!("syndic-card {}",
                                            if enabled_channels.get().contains(ch_id) { "syndic-card--on" } else { "" }
                                        )>
                                            <div class="syndic-card-top">
                                                <span class="syndic-card-icon">{ch_icon}</span>
                                                <div class="syndic-card-meta">
                                                    <div class="syndic-card-name">{ch_name}</div>
                                                    <div class="syndic-card-desc">{ch_desc}</div>
                                                </div>
                                            </div>
                                            <div class="syndic-card-toggle-row">
                                                <label class="syndic-toggle-wrap">
                                                    <input
                                                        type="checkbox"
                                                        class="syndic-toggle-input"
                                                        prop:checked=move || enabled_channels.get().contains(ch_id)
                                                        disabled=is_atlas
                                                        on:change=move |ev: web_sys::Event| {
                                                            let el = Some(event_target::<web_sys::HtmlInputElement>(&ev));
                                                            if let Some(el) = el {
                                                                enabled_channels.update(|s| {
                                                                    if el.checked() { s.insert(ch_id); }
                                                                    else { s.remove(ch_id); }
                                                                });
                                                            }
                                                        }
                                                    />
                                                    <span class="syndic-toggle-track"></span>
                                                </label>
                                                <span class="syndic-toggle-label">
                                                    {move || if enabled_channels.get().contains(ch_id) { "Active" } else { "Inactive" }}
                                                </span>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()
            }

        </div>
    }
}
