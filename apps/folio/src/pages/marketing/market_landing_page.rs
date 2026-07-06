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
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::geo::{VisitorGeo, get_visitor_geo};
use crate::pages::not_found::NotFound;
use crate::components::lang::{LanguageSwitcher, get_current_lang};

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
        .unwrap_or_else(|| "The only property management platform built for independent landlords. Collect rent, manage leases, handle maintenance, and run vacation rentals — one login.".to_string());
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
            <MktgTenantPortal/>
            <MktgStr/>
            <MktgAppPreview/>
            <MktgInternational/>
            <MktgPayments/>
            <MktgPricing/>
            <MktgCta/>
            <BetaCalloutStrip/>
            <MktgFooter/>
            <MktgScripts/>
        </div>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn MktgNav() -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let role_open = RwSignal::new(false);
    let lang_res  = Resource::new(|| (), |_| get_current_lang());
    // Close role dropdown on any outside click
    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <A href="/" attr:class="mktg-nav-logo">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </A>
                // ── Desktop links ──────────────────────────────────────────
                <div class="mktg-nav-links">
                    <a href="#features">"Features"</a>
                    <a href="#tenant-portal">"Tenant Portal"</a>
                    <a href="#str">"Vacation Rentals"</a>
                    <a href="#app-preview">"How it works"</a>
                    <A href="/cohost-market">"Cohost Network"</A>
                    <a href="#pricing">"Pricing"</a>
                    // ── Role dropdown ────────────────────────────────────
                    <div class="mktg-nav-role-dropdown">
                        <button
                            class="mktg-nav-role-trigger"
                            aria-expanded=move || role_open.get().to_string()
                            aria-label="Select your role"
                            on:click=move |e| { e.stop_propagation(); role_open.update(|o| *o = !*o); }
                        >
                            "For you"
                            <span class=move || if role_open.get() {
                                "mktg-nav-role-arrow mktg-nav-role-arrow--open"
                            } else {
                                "mktg-nav-role-arrow"
                            }>
                                <span class="material-symbols-outlined" style="font-size:15px">"expand_more"</span>
                            </span>
                        </button>
                        <div class=move || if role_open.get() {
                            "mktg-nav-role-panel mktg-nav-role-panel--open"
                        } else {
                            "mktg-nav-role-panel"
                        }>
                            <A href="/" attr:class="mktg-nav-role-item mktg-nav-role-item--active" on:click=move |_| role_open.set(false)>
                                <span class="mktg-nav-role-icon">"🏠"</span>
                                "For Landlords"
                            </A>
                            <A href="/property-managers" attr:class="mktg-nav-role-item" on:click=move |_| role_open.set(false)>
                                <span class="mktg-nav-role-icon">"🏢"</span>
                                "For Property Managers"
                            </A>
                            <A href="/brokers" attr:class="mktg-nav-role-item" on:click=move |_| role_open.set(false)>
                                <span class="mktg-nav-role-icon">"🤝"</span>
                                "For Brokers"
                            </A>
                            <A href="/vendors" attr:class="mktg-nav-role-item" on:click=move |_| role_open.set(false)>
                                <span class="mktg-nav-role-icon">"🔧"</span>
                                "For Vendors"
                            </A>
                        </div>
                    </div>
                    <A href="/founding" attr:class="mktg-nav-broker-link">"Founders ✦"</A>
                </div>
                <div class="mktg-nav-actions">
                    // ── Language switcher ──────────────────────────────────
                    <Suspense fallback=|| ()>
                        {move || lang_res.get().and_then(|r| r.ok()).map(|code| view! {
                            <LanguageSwitcher current_lang=code/>
                        })}
                    </Suspense>
                    <A href="/login" attr:class="mktg-btn-signin" attr:id="nav-signin-btn">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </A>
                    <a href="#waitlist-wrap" class="mktg-btn-accent">"Join waitlist"</a>
                    // ── Hamburger (mobile only) ────────────────────────────
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
        // ── Mobile nav drawer ─────────────────────────────────────────────
        <div class=move || if menu_open.get() {
            "mktg-mobile-nav mktg-mobile-nav--open"
        } else {
            "mktg-mobile-nav"
        }>
            <a href="#features"    on:click=move |_| menu_open.set(false)>"Features"</a>
            <a href="#tenant-portal" on:click=move |_| menu_open.set(false)>"Tenant Portal"</a>
            <a href="#str"         on:click=move |_| menu_open.set(false)>"Vacation Rentals"</a>
            <A href="/cohost-market" on:click=move |_| menu_open.set(false)>"Cohost Network"</A>
            <a href="#pricing"     on:click=move |_| menu_open.set(false)>"Pricing"</a>
            <A href="/brokers"     on:click=move |_| menu_open.set(false) attr:class="mktg-mobile-nav-broker">"For Brokers"</A>
            <A href="/property-managers" on:click=move |_| menu_open.set(false)>"For Property Managers"</A>
            <A href="/vendors"    on:click=move |_| menu_open.set(false)>"For Vendors"</A>
            <a href="#waitlist-wrap" on:click=move |_| menu_open.set(false)>"Join waitlist"</a>
            <A href="/founding"      on:click=move |_| menu_open.set(false)>"Founding ✦"</A>
            <A href="/beta"          on:click=move |_| menu_open.set(false)>"Apply for Beta"</A>
        </div>
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

    let role_pills = ["🏠 Landlord", "💼 Property Manager", "🏨 Vacation Rental Host", "🏡 Tenant", "🔧 Vendor", "📊 Investor"];
    let size_pills = ["1–5 units", "6–20 units", "21–100 units", "100+ units", "Not applicable"];

    view! {
        <section id="hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"science"</span>
                    " Beta Access Open · US · Canada · Brazil"
                </div>
                <h1 class="mktg-hero-h1">
                    "The operating system"
                    <span class="mktg-h1-accent"> " for the modern real estate investor."</span>
                </h1>
                <p class="mktg-hero-sub">
                    "Your rental business runs on gut feel, spreadsheets, and three apps that don’t talk to each other. \
                     Folio is the purpose-built platform that handles rent, leases, maintenance, vacation rentals, \
                     and compliance — so you can run your properties like the business they actually are."
                </p>

                // Waitlist form — 3-step reactive form
                <div id="waitlist-wrap" class="mktg-wl-wrap"
                    data-variant-id=variant_slug
                    data-country=country_code
                >
                    // ── Step 0: email entry ──────────────────────────────────
                    <Show when=move || step.get() == 0 fallback=|| ()>
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
                                    "Get early access →"
                                </button>
                            </div>
                            <Show when=move || !err_msg.get().is_empty() fallback=|| ()>
                                <p class="mktg-wl-err">{move || err_msg.get()}</p>
                            </Show>
                            <p class="mktg-wl-count-line">
                                <span class="mktg-wl-count">{move || position.get()}</span>
                                " landlords already in line for beta access"
                            </p>
                            <p class="mktg-wl-signin-hint">
                                "Already have access? "
                                <a href="/login" class="mktg-wl-signin-link" id="hero-signin-link">"Sign in →"</a>
                            </p>
                        </div>
                    </Show>

                    // ── Step 1: details ─────────────────────────────────────────
                    <Show when=move || step.get() == 1 fallback=|| ()>
                        <div class="mktg-wl-step mktg-wl-details">
                            <div class="mktg-wl-card">
                            <p class="mktg-wl-card-head">"One more thing — takes 30 seconds"</p>

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
                    </Show>

                    // ── Step 2: success ──────────────────────────────────────
                    <Show when=move || step.get() == 2 fallback=|| ()>
                        <div class="mktg-wl-success">
                            <div class="mktg-success-icon">
                                <span class="material-symbols-outlined" style="font-size:36px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            </div>
                            <h3 class="mktg-success-h3">"You're in! Beta access reserved."</h3>
                            <p class="mktg-success-sub">"Check your inbox for a confirmation. You'll be among the first landlords to access Folio and the Cohost Network when we open the doors."</p>
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
                    </Show>
                </div>

                // Proof strip (visible only on step 0)
                <Show when=move || step.get() == 0 fallback=|| ()>
                    <div class="mktg-proof-strip">
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"science"</span>
                            "Beta — be one of the first"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"verified"</span>
                            "Built by a landlord"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "No setup fee"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "Long-term + vacation rentals"
                        </span>
                        <span class="mktg-proof-sep"></span>
                        <span class="mktg-proof-item">
                            <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "US · Canada · Brazil"
                        </span>
                    </div>
                </Show>
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
                ("5 min", "Average setup time"),
                ("1 login", "For your whole portfolio"),
                ("3",      "Countries at launch"),
                ("$0",     "Setup fee · no contracts"),
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
            "Owner statements & reports",
            "Branded tenant portal",
            "Owner disbursement & fees",
        ]),
        ("🏨", "Vacation Rental Host", "gold", "Airbnb + direct", vec![
            "Unified booking calendar",
            "Channel sync",
            "Guest messaging",
            "Vacation rental licensing & compliance",
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
        ("payments", "Rent collection", "Bank transfer, card, and international payment methods. Automatically records every payment. No separate accounting tool needed."),
        ("description", "Lease management", "Create, send, e-sign, renew, and store leases. Templates cover the required disclosures for your state or country."),
        ("build", "Maintenance tracking", "Tenants report issues, you approve the work, contractors receive the job and send invoices — all in one place."),
        ("calendar_month", "Vacation rental calendar", "Airbnb, VRBO, Booking.com and your own direct bookings in one calendar. No double-bookings, ever."),
        ("verified_user", "Compliance reminders", "Vacation rental permits, fair housing requirements, and local registration renewals — tracked automatically so nothing slips."),
        ("analytics", "Portfolio dashboard", "See your income, vacancies, and maintenance costs across every property at a glance."),
        ("campaign", "Vacancy marketing", "List your vacancy, collect applications, screen tenants, and convert to a signed lease — one workflow."),
        ("groups", "Contractor marketplace", "Find vetted contractors, send them work orders, receive invoices, and leave reviews — all inside Folio."),
        ("language", "Multi-country", "United States · Canada · Brazil — with more countries on the way."),
    ];

    view! {
        <section id="features" class="mktg-section mktg-features">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"What's included"</p>
                <h2 class="mktg-section-h2">"From first lease to tax season — covered."</h2>
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

// ── Tenant portal highlight ───────────────────────────────────────────────────

#[component]
fn MktgTenantPortal() -> impl IntoView {
    view! {
        <section id="tenant-portal" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Tenant experience"</p>
                <h2 class="mktg-section-h2">"Happy tenants pay on time. Give them a portal worth logging into."</h2>
                <p class="mktg-section-sub">
                    "When your tenant logs in, they see their own dashboard — not yours. \
                     They can pay rent, report a problem, sign their lease, and track move-in \
                     documents without calling you. Less back-and-forth for you. Better experience for them."
                </p>
                <div class="mktg-str-grid">
                    {[
                        ("payments",      "Pay rent online",           "Bank transfer, card, or local payment method. Rent hits your account automatically."),
                        ("build",         "Maintenance requests",      "Tenants describe the issue, upload a photo, and you get notified instantly. No more texts."),
                        ("description",   "Lease & documents",         "Tenants can read, sign, and download their lease anytime. No printing. No scanning."),
                        ("notifications", "Move-in checklist",         "Guide tenants through move-in day: what to submit, what to expect, how to reach you."),
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

// ── Vacation rental section ───────────────────────────────────────────────────

#[component]
fn MktgStr() -> impl IntoView {
    view! {
        <section id="str" class="mktg-section mktg-str-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow mktg-eyebrow-light">"Vacation rentals"</p>
                <h2 class="mktg-section-h2 mktg-h2-light">"Your vacation rental, fully under control."</h2>
                <p class="mktg-section-sub mktg-sub-light">
                    "Most landlord software treats vacation rentals as an afterthought. \
                     Folio treats them as a first-class product — one calendar, one inbox, one platform."
                </p>
                <div class="mktg-str-grid">
                    {[
                        ("calendar_month", "Unified booking calendar",
                         "Airbnb, VRBO, Booking.com and your own direct bookings in one drag-and-drop calendar. Block dates, set minimums, sync instantly."),
                        ("verified_user",  "Permits & compliance",
                         "Vacation rental permit tracking, renewal reminders, and local registration filings — so you never get caught operating without a license."),
                        ("payments",       "Collect directly from guests",
                         "Take deposits, damage holds, and nightly rates from guests without paying a middleman's fee stack."),
                    ].iter().map(|(icon, title, desc)| view! {
                        <div class="mktg-str-card">
                            <span class="material-symbols-outlined mktg-str-icon">{*icon}</span>
                            <h3 class="mktg-str-title">{*title}</h3>
                            <p class="mktg-str-desc">{*desc}</p>
                        </div>
                    }).collect_view()}
                    // Cohost Network — live page
                    <a href="/cohost-market" class="mktg-str-card mktg-str-card--cohost" style="display:block;text-decoration:none;cursor:pointer;">
                        <div style="display:flex;align-items:center;gap:8px;margin-bottom:10px;">
                            <span class="material-symbols-outlined mktg-str-icon" style="margin-bottom:0">"handshake"</span>
                            <span style="font-size:10px;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;padding:2px 8px;border-radius:4px;background:rgba(6,214,160,0.15);color:#06d6a0;">"New"</span>
                        </div>
                        <h3 class="mktg-str-title">"Cohost Network"</h3>
                        <p class="mktg-str-desc">
                            "Co-host your vacation rental through Folio's partner network. \
                             Trusted local co-hosts handle check-ins, cleaning, and guest communication \
                             while you stay in full control of your property and your money."
                        </p>
                        <div style="margin-top:10px;font-size:11px;font-weight:700;color:#ff6b35;">
                            "Browse co-hosts →"
                        </div>
                    </a>
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
                <p class="mktg-section-eyebrow">"Rent collection"</p>
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
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 2.5rem;">"Start free. Pay as you grow. Every plan includes the tenant portal and maintenance hub — no hidden add-ons."</p>
                <div class="mktg-pricing-grid">

                    // ── Free — try the product ─────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Free"</span>
                        <div class="mktg-pricing-price">"$0"</div>
                        <div class="mktg-pricing-sub">"Up to 2 units · free forever"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Landlord dashboard"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Lease management"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Tenant portal"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Maintenance requests"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="pricing-free-cta">"Join waitlist"</a>
                    </div>

                    // ── Grow — small landlord scaling up ───────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Grow"</span>
                        <div class="mktg-pricing-price">"$29"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Up to 10 units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Everything in Free"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Rent collection (ACH + card)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Vacancy marketing"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Contractor marketplace"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Basic analytics"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="pricing-grow-cta">"Join waitlist"</a>
                    </div>

                    // ── Pro — active investor (FEATURED) ───────────────────
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Pro"</span>
                        <div class="mktg-pricing-price">"$79"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Up to 30 units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Grow"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Vacation rental calendar & channels"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"STR compliance & permits"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Portfolio analytics"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Multi-country (US, Canada, Brazil)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Priority support"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-accent" id="pricing-pro-cta">"Join waitlist"</a>
                    </div>

                    // ── Investor — full-time investor ──────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Investor"</span>
                        <div class="mktg-pricing-price">"$149"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Unlimited units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Pro"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Cohost Network access"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Co-host revenue share tracking"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Dedicated onboarding"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"API access"</li>
                        </ul>
                        <a href="#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="pricing-investor-cta">"Join waitlist"</a>
                    </div>
                </div>

                // ── Property Manager callout ────────────────────────────────
                <div class="mktg-pricing-pm-callout">
                    <span class="material-symbols-outlined" style="font-size:20px;color:#06d6a0">"business_center"</span>
                    <div>
                        <strong>"Managing properties for clients?"</strong>
                        " Property managers and PMCs get a dedicated plan with owner portals, trust accounting, and multi-portfolio billing. "
                        <a href="/property-managers">"See Property Manager pricing →"</a>
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
                <p class="mktg-section-eyebrow" style="color:#ff6b35;">"Limited beta spots available"</p>
                <h2 class="mktg-cta-h2">"Be one of the first landlords inside."</h2>
                <p class="mktg-cta-sub">
                    "Join the waitlist now and get exclusive early access to Folio \
                     and the Cohost Network before we open to the public. \
                     Beta members help shape the product and lock in founder pricing."
                </p>
                <a href="#waitlist-wrap" class="mktg-btn-accent mktg-btn-lg">"Reserve my beta spot →"</a>
                <p style="margin-top:16px;font-size:12px;color:#9ca3af;">"No credit card. No contracts. Cancel anytime."</p>
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
                <A href="/beta" attr:class="beta-callout-cta" attr:id="beta-strip-cta">
                    "Apply now"
                    <span class="material-symbols-outlined" style="font-size:16px">"arrow_forward"</span>
                </A>
            </div>
        </div>
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
                    <A href="/login">"Sign in"</A>
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
    let js = r##"
(function() {
  // ── Nav scroll: darken on scroll ──────────────────────────────────────────
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

  // ── Share button ──────────────────────────────────────────────────────────
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

  // ── Scroll-spy: sync nav highlight with visible section ───────────────────
  // Maps section IDs to the href value on the corresponding nav <a>
  var SPY_MAP = {
    'features':     '#features',
    'tenant-portal':'#tenant-portal',
    'str':          '#str',
    'app-preview':  '#app-preview',
    'pricing':      '#pricing'
  };

  var navLinks = nav ? nav.querySelectorAll('.mktg-nav-links a[href^="#"]') : [];

  function setActive(href) {
    navLinks.forEach(function(a) {
      if (a.getAttribute('href') === href) {
        a.classList.add('mktg-nav--active');
      } else {
        a.classList.remove('mktg-nav--active');
      }
    });
  }

  // Clear all active markers when near the top (hero section)
  function clearActive() {
    navLinks.forEach(function(a) { a.classList.remove('mktg-nav--active'); });
  }

  if ('IntersectionObserver' in window && nav && navLinks.length) {
    // rootMargin: top=-40% means the element must have scrolled 40% into the
    // viewport before we fire; bottom=-55% cuts off early so we don't mark the
    // next section while still reading the current one.
    var observer = new IntersectionObserver(function(entries) {
      entries.forEach(function(entry) {
        if (entry.isIntersecting) {
          var href = SPY_MAP[entry.target.id];
          if (href) setActive(href);
        }
      });
    }, {
      rootMargin: '-40% 0px -55% 0px',
      threshold: 0
    });

    // Also watch the hero to clear highlights when user scrolls back to top
    var heroObs = new IntersectionObserver(function(entries) {
      if (entries[0] && entries[0].isIntersecting) clearActive();
    }, { threshold: 0.15 });

    Object.keys(SPY_MAP).forEach(function(id) {
      var el = document.getElementById(id);
      if (el) observer.observe(el);
    });

    var hero = document.getElementById('hero');
    if (hero) heroObs.observe(hero);
  }
})();
    "##;
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


// ── App Preview — CSS-only radio tabs, real app shell mockup ─────────────────
/// Six-tab walkthrough of the Folio landlord dashboard using pure CSS radio
/// tabs (no JS / no WASM hydration required).
/// Tabs: Dashboard · Tenants · Listings · Maintenance · Payments · Owner Portal
#[component]
fn MktgAppPreview() -> impl IntoView {
    view! {
        <section class="mktg-section ap-section" id="app-preview">
            <div class="mktg-container">
                <div class="ap-header">
                    <span class="mktg-label">"Inside the platform"</span>
                    <h2 class="mktg-h2">"Purpose-built for how you actually operate"</h2>
                    <p class="mktg-sub">"Not a generic SaaS tool with a landlord skin. Folio was designed ground-up for the realities of owning and managing real property — from first lease to Schedule E."</p>
                </div>

                <div class="asp-outer">
                    <p class="asp-caption">"↓ Click any tab to explore"</p>

                    // All radios MUST come before .asp-tabs and .asp-window
                    // so the CSS sibling selector (#lp-tN:checked ~ .asp-window) works.
                    <input type="radio" name="lp" id="lp-t1" class="asp-radio" checked/>
                    <input type="radio" name="lp" id="lp-t2" class="asp-radio"/>
                    <input type="radio" name="lp" id="lp-t3" class="asp-radio"/>
                    <input type="radio" name="lp" id="lp-t4" class="asp-radio"/>
                    <input type="radio" name="lp" id="lp-t5" class="asp-radio"/>
                    <input type="radio" name="lp" id="lp-t6" class="asp-radio"/>

                    <div class="asp-tabs">
                        <label for="lp-t1" class="asp-tab-label">"🏠 Dashboard"</label>
                        <label for="lp-t2" class="asp-tab-label">"👤 Tenants"</label>
                        <label for="lp-t3" class="asp-tab-label">"🏘 Listings"</label>
                        <label for="lp-t4" class="asp-tab-label">"🔧 Maintenance"</label>
                        <label for="lp-t5" class="asp-tab-label">"💳 Payments"</label>
                        <label for="lp-t6" class="asp-tab-label">"📊 Owner Portal"</label>
                    </div>

                    <div class="asp-window">
                        <div class="asp-chrome-bar">
                            <span class="asp-dot asp-dot-red"></span>
                            <span class="asp-dot asp-dot-yellow"></span>
                            <span class="asp-dot asp-dot-green"></span>
                            <span class="asp-url" id="lp-url-bar">"app.folio.co/l/dashboard"</span>
                        </div>
                        <div class="asp-shell">
                            // ── Sidebar nav ──────────────────────────────────
                            <aside class="asp-sidebar">
                                <div class="asp-sidebar-logo">
                                    "Folio"
                                    <span>"Landlord"</span>
                                </div>
                                <a class="asp-nav-item asp-nav-item--active">
                                    <span class="asp-nav-icon">"🏠"</span>"Dashboard"
                                </a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"🏘"</span>"Portfolio"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"👤"</span>"Tenants"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"📋"</span>"Leases"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"🔧"</span>"Maintenance"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"💳"</span>"Payments"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"📅"</span>"Reservations"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"📊"</span>"Reports"</a>
                            </aside>
                            // ── Tab panels ───────────────────────────────────
                            <main class="asp-main">

                                // TAB 1: Dashboard
                                <div class="asp-panel" data-tab="1">
                                    <div class="asp-page-title">"Welcome back, Sarah!"</div>
                                    <div class="asp-page-sub">"Here's what's happening across your portfolio today."</div>
                                    <div class="asp-stat-grid">
                                        <div class="asp-stat-card">
                                            <div class="asp-stat-icon">"🏠"</div>
                                            <div class="asp-stat-label">"Properties"</div>
                                            <div class="asp-stat-value">"19"</div>
                                        </div>
                                        <div class="asp-stat-card">
                                            <div class="asp-stat-icon">"📋"</div>
                                            <div class="asp-stat-label">"Active Leases"</div>
                                            <div class="asp-stat-value">"18"</div>
                                        </div>
                                        <div class="asp-stat-card">
                                            <div class="asp-stat-icon">"💰"</div>
                                            <div class="asp-stat-label">"Revenue MTD"</div>
                                            <div class="asp-stat-value">"$24.8K"</div>
                                            <div class="asp-stat-delta asp-delta-up">"↑ 4.2%"</div>
                                        </div>
                                        <div class="asp-stat-card">
                                            <div class="asp-stat-icon">"🔧"</div>
                                            <div class="asp-stat-label">"Open Work Orders"</div>
                                            <div class="asp-stat-value">"2"</div>
                                            <div class="asp-stat-delta asp-delta-warn">"1 overdue"</div>
                                        </div>
                                    </div>
                                    <div class="asp-section-hdr">"Recent activity"</div>
                                    <table class="asp-table">
                                        <tbody>
                                            <tr><td>"💰 Rent received"</td><td>"Unit 4B · Marcus Reid"</td><td class="asp-credit">"+$1,850"</td><td class="asp-muted">"2h ago"</td></tr>
                                            <tr><td>"🔧 Work order dispatched"</td><td>"HVAC · Unit 2A"</td><td class="asp-muted">"$340 est."</td><td class="asp-muted">"Yesterday"</td></tr>
                                            <tr><td>"📋 Lease renewed"</td><td>"Unit 7C · Aisha Okonkwo"</td><td class="asp-muted">"12 mo"</td><td class="asp-muted">"2 days ago"</td></tr>
                                            <tr><td>"⚠️ Late notice sent"</td><td>"Unit 9A · Tom Rivas"</td><td class="asp-debit">"$1,240"</td><td class="asp-muted">"3 days ago"</td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 2: Tenants
                                <div class="asp-panel" data-tab="2">
                                    <div class="asp-page-title">"Tenant — Marcus Reid"</div>
                                    <div class="asp-page-sub">"Unit 4B · 182 Oak St · Lease ends Dec 31, 2026"</div>
                                    <div style="display:flex;align-items:flex-start;gap:.85rem;margin-bottom:1rem;">
                                        <div class="asp-avatar asp-avatar-lg" style="background:rgba(99,102,241,.2);color:#818cf8;">"MR"</div>
                                        <div style="flex:1;">
                                            <div style="display:flex;gap:.35rem;flex-wrap:wrap;margin-bottom:.5rem;">
                                                <span class="asp-status asp-status--green">"✓ ID Verified"</span>
                                                <span class="asp-status asp-status--green">"✓ Background Clear"</span>
                                                <span class="asp-status asp-status--blue">"On-time payer"</span>
                                            </div>
                                            <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                                <div class="asp-stat-card"><div class="asp-stat-label">"Monthly rent"</div><div class="asp-stat-value" style="font-size:.9rem;">"$1,850"</div></div>
                                                <div class="asp-stat-card"><div class="asp-stat-label">"Tenancy"</div><div class="asp-stat-value" style="font-size:.9rem;">"18 mo"</div></div>
                                                <div class="asp-stat-card"><div class="asp-stat-label">"On-time rate"</div><div class="asp-stat-value" style="font-size:.9rem;">"100%"</div></div>
                                                <div class="asp-stat-card"><div class="asp-stat-label">"Score"</div><div class="asp-stat-value" style="font-size:.9rem;color:#818cf8;">"94"</div></div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="asp-section-hdr">"Payment history"</div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Date"</th><th>"Description"</th><th>"Status"</th><th>"Amount"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"Jul 1, 2026"</td><td>"Monthly rent"</td><td><span class="asp-status asp-status--green">"Paid"</span></td><td class="asp-credit">"$1,850"</td></tr>
                                            <tr><td>"Jun 1, 2026"</td><td>"Monthly rent"</td><td><span class="asp-status asp-status--green">"Paid"</span></td><td class="asp-credit">"$1,850"</td></tr>
                                            <tr><td>"May 1, 2026"</td><td>"Monthly rent"</td><td><span class="asp-status asp-status--green">"Paid"</span></td><td class="asp-credit">"$1,850"</td></tr>
                                            <tr><td>"Apr 1, 2026"</td><td>"Monthly rent"</td><td><span class="asp-status asp-status--green">"Paid"</span></td><td class="asp-credit">"$1,850"</td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 3: Listings
                                <div class="asp-panel" data-tab="3">
                                    <div class="asp-page-title">"Portfolio"</div>
                                    <div class="asp-page-sub">"19 units · 94.7% occupancy · 1 vacant"</div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Unit"</th><th>"Type"</th><th>"Tenant / Status"</th><th>"Rent"</th><th>"Status"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"182 Oak St · 4B"</td><td><span class="asp-status asp-status--blue">"LTR"</span></td><td>"Marcus Reid"</td><td>"$1,850"</td><td><span class="asp-status asp-status--green">"Occupied"</span></td></tr>
                                            <tr><td>"45 Beach Ave · 1"</td><td><span class="asp-status asp-status--gray">"STR"</span></td><td>"Next: Chen family Aug 3"</td><td>"$285/night"</td><td><span class="asp-status asp-status--green">"89% booked"</span></td></tr>
                                            <tr><td>"77 Maple Dr · 9A"</td><td><span class="asp-status asp-status--blue">"LTR"</span></td><td>"Tom Rivas"</td><td>"$1,240"</td><td><span class="asp-status asp-status--warn">"Rent overdue"</span></td></tr>
                                            <tr><td>"14 Elm Ct · 2"</td><td><span class="asp-status asp-status--blue">"LTR"</span></td><td>"Vacant"</td><td>"$1,650"</td><td><span class="asp-status asp-status--gray">"Available Aug 1"</span></td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 4: Maintenance
                                <div class="asp-panel" data-tab="4">
                                    <div class="asp-page-title">"Maintenance"</div>
                                    <div class="asp-page-sub">"2 open · 1 awaiting invoice approval · 3 closed this month"</div>
                                    <div class="asp-card">
                                        <div class="asp-card-row">
                                            <div>
                                                <div style="display:flex;gap:.35rem;align-items:center;margin-bottom:.3rem;">
                                                    <span class="asp-status asp-status--warn">"Open"</span>
                                                    <span class="asp-muted" style="font-size:.68rem;">"#WO-2841 · Jul 3"</span>
                                                    <span class="asp-status asp-status--red">"High priority"</span>
                                                </div>
                                                <div class="asp-card-title">"HVAC not cooling · Unit 2A"</div>
                                                <div class="asp-card-sub">"Arctic Cool HVAC dispatched · ETA Jul 6"</div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="asp-card">
                                        <div class="asp-card-row">
                                            <div style="flex:1;">
                                                <div style="display:flex;gap:.35rem;align-items:center;margin-bottom:.3rem;">
                                                    <span class="asp-status asp-status--blue">"Invoice Review"</span>
                                                    <span class="asp-muted" style="font-size:.68rem;">"#WO-2839 · Jun 28"</span>
                                                </div>
                                                <div class="asp-card-title">"Leaking pipe under sink · Unit 7C"</div>
                                                <div class="asp-card-sub">"Invoice: $285 · RapidFix Plumbing"</div>
                                            </div>
                                            <div style="display:flex;gap:.35rem;">
                                                <button class="asp-btn asp-btn-approve">"✓ Approve"</button>
                                                <button class="asp-btn" style="background:rgba(248,113,113,.08);border-color:rgba(248,113,113,.25);color:#f87171;">"✗ Dispute"</button>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="asp-card" style="opacity:.6;">
                                        <div class="asp-card-row">
                                            <div>
                                                <div style="display:flex;gap:.35rem;align-items:center;margin-bottom:.3rem;">
                                                    <span class="asp-status asp-status--gray">"Closed"</span>
                                                    <span class="asp-muted" style="font-size:.68rem;">"#WO-2830 · Jun 15"</span>
                                                </div>
                                                <div class="asp-card-title">"Broken lock · Unit 11B"</div>
                                                <div class="asp-card-sub">"Paid $120 · SafeKey Locksmith"</div>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                // TAB 5: Payments
                                <div class="asp-panel" data-tab="5">
                                    <div class="asp-page-title">"Payments"</div>
                                    <div class="asp-page-sub">"Ledger — July 2026"</div>
                                    <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Collected · Jul"</div><div class="asp-stat-value" style="color:#22c55e;">"$24,850"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Outstanding"</div><div class="asp-stat-value" style="color:#f59e0b;">"$1,240"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Expenses · Jul"</div><div class="asp-stat-value">"$3,180"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Net income"</div><div class="asp-stat-value" style="color:#22c55e;">"$21,670"</div></div>
                                    </div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Date"</th><th>"Description"</th><th>"Amount"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"Jul 1"</td><td>"Rent · Marcus Reid"</td><td class="asp-credit">"+$1,850"</td></tr>
                                            <tr><td>"Jul 1"</td><td>"Rent · Aisha Okonkwo"</td><td class="asp-credit">"+$2,100"</td></tr>
                                            <tr><td>"Jul 3"</td><td>"Arctic Cool HVAC"</td><td class="asp-debit">"-$340"</td></tr>
                                            <tr><td>"Jul 4"</td><td>"Insurance premium"</td><td class="asp-debit">"-$480"</td></tr>
                                            <tr><td>"Jul 4"</td><td>"Rent · Tom Rivas"</td><td style="color:#f59e0b;">"+$1,240 (pending)"</td></tr>
                                        </tbody>
                                    </table>
                                    <div style="display:flex;gap:.5rem;margin-top:.75rem;">
                                        <button class="asp-btn asp-btn-export">"⬇ Export Schedule E (IRS)"</button>
                                        <button class="asp-btn asp-btn-export">"⬇ Export CSV"</button>
                                    </div>
                                </div>

                                // TAB 6: Owner Portal
                                <div class="asp-panel" data-tab="6">
                                    <div class="asp-page-title">"Owner Portal"</div>
                                    <div class="asp-page-sub">"Read-only investor view — equity, distributions, and tax documents"</div>
                                    <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Portfolio value"</div><div class="asp-stat-value">"$2.4M"</div><div class="asp-stat-delta asp-delta-up">"↑ $180K YTD"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Total equity"</div><div class="asp-stat-value">"$940K"</div><div class="asp-stat-delta asp-delta-up">"39% avg LTV"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Net income · Jul"</div><div class="asp-stat-value" style="color:#22c55e;">"$21,670"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Cash-on-cash"</div><div class="asp-stat-value" style="color:#818cf8;">"7.8%"</div></div>
                                    </div>
                                    <div class="asp-section-hdr">"Distribution history"</div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Period"</th><th>"Status"</th><th>"Amount"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"Jul 2026"</td><td><span class="asp-status asp-status--green">"Deposited"</span></td><td class="asp-credit">"+$8,240"</td></tr>
                                            <tr><td>"Jun 2026"</td><td><span class="asp-status asp-status--green">"Deposited"</span></td><td class="asp-credit">"+$7,980"</td></tr>
                                            <tr><td>"May 2026"</td><td><span class="asp-status asp-status--green">"Deposited"</span></td><td class="asp-credit">"+$8,100"</td></tr>
                                        </tbody>
                                    </table>
                                    <div class="asp-section-hdr">"Documents"</div>
                                    <table class="asp-table">
                                        <tbody>
                                            <tr><td>"July 2026 Statement"</td><td><span class="asp-status asp-status--green">"Ready"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                            <tr><td>"Depreciation schedule (MACRS)"</td><td><span class="asp-status asp-status--green">"Ready"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                            <tr><td>"Schedule E package · 2025"</td><td><span class="asp-status asp-status--green">"Filed"</span></td><td><button class="asp-btn asp-btn-export">"⬇ ZIP"</button></td></tr>
                                        </tbody>
                                    </table>
                                    <div class="asp-callout">"<strong>🔒 Read-only access</strong> — Owners see their numbers. They cannot access rent rolls, lease documents, or tenant contact data."</div>
                                </div>

                            </main>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
