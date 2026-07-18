//! VendorLandingPage — marketing page targeting vendors, contractors & service providers.
//!
//! Served at: `/vendors`
//!
//! Zero-auth, accessible to any visitor. Independently managed under
//! `app_id = "folio-vendor"` in platform-admin.
//!
//! # Value proposition
//! Vendors get dispatched jobs, in-platform invoicing, a scored marketplace
//! profile, and payment — without chasing landlords for checks.
//!
//! # Pricing model
//! Freemium: free marketplace listing + job acceptance;
//! paid tiers unlock priority placement, auto-invoicing, and 0% platform fee.

use crate::components::marketing_nav::{
    resolve_marketing_nav_cta, MarketingNav, MarketingNavRole, MarketingNavSectionLink,
    DEFAULT_MARKETING_NAV_CTA,
};
use crate::pages::marketing::block_renderer::{
    has_full_page_block, parse_section_blocks, BlockRenderer, CtaBlock, FeatureGridBlock, FooterBlock,
    SectionBlocks, TradeCategoriesBlock,
};
use crate::pages::marketing::hero_content::HeroContent;
use crate::pages::marketing::market_landing_page::fire_lp_view_event;
use crate::pages::marketing::marketing_pricing::{
    MarketingPlan, MarketingPricingGrid, PlanBillingInterval,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use shared_ui::marketing::{FolioMarketingSlug, VendorTradeKey};

const VENDOR_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Features",
        href: "#vendor-features",
    },
    MarketingNavSectionLink {
        label: "How it works",
        href: "#vendor-how",
    },
    MarketingNavSectionLink {
        label: "Pricing",
        href: "#vendor-pricing",
    },
];

// ── Server function ───────────────────────────────────────────────────────────

#[server(LoadVendorPage, "/api")]
pub async fn load_vendor_page() -> Result<
    crate::pages::marketing::market_landing_page::LandingPageData,
    server_fn::error::ServerFnError,
> {
    crate::atlas_client::fetch::<crate::pages::marketing::market_landing_page::LandingPageData>(
        &FolioMarketingSlug::FolioVendor.pub_product_path(),
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Vendor page load failed: {e}")))
}

// ── Page shell ───────────────────────────────────────────────────────────────

#[component]
pub fn VendorLandingPage() -> impl IntoView {
    let page = Resource::new(|| (), |_| load_vendor_page());

    view! {
        <Suspense fallback=|| view! {
            <VendorDefault
                plans=Vec::new()
                hero=HeroContent::default()
                cta_label=DEFAULT_MARKETING_NAV_CTA.to_string()
                section_blocks=SectionBlocks::default()
            />
        }>
            {move || page.get().map(|result| {
                match result {
                    Ok(data) if has_full_page_block(&data.blocks_payload) => {
                        let title = data.meta_title.clone().unwrap_or_else(|| data.product_name.clone());
                        let description = data.meta_description.clone().unwrap_or_default();
                        let plans = data.plans.clone();
                        let hero = HeroContent::from_value(&data.hero_payload);
                        let cta_label = resolve_marketing_nav_cta(&data.cta_label);
                        fire_lp_view_event(data.page_id, data.variant_id);
                        view! {
                            <Title text=title.clone()/>
                            <Meta name="description" content=description.clone()/>
                            <Meta property="og:title" content=title/>
                            <Meta property="og:description" content=description/>
                            <Link rel="canonical" href="/vendors"/>
                            <div class="folio-mktg">
                                <MarketingNav
                                    active=MarketingNavRole::Vendors
                                    section_links=VENDOR_SECTION_LINKS
                                    cta_label=cta_label
                                    cta_href="#vendor-signup"
                                />
                                <BlockRenderer hero=data.hero_payload blocks=data.blocks_payload/>
                                <VendorPricing plans=plans hero=hero/>
                                <VendorFooter/>
                            </div>
                        }.into_any()
                    }
                    Ok(data) => {
                        let hero = HeroContent::from_value(&data.hero_payload);
                        let section_blocks = parse_section_blocks(&data.blocks_payload);
                        view! {
                            <VendorDefault
                                plans=data.plans
                                hero=hero
                                cta_label=resolve_marketing_nav_cta(&data.cta_label)
                                section_blocks=section_blocks
                            />
                        }.into_any()
                    }
                    Err(_) => view! {
                        <VendorDefault
                            plans=Vec::new()
                            hero=HeroContent::default()
                            cta_label=DEFAULT_MARKETING_NAV_CTA.to_string()
                            section_blocks=SectionBlocks::default()
                        />
                    }.into_any(),
                }
            })}
        </Suspense>
    }
}

#[component]
fn VendorDefault(
    plans: Vec<MarketingPlan>,
    hero: HeroContent,
    cta_label: String,
    section_blocks: SectionBlocks,
) -> impl IntoView {
    let nav_cta_label = resolve_marketing_nav_cta(&cta_label);
    let nav_links = section_blocks.nav_sections.as_ref().map(|block| {
        block
            .items
            .iter()
            .map(|item| (item.label.clone(), item.href.clone()))
            .collect::<Vec<_>>()
    });

    view! {
        <Title text="Folio for Vendors – Get Jobs, Get Paid, No Chasing"/>
        <Meta name="description" content="Folio connects vendors and contractors to property managers and landlords. Get dispatched jobs, send invoices, collect payment, and grow your service business."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/vendors"/>

        <MarketingNav
            active=MarketingNavRole::Vendors
            section_links=VENDOR_SECTION_LINKS
            section_link_overrides=nav_links
            cta_label=nav_cta_label
            cta_href="#vendor-signup"
        />
        <VendorHero
            hero=hero.clone()
            cta_label=cta_label
            trade_categories=section_blocks.trade_categories.clone()
        />
        <VendorProblem/>
        <VendorHow/>
        <VendorTrades override_block=section_blocks.trade_categories.clone()/>
        <VendorFeatures override_block=section_blocks.feature_grid.clone()/>
        <VendorProfilePreviews/>
        <VendorPricing plans=plans hero=hero/>
        <VendorCta override_block=section_blocks.cta.clone()/>
        <VendorFooter override_block=section_blocks.footer.clone()/>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn VendorHero(
    hero: HeroContent,
    cta_label: String,
    #[prop(default = None)] trade_categories: Option<TradeCategoriesBlock>,
) -> impl IntoView {
    let eyebrow = hero
        .eyebrow
        .clone()
        .unwrap_or_else(|| "Free to join · 19 trade categories · US · Canada".to_string());
    let headline = hero
        .headline
        .clone()
        .unwrap_or_else(|| "The trade network".to_string());
    let headline_accent = hero
        .headline_accent
        .clone()
        .unwrap_or_else(|| " that finds you work.".to_string());
    let subhead = hero.subhead.clone().unwrap_or_else(|| {
        "Property managers and landlords on Folio dispatch jobs directly to verified tradespeople in their area. You get the job details, accept with one tap, invoice in the app, and get paid in 24 hours. No cold calls. No chasing checks.".to_string()
    });
    let primary_cta = resolve_marketing_nav_cta(&cta_label);
    let primary_cta = RwSignal::new(primary_cta);
    let step = RwSignal::new(0u8); // 0=trade, 1=details, 2=success
    let trade = RwSignal::new(String::new());
    let trade_label = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let biz_name = RwSignal::new(String::new());
    let service_area = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let err_msg = RwSignal::new(String::new());

    // CMS `trade_categories` overlays this fallback when seeded.
    let trades = trade_categories
        .as_ref()
        .and_then(|block| (!block.items.is_empty()).then_some(block.items.clone()))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.key, item.label))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(default_vendor_trade_options);

    let submit = move |_| {
        if email.get().is_empty() || service_area.get().is_empty() {
            return;
        }
        loading.set(true);
        err_msg.set(String::new());
        let e = email.get();
        let business = biz_name.get();
        let trade_name = trade_label.get();
        let area = service_area.get();
        let company = if business.trim().is_empty() {
            format!("{trade_name} · {area}")
        } else {
            format!("{business} · {trade_name} · {area}")
        };
        leptos::task::spawn_local(async move {
            let mut body = serde_json::json!({
                "email": e,
                "company": company,
                "role": "Vendor"
            });
            crate::marketing_attribution::merge_into_waitlist_body(&mut body);
            let resp = gloo_net::http::Request::post(&FolioMarketingSlug::FolioVendor.waitlist_path())
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;
            loading.set(false);
            match resp {
                Ok(r) if r.ok() => step.set(2),
                Ok(_) => err_msg
                    .set("We couldn't join the vendor waitlist. Please try again.".to_string()),
                Err(_) => err_msg.set("Network issue. Please try again in a moment.".to_string()),
            }
        });
    };

    view! {
        <section id="vendor-hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:860px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"handyman"</span>
                    " "
                    {eyebrow}
                </div>
                <h1 class="mktg-hero-h1">
                    {headline}
                    <span class="mktg-h1-accent"> {headline_accent}</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:580px;margin:1.5rem auto 0;">
                    {subhead}
                </p>

                // ── Multi-step vendor signup ──────────────────────────────
                <div id="vendor-signup" style="margin-top:2.5rem;" class="mktg-wl-wrap">

                    // Step 0: pick your trade
                    <Show when=move || step.get() == 0 fallback=|| ()>
                        <div class="mktg-wl-step">
                            <div class="mktg-wl-card" style="text-align:left;">
                                <p class="mktg-wl-card-head">"What's your trade?"</p>
                                <div class="mktg-pill-row" style="flex-wrap:wrap;gap:.5rem;">
                                    {trades.iter().map(|(key, label)| {
                                        let k   = key.to_string();
                                        let lbl = label.to_string();
                                        let k2  = k.clone();
                                        let lbl2 = lbl.clone();
                                        view! {
                                            <button
                                                class=move || if trade.get() == k {
                                                    "mktg-pill mktg-pill-role selected"
                                                } else {
                                                    "mktg-pill mktg-pill-role"
                                                }
                                                on:click=move |_| {
                                                    trade.set(k2.clone());
                                                    trade_label.set(lbl2.clone());
                                                }
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
                    </Show>

                    // Step 1: contact + service area
                    <Show when=move || step.get() == 1 fallback=|| ()>
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

                                <button
                                    class="mktg-btn-accent mktg-btn-lg"
                                    style="width:100%;margin-top:.75rem;"
                                    disabled=move || loading.get()
                                    on:click=submit.clone()
                                >
                                    <span class="material-symbols-outlined" style="font-size:20px;font-variation-settings:'FILL' 1">"check_circle"</span>
                                    {move || if loading.get() { "Submitting…".to_string() } else { primary_cta.get() }}
                                </button>
                                <Show when=move || !err_msg.get().is_empty() fallback=|| ()>
                                    <p style="font-size:.78rem;color:#f87171;margin-top:.75rem;">{move || err_msg.get()}</p>
                                </Show>
                                <button
                                    style="background:none;border:none;color:var(--mk-muted);font-size:.8rem;cursor:pointer;margin-top:.5rem;"
                                    on:click=move |_| step.set(0)
                                >"← Change trade"</button>
                            </div>
                        </div>
                    </Show>

                    // Step 2: success
                    <Show when=move || step.get() == 2 fallback=|| ()>
                        <div class="mktg-wl-success">
                            <div class="mktg-success-icon">
                                <span class="material-symbols-outlined" style="font-size:36px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            </div>
                            <h3 class="mktg-success-h3">"You're in the vendor network!"</h3>
                            <p class="mktg-success-sub">
                                "We'll reach out with your marketplace profile setup link \
                                 when we launch in your area. The more vendors join early, \
                                 the faster we can connect you to property owners near you."
                            </p>
                            <div class="mktg-success-card">
                                <div>
                                    <div class="mktg-success-label">"Your trade"</div>
                                    <div class="mktg-success-num" style="font-size:1.1rem;">{move || trade_label.get()}</div>
                                </div>
                                <div class="mktg-success-div"></div>
                                <div>
                                    <div class="mktg-success-label">"Service area"</div>
                                    <div class="mktg-success-num" style="font-size:1.1rem;">{move || service_area.get()}</div>
                                </div>
                            </div>
                        </div>
                    </Show>
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
                        <div class="mktg-stat-num">"19"</div>
                        <div class="mktg-stat-label">"trade categories"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"0"</div>
                        <div class="mktg-stat-label">"chasing or cold calls"</div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Problem section ───────────────────────────────────────────────────────────

#[component]
fn VendorProblem() -> impl IntoView {
    view! {
        <section class="mktg-section"
            style="background:rgba(255,107,53,.03);border-top:1px solid rgba(255,107,53,.12);border-bottom:1px solid rgba(255,107,53,.12);">
            <div class="mktg-section-inner" style="text-align:center;">
                <p class="mktg-section-eyebrow" style="color:#ff6b35;">"The problem with trade work today"</p>
                <h2 class="mktg-section-h2" style="max-width:700px;margin:0 auto 1rem;">
                    "You do great work. Getting paid for it shouldn't be a second job."
                </h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 3rem;">
                    "Property managers have jobs. You have availability. But finding each other means \
                     referrals, phone tag, and paper invoices that get lost. Folio removes every step \
                     between 'job posted' and 'money in your account.'"
                </p>
                <div class="mktg-feature-grid" style="max-width:900px;margin:0 auto;">
                    <div class="mktg-feature-card" style="border-color:rgba(239,68,68,.2);background:rgba(239,68,68,.04);">
                        <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"warning"</span>
                        <h3>"Jobs come from word of mouth"</h3>
                        <p>"You're dependent on referrals that dry up. No way to show up where property managers are actually searching."</p>
                    </div>
                    <div class="mktg-feature-card" style="border-color:rgba(239,68,68,.2);background:rgba(239,68,68,.04);">
                        <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"warning"</span>
                        <h3>"Invoicing is manual and slow"</h3>
                        <p>"Paper invoices, email threads, QuickBooks you barely use. Getting paid in 30 days if you're lucky."</p>
                    </div>
                    <div class="mktg-feature-card" style="border-color:rgba(239,68,68,.2);background:rgba(239,68,68,.04);">
                        <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"warning"</span>
                        <h3>"No proof of your reputation"</h3>
                        <p>"You've done 200 great jobs, but a new property manager has no way to verify that. They go with who they know."</p>
                    </div>
                </div>
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
                <p class="mktg-section-sub" style="max-width:520px;margin:0 auto 2.5rem;">"No account managers. No gatekeeping. You set your trade and area — we send you the work."</p>
                <div class="vnd-how-grid">
                    <div class="vnd-how-step">
                        <div class="vnd-how-icon" style="background:rgba(6,214,160,.12);border-color:#06d6a0;">
                            <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"person_add"</span>
                        </div>
                        <div class="vnd-how-num">"1"</div>
                        <h3 class="vnd-how-title">"Create your profile"</h3>
                        <p class="vnd-how-desc">"List your trade, service area, and availability. Free in under 5 minutes. Your profile becomes searchable to every landlord and PM on Folio."</p>
                    </div>
                    <div class="vnd-how-step">
                        <div class="vnd-how-icon" style="background:rgba(245,158,11,.12);border-color:#f59e0b;">
                            <span class="material-symbols-outlined" style="color:#f59e0b;font-variation-settings:'FILL' 1">"notifications_active"</span>
                        </div>
                        <div class="vnd-how-num">"2"</div>
                        <h3 class="vnd-how-title">"Get dispatched"</h3>
                        <p class="vnd-how-desc">"Receive job notifications with photos, property details, and scope of work. Accept or decline in one tap — on your schedule."</p>
                    </div>
                    <div class="vnd-how-step">
                        <div class="vnd-how-icon" style="background:rgba(255,107,53,.12);border-color:#ff6b35;">
                            <span class="material-symbols-outlined" style="color:#ff6b35;font-variation-settings:'FILL' 1">"payments"</span>
                        </div>
                        <div class="vnd-how-num">"3"</div>
                        <h3 class="vnd-how-title">"Invoice & get paid"</h3>
                        <p class="vnd-how-desc">"Submit your invoice in the app when the job is done. Payment hits your account within 24 hours via ACH. No paperwork. No waiting."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Trade Categories ──────────────────────────────────────────────────────────

#[component]
fn VendorTrades(
    #[prop(default = None)] override_block: Option<TradeCategoriesBlock>,
) -> impl IntoView {
    // CMS `trade_categories` overlays these hardcoded launch categories when seeded.
    let categories: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| {
                    let (icon, name) = split_trade_label(&item.label);
                    (icon, name, item.description.unwrap_or_default())
                })
                .collect()
        })
        .unwrap_or_else(default_vendor_trade_cards);

    view! {
        <section id="vendor-trades" class="mktg-section"
            style="background:rgba(6,214,160,.02);border-top:1px solid rgba(6,214,160,.08);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Open to all trades"</p>
                <h2 class="mktg-section-h2">"Every trade. One marketplace."</h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 2.5rem;">
                    "We're signing up vendors across all 19 categories before launch. \
                     Early vendors get priority placement and the Founding Vendor rate."
                </p>
                <div class="vnd-trades-grid">
                    {categories.into_iter().map(|(icon, name, desc)| view! {
                        // div, not anchor — avoids browser blue link color bleed-through
                        <div class="vnd-trade-card"
                             onclick="document.getElementById('vendor-signup').scrollIntoView({behavior:'smooth'})">
                            <div class="vnd-trade-icon">{icon}</div>
                            <div>
                                <div class="vnd-trade-name">{name}</div>
                                {(!desc.is_empty()).then(|| view! {
                                    <div class="vnd-trade-desc">{desc}</div>
                                })}
                            </div>
                        </div>
                    }).collect_view()}
                </div>
                <div style="text-align:center;margin-top:2.5rem;">
                    <a href="#vendor-signup" class="mktg-btn-accent mktg-btn-lg" id="vendor-trades-cta">
                        "Register your trade →"
                    </a>
                    <p style="margin-top:.75rem;font-size:.78rem;color:var(--mk-muted);">"Free to join. No subscription required."</p>
                </div>
            </div>
        </section>
    }
}

fn default_vendor_trade_options() -> Vec<(String, String)> {
    VendorTradeKey::ALL
        .iter()
        .map(|trade| (trade.as_str().to_string(), trade.default_label().to_string()))
        .collect()
}

fn default_vendor_trade_cards() -> Vec<(String, String, String)> {
    [
        (
            "🧹",
            "Cleaning & Turnover",
            "Move-out cleans, vacation rental turnovers, recurring housekeeping",
        ),
        (
            "🔧",
            "Handyman",
            "Minor repairs, furniture assembly, caulking, drywall patches",
        ),
        (
            "🚿",
            "Plumbing",
            "Leaks, fixture replacements, drain clearing, water heater service",
        ),
        (
            "⚡",
            "Electrical",
            "Outlet repairs, panel work, lighting installs, code compliance",
        ),
        (
            "❄️",
            "HVAC",
            "AC service, furnace repair, filter programs, duct cleaning",
        ),
        (
            "🖌️",
            "Painting",
            "Interior & exterior, unit turns, touch-ups, power washing",
        ),
        (
            "🌿",
            "Landscaping",
            "Lawn care, tree trimming, irrigation, seasonal cleanups",
        ),
        (
            "🏠",
            "Roofing",
            "Inspections, leak repairs, gutter cleaning, full replacements",
        ),
        (
            "🪵",
            "Flooring",
            "Hardwood, tile, LVP install and repair, carpet replacement",
        ),
        (
            "🐛",
            "Pest Control",
            "Extermination, prevention programs, termite inspections",
        ),
        (
            "🛠️",
            "Appliance Repair",
            "Washers, dryers, refrigerators, dishwashers, stoves",
        ),
        (
            "🔐",
            "Locksmith",
            "Rekeying, lock installs, smart lock setup, access control",
        ),
        (
            "🔍",
            "Inspection",
            "Move-in/out inspections, general home inspections, code checks",
        ),
        (
            "📦",
            "Movers",
            "Residential & commercial moves, furniture, appliance relocation",
        ),
        (
            "🗑️",
            "Junk Removal",
            "Tenant cleanouts, bulk hauling, estate clearances, debris removal",
        ),
        (
            "🏊",
            "Pool & Spa",
            "Cleaning, chemical balance, equipment repair, opening/closing",
        ),
        (
            "📷",
            "Security",
            "Camera installs, alarm systems, smart home setup",
        ),
        (
            "☀️",
            "Solar",
            "Panel installs, maintenance, battery storage, inspections",
        ),
        (
            "🏗️",
            "General Contractor",
            "Renovations, additions, unit upgrades, larger project management",
        ),
    ]
    .into_iter()
    .map(|(icon, name, desc)| (icon.to_string(), name.to_string(), desc.to_string()))
    .collect()
}

fn split_trade_label(label: &str) -> (String, String) {
    label
        .split_once(' ')
        .map(|(icon, name)| (icon.to_string(), name.to_string()))
        .unwrap_or_else(|| ("🔧".to_string(), label.to_string()))
}

// ── Features ──────────────────────────────────────────────────────────────────

#[component]
fn VendorFeatures(
    #[prop(default = None)] override_block: Option<FeatureGridBlock>,
) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Platform features".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Built for tradespeople, not accountants.".to_string());
    // CMS section blocks overlay these defaults when seeded.
    let default_features = vec![
        ("search",         "Marketplace listing",    "Your profile surfaces to every landlord and PM on Folio — searchable by trade, location, availability, and Atlas Score. Free, forever."),
        ("assignment",     "Instant job dispatch",   "Receive jobs with photos, descriptions, and full property context. No phone tag, no back-and-forth — just the info you need to say yes or no."),
        ("receipt_long",   "One-tap invoicing",      "Build an invoice from a job template in 60 seconds. Send it directly to the property manager. No spreadsheets. No email chains."),
        ("account_balance","24-hour payment",        "Approved invoices pay out via ACH within 24 hours. Know exactly when money is coming and never chase a check again."),
        ("star",           "Atlas Score reputation", "Every completed job builds your Atlas Score — a verified reputation engine based on response time, work quality, reliability, and pricing accuracy."),
        ("trending_up",    "Job analytics",          "Earnings, job history, response rate, and client ratings — so you know what's working."),
        ("groups",         "Multi-tech accounts",    "Running a crew? Business plan lets you add technicians under your company profile. Each gets their own login and job queue."),
        ("local_shipping", "Branded company profile","Customize your Folio profile with your business name, logo, trade specialties, and service area to stand out in every search."),
    ];
    let features: Vec<(String, String, String)> = override_block
        .and_then(|block| (!block.items.is_empty()).then_some(block.items))
        .map(|items| {
            items
                .into_iter()
                .map(|item| (item.icon, item.title, item.description))
                .collect()
        })
        .unwrap_or_else(|| {
            default_features
                .into_iter()
                .map(|(icon, title, desc)| (icon.to_string(), title.to_string(), desc.to_string()))
                .collect()
        });

    view! {
        <section id="vendor-features" class="mktg-section"
            style="background:rgba(6,214,160,.03);border-top:1px solid rgba(6,214,160,.1);border-bottom:1px solid rgba(6,214,160,.1);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">{eyebrow}</p>
                <h2 class="mktg-section-h2">{heading}</h2>
                <div class="mktg-feature-grid">
                    {features.into_iter().map(|(icon, title, desc)| view! {
                        <div class="mktg-feature-card">
                            <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">{icon}</span>
                            <h3>{title}</h3>
                            <p>{desc}</p>
                        </div>
                    }).collect_view()}
                </div>
            </div>
        </section>
    }
}

// ── Profile Previews — CSS-only radio tabs ────────────────────────────────────
///
/// Three views showing how a vendor appears across Folio surfaces:
///   Tab 1 — Your Dashboard (the vendor's own earnings + scorecard view)
///   Tab 2 — Network Search  (how a PM/landlord finds you)
///   Tab 3 — Service Finder  (consumer/tenant-facing card)
#[component]
fn VendorProfilePreviews() -> impl IntoView {
    view! {
        <section id="vendor-preview" class="mktg-section"
            style="background:rgba(255,107,53,.02);border-top:1px solid rgba(255,107,53,.08);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Your presence on the platform"</p>
                <h2 class="mktg-section-h2">"See exactly how you show up."</h2>
                <p class="mktg-section-sub" style="max-width:580px;margin:0 auto 2.5rem;">
                    "Your Folio profile surfaces differently depending on who's looking — but every \
                     view is powered by verified job data, not just star ratings."
                </p>

                <div class="asp-outer">
                    <p class="asp-caption">"↓ Click any tab to explore"</p>

                    // Radios BEFORE tabs and window — CSS sibling combinator requires this
                    <input type="radio" name="vnd" id="vnd-t1" class="asp-radio" checked/>
                    <input type="radio" name="vnd" id="vnd-t2" class="asp-radio"/>
                    <input type="radio" name="vnd" id="vnd-t3" class="asp-radio"/>

                    <div class="asp-tabs">
                        <label for="vnd-t1" class="asp-tab-label">"📊 Your Dashboard"</label>
                        <label for="vnd-t2" class="asp-tab-label">"🔍 Network Search"</label>
                        <label for="vnd-t3" class="asp-tab-label">"🛠 Service Finder"</label>
                    </div>

                    <div class="asp-window">
                        <div class="asp-chrome-bar">
                            <span class="asp-dot asp-dot-red"></span>
                            <span class="asp-dot asp-dot-yellow"></span>
                            <span class="asp-dot asp-dot-green"></span>
                            <span class="asp-url">"app.folio.co/vendor/dashboard"</span>
                        </div>

                        <div style="padding:1.25rem 1.5rem;">

                            // ── TAB 1: Dashboard / Scorecard ─────────────────
                            <div class="asp-panel" data-tab="1">
                                <div class="asp-card-row" style="margin-bottom:1rem;align-items:flex-start;gap:.85rem;">
                                    <div class="asp-avatar asp-avatar-lg" style="background:linear-gradient(135deg,#ff6b35,#ff9a3c);">"MR"</div>
                                    <div style="flex:1;">
                                        <div style="font-size:.95rem;font-weight:700;color:#e2e8f0;margin-bottom:.2rem;">
                                            "Martinez Plumbing LLC"
                                        </div>
                                        <div style="font-size:.78rem;color:#64748b;margin-bottom:.35rem;">
                                            "🚿 Plumbing · Miami-Dade, FL · 12 mi radius"
                                        </div>
                                        <div style="display:flex;gap:.35rem;flex-wrap:wrap;">
                                            <span class="asp-status asp-status--green">"✓ Verified"</span>
                                            <span class="asp-status asp-status--blue">"Elite · Score 94"</span>
                                            <span class="asp-status asp-status--green">"Available Mon–Fri"</span>
                                        </div>
                                    </div>
                                </div>
                                <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                    <div class="asp-stat-card">
                                        <div class="asp-stat-label">"Jobs · YTD"</div>
                                        <div class="asp-stat-value">"147"</div>
                                        <div class="asp-stat-delta asp-delta-up">"↑ 18 vs last yr"</div>
                                    </div>
                                    <div class="asp-stat-card">
                                        <div class="asp-stat-label">"Earned · YTD"</div>
                                        <div class="asp-stat-value" style="color:#22c55e;">"$41.8K"</div>
                                        <div class="asp-stat-delta asp-delta-up">"↑ 24%"</div>
                                    </div>
                                    <div class="asp-stat-card">
                                        <div class="asp-stat-label">"Avg ticket"</div>
                                        <div class="asp-stat-value">"$285"</div>
                                    </div>
                                    <div class="asp-stat-card">
                                        <div class="asp-stat-label">"On-time rate"</div>
                                        <div class="asp-stat-value" style="color:#818cf8;">"98%"</div>
                                    </div>
                                </div>
                                <div class="asp-section-hdr">"Atlas Score dimensions"</div>
                                <div style="display:flex;flex-direction:column;gap:.55rem;">
                                    <div style="display:grid;grid-template-columns:130px 1fr 40px;gap:.5rem;align-items:center;">
                                        <span style="font-size:.78rem;color:#e2e8f0;">"Response Time"</span>
                                        <div style="height:6px;background:rgba(255,255,255,.1);border-radius:3px;overflow:hidden;">
                                            <div style="height:100%;width:97%;background:linear-gradient(90deg,#06d6a0,#00b894);border-radius:3px;"></div>
                                        </div>
                                        <span style="font-size:.7rem;color:#64748b;text-align:right;">"97"</span>
                                    </div>
                                    <div style="display:grid;grid-template-columns:130px 1fr 40px;gap:.5rem;align-items:center;">
                                        <span style="font-size:.78rem;color:#e2e8f0;">"Work Quality"</span>
                                        <div style="height:6px;background:rgba(255,255,255,.1);border-radius:3px;overflow:hidden;">
                                            <div style="height:100%;width:96%;background:linear-gradient(90deg,#06d6a0,#00b894);border-radius:3px;"></div>
                                        </div>
                                        <span style="font-size:.7rem;color:#64748b;text-align:right;">"96"</span>
                                    </div>
                                    <div style="display:grid;grid-template-columns:130px 1fr 40px;gap:.5rem;align-items:center;">
                                        <span style="font-size:.78rem;color:#e2e8f0;">"Reliability"</span>
                                        <div style="height:6px;background:rgba(255,255,255,.1);border-radius:3px;overflow:hidden;">
                                            <div style="height:100%;width:93%;background:linear-gradient(90deg,#06d6a0,#00b894);border-radius:3px;"></div>
                                        </div>
                                        <span style="font-size:.7rem;color:#64748b;text-align:right;">"93"</span>
                                    </div>
                                    <div style="display:grid;grid-template-columns:130px 1fr 40px;gap:.5rem;align-items:center;">
                                        <span style="font-size:.78rem;color:#e2e8f0;">"Pricing Accuracy"</span>
                                        <div style="height:6px;background:rgba(255,255,255,.1);border-radius:3px;overflow:hidden;">
                                            <div style="height:100%;width:91%;background:linear-gradient(90deg,#f59e0b,#e67e22);border-radius:3px;"></div>
                                        </div>
                                        <span style="font-size:.7rem;color:#64748b;text-align:right;">"91"</span>
                                    </div>
                                </div>
                                <div class="asp-callout">
                                    "<strong>📈 12-month trend: ↑ +6 pts</strong> — Every completed job updates your score automatically. The more you work on Folio, the stronger your profile gets."
                                </div>
                            </div>

                            // ── TAB 2: Network Search (PM/Landlord view) ──────
                            <div class="asp-panel" data-tab="2">
                                <div style="display:flex;align-items:center;gap:.75rem;background:rgba(255,255,255,.06);border:1px solid #2a2d3a;border-radius:8px;padding:.6rem 1rem;margin-bottom:1rem;">
                                    <span style="color:#64748b;font-size:.85rem;">"🔍"</span>
                                    <span style="color:#64748b;font-size:.85rem;">"Plumbing · Miami, FL · Available this week"</span>
                                    <span class="asp-status asp-status--green" style="margin-left:auto;">"3 results"</span>
                                </div>
                                <div style="display:flex;flex-direction:column;gap:.75rem;">
                                    // Your card — #1 ranked, highlighted
                                    <div style="display:flex;align-items:flex-start;gap:.75rem;background:rgba(255,107,53,.05);border:1px solid rgba(255,107,53,.25);border-radius:10px;padding:.9rem 1rem;">
                                        <span style="font-size:.75rem;font-weight:800;color:#ff6b35;padding-top:.15rem;min-width:1.5rem;">"#1"</span>
                                        <div class="asp-avatar" style="width:36px;height:36px;font-size:.8rem;border-radius:9px;background:linear-gradient(135deg,#ff6b35,#ff9a3c);color:#fff;">"MR"</div>
                                        <div style="flex:1;min-width:0;">
                                            <div style="display:flex;align-items:center;gap:.4rem;flex-wrap:wrap;">
                                                <strong style="font-size:.9rem;color:#e2e8f0;">"Martinez Plumbing LLC"</strong>
                                                <span class="asp-status asp-status--green">"✓ Verified"</span>
                                                <span class="asp-status asp-status--blue">"94"</span>
                                            </div>
                                            <div style="font-size:.78rem;color:#64748b;margin-top:.15rem;">
                                                "🚿 Plumbing · 3.2 mi away · "
                                                <span style="color:#22c55e;font-weight:600;">"Available Mon–Fri"</span>
                                            </div>
                                            <div style="display:flex;gap:.4rem;margin-top:.4rem;flex-wrap:wrap;">
                                                <span class="asp-status asp-status--gray">"147 jobs"</span>
                                                <span class="asp-status asp-status--gray">"Avg $285"</span>
                                                <span class="asp-status asp-status--green">"98% on-time"</span>
                                                <span class="asp-status asp-status--green">"Replied in 4 min"</span>
                                            </div>
                                        </div>
                                        <button class="asp-btn asp-btn-approve" style="flex-shrink:0;">"Dispatch →"</button>
                                    </div>
                                    // Competitors, dimmed
                                    <div style="display:flex;align-items:flex-start;gap:.75rem;background:rgba(255,255,255,.03);border:1px solid rgba(255,255,255,.07);border-radius:10px;padding:.9rem 1rem;opacity:.55;">
                                        <span style="font-size:.75rem;font-weight:800;color:#64748b;padding-top:.15rem;min-width:1.5rem;">"#2"</span>
                                        <div class="asp-avatar" style="width:36px;height:36px;font-size:.8rem;border-radius:9px;background:rgba(255,255,255,.08);color:#64748b;">"JT"</div>
                                        <div style="flex:1;">
                                            <div style="font-size:.88rem;color:#e2e8f0;">"Joe's Plumbing"</div>
                                            <div style="font-size:.75rem;color:#64748b;margin-top:.15rem;">"🚿 Plumbing · 7.8 mi away · Next available Thu · Score: 71"</div>
                                        </div>
                                    </div>
                                    <div style="display:flex;align-items:flex-start;gap:.75rem;background:rgba(255,255,255,.02);border:1px solid rgba(255,255,255,.06);border-radius:10px;padding:.9rem 1rem;opacity:.4;">
                                        <span style="font-size:.75rem;font-weight:800;color:#64748b;padding-top:.15rem;min-width:1.5rem;">"#3"</span>
                                        <div class="asp-avatar" style="width:36px;height:36px;font-size:.8rem;border-radius:9px;background:rgba(255,255,255,.05);color:#64748b;">"RS"</div>
                                        <div style="flex:1;">
                                            <div style="font-size:.88rem;color:#e2e8f0;">"Reliable Solutions"</div>
                                            <div style="font-size:.75rem;color:#64748b;margin-top:.15rem;">"🚿 Plumbing · 11.2 mi away · Next available Fri · Score: 58"</div>
                                        </div>
                                    </div>
                                </div>
                                <div class="asp-callout">
                                    "<strong>Your Atlas Score = your rank.</strong> Higher score means first slot in every PM and landlord's search."
                                </div>
                            </div>

                            // ── TAB 3: Service Finder (consumer-facing) ───────
                            <div class="asp-panel" data-tab="3">
                                <div style="font-size:.75rem;color:#64748b;margin-bottom:1rem;">"Plumbers near Miami, FL · 3 verified vendors"</div>
                                <div style="background:rgba(255,255,255,.04);border:1px solid rgba(255,255,255,.09);border-radius:12px;padding:1.25rem;">
                                    <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:1rem;">
                                        <div style="display:flex;gap:1rem;align-items:center;">
                                            <div class="asp-avatar asp-avatar-lg" style="background:linear-gradient(135deg,#ff6b35,#ff9a3c);">"MR"</div>
                                            <div>
                                                <div style="display:flex;align-items:center;gap:.5rem;">
                                                    <strong style="font-size:.95rem;color:#e2e8f0;">"Martinez Plumbing LLC"</strong>
                                                    <span class="asp-status asp-status--green">"✓"</span>
                                                </div>
                                                <div style="font-size:.78rem;color:#64748b;margin-top:.1rem;">"Licensed · Insured · Miami-Dade"</div>
                                                <div style="display:flex;align-items:center;gap:.35rem;margin-top:.3rem;">
                                                    <span style="color:#f59e0b;">"★★★★★"</span>
                                                    <strong style="font-size:.85rem;color:#e2e8f0;">"4.9"</strong>
                                                    <span style="font-size:.75rem;color:#64748b;">"(147 reviews)"</span>
                                                </div>
                                            </div>
                                        </div>
                                        <div style="text-align:right;flex-shrink:0;">
                                            <div style="font-size:.7rem;color:#64748b;">"Starting from"</div>
                                            <div style="font-size:1.4rem;font-weight:800;color:#e2e8f0;">"$95"</div>
                                            <div style="font-size:.68rem;color:#64748b;">"service call"</div>
                                        </div>
                                    </div>
                                    <div style="display:flex;gap:.4rem;flex-wrap:wrap;margin-top:.9rem;">
                                        <span class="asp-status asp-status--green">"Available today"</span>
                                        <span class="asp-status asp-status--gray">"Emergency service"</span>
                                        <span class="asp-status asp-status--gray">"Free estimate"</span>
                                        <span class="asp-status asp-status--gray">"ACH accepted"</span>
                                    </div>
                                    <div style="border-top:1px solid rgba(255,255,255,.07);margin-top:.9rem;padding-top:.75rem;">
                                        <div style="font-size:.68rem;font-weight:700;color:#64748b;text-transform:uppercase;letter-spacing:.06em;margin-bottom:.5rem;">"Recent jobs"</div>
                                        <div style="font-size:.78rem;color:#64748b;display:flex;justify-content:space-between;">"Water heater replacement · 2 days ago"<span style="color:#f59e0b;">"★ 5.0"</span></div>
                                        <div style="font-size:.78rem;color:#64748b;display:flex;justify-content:space-between;margin-top:.3rem;">"Drain clearing · 1 week ago"<span style="color:#f59e0b;">"★ 5.0"</span></div>
                                        <div style="font-size:.78rem;color:#64748b;display:flex;justify-content:space-between;margin-top:.3rem;">"Fixture replacement · 2 weeks ago"<span style="color:#f59e0b;">"★ 4.8"</span></div>
                                    </div>
                                    <div style="display:flex;gap:.75rem;margin-top:1rem;">
                                        <a href="#vendor-signup" class="mktg-btn-accent" style="flex:1;text-align:center;padding:.6rem .75rem;font-size:.85rem;">"Book now"</a>
                                        <button style="flex:1;padding:.6rem;border-radius:8px;border:1px solid #2a2d3a;background:none;color:#64748b;font-size:.82rem;cursor:pointer;">"Get estimate"</button>
                                    </div>
                                </div>
                                <div class="asp-callout">
                                    "<strong>🌐 Consumer visibility</strong> — Your profile is discoverable not just by PMs but also by tenants and homeowners searching for services in your area."
                                </div>
                            </div>

                        </div>
                    </div>

                </div>
            </div>
        </section>
    }
}

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn VendorPricing(plans: Vec<MarketingPlan>, hero: HeroContent) -> impl IntoView {
    let plans = if plans.is_empty() {
        vendor_fallback_plans()
    } else {
        plans
    };

    view! {
        <MarketingPricingGrid
            plans=plans
            section_id="vendor-pricing".to_string()
            eyebrow=hero.pricing_eyebrow.clone().unwrap_or_else(|| "Pricing".to_string())
            heading=hero.pricing_heading.clone().unwrap_or_else(|| "Start free. Upgrade when you're ready.".to_string())
            subtitle=hero.pricing_subtitle.clone().unwrap_or_else(|| "Every vendor gets a marketplace profile and can accept jobs at no cost. Paid plans unlock the tools that help you win more work.".to_string())
            default_cta_href="#vendor-signup".to_string()
        />
    }
}

fn vendor_fallback_plans() -> Vec<MarketingPlan> {
    // CMS/product pricing overlays this fallback once plans are seeded.
    vec![
        MarketingPlan {
            slug: "basic".into(),
            name: "Basic".into(),
            tagline: "Free forever".into(),
            price_cents: 0,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Forever,
            features: vec![
                "Marketplace profile".into(),
                "Accept & complete jobs".into(),
                "In-platform invoicing".into(),
                "ACH payment in 24h".into(),
            ],
            cta_label: "Join free".into(),
            cta_href: Some("#vendor-signup".into()),
            is_featured: false,
            sort_order: 0,
        },
        MarketingPlan {
            slug: "pro".into(),
            name: "Pro Vendor".into(),
            tagline: "Priority placement".into(),
            price_cents: 2900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Basic".into(),
                "Priority search placement".into(),
                "Auto-invoicing templates".into(),
                "Job analytics dashboard".into(),
                "Verified badge".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#vendor-signup".into()),
            is_featured: true,
            sort_order: 1,
        },
        MarketingPlan {
            slug: "business".into(),
            name: "Business".into(),
            tagline: "0% platform fee".into(),
            price_cents: 7900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Pro Vendor".into(),
                "0% platform fee on jobs".into(),
                "Multi-tech accounts".into(),
                "Branded company profile".into(),
                "Dedicated account manager".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#vendor-signup".into()),
            is_featured: false,
            sort_order: 2,
        },
    ]
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn VendorCta(#[prop(default = None)] override_block: Option<CtaBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Open to all trades".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Stop waiting for referrals. Start getting jobs.".to_string());
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| "Join the Folio vendor marketplace and get connected to property managers and landlords who need your services — starting today.".to_string());
    let button_label = override_block
        .as_ref()
        .and_then(|block| block.button_label.clone())
        .unwrap_or_else(|| "Join the marketplace →".to_string());
    let button_href = override_block
        .as_ref()
        .and_then(|block| block.button_href.clone())
        .unwrap_or_else(|| "#vendor-signup".to_string());

    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#f59e0b;">{eyebrow}</p>
                <h2 class="mktg-cta-h2">{heading}</h2>
                <p class="mktg-cta-sub">{subhead}</p>
                <a href=button_href class="mktg-btn-accent mktg-btn-lg" id="vendor-cta-btn">{button_label}</a>
                <p style="margin-top:16px;font-size:12px;color:var(--mk-muted);">"Free to join. No credit card required."</p>
            </div>
        </section>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn VendorFooter(#[prop(default = None)] override_block: Option<FooterBlock>) -> impl IntoView {
    use crate::components::marketing_footer::MarketingFooter;
    let tagline = override_block
        .as_ref()
        .and_then(|block| block.tagline.clone())
        .unwrap_or_else(|| "The Landlord OS · Vendor Marketplace".to_string());
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
