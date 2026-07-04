//! MarketLandingPage — SSR landing page for Folio.
//!
//! Served at:
//!   /              → marketing homepage (unauthenticated visitors via HomeDispatch)
//!   /lp            → direct product page
//!   /lp/:variant_slug → geo/market variant
//!
//! This component is **zero-auth** — no session cookie required. It is accessible
//! to any visitor, crawler, or CDN edge worker.
//!
//! # Geo variant selection
//! On SSR render, `get_visitor_geo()` reads the Cloudflare `CF-IPCountry` header
//! and selects the appropriate `app_page_variant` slug. The variant slug and
//! country code are embedded as data attributes on the waitlist form so the
//! client can pass them in the `POST /api/pub/products/folio/waitlist` payload
//! without an additional round-trip.
//!
//! # Waitlist flow
//! The inline hero form submits to `/api/pub/products/folio/waitlist` which
//! creates an `atlas_lead` record (G-31) via `join_waitlist_inner`. No new
//! migration is required — this endpoint is already deployed.
//!
//! # Launch mode gating
//! | Mode       | CTA                                          |
//! |------------|----------------------------------------------|
//! | Waitlist   | 3-step inline hero form (this design)        |
//! | Active     | Sign up / onboarding link                    |
//! | PreOrder   | Stripe Checkout button                       |
//! | PreLaunch  | Coming soon (no conversion form)             |
//! | Draft      | NotFound                                     |

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::geo::{VisitorGeo, get_visitor_geo};
use crate::pages::not_found::NotFound;

// ── Page data types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LaunchMode {
    Active,
    Waitlist,
    PreOrder,
    PreLaunch,
    Draft,
}

impl Default for LaunchMode {
    fn default() -> Self { Self::Draft }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HreflangEntry {
    pub locale: String,
    pub url:    String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PixelSnippet {
    pub pixel_type: String,
    pub snippet:    String,
    pub inject_at:  String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LandingPageData {
    pub product_slug:       String,
    pub variant_slug:       Option<String>,
    pub launch_mode:        LaunchMode,
    pub product_name:       String,
    /// Tagline / subtitle (not always returned by backend master endpoint)
    #[serde(default)]
    pub tagline:            Option<String>,
    pub meta_title:         Option<String>,
    pub meta_description:   Option<String>,
    pub og_image_url:       Option<String>,
    pub canonical_url:      Option<String>,
    pub structured_data:    Option<serde_json::Value>,
    pub cta_label:          String,
    pub cta_action:         String,
    #[serde(rename = "hero")]
    pub hero_payload:       serde_json::Value,
    #[serde(rename = "blocks")]
    pub blocks_payload:     serde_json::Value,
    #[serde(default)]
    pub hreflang:           Vec<HreflangEntry>,
    #[serde(default)]
    pub pixels:             Vec<PixelSnippet>,
    pub city:               Option<String>,
    /// Region label (returned by variant endpoint, absent on master)
    #[serde(default)]
    pub region:             Option<String>,
    #[serde(default = "default_locale")]
    pub locale:             String,
}

fn default_locale() -> String { "en-US".to_string() }

// ── Server function ───────────────────────────────────────────────────────────

#[server(LoadLandingPage, "/api")]
pub async fn load_landing_page(
    variant_slug: Option<String>,
) -> Result<LandingPageData, server_fn::error::ServerFnError> {
    const PRODUCT_SLUG: &str = "folio";
    let path = match &variant_slug {
        Some(v) if !v.is_empty() => format!("/api/pub/products/{PRODUCT_SLUG}/{v}"),
        _                        => format!("/api/pub/products/{PRODUCT_SLUG}"),
    };
    crate::atlas_client::fetch::<LandingPageData>(&path)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(
            format!("Landing page load failed: {e}")
        ))
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn MarketLandingPage() -> impl IntoView {
    let params       = use_params_map();
    let variant_slug = move || params.with(|p| p.get("variant_slug"));

    let page = Resource::new(variant_slug, |slug| load_landing_page(slug));
    let geo  = Resource::new(|| (), |_| get_visitor_geo());

    view! {
        <Suspense fallback=|| view! { <LandingPageSkeleton/> }>
            {move || {
                let geo_data = geo.get()
                    .and_then(|r| r.ok())
                    .unwrap_or_default();

                page.get().map(|result| match result {
                    Err(_)   => view! { <NotFound/> }.into_any(),
                    Ok(data) => match data.launch_mode {
                        LaunchMode::Draft => view! { <NotFound/> }.into_any(),
                        _                 => view! {
                            <FolioLandingFull data=data geo=geo_data/>
                        }.into_any(),
                    },
                })
            }}
        </Suspense>
    }
}

// ── Shell — head metadata + full page ────────────────────────────────────────

#[component]
fn FolioLandingFull(data: LandingPageData, geo: VisitorGeo) -> impl IntoView {
    let title       = data.meta_title.clone().unwrap_or_else(|| "Folio — Modern Landlord OS".to_string());
    let description = data.meta_description.clone()
        .unwrap_or_else(|| "The only property management platform built for independent landlords. LTR + STR + payments + compliance — one login.".to_string());
    let og_image  = data.og_image_url.clone().unwrap_or_default();
    let canonical = data.canonical_url.clone().unwrap_or_default();
    let jsonld    = data.structured_data.as_ref()
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_default();

    let head_pixels: Vec<_> = data.pixels.iter()
        .filter(|p| p.inject_at == "head")
        .map(|p| p.snippet.clone())
        .collect();

    let variant_slug  = data.variant_slug.clone().unwrap_or_else(|| geo.variant_slug().to_string());
    let country_code  = geo.country_code.clone();
    let product_slug  = data.product_slug.clone();
    let launch_mode   = data.launch_mode.clone();

    view! {
        // ── <head> ──────────────────────────────────────────────────────────
        <Title text=title.clone()/>
        <Meta name="description"        content=description.clone()/>
        <Meta property="og:title"       content=title.clone()/>
        <Meta property="og:description" content=description/>
        <Meta property="og:image"       content=og_image/>
        <Meta property="og:type"        content="website"/>
        <Meta name="twitter:card"       content="summary_large_image"/>
        {(!canonical.is_empty()).then(|| view! { <Link rel="canonical" href=canonical/> })}
        {data.hreflang.iter().map(|h| view! {
            <Link rel="alternate" hreflang=h.locale.clone() href=h.url.clone()/>
        }).collect_view()}
        {(!jsonld.is_empty()).then(|| view! {
            <script type="application/ld+json">{jsonld}</script>
        })}
        {head_pixels.into_iter().map(|s| view! {
            <script inner_html=s></script>
        }).collect_view()}

        // ── Page body ───────────────────────────────────────────────────────
        <div class="folio-mktg">
            <MktgNav/>
            <MktgHero launch_mode=launch_mode product_slug=product_slug variant_slug=variant_slug country_code=country_code/>
            <MktgStats/>
            <MktgPersonas/>
            <MktgFeatures/>
            <MktgStr/>
            <MktgInternational/>
            <MktgPayments/>
            <MktgPricing/>
            <MktgCta/>
            <MktgFooter/>
            <MktgScripts/>
        </div>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn MktgNav() -> impl IntoView {
    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <a href="/" class="mktg-nav-logo">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </a>
                <div class="mktg-nav-links">
                    <a href="#features">"Features"</a>
                    <a href="#str">"STR"</a>
                    <a href="#pricing">"Pricing"</a>
                    <a href="#international">"International"</a>
                </div>
                <div class="mktg-nav-actions">
                    <a href="/login" class="mktg-btn-signin" id="nav-signin-btn">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </a>
                    <a href="#waitlist-wrap" class="mktg-btn-accent">"Join waitlist"</a>
                </div>
            </div>
        </nav>
    }
}

// ── Hero + waitlist form ──────────────────────────────────────────────────────

#[allow(unused_variables)]
#[component]
fn MktgHero(launch_mode: LaunchMode, product_slug: String, variant_slug: String, country_code: String) -> impl IntoView {
    let _ = launch_mode; // Future: gate CTA on Active/PreLaunch modes
    let waitlist_url = format!("/api/pub/products/{}/waitlist", product_slug);

    // Form step: 0 = email, 1 = details, 2 = success
    let step     = RwSignal::new(0u8);
    let email    = RwSignal::new(String::new());
    let role     = RwSignal::new(String::new());
    let size     = RwSignal::new(String::new());
    let source   = RwSignal::new(String::new());
    let phone    = RwSignal::new(String::new());
    let position = RwSignal::new(247u32);
    let err_msg  = RwSignal::new(String::new());
    let loading  = RwSignal::new(false);

    // Step 1 validation — no event argument, called from both click and keydown
    let validate_and_next = move || {
        let e = email.get();
        if e.is_empty() || !e.contains('@') || !e.contains('.') {
            err_msg.set("Please enter a valid email address.".to_string());
            return;
        }
        err_msg.set(String::new());
        step.set(1);
        // Scroll the waitlist card into view so the user sees the details form
        #[cfg(feature = "hydrate")]
        {
            let _ = js_sys::eval(
                "(function(){\
                    var el = document.getElementById('waitlist-wrap');\
                    if(el) el.scrollIntoView({behavior:'smooth',block:'start'});\
                })()"
            );
        }
    };

    let wl_url = waitlist_url.clone();
    let vs     = variant_slug.clone();

    let submit_step2 = {
        let wl_url2 = wl_url.clone();
        let vs2     = vs.clone();
        move |_| {
            if loading.get() { return; }
            loading.set(true);
            let url  = wl_url2.clone();
            let vs3  = vs2.clone();
            let e    = email.get();
            let r    = role.get();
            let s    = size.get();
            let src  = source.get();
            let p    = phone.get();
            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "email":               e,
                    "role":                if r.is_empty() { serde_json::Value::Null } else { r.into() },
                    "portfolio_size_label": if s.is_empty() { serde_json::Value::Null } else { s.into() },
                    "phone":               if p.is_empty() { serde_json::Value::Null } else { p.into() },
                    "utm_source":          if src.is_empty() { serde_json::Value::Null } else { src.into() },
                    "variant_slug":        vs3,
                });
                let resp = gloo_net::http::Request::post(&url)
                    .header("Content-Type", "application/json")
                    .body(body.to_string())
                    .unwrap()
                    .send()
                    .await;
                if let Ok(r) = resp {
                    if let Ok(json) = r.json::<serde_json::Value>().await {
                        if let Some(pos) = json.get("position").and_then(|v| v.as_u64()) {
                            position.set(pos as u32);
                        }
                    }
                }
                loading.set(false);
                step.set(2);
            });
        }
    };

    let role_pills = ["🏠 Landlord", "💼 Property Manager", "🏨 STR Host", "🏡 Tenant", "🔧 Vendor", "📊 Investor"];
    let size_pills = ["1–5 units", "6–20 units", "21–100 units", "100+ units", "Not applicable"];

    view! {
        <section id="hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"verified"</span>
                    " Modern Landlord OS — US · Canada · Brazil"
                </div>
                <h1 class="mktg-hero-h1">
                    "Your entire portfolio."
                    <span class="mktg-h1-accent">" One login."</span>
                </h1>
                <p class="mktg-hero-sub">
                    "Stop juggling five apps. Folio connects rent collection, leases, maintenance, \
                     STR calendars and local compliance into a single platform built for how you \
                     actually work."
                </p>

                // Waitlist form — 3-step reactive form
                <div id="waitlist-wrap" class="mktg-wl-wrap"
                    data-variant-id=variant_slug
                    data-country=country_code
                >
                    // ── Step 0: email entry ──────────────────────────────────
                    {move || (step.get() == 0).then(|| view! {
                        <div class="mktg-wl-step">
                            <div class="mktg-wl-row">
                                <input
                                    type="email"
                                    class="mktg-wl-email"
                                    placeholder="Enter your email address"
                                    autocomplete="email"
                                    prop:value=move || email.get()
                                    on:input=move |ev| email.set(event_target_value(&ev))
                                    on:keydown=move |ev| {
                                        if ev.key() == "Enter" { validate_and_next(); }
                                    }
                                />
                                <button class="mktg-btn-accent mktg-btn-lg"
                                    on:click=move |_| validate_and_next()
                                >
                                    "Join the waitlist →"
                                </button>
                            </div>
                            {move || (!err_msg.get().is_empty()).then(|| view! {
                                <p class="mktg-wl-err">{err_msg.get()}</p>
                            })}
                            <p class="mktg-wl-count-line">
                                <span class="mktg-wl-count">{move || position.get()}</span>
                                " people already on the list · No spam, ever"
                            </p>
                            <p class="mktg-wl-signin-hint">
                                "Already have access? "
                                <a href="/login" class="mktg-wl-signin-link" id="hero-signin-link">"Sign in →"</a>
                            </p>
                        </div>
                    })}

                    // ── Step 1: details ──────────────────────────────────────
                    {move || (step.get() == 1).then(|| view! {
                        <div class="mktg-wl-step mktg-wl-details">
                            <div class="mktg-wl-card">
                                <p class="mktg-wl-card-head">"Tell us about yourself — takes 30 seconds"</p>

                                // Role pills
                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">"I am a…"</label>
                                    <div class="mktg-pill-row">
                                        {role_pills.iter().map(|r| {
                                            let label   = r.to_string();
                                            let r_val   = label.split_once(' ')
                                                .map(|(_, v)| v.to_string())
                                                .unwrap_or_else(|| label.clone());
                                            let rv_cls  = r_val.clone();
                                            let rv_set  = r_val.clone();
                                            view! {
                                                <button
                                                    class=move || if role.get() == rv_cls {
                                                        "mktg-pill mktg-pill-role selected"
                                                    } else {
                                                        "mktg-pill mktg-pill-role"
                                                    }
                                                    on:click=move |_| role.set(rv_set.clone())
                                                >{label}</button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                // Size pills
                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">"Portfolio size"</label>
                                    <div class="mktg-pill-row">
                                        {size_pills.iter().map(|s| {
                                            let label  = s.to_string();
                                            let sv_cls = label.clone();
                                            let sv_set = label.clone();
                                            view! {
                                                <button
                                                    class=move || if size.get() == sv_cls {
                                                        "mktg-pill mktg-pill-size selected"
                                                    } else {
                                                        "mktg-pill mktg-pill-size"
                                                    }
                                                    on:click=move |_| size.set(sv_set.clone())
                                                >{label}</button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>

                                // Source
                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">"How did you hear about Folio?"</label>
                                    <select class="mktg-wl-select"
                                        on:change=move |ev| source.set(event_target_value(&ev))
                                    >
                                        <option value="">"Select one…"</option>
                                        <option value="social">"Social media"</option>
                                        <option value="friend">"Friend or colleague"</option>
                                        <option value="search">"Google / Search"</option>
                                        <option value="podcast">"Podcast or YouTube"</option>
                                        <option value="newsletter">"Newsletter"</option>
                                        <option value="other">"Other"</option>
                                    </select>
                                </div>

                                // Phone (optional)
                                <div class="mktg-wl-field">
                                    <label class="mktg-wl-label">
                                        "Phone "
                                        <span class="mktg-wl-optional">"(optional — SMS launch announcement)"</span>
                                    </label>
                                    <input type="tel" class="mktg-wl-input" placeholder="+1 (305) 000-0000"
                                        prop:value=move || phone.get()
                                        on:input=move |ev| phone.set(event_target_value(&ev))
                                    />
                                </div>
                            </div>
                            <button class="mktg-btn-green mktg-btn-lg mktg-btn-full"
                                disabled=move || loading.get()
                                on:click=submit_step2.clone()
                            >
                                <span class="material-symbols-outlined" style="font-size:20px;font-variation-settings:'FILL' 1">"check_circle"</span>
                                {move || if loading.get() { "Submitting…" } else { "Secure my spot" }}
                            </button>
                        </div>
                    })}

                    // ── Step 2: success ──────────────────────────────────────
                    {move || (step.get() == 2).then(|| view! {
                        <div class="mktg-wl-success">
                            <div class="mktg-success-icon">
                                <span class="material-symbols-outlined" style="font-size:36px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            </div>
                            <h3 class="mktg-success-h3">"You're on the list!"</h3>
                            <p class="mktg-success-sub">"Check your inbox for a confirmation. We'll email you the moment early access opens."</p>
                            <div class="mktg-success-card">
                                <div>
                                    <div class="mktg-success-label">"Your position"</div>
                                    <div class="mktg-success-num">"#"{move || position.get()}</div>
                                </div>
                                <div class="mktg-success-div"></div>
                                <div>
                                    <div class="mktg-success-share-text">
                                        "Share with a landlord friend"<br/>"and move up the list."
                                    </div>
                                    <button class="mktg-success-share-btn" id="mktg-share-btn">
                                        <span class="material-symbols-outlined" style="font-size:14px">"share"</span>
                                        " Share Folio"
                                    </button>
                                </div>
                            </div>
                        </div>
                    })}
                </div>

                // Proof strip (visible only on step 0)
                {move || (step.get() == 0).then(|| view! {
                    <div class="mktg-proof-strip">
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "No setup fee"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "No contracts"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "LTR + STR in one platform"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "US · Canada · Brazil & beyond"
                        </span>
                    </div>
                })}
            </div>
        </section>
    }
}

// ── Stats band ────────────────────────────────────────────────────────────────

#[component]
fn MktgStats() -> impl IntoView {
    view! {
        <section class="mktg-stats">
            {[
                ("34+", "PM generics deployed"),
                ("7",   "role portals"),
                ("24+", "API handler files"),
                ("3",   "countries at launch"),
            ].iter().map(|(val, label)| view! {
                <div class="mktg-stat">
                    <span class="mktg-stat-val">{*val}</span>
                    <span class="mktg-stat-label">{*label}</span>
                </div>
            }).collect_view()}
        </section>
    }
}

// ── Personas ──────────────────────────────────────────────────────────────────

#[component]
fn MktgPersonas() -> impl IntoView {
    let personas = vec![
        ("🏠", "Independent Landlord", "coral", "1–20 units", vec![
            "Portfolio overview & analytics",
            "Automated rent reminders",
            "Lease templates & e-sign",
            "Maintenance dispatch",
        ]),
        ("💼", "Property Manager", "teal", "Any size", vec![
            "Multi-client portfolio",
            "Owner statement exports",
            "Branded tenant portal",
            "Commission split tracking",
        ]),
        ("🏨", "STR Host", "gold", "Airbnb + direct", vec![
            "Unified calendar",
            "Channel sync",
            "Guest messaging",
            "STR licensing & compliance",
        ]),
        ("🏡", "Tenant", "green", "Renter portal", vec![
            "Pay rent online",
            "Submit maintenance requests",
            "View & sign lease",
            "Track move-in docs",
        ]),
        ("🔧", "Vendor / Contractor", "orange", "Work order portal", vec![
            "Receive job dispatches",
            "Submit invoices",
            "Schedule management",
            "Marketplace profile",
        ]),
        ("📊", "Owner / Investor", "purple", "Read-only portal", vec![
            "Equity dashboard",
            "Property statements",
            "Distribution history",
            "Maintenance approvals",
        ]),
    ];

    view! {
        <section id="personas" class="mktg-section mktg-personas">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Built for every role"</p>
                <h2 class="mktg-section-h2">"One platform. Every person in the deal."</h2>
                <p class="mktg-section-sub">"Folio issues role-based portals so landlords, tenants, vendors, owners, and managers each see exactly what they need — nothing more."</p>
                <div class="mktg-personas-scroll">
                    {personas.into_iter().map(|(icon, name, accent, sub, bullets)| view! {
                        <div class=format!("mktg-persona-card mktg-accent--{}", accent)>
                            <div class="mktg-persona-icon">{icon}</div>
                            <h3 class="mktg-persona-name">{name}</h3>
                            <p class="mktg-persona-sub">{sub}</p>
                            <ul class="mktg-persona-bullets">
                                {bullets.into_iter().map(|b| view! {
                                    <li class="mktg-persona-bullet">
                                        <span class="material-symbols-outlined" style="font-size:13px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>
                                        {b}
                                    </li>
                                }).collect_view()}
                            </ul>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── Feature grid ──────────────────────────────────────────────────────────────

#[component]
fn MktgFeatures() -> impl IntoView {
    let cells = vec![
        ("payments", "Unified rent collection", "ACH, card, and international rails. Auto-split between ledger categories. No stripe dashboard required."),
        ("description", "Lease lifecycle", "Draft, negotiate, e-sign, renew, and archive. Templates handle state-specific disclosures automatically."),
        ("build", "Maintenance dispatch", "Tenants submit, landlords approve, vendors receive and invoice — all tracked in one thread."),
        ("calendar_month", "STR calendar sync", "Airbnb, VRBO, Booking.com and direct booking in one drag-and-drop calendar. No double-bookings."),
        ("verified_user", "Compliance engine", "STR licensing, fair housing checks, and local regulatory registration — tracked and renewed automatically."),
        ("analytics", "Portfolio analytics", "Live metrics across all units: occupancy, NOI, vacancy days, maintenance cost per unit."),
        ("campaign", "Vacancy campaigns", "Post to listing networks, track lead pipeline, run applications, and convert to lease — one workflow."),
        ("groups", "Vendor marketplace", "Find and hire vetted contractors. Rate them. They rate you. Work orders and invoices stay in the platform."),
        ("language", "Multi-market", "US (all states) · Canada (ON/BC/QC) · Brazil (LGPD-compliant) · more markets on the roadmap."),
    ];

    view! {
        <section id="features" class="mktg-section mktg-features">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Platform generics"</p>
                <h2 class="mktg-section-h2">"Everything you need. Nothing you don't."</h2>
                <div class="mktg-feature-grid">
                    {cells.into_iter().map(|(icon, title, desc)| view! {
                        <div class="mktg-feature-cell">
                            <span class="material-symbols-outlined mktg-feature-icon">{icon}</span>
                            <h3 class="mktg-feature-title">{title}</h3>
                            <p class="mktg-feature-desc">{desc}</p>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── STR unlock ───────────────────────────────────────────────────────────────

#[component]
fn MktgStr() -> impl IntoView {
    view! {
        <section id="str" class="mktg-section mktg-str-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow mktg-eyebrow-light">"Short-term rentals"</p>
                <h2 class="mktg-section-h2 mktg-h2-light">"Your STR, fully unlocked."</h2>
                <p class="mktg-section-sub mktg-sub-light">"Most landlord software treats STR as an afterthought. Folio treats it as a first-class product."</p>
                <div class="mktg-str-grid">
                    {[
                        ("calendar_month", "Unified calendar", "All channels in one drag-and-drop view. Block dates, set minimums, sync instantly."),
                        ("verified_user",  "Compliance first", "STR license tracking, renewal reminders, and local regulatory filings — built in."),
                        ("payments",       "Direct booking",   "Collect deposits, damage holds, and nightly rates without third-party fee stacks."),
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
    }
}

// ── International ────────────────────────────────────────────────────────────

#[component]
fn MktgInternational() -> impl IntoView {
    let markets = vec![
        ("🇺🇸", "United States", "All 50 states · Federal fair housing · ACH + card"),
        ("🇨🇦", "Canada", "ON · BC · QC · PIPEDA-compliant · EFT rails"),
        ("🇧🇷", "Brazil", "LGPD-compliant · PIX payment rail · Curitiba + São Paulo"),
        ("🌎", "More markets", "Latin America expansion Q3 2026 · EU planned"),
    ];

    view! {
        <section id="international" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Global reach"</p>
                <h2 class="mktg-section-h2">"Built for the Americas. Ready for the world."</h2>
                <p class="mktg-section-sub">"Folio handles multi-currency ledgers, local compliance rules, and payment rails specific to each country — so you don't have to."</p>
                <div class="mktg-market-grid">
                    {markets.into_iter().map(|(flag, name, desc)| view! {
                        <div class="mktg-market-card">
                            <span class="mktg-market-flag">{flag}</span>
                            <div>
                                <div class="mktg-market-name">{name}</div>
                                <div class="mktg-market-desc">{desc}</div>
                            </div>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── Payments ─────────────────────────────────────────────────────────────────

#[component]
fn MktgPayments() -> impl IntoView {
    let rails = vec![
        ("💳", "ACH / EFT", "US and Canada bank transfers. 1–2 business day settlement."),
        ("⚡", "PIX",        "Brazil's instant payment rail. Settlement in seconds."),
        ("💰", "Card",       "Visa, Mastercard, Amex. Tenant pays the processing fee."),
        ("🏦", "Ledger",     "Every transaction split by category. Export-ready for your accountant."),
    ];

    view! {
        <section class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"G-03 Atlas Payments"</p>
                <h2 class="mktg-section-h2">"Rent collected. Split. Reported."</h2>
                <p class="mktg-section-sub">"The unified ledger handles every rail — ACH, card, PIX — and automatically splits payments between principal, fees, and reserves."</p>
                <div class="mktg-rail-grid">
                    {rails.into_iter().map(|(icon, name, desc)| view! {
                        <div class="mktg-rail-card">
                            <span class="mktg-rail-icon">{icon}</span>
                            <div class="mktg-rail-name">{name}</div>
                            <div class="mktg-rail-desc">{desc}</div>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn MktgPricing() -> impl IntoView {
    view! {
        <section id="pricing" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Pricing"</p>
                <h2 class="mktg-section-h2">"Simple. Transparent. No surprises."</h2>
                <div class="mktg-pricing-grid">
                    // Free
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Starter"</span>
                        <div class="mktg-pricing-price">"$0"</div>
                        <div class="mktg-pricing-sub">"Up to 3 units, forever free"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"1 landlord portal"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Lease management"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Maintenance requests"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Tenant portal"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost">"Join waitlist"</a>
                    </div>
                    // Pro
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Landlord OS"</span>
                        <div class="mktg-pricing-price">"$29"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Per portfolio, unlimited units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Starter"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"STR calendar & channels"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"ACH rent collection"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Vacancy campaigns"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Vendor marketplace"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Portfolio analytics"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-accent">"Join waitlist"</a>
                    </div>
                    // Enterprise
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Enterprise / PMC"</span>
                        <div class="mktg-pricing-price">"Custom"</div>
                        <div class="mktg-pricing-sub">"Property managers & brokerages"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Everything in Landlord OS"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Multi-client portfolio"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Owner portals & statements"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Brokerage mode (agents + brokers)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Dedicated onboarding"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost">"Contact us"</a>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn MktgCta() -> impl IntoView {
    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <h2 class="mktg-cta-h2">"Ready to simplify your portfolio?"</h2>
                <p class="mktg-cta-sub">"Join the waitlist. Be among the first landlords to get access."</p>
                <a href="#waitlist-wrap" class="mktg-btn-accent mktg-btn-lg">"Join the waitlist →"</a>
            </div>
        </section>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn MktgFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/login">"Sign in"</a>
                    <a href="#pricing">"Pricing"</a>
                    <a href="#features">"Features"</a>
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

// ── Inline JS (nav scroll + share button) ────────────────────────────────────

#[component]
fn MktgScripts() -> impl IntoView {
    let js = r#"
(function() {
  // Nav scroll: darken slightly on scroll (page is always dark)
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
  // Share button
  var shareBtn = document.getElementById('mktg-share-btn');
  if (shareBtn) {
    shareBtn.addEventListener('click', function() {
      var url = window.location.href.split('?')[0];
      var text = "I just joined the waitlist for Folio — the property management OS built for independent landlords. Check it out:";
      if (navigator.share) {
        navigator.share({ title: 'Folio — Modern Landlord OS', text: text, url: url });
      } else {
        navigator.clipboard.writeText(url).then(function() {
          shareBtn.textContent = '✓ Link copied!';
          setTimeout(function() { shareBtn.innerHTML = '<span class="material-symbols-outlined" style="font-size:14px">share</span> Share Folio'; }, 2000);
        });
      }
    });
  }
})();
    "#;
    view! { <script>{js}</script> }
}

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn LandingPageSkeleton() -> impl IntoView {
    view! {
        <div class="folio-mktg mktg-skeleton-page" aria-busy="true">
            <div class="mktg-hero mktg-hero--skeleton">
                <div class="mktg-skeleton mktg-sk-h1"></div>
                <div class="mktg-skeleton mktg-sk-sub"></div>
                <div class="mktg-skeleton mktg-sk-btn"></div>
            </div>
        </div>
    }
}
