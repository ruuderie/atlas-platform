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

use crate::pages::not_found::NotFound;
use crate::components::lang::{LanguageSwitcher, get_current_lang};

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
            <BrokerNav/>
            <BrokerHero/>
            <BrokerFeatures/>
            <BrokerPortals/>
            <BrokerAgents/>
            <BrokerPricing/>
            <BrokerCta/>
            <BetaCalloutStrip/>
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
                    <a href="/" class="mktg-nav-broker-link">"For Landlords"</a>
                    <a href="/property-managers">"For PMs"</a>
                    <a href="/vendors">"For Vendors"</a>
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
            <a href="/"                on:click=move |_| menu_open.set(false)>"For Landlords"</a>
            <a href="/property-managers" on:click=move |_| menu_open.set(false)>"For Property Managers"</a>
            <a href="/vendors"         on:click=move |_| menu_open.set(false)>"For Vendors"</a>
            <a href="/#waitlist-wrap"  on:click=move |_| menu_open.set(false)>"Get early access"</a>
            <a href="/founding"        on:click=move |_| menu_open.set(false)>"Founding ✦"</a>
            <a href="/beta"            on:click=move |_| menu_open.set(false)>"Apply for Beta"</a>
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
                    "Run your whole brokerage"
                    <span class="mktg-h1-accent">" from one place."</span>
                </h1>
                <p class="mktg-hero-sub">
                    "Folio gives licensed brokers and their agents a single platform to manage \
                     listings, track every buyer and seller in the pipeline, close more deals, \
                     and keep commissions straight — without juggling spreadsheets and three different tools."
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
                <p class="mktg-section-eyebrow mktg-eyebrow-light">"Client & agent portals"</p>
                <h2 class="mktg-section-h2 mktg-h2-light">"Your clients and agents always know where every deal stands."</h2>
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
                <p class="mktg-section-eyebrow">"Built for teams"</p>
                <h2 class="mktg-section-h2">"Agents work in the same platform. Not on a separate spreadsheet."</h2>
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
