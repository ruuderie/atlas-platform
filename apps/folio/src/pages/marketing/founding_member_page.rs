//! FoundingMemberPage — lifetime license / founding member fundraising page.
//!
//! Served at: `/founding`
//!
//! Lifetime licenses are role-specific — a landlord buys a Landlord lifetime,
//! a broker buys a Broker lifetime. Nobody buys "all access" — that's not how
//! real estate professionals think about their tools.
//!
//! # Fundraising math (at full sell-through):
//!   Landlord slots:  500 × avg $550  = ~$275,000
//!   Broker slots:    200 × avg $800  = ~$160,000
//!   PM slots:        100 × avg $1,200 = ~$120,000
//!   Vendor slots:    300 × $199      = ~$60,000
//!   Total potential: ~$615,000 committed upfront from the user base.

use crate::components::marketing_nav::{MarketingNav, MarketingNavSectionLink};
use crate::pages::marketing::hero_content::HeroContent;
use crate::pages::marketing::market_landing_page::LandingPageData;
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use shared_ui::marketing::{FolioMarketingSlug, FoundingSpotTier};

const FOUNDING_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Landlords",
        href: "#founding-landlord",
    },
    MarketingNavSectionLink {
        label: "Brokers",
        href: "#founding-broker",
    },
    MarketingNavSectionLink {
        label: "PMs",
        href: "#founding-pm",
    },
    MarketingNavSectionLink {
        label: "Vendors",
        href: "#founding-vendor",
    },
    MarketingNavSectionLink {
        label: "FAQ",
        href: "#founding-faq",
    },
];

// ── Spot availability (CMS-overridable via hero_payload.spot_inventory) ───────

#[derive(Clone, Copy, Debug)]
struct Spots {
    total: u32,
    taken: u32,
}
impl Spots {
    fn left(&self) -> u32 {
        self.total.saturating_sub(self.taken)
    }
    fn pct(&self) -> u32 {
        (self.taken * 100) / self.total.max(1)
    }
}

#[derive(Clone, Copy, Debug)]
struct SpotInventory {
    ll_grow: Spots,
    ll_pro: Spots,
    ll_investor: Spots,
    br_solo: Spots,
    br_team: Spots,
    br_firm: Spots,
    pm_starter: Spots,
    pm_growth: Spots,
    vd_pro: Spots,
}

impl Default for SpotInventory {
    fn default() -> Self {
        Self {
            ll_grow: default_spot(FoundingSpotTier::LlGrow),
            ll_pro: default_spot(FoundingSpotTier::LlPro),
            ll_investor: default_spot(FoundingSpotTier::LlInvestor),
            br_solo: default_spot(FoundingSpotTier::BrSolo),
            br_team: default_spot(FoundingSpotTier::BrTeam),
            br_firm: default_spot(FoundingSpotTier::BrFirm),
            pm_starter: default_spot(FoundingSpotTier::PmStarter),
            pm_growth: default_spot(FoundingSpotTier::PmGrowth),
            vd_pro: default_spot(FoundingSpotTier::VdPro),
        }
    }
}

impl SpotInventory {
    fn set(&mut self, tier: FoundingSpotTier, spots: Spots) {
        match tier {
            FoundingSpotTier::LlGrow => self.ll_grow = spots,
            FoundingSpotTier::LlPro => self.ll_pro = spots,
            FoundingSpotTier::LlInvestor => self.ll_investor = spots,
            FoundingSpotTier::BrSolo => self.br_solo = spots,
            FoundingSpotTier::BrTeam => self.br_team = spots,
            FoundingSpotTier::BrFirm => self.br_firm = spots,
            FoundingSpotTier::PmStarter => self.pm_starter = spots,
            FoundingSpotTier::PmGrowth => self.pm_growth = spots,
            FoundingSpotTier::VdPro => self.vd_pro = spots,
        }
    }
}

fn default_spot(tier: FoundingSpotTier) -> Spots {
    let (total, taken) = tier.default_spots();
    Spots { total, taken }
}

fn spot(inventory: Option<&serde_json::Value>, tier: FoundingSpotTier) -> Spots {
    let fallback = default_spot(tier);
    let Some(raw) = inventory.and_then(|v| v.get(tier.as_str())) else {
        return fallback;
    };

    let total = raw
        .get("total")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(fallback.total);
    let taken = raw
        .get("taken")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(fallback.taken);

    Spots { total, taken }
}

fn spots_from_hero(hero_payload: &serde_json::Value) -> SpotInventory {
    let inventory = hero_payload.get("spot_inventory");
    let mut spots = SpotInventory::default();
    for tier in FoundingSpotTier::ALL {
        spots.set(*tier, spot(inventory, *tier));
    }
    spots
}

fn text_or(value: Option<String>, fallback: &str) -> String {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

// ── Root component ────────────────────────────────────────────────────────────

#[server(LoadFoundingPage, "/api")]
pub async fn load_founding_page() -> Result<LandingPageData, server_fn::error::ServerFnError> {
    crate::atlas_client::fetch::<LandingPageData>(
        &FolioMarketingSlug::FolioFounding.pub_product_path(),
    )
        .await
        .map_err(|e| {
            server_fn::error::ServerFnError::new(format!("Founding page load failed: {e}"))
        })
}

#[component]
pub fn FoundingMemberPage() -> impl IntoView {
    let page = Resource::new(|| (), |_| load_founding_page());

    view! {
        <Suspense fallback=|| view! { <FoundingDefault data=None/> }>
            {move || {
                page.get().map(|result| {
                    let data = result.ok();
                    view! { <FoundingDefault data=data/> }
                })
            }}
        </Suspense>
    }
}

#[component]
fn FoundingDefault(data: Option<LandingPageData>) -> impl IntoView {
    let hero = data
        .as_ref()
        .map(|data| HeroContent::from_value(&data.hero_payload))
        .unwrap_or_default();
    let spots = data
        .as_ref()
        .map(|data| spots_from_hero(&data.hero_payload))
        .unwrap_or_default();
    let title = data
        .as_ref()
        .and_then(|data| data.meta_title.clone())
        .unwrap_or_else(|| "Folio Founding Member — Lifetime Access, No Monthly Fees".to_string());
    let description = data
        .as_ref()
        .and_then(|data| data.meta_description.clone())
        .unwrap_or_else(|| "Lock in lifetime access to Folio for a one-time payment. Choose the license for your role — landlord, broker, property manager, or vendor. Limited spots. No monthly fees, ever.".to_string());
    let cta_label = text_or(
        hero.cta_label
            .clone()
            .or_else(|| data.as_ref().map(|d| d.cta_label.clone())),
        "See founding tiers",
    );
    let cta_href = text_or(hero.cta_href.clone(), "#founding-landlord");

    view! {
        <Title text=title.clone()/>
        <Meta name="description" content=description.clone()/>
        <Meta property="og:title" content=title/>
        <Meta property="og:description" content=description/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/founding"/>

        <MarketingNav
            section_links=FOUNDING_SECTION_LINKS
            cta_label=cta_label.clone()
            cta_href="#founding-landlord"
        />
        <FoundingHero hero=hero cta_label=cta_label cta_href=cta_href/>
        <FoundingWhy/>
        <FoundingLandlord spots=spots/>
        <FoundingBroker spots=spots/>
        <FoundingPM spots=spots/>
        <FoundingVendor spots=spots/>
        <FoundingGuarantee/>
        <FoundingFaq/>
        <FoundingCta/>
        <BetaCalloutStrip/>
        <FoundingFooter/>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn FoundingHero(hero: HeroContent, cta_label: String, cta_href: String) -> impl IntoView {
    let eyebrow = text_or(hero.eyebrow, "Founding Member Program · Limited Spots");
    let headline = text_or(hero.headline, "Pay once.");
    let headline_accent = text_or(hero.headline_accent, " Use Folio forever.");
    let subhead = text_or(
        hero.subhead,
        "Lock in lifetime access at a price that will never go up. Pick the license that matches what you do — landlord, broker, property manager, or vendor. No subscription. No renewal. No surprises.",
    );

    view! {
        <section id="founding-hero" class="mktg-hero founding-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:820px;">
                <div class="mktg-eyebrow" style="color:#f59e0b;">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"workspace_premium"</span>
                    {format!(" {eyebrow}")}
                </div>
                <h1 class="mktg-hero-h1">
                    {headline}
                    <span class="mktg-h1-accent">{headline_accent}</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:580px;margin:1.5rem auto 0;">
                    {subhead}
                </p>

                // ── Role jump links ────────────────────────────────────────
                <div class="founding-jump-links">
                    <a href=cta_href class="founding-jump-btn" id="jump-cta">
                        <span class="material-symbols-outlined" style="font-variation-settings:'FILL' 1">"workspace_premium"</span>
                        {cta_label}
                    </a>
                    <a href="#founding-landlord" class="founding-jump-btn" id="jump-landlord">
                        <span class="material-symbols-outlined" style="font-variation-settings:'FILL' 1">"home"</span>
                        "Landlord"
                    </a>
                    <a href="#founding-broker" class="founding-jump-btn" id="jump-broker">
                        <span class="material-symbols-outlined" style="font-variation-settings:'FILL' 1">"real_estate_agent"</span>
                        "Broker"
                    </a>
                    <a href="#founding-pm" class="founding-jump-btn" id="jump-pm">
                        <span class="material-symbols-outlined" style="font-variation-settings:'FILL' 1">"corporate_fare"</span>
                        "Property Manager"
                    </a>
                    <a href="#founding-vendor" class="founding-jump-btn" id="jump-vendor">
                        <span class="material-symbols-outlined" style="font-variation-settings:'FILL' 1">"handyman"</span>
                        "Vendor"
                    </a>
                </div>

                <div class="mktg-stats" style="margin-top:2.5rem;border-top:1px solid var(--mk-border);padding-top:2rem;">
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"1×"</div>
                        <div class="mktg-stat-label">"payment, no recurring fees"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"4"</div>
                        <div class="mktg-stat-label">"role-specific license tracks"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"30-day"</div>
                        <div class="mktg-stat-label">"money-back guarantee"</div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Why Lifetime ──────────────────────────────────────────────────────────────

#[component]
fn FoundingWhy() -> impl IntoView {
    view! {
        <section class="mktg-section" style="padding-block:4rem;">
            <div class="mktg-section-inner">
                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:1.5rem;max-width:900px;margin:0 auto;">
                    <div class="mktg-str-card" style="padding:1.75rem;">
                        <span class="material-symbols-outlined" style="color:#f59e0b;font-variation-settings:'FILL' 1;font-size:28px;margin-bottom:.75rem;display:block">"lock"</span>
                        <h3 style="font-size:1rem;font-weight:700;margin-bottom:.5rem;">"Price locked forever"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"When we raise prices — and we will — your founding license stays exactly where it is."</p>
                    </div>
                    <div class="mktg-str-card" style="padding:1.75rem;">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1;font-size:28px;margin-bottom:.75rem;display:block">"rocket_launch"</span>
                        <h3 style="font-size:1rem;font-weight:700;margin-bottom:.5rem;">"Every future update included"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"New features in your tier are yours automatically. No upgrade fees. No version charges."</p>
                    </div>
                    <div class="mktg-str-card" style="padding:1.75rem;">
                        <span class="material-symbols-outlined" style="color:#3b82f6;font-variation-settings:'FILL' 1;font-size:28px;margin-bottom:.75rem;display:block">"badge"</span>
                        <h3 style="font-size:1rem;font-weight:700;margin-bottom:.5rem;">"Founding Member status"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"Your profile carries a permanent Founding Member badge visible across the platform and marketplace."</p>
                    </div>
                    <div class="mktg-str-card" style="padding:1.75rem;">
                        <span class="material-symbols-outlined" style="color:#a855f7;font-variation-settings:'FILL' 1;font-size:28px;margin-bottom:.75rem;display:block">"support_agent"</span>
                        <h3 style="font-size:1rem;font-weight:700;margin-bottom:.5rem;">"Priority support, always"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"Founding members go to the front of the support queue. No tiers, no waiting."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Shared: Spot Bar ──────────────────────────────────────────────────────────

#[component]
fn SpotBar(spots: Spots, accent: &'static str) -> impl IntoView {
    let pct = spots.pct();
    let left = spots.left();
    view! {
        <div class="founding-spot-bar-wrap">
            <div class="founding-spot-bar-track">
                <div
                    class="founding-spot-bar-fill"
                    style=format!("width:{}%;background:{}", pct, accent)
                ></div>
            </div>
            <span class="founding-spot-label">
                <strong style=format!("color:{}", accent)>{left}</strong>
                {format!(" of {} spots left", spots.total)}
            </span>
        </div>
    }
}

// ── Shared: Signup widget ─────────────────────────────────────────────────────

#[component]
fn FoundingSignup(
    #[prop(into)] id: String,
    #[prop(into)] label: String,
    #[prop(into)] tier_key: String,
    is_featured: bool,
) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);
    let btn_class = if is_featured {
        "founding-claim-btn founding-claim-btn--featured"
    } else {
        "founding-claim-btn"
    };

    view! {
        <div class="founding-checkout-wrap" style="margin-top:1.5rem;">
            {move || if submitted.get() {
                view! {
                    <div class="mktg-success-card">
                        <span class="material-symbols-outlined" style="font-size:1.5rem;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                        <div>
                            <div class="mktg-success-h">"Spot reserved!"</div>
                            <div class="mktg-success-sub">"We'll send your secure payment link within 24 hours."</div>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div>
                        <input
                            type="email"
                            class="mktg-wl-input"
                            placeholder="your@email.com"
                            id=id.clone()
                            prop:value=move || email.get()
                            on:input=move |e| email.set(event_target_value(&e))
                        />
                        <button
                            class=btn_class
                            data-tier=tier_key.clone()
                            on:click=move |_| {
                                if !email.get().is_empty() { submitted.set(true); }
                            }
                        >
                            {label.clone()}
                        </button>
                    </div>
                }.into_any()
            }}
        </div>
    }
}

// ── Landlord Lifetime Section ─────────────────────────────────────────────────

#[component]
fn FoundingLandlord(spots: SpotInventory) -> impl IntoView {
    view! {
        <section id="founding-landlord" class="founding-role-section founding-role-landlord">
            <div class="mktg-section-inner">
                <div class="founding-role-header">
                    <span class="material-symbols-outlined founding-role-icon" style="color:#06d6a0;font-variation-settings:'FILL' 1">"home"</span>
                    <div>
                        <p class="mktg-section-eyebrow">"For Landlords"</p>
                        <h2 class="mktg-section-h2" style="margin-bottom:.5rem;">"Landlord Lifetime License"</h2>
                        <p class="mktg-section-sub" style="margin:0;">"Own your landlord stack forever. Leases, rent collection, maintenance, STR calendar, and portfolio analytics — one payment."</p>
                    </div>
                </div>

                <div class="founding-tiers-grid" style="margin-top:2.5rem;">

                    // Grow Lifetime
                    <div class="founding-tier-card">
                        <div class="founding-tier-name">"Grow Lifetime"</div>
                        <SpotBar spots=spots.ll_grow accent="#06d6a0"/>
                        <div class="founding-price">"$299"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $29/mo · breaks even in 10 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Up to 10 units"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Rent collection & ACH"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Maintenance queue"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Tenant portal"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Vacancy marketing"</li>
                        </ul>
                        <FoundingSignup id="ll-grow-email" label="Reserve Grow Lifetime" tier_key="ll-grow" is_featured=false/>
                    </div>

                    // Pro Lifetime (featured)
                    <div class="founding-tier-card founding-tier-featured">
                        <div class="founding-popular-badge">"Most popular"</div>
                        <div class="founding-tier-name">"Pro Lifetime"</div>
                        <SpotBar spots=spots.ll_pro accent="#ff6b35"/>
                        <div class="founding-price">"$799"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $99/mo · breaks even in 8 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Up to 30 units"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Grow"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"STR host portal (full)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"STR compliance & permits"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Cohost Network access"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Portfolio analytics"</li>
                        </ul>
                        <FoundingSignup id="ll-pro-email" label="Reserve Pro Lifetime" tier_key="ll-pro" is_featured=true/>
                    </div>

                    // Investor Lifetime
                    <div class="founding-tier-card">
                        <div class="founding-tier-name">"Investor Lifetime"</div>
                        <SpotBar spots=spots.ll_investor accent="#f59e0b"/>
                        <div class="founding-price">"$1,499"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $249/mo · breaks even in 6 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Unlimited units"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Pro"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Multi-country (US, Brazil, USVI)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Bitcoin & Lightning payments"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Owner portal (for investors)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"API access"</li>
                        </ul>
                        <FoundingSignup id="ll-investor-email" label="Reserve Investor Lifetime" tier_key="ll-investor" is_featured=false/>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Broker Lifetime Section ───────────────────────────────────────────────────

#[component]
fn FoundingBroker(spots: SpotInventory) -> impl IntoView {
    view! {
        <section id="founding-broker" class="founding-role-section founding-role-alt">
            <div class="mktg-section-inner">
                <div class="founding-role-header">
                    <span class="material-symbols-outlined founding-role-icon" style="color:#3b82f6;font-variation-settings:'FILL' 1">"real_estate_agent"</span>
                    <div>
                        <p class="mktg-section-eyebrow">"For Brokers"</p>
                        <h2 class="mktg-section-h2" style="margin-bottom:.5rem;">"Broker Lifetime License"</h2>
                        <p class="mktg-section-sub" style="margin:0;">"Your brokerage's complete back-office — listing management, agent accounts, commission tracking, client CRM — once, forever."</p>
                    </div>
                </div>

                <div class="founding-tiers-grid" style="margin-top:2.5rem;">

                    // Solo Lifetime
                    <div class="founding-tier-card">
                        <div class="founding-tier-name">"Solo Lifetime"</div>
                        <SpotBar spots=spots.br_solo accent="#3b82f6"/>
                        <div class="founding-price">"$499"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $99/mo · breaks even in 5 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#3b82f6;font-variation-settings:'FILL' 1">"check"</span>"1 licensed seat"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#3b82f6;font-variation-settings:'FILL' 1">"check"</span>"Listing management"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#3b82f6;font-variation-settings:'FILL' 1">"check"</span>"Client CRM"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#3b82f6;font-variation-settings:'FILL' 1">"check"</span>"Transaction dashboard"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#3b82f6;font-variation-settings:'FILL' 1">"check"</span>"Agent profile page"</li>
                        </ul>
                        <FoundingSignup id="br-solo-email" label="Reserve Solo Lifetime" tier_key="br-solo" is_featured=false/>
                    </div>

                    // Team Lifetime (featured)
                    <div class="founding-tier-card founding-tier-featured">
                        <div class="founding-popular-badge">"Most popular"</div>
                        <div class="founding-tier-name">"Team Lifetime"</div>
                        <SpotBar spots=spots.br_team accent="#ff6b35"/>
                        <div class="founding-price">"$1,499"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $299/mo · 5 seats · breaks even in 5 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"5 agent seats"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Solo"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Commission plan tracking"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Team performance dashboard"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Shared listing pipeline"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Branded client portal"</li>
                        </ul>
                        <FoundingSignup id="br-team-email" label="Reserve Team Lifetime" tier_key="br-team" is_featured=true/>
                    </div>

                    // Firm Lifetime
                    <div class="founding-tier-card">
                        <div class="founding-tier-name">"Firm Lifetime"</div>
                        <SpotBar spots=spots.br_firm accent="#f59e0b"/>
                        <div class="founding-price">"$2,999"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $599/mo · 25 seats · breaks even in 5 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"25 agent seats"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Team"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Recruiting & onboarding module"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Multi-office support"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Scorecard analytics per agent"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"API access"</li>
                        </ul>
                        <FoundingSignup id="br-firm-email" label="Reserve Firm Lifetime" tier_key="br-firm" is_featured=false/>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Property Manager Lifetime Section ────────────────────────────────────────

#[component]
fn FoundingPM(spots: SpotInventory) -> impl IntoView {
    view! {
        <section id="founding-pm" class="founding-role-section founding-role-pm">
            <div class="mktg-section-inner">
                <div class="founding-role-header">
                    <span class="material-symbols-outlined founding-role-icon" style="color:#a855f7;font-variation-settings:'FILL' 1">"corporate_fare"</span>
                    <div>
                        <p class="mktg-section-eyebrow">"For Property Managers"</p>
                        <h2 class="mktg-section-h2" style="margin-bottom:.5rem;">"PM Lifetime License"</h2>
                        <p class="mktg-section-sub" style="margin:0;">"Trust accounting, owner portals, maintenance dispatch, vendor marketplace — everything a professional PM needs, one payment."</p>
                    </div>
                </div>

                <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:2rem;margin-top:2.5rem;max-width:760px;margin-inline:auto;">

                    // Starter PM Lifetime
                    <div class="founding-tier-card">
                        <div class="founding-tier-name">"Starter PM Lifetime"</div>
                        <SpotBar spots=spots.pm_starter accent="#a855f7"/>
                        <div class="founding-price">"$699"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $149/mo · up to 20 units · breaks even in 5 months"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#a855f7;font-variation-settings:'FILL' 1">"check"</span>"1 portfolio, up to 20 units"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#a855f7;font-variation-settings:'FILL' 1">"check"</span>"Trust accounting"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#a855f7;font-variation-settings:'FILL' 1">"check"</span>"Owner portal (per property)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#a855f7;font-variation-settings:'FILL' 1">"check"</span>"Maintenance dispatch"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#a855f7;font-variation-settings:'FILL' 1">"check"</span>"Tenant + vendor portals"</li>
                        </ul>
                        <FoundingSignup id="pm-starter-email" label="Reserve Starter PM Lifetime" tier_key="pm-starter" is_featured=false/>
                    </div>

                    // Growth PM Lifetime (featured)
                    <div class="founding-tier-card founding-tier-featured">
                        <div class="founding-popular-badge">"Best value"</div>
                        <div class="founding-tier-name">"Growth PM Lifetime"</div>
                        <SpotBar spots=spots.pm_growth accent="#ff6b35"/>
                        <div class="founding-price">"$1,499"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $299/mo · up to 100 units · AppFolio would charge $149+/mo"</div>
                        <ul class="mktg-pricing-features founding-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"5 portfolios, up to 100 units"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Starter PM"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Automated owner disbursement"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Multi-user (3 team members)"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Reporting & owner statements"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Vendor marketplace access"</li>
                        </ul>
                        <FoundingSignup id="pm-growth-email" label="Reserve Growth PM Lifetime" tier_key="pm-growth" is_featured=true/>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Vendor Lifetime Section ───────────────────────────────────────────────────

#[component]
fn FoundingVendor(spots: SpotInventory) -> impl IntoView {
    view! {
        <section id="founding-vendor" class="founding-role-section founding-role-alt">
            <div class="mktg-section-inner">
                <div class="founding-role-header">
                    <span class="material-symbols-outlined founding-role-icon" style="color:#06d6a0;font-variation-settings:'FILL' 1">"handyman"</span>
                    <div>
                        <p class="mktg-section-eyebrow">"For Vendors & Contractors"</p>
                        <h2 class="mktg-section-h2" style="margin-bottom:.5rem;">"Vendor Pro Lifetime"</h2>
                        <p class="mktg-section-sub" style="margin:0;">"Priority placement, auto-invoicing, verified badge, and 0% platform fee — for contractors who are serious about growing their book of business."</p>
                    </div>
                </div>

                <div style="display:grid;grid-template-columns:1fr;gap:2rem;margin-top:2.5rem;max-width:480px;margin-inline:auto;">
                    <div class="founding-tier-card founding-tier-featured" style="text-align:center;">
                        <div class="founding-tier-name">"Vendor Pro Lifetime"</div>
                        <SpotBar spots=spots.vd_pro accent="#06d6a0"/>
                        <div class="founding-price" style="justify-content:center;">"$199"<span class="founding-price-once">" one-time"</span></div>
                        <div class="founding-saves">"Saves $29/mo · 0% platform fee · priority placement forever"</div>
                        <ul class="mktg-pricing-features founding-features" style="text-align:left;max-width:380px;margin-inline:auto;">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Priority search placement"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"0% platform fee on every job"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Auto-invoicing templates"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Verified contractor badge"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Job analytics dashboard"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:15px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Founding Member badge"</li>
                        </ul>
                        <FoundingSignup id="vd-pro-email" label="Reserve Vendor Pro Lifetime" tier_key="vd-pro" is_featured=true/>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Guarantee strip ───────────────────────────────────────────────────────────

#[component]
fn FoundingGuarantee() -> impl IntoView {
    view! {
        <section class="mktg-section" style="padding-block:3rem;">
            <div class="mktg-section-inner">
                <div class="founding-guarantee-strip">
                    <span class="material-symbols-outlined" style="font-size:2rem;color:#06d6a0;font-variation-settings:'FILL' 1;flex-shrink:0">"verified_user"</span>
                    <div>
                        <strong style="font-size:1rem;">"30-day money-back guarantee on every founding tier."</strong>
                        <p style="color:var(--mk-muted);margin:.5rem 0 0;font-size:.9rem;">"If Folio isn't right for your business within 30 days of payment, we'll refund 100%. No forms, no friction, no questions."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── FAQ ───────────────────────────────────────────────────────────────────────

#[component]
fn FoundingFaq() -> impl IntoView {
    view! {
        <section id="founding-faq" class="mktg-section" style="background:rgba(6,214,160,.02);border-top:1px solid rgba(6,214,160,.08);border-bottom:1px solid rgba(6,214,160,.08);">
            <div class="mktg-section-inner" style="max-width:760px;">
                <p class="mktg-section-eyebrow">"FAQ"</p>
                <h2 class="mktg-section-h2">"Common questions."</h2>
                <div class="founding-faq-list">
                    <div class="founding-faq-item">
                        <h3>"What does 'lifetime' mean?"</h3>
                        <p>"You pay once. You never pay again. Every update, every new feature within your tier ships to you automatically — no upgrade fees, no version charges."</p>
                    </div>
                    <div class="founding-faq-item">
                        <h3>"Can I upgrade my tier later?"</h3>
                        <p>"Yes — you pay the price difference between your current founding tier and the higher one, not the full new price. We reward early commitment."</p>
                    </div>
                    <div class="founding-faq-item">
                        <h3>"Is the product available today?"</h3>
                        <p>"Folio is in active beta. Core features are live. Your founding license gives you immediate access to everything available now, plus all future releases."</p>
                    </div>
                    <div class="founding-faq-item">
                        <h3>"Why are spots limited?"</h3>
                        <p>"We're capping the founding program so we can give each founding member the hands-on onboarding and product access they deserve. This isn't artificial scarcity — we're serious about the commitment."</p>
                    </div>
                    <div class="founding-faq-item">
                        <h3>"What payment methods do you accept?"</h3>
                        <p>"All major credit cards, ACH, and — fitting for a platform with Lightning support — Bitcoin and Lightning Network. We'll send your secure payment link within 24 hours of reserving your spot."</p>
                    </div>
                    <div class="founding-faq-item">
                        <h3>"Is this transferable to another person?"</h3>
                        <p>"Founding licenses are tied to your business account. They are not transferable or resellable, but they cover your entire organization under the licensed tier."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn FoundingCta() -> impl IntoView {
    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#f59e0b;">"Limited time"</p>
                <h2 class="mktg-cta-h2">"The founding program closes when the spots are gone."</h2>
                <p class="mktg-cta-sub">"Monthly pricing takes over when founding spots sell out — no exceptions and no grandfathering for late signups."</p>
                <div style="display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2rem;">
                    <a href="#founding-landlord" class="mktg-btn-accent" id="founding-cta-ll">"Landlord lifetime"</a>
                    <a href="#founding-broker"   class="mktg-btn-accent" id="founding-cta-br">"Broker lifetime"</a>
                    <a href="#founding-pm"       class="mktg-btn-accent" id="founding-cta-pm">"PM lifetime"</a>
                    <a href="#founding-vendor"   class="mktg-btn-accent" id="founding-cta-vd">"Vendor lifetime"</a>
                </div>
                <p style="margin-top:1.5rem;font-size:.8rem;color:#9ca3af;">"30-day money-back guarantee · Cards, ACH, Bitcoin & Lightning accepted"</p>
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
                    <p>"Get discounted access during beta in exchange for real feedback. We review every                        application — accepted members shape the product roadmap."</p>
                </div>
                <a href="/beta" class="beta-callout-cta" id="beta-strip-cta" rel="external">
                    "Apply now"
                    <span class="material-symbols-outlined" style="font-size:16px">"arrow_forward"</span>
                </a>
            </div>
        </div>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn FoundingFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS · Founding Member Program"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/" rel="external">"For Landlords"</a>
                    <a href="/brokers" rel="external">"For Brokers"</a>
                    <a href="/property-managers" rel="external">"For PMs"</a>
                    <a href="/vendors" rel="external">"For Vendors"</a>
                    <a href="/" rel="external">"Monthly pricing"</a>
                    <a href="/login" rel="external">"Sign in"</a>
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
