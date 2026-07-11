//! BrokerLandingPage — marketing page targeting licensed brokers & real estate agents.
//!
//! Served at: `/brokers`
//!
//! This is a **zero-auth** page accessible to any visitor. It exists as an
//! independent managed page under `app_id = "folio-broker"` in platform-admin,
//! so marketing can publish, A/B test, and update it without a code deployment.
//!
//! # Backend integration
//! Calls `GET /api/pub/products/folio-broker` via `load_broker_page()` which
//! returns a `LandingPageData` record. When no published record exists the page
//! renders the built-in default content below.
//!
//! # Platform-admin
//! In the "Landing Pages" section, select the "🤝 Broker Page" app pill to
//! manage this page independently — its own slug (`brokers`), A/B variants,
//! tracking pixels, UTM presets, and funnel analytics.

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

use crate::components::marketing_nav::{
    MarketingNav, MarketingNavRole, MarketingNavSectionLink,
};

const BROKER_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink { label: "Features", href: "#broker-features" },
    MarketingNavSectionLink { label: "How it works", href: "#broker-app-preview" },
    MarketingNavSectionLink { label: "Pricing", href: "#broker-pricing" },
];

// ── Server function ───────────────────────────────────────────────────────────

/// Loads the broker landing page record from the backend.
/// Falls back gracefully — if the record is Draft or missing, `NotFound` renders.
#[server(LoadBrokerPage, "/api")]
pub async fn load_broker_page() -> Result<crate::pages::marketing::market_landing_page::LandingPageData, server_fn::error::ServerFnError> {
    crate::atlas_client::fetch::<crate::pages::marketing::market_landing_page::LandingPageData>(
        "/api/pub/products/folio-broker"
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(format!("Broker page load failed: {e}")))
}

// ── Root component ────────────────────────────────────────────────────────────

#[component]
pub fn BrokerLandingPage() -> impl IntoView {
    view! { <BrokerDefault/> }
}

// ── Default hardcoded content (used until DB record is published) ─────────────

#[component]
fn BrokerDefault() -> impl IntoView {
    let title       = "Folio for Brokers & Real Estate Agents — Run Your Whole Brokerage";
    let description = "Listing management, buyer & seller CRM, commission tracking, and agent accounts — built for licensed brokers and real estate teams.";

    view! {
        <Title text=title/>
        <Meta name="description"        content=description/>
        <Meta property="og:title"       content=title/>
        <Meta property="og:description" content=description/>
        <Meta property="og:type"        content="website"/>
        <Meta name="twitter:card"       content="summary_large_image"/>
        <Link rel="canonical" href="/brokers"/>

        <div class="folio-mktg">
            <MarketingNav
                active=MarketingNavRole::Brokers
                section_links=BROKER_SECTION_LINKS
                cta_label="Get early access"
                cta_href="/#waitlist-wrap"
            />
            <BrokerHero/>
            <BrokerFeatures/>
            <BrokerPortals/>
            <BrokerAgents/>
            <BrokerAppPreview/>
            <BrokerPricing/>
            <BrokerCta/>
            <BetaCalloutStrip/>
        <BrokerFooter/>
        </div>
    }
}

// ── Hero ─────────────────────────────────────────────────────────────────────

#[component]
fn BrokerHero() -> impl IntoView {
    view! {
        <section id="broker-hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"science"</span>
                    " Beta Access Open · Built for licensed brokers & real estate teams"
                </div>
                <h1 class="mktg-hero-h1">
                    "Close more deals."
                    <span class="mktg-h1-accent"> " Keep your commission straight."</span>
                </h1>
                <p class="mktg-hero-sub">
                    "Folio is the brokerage platform that connects your listing pipeline, client portals, \
                     agent accounts, and commission ledger — under your brand, without the enterprise price tag."
                </p>

                <div class="mktg-proof-strip" style="margin-top:32px">
                    <span class="mktg-proof-item">
                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                        "Multi-client portfolio"
                    </span>
                    <span class="mktg-proof-sep"></span>
                    <span class="mktg-proof-item">
                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                        "Branded owner portals"
                    </span>
                    <span class="mktg-proof-sep"></span>
                    <span class="mktg-proof-item">
                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                        "Agent accounts"
                    </span>
                    <span class="mktg-proof-sep"></span>
                    <span class="mktg-proof-item">
                        <span class="material-symbols-outlined" style="font-size:14px;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                        "Commission tracking"
                    </span>
                </div>

                <div style="margin-top:40px;display:flex;gap:16px;flex-wrap:wrap">
                    <a href="/#waitlist-wrap" class="mktg-btn-accent mktg-btn-lg" id="broker-hero-cta">
                        "Reserve beta access →"
                    </a>
                    <a href="/login" class="mktg-btn-signin" style="padding:14px 24px;font-size:15px" id="broker-hero-signin" rel="external">
                        "Sign in"
                    </a>
                </div>
            </div>
        </section>
    }
}

// ── Feature grid ──────────────────────────────────────────────────────────────

#[component]
fn BrokerFeatures() -> impl IntoView {
    let features = vec![
        ("home_work",    "Listing management",           "Manage all your active, pending, and closed listings in one place. Track price changes, days on market, and showing history per property."),
        ("group",        "Buyer & seller CRM",           "Every buyer and seller has a profile with their timeline, preferences, offers, and communication history. Never lose track of a deal."),
        ("payments",     "Commission tracking",          "Define commission splits per deal or per agent. Folio calculates what you're owed at close and keeps a running ledger."),
        ("person",       "Agent accounts",               "Add agents under your brokerage license. They see only their own deals and clients. You see everything. Full visibility, clear access control."),
        ("gavel",        "License & compliance",         "Track your brokerage license renewal, agent certifications, E&O insurance, and fair housing deadlines — all in one place."),
        ("analytics",    "Brokerage analytics",          "GCI by agent, conversion rates, average days to close, and deal volume trends — dashboards built for running a team, not filing taxes."),
        ("calendar_month", "Showing & appointment scheduler", "Coordinate showings across your team. Buyers, sellers, and agents see the same calendar with no double bookings."),
        ("language",     "US · Canada · Brazil",          "Operate across borders. Folio handles local compliance, licensing rules, and currency so you don't have to."),
    ];

    view! {
        <section id="broker-features" class="mktg-section mktg-features">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"The platform"</p>
                <h2 class="mktg-section-h2">"Built for the way brokerages actually run."</h2>
                <div class="mktg-feature-grid">
                    {features.into_iter().map(|(icon, title, desc)| view! {
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

// ── Owner portals ─────────────────────────────────────────────────────────────

#[component]
fn BrokerPortals() -> impl IntoView {
    view! {
        <section id="broker-portals" class="mktg-section mktg-str-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow mktg-eyebrow-light">"Client & agent experience"</p>
                <h2 class="mktg-section-h2 mktg-h2-light">"Your brand on every client touchpoint. Your agents never out of the loop."</h2>
                <p class="mktg-section-sub mktg-sub-light">
                    "Buyers track their offer status. Sellers see showing feedback and market comparisons. \
                     Agents get a deal board scoped to their pipeline. You see the whole brokerage. \
                     Every party has exactly the visibility they need — nothing more."
                </p>
                <div class="mktg-str-grid">
                    {[
                        ("home_work", "Buyer portal",
                         "Buyers log in to see the properties you've shared with them, offer status, and next steps in their transaction. No email chains for every update."),
                        ("storefront", "Seller dashboard",
                         "Sellers see their listing performance, showing requests, feedback, and offers received — without calling you every day."),
                        ("receipt",   "Transaction timeline",
                         "Every deal has a shared timeline: listing, offer, inspection, close. Both client and agent see where they are and what's next."),
                        ("lock",      "Data separation",
                         "Each client sees only their deal. Agents see only their pipeline. You see the full brokerage. Access is scoped, not shared."),
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

// ── Agent accounts ────────────────────────────────────────────────────────────

#[component]
fn BrokerAgents() -> impl IntoView {
    view! {
        <section id="broker-agents" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Agent management"</p>
                <h2 class="mktg-section-h2">"Grow your team without losing control of your commission."</h2>
                <p class="mktg-section-sub">
                    "Folio's brokerage mode lets you add agents under your license. \
                     Each agent works their own deal pipeline scoped to their clients. \
                     You see every deal across every agent — commissions, pipeline stage, \
                     and closing velocity — from a single broker view."
                </p>
                <div class="mktg-str-grid">
                    {[
                        ("person_add", "Agent accounts",
                         "Invite agents under your license. Each gets their own login scoped to their clients and active deals."),
                        ("assignment", "Deal assignment",
                         "Assign buyers, sellers, and listings to agents. Reassign instantly when a deal changes hands or your team shifts."),
                        ("supervisor_account", "Broker oversight",
                         "Full visibility across every agent, every deal, and every commission in flight — at all times."),
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

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn BrokerPricing() -> impl IntoView {
    view! {
        <section id="broker-pricing" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Pricing"</p>
                <h2 class="mktg-section-h2">"Priced for your team, not per listing."</h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 2.5rem;">"Every seat includes the full platform. Pick the plan that fits your team size — upgrade as you grow."</p>
                <div class="mktg-pricing-grid">

                    // ── Solo — independent broker/agent ───────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Solo"</span>
                        <div class="mktg-pricing-price">"$99"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"1 agent seat"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Active listing management"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Buyer & seller CRM"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Commission tracking"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Transaction timelines"</li>
                        </ul>
                        <a href="/#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="broker-pricing-solo">"Join waitlist"</a>
                    </div>

                    // ── Team — boutique firm (FEATURED) ───────────────────
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Team"</span>
                        <div class="mktg-pricing-price">"$249"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Up to 5 agent seats"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Solo"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Agent account management"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Agent profiles & bios"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Commission tracking"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Team analytics dashboard"</li>
                        </ul>
                        <a href="/#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-accent" id="broker-pricing-team">"Get early access"</a>
                    </div>

                    // ── Firm — mid-size brokerage ──────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Firm"</span>
                        <div class="mktg-pricing-price">"$499"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Up to 25 agent seats"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Team"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Branded listing portal"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Client management hub"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Brokerage analytics"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Priority support"</li>
                        </ul>
                        <a href="/#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="broker-pricing-firm">"Get early access"</a>
                    </div>

                    // ── Enterprise — large brokerage ───────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Enterprise"</span>
                        <div class="mktg-pricing-price">"Custom"</div>
                        <div class="mktg-pricing-sub">"25+ seats · white-label · SLA"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Everything in Firm"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"White-label branding"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Dedicated onboarding"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"API access & SSO"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Uptime SLA"</li>
                        </ul>
                        <a href="/#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="broker-pricing-enterprise">"Contact us"</a>
                    </div>

                </div>
            </div>
        </section>
    }
}


// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn BrokerCta() -> impl IntoView {
    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#f59e0b;">"Limited beta spots available"</p>
                <h2 class="mktg-cta-h2">"Be one of the first brokerages inside."</h2>
                <p class="mktg-cta-sub">
                    "Join the waitlist for exclusive early access. Beta members help shape \
                     the brokerage features and lock in founder pricing before we open to the public."
                </p>
                <a href="/#waitlist-wrap" class="mktg-btn-accent mktg-btn-lg" id="broker-cta-btn">
                    "Reserve my beta spot →"
                </a>
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
fn BrokerFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS · Broker Edition"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/" rel="external">"← Main page"</a>
                    <a href="/login" rel="external">"Sign in"</a>
                    <a href="#broker-pricing">"Pricing"</a>
                    <a href="#broker-features">"Features"</a>
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

// ── Skeleton ──────────────────────────────────────────────────────────────────

#[component]
fn BrokerSkeleton() -> impl IntoView {
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


// ── App Preview — Broker dashboard (CSS-only radio tabs) ─────────────────────
/// Five-tab walkthrough of the Folio Broker experience using pure CSS radio tabs.
/// Tabs: Listings CRM · Client Profiles · Commissions · Agent Accounts · Client Portal
#[component]
fn BrokerAppPreview() -> impl IntoView {
    view! {
        <section class="mktg-section ap-section" id="broker-app-preview"
            style="background:rgba(99,79,235,.015);border-top:1px solid rgba(99,79,235,.08);">
            <div class="mktg-container">
                <div class="ap-header">
                    <span class="mktg-label">"Inside the platform"</span>
                    <h2 class="mktg-h2">"From first showing to closed deal — tracked in one place"</h2>
                    <p class="mktg-sub">"Every tab below is a real screen from the Folio brokerage platform. Your brand. Your pipeline. Your commission ledger."</p>
                </div>

                <div class="asp-outer">
                    <p class="asp-caption">"↓ Click any tab to explore"</p>

                    <input type="radio" name="br" id="br-t1" class="asp-radio" checked/>
                    <input type="radio" name="br" id="br-t2" class="asp-radio"/>
                    <input type="radio" name="br" id="br-t3" class="asp-radio"/>
                    <input type="radio" name="br" id="br-t4" class="asp-radio"/>
                    <input type="radio" name="br" id="br-t5" class="asp-radio"/>

                    <div class="asp-tabs">
                        <label for="br-t1" class="asp-tab-label">"🏠 Listings CRM"</label>
                        <label for="br-t2" class="asp-tab-label">"👤 Client Profiles"</label>
                        <label for="br-t3" class="asp-tab-label">"💰 Commissions"</label>
                        <label for="br-t4" class="asp-tab-label">"👥 Agent Accounts"</label>
                        <label for="br-t5" class="asp-tab-label">"🌐 Client Portal"</label>
                    </div>

                    <div class="asp-window">
                        <div class="asp-chrome-bar">
                            <span class="asp-dot asp-dot-red"></span>
                            <span class="asp-dot asp-dot-yellow"></span>
                            <span class="asp-dot asp-dot-green"></span>
                            <span class="asp-url">"folio.co/brokers/crm"</span>
                        </div>
                        <div class="asp-shell">
                            <aside class="asp-sidebar" style="--folio-accent2:#a78bfa;">
                                <div class="asp-sidebar-logo" style="color:#a78bfa;">
                                    "Folio"
                                    <span>"Brokerage"</span>
                                </div>
                                <a class="asp-nav-item asp-nav-item--active" style="background:rgba(99,79,235,.14);color:#a78bfa;"><span class="asp-nav-icon">"🏠"</span>"Listings CRM"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"👤"</span>"Clients"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"💰"</span>"Commissions"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"👥"</span>"Agents"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"🌐"</span>"Client Portal"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"📈"</span>"Reports"</a>
                                <a class="asp-nav-item"><span class="asp-nav-icon">"⚙"</span>"Settings"</a>
                            </aside>

                            <main class="asp-main">

                                // TAB 1: Listings CRM
                                <div class="asp-panel" data-tab="1">
                                    <div class="asp-page-title">"Listings CRM"</div>
                                    <div class="asp-page-sub">"14 active listings · $6.2M pipeline"</div>
                                    <div class="asp-stat-grid">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Active listings"</div><div class="asp-stat-value">"14"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Avg days on market"</div><div class="asp-stat-value">"18"</div><div class="asp-stat-delta asp-delta-up">"↓ 4 vs last mo"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Pipeline value"</div><div class="asp-stat-value" style="color:#a78bfa;">"$6.2M"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Offers received"</div><div class="asp-stat-value">"5"</div></div>
                                    </div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Address"</th><th>"Type"</th><th>"List price"</th><th>"DOM"</th><th>"Status"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"1402 Brickell Ave #14B"</td><td>"For sale"</td><td>"$740,000"</td><td>"12"</td><td><span class="asp-status asp-status--warn">"2 offers"</span></td></tr>
                                            <tr><td>"81 NW 2nd Ave"</td><td>"For rent"</td><td>"$3,200/mo"</td><td>"4"</td><td><span class="asp-status asp-status--green">"Showing today"</span></td></tr>
                                            <tr><td>"3388 SW 27th Ct"</td><td>"For sale"</td><td>"$520,000"</td><td>"22"</td><td><span class="asp-status asp-status--blue">"Under contract"</span></td></tr>
                                            <tr><td>"920 Edgewater Dr"</td><td>"For sale"</td><td>"$1.1M"</td><td>"8"</td><td><span class="asp-status asp-status--green">"Active"</span></td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 2: Client Profiles
                                <div class="asp-panel" data-tab="2">
                                    <div class="asp-page-title">"Client — James & Keiko Okafor"</div>
                                    <div class="asp-page-sub">"Buyer profile · Budget $600–750K · Agent: Sofia Martins"</div>
                                    <div style="display:flex;gap:.85rem;margin-bottom:.85rem;align-items:flex-start;">
                                        <div class="asp-avatar asp-avatar-lg" style="background:rgba(99,79,235,.2);color:#a78bfa;">"JO"</div>
                                        <div>
                                            <div style="display:flex;gap:.35rem;flex-wrap:wrap;">
                                                <span class="asp-status asp-status--green">"✓ Pre-qualified $725K"</span>
                                                <span class="asp-status asp-status--blue">"12 showings"</span>
                                                <span class="asp-status asp-status--gray">"Score: 82"</span>
                                            </div>
                                        </div>
                                    </div>
                                    <div class="asp-section-hdr">"Timeline"</div>
                                    <table class="asp-table">
                                        <tbody>
                                            <tr><td>"Jun 30"</td><td>"Showing — 1402 Brickell Ave"</td><td><span class="asp-status asp-status--warn">"Offer pending"</span></td></tr>
                                            <tr><td>"Jun 28"</td><td>"Showing — 3388 SW 27th Ct"</td><td><span class="asp-status asp-status--gray">"No offer"</span></td></tr>
                                            <tr><td>"Jun 24"</td><td>"Pre-qual confirmed — Beacon Mortgage"</td><td><span class="asp-status asp-status--green">"Approved"</span></td></tr>
                                            <tr><td>"Jun 18"</td><td>"Intake call with Sofia Martins"</td><td><span class="asp-status asp-status--green">"Done"</span></td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 3: Commissions
                                <div class="asp-panel" data-tab="3">
                                    <div class="asp-page-title">"Commissions"</div>
                                    <div class="asp-page-sub">"YTD brokerage earnings"</div>
                                    <div class="asp-stat-grid" style="grid-template-columns:repeat(4,1fr);">
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Closed · YTD"</div><div class="asp-stat-value" style="color:#22c55e;">"$148,200"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"In escrow"</div><div class="asp-stat-value" style="color:#f59e0b;">"$38,000"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Pending"</div><div class="asp-stat-value">"$52,400"</div></div>
                                        <div class="asp-stat-card"><div class="asp-stat-label">"Deals closed · YTD"</div><div class="asp-stat-value">"9"</div></div>
                                    </div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Property"</th><th>"Closed"</th><th>"Commission"</th><th>"Status"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"920 Edgewater Dr"</td><td>"In escrow"</td><td class="asp-credit">"$33,000"</td><td><span class="asp-status asp-status--warn">"Awaiting close"</span></td></tr>
                                            <tr><td>"3388 SW 27th Ct"</td><td>"Jun 15"</td><td class="asp-credit">"$15,600"</td><td><span class="asp-status asp-status--green">"Paid"</span></td></tr>
                                            <tr><td>"1100 Biscayne Blvd"</td><td>"May 28"</td><td class="asp-credit">"$24,000"</td><td><span class="asp-status asp-status--green">"Paid"</span></td></tr>
                                        </tbody>
                                    </table>
                                </div>

                                // TAB 4: Agent Accounts
                                <div class="asp-panel" data-tab="4">
                                    <div class="asp-page-title">"Agent Accounts"</div>
                                    <div class="asp-page-sub">"Isolated dashboards — each agent sees only their clients"</div>
                                    <table class="asp-table">
                                        <thead><tr><th>"Agent"</th><th>"Active clients"</th><th>"YTD earnings"</th><th>"Split"</th><th>"Status"</th></tr></thead>
                                        <tbody>
                                            <tr><td>"Sofia Martins"</td><td>"7"</td><td class="asp-credit">"$61,200"</td><td>"70 / 30"</td><td><span class="asp-status asp-status--green">"Active"</span></td></tr>
                                            <tr><td>"Carlos Reyes"</td><td>"4"</td><td class="asp-credit">"$42,800"</td><td>"65 / 35"</td><td><span class="asp-status asp-status--green">"Active"</span></td></tr>
                                            <tr><td>"Yuki Tanaka"</td><td>"3"</td><td class="asp-credit">"$29,400"</td><td>"60 / 40"</td><td><span class="asp-status asp-status--gray">"On leave"</span></td></tr>
                                        </tbody>
                                    </table>
                                    <div class="asp-callout">"<strong>🔒 Data isolation</strong> — Each agent logs into their own dashboard. They see their clients, their pipeline, their commissions — not anyone else's."</div>
                                </div>

                                // TAB 5: Client Portal
                                <div class="asp-panel" data-tab="5">
                                    <div class="asp-page-title">"Client Portal"</div>
                                    <div class="asp-page-sub">"White-labeled under your brand — clients.miamirealty.co"</div>
                                    <div class="asp-section-hdr">"James & Keiko Okafor — Closing tracker"</div>
                                    <table class="asp-table">
                                        <tbody>
                                            <tr><td>"📋"</td><td>"Purchase agreement signed"</td><td><span class="asp-status asp-status--green">"Complete"</span></td><td class="asp-muted">"Jun 22"</td></tr>
                                            <tr><td>"🏦"</td><td>"Mortgage underwriting"</td><td><span class="asp-status asp-status--green">"Approved"</span></td><td class="asp-muted">"Jun 30"</td></tr>
                                            <tr><td>"🔍"</td><td>"Home inspection"</td><td><span class="asp-status asp-status--green">"Passed"</span></td><td class="asp-muted">"Jul 2"</td></tr>
                                            <tr><td>"🏡"</td><td>"Title search"</td><td><span class="asp-status asp-status--warn">"In progress"</span></td><td class="asp-muted">"Est. Jul 8"</td></tr>
                                            <tr><td>"📦"</td><td>"Closing & key handover"</td><td><span class="asp-status asp-status--gray">"Scheduled"</span></td><td class="asp-muted">"Jul 15"</td></tr>
                                        </tbody>
                                    </table>
                                    <div class="asp-callout">"<strong>🌐 Your brand, not ours</strong> — Clients see your brokerage name and logo. Folio is invisible infrastructure."</div>
                                </div>

                            </main>
                        </div>
                    </div>
                </div>
            </div>
        </section>
    }
}
