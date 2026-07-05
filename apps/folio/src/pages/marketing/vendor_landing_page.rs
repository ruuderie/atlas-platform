//! VendorLandingPage — marketing page targeting vendors, contractors & service providers.
//!
//! Served at: `/vendors`
//!
//! Zero-auth, accessible to any visitor. Independently managed under
//! `app_id = "folio-vendor"` in platform-admin.
//!
//! # Value proposition
//! Vendors (contractors, plumbers, HVAC, cleaners, landscapers) get job dispatch,
//! invoicing, a marketplace profile, and payment — all in one place.
//!
//! # Pricing model
//! Freemium: free marketplace listing + job acceptance; paid tiers unlock priority
//! placement, auto-invoicing, and 0% platform fee.

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use crate::components::lang::{LanguageSwitcher, get_current_lang};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn VendorLandingPage() -> impl IntoView {
    view! {
        <Title text="Folio for Vendors – Get Jobs, Get Paid, No Chasing"/>
        <Meta name="description" content="Folio connects vendors and contractors to property managers and landlords. Get dispatched jobs, send invoices, collect payment, and grow your service business — all on one platform."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/vendors"/>

        <VendorNav/>
        <VendorHero/>
        <VendorTrades/>
        <VendorHow/>
        <VendorFeatures/>
        <VendorProfilePreviews/>
        <VendorPricing/>
        <VendorCta/>
        <BetaCalloutStrip/>
        <VendorFooter/>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn VendorNav() -> impl IntoView {
    let menu_open = RwSignal::new(false);
    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <a href="/" class="mktg-nav-logo">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </a>
                <div class="mktg-nav-links">
                    <a href="#vendor-how">"How it works"</a>
                    <a href="#vendor-features">"Features"</a>
                    <a href="#vendor-pricing">"Pricing"</a>
                    <a href="/" class="mktg-nav-broker-link">"For Landlords"</a>
                    <a href="/property-managers">"For PMs"</a>
                    <a href="/founding" class="mktg-nav-broker-link">"Founders ✦"</a>
                </div>
                <div class="mktg-nav-actions">
                    {
                        let lang_res = Resource::new(|| (), |_| get_current_lang());
                        view! {
                            <Suspense fallback=|| ()>
                                {move || lang_res.get().and_then(|r| r.ok()).map(|code| view! {
                                    <LanguageSwitcher current_lang=code/>
                                })}
                            </Suspense>
                        }
                    }
                    <a href="/login" class="mktg-btn-signin" id="vendor-nav-signin-btn">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </a>
                    <a href="#vendor-signup" class="mktg-btn-accent">"Join marketplace"</a>
                    <button
                        class="mktg-nav-hamburger"
                        aria-label="Toggle navigation menu"
                        on:click=move |_| menu_open.update(|o| *o = !*o)
                    >
                        <span class="material-symbols-outlined">
                            {move || if menu_open.get() { "close" } else { "menu" }}
                        </span>
                    </button>
                </div>
            </div>
        </nav>
        <div class=move || if menu_open.get() {
            "mktg-mobile-nav mktg-mobile-nav--open"
        } else {
            "mktg-mobile-nav"
        }>
            <a href="#vendor-how"          on:click=move |_| menu_open.set(false)>"How it works"</a>
            <a href="#vendor-features"     on:click=move |_| menu_open.set(false)>"Features"</a>
            <a href="#vendor-pricing"      on:click=move |_| menu_open.set(false)>"Pricing"</a>
            <a href="/"                    on:click=move |_| menu_open.set(false)>"For Landlords"</a>
            <a href="/property-managers"   on:click=move |_| menu_open.set(false)>"For PMs"</a>
            <a href="/brokers"             on:click=move |_| menu_open.set(false)>"For Brokers"</a>
            <a href="#vendor-signup"       on:click=move |_| menu_open.set(false)>"Join marketplace"</a>
            <a href="/founding"            on:click=move |_| menu_open.set(false)>"Founding ✦"</a>
            <a href="/beta"               on:click=move |_| menu_open.set(false)>"Apply for Beta"</a>
        </div>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn VendorHero() -> impl IntoView {
    let step         = RwSignal::new(0u8);  // 0=category, 1=details, 2=success
    let trade        = RwSignal::new(String::new());
    let email        = RwSignal::new(String::new());
    let biz_name     = RwSignal::new(String::new());
    let service_area = RwSignal::new(String::new());
    let loading      = RwSignal::new(false);
    let err_msg      = RwSignal::new(String::new());

    // Trade categories — what we need to build the vendor network
    let trades: &[(&str, &str)] = &[
        ("cleaning",      "🧹 Cleaning"),
        ("handyman",      "🔧 Handyman"),
        ("plumbing",      "🚿 Plumbing"),
        ("electrical",    "⚡ Electrical"),
        ("hvac",          "❄️ HVAC"),
        ("painting",      "🖌️ Painting"),
        ("landscaping",   "🌿 Landscaping"),
        ("roofing",       "🏠 Roofing"),
        ("flooring",      "🪵 Flooring"),
        ("pest-control",  "🐛 Pest Control"),
        ("appliance",     "🛠️ Appliances"),
        ("locksmith",     "🔐 Locksmith"),
        ("inspection",    "🔍 Inspection"),
        ("movers",        "📦 Movers"),
        ("junk-removal",  "🗑️ Junk Removal"),
        ("pool-spa",      "🏊 Pool & Spa"),
        ("security",      "📷 Security"),
        ("solar",         "☀️ Solar"),
        ("general",       "🏗️ General Contractor"),
    ];

    let submit = move |_| {
        if email.get().is_empty() || service_area.get().is_empty() { return; }
        loading.set(true);
        let payload = format!(
            r#"{{"email":"{}","trade":"{}","biz_name":"{}","service_area":"{}","source":"vendor-page"}}"#,
            email.get(), trade.get(), biz_name.get(), service_area.get()
        );
        leptos::task::spawn_local(async move {
            let resp = gloo_net::http::Request::post("/api/waitlist-signup")
                .header("Content-Type", "application/json")
                .body(payload)
                .unwrap()
                .send()
                .await;
            let _ = resp;
            loading.set(false);
            step.set(2);
        });
    };

    view! {
        <section id="vendor-hero" class="mktg-hero" style="background:linear-gradient(160deg,#0a1628 0%,#0d1f3c 50%,#0a1220 100%);">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:860px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"handyman"</span>
                    " Free to join · Get paid instantly · No chasing"
                </div>
                <h1 class="mktg-hero-h1">
                    "Get jobs."
                    <span class="mktg-h1-accent"> " Get paid."</span>
                    <br/>
                    "No chasing."
                </h1>
                <p class="mktg-hero-sub" style="max-width:580px;margin:1.5rem auto 0;">
                    "Join the Folio vendor marketplace and get connected to property managers \
                     and landlords in your area. Jobs dispatched to your phone. \
                     Invoices sent in one tap. Payment in 24 hours."
                </p>

                // ── Multi-step vendor signup ────────────────────────────────
                <div id="vendor-signup" style="margin-top:2.5rem;" class="mktg-wl-wrap">

                    // Step 0: pick your trade
                    {move || (step.get() == 0).then(|| view! {
                        <div class="mktg-wl-step">
                            <div class="mktg-wl-card" style="text-align:left;">
                                <p class="mktg-wl-card-head">"What's your trade?"</p>
                                <div class="mktg-pill-row" style="flex-wrap:wrap;gap:.5rem;">
                                    {trades.iter().map(|(key, label)| {
                                        let k   = key.to_string();
                                        let lbl = label.to_string();
                                        let k2  = k.clone();
                                        view! {
                                            <button
                                                class=move || if trade.get() == k {
                                                    "mktg-pill mktg-pill-role selected"
                                                } else {
                                                    "mktg-pill mktg-pill-role"
                                                }
                                                on:click=move |_| trade.set(k2.clone())
                                            >{lbl}</button>
                                        }
                                    }).collect_view()}
                                </div>
                                <button
                                    class="mktg-btn-accent mktg-btn-lg"
                                    style="margin-top:1.5rem;width:100%;"
                                    disabled=move || trade.get().is_empty()
                                    on:click=move |_| {
                                        if !trade.get().is_empty() { step.set(1); }
                                    }
                                >
                                    "Continue →"
                                </button>
                            </div>
                        </div>
                    })}

                    // Step 1: contact + service area
                    {move || (step.get() == 1).then(|| view! {
                        <div class="mktg-wl-step mktg-wl-details">
                            <div class="mktg-wl-card" style="text-align:left;">
                                <p class="mktg-wl-card-head">"Almost done — takes 30 seconds"</p>

                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">"Email address"</label>
                                    <input
                                        type="email" class="mktg-wl-input"
                                        placeholder="you@yourbusiness.com"
                                        id="vendor-hero-email"
                                        prop:value=move || email.get()
                                        on:input=move |e| email.set(event_target_value(&e))
                                    />
                                </div>

                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">
                                        "Business name "
                                        <span class="mktg-wl-optional">"(optional)"</span>
                                    </label>
                                    <input
                                        type="text" class="mktg-wl-input"
                                        placeholder="e.g. Rodriguez Plumbing LLC"
                                        prop:value=move || biz_name.get()
                                        on:input=move |e| biz_name.set(event_target_value(&e))
                                    />
                                </div>

                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">"Service area (city or zip code)"</label>
                                    <input
                                        type="text" class="mktg-wl-input"
                                        placeholder="e.g. Miami, FL or 33101"
                                        prop:value=move || service_area.get()
                                        on:input=move |e| service_area.set(event_target_value(&e))
                                    />
                                </div>

                                {move || (!err_msg.get().is_empty()).then(|| view! {
                                    <p class="mktg-wl-err">{err_msg.get()}</p>
                                })}

                                <button
                                    class="mktg-btn-green mktg-btn-lg"
                                    style="width:100%;margin-top:.75rem;"
                                    disabled=move || loading.get()
                                    on:click=submit.clone()
                                >
                                    <span class="material-symbols-outlined" style="font-size:20px;font-variation-settings:'FILL' 1">"check_circle"</span>
                                    {move || if loading.get() { "Submitting…" } else { "Join the marketplace" }}
                                </button>
                                <button
                                    style="background:none;border:none;color:var(--mk-muted);font-size:.8rem;cursor:pointer;margin-top:.5rem;"
                                    on:click=move |_| step.set(0)
                                >"← Change trade"</button>
                            </div>
                        </div>
                    })}

                    // Step 2: success
                    {move || (step.get() == 2).then(|| view! {
                        <div class="mktg-wl-success">
                            <div class="mktg-success-icon">
                                <span class="material-symbols-outlined" style="font-size:36px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            </div>
                            <h3 class="mktg-success-h3">"You're in the vendor network!"</h3>
                            <p class="mktg-success-sub">
                                "We'll reach out with your marketplace profile setup link \
                                 when we launch in your area. The more vendors join early, \
                                 the faster we can serve landlords and PMs near you."
                            </p>
                            <div class="mktg-success-card">
                                <div>
                                    <div class="mktg-success-label">"Your trade"</div>
                                    <div class="mktg-success-num" style="font-size:1.1rem;">{move || trade.get()}</div>
                                </div>
                                <div class="mktg-success-div"></div>
                                <div>
                                    <div class="mktg-success-label">"Service area"</div>
                                    <div class="mktg-success-num" style="font-size:1.1rem;">{move || service_area.get()}</div>
                                </div>
                            </div>
                        </div>
                    })}
                </div>

                <div class="mktg-stats" style="margin-top:3rem;border-top:1px solid var(--mk-border);padding-top:2rem;">
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"Free"</div>
                        <div class="mktg-stat-label">"to join & accept jobs"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"24h"</div>
                        <div class="mktg-stat-label">"payment turnaround"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"0"</div>
                        <div class="mktg-stat-label">"invoicing or chasing"</div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Trades We're Building ─────────────────────────────────────────────────────

#[component]
fn VendorTrades() -> impl IntoView {
    let categories = vec![
        ("cleaning",      "🧹", "Cleaning & Turnover",  "Move-out cleans, vacation rental turnovers, recurring housekeeping"),
        ("handyman",      "🔧", "Handyman",             "Minor repairs, furniture assembly, caulking, drywall patches"),
        ("plumbing",      "🚿", "Plumbing",             "Leaks, fixture replacements, drain clearing, water heater service"),
        ("electrical",    "⚡", "Electrical",           "Outlet repairs, panel work, lighting installs, code compliance"),
        ("hvac",          "❄️", "HVAC",                 "AC service, furnace repair, filter programs, duct cleaning"),
        ("painting",      "🖌️", "Painting",             "Interior & exterior, unit turns, touch-ups, power washing"),
        ("landscaping",   "🌿", "Landscaping",          "Lawn care, tree trimming, irrigation, seasonal cleanups"),
        ("roofing",       "🏠", "Roofing",              "Inspections, leak repairs, gutter cleaning, full replacements"),
        ("flooring",      "🪵", "Flooring",             "Hardwood, tile, LVP install and repair, carpet replacement"),
        ("pest-control",  "🐛", "Pest Control",         "Extermination, prevention programs, termite inspections"),
        ("appliance",     "🛠️", "Appliance Repair",     "Washers, dryers, refrigerators, dishwashers, stoves"),
        ("locksmith",     "🔐", "Locksmith",            "Rekeying, lock installs, smart lock setup, access control"),
        ("inspection",    "🔍", "Inspection",           "Move-in/out inspections, general home inspections, code checks"),
        ("movers",        "📦", "Movers",               "Residential & commercial moves, furniture, appliance relocation"),
        ("junk-removal",  "🗑️", "Junk Removal",         "Tenant cleanouts, bulk hauling, estate clearances, debris removal"),
        ("pool-spa",      "🏊", "Pool & Spa",           "Cleaning, chemical balance, equipment repair, opening/closing"),
        ("security",      "📷", "Security",             "Camera installs, alarm systems, smart home setup"),
        ("solar",         "☀️", "Solar",                "Panel installs, maintenance, battery storage, inspections"),
        ("general",       "🏗️", "General Contractor",  "Renovations, additions, unit upgrades, larger project management"),
    ];

    view! {
        <section id="vendor-trades" class="mktg-section" style="background:rgba(6,214,160,.02);border-top:1px solid rgba(6,214,160,.08);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Building the network"</p>
                <h2 class="mktg-section-h2">"Every trade. One marketplace."</h2>
                <p class="mktg-section-sub">
                    "We're signing up vendors across all 19 categories before launch. \
                     Early vendors get priority placement and the Founding Vendor rate — free for life."
                </p>
                <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(280px,1fr));gap:1rem;margin-top:2.5rem;">
                    {categories.into_iter().map(|(_, icon, name, desc)| view! {
                        <a href="#vendor-signup" style="text-decoration:none;">
                            <div class="mktg-str-card" style="cursor:pointer;transition:border-color .15s,transform .15s;"
                                 onmouseover="this.style.borderColor='rgba(6,214,160,.35)';this.style.transform='translateY(-2px)'"
                                 onmouseout="this.style.borderColor='';this.style.transform=''">
                                <div style="display:flex;align-items:center;gap:.75rem;margin-bottom:.6rem;">
                                    <span style="font-size:1.5rem;">{icon}</span>
                                    <strong style="font-size:.95rem;color:#fff;">{name}</strong>
                                </div>
                                <p style="font-size:.82rem;color:var(--mk-muted);margin:0;line-height:1.5;">{desc}</p>
                            </div>
                        </a>
                    }).collect_view()}
                </div>
                <div style="text-align:center;margin-top:2.5rem;">
                    <a href="#vendor-signup" class="mktg-btn-accent mktg-btn-lg" id="vendor-trades-cta">
                        "Register your trade →"
                    </a>
                    <p style="margin-top:.75rem;font-size:.78rem;color:#6b7280;">"Free to join. No subscription required."</p>
                </div>
            </div>
        </section>
    }
}

// ── Profile Previews ─────────────────────────────────────────────────────────

/// Three-tab mock of what a vendor profile looks like across platform surfaces:
///   Tab 0 — Your Profile Card   (the vendor's own view, G-27 scorecard)
///   Tab 1 — Network Search      (how a landlord/PM finds you inside their instance)
///   Tab 2 — Service Finder      (tenant-facing search result card)
#[component]
fn VendorProfilePreviews() -> impl IntoView {
    let active_tab = RwSignal::new(0u8);

    view! {
        <section id="vendor-preview" class="mktg-section" style="background:rgba(255,107,53,.02);border-top:1px solid rgba(255,107,53,.08);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Your profile on the platform"</p>
                <h2 class="mktg-section-h2">"See exactly how you show up."</h2>
                <p class="mktg-section-sub" style="max-width:580px;margin:0 auto 2.5rem;">
                    "Your Folio profile surfaces differently depending on who's looking. \
                     Every view is powered by the Atlas G-27 Scorecard — a real-time \
                     reputation engine built on verified job data, not just star ratings."
                </p>

                // ── Tab bar ───────────────────────────────────────────────────
                <div class="vp-tab-row">
                    <button class=move || if active_tab.get()==0 {"vp-tab vp-tab--active"} else {"vp-tab"}
                            on:click=move |_| active_tab.set(0)>
                        <span class="material-symbols-outlined" style="font-size:18px;font-variation-settings:'FILL' 1">"badge"</span>
                        "Your Profile"
                    </button>
                    <button class=move || if active_tab.get()==1 {"vp-tab vp-tab--active"} else {"vp-tab"}
                            on:click=move |_| active_tab.set(1)>
                        <span class="material-symbols-outlined" style="font-size:18px;font-variation-settings:'FILL' 1">"manage_search"</span>
                        "Network Search"
                    </button>
                    <button class=move || if active_tab.get()==2 {"vp-tab vp-tab--active"} else {"vp-tab"}
                            on:click=move |_| active_tab.set(2)>
                        <span class="material-symbols-outlined" style="font-size:18px;font-variation-settings:'FILL' 1">"home_repair_service"</span>
                        "Service Finder"
                    </button>
                </div>

                // ── Tab 0: Your Profile (G-27 scorecard) ─────────────────────
                {move || (active_tab.get() == 0).then(|| view! {
                    <div class="vp-panel">
                        // Browser chrome mockup
                        <div class="vp-chrome">
                            <span class="vp-chrome-dot" style="background:#ff5f57;"></span>
                            <span class="vp-chrome-dot" style="background:#ffbd2e;"></span>
                            <span class="vp-chrome-dot" style="background:#28ca41;"></span>
                            <span class="vp-chrome-url">"app.folio.co/vendor/profile"</span>
                        </div>
                        <div class="vp-screen">

                            // ── Profile header ───────────────────────────────
                            <div class="vp-profile-header">
                                <div class="vp-avatar">"MR"</div>
                                <div class="vp-profile-meta">
                                    <div style="display:flex;align-items:center;gap:.5rem;">
                                        <h3 class="vp-profile-name">"Martinez Plumbing LLC"</h3>
                                        <span class="vp-verified-badge">
                                            <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"verified"</span>
                                            "Verified"
                                        </span>
                                    </div>
                                    <div class="vp-profile-trade">"🚿 Plumbing · Miami-Dade, FL · 12 mi radius"</div>
                                    <div style="display:flex;align-items:center;gap:1rem;margin-top:.4rem;">
                                        <span class="vp-stat-chip">
                                            <span class="material-symbols-outlined" style="font-size:13px">"work"</span>
                                            "147 jobs completed"
                                        </span>
                                        <span class="vp-stat-chip">
                                            <span class="material-symbols-outlined" style="font-size:13px">"schedule"</span>
                                            "4.2 yr on platform"
                                        </span>
                                    </div>
                                </div>
                                // ── Composite score badge ─────────────────────
                                <div class="vp-score-badge">
                                    <div class="vp-score-num">"94"</div>
                                    <div class="vp-score-label">"Atlas Score"</div>
                                    <div class="vp-score-band vp-band-elite">"Elite"</div>
                                </div>
                            </div>

                            // ── G-27 dimension scorecard ─────────────────────
                            <div class="vp-scorecard-section">
                                <div class="vp-sc-header">
                                    <span style="font-size:.75rem;font-weight:700;text-transform:uppercase;letter-spacing:.07em;color:var(--mk-muted);">"G-27 Atlas Scorecard"</span>
                                    <span class="vp-confidence-chip">"High confidence · 147 verified entries"</span>
                                </div>
                                <div class="vp-dimensions">
                                    // Response Time
                                    <div class="vp-dim">
                                        <div class="vp-dim-label">
                                            <span>"Response Time"</span>
                                            <span class="vp-dim-score">"97"</span>
                                        </div>
                                        <div class="vp-dim-bar-track">
                                            <div class="vp-dim-bar" style="width:97%;background:linear-gradient(90deg,#06d6a0,#00b894);"></div>
                                        </div>
                                        <span class="vp-dim-pct">"Top 3%"</span>
                                    </div>
                                    // Work Quality
                                    <div class="vp-dim">
                                        <div class="vp-dim-label">
                                            <span>"Work Quality"</span>
                                            <span class="vp-dim-score">"96"</span>
                                        </div>
                                        <div class="vp-dim-bar-track">
                                            <div class="vp-dim-bar" style="width:96%;background:linear-gradient(90deg,#06d6a0,#00b894);"></div>
                                        </div>
                                        <span class="vp-dim-pct">"Top 4%"</span>
                                    </div>
                                    // Reliability
                                    <div class="vp-dim">
                                        <div class="vp-dim-label">
                                            <span>"Reliability"</span>
                                            <span class="vp-dim-score">"93"</span>
                                        </div>
                                        <div class="vp-dim-bar-track">
                                            <div class="vp-dim-bar" style="width:93%;background:linear-gradient(90deg,#06d6a0,#00b894);"></div>
                                        </div>
                                        <span class="vp-dim-pct">"Top 7%"</span>
                                    </div>
                                    // Pricing Accuracy
                                    <div class="vp-dim">
                                        <div class="vp-dim-label">
                                            <span>"Pricing Accuracy"</span>
                                            <span class="vp-dim-score">"91"</span>
                                        </div>
                                        <div class="vp-dim-bar-track">
                                            <div class="vp-dim-bar" style="width:91%;background:linear-gradient(90deg,#f59e0b,#e67e22);"></div>
                                        </div>
                                        <span class="vp-dim-pct">"Top 9%"</span>
                                    </div>
                                    // Communication
                                    <div class="vp-dim">
                                        <div class="vp-dim-label">
                                            <span>"Communication"</span>
                                            <span class="vp-dim-score">"95"</span>
                                        </div>
                                        <div class="vp-dim-bar-track">
                                            <div class="vp-dim-bar" style="width:95%;background:linear-gradient(90deg,#06d6a0,#00b894);"></div>
                                        </div>
                                        <span class="vp-dim-pct">"Top 5%"</span>
                                    </div>
                                </div>

                                // ── Trend sparkline (time-series) ─────────────
                                <div class="vp-trend-row">
                                    <span style="font-size:.75rem;color:var(--mk-muted);">"12-month trend"</span>
                                    <div class="vp-sparkline">
                                        <svg viewBox="0 0 120 32" style="width:120px;height:32px;">
                                            <polyline
                                                points="0,28 15,24 30,26 45,20 60,18 75,15 90,10 105,8 120,6"
                                                fill="none" stroke="#06d6a0" stroke-width="2"
                                                stroke-linecap="round" stroke-linejoin="round"
                                            />
                                            <circle cx="120" cy="6" r="3" fill="#06d6a0"/>
                                        </svg>
                                        <span style="font-size:.75rem;color:#06d6a0;font-weight:700;">"↑ +6 pts"</span>
                                    </div>
                                </div>
                            </div>

                            // ── Quick stats footer ───────────────────────────
                            <div class="vp-profile-footer">
                                <div class="vp-foot-stat">
                                    <span class="vp-foot-num">"$285"</span>
                                    <span class="vp-foot-label">"Avg ticket"</span>
                                </div>
                                <div class="vp-foot-stat">
                                    <span class="vp-foot-num">"98%"</span>
                                    <span class="vp-foot-label">"On-time"</span>
                                </div>
                                <div class="vp-foot-stat">
                                    <span class="vp-foot-num">"$41.8k"</span>
                                    <span class="vp-foot-label">"Earned YTD"</span>
                                </div>
                                <div class="vp-foot-stat">
                                    <span class="vp-foot-num">"4.9 ★"</span>
                                    <span class="vp-foot-label">"Client rating"</span>
                                </div>
                            </div>
                        </div>
                        <p class="vp-caption">"Your dashboard — live scores, earnings, and trend data in one place."</p>
                    </div>
                })}

                // ── Tab 1: Network Instance Search ────────────────────────────
                {move || (active_tab.get() == 1).then(|| view! {
                    <div class="vp-panel">
                        <div class="vp-chrome">
                            <span class="vp-chrome-dot" style="background:#ff5f57;"></span>
                            <span class="vp-chrome-dot" style="background:#ffbd2e;"></span>
                            <span class="vp-chrome-dot" style="background:#28ca41;"></span>
                            <span class="vp-chrome-url">"app.folio.co/maintenance/find-vendor"</span>
                        </div>
                        <div class="vp-screen">
                            // Search bar
                            <div class="vp-search-bar">
                                <span class="material-symbols-outlined" style="color:var(--mk-muted);font-size:18px">"search"</span>
                                <span style="color:var(--mk-muted);font-size:.9rem;">"Plumbing · Miami, FL · Available this week"</span>
                                <span class="vp-search-badge">"3 results"</span>
                            </div>

                            // Result cards (your card is first / featured)
                            <div style="display:flex;flex-direction:column;gap:.75rem;margin-top:1rem;">

                                // Your card — ranked #1
                                <div class="vp-search-card vp-search-card--top">
                                    <div class="vp-sc-rank">"#1"</div>
                                    <div class="vp-avatar vp-avatar-sm">"MR"</div>
                                    <div style="flex:1;min-width:0;">
                                        <div style="display:flex;align-items:center;gap:.5rem;">
                                            <strong style="font-size:.95rem;color:#fff;">"Martinez Plumbing LLC"</strong>
                                            <span class="vp-verified-badge">"✓ Verified"</span>
                                            <span class="vp-search-atlas-score">"94 ⬛"</span>
                                        </div>
                                        <div style="font-size:.8rem;color:var(--mk-muted);margin-top:.15rem;">
                                            "🚿 Plumbing · 3.2 mi away · "
                                            <span style="color:#06d6a0;font-weight:600;">"Available Mon–Fri"</span>
                                        </div>
                                        <div style="display:flex;gap:.5rem;margin-top:.4rem;flex-wrap:wrap;">
                                            <span class="vp-tag">"147 jobs"</span>
                                            <span class="vp-tag">"Avg $285"</span>
                                            <span class="vp-tag">"98% on-time"</span>
                                            <span class="vp-tag vp-tag-green">"Responded in 4 min"</span>
                                        </div>
                                    </div>
                                    <button class="mktg-btn-accent" style="font-size:.8rem;padding:.4rem .9rem;flex-shrink:0;">"Dispatch →"</button>
                                </div>

                                // Competitor #2 — greyed out
                                <div class="vp-search-card" style="opacity:.55;">
                                    <div class="vp-sc-rank" style="color:var(--mk-muted);">"#2"</div>
                                    <div class="vp-avatar vp-avatar-sm" style="background:rgba(255,255,255,.1);">"JT"</div>
                                    <div style="flex:1;">
                                        <div style="font-size:.9rem;color:var(--mk-text);">"Joe's Plumbing"</div>
                                        <div style="font-size:.78rem;color:var(--mk-muted);">"🚿 Plumbing · 7.8 mi away · Next available Thu"</div>
                                        <div style="display:flex;gap:.5rem;margin-top:.4rem;">
                                            <span class="vp-tag">"62 jobs"</span>
                                            <span class="vp-tag">"Avg $310"</span>
                                            <span class="vp-tag">"Atlas Score: 71"</span>
                                        </div>
                                    </div>
                                </div>

                                // Competitor #3 — greyed out
                                <div class="vp-search-card" style="opacity:.4;">
                                    <div class="vp-sc-rank" style="color:var(--mk-muted);">"#3"</div>
                                    <div class="vp-avatar vp-avatar-sm" style="background:rgba(255,255,255,.07);">"RS"</div>
                                    <div style="flex:1;">
                                        <div style="font-size:.9rem;color:var(--mk-text);">"Reliable Solutions"</div>
                                        <div style="font-size:.78rem;color:var(--mk-muted);">"🚿 Plumbing · 11.2 mi away · Next available Fri"</div>
                                        <div style="display:flex;gap:.5rem;margin-top:.4rem;">
                                            <span class="vp-tag">"29 jobs"</span>
                                            <span class="vp-tag">"Atlas Score: 58"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <p class="vp-caption">"Your Atlas Score determines your rank in every landlord and PM's search. Higher score = top of the list."</p>
                    </div>
                })}

                // ── Tab 2: Service Finder (tenant / landlord consumer view) ───
                {move || (active_tab.get() == 2).then(|| view! {
                    <div class="vp-panel">
                        <div class="vp-chrome">
                            <span class="vp-chrome-dot" style="background:#ff5f57;"></span>
                            <span class="vp-chrome-dot" style="background:#ffbd2e;"></span>
                            <span class="vp-chrome-dot" style="background:#28ca41;"></span>
                            <span class="vp-chrome-url">"folio.co/services/plumbing/miami"</span>
                        </div>
                        <div class="vp-screen">
                            <div style="font-size:.75rem;color:var(--mk-muted);margin-bottom:1rem;">
                                "Plumbers near Miami, FL · 3 verified vendors"
                            </div>

                            // Featured vendor card — consumer view
                            <div class="vp-service-card">
                                <div style="display:flex;justify-content:space-between;align-items:flex-start;">
                                    <div style="display:flex;gap:1rem;align-items:center;">
                                        <div class="vp-avatar" style="width:52px;height:52px;font-size:1.1rem;">"MR"</div>
                                        <div>
                                            <div style="display:flex;align-items:center;gap:.5rem;">
                                                <strong style="font-size:1rem;color:#fff;">"Martinez Plumbing LLC"</strong>
                                                <span class="vp-verified-badge">"✓"</span>
                                            </div>
                                            <div style="font-size:.8rem;color:var(--mk-muted);margin-top:.1rem;">"Licensed · Insured · Miami-Dade"</div>
                                            // Star rating (rendered from G-27 composite)
                                            <div style="display:flex;align-items:center;gap:.35rem;margin-top:.3rem;">
                                                <span style="color:#f59e0b;font-size:.95rem;">"★★★★★"</span>
                                                <span style="font-size:.85rem;font-weight:700;color:#fff;">"4.9"</span>
                                                <span style="font-size:.78rem;color:var(--mk-muted);">"(147 reviews)"</span>
                                            </div>
                                        </div>
                                    </div>
                                    <div style="text-align:right;">
                                        <div style="font-size:.75rem;color:var(--mk-muted);">"Starting from"</div>
                                        <div style="font-size:1.4rem;font-weight:800;color:#fff;">"$95"</div>
                                        <div style="font-size:.72rem;color:var(--mk-muted);">"service call"</div>
                                    </div>
                                </div>

                                // Tags
                                <div style="display:flex;gap:.4rem;flex-wrap:wrap;margin-top:1rem;">
                                    <span class="vp-tag vp-tag-green">"Available today"</span>
                                    <span class="vp-tag">"Emergency service"</span>
                                    <span class="vp-tag">"Free estimate"</span>
                                    <span class="vp-tag">"ACH accepted"</span>
                                </div>

                                // Recent jobs
                                <div class="vp-recent-jobs">
                                    <div style="font-size:.72rem;font-weight:700;color:var(--mk-muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:.5rem;">"Recent jobs"</div>
                                    <div class="vp-job-row">
                                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                                        <span>"Water heater replacement · 2 days ago"</span>
                                        <span class="vp-job-rating">"★ 5.0"</span>
                                    </div>
                                    <div class="vp-job-row">
                                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                                        <span>"Drain clearing · 1 week ago"</span>
                                        <span class="vp-job-rating">"★ 5.0"</span>
                                    </div>
                                    <div class="vp-job-row">
                                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                                        <span>"Fixture replacement · 2 weeks ago"</span>
                                        <span class="vp-job-rating">"★ 4.8"</span>
                                    </div>
                                </div>

                                // CTA
                                <div style="display:flex;gap:.75rem;margin-top:1.25rem;">
                                    <a href="#vendor-signup" class="mktg-btn-accent" style="flex:1;text-align:center;padding:.7rem;">"Book now"</a>
                                    <button style="flex:1;padding:.7rem;border-radius:8px;border:1px solid var(--mk-border);background:none;color:var(--mk-muted);font-size:.85rem;cursor:pointer;">"Get estimate"</button>
                                </div>
                            </div>
                        </div>
                        <p class="vp-caption">"What tenants and landlords see when they search for services. Your reviews and score determine placement."</p>
                    </div>
                })}
            </div>
        </section>
    }
}

// ── How It Works ──────────────────────────────────────────────────────────────

#[component]
fn VendorHow() -> impl IntoView {
    view! {
        <section id="vendor-how" class="mktg-section">
            <div class="mktg-section-inner" style="text-align:center;">
                <p class="mktg-section-eyebrow">"How it works"</p>
                <h2 class="mktg-section-h2">"Three steps from job to payment."</h2>
                <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:2rem;margin-top:2.5rem;max-width:900px;margin-inline:auto;">
                    <div class="mktg-str-card" style="text-align:center;padding:2rem;">
                        <div style="width:48px;height:48px;border-radius:50%;background:rgba(6,214,160,.15);border:2px solid #06d6a0;display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
                            <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"person_add"</span>
                        </div>
                        <h3 style="font-size:1rem;font-weight:600;margin-bottom:.5rem;">"1. Create your profile"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"List your trade, service area, and availability. Free in under 5 minutes."</p>
                    </div>
                    <div class="mktg-str-card" style="text-align:center;padding:2rem;">
                        <div style="width:48px;height:48px;border-radius:50%;background:rgba(245,158,11,.15);border:2px solid #f59e0b;display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
                            <span class="material-symbols-outlined" style="color:#f59e0b;font-variation-settings:'FILL' 1">"notifications"</span>
                        </div>
                        <h3 style="font-size:1rem;font-weight:600;margin-bottom:.5rem;">"2. Get dispatched"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"Receive job requests from landlords and PMs in your area. Accept with one tap."</p>
                    </div>
                    <div class="mktg-str-card" style="text-align:center;padding:2rem;">
                        <div style="width:48px;height:48px;border-radius:50%;background:rgba(255,107,53,.15);border:2px solid #ff6b35;display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
                            <span class="material-symbols-outlined" style="color:#ff6b35;font-variation-settings:'FILL' 1">"payments"</span>
                        </div>
                        <h3 style="font-size:1rem;font-weight:600;margin-bottom:.5rem;">"3. Invoice & get paid"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"Submit your invoice in the app. Payment hits your account within 24 hours."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Features ─────────────────────────────────────────────────────────────────

#[component]
fn VendorFeatures() -> impl IntoView {
    view! {
        <section id="vendor-features" class="mktg-section" style="background:rgba(6,214,160,.03);border-top:1px solid rgba(6,214,160,.1);border-bottom:1px solid rgba(6,214,160,.1);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Features"</p>
                <h2 class="mktg-section-h2">"Built for tradespeople, not accountants."</h2>
                <div class="mktg-feature-grid">
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"search"</span>
                        <h3>"Marketplace listing"</h3>
                        <p>"Your business profile is visible to every landlord and PM on the Folio platform. Free, forever."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"assignment"</span>
                        <h3>"Job dispatch"</h3>
                        <p>"Receive job requests with photos, descriptions, and property details. Accept or decline in one tap."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"receipt"</span>
                        <h3>"Instant invoicing"</h3>
                        <p>"Create and send invoices from your phone in 60 seconds. No spreadsheets. No chasing."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"account_balance_wallet"</span>
                        <h3>"Fast payment"</h3>
                        <p>"Get paid directly via ACH. No more waiting 30+ days for a check in the mail."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"star"</span>
                        <h3>"Reviews & ratings"</h3>
                        <p>"Build your reputation on the platform. Highly-rated vendors get priority placement and more jobs."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"bar_chart"</span>
                        <h3>"Job analytics"</h3>
                        <p>"Track your earnings, completed jobs, average ticket size, and client retention over time."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn VendorPricing() -> impl IntoView {
    view! {
        <section id="vendor-pricing" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Pricing"</p>
                <h2 class="mktg-section-h2">"Free to start. Upgrade when you're ready."</h2>
                <p class="mktg-section-sub" style="max-width:520px;margin:0 auto 2.5rem;">"No subscription required to accept jobs. Pay only when you want priority placement and extra features."</p>
                <div class="mktg-pricing-grid" style="grid-template-columns:repeat(3,1fr);">

                    // ── Basic listing — free ───────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Basic"</span>
                        <div class="mktg-pricing-price">"$0"</div>
                        <div class="mktg-pricing-sub">"Free forever"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Marketplace profile"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Accept & complete jobs"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"In-platform invoicing"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"ACH payment"</li>
                        </ul>
                        <a href="#vendor-signup" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="vendor-pricing-basic">"Join free"</a>
                    </div>

                    // ── Pro Vendor (FEATURED) ──────────────────────────────
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Pro Vendor"</span>
                        <div class="mktg-pricing-price">"$29"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Priority placement"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Basic"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Priority search placement"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Auto-invoicing templates"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Job analytics dashboard"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Verified badge"</li>
                        </ul>
                        <a href="#vendor-signup" class="mktg-pricing-btn mktg-pricing-btn-accent" id="vendor-pricing-pro">"Get early access"</a>
                    </div>

                    // ── Business ───────────────────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Business"</span>
                        <div class="mktg-pricing-price">"$79"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"0% platform fee"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Pro Vendor"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"0% platform fee on jobs"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Multi-tech accounts"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Branded company profile"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Dedicated account manager"</li>
                        </ul>
                        <a href="#vendor-signup" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="vendor-pricing-business">"Get early access"</a>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn VendorCta() -> impl IntoView {
    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#f59e0b;">"Open to all trades"</p>
                <h2 class="mktg-cta-h2">"Stop waiting for referrals. Start getting jobs."</h2>
                <p class="mktg-cta-sub">
                    "Join the Folio vendor marketplace and get connected to property managers \
                     and landlords who need your services — starting today."
                </p>
                <a href="#vendor-signup" class="mktg-btn-accent mktg-btn-lg" id="vendor-cta-btn">"Join the marketplace →"</a>
                <p style="margin-top:16px;font-size:12px;color:#9ca3af;">"Free to join. No credit card required."</p>
            </div>
        </section>
    }
}

// ── Beta program callout strip ────────────────────────────────────────────────

#[component]
fn BetaCalloutStrip() -> impl IntoView {
    view! {
        <div class="mktg-section-inner">
            <div class="beta-callout-strip">
                <span class="material-symbols-outlined beta-callout-strip-icon"
                      style="font-variation-settings:'FILL' 1">"science"</span>
                <div class="beta-callout-text">
                    <strong>"Apply for the Folio Beta Program"</strong>
                    <p>"Get free access during beta in exchange for real feedback. We review every                        application — accepted members shape the product roadmap."</p>
                </div>
                <a href="/beta" class="beta-callout-cta" id="beta-strip-cta">
                    "Apply now"
                    <span class="material-symbols-outlined" style="font-size:16px">"arrow_forward"</span>
                </a>
            </div>
        </div>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn VendorFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS · Vendor Marketplace"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/">"For Landlords"</a>
                    <a href="/property-managers">"For Property Managers"</a>
                    <a href="/brokers">"For Brokers"</a>
                    <a href="/cohost-market">"Cohost Network"</a>
                    <a href="/login">"Sign in"</a>
                </div>
                <div class="mktg-footer-legal">
                    "© 2026 Folio · Atlas Platform · "
                    <a href="/legal/privacy">"Privacy"</a>
                    " · "
                    <a href="/legal/terms">"Terms"</a>
                </div>
            </div>
        </footer>
    }
}
