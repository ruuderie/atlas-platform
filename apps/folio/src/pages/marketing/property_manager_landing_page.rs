//! PropertyManagerLandingPage — marketing page targeting property managers & PMCs.
//!
//! Served at: `/property-managers`
//!
//! Zero-auth, accessible to any visitor. Independently managed under
//! `app_id = "folio-pm"` in platform-admin so marketing can publish, A/B test,
//! and update it without a code deployment.
//!
//! # Pricing model
//! Per-portfolio / per-unit — distinct from landlord (per-door) and broker (per-seat).
//!
//! # Platform-admin
//! In the "Landing Pages" section, select "🏢 Property Manager Page" to manage
//! this page independently.

use crate::components::marketing_nav::{
    resolve_marketing_nav_cta, MarketingNav, MarketingNavRole, MarketingNavSectionLink,
    DEFAULT_MARKETING_NAV_CTA,
};
use crate::pages::marketing::block_renderer::{
    has_full_page_block, parse_section_blocks, BetaStripBlock, BlockRenderer, CtaBlock,
    FeatureGridBlock, FooterBlock, SectionBlocks,
};
use crate::pages::marketing::hero_content::HeroContent;
use crate::pages::marketing::market_landing_page::fire_lp_view_event;
use crate::pages::marketing::marketing_pricing::{
    MarketingPlan, MarketingPricingGrid, PlanBillingInterval,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use shared_ui::marketing::FolioMarketingSlug;

const PM_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Features",
        href: "#pm-features",
    },
    MarketingNavSectionLink {
        label: "How it works",
        href: "#pm-app-preview",
    },
    MarketingNavSectionLink {
        label: "Pricing",
        href: "#pm-pricing",
    },
];

// ── Server function ───────────────────────────────────────────────────────────

#[server(LoadPropertyManagerPage, "/api")]
pub async fn load_property_manager_page() -> Result<
    crate::pages::marketing::market_landing_page::LandingPageData,
    server_fn::error::ServerFnError,
> {
    crate::atlas_client::fetch::<crate::pages::marketing::market_landing_page::LandingPageData>(
        &FolioMarketingSlug::FolioPm.pub_product_path(),
    )
    .await
    .map_err(|e| {
        server_fn::error::ServerFnError::new(format!("Property manager page load failed: {e}"))
    })
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PropertyManagerLandingPage() -> impl IntoView {
    let page = Resource::new(|| (), |_| load_property_manager_page());

    view! {
        <Suspense fallback=|| view! {
            <PropertyManagerDefault
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
                            <Link rel="canonical" href="/property-managers"/>
                            <div class="folio-mktg">
                                <MarketingNav
                                    active=MarketingNavRole::PropertyManagers
                                    section_links=PM_SECTION_LINKS
                                    cta_label=cta_label
                                    cta_href="#pm-waitlist"
                                />
                                <BlockRenderer hero=data.hero_payload blocks=data.blocks_payload/>
                                <PmPricing plans=plans hero=hero/>
                                <PmFooter/>
                            </div>
                        }.into_any()
                    }
                    Ok(data) => {
                        let hero = HeroContent::from_value(&data.hero_payload);
                        let section_blocks = parse_section_blocks(&data.blocks_payload);
                        view! {
                            <PropertyManagerDefault
                                plans=data.plans
                                hero=hero
                                cta_label=resolve_marketing_nav_cta(&data.cta_label)
                                section_blocks=section_blocks
                            />
                        }.into_any()
                    }
                    Err(_) => view! {
                        <PropertyManagerDefault
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
fn PropertyManagerDefault(
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
        <Title text="Folio for Property Managers – Run Every Portfolio, Bill Every Owner"/>
        <Meta name="description" content="Folio gives property managers owner portals, trust accounting, maintenance dispatch, and multi-portfolio billing in one platform. Start free, scale to hundreds of units."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/property-managers"/>

        <MarketingNav
            active=MarketingNavRole::PropertyManagers
            section_links=PM_SECTION_LINKS
            section_link_overrides=nav_links
            cta_label=nav_cta_label
            cta_href="#pm-waitlist"
        />
        <PmHero hero=hero.clone() cta_label=cta_label/>
        <PmProblem/>
        <PmFeatures override_block=section_blocks.feature_grid.clone()/>
        <PmOwnerPortal/>
        <PmAppPreview/>
        <PmPricing plans=plans hero=hero/>
        <PmCta override_block=section_blocks.cta.clone()/>
        <BetaCalloutStrip override_block=section_blocks.beta_strip.clone()/>
        <PmFooter override_block=section_blocks.footer.clone()/>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn PmHero(hero: HeroContent, cta_label: String) -> impl IntoView {
    let eyebrow = hero.eyebrow.clone().unwrap_or_else(|| {
        "Built for property managers & PMCs · Multi-portfolio edition".to_string()
    });
    let headline = hero
        .headline
        .clone()
        .unwrap_or_else(|| "Manage every portfolio.".to_string());
    let headline_accent = hero
        .headline_accent
        .clone()
        .unwrap_or_else(|| " Impress every owner.".to_string());
    let subhead = hero.subhead.clone().unwrap_or_else(|| {
        "Professional property management runs on owner trust. Folio gives you branded portals, automated statements, trust accounting, and maintenance dispatch — so you run like a firm of 50, even when you're a team of three.".to_string()
    });
    let primary_cta = resolve_marketing_nav_cta(&cta_label);
    let primary_cta = RwSignal::new(primary_cta);
    let email = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let err_msg = RwSignal::new(String::new());
    let submit = move |_| {
        if loading.get() || email.get().trim().is_empty() {
            return;
        }
        loading.set(true);
        err_msg.set(String::new());
        let e = email.get();
        leptos::task::spawn_local(async move {
            let mut body = serde_json::json!({
                "email": e,
                "role": "Property Manager"
            });
            crate::marketing_attribution::merge_into_waitlist_body(&mut body);
            let resp = gloo_net::http::Request::post(&FolioMarketingSlug::FolioPm.waitlist_path())
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;
            loading.set(false);
            match resp {
                Ok(r) if r.ok() => submitted.set(true),
                Ok(_) => {
                    err_msg.set("We couldn't join the waitlist. Please try again.".to_string())
                }
                Err(_) => err_msg.set("Network issue. Please try again in a moment.".to_string()),
            }
        });
    };

    view! {
        <section id="pm-hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:800px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"business_center"</span>
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

                // ── Inline lead capture ────────────────────────────────────
                <div id="pm-waitlist" style="margin-top:2.5rem;" class="mktg-wl-wrap">
                    {move || if submitted.get() {
                        view! {
                            <div class="mktg-success-card">
                                <span class="material-symbols-outlined" style="font-size:2rem;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                                <div>
                                    <div class="mktg-success-h">"You're on the list!"</div>
                                    <div class="mktg-success-sub">"We'll reach out before launch with early access details."</div>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        let primary_cta_label = primary_cta.get();
                        view! {
                            <div class="mktg-wl-row">
                                <input
                                    type="email"
                                    class="mktg-wl-input"
                                    placeholder="your@email.com"
                                    id="pm-hero-email"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))
                                />
                                <button
                                    class="mktg-btn-accent"
                                    id="pm-hero-cta"
                                    disabled=move || loading.get()
                                    on:click=submit.clone()
                                >
                                    {move || if loading.get() { "Submitting…".to_string() } else { primary_cta_label.clone() }}
                                </button>
                            </div>
                            <Show when=move || !err_msg.get().is_empty() fallback=|| ()>
                                <p style="font-size:.78rem;color:#f87171;margin-top:.75rem;">{move || err_msg.get()}</p>
                            </Show>
                            <p style="font-size:.75rem;color:#6b7280;margin-top:.75rem;">"No credit card. No contracts. Cancel anytime."</p>
                        }.into_any()
                    }}
                </div>

                // ── Social proof ───────────────────────────────────────────
                <div class="mktg-stats" style="margin-top:3rem;border-top:1px solid var(--mk-border);padding-top:2rem;">
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"$2–5"</div>
                        <div class="mktg-stat-label">"per unit/mo vs $14+ at AppFolio"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"0"</div>
                        <div class="mktg-stat-label">"per-unit setup fees"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"5 min"</div>
                        <div class="mktg-stat-label">"to first owner portal"</div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Problem ───────────────────────────────────────────────────────────────────

#[component]
fn PmProblem() -> impl IntoView {
    view! {
        <section class="mktg-section" style="background:rgba(255,107,53,.03);border-top:1px solid rgba(255,107,53,.12);border-bottom:1px solid rgba(255,107,53,.12);">
            <div class="mktg-section-inner" style="text-align:center;">
                <p class="mktg-section-eyebrow" style="color:#ff6b35;">"The problem with PM software today"</p>
                <h2 class="mktg-section-h2" style="max-width:700px;margin:0 auto 1rem;">"You're running a professional business on consumer tools. Your owners deserve better."</h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 3rem;">
                    "Enterprise platforms cost $280/mo before you touch a unit. \
                     Lightweight tools lack trust accounting and owner portals. \
                     Folio is the missing middle — professional-grade at an independent PM price."
                </p>
                <div class="mktg-feature-grid" style="max-width:900px;margin:0 auto;">
                    <div class="mktg-feature-card" style="border-color:rgba(239,68,68,.2);background:rgba(239,68,68,.04);">
                        <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"warning"</span>
                        <h3>"No owner visibility"</h3>
                        <p>"Owners email constantly asking for statements. You spend hours building PDFs."</p>
                    </div>
                    <div class="mktg-feature-card" style="border-color:rgba(239,68,68,.2);background:rgba(239,68,68,.04);">
                        <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"warning"</span>
                        <h3>"Trust accounting is manual"</h3>
                        <p>"Security deposits, reserve funds, and disbursements tracked in Excel — one error away from a compliance issue."</p>
                    </div>
                    <div class="mktg-feature-card" style="border-color:rgba(239,68,68,.2);background:rgba(239,68,68,.04);">
                        <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"warning"</span>
                        <h3>"Maintenance is chaos"</h3>
                        <p>"Tenants text you. You call contractors. Nobody knows what's happening. Jobs fall through."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Features ─────────────────────────────────────────────────────────────────

#[component]
fn PmFeatures(#[prop(default = None)] override_block: Option<FeatureGridBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Platform capabilities".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| "Built for PMCs. Not adapted from something else.".to_string());
    // CMS section blocks overlay these defaults when seeded.
    let default_features = vec![
        ("account_tree", "Multi-portfolio management", "Manage dozens of client portfolios from a single dashboard. Each owner sees only their properties."),
        ("receipt_long", "Owner portals & statements", "Branded portals per owner. Monthly statements generated automatically. No more PDF emails."),
        ("account_balance", "Trust accounting", "Security deposit ledgers, reserve funds, disbursements, and reconciliation — built-in and auditable."),
        ("build", "Maintenance dispatch", "Tenants submit requests. You assign to vendors. Track status, photos, and invoices in one place."),
        ("payments", "Rent collection & disbursement", "Collect via ACH or card. Automatically split management fees and disburse to owner accounts."),
        ("description", "Lease & compliance", "Digital lease signing, renewal reminders, and jurisdiction-specific compliance checklists."),
        ("person", "Tenant portal", "Tenants pay rent, submit requests, and view their lease — reducing inbound calls by 60%."),
        ("analytics", "Portfolio analytics", "Occupancy rates, rent collection trends, maintenance costs, and NOI across all your clients."),
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
        <section id="pm-features" class="mktg-section">
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

// ── Owner Portal callout ──────────────────────────────────────────────────────

#[component]
fn PmOwnerPortal() -> impl IntoView {
    view! {
        <section id="pm-owner-portal" class="mktg-section" style="background:rgba(6,214,160,.04);border-top:1px solid rgba(6,214,160,.12);border-bottom:1px solid rgba(6,214,160,.12);">
            <div class="mktg-section-inner" style="display:grid;grid-template-columns:1fr 1fr;gap:3rem;align-items:center;">
                <div>
                    <p class="mktg-section-eyebrow">"Owner retention"</p>
                    <h2 class="mktg-section-h2" style="font-size:clamp(1.6rem,3vw,2.2rem);">"When owners can see their numbers anytime, they stop calling you."</h2>
                    <p style="color:var(--mk-muted);line-height:1.7;margin:1rem 0 1.5rem;">
                        "Every owner gets a branded portal showing their properties, \
                         monthly statements, maintenance history, and account balance. \
                         One link. No PDFs. No calls asking \"where's my money?\""
                    </p>
                    <ul style="list-style:none;padding:0;display:flex;flex-direction:column;gap:.75rem;">
                        <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>" Auto-generated monthly statements"</li>
                        <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>" Real-time maintenance status"</li>
                        <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>" Disbursement history & trust ledger"</li>
                        <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>" Branded with your company logo"</li>
                    </ul>
                    <a href="#pm-waitlist" class="mktg-btn-accent" style="margin-top:1.5rem;display:inline-block;" id="pm-portal-cta">"See it in action →"</a>
                </div>
                <div class="mktg-str-card" style="padding:2rem;min-height:280px;display:flex;flex-direction:column;justify-content:space-between;">
                    <div>
                        <div style="font-size:.75rem;color:#06d6a0;font-weight:600;letter-spacing:.1em;text-transform:uppercase;margin-bottom:1rem;">"Owner Portal — October 2026"</div>
                        <div style="display:flex;flex-direction:column;gap:.75rem;">
                            <div style="display:flex;justify-content:space-between;padding:.75rem;background:rgba(255,255,255,.04);border-radius:8px;border:1px solid rgba(255,255,255,.06);">
                                <span style="color:var(--mk-text);font-size:.9rem;">"Net income"</span>
                                <span style="color:#06d6a0;font-weight:600;">"$4,280"</span>
                            </div>
                            <div style="display:flex;justify-content:space-between;padding:.75rem;background:rgba(255,255,255,.04);border-radius:8px;border:1px solid rgba(255,255,255,.06);">
                                <span style="color:var(--mk-text);font-size:.9rem;">"Management fee"</span>
                                <span style="color:var(--mk-muted);font-weight:600;">"-$428"</span>
                            </div>
                            <div style="display:flex;justify-content:space-between;padding:.75rem;background:rgba(255,255,255,.04);border-radius:8px;border:1px solid rgba(255,255,255,.06);">
                                <span style="color:var(--mk-text);font-size:.9rem;">"Disbursed to owner"</span>
                                <span style="color:#f59e0b;font-weight:600;">"$3,667"</span>
                            </div>
                        </div>
                    </div>
                    <div style="margin-top:1.5rem;padding-top:1rem;border-top:1px solid rgba(255,255,255,.06);display:flex;justify-content:space-between;align-items:center;">
                        <span style="color:var(--mk-muted);font-size:.85rem;">"Owner payout"</span>
                        <span style="color:#f59e0b;font-size:1.1rem;font-weight:700;">"$3,667"</span>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn PmPricing(plans: Vec<MarketingPlan>, hero: HeroContent) -> impl IntoView {
    let plans = if plans.is_empty() {
        pm_fallback_plans()
    } else {
        plans
    };

    view! {
        <section class="mktg-section" style="padding-bottom:0;">
            <div class="mktg-section-inner">
                // ── Audience qualifier ────────────────────────────────────────
                // If you only manage your own properties, you are a landlord,
                // not a PM. This strip routes mis-matched visitors before
                // they select a tier that doesn't match their actual use case.
                <div class="mktg-pricing-audience-guard">
                    <span class="material-symbols-outlined" style="font-size:18px;color:#f59e0b">"info"</span>
                    <div>
                        "These plans are for businesses that "
                        <strong>"charge clients a management fee"</strong>
                        " and manage properties on their behalf. \
                          If you only manage your own portfolio, "
                        <a href="/" rel="external">"see Landlord plans →"</a>
                    </div>
                </div>
            </div>
        </section>
        <MarketingPricingGrid
            plans=plans
            section_id="pm-pricing".to_string()
            eyebrow=hero.pricing_eyebrow.clone().unwrap_or_else(|| "Pricing".to_string())
            heading=hero.pricing_heading.clone().unwrap_or_else(|| "Pay per portfolio, not per feature.".to_string())
            subtitle=hero.pricing_subtitle.clone().unwrap_or_else(|| "Every plan includes trust accounting, owner portals, and maintenance dispatch. No surprise add-ons.".to_string())
            default_cta_href="#pm-waitlist".to_string()
        />
        <div class="mktg-section" style="padding-top:0;">
            <div class="mktg-section-inner">
                <div class="mktg-pricing-pm-callout">
                    <span class="material-symbols-outlined" style="font-size:20px;color:#f59e0b">"trending_down"</span>
                    <div>
                        <strong>"AppFolio starts at $280/mo minimum."</strong>
                        " Buildium charges per unit after 20 — costs balloon as your portfolio grows. \
                          Folio Growth PM covers 5 client portfolios for $199, priced per portfolio, not per unit. "
                        <strong>"No surprise overage charges."</strong>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn pm_fallback_plans() -> Vec<MarketingPlan> {
    // CMS/product pricing overlays this fallback once plans are seeded.
    vec![
        MarketingPlan {
            slug: "starter".into(),
            name: "Starter PM".into(),
            tagline: "1 client portfolio · up to 20 units".into(),
            price_cents: 9900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Full landlord platform".into(),
                "1 branded owner portal".into(),
                "Trust accounting ledger".into(),
                "Maintenance dispatch".into(),
                "Requires 2+ owner-clients".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#pm-waitlist".into()),
            is_featured: false,
            sort_order: 0,
        },
        MarketingPlan {
            slug: "growth".into(),
            name: "Growth PM".into(),
            tagline: "Up to 5 client portfolios · 100 units".into(),
            price_cents: 19900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Starter PM".into(),
                "5 branded owner portals".into(),
                "Auto-disbursement & fee split".into(),
                "Portfolio analytics".into(),
                "Vacancy marketing".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#pm-waitlist".into()),
            is_featured: true,
            sort_order: 1,
        },
        MarketingPlan {
            slug: "scale".into(),
            name: "Scale PM".into(),
            tagline: "Up to 15 client portfolios · 300 units".into(),
            price_cents: 39900,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Month,
            features: vec![
                "Everything in Growth PM".into(),
                "Full trust accounting suite".into(),
                "Multi-user team access".into(),
                "Priority support".into(),
                "Advanced reporting".into(),
            ],
            cta_label: DEFAULT_MARKETING_NAV_CTA.into(),
            cta_href: Some("#pm-waitlist".into()),
            is_featured: false,
            sort_order: 2,
        },
        MarketingPlan {
            slug: "enterprise".into(),
            name: "Enterprise".into(),
            tagline: "Unlimited portfolios · white-label · API".into(),
            price_cents: 0,
            currency: "USD".into(),
            billing_interval: PlanBillingInterval::Custom,
            features: vec![
                "Everything in Scale PM".into(),
                "White-label branding".into(),
                "API access & SSO".into(),
                "Dedicated onboarding".into(),
                "Uptime SLA".into(),
            ],
            cta_label: "Contact us".into(),
            cta_href: Some("#pm-waitlist".into()),
            is_featured: false,
            sort_order: 3,
        },
    ]
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn PmCta(#[prop(default = None)] override_block: Option<CtaBlock>) -> impl IntoView {
    let eyebrow = override_block
        .as_ref()
        .and_then(|block| block.eyebrow.clone())
        .unwrap_or_else(|| "Limited beta spots available".to_string());
    let heading = override_block
        .as_ref()
        .and_then(|block| block.heading.clone())
        .unwrap_or_else(|| {
            "Stop managing with spreadsheets. Start running a real business.".to_string()
        });
    let subhead = override_block
        .as_ref()
        .and_then(|block| block.subhead.clone())
        .unwrap_or_else(|| "Join the waitlist for exclusive early access. Beta members lock in founder pricing and help shape the property management features before we open to the public.".to_string());
    let button_label = override_block
        .as_ref()
        .and_then(|block| block.button_label.clone())
        .unwrap_or_else(|| "Reserve my beta spot →".to_string());
    let button_href = override_block
        .as_ref()
        .and_then(|block| block.button_href.clone())
        .unwrap_or_else(|| "#pm-waitlist".to_string());

    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#06d6a0;">{eyebrow}</p>
                <h2 class="mktg-cta-h2">{heading}</h2>
                <p class="mktg-cta-sub">{subhead}</p>
                <a href=button_href class="mktg-btn-accent mktg-btn-lg" id="pm-cta-btn">{button_label}</a>
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
        .unwrap_or_else(|| "Get discounted access during beta in exchange for real feedback. We review every application — accepted members shape the product roadmap.".to_string());
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
                <a href=button_href class="beta-callout-cta" id="beta-strip-cta-pm" rel="external">
                    {button_label}
                    <span class="material-symbols-outlined" style="font-size:16px">"arrow_forward"</span>
                </a>
            </div>
        </div>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn PmFooter(#[prop(default = None)] override_block: Option<FooterBlock>) -> impl IntoView {
    use crate::components::marketing_footer::MarketingFooter;
    let tagline = override_block
        .as_ref()
        .and_then(|block| block.tagline.clone())
        .unwrap_or_else(|| "Modern Landlord OS · Property Manager Edition".to_string());
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
        <MarketingFooter
            tagline=tagline
            show_page_anchors=true
            override_links=override_links
        />
    }
}

// ── App Preview — PM dashboard (CSS-only radio tabs) ─────────────────────────
/// Five-tab walkthrough of the Folio PM experience using pure CSS radio tabs.
/// Tabs: Portfolio · Owner Portal · Maintenance · Trust Accounting · Reports
#[component]
fn PmAppPreview() -> impl IntoView {
    view! {
        <section class="mktg-section ap-section" id="pm-app-preview"
            style="background:rgba(6,214,160,.015);border-top:1px solid rgba(6,214,160,.08);">
            <div class="mktg-container">
                <div class="ap-header">
                    <span class="mktg-label">"Inside the platform"</span>
                    <h2 class="mktg-h2">"The full PM workflow — from dispatch to disbursement"</h2>
                    <p class="mktg-sub">"Every screen below is a real view from Folio PM. 147 units, 23 owners, all of it in one dashboard — with each owner seeing only their portfolio."</p>
                </div>

                <div class="asp-outer">
                    <p class="asp-caption">"↓ Click any tab to explore"</p>

                    <input type="radio" name="pm" id="pm-t1" class="asp-radio" checked/>
                    <input type="radio" name="pm" id="pm-t2" class="asp-radio"/>
                    <input type="radio" name="pm" id="pm-t3" class="asp-radio"/>
                    <input type="radio" name="pm" id="pm-t4" class="asp-radio"/>
                    <input type="radio" name="pm" id="pm-t5" class="asp-radio"/>

                    <div class="asp-tabs">
                        <label for="pm-t1" class="asp-tab-label">"📊 Portfolio"</label>
                        <label for="pm-t2" class="asp-tab-label">"🏠 Owner Portal"</label>
                        <label for="pm-t3" class="asp-tab-label">"🔧 Maintenance"</label>
                        <label for="pm-t4" class="asp-tab-label">"🏦 Trust Accounting"</label>
                        <label for="pm-t5" class="asp-tab-label">"📈 Reports"</label>
                    </div>

                    <div class="asp-window">
                        <div class="asp-chrome-bar">
                            <span class="asp-dot asp-dot-red"></span>
                            <span class="asp-dot asp-dot-yellow"></span>
                            <span class="asp-dot asp-dot-green"></span>
                            <span class="asp-url">"pm.folio.co/portfolio"</span>
                        </div>
                        <div class="asp-shell">
                            <aside class="asp-sidebar" style="--folio-accent:#0d9488;--folio-accent2:#2dd4bf;">
                                <div class="asp-sidebar-logo" style="color:#2dd4bf;">
                                    "Folio PM"
                                    <span>"Property Manager"</span>
                                </div>
                                <a class="asp-nav-item asp-nav-item--active" style="background:rgba(13,148,136,.14);color:#2dd4bf;"><span class="asp-nav-icon">"📊"</span>"Portfolio"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"🏠"</span>"Owner Portals"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"👤"</span>"Tenants"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"📋"</span>"Leases"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"🔧"</span>"Maintenance"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"🏦"</span>"Trust Accounts"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"💳"</span>"Billing"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"📈"</span>"Reports"</a>
                            </aside>

                            <main class="asp-main">

                                // TAB 1: Portfolio overview
                                <div class="asp-panel" data-tab="1">
                                    <div class="asp-page-title">"Portfolio Overview"</div>
                                    <div class="asp-page-sub">"147 units · 23 owners · 96.3% occupancy"</div>
                                    <div class="asp-stat-grid">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Units managed"</div><div class="asp-stat-value">"147"</div><div class="asp-stat-delta asp-delta-up">"↑ 12 this qtr"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Owners / clients"</div><div class="asp-stat-value">"23"</div><div class="asp-stat-delta asp-delta-up">"3 new"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Rent collected · Jul"</div><div class="asp-stat-value" style="color:#2dd4bf;">"$182K"</div><div class="asp-stat-delta asp-delta-up">"98.6%"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Mgmt fees earned"</div><div class="asp-stat-value" style="color:#2dd4bf;">"$9,120"</div></div>
                                    </div>
                                    <div class="asp-section-hdr">"Client portfolios"</div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Owner"</th><th>"Units"</th><th>"Location"</th><th>"Collection"</th><th>"Mgmt fee"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"David & Wendy Chen"</td><td>"34"</td><td>"Miami, FL"</td><td><span class="asp-status asp-status--green">"100%"</span></td><td>"$1,710/mo"</td></tr>
                                            <tr><td>"Reginald Johnson LLC"</td><td>"18"</td><td>"Atlanta, GA"</td><td><span class="asp-status asp-status--warn">"1 late"</span></td><td>"$900/mo"</td></tr>
                                            <tr><td>"Sofia Patel Investments"</td><td>"20"</td><td>"Miami Beach"</td><td><span class="asp-status asp-status--green">"All current"</span></td><td>"$1,200/mo"</td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 2: Owner Portal
                                <div class="asp-panel" data-tab="2">
                                    <div class="asp-page-title">"Owner Portal — David & Wendy Chen"</div>
                                    <div class="asp-page-sub">"White-labeled at owners.premierpm.co · Auto-statements on the 1st"</div>
                                    <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"July net income"</div><div class="asp-stat-value" style="color:#2dd4bf;">"$12,350"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Units occupied"</div><div class="asp-stat-value">"34 / 34"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Collection rate"</div><div class="asp-stat-value">"100%"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Open work orders"</div><div class="asp-stat-value">"1"</div></div>
                                    </div>
                                    <div class="asp-section-hdr">"Owner statements"</div>
                                    <table class="asp-table">
                                        <tbody>
                                            <tr><td>"July 2026 Statement"</td><td><span class="asp-status asp-status--green">"Ready"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                            <tr><td>"June 2026 Statement"</td><td><span class="asp-status asp-status--green">"Delivered"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                            <tr><td>"May 2026 Statement"</td><td><span class="asp-status asp-status--green">"Delivered"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                        </tbody>
                                    </table>
                                    <div class="asp-callout">"<strong>✉ Auto-delivery</strong> — Monthly statements go out automatically on the 1st. Owners get their numbers, you skip the calls."</div>
                                </div>

                                // TAB 3: Maintenance
                                <div class="asp-panel" data-tab="3">
                                    <div class="asp-page-title">"Maintenance"</div>
                                    <div class="asp-page-sub">"You approve invoices so owners are never interrupted"</div>
                                    <div class="asp-card">
                                        <div class="asp-card-row">
                                            <div>
                                                <div style="display:flex;gap:.35rem;margin-bottom:.3rem;">
                                                    <span class="asp-status asp-status--warn">"Open"</span>
                                                    <span class="asp-status asp-status--red">"Emergency"</span>
                                                    <span class="asp-muted" style="font-size:.68rem;">"#WO-4421 · Jul 4"</span>
                                                </div>
                                                <div class="asp-card-title">"Water heater failure · Chen Portfolio · Unit 8"</div>
                                                <div class="asp-card-sub">"HeatWave Plumbing dispatched · ETA: Today"</div>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="asp-card">
                                        <div class="asp-card-row">
                                            <div style="flex:1;">
                                                <div style="display:flex;gap:.35rem;margin-bottom:.3rem;">
                                                    <span class="asp-status asp-status--blue">"Invoice Review"</span>
                                                    <span class="asp-muted" style="font-size:.68rem;">"#WO-4418 · Within $2,500 threshold"</span>
                                                </div>
                                                <div class="asp-card-title">"AC servicing · Patel Portfolio · 5 units"</div>
                                                <div class="asp-card-sub">"Invoice: $1,250 · CoolAir Services"</div>
                                            </div>
                                            <div style="display:flex;gap:.35rem;">
                                                <button class="asp-btn asp-btn-approve">"✓ Approve all"</button>
                                                <button class="asp-btn asp-btn-neutral">"Review"</button>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="asp-callout">"<strong>🛡 PM Approval Authority</strong> — Set a threshold per client (e.g. $2,500). Below it, you approve directly. Above it, the owner gets an email to sign off."</div>
                                </div>

                                // TAB 4: Trust Accounting
                                <div class="asp-panel" data-tab="4">
                                    <div class="asp-page-title">"Trust Accounting"</div>
                                    <div class="asp-page-sub">"Separate client ledgers · Automated disbursements · Full audit trail"</div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Client"</th><th>"Trust balance"</th><th>"Status"</th><th>"Action"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"Chen Portfolio"</td><td class="asp-credit">"$18,420"</td><td><span class="asp-status asp-status--green">"Reconciled"</span></td><td><button class="asp-btn asp-btn-approve">"Disburse $12,350"</button></td></tr>
                                            <tr><td>"Johnson LLC"</td><td class="asp-credit">"$9,840"</td><td><span class="asp-status asp-status--green">"Reconciled"</span></td><td><button class="asp-btn asp-btn-approve">"Disburse $8,100"</button></td></tr>
                                            <tr><td>"Patel Investments"</td><td class="asp-credit">"$14,290"</td><td><span class="asp-status asp-status--warn">"1 pending expense"</span></td><td><button class="asp-btn asp-btn-export">"Hold"</button></td></tr>
                                        </tbody>
                                    </table>
                                    <div class="asp-callout">"<strong>🔒 Zero commingling</strong> — Each client's funds are held in a separate ledger. You cannot accidentally mix portfolios."</div>
                                </div>

                                // TAB 5: Reports
                                <div class="asp-panel" data-tab="5">
                                    <div class="asp-page-title">"Reports & Analytics"</div>
                                    <div class="asp-page-sub">"Portfolio-wide KPIs and owner-ready exports"</div>
                                    <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Collection rate · Jul"</div><div class="asp-stat-value" style="color:#2dd4bf;">"98.6%"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Revenue per door"</div><div class="asp-stat-value">"$1,240"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Occupancy"</div><div class="asp-stat-value">"96.3%"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Mgmt fees · Jul"</div><div class="asp-stat-value" style="color:#2dd4bf;">"$9,120"</div></div>
                                    </div>
                                    <div class="asp-section-hdr">"Available reports"</div>
                                    <table class="asp-table">
                                        <tbody>
                                            <tr><td>"Monthly owner statements (23 owners)"</td><td><span class="asp-status asp-status--green">"Auto-send on 1st"</span></td><td><button class="asp-btn asp-btn-export">"⬇ Bulk"</button></td></tr>
                                            <tr><td>"Maintenance cost summary"</td><td><span class="asp-status asp-status--green">"Ready"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                            <tr><td>"Vacancy & turnover report"</td><td><span class="asp-status asp-status--green">"Ready"</span></td><td><button class="asp-btn asp-btn-export">"⬇ PDF"</button></td></tr>
                                            <tr><td>"Tax prep package (all portfolios)"</td><td><span class="asp-status asp-status--green">"Ready"</span></td><td><button class="asp-btn asp-btn-export">"⬇ ZIP"</button></td></tr>
                                        </tbody>
                                    </table>
                                </div>

                            </main>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
