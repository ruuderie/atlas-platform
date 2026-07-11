// apps/folio/src/pages/marketing/cohost_marketplace.rs
//
// CohostMarketplace — /cohost-market
//
// Public two-sided marketplace for co-hosting:
//
//   - Owners tab:   Browse verified co-hosts (reputation, occupancy, specialties)
//   - Co-hosts tab: Browse properties listed by owners seeking a co-host
//
// Zero-auth (no session required). Designed to surface as a discovery landing
// page from the Folio marketing homepage "Cohost Network → Coming Soon" teaser
// card, and linked from the platform-admin products landing pages panel.
//
// Back-link pattern: shared MarketingNav logo → /  (Folio marketing homepage).
//
// Data model:
//   The first cut uses seeded mock data (same pattern as ltr_listings.rs before
//   a real backend endpoint exists). When the `str_cohost` RBAC role + the
//   atlas_cohost_profiles table are shipped (Phase 1 of cohosting_gap_analysis),
//   replace `seeded_cohosts()` / `seeded_properties()` with a `#[server]` fn
//   calling `/api/pub/cohost/marketplace`.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos_meta::{Meta, Title};

use crate::components::marketing_nav::{MarketingNav, MarketingNavRole};

// ── Domain types ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct CohostProfile {
    pub initials: &'static str,
    pub name: &'static str,
    pub city: &'static str,
    pub trust_score: f32,
    pub guest_rating: f32,
    pub response_rate: u8,
    pub avg_occupancy: u8,
    pub total_stays: u32,
    pub typical_split: &'static str, // e.g. "18–22%"
    pub availability: u8,            // max units they can take on
    pub superhost: bool,
    pub specialties: Vec<&'static str>,
    pub bio_short: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OwnerProperty {
    pub initials: &'static str,
    pub owner_name: &'static str,
    pub city: &'static str,
    pub beds: u8,
    pub baths: f32,
    pub property_type: &'static str, // "Beachfront" | "Condo" | "Townhouse"
    pub icon: &'static str,          // material symbol
    pub seeking: &'static str,       // short note on what they need
    pub posted_ago: &'static str,    // "3 days ago"
}

// ── Seed data (static mock — swap for #[server] fn when API ships) ────────────

fn seeded_cohosts() -> Vec<CohostProfile> {
    vec![
        CohostProfile {
            initials: "MR", name: "Marcus Rivera", city: "Miami Beach, FL",
            trust_score: 9.4, guest_rating: 4.9, response_rate: 99,
            avg_occupancy: 84, total_stays: 87, typical_split: "18–22%",
            availability: 2, superhost: true,
            specialties: vec!["Luxury STR", "Beachfront", "OTA Channels", "Guest Vetting", "Turnover Mgmt"],
            bio_short: "Miami-based co-host with 4+ years managing beachfront STRs on Airbnb, VRBO, and Booking.com.",
        },
        CohostProfile {
            initials: "AS", name: "Alicia Santos", city: "Austin, TX",
            trust_score: 8.9, guest_rating: 4.87, response_rate: 97,
            avg_occupancy: 79, total_stays: 54, typical_split: "15–20%",
            availability: 3, superhost: true,
            specialties: vec!["Urban STR", "Dynamic Pricing", "Smart Home", "Professional Photos"],
            bio_short: "Austin co-host specialising in tech-enabled properties near the convention centre and university district.",
        },
        CohostProfile {
            initials: "DK", name: "David Kim", city: "Asheville, NC",
            trust_score: 9.1, guest_rating: 4.93, response_rate: 100,
            avg_occupancy: 88, total_stays: 112, typical_split: "20–25%",
            availability: 1, superhost: true,
            specialties: vec!["Mountain Cabins", "Nature Retreats", "STR Licensing", "Guest Comms"],
            bio_short: "Mountain cabin specialist with 112 completed stays and zero guest disputes. Only 1 unit slot remaining.",
        },
        CohostProfile {
            initials: "JP", name: "Jamie Park", city: "Nashville, TN",
            trust_score: 7.8, guest_rating: 4.72, response_rate: 95,
            avg_occupancy: 71, total_stays: 28, typical_split: "15–18%",
            availability: 4, superhost: false,
            specialties: vec!["Music District", "Event Weekends", "Pricing Strategy"],
            bio_short: "Rising Nashville co-host with a focus on weekend event demand and competitive dynamic pricing.",
        },
        CohostProfile {
            initials: "LN", name: "Laura Nguyen", city: "San Diego, CA",
            trust_score: 8.6, guest_rating: 4.85, response_rate: 98,
            avg_occupancy: 81, total_stays: 63, typical_split: "18–23%",
            availability: 2, superhost: true,
            specialties: vec!["Coastal STR", "Pet-Friendly", "Military Stays", "CA Compliance"],
            bio_short: "San Diego co-host known for pet-friendly listings and navigating California's evolving STR regulations.",
        },
        CohostProfile {
            initials: "BO", name: "Ben Okafor", city: "Atlanta, GA",
            trust_score: 8.3, guest_rating: 4.81, response_rate: 96,
            avg_occupancy: 76, total_stays: 41, typical_split: "17–21%",
            availability: 3, superhost: false,
            specialties: vec!["Urban Core", "Corporate Stays", "Keyless Entry", "Cleaning Network"],
            bio_short: "Atlanta-based co-host with a vetted cleaning crew and strong corporate extended-stay relationships.",
        },
    ]
}

fn seeded_properties() -> Vec<OwnerProperty> {
    vec![
        OwnerProperty {
            initials: "CW", owner_name: "Christine Walsh", city: "Miami Beach, FL",
            beds: 3, baths: 2.0, property_type: "Beachfront",
            icon: "beach_access",
            seeking: "Looking for a full-service co-host to handle everything from listing to cleanout.",
            posted_ago: "2 days ago",
        },
        OwnerProperty {
            initials: "TM", owner_name: "Tom Marcello", city: "Scottsdale, AZ",
            beds: 4, baths: 3.0, property_type: "Pool Villa",
            icon: "pool",
            seeking: "Need a co-host who can manage peak snowbird season Oct–Mar and handle off-season too.",
            posted_ago: "5 days ago",
        },
        OwnerProperty {
            initials: "SL", owner_name: "Sarah Lin", city: "Nashville, TN",
            beds: 2, baths: 1.0, property_type: "Downtown Condo",
            icon: "apartment",
            seeking: "First-time Airbnb host. Need help with setup, listing, and ongoing guest management.",
            posted_ago: "1 week ago",
        },
        OwnerProperty {
            initials: "RH", owner_name: "Ryan Hayes", city: "Asheville, NC",
            beds: 2, baths: 1.0, property_type: "Mountain Cabin",
            icon: "cabin",
            seeking: "Seeking a co-host familiar with Asheville STR regulations and mountain property upkeep.",
            posted_ago: "3 days ago",
        },
    ]
}

// ── Root component (zero-auth) ────────────────────────────────────────────────

#[component]
pub fn CohostMarketplace() -> impl IntoView {
    // "find_cohost" | "list_property"
    let active_tab = RwSignal::new("find_cohost");
    // Profile index currently shown in the connect modal (None = closed)
    let connect_open: RwSignal<Option<usize>> = RwSignal::new(None);
    let search_query = RwSignal::new(String::new());

    // Wrap seed data in StoredValue so multiple reactive `move ||` closures
    // can call .get_value() (which clones) without consuming the original Vec.
    let cohosts_sv = StoredValue::new(seeded_cohosts());
    let cohosts_sv2 = cohosts_sv;
    let props_sv = StoredValue::new(seeded_properties());

    view! {
        <Title text="Cohost Network — Folio"/>
        <Meta name="description"
              content="Find a verified co-host for your Airbnb, or list your property for co-host management. Folio's Cohost Network connects property owners with trusted local experts."/>

        <div class="folio-mktg">
            <MarketingNav
                active=MarketingNavRole::Cohosts
                cta_label="Join Folio"
                cta_href="/#waitlist-wrap"
            />

            // ── Hero ─────────────────────────────────────────────────────────
            <section class="mktg-hero" style="min-height:52vh;padding-bottom:3rem;">
                <div class="mktg-hero-grid-overlay"></div>
                <div class="mktg-hero-inner" style="max-width:760px;">
                    <div class="mktg-eyebrow">
                        <span class="material-symbols-outlined"
                              style="font-size:14px;font-variation-settings:'FILL' 1">
                            "handshake"
                        </span>
                        " Cohost Network · Folio Verified Co-hosts"
                    </div>
                    <h1 class="mktg-hero-h1" style="font-size:clamp(2rem,5vw,3.25rem);">
                        "The marketplace for"
                        <span class="mktg-h1-accent">" trusted co-hosts."</span>
                    </h1>
                    <p class="mktg-hero-sub" style="max-width:580px;">
                        "Own a property but don't want to manage the Airbnb? Find a verified local co-host
                         who handles everything — and earns a share of every booking through Folio's
                         automatic revenue split."
                    </p>

                    // ── Tab switcher ─────────────────────────────────────────
                    <div style="display:flex;gap:8px;margin-bottom:2rem;flex-wrap:wrap;">
                        <button
                            id="tab-find-cohost"
                            class=move || if active_tab.get() == "find_cohost" {
                                "mktg-btn-accent"
                            } else {
                                "mktg-btn-signin"
                            }
                            on:click=move |_| active_tab.set("find_cohost")
                        >
                            <span class="material-symbols-outlined"
                                  style="font-size:16px;vertical-align:middle;margin-right:4px">
                                "search"
                            </span>
                            "I have a property"
                        </button>
                        <button
                            id="tab-list-property"
                            class=move || if active_tab.get() == "list_property" {
                                "mktg-btn-accent"
                            } else {
                                "mktg-btn-signin"
                            }
                            on:click=move |_| active_tab.set("list_property")
                        >
                            <span class="material-symbols-outlined"
                                  style="font-size:16px;vertical-align:middle;margin-right:4px">
                                "home_work"
                            </span>
                            "I want to co-host"
                        </button>
                    </div>

                    // ── Search input ─────────────────────────────────────────
                    <div style="display:flex;gap:10px;max-width:520px;">
                        <input
                            type="search"
                            id="cohost-search"
                            class="mktg-wl-email"
                            style="flex:1;"
                            placeholder=move || if active_tab.get() == "find_cohost" {
                                "Search city or market (e.g. Miami, Asheville)…"
                            } else {
                                "Search co-hosts by city or specialty…"
                            }
                            prop:value=move || search_query.get()
                            on:input=move |ev| search_query.set(event_target_value(&ev))
                        />
                        <button class="mktg-btn-accent" id="cohost-search-btn">
                            "Search"
                        </button>
                    </div>
                </div>
            </section>

            // ── Stats band ───────────────────────────────────────────────────
            <section class="mktg-stats">
                {[
                    ("247+", "Verified co-hosts"),
                    ("4.88", "Avg guest rating"),
                    ("82%",  "Avg occupancy"),
                    ("$0",   "Marketplace fee for owners"),
                ].iter().map(|(val, label)| view! {
                    <div class="mktg-stat">
                        <span class="mktg-stat-val">{*val}</span>
                        <span class="mktg-stat-label">{*label}</span>
                    </div>
                }).collect_view()}
            </section>

            // ── Main content — swaps on tab ───────────────────────────────────
            <section class="mktg-section">
                <div class="mktg-section-inner">

                    // ── OWNER TAB: Browse co-hosts ────────────────────────────
                    {move || (active_tab.get() == "find_cohost").then(|| {
                        let cohosts = cohosts_sv.get_value();
                        let q = search_query.get().to_lowercase();
                        let filtered: Vec<(usize, CohostProfile)> = cohosts
                            .iter()
                            .enumerate()
                            .filter(|(_, c)| {
                                q.is_empty()
                                    || c.city.to_lowercase().contains(&q)
                                    || c.name.to_lowercase().contains(&q)
                                    || c.specialties.iter().any(|s| s.to_lowercase().contains(&q))
                            })
                            .map(|(i, c)| (i, c.clone()))
                            .collect();

                        view! {
                            <div>
                                <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:1.5rem;">
                                    <div>
                                        <p class="mktg-section-eyebrow">"Verified co-hosts"</p>
                                        <h2 class="mktg-section-h2" style="margin-bottom:0.25rem;">
                                            "Find your co-host."
                                        </h2>
                                        <p style="font-size:0.8rem;color:#9ca3af;">
                                            "All reputation data is pulled from connected OTA accounts — not self-reported."
                                        </p>
                                    </div>
                                    <div style="font-size:12px;color:#9ca3af;">
                                        {filtered.len()}" co-hosts"
                                    </div>
                                </div>

                                // ── Co-host grid ─────────────────────────────
                                <div class="mktg-feature-grid" style="grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:1.25rem;">
                                    {filtered.into_iter().map(|(idx, cohost)| {
                                        let idx2 = idx;
                                        view! {
                                            <div class="mktg-str-card" style="display:flex;flex-direction:column;gap:1rem;">
                                                // ── Header row ──────────────────
                                                <div style="display:flex;align-items:center;gap:12px;">
                                                    <div style="
                                                        width:44px;height:44px;border-radius:9999px;
                                                        background:linear-gradient(135deg,#334155,#0f172a);
                                                        display:flex;align-items:center;justify-content:center;
                                                        color:#fff;font-weight:700;font-size:14px;flex-shrink:0;">
                                                        {cohost.initials}
                                                    </div>
                                                    <div style="flex:1;min-width:0;">
                                                        <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap;">
                                                            <span style="font-weight:700;font-size:15px;">{cohost.name}</span>
                                                            {cohost.superhost.then(|| view! {
                                                                <span style="
                                                                    font-size:9px;font-weight:700;
                                                                    background:rgba(255,107,53,.15);
                                                                    color:#ff6b35;border:1px solid rgba(255,107,53,.3);
                                                                    border-radius:4px;padding:1px 5px;
                                                                    text-transform:uppercase;letter-spacing:.05em;">
                                                                    "Superhost"
                                                                </span>
                                                            })}
                                                        </div>
                                                        <div style="font-size:12px;color:#9ca3af;">{cohost.city}</div>
                                                    </div>
                                                    // Trust score ring
                                                    <div style="
                                                        width:40px;height:40px;border-radius:9999px;
                                                        display:flex;align-items:center;justify-content:center;
                                                        font-size:12px;font-weight:800;
                                                        background:rgba(6,214,160,.15);
                                                        border:2px solid #06d6a0;color:#06d6a0;
                                                        flex-shrink:0;">
                                                        {format!("{:.1}", cohost.trust_score)}
                                                    </div>
                                                </div>

                                                // ── Bio ──────────────────────────
                                                <p style="font-size:13px;color:#9ca3af;line-height:1.55;margin:0;">
                                                    {cohost.bio_short}
                                                </p>

                                                // ── Reputation bars ───────────────
                                                <div style="display:flex;flex-direction:column;gap:6px;">
                                                    <CohostRepBar
                                                        label="Guest rating".to_string()
                                                        value_str={format!("{:.2}", cohost.guest_rating)}
                                                        pct={((cohost.guest_rating / 5.0) * 100.0) as u8}
                                                        color="#06d6a0"
                                                    />
                                                    <CohostRepBar
                                                        label="Response rate".to_string()
                                                        value_str={format!("{}%", cohost.response_rate)}
                                                        pct={cohost.response_rate}
                                                        color="#06d6a0"
                                                    />
                                                    <CohostRepBar
                                                        label="Avg occupancy".to_string()
                                                        value_str={format!("{}%", cohost.avg_occupancy)}
                                                        pct={cohost.avg_occupancy}
                                                        color="#60a5fa"
                                                    />
                                                </div>

                                                // ── Quick stats ───────────────────
                                                <div style="display:flex;gap:16px;font-size:12px;">
                                                    <div style="text-align:center;">
                                                        <div style="font-weight:800;font-size:15px;">{cohost.total_stays}</div>
                                                        <div style="color:#9ca3af;">"Total stays"</div>
                                                    </div>
                                                    <div style="text-align:center;">
                                                        <div style="font-weight:800;font-size:15px;">{cohost.typical_split}</div>
                                                        <div style="color:#9ca3af;">"Typical split"</div>
                                                    </div>
                                                    <div style="text-align:center;">
                                                        <div style="font-weight:800;font-size:15px;">{cohost.availability}</div>
                                                        <div style="color:#9ca3af;">"Units open"</div>
                                                    </div>
                                                </div>

                                                // ── Specialties ──────────────────
                                                <div style="display:flex;flex-wrap:wrap;gap:5px;">
                                                    {cohost.specialties.iter().map(|s| view! {
                                                        <span style="
                                                            font-size:10px;font-weight:700;
                                                            background:#1e293b;color:#cbd5e1;
                                                            border:1px solid #334155;
                                                            border-radius:4px;padding:2px 7px;
                                                            text-transform:uppercase;letter-spacing:.05em;">
                                                            {*s}
                                                        </span>
                                                    }).collect_view()}
                                                </div>

                                                // ── Actions ──────────────────────
                                                <div style="display:flex;gap:8px;margin-top:auto;">
                                                    <button
                                                        id={format!("btn-connect-{idx}")}
                                                        class="mktg-btn-accent"
                                                        style="flex:1;padding:.5rem .75rem;font-size:13px;"
                                                        on:click=move |_| connect_open.set(Some(idx2))
                                                    >
                                                        "Connect"
                                                    </button>
                                                    <a
                                                        href={format!("/cohost/{}", idx)}
                                                        class="mktg-btn-signin"
                                                        style="padding:.5rem .75rem;font-size:13px;"
                                                    >
                                                        "View profile"
                                                    </a>
                                                </div>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>

                                // ── Empty state ───────────────────────────────
                                {move || {
                                    let q = search_query.get().to_lowercase();
                                    let all_cohosts = cohosts_sv2.get_value();
                                    if !q.is_empty() && all_cohosts.iter().all(|c| {
                                        !c.city.to_lowercase().contains(&q)
                                        && !c.name.to_lowercase().contains(&q)
                                        && !c.specialties.iter().any(|s| s.to_lowercase().contains(&q))
                                    }) {
                                        view! {
                                            <div style="text-align:center;padding:3rem;color:#9ca3af;">
                                                <span class="material-symbols-outlined" style="font-size:36px;opacity:.4;display:block;margin-bottom:1rem;">
                                                    "person_search"
                                                </span>
                                                <p style="font-weight:600;margin-bottom:.5rem;">"No co-hosts found in that market yet."</p>
                                                <p style="font-size:13px;">"We're onboarding co-hosts nationwide. "<a href="/lp#waitlist-wrap" style="color:#ff6b35;">"Join the waitlist"</a>" to be notified when your market opens."</p>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! { <></> }.into_any()
                                    }
                                }}
                            </div>
                        }
                    })}

                    // ── COHOST TAB: Browse properties seeking a co-host ────────
                    {move || (active_tab.get() == "list_property").then(|| {
                        let properties = props_sv.get_value();
                        view! {
                            <div>
                                <div style="margin-bottom:1.5rem;">
                                    <p class="mktg-section-eyebrow">"Properties seeking a co-host"</p>
                                    <h2 class="mktg-section-h2" style="margin-bottom:.25rem;">
                                        "Find your next property to manage."
                                    </h2>
                                    <p style="font-size:.8rem;color:#9ca3af;">
                                        "Owners post their properties when they're ready to hand off management. Your Folio reputation follows you."
                                    </p>
                                </div>

                                <div class="mktg-feature-grid" style="grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:1.25rem;">
                                    {properties.iter().map(|prop| view! {
                                        <div class="mktg-str-card" style="display:flex;flex-direction:column;gap:.875rem;">
                                            // ── Property image placeholder ───────
                                            <div style="
                                                height:80px;border-radius:.5rem;
                                                background:linear-gradient(135deg,#1e293b,#0f172a);
                                                display:flex;align-items:center;justify-content:center;">
                                                <span class="material-symbols-outlined" style="font-size:32px;color:#334155;">
                                                    {prop.icon}
                                                </span>
                                            </div>
                                            // ── Property meta ────────────────────
                                            <div style="display:flex;align-items:center;justify-content:space-between;">
                                                <div>
                                                    <div style="font-weight:700;font-size:14px;">{prop.city}</div>
                                                    <div style="font-size:12px;color:#9ca3af;">
                                                        {format!("{} · {}BR / {}BA · {}", prop.property_type, prop.beds, prop.baths, prop.posted_ago)}
                                                    </div>
                                                </div>
                                                <div style="
                                                    width:32px;height:32px;border-radius:9999px;
                                                    background:linear-gradient(135deg,#334155,#0f172a);
                                                    display:flex;align-items:center;justify-content:center;
                                                    color:#fff;font-weight:700;font-size:11px;flex-shrink:0;">
                                                    {prop.initials}
                                                </div>
                                            </div>
                                            // ── Seeking note ──────────────────────
                                            <p style="font-size:13px;color:#9ca3af;line-height:1.5;margin:0;flex:1;">
                                                {prop.seeking}
                                            </p>
                                            // ── Actions ──────────────────────────
                                            <button
                                                class="mktg-btn-accent"
                                                style="width:100%;padding:.5rem;font-size:13px;"
                                            >
                                                "Express interest"
                                            </button>
                                        </div>
                                    }).collect_view()}
                                </div>

                                // ── Are you a co-host CTA ─────────────────────
                                <div style="
                                    margin-top:3rem;padding:2rem;
                                    border:1px dashed #334155;border-radius:1rem;
                                    text-align:center;">
                                    <p style="font-weight:700;font-size:16px;margin-bottom:.5rem;">
                                        "Want to list yourself as a co-host?"
                                    </p>
                                    <p style="font-size:13px;color:#9ca3af;margin-bottom:1.25rem;">
                                        "Join the Folio waitlist and we'll set up your verified co-host profile
                                         — with your OTA reputation data imported automatically."
                                    </p>
                                    <a href="/lp#waitlist-wrap" class="mktg-btn-accent">
                                        "Join the waitlist →"
                                    </a>
                                </div>
                            </div>
                        }
                    })}

                </div>
            </section>

            // ── How it works ──────────────────────────────────────────────────
            <section class="mktg-section">
                <div class="mktg-section-inner">
                    <p class="mktg-section-eyebrow">"How it works"</p>
                    <h2 class="mktg-section-h2">"Passive income, professionally managed."</h2>
                    <p class="mktg-section-sub">
                        "Folio handles the money automatically — owners and co-hosts never chase payments."
                    </p>
                    <div class="mktg-str-grid">
                        {[
                            ("search",       "Find a co-host",
                             "Browse verified co-hosts, filter by city and specialty, and read their verified OTA reputation — guest ratings, occupancy, response rate."),
                            ("handshake",    "Agree on terms",
                             "Send a connection request, agree on the split percentage and which properties to include. Everything is documented in Folio."),
                            ("payments",     "Automatic splits",
                             "When a booking confirms, Folio's ledger automatically routes the owner's share and the co-host's commission. No manual transfers."),
                            ("verified_user","Reputation travels with you",
                             "Your co-host's ratings, occupancy, and verified stays are portable. Great co-hosts build a track record that attracts more owners."),
                        ].iter().map(|(icon, title, desc)| view! {
                            <div class="mktg-str-card">
                                <span class="material-symbols-outlined mktg-str-icon">{*icon}</span>
                                <h3 class="mktg-str-title">{*title}</h3>
                                <p class="mktg-str-desc">{*desc}</p>
                            </div>
                        }).collect_view()}
                    </div>
                </div>
            </section>

            // ── Bottom CTA ────────────────────────────────────────────────────
            <section class="mktg-cta-section">
                <div class="mktg-section-inner mktg-cta-inner">
                    <p class="mktg-section-eyebrow" style="color:#ff6b35;">"Beta access open now"</p>
                    <h2 class="mktg-cta-h2">
                        "Your property. A trusted partner. One platform."
                    </h2>
                    <p class="mktg-cta-sub">
                        "Join the Folio waitlist and lock in your place in the Cohost Network
                         before we open to the public."
                    </p>
                    <a href="/lp#waitlist-wrap" class="mktg-btn-accent mktg-btn-lg" id="cohost-mkt-cta-btn">
                        "Reserve my beta spot →"
                    </a>
                    <p style="margin-top:16px;font-size:12px;color:#9ca3af;">
                        "No credit card. No contracts. Cancel anytime."
                    </p>
                </div>
            </section>

            // ── Footer ────────────────────────────────────────────────────────
            <footer class="mktg-footer">
                <div class="mktg-footer-inner">
                    <div>
                        <div class="mktg-footer-logo">"Folio"</div>
                        <div class="mktg-footer-tagline">"Modern Landlord OS"</div>
                    </div>
                    <div class="mktg-footer-links">
                        <a href="/login">"Sign in"</a>
                        <a href="/lp#pricing">"Pricing"</a>
                        <a href="/lp#features">"Features"</a>
                        <a href="/cohost-market">"Cohost Network"</a>
                    </div>
                    <div class="mktg-footer-legal">
                        "© 2026 Folio · Atlas Platform · "
                        <a href="/legal/privacy">"Privacy"</a>
                        " · "
                        <a href="/legal/terms">"Terms"</a>
                    </div>
                </div>
            </footer>

            // ── Connect modal ─────────────────────────────────────────────────
            {move || connect_open.get().map(|idx| {
                let cohosts_m = seeded_cohosts();
                let cohost = cohosts_m.get(idx).cloned();
                if let Some(c) = cohost {
                    let name = c.name;
                    view! {
                        <div
                            style="
                                position:fixed;inset:0;z-index:100;
                                background:rgba(0,0,0,.7);
                                display:flex;align-items:center;justify-content:center;
                                padding:1rem;"
                            on:click=move |_| connect_open.set(None)
                        >
                            <div
                                style="
                                    background:#1a1a2e;border:1px solid #334155;border-radius:1rem;
                                    padding:2rem;max-width:420px;width:100%;
                                    box-shadow:0 25px 50px rgba(0,0,0,.5);"
                                on:click=|ev| ev.stop_propagation()
                            >
                                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:1.25rem;">
                                    <div>
                                        <h3 style="font-weight:700;font-size:17px;margin:0;">
                                            "Connect with "{name}
                                        </h3>
                                        <p style="font-size:12px;color:#9ca3af;margin:.25rem 0 0;">
                                            "Send an introduction through Folio"
                                        </p>
                                    </div>
                                    <button
                                        on:click=move |_| connect_open.set(None)
                                        style="background:none;border:none;color:#9ca3af;cursor:pointer;font-size:1.5rem;line-height:1;"
                                    >
                                        "×"
                                    </button>
                                </div>
                                <div style="display:flex;flex-direction:column;gap:12px;">
                                    <input type="text" class="mktg-wl-input" placeholder="Your name"/>
                                    <input type="email" class="mktg-wl-input" placeholder="Your email"/>
                                    <textarea class="mktg-wl-input" rows="3"
                                        placeholder="Tell them about your property and what you're looking for…"
                                        style="resize:vertical;"></textarea>
                                    <button class="mktg-btn-accent mktg-btn-full" id="modal-send-intro-btn">
                                        "Send Introduction →"
                                    </button>
                                    <p style="font-size:11px;color:#9ca3af;text-align:center;">
                                        "You'll need a Folio account to complete the co-hosting agreement. "
                                        <a href="/lp#waitlist-wrap" style="color:#ff6b35;">"Join the waitlist"</a>
                                        " to get access."
                                    </p>
                                </div>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            })}

            // ── Nav scroll JS ────────────────────────────────────────────────
            <script>{r#"
(function(){
  var nav = document.getElementById('mktg-nav');
  if (nav) {
    window.addEventListener('scroll', function() {
      if (window.scrollY > 40) {
        nav.classList.add('mktg-nav--scrolled');
      } else {
        nav.classList.remove('mktg-nav--scrolled');
      }
    }, { passive: true });
  }
})();
            "#}</script>
        </div>
    }
}

// ── Sub-component: inline reputation progress bar ─────────────────────────────

#[component]
fn CohostRepBar(label: String, value_str: String, pct: u8, color: &'static str) -> impl IntoView {
    view! {
        <div>
            <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:3px;">
                <span style="color:#9ca3af;">{label}</span>
                <span style="font-weight:700;">{value_str}</span>
            </div>
            <div style="height:5px;border-radius:9999px;background:#1e293b;overflow:hidden;">
                <div style={format!("height:100%;border-radius:9999px;background:{};width:{}%;", color, pct)}>
                </div>
            </div>
        </div>
    }
}
