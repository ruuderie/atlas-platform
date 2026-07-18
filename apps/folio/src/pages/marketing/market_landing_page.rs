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
use shared_ui::marketing::{CtaAction, FolioMarketingSlug};
use uuid::Uuid;

use crate::components::marketing_nav::{
    MarketingNav, MarketingNavRole, DEFAULT_MARKETING_NAV_CTA, HOME_MARKETING_SECTION_LINKS,
};
use crate::geo::{get_visitor_geo, VisitorGeo};
use crate::pages::marketing::block_renderer::{
    has_full_page_block, parse_section_blocks, BetaStripBlock, BlockRenderer, CardSectionBlock,
    CtaBlock, FeatureGridBlock, FooterBlock, MarketsBlock, PaymentRailsBlock, PersonasBlock,
    PricingIntroBlock, StatsBlock,
};
use crate::pages::marketing::hero_content::HeroContent;
use crate::pages::not_found::NotFound;

// ── Page data types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
// Keep aligned with `atlas_backend::types::gtm::LaunchMode`.
pub enum LaunchMode {
    Active,
    Waitlist,
    PreOrder,
    PreLaunch,
    Draft,
}

impl Default for LaunchMode {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HreflangEntry {
    pub locale: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PixelSnippet {
    pub pixel_type: String,
    pub snippet: String,
    pub inject_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ServeSource {
    Cms,
    ProductTemplate,
}

impl Default for ServeSource {
    fn default() -> Self {
        Self::ProductTemplate
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LandingPageData {
    #[serde(default)]
    pub page_id: Option<Uuid>,
    #[serde(default)]
    pub variant_id: Option<Uuid>,
    #[serde(default)]
    pub serve_source: ServeSource,
    pub product_slug: String,
    pub variant_slug: Option<String>,
    pub launch_mode: LaunchMode,
    pub product_name: String,
    /// Tagline / subtitle (not always returned by backend master endpoint)
    #[serde(default)]
    pub tagline: Option<String>,
    pub meta_title: Option<String>,
    pub meta_description: Option<String>,
    pub og_image_url: Option<String>,
    pub canonical_url: Option<String>,
    pub structured_data: Option<serde_json::Value>,
    pub cta_label: String,
    pub cta_action: String,
    #[serde(default)]
    pub waitlist_count: i32,
    #[serde(rename = "hero")]
    pub hero_payload: serde_json::Value,
    #[serde(rename = "blocks")]
    pub blocks_payload: serde_json::Value,
    #[serde(default)]
    pub hreflang: Vec<HreflangEntry>,
    #[serde(default)]
    pub pixels: Vec<PixelSnippet>,
    pub city: Option<String>,
    /// Region label (returned by variant endpoint, absent on master)
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default = "default_locale")]
    pub locale: String,
    /// Marketing pricing cards from `platform_product_plans` (empty → Folio hardcoded fallback).
    #[serde(default)]
    pub plans: Vec<crate::pages::marketing::marketing_pricing::MarketingPlan>,
}

impl LandingPageData {
    pub fn parsed_cta_action(&self) -> Option<CtaAction> {
        CtaAction::try_from(self.cta_action.as_str()).ok()
    }
}

fn default_locale() -> String {
    "en-US".to_string()
}

pub fn fire_lp_view_event(page_id: Option<Uuid>, variant_id: Option<Uuid>) {
    #[cfg(feature = "hydrate")]
    if let Some(page_id) = page_id {
        leptos::task::spawn_local(async move {
            let session_id = web_sys::window()
                .and_then(|window| window.local_storage().ok().flatten())
                .and_then(|storage| {
                    if let Ok(Some(existing)) = storage.get_item("atlas_lp_session") {
                        return Some(existing);
                    }
                    let next = format!("lp-{}", js_sys::Date::now() as i64);
                    let _ = storage.set_item("atlas_lp_session", &next);
                    Some(next)
                })
                .unwrap_or_else(|| format!("lp-{}", js_sys::Date::now() as i64));

            let referrer = web_sys::window()
                .and_then(|window| window.document())
                .map(|document| document.referrer())
                .filter(|value| !value.is_empty());

            let body = serde_json::json!({
                "app_page_id": page_id,
                "variant_id": variant_id,
                "event_type": "view",
                "session_id": session_id,
                "referrer": referrer,
            });

            let _ = gloo_net::http::Request::post("/api/pub/lp-events")
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (page_id, variant_id);
    }
}

// ── Server function ───────────────────────────────────────────────────────────

#[server(LoadLandingPage, "/api")]
pub async fn load_landing_page(
    variant_slug: Option<String>,
) -> Result<LandingPageData, server_fn::error::ServerFnError> {
    const PRODUCT_SLUG: FolioMarketingSlug = FolioMarketingSlug::Folio;
    let path = match &variant_slug {
        Some(v) if !v.is_empty() => format!("{}/{v}", PRODUCT_SLUG.pub_product_path()),
        _ => PRODUCT_SLUG.pub_product_path(),
    };
    crate::atlas_client::fetch::<LandingPageData>(&path)
        .await
        .map_err(|e| server_fn::error::ServerFnError::new(format!("Landing page load failed: {e}")))
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn MarketLandingPage() -> impl IntoView {
    let params = use_params_map();
    let variant_slug = move || params.with(|p| p.get("variant_slug"));

    let page = Resource::new(variant_slug, |slug| load_landing_page(slug));
    let geo = Resource::new(|| (), |_| get_visitor_geo());

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
    let title = data
        .meta_title
        .clone()
        .unwrap_or_else(|| "Folio — Modern Landlord OS".to_string());
    let description = data.meta_description.clone()
        .unwrap_or_else(|| "The only property management platform built for independent landlords. Collect rent, manage leases, handle maintenance, and run vacation rentals — one login.".to_string());
    let og_image = data.og_image_url.clone().unwrap_or_default();
    let canonical = data.canonical_url.clone().unwrap_or_default();
    let jsonld = data
        .structured_data
        .as_ref()
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_default();

    let head_pixels: Vec<_> = data
        .pixels
        .iter()
        .filter(|p| p.inject_at == "head")
        .map(|p| p.snippet.clone())
        .collect();

    let variant_slug = data
        .variant_slug
        .clone()
        .unwrap_or_else(|| geo.variant_slug().to_string());
    let country_code = geo.country_code.clone();
    let product_slug = data.product_slug.clone();
    let launch_mode = data.launch_mode.clone();
    let render_cms = has_full_page_block(&data.blocks_payload);
    let section_blocks = parse_section_blocks(&data.blocks_payload);
    let hero = HeroContent::from_value(&data.hero_payload);
    let hero_payload = data.hero_payload.clone();
    let blocks_payload = data.blocks_payload.clone();
    let plans = data.plans.clone();
    let nav_links = section_blocks.nav_sections.as_ref().map(|block| {
        block
            .items
            .iter()
            .map(|item| (item.label.clone(), item.href.clone()))
            .collect::<Vec<_>>()
    });
    let waitlist_count = data.waitlist_count;
    let cta_label = data.cta_label.clone();
    fire_lp_view_event(data.page_id, data.variant_id);

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
            <MarketingNav
                active=MarketingNavRole::Landlords
                section_links=HOME_MARKETING_SECTION_LINKS
                section_link_overrides=nav_links
                cta_href="#waitlist-wrap"
            />
            {if render_cms {
                view! {
                    <BlockRenderer hero=hero_payload blocks=blocks_payload/>
                    {(!plans.is_empty()).then(|| view! {
                        <crate::pages::marketing::marketing_pricing::MarketingPricingGrid
                            plans=plans.clone()
                            section_id="pricing".to_string()
                            eyebrow=hero.pricing_eyebrow.clone().unwrap_or_else(|| "For landlords · your own properties".to_string())
                            heading=hero.pricing_heading.clone().unwrap_or_else(|| "Simple. Transparent. No surprises.".to_string())
                            subtitle=hero.pricing_subtitle.clone().unwrap_or_else(|| "Start free. Pay as you grow. Built for landlords managing their own portfolio — no owner-client billing, no trust accounting.".to_string())
                            default_cta_href="#waitlist-wrap".to_string()
                        />
                    })}
                }.into_any()
            } else {
                view! {
                    <MktgHero
                        launch_mode=launch_mode
                        product_slug=product_slug
                        variant_slug=variant_slug
                        country_code=country_code
                        hero=hero.clone()
                        waitlist_count=waitlist_count
                        cta_label=cta_label
                    />
                    <MktgStats override_block=section_blocks.stats.clone()/>
                    <MktgPersonas override_block=section_blocks.personas.clone()/>
                    <MktgFeatures override_block=section_blocks.feature_grid.clone()/>
                    <MktgTenantPortal override_block=section_blocks.tenant_portal.clone()/>
                    <MktgStr override_block=section_blocks.str_section.clone()/>
                    <MktgAppPreview/>
                    <MktgInternational override_block=section_blocks.markets.clone()/>
                    <MktgPayments override_block=section_blocks.payment_rails.clone()/>
                    <MktgPricing plans=plans hero=hero pricing_intro=section_blocks.pricing_intro.clone()/>
                    <MktgCta override_block=section_blocks.cta.clone()/>
                    <BetaCalloutStrip override_block=section_blocks.beta_strip.clone()/>
                }.into_any()
            }}
            <MktgFooter override_block=section_blocks.footer.clone()/>
            <MktgScripts/>
        </div>
    }
}

// ── Hero + waitlist form ──────────────────────────────────────────────────────

#[allow(unused_variables)]
#[component]
fn MktgHero(
    launch_mode: LaunchMode,
    product_slug: String,
    variant_slug: String,
    country_code: String,
    hero: HeroContent,
    waitlist_count: i32,
    cta_label: String,
) -> impl IntoView {
    let _ = launch_mode; // Future: gate CTA on Active/PreLaunch modes
    let _ = product_slug;
    let waitlist_url = FolioMarketingSlug::Folio.waitlist_path();
    let eyebrow = hero
        .eyebrow
        .clone()
        .unwrap_or_else(|| "Beta Access Open · US · Canada".to_string());
    let headline = hero
        .headline
        .clone()
        .unwrap_or_else(|| "The operating system".to_string());
    let headline_accent = hero
        .headline_accent
        .clone()
        .unwrap_or_else(|| " for the modern real estate investor.".to_string());
    let subhead = hero.subhead.clone().unwrap_or_else(|| {
        "Your rental business runs on gut feel, spreadsheets, and three apps that don’t talk to each other. Folio is the purpose-built platform that handles rent, leases, maintenance, vacation rentals, and compliance — so you can run your properties like the business they actually are.".to_string()
    });
    let proof_items = if hero.proof_items.is_empty() {
        vec![
            "Beta — be one of the first".to_string(),
            "Built by a landlord".to_string(),
            "No setup fee".to_string(),
            "Long-term + vacation rentals".to_string(),
            "US · Canada".to_string(),
        ]
    } else {
        hero.proof_items
            .iter()
            .filter(|item| !item.trim().is_empty())
            .cloned()
            .collect()
    };
    let primary_cta = if cta_label.trim().is_empty() {
        format!("{DEFAULT_MARKETING_NAV_CTA} →")
    } else {
        cta_label
    };

    // Form step: 0 = email, 1 = details, 2 = success
    let step = RwSignal::new(0u8);
    let email = RwSignal::new(String::new());
    let role = RwSignal::new(String::new());
    let size = RwSignal::new(String::new());
    let source = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let position = RwSignal::new(waitlist_count.max(0) as u32);
    let err_msg = RwSignal::new(String::new());
    let loading = RwSignal::new(false);

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
                })()",
            );
        }
    };

    let wl_url = waitlist_url.clone();
    let vs = variant_slug.clone();

    let submit_step2 = {
        let wl_url2 = wl_url.clone();
        let vs2 = vs.clone();
        move |_| {
            if loading.get() {
                return;
            }
            loading.set(true);
            let url = wl_url2.clone();
            let vs3 = vs2.clone();
            let e = email.get();
            let r = role.get();
            let s = size.get();
            let src = source.get();
            let p = phone.get();
            leptos::task::spawn_local(async move {
                let mut body = serde_json::json!({
                    "email":               e,
                    "role":                if r.is_empty() { serde_json::Value::Null } else { r.into() },
                    "portfolio_size_label": if s.is_empty() { serde_json::Value::Null } else { s.into() },
                    "phone":               if p.is_empty() { serde_json::Value::Null } else { p.into() },
                    "utm_source":          if src.is_empty() { serde_json::Value::Null } else { src.into() },
                    "variant_slug":        vs3,
                });
                crate::marketing_attribution::merge_into_waitlist_body(&mut body);
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

    let role_pills = [
        "🏠 Landlord",
        "💼 Property Manager",
        "🏨 Vacation Rental Host",
        "🏡 Tenant",
        "🔧 Vendor",
        "📊 Investor",
    ];
    let size_pills = [
        "1–5 units",
        "6–20 units",
        "21–100 units",
        "100+ units",
        "Not applicable",
    ];

    view! {
        <section id="hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"science"</span>
                    " "
                    {eyebrow}
                </div>
                <h1 class="mktg-hero-h1">
                    {headline}
                    <span class="mktg-h1-accent"> {headline_accent}</span>
                </h1>
                <p class="mktg-hero-sub">
                    {subhead}
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
                                    {primary_cta.clone()}
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
                                <a href="/login" class="mktg-wl-signin-link" id="hero-signin-link" rel="external">"Sign in →"</a>
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
                        {proof_items.iter().enumerate().map(|(idx, item)| {
                            let icon = if idx == 0 { "science" } else if idx == 1 { "verified" } else { "check_circle" };
                            view! {
                                {(idx > 0).then(|| view! { <span class="mktg-proof-sep"></span> })}
                                <span class="mktg-proof-item">
                                    <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">{icon}</span>
                                    {item.clone()}
                                </span>
                            }
                        }).collect_view()}
                    </div>
                </Show>
            </div>
        </section>
    }
}

// ── Stats band ────────────────────────────────────────────────────────────────

#[component]
fn MktgStats(#[prop(default = None)] override_block: Option<StatsBlock>) -> impl IntoView {
    let items: Vec<(String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.value, item.label))
                .collect()
        })
        .unwrap_or_else(|| {
            vec![
                ("5 min".to_string(), "Average setup time".to_string()),
                (
                    "1 login".to_string(),
                    "For your whole portfolio".to_string(),
                ),
                ("3".to_string(), "Countries at launch".to_string()),
                ("$0".to_string(), "Setup fee · no contracts".to_string()),
            ]
        });

    view! {
        <section class="mktg-stats">
            {items.into_iter().map(|(val, label)| view! {
                <div class="mktg-stat">
                    <span class="mktg-stat-val">{val}</span>
                    <span class="mktg-stat-label">{label}</span>
                </div>
            }).collect_view()}
        </section>
    }
}

// ── Personas ──────────────────────────────────────────────────────────────────

#[component]
fn MktgPersonas(#[prop(default = None)] override_block: Option<PersonasBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Built for every role".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "One platform. Every person in the deal.".to_string());
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| "Folio issues role-based portals so landlords, tenants, vendors, owners, and managers each see exactly what they need — nothing more.".to_string());
    let default_personas: Vec<(String, String, String, String, Vec<String>)> = vec![
        (
            "🏠".to_string(),
            "Independent Landlord".to_string(),
            "coral".to_string(),
            "1–20 units".to_string(),
            vec![
                "Dashboard and reports".to_string(),
                "Automated rent reminders".to_string(),
                "Lease templates & e-sign".to_string(),
                "Maintenance dispatch".to_string(),
            ],
        ),
        (
            "💼".to_string(),
            "Property Manager".to_string(),
            "teal".to_string(),
            "Any size".to_string(),
            vec![
                "Multi-client portfolio".to_string(),
                "Owner statements & reports".to_string(),
                "Branded tenant portal".to_string(),
                "Owner disbursement & fees".to_string(),
            ],
        ),
        (
            "🏨".to_string(),
            "Vacation Rental Host".to_string(),
            "gold".to_string(),
            "Airbnb + direct".to_string(),
            vec![
                "Booking calendar".to_string(),
                "Channel sync".to_string(),
                "Guest messaging".to_string(),
                "Vacation rental licensing & compliance".to_string(),
            ],
        ),
        (
            "🏡".to_string(),
            "Tenant".to_string(),
            "green".to_string(),
            "Renter portal".to_string(),
            vec![
                "Pay rent online".to_string(),
                "Submit maintenance requests".to_string(),
                "View & sign lease".to_string(),
                "Track move-in docs".to_string(),
            ],
        ),
        (
            "🔧".to_string(),
            "Vendor / Contractor".to_string(),
            "orange".to_string(),
            "Work order portal".to_string(),
            vec![
                "Receive job dispatches".to_string(),
                "Submit invoices".to_string(),
                "Schedule management".to_string(),
                "Marketplace profile".to_string(),
            ],
        ),
    ];
    let personas: Vec<(String, String, String, String, Vec<String>)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .enumerate()
                .map(|(idx, item)| {
                    let fallback = default_personas[idx.min(default_personas.len() - 1)].clone();
                    (
                        item.icon.unwrap_or(fallback.0),
                        if item.title.is_empty() {
                            fallback.1
                        } else {
                            item.title
                        },
                        item.accent.unwrap_or(fallback.2),
                        item.subhead.unwrap_or(fallback.3),
                        if item.bullets.is_empty() {
                            fallback.4
                        } else {
                            item.bullets
                        },
                    )
                })
                .collect()
        })
        .unwrap_or(default_personas);

    view! {
        <section id="personas" class="mktg-section mktg-personas">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
                <p class="mktg-section-sub">{subhead}</p>
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
fn MktgFeatures(#[prop(default = None)] override_block: Option<FeatureGridBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "What's included".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "From first lease to tax season — covered.".to_string());
    // CMS section blocks overlay these defaults when seeded.
    let default_cells = vec![
        ("payments", "Rent collection", "Bank transfer, card, and international payment methods. Automatically records every payment. No separate accounting tool needed."),
        ("description", "Lease management", "Create, send, e-sign, renew, and store leases. Templates cover the required disclosures for your state or country."),
        ("build", "Maintenance tracking", "Tenants report issues, you approve the work, contractors receive the job and send invoices — all in one place."),
        ("calendar_month", "Vacation rental calendar", "Airbnb, VRBO, Booking.com and your own direct bookings in one calendar. No double-bookings, ever."),
        ("verified_user", "Compliance reminders", "Vacation rental permits, fair housing requirements, and local registration renewals — tracked automatically so nothing slips."),
        ("analytics", "Portfolio dashboard", "See income, vacancies, and maintenance costs across every property."),
        ("campaign", "Vacancy marketing", "List your vacancy, collect applications, screen tenants, and convert to a signed lease — one workflow."),
        ("groups", "Contractor marketplace", "Find vetted contractors, send them work orders, receive invoices, and leave reviews — all inside Folio."),
        ("language", "Multi-country", "United States · Canada — with more markets on the way."),
    ];
    let cells: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.icon, item.title, item.description))
                .collect()
        })
        .unwrap_or_else(|| {
            default_cells
                .into_iter()
                .map(|(icon, title, desc)| (icon.to_string(), title.to_string(), desc.to_string()))
                .collect()
        });

    view! {
        <section id="features" class="mktg-section mktg-features">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
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
fn MktgTenantPortal(
    #[prop(default = None)] override_block: Option<CardSectionBlock>,
) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Tenant experience".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| {
            "Happy tenants pay on time. Give them a portal worth logging into.".to_string()
        });
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| {
            "When your tenant logs in, they see their own dashboard — not yours. They can pay rent, report a problem, sign their lease, and track move-in documents without calling you. Less back-and-forth for you. Better experience for them.".to_string()
        });
    let default_items = vec![
        ("payments", "Pay rent online", "Bank transfer, card, or local payment method. Rent hits your account automatically."),
        ("build", "Maintenance requests", "Tenants describe the issue, upload a photo, and you get notified instantly. No more texts."),
        ("description", "Lease & documents", "Tenants can read, sign, and download their lease anytime. No printing. No scanning."),
        ("notifications", "Move-in checklist", "Guide tenants through move-in day: what to submit, what to expect, how to reach you."),
    ];
    let items: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.icon, item.title, item.desc))
                .collect()
        })
        .unwrap_or_else(|| {
            default_items
                .into_iter()
                .map(|(icon, title, desc)| (icon.to_string(), title.to_string(), desc.to_string()))
                .collect()
        });

    view! {
        <section id="tenant-portal" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
                <p class="mktg-section-sub">{subhead}</p>
                <div class="mktg-str-grid">
                    {items.into_iter().map(|(icon, title, desc)| view! {
                        <div class="mktg-str-card">
                            <span class="material-symbols-outlined mktg-str-icon">{icon}</span>
                            <h3 class="mktg-str-title">{title}</h3>
                            <p class="mktg-str-desc">{desc}</p>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── Vacation rental section ───────────────────────────────────────────────────

#[component]
fn MktgStr(#[prop(default = None)] override_block: Option<CardSectionBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Vacation rentals".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Your vacation rental, fully under control.".to_string());
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| {
            "Most landlord software treats vacation rentals as an afterthought. Folio treats them as a first-class product — one calendar, one inbox, one platform.".to_string()
        });
    let default_items = vec![
        ("calendar_month", "Booking calendar", "Airbnb, VRBO, Booking.com and your own direct bookings in one calendar. Block dates, set minimums, sync instantly."),
        ("verified_user", "Permits & compliance", "Vacation rental permit tracking, renewal reminders, and local registration filings — so you never get caught operating without a license."),
        ("payments", "Collect directly from guests", "Take deposits, damage holds, and nightly rates from guests without paying a middleman's fee stack."),
    ];
    let items: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.icon, item.title, item.desc))
                .collect()
        })
        .unwrap_or_else(|| {
            default_items
                .into_iter()
                .map(|(icon, title, desc)| (icon.to_string(), title.to_string(), desc.to_string()))
                .collect()
        });

    view! {
        <section id="str" class="mktg-section mktg-str-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow mktg-eyebrow-light">{eyebrow}</p>
                <h2 class="mktg-section-h2 mktg-h2-light">{heading}</h2>
                <p class="mktg-section-sub mktg-sub-light">{subhead}</p>
                <div class="mktg-str-grid">
                    {items.into_iter().map(|(icon, title, desc)| view! {
                        <div class="mktg-str-card">
                            <span class="material-symbols-outlined mktg-str-icon">{icon}</span>
                            <h3 class="mktg-str-title">{title}</h3>
                            <p class="mktg-str-desc">{desc}</p>
                        </div>
                    }).collect_view()}
                    // Cohost Network — live page
                    <a href="/cohost-market" class="mktg-str-card mktg-str-card--cohost" style="display:block;text-decoration:none;cursor:pointer;" rel="external">
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
fn MktgInternational(
    #[prop(default = None)] override_block: Option<MarketsBlock>,
) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Where Folio works".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Built for the Americas. Ready for the world.".to_string());
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| "Folio handles multi-currency ledgers, local compliance rules, and payment rails specific to each country — so you don't have to.".to_string());
    let default_markets = vec![
        (
            "🇺🇸",
            "United States",
            "All 50 states · Federal fair housing · ACH + card",
        ),
        (
            "🇨🇦",
            "Canada",
            "ON · BC · QC · PIPEDA-compliant · EFT rails",
        ),
        (
            "🌎",
            "More markets",
            "Additional markets planned — expand when you're ready",
        ),
    ];
    let markets: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.flag, item.name, item.desc))
                .collect()
        })
        .unwrap_or_else(|| {
            default_markets
                .into_iter()
                .map(|(flag, name, desc)| (flag.to_string(), name.to_string(), desc.to_string()))
                .collect()
        });

    view! {
        <section id="international" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
                <p class="mktg-section-sub">{subhead}</p>
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
fn MktgPayments(
    #[prop(default = None)] override_block: Option<PaymentRailsBlock>,
) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Rent collection".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Rent collected. Split. Reported.".to_string());
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| "Payments in one ledger — ACH, EFT, and card — with automatic splits for principal, fees, and reserves.".to_string());
    let default_rails = vec![
        (
            "💳",
            "ACH / EFT",
            "US and Canada bank transfers. 1–2 business day settlement.",
        ),
        (
            "💰",
            "Card",
            "Visa, Mastercard, Amex. Tenant pays the processing fee.",
        ),
        (
            "🏦",
            "Ledger",
            "Every transaction split by category. Export-ready for your accountant.",
        ),
    ];
    let rails: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.icon, item.name, item.desc))
                .collect()
        })
        .unwrap_or_else(|| {
            default_rails
                .into_iter()
                .map(|(icon, name, desc)| (icon.to_string(), name.to_string(), desc.to_string()))
                .collect()
        });

    view! {
        <section class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
                <p class="mktg-section-sub">{subhead}</p>
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
fn MktgPricing(
    plans: Vec<crate::pages::marketing::marketing_pricing::MarketingPlan>,
    hero: HeroContent,
    #[prop(default = None)] pricing_intro: Option<PricingIntroBlock>,
) -> impl IntoView {
    let plans = if plans.is_empty() {
        folio_landlord_fallback_plans()
    } else {
        plans
    };

    view! {
        <crate::pages::marketing::marketing_pricing::MarketingPricingGrid
            plans=plans
            section_id="pricing".to_string()
            eyebrow=pricing_intro.as_ref().and_then(|block| block.eyebrow.clone()).or_else(|| hero.pricing_eyebrow.clone()).unwrap_or_else(|| "For landlords · your own properties".to_string())
            heading=pricing_intro.as_ref().and_then(|block| block.heading.clone()).or_else(|| hero.pricing_heading.clone()).unwrap_or_else(|| "Simple. Transparent. No surprises.".to_string())
            subtitle=pricing_intro.as_ref().and_then(|block| block.subtitle.clone()).or_else(|| hero.pricing_subtitle.clone()).unwrap_or_else(|| "Start free. Pay as you grow. Built for landlords managing their own portfolio — no owner-client billing, no trust accounting.".to_string())
            default_cta_href="#waitlist-wrap".to_string()
        />
        <div class="mktg-section" style="padding-top:0;">
            <div class="mktg-section-inner">
                <div class="mktg-pricing-pm-callout">
                    <p>
                        "Managing properties for other owners? "
                        <a href="/property-managers" rel="external">"See Property Manager pricing →"</a>
                    </p>
                </div>
            </div>
        </div>
    }
}

fn folio_landlord_fallback_plans() -> Vec<crate::pages::marketing::marketing_pricing::MarketingPlan>
{
    use crate::pages::marketing::marketing_pricing::{MarketingPlan, PlanBillingInterval};
    // CMS/product pricing overlays this fallback once plans are seeded.
    vec![
        MarketingPlan {
            slug: "free".into(),
            name: "Free".into(),
            tagline: "Up to 2 units · free forever".into(),
            price_cents: 0,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Forever,
            features: vec![
                "Landlord dashboard".into(),
                "Lease management".into(),
                "Tenant portal".into(),
                "Maintenance requests".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#waitlist-wrap".into()),
            is_featured: false,
            sort_order: 0,
        },
        MarketingPlan {
            slug: "grow".into(),
            name: "Grow".into(),
            tagline: "Up to 10 units".into(),
            price_cents: 2900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Free".into(),
                "Rent collection (ACH + card)".into(),
                "Vacancy marketing".into(),
                "Contractor marketplace".into(),
                "Basic analytics".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#waitlist-wrap".into()),
            is_featured: false,
            sort_order: 1,
        },
        MarketingPlan {
            slug: "pro".into(),
            name: "Pro".into(),
            tagline: "Up to 30 units".into(),
            price_cents: 7900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Grow".into(),
                "Vacation rental calendar & channels".into(),
                "STR compliance & permits".into(),
                "Portfolio analytics".into(),
                "Multi-country (US, Canada)".into(),
                "Priority support".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#waitlist-wrap".into()),
            is_featured: true,
            sort_order: 2,
        },
        MarketingPlan {
            slug: "investor".into(),
            name: "Investor".into(),
            tagline: "Unlimited units".into(),
            price_cents: 14900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Pro".into(),
                "Cohost Network access".into(),
                "Co-host revenue share tracking".into(),
                "Dedicated onboarding".into(),
                "API access".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#waitlist-wrap".into()),
            is_featured: false,
            sort_order: 3,
        },
    ]
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn MktgCta(#[prop(default = None)] override_block: Option<CtaBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Limited beta spots available".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Be one of the first landlords inside.".to_string());
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| {
            "Join the waitlist now and get exclusive early access to Folio and the Cohost Network before we open to the public. Beta members help shape the product and lock in founder pricing.".to_string()
        });
    let button_label = override_block
        .as_ref()
        .and_then(|block| block.button_label.clone())
        .unwrap_or_else(|| "Reserve my beta spot →".to_string());
    let button_href = override_block
        .as_ref()
        .and_then(|block| block.button_href.clone())
        .unwrap_or_else(|| "#waitlist-wrap".to_string());

    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#ff6b35;">{eyebrow}</p>
                <h2 class="mktg-cta-h2">{heading}</h2>
                <p class="mktg-cta-sub">{subhead}</p>
                <a href=button_href class="mktg-btn-accent mktg-btn-lg">{button_label}</a>
                <p style="margin-top:16px;font-size:12px;color:#9ca3af;">"No credit card. No contracts. Cancel anytime."</p>
            </div>
        </section>
    }
}

// ── Beta program callout strip ────────────────────────────────────────────────

#[component]
fn BetaCalloutStrip(
    #[prop(default = None)] override_block: Option<BetaStripBlock>,
) -> impl IntoView {
    let title = override_block
        .as_ref()
        .and_then(|block| block.title.clone())
        .unwrap_or_else(|| "Apply for the Folio Beta Program".to_string());
    let body = override_block
        .as_ref()
        .and_then(|block| block.body.clone())
        .unwrap_or_else(|| "Get free access during beta in exchange for real feedback. We review every application — accepted members shape the product roadmap.".to_string());
    let button_label = override_block
        .as_ref()
        .and_then(|block| block.button_label.clone())
        .unwrap_or_else(|| "Apply now".to_string());
    let button_href = override_block
        .as_ref()
        .and_then(|block| block.button_href.clone())
        .unwrap_or_else(|| "/beta".to_string());

    view! {
        <div class="mktg-section-inner">
            <div class="beta-callout-strip">
                <span class="material-symbols-outlined beta-callout-strip-icon"
                      style="font-variation-settings:'FILL' 1">"science"</span>
                <div class="beta-callout-text">
                    <strong>{title}</strong>
                    <p>{body}</p>
                </div>
                <a href=button_href class="beta-callout-cta" id="beta-strip-cta" rel="external">
                    {button_label}
                    <span class="material-symbols-outlined" style="font-size:16px">"arrow_forward"</span>
                </a>
            </div>
        </div>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn MktgFooter(#[prop(default = None)] override_block: Option<FooterBlock>) -> impl IntoView {
    use crate::components::marketing_footer::MarketingFooter;
    let tagline = override_block
        .as_ref()
        .and_then(|block| block.tagline.clone())
        .unwrap_or_else(|| "Modern Landlord OS".to_string());
    let override_links = override_block
        .filter(|block| !block.links.is_empty())
        .map(|block| {
            block
                .links
                .into_iter()
                .map(|link| (link.label, link.href))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    view! {
        <MarketingFooter tagline=tagline override_links=override_links/>
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
                                    <div class="asp-page-sub">"Today's numbers"</div>
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
