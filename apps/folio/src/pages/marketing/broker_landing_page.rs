//! BrokerLandingPage — marketing page targeting property managers & brokerages.
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

use crate::pages::not_found::NotFound;

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
    let page = Resource::new(|| (), |_| load_broker_page());

    view! {
        <Suspense fallback=|| view! { <BrokerSkeleton/> }>
            {move || page.get().map(|result| match result {
                Err(_) => view! { <BrokerDefault/> }.into_any(),
                Ok(data) => match data.launch_mode {
                    crate::pages::marketing::market_landing_page::LaunchMode::Draft
                        => view! { <NotFound/> }.into_any(),
                    _ => view! { <BrokerDefault/> }.into_any(),
                },
            })}
        </Suspense>
    }
}

// ── Default hardcoded content (used until DB record is published) ─────────────

#[component]
fn BrokerDefault() -> impl IntoView {
    let title       = "Folio for Brokers & Property Managers — Run Your Whole Brokerage";
    let description = "Multi-client portfolio management, owner portals, commission tracking, and agent accounts — built for property managers and licensed brokers.";

    view! {
        <Title text=title/>
        <Meta name="description"        content=description/>
        <Meta property="og:title"       content=title/>
        <Meta property="og:description" content=description/>
        <Meta property="og:type"        content="website"/>
        <Meta name="twitter:card"       content="summary_large_image"/>
        <Link rel="canonical" href="/brokers"/>

        <div class="folio-mktg">
            <BrokerNav/>
            <BrokerHero/>
            <BrokerFeatures/>
            <BrokerPortals/>
            <BrokerAgents/>
            <BrokerPricing/>
            <BrokerCta/>
            <BrokerFooter/>
        </div>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn BrokerNav() -> impl IntoView {
    let menu_open = RwSignal::new(false);
    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <a href="/" class="mktg-nav-logo">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </a>
                // ── Desktop links ──────────────────────────────────────────
                <div class="mktg-nav-links">
                    <a href="#broker-features">"Features"</a>
                    <a href="#broker-portals">"Portals"</a>
                    <a href="#broker-agents">"Agent Accounts"</a>
                    <a href="#broker-pricing">"Pricing"</a>
                    <a href="/" class="mktg-nav-broker-link">"← Landlord page"</a>
                </div>
                <div class="mktg-nav-actions">
                    <a href="/login" class="mktg-btn-signin" id="broker-nav-signin-btn">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </a>
                    <a href="/#waitlist-wrap" class="mktg-btn-accent">"Get early access"</a>
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
            <a href="#broker-features" on:click=move |_| menu_open.set(false)>"Features"</a>
            <a href="#broker-portals"  on:click=move |_| menu_open.set(false)>"Portals"</a>
            <a href="#broker-agents"   on:click=move |_| menu_open.set(false)>"Agent Accounts"</a>
            <a href="#broker-pricing"  on:click=move |_| menu_open.set(false)>"Pricing"</a>
            <a href="/"                on:click=move |_| menu_open.set(false) class="mktg-mobile-nav-broker">"← Landlord page"</a>
            <a href="/#waitlist-wrap"  on:click=move |_| menu_open.set(false) class="mktg-btn-accent mktg-mobile-nav-cta">"Get early access"</a>
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
                    " Beta Access Open · Built for property managers & brokerages"
                </div>
                <h1 class="mktg-hero-h1">
                    "Run your whole brokerage"
                    <span class="mktg-h1-accent">" from one place."</span>
                </h1>
                <p class="mktg-hero-sub">
                    "Folio gives property managers and licensed brokers a single platform to manage \
                     multiple client portfolios, track commissions, give owners a branded portal, \
                     and keep agents organised — without duct-taping three different tools together."
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
                    <a href="/login" class="mktg-btn-signin" style="padding:14px 24px;font-size:15px" id="broker-hero-signin">
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
        ("groups",       "Multi-client portfolio",      "Manage properties for multiple owners under one login. Every client's portfolio is separate, private, and clearly labelled."),
        ("receipt_long", "Owner statements",            "Generate monthly owner statements with income, expenses, and net distributions. Export as PDF or send directly from the platform."),
        ("payments",     "Commission tracking",         "Define commission splits per property or per owner. Folio tracks every payment and tells you exactly what you're owed."),
        ("person",       "Agent accounts",              "Add agents to your brokerage. They see only their assigned properties. You see everything. One platform, clear boundaries."),
        ("gavel",        "Compliance & licensing",      "Track your brokerage license renewals, agent certifications, and fair housing training deadlines — all in one place."),
        ("analytics",    "Brokerage analytics",         "Revenue by client, vacancy rate across your book, and maintenance cost trends — dashboards built for your business, not your clients'."),
        ("build",        "Maintenance dispatch",        "Tenants submit, you approve or assign to an agent, vendors receive the job. Work orders are tracked end to end."),
        ("home_work",    "Vacation rental support",     "Manage vacation rentals alongside long-term leases. Unified calendar, direct booking, and compliance — all inside Folio."),
        ("language",     "US · Canada · Brazil",        "Operate across borders. Folio handles local payment rails, compliance rules, and currency — so you don't have to."),
    ];

    view! {
        <section id="broker-features" class="mktg-section mktg-features">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"What's included"</p>
                <h2 class="mktg-section-h2">"Everything your brokerage needs. Nothing it doesn't."</h2>
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
                <p class="mktg-section-eyebrow mktg-eyebrow-light">"Owner & tenant portals"</p>
                <h2 class="mktg-section-h2 mktg-h2-light">"Your clients see exactly what you want them to see."</h2>
                <p class="mktg-section-sub mktg-sub-light">
                    "Every owner gets a branded read-only portal showing their properties, income, \
                     maintenance history, and statements. Every tenant gets a payment and maintenance \
                     portal. You control what's visible — they never see another client's data."
                </p>
                <div class="mktg-str-grid">
                    {[
                        ("home_work", "Owner portal",
                         "Each property owner logs in to a branded dashboard showing their portfolio income, active leases, open maintenance items, and monthly statements. No shared data. No confusion."),
                        ("person",    "Tenant portal",
                         "Tenants pay rent, submit maintenance requests, sign leases, and track move-in documents — without calling you. Works for all your clients' tenants, one system."),
                        ("receipt",   "Statement delivery",
                         "Generate and send monthly owner statements with one click. PDF export, direct email from the platform, or downloadable by the owner from their portal."),
                        ("lock",      "Data separation",
                         "Each client's portfolio is completely private. A property owner can only see their own properties. Agents can only see their assigned accounts."),
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
                <p class="mktg-section-eyebrow">"Built for teams"</p>
                <h2 class="mktg-section-h2">"Agents work in the same platform. Not on a separate spreadsheet."</h2>
                <p class="mktg-section-sub">
                    "Folio's brokerage mode lets you create agent accounts under your brokerage login. \
                     Each agent sees only their assigned properties. You see the whole book. \
                     Work orders, maintenance, and leases flow through one system."
                </p>
                <div class="mktg-str-grid">
                    {[
                        ("person_add", "Agent accounts",
                         "Invite agents to your brokerage. They get their own login scoped to their assigned properties."),
                        ("assignment", "Property assignment",
                         "Assign any property to any agent. Reassign instantly when your team changes."),
                        ("supervisor_account", "Broker oversight",
                         "As the broker you have full visibility across all agents, all clients, and all properties at all times."),
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
                <h2 class="mktg-section-h2">"Priced for your business, not per unit."</h2>
                <div class="mktg-pricing-grid">

                    // Portfolio tier
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Portfolio"</span>
                        <div class="mktg-pricing-price">"$99"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Full platform access · solo operator"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Single-operator portfolio"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Rent collection & leases"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Tenant portal"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Vacation rental calendar"</li>
                        </ul>
                        <a href="/#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="broker-pricing-portfolio">"Join waitlist"</a>
                    </div>

                    // Enterprise / PMC — featured
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Enterprise / PMC"</span>
                        <div class="mktg-pricing-price">"Custom"</div>
                        <div class="mktg-pricing-sub">"Property managers & brokerages"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Portfolio"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Multi-client portfolio"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Branded owner portals"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Agent accounts"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Commission tracking"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Brokerage analytics"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Dedicated onboarding"</li>
                        </ul>
                        <a href="/#waitlist-wrap" class="mktg-pricing-btn mktg-pricing-btn-accent" id="broker-pricing-enterprise">"Get early access"</a>
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
                    <a href="/">"← Main page"</a>
                    <a href="/login">"Sign in"</a>
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
