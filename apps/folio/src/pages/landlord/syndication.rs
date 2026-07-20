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

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::pages::landlord::catalog::list_catalog_entries;

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
    // Read-only channel catalog — no landlord-facing channel-prefs persist API yet.
    let catalog = Resource::new(|| (), |_| async move { list_catalog_entries().await });
    let channels = all_channels();

    view! {
        <div class="main-area">

            <PageHeader
                title=Signal::derive(|| "Syndication".to_string())
                subtitle=Signal::derive(|| {
                    "Listing inventory and channel destinations (read-only)".to_string()
                })
            >
                <a class="folio-btn folio-btn--primary folio-btn--sm" href=FolioRoute::LandlordCatalog.path()>"Open catalog"</a>
            </PageHeader>

            <div class="kpi-row" style="margin-bottom:1.25rem;">
                <div class="kpi-card">
                    <span class="kpi-label">"Active channel"</span>
                    <span class="kpi-value" style="color:var(--green)">"1"</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Available Networks"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">
                        {channels.len().to_string()}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Catalog listings"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">
                        {move || catalog
                            .get()
                            .and_then(|r| r.ok())
                            .map(|items| items.len().to_string())
                            .unwrap_or_else(|| "—".into())}
                    </span>
                </div>
            </div>

            <div class="syndic-notice">
                <span class="syndic-notice-icon">"⚡"</span>
                <span>"Atlas Network is active for catalog listings. Other channels are shown for reference — channel preferences are not configurable here yet. Manage listings in Catalog."</span>
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
                                        <div class=format!("syndic-card {}", if is_atlas { "syndic-card--on" } else { "" })>
                                            <div class="syndic-card-top">
                                                <span class="syndic-card-icon">{ch_icon}</span>
                                                <div class="syndic-card-meta">
                                                    <div class="syndic-card-name">{ch_name}</div>
                                                    <div class="syndic-card-desc">{ch_desc}</div>
                                                </div>
                                            </div>
                                            <div class="syndic-card-toggle-row">
                                                <span class="syndic-toggle-label">
                                                    {if is_atlas { "Active" } else { "Not configured" }}
                                                </span>
                                                <a
                                                    class="folio-btn folio-btn--ghost folio-btn--sm"
                                                    href=FolioRoute::LandlordCatalog.path()
                                                >
                                                    "Catalog"
                                                </a>
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
