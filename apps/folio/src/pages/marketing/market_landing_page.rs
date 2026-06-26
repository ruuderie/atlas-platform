//! MarketLandingPage — SSR landing page for Atlas Platform products.
//!
//! Served at:
//!   /lp                → product master (apex domain visit, e.g. folio.app)
//!   /lp/:variant_slug  → market variant (e.g. /lp/miami-fl for miami.folio.app)
//!
//! This component is **zero-auth** — no session cookie required. It is accessible
//! to any visitor, crawler, or CDN edge worker. Pixel snippets are injected into
//! `<head>` at SSR render time; visitors receive 0 bytes of WASM.
//!
//! # Data flow
//! ```text
//! Browser / CDN → GET /lp/:variant_slug (this Folio SSR binary)
//!     → #[server] load_landing_page()     (in-process, no HTTP round-trip)
//!         → atlas_client::fetch("/api/pub/products/folio/{slug}")
//!     → MarketLandingPage renders HTML
//! ```
//!
//! # Launch mode gating
//! The CTA section is gated by `LaunchMode` so the correct call-to-action is
//! shown without any client-side JS:
//!
//! | Mode       | Rendered CTA                              |
//! |------------|-------------------------------------------|
//! | Active     | Sign up / onboarding link                 |
//! | Waitlist   | Email capture form                        |
//! | PreOrder   | Stripe Checkout button                    |
//! | PreLaunch  | Coming soon (no conversion form)          |
//! | Draft      | NotFound (page does not exist publicly)   |

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

use crate::pages::not_found::NotFound;

// ── Page data types (mirrors backend pub_products response) ───────────────────

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
    pub tagline:            Option<String>,
    pub meta_title:         Option<String>,
    pub meta_description:   Option<String>,
    pub og_image_url:       Option<String>,
    pub canonical_url:      Option<String>,
    pub structured_data:    Option<serde_json::Value>,
    pub cta_label:          String,
    pub cta_action:         String,
    pub hero_payload:       serde_json::Value,
    pub blocks_payload:     serde_json::Value,
    pub hreflang:           Vec<HreflangEntry>,
    pub pixels:             Vec<PixelSnippet>,
    pub city:               Option<String>,
    pub region:             Option<String>,
    pub locale:             String,
}

// ── Server function ───────────────────────────────────────────────────────────

/// Loads landing page data from the backend in-process during SSR.
/// Falls back to a reqwest HTTP call on the client side (CSR navigation).
/// No session token required — these are public endpoints.
#[server(LoadLandingPage, "/api")]
pub async fn load_landing_page(
    variant_slug: Option<String>,
) -> Result<LandingPageData, server_fn::error::ServerFnError> {
    // Hard-coded to "folio" — this component lives in the Folio binary.
    // Other app binaries (Anchor, etc.) will have their own equivalent.
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

// ── Component ─────────────────────────────────────────────────────────────────

/// Root SSR landing page component. Wired at `/lp` and `/lp/:variant_slug`
/// in `app.rs`.
#[component]
pub fn MarketLandingPage() -> impl IntoView {
    let params = use_params_map();
    let variant_slug = move || {
        params.with(|p| p.get("variant_slug"))
    };

    let page = Resource::new(variant_slug, |slug| load_landing_page(slug));

    view! {
        <Suspense fallback=|| view! { <LandingPageSkeleton/> }>
            {move || page.get().map(|result| match result {
                Err(_) => view! { <NotFound/> }.into_any(),
                Ok(data) => match data.launch_mode {
                    LaunchMode::Draft => view! { <NotFound/> }.into_any(),
                    _ => view! { <LandingPageShell data=data/> }.into_any(),
                },
            })}
        </Suspense>
    }
}

// ── Shell — injecting <head> and page structure ───────────────────────────────

#[component]
fn LandingPageShell(data: LandingPageData) -> impl IntoView {
    let title = data.meta_title.clone()
        .unwrap_or_else(|| data.product_name.clone());
    let description = data.meta_description.clone()
        .unwrap_or_else(|| data.tagline.clone().unwrap_or_default());
    let og_image = data.og_image_url.clone().unwrap_or_default();
    let canonical = data.canonical_url.clone().unwrap_or_default();

    // Structured data JSON-LD (if provided)
    let jsonld = data.structured_data
        .as_ref()
        .and_then(|v| serde_json::to_string(v).ok())
        .unwrap_or_default();

    // Head pixel snippets (inject_at = "head")
    let head_pixels: Vec<_> = data.pixels.iter()
        .filter(|p| p.inject_at == "head")
        .map(|p| p.snippet.clone())
        .collect();

    view! {
        // ── <head> metadata ────────────────────────────────────────────────────
        <Title text=title.clone()/>
        <Meta name="description" content=description.clone()/>
        <Meta property="og:title"       content=title.clone()/>
        <Meta property="og:description" content=description/>
        <Meta property="og:image"       content=og_image/>
        <Meta property="og:type"        content="website"/>
        <Meta name="twitter:card"       content="summary_large_image"/>
        {if !canonical.is_empty() {
            Some(view! { <Link rel="canonical" href=canonical/> })
        } else { None }}

        // hreflang alternate links for cross-market SEO
        {data.hreflang.iter().map(|h| view! {
            <Link rel="alternate" hreflang=h.locale.clone() href=h.url.clone()/>
        }).collect_view()}

        // Structured data JSON-LD
        {if !jsonld.is_empty() {
            Some(view! {
                <script type="application/ld+json">{jsonld}</script>
            })
        } else { None }}

        // Tracking pixel snippets (injected verbatim into <head>)
        // Note: snippets are operator-controlled; sanitization is the
        // operator's responsibility (these are platform-level trust pixels).
        {head_pixels.into_iter().map(|snippet| view! {
            <script inner_html=snippet></script>
        }).collect_view()}

        // ── Page body ──────────────────────────────────────────────────────────
        <main class="market-landing-page">
            <LandingHero data=data.clone()/>
            <LandingCta data=data/>
        </main>
    }
}

// ── Hero section ──────────────────────────────────────────────────────────────

#[component]
fn LandingHero(data: LandingPageData) -> impl IntoView {
    // Market label: "Miami, Florida" | "Florida" | "Miami" | locale fallback
    let market_label = match (&data.city, &data.region) {
        (Some(c), Some(r)) => format!("{c}, {r}"),
        (Some(c), None)    => c.clone(),
        (None, Some(r))    => r.clone(),
        _                  => String::new(),
    };

    // Headline from hero_payload.headline, falling back to product name
    let headline = data.hero_payload
        .get("headline")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| data.product_name.clone());

    let subheadline = data.hero_payload
        .get("subheadline")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| data.tagline.clone().unwrap_or_default());

    view! {
        <section class="hero">
            {(!market_label.is_empty()).then(|| view! {
                <p class="market-label">{market_label}</p>
            })}
            <h1>{headline}</h1>
            <p class="subheadline">{subheadline}</p>
        </section>
    }
}

// ── CTA section — gated by LaunchMode ────────────────────────────────────────

#[component]
fn LandingCta(data: LandingPageData) -> impl IntoView {
    let product_slug  = data.product_slug.clone();
    let variant_slug  = data.variant_slug.clone().unwrap_or_default();

    match data.launch_mode {
        LaunchMode::Active => view! {
            <section class="cta cta--active">
                <a href=data.cta_action class="btn btn--primary">
                    {data.cta_label}
                </a>
            </section>
        }.into_any(),

        LaunchMode::Waitlist => {
            let waitlist_url = if variant_slug.is_empty() {
                format!("/api/pub/products/{product_slug}/waitlist")
            } else {
                format!("/api/pub/products/{product_slug}/{variant_slug}/waitlist")
            };
            view! {
                <section class="cta cta--waitlist">
                    <WaitlistForm action=waitlist_url cta_label=data.cta_label/>
                </section>
            }.into_any()
        },

        LaunchMode::PreOrder => view! {
            <section class="cta cta--preorder">
                <a href=data.cta_action class="btn btn--primary">
                    {data.cta_label}
                </a>
            </section>
        }.into_any(),

        LaunchMode::PreLaunch => view! {
            <section class="cta cta--prelaunching">
                <p class="coming-soon">{"Coming soon"}</p>
            </section>
        }.into_any(),

        LaunchMode::Draft => view! { <NotFound/> }.into_any(),
    }
}

// ── Waitlist form ─────────────────────────────────────────────────────────────

#[component]
fn WaitlistForm(action: String, cta_label: String) -> impl IntoView {
    view! {
        // Progressive enhancement: works without JS via standard form POST.
        // Leptos intercepts and enhances with client-side submission when available.
        <form
            method="POST"
            action=action
            class="waitlist-form"
        >
            <input
                type="email"
                name="email"
                placeholder="your@email.com"
                required
                class="waitlist-form__email"
                id="waitlist-email"
            />
            <button type="submit" class="btn btn--primary" id="waitlist-submit">
                {cta_label}
            </button>
        </form>
    }
}

// ── Skeleton (shown during Suspense) ──────────────────────────────────────────

#[component]
fn LandingPageSkeleton() -> impl IntoView {
    view! {
        <main class="market-landing-page market-landing-page--loading" aria-busy="true">
            <section class="hero">
                <div class="skeleton skeleton--title"/>
                <div class="skeleton skeleton--subtitle"/>
            </section>
            <section class="cta">
                <div class="skeleton skeleton--button"/>
            </section>
        </main>
    }
}
