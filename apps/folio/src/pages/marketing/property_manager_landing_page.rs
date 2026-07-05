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

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn PropertyManagerLandingPage() -> impl IntoView {
    view! {
        <Title text="Folio for Property Managers – Run Every Portfolio, Bill Every Owner"/>
        <Meta name="description" content="Folio gives property managers owner portals, trust accounting, maintenance dispatch, and multi-portfolio billing in one platform. Start free, scale to hundreds of units."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/property-managers"/>

        <PmNav/>
        <PmHero/>
        <PmProblem/>
        <PmFeatures/>
        <PmOwnerPortal/>
        <PmPricing/>
        <PmCta/>
        <PmFooter/>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn PmNav() -> impl IntoView {
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
                    <a href="#pm-features">"Features"</a>
                    <a href="#pm-owner-portal">"Owner Portal"</a>
                    <a href="#pm-pricing">"Pricing"</a>
                    <a href="/brokers" class="mktg-nav-broker-link">"For Brokers →"</a>
                    <a href="/vendors">"For Vendors"</a>
                </div>
                <div class="mktg-nav-actions">
                    <a href="/login" class="mktg-btn-signin" id="pm-nav-signin-btn">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </a>
                    <a href="#pm-waitlist" class="mktg-btn-accent">"Get early access"</a>
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
        <div class=move || if menu_open.get() {
            "mktg-mobile-nav mktg-mobile-nav--open"
        } else {
            "mktg-mobile-nav"
        }>
            <a href="#pm-features"    on:click=move |_| menu_open.set(false)>"Features"</a>
            <a href="#pm-owner-portal" on:click=move |_| menu_open.set(false)>"Owner Portal"</a>
            <a href="#pm-pricing"     on:click=move |_| menu_open.set(false)>"Pricing"</a>
            <a href="/"              on:click=move |_| menu_open.set(false)>"For Landlords"</a>
            <a href="/brokers"       on:click=move |_| menu_open.set(false)>"For Brokers"</a>
            <a href="/vendors"       on:click=move |_| menu_open.set(false)>"For Vendors"</a>
            <a href="#pm-waitlist"   on:click=move |_| menu_open.set(false) class="mktg-btn-accent mktg-mobile-nav-cta">"Get early access"</a>
        </div>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn PmHero() -> impl IntoView {
    let email    = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);

    view! {
        <section id="pm-hero" class="mktg-hero">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:800px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"business_center"</span>
                    " Built for property managers & PMCs · Multi-portfolio edition"
                </div>
                <h1 class="mktg-hero-h1">
                    "Run every property."
                    <span class="mktg-h1-accent"> " Bill every owner."</span>
                    <br/>
                    "Scale without chaos."
                </h1>
                <p class="mktg-hero-sub" style="max-width:580px;margin:1.5rem auto 0;">
                    "Folio gives property managers one platform for multi-client portfolios, \
                     owner statements, trust accounting, and maintenance dispatch — \
                     without the $280/mo AppFolio minimum."
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
                                    on:click=move |_| {
                                        if !email.get().is_empty() { submitted.set(true); }
                                    }
                                >
                                    "Get early access"
                                </button>
                            </div>
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
                <p class="mktg-section-eyebrow" style="color:#ff6b35;">"The Problem"</p>
                <h2 class="mktg-section-h2" style="max-width:700px;margin:0 auto 1rem;">"You're managing 50 units across 8 owners with spreadsheets, texts, and 3 different apps."</h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 3rem;">
                    "Traditional property management software either costs $280/mo before you touch a unit, \
                     or lacks the owner portal and trust accounting features real PMCs need."
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
fn PmFeatures() -> impl IntoView {
    view! {
        <section id="pm-features" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Features"</p>
                <h2 class="mktg-section-h2">"Everything a PMC needs. Nothing it doesn't."</h2>
                <div class="mktg-feature-grid">
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"account_tree"</span>
                        <h3>"Multi-portfolio management"</h3>
                        <p>"Manage dozens of client portfolios from a single dashboard. Each owner sees only their properties."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"receipt_long"</span>
                        <h3>"Owner portals & statements"</h3>
                        <p>"Branded portals per owner. Monthly statements generated automatically. No more PDF emails."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"account_balance"</span>
                        <h3>"Trust accounting"</h3>
                        <p>"Security deposit ledgers, reserve funds, disbursements, and reconciliation — built-in and auditable."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"build"</span>
                        <h3>"Maintenance dispatch"</h3>
                        <p>"Tenants submit requests. You assign to vendors. Track status, photos, and invoices in one place."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"payments"</span>
                        <h3>"Rent collection & disbursement"</h3>
                        <p>"Collect via ACH or card. Automatically split management fees and disburse to owner accounts."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"description"</span>
                        <h3>"Lease & compliance"</h3>
                        <p>"Digital lease signing, renewal reminders, and jurisdiction-specific compliance checklists."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"person"</span>
                        <h3>"Tenant portal"</h3>
                        <p>"Tenants pay rent, submit requests, and view their lease — reducing inbound calls by 60%."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"analytics"</span>
                        <h3>"Portfolio analytics"</h3>
                        <p>"Occupancy rates, rent collection trends, maintenance costs, and NOI across all your clients."</p>
                    </div>
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
                    <p class="mktg-section-eyebrow">"Owner Portal"</p>
                    <h2 class="mktg-section-h2" style="font-size:clamp(1.6rem,3vw,2.2rem);">"Your clients get a professional portal. You stop answering emails."</h2>
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
                                <span style="color:var(--mk-text);font-size:.9rem;">"Maintenance"</span>
                                <span style="color:var(--mk-muted);font-weight:600;">"-$185"</span>
                            </div>
                        </div>
                    </div>
                    <div style="margin-top:1.5rem;padding-top:1rem;border-top:1px solid rgba(255,255,255,.06);display:flex;justify-content:space-between;align-items:center;">
                        <span style="color:var(--mk-muted);font-size:.85rem;">"Disbursed to owner"</span>
                        <span style="color:#f59e0b;font-size:1.1rem;font-weight:700;">"$3,667"</span>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn PmPricing() -> impl IntoView {
    view! {
        <section id="pm-pricing" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Pricing"</p>
                <h2 class="mktg-section-h2">"Pay per portfolio, not per feature."</h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 2.5rem;">"Every plan includes trust accounting, owner portals, and maintenance dispatch. No surprise add-ons."</p>
                <div class="mktg-pricing-grid">

                    // ── Starter PM ─────────────────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Starter PM"</span>
                        <div class="mktg-pricing-price">"$79"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"1 portfolio · up to 20 units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Full landlord platform"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"1 branded owner portal"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Trust accounting ledger"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Maintenance dispatch"</li>
                        </ul>
                        <a href="#pm-waitlist" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="pm-pricing-starter">"Join waitlist"</a>
                    </div>

                    // ── Growth PM (FEATURED) ───────────────────────────────
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Growth PM"</span>
                        <div class="mktg-pricing-price">"$199"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Up to 5 portfolios · 100 units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Starter PM"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"5 branded owner portals"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Auto-disbursement & fee split"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Portfolio analytics"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Vacancy marketing"</li>
                        </ul>
                        <a href="#pm-waitlist" class="mktg-pricing-btn mktg-pricing-btn-accent" id="pm-pricing-growth">"Get early access"</a>
                    </div>

                    // ── Scale PM ───────────────────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Scale PM"</span>
                        <div class="mktg-pricing-price">"$399"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Up to 15 portfolios · 300 units"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Growth PM"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Full trust accounting suite"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Multi-user team access"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Priority support"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Advanced reporting"</li>
                        </ul>
                        <a href="#pm-waitlist" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="pm-pricing-scale">"Get early access"</a>
                    </div>

                    // ── Enterprise PM ──────────────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Enterprise"</span>
                        <div class="mktg-pricing-price">"Custom"</div>
                        <div class="mktg-pricing-sub">"Unlimited portfolios · white-label · API"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Everything in Scale PM"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"White-label branding"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"API access & SSO"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Dedicated onboarding"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Uptime SLA"</li>
                        </ul>
                        <a href="#pm-waitlist" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="pm-pricing-enterprise">"Contact us"</a>
                    </div>
                </div>

                // ── Competitive callout ────────────────────────────────────
                <div class="mktg-pricing-pm-callout">
                    <span class="material-symbols-outlined" style="font-size:20px;color:#f59e0b">"trending_down"</span>
                    <div>
                        <strong>"AppFolio starts at $280/mo minimum."</strong>
                        " Buildium starts at $55/mo but charges per unit after 20. Folio's Growth PM covers 100 units for $199 — "
                        <strong>"less than $2/unit."</strong>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn PmCta() -> impl IntoView {
    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#06d6a0;">"Limited beta spots available"</p>
                <h2 class="mktg-cta-h2">"Stop managing with spreadsheets. Start running a real business."</h2>
                <p class="mktg-cta-sub">
                    "Join the waitlist for exclusive early access. Beta members lock in founder pricing \
                     and help shape the property management features before we open to the public."
                </p>
                <a href="#pm-waitlist" class="mktg-btn-accent mktg-btn-lg" id="pm-cta-btn">"Reserve my beta spot →"</a>
                <p style="margin-top:16px;font-size:12px;color:#9ca3af;">"No credit card. No contracts. Cancel anytime."</p>
            </div>
        </section>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn PmFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS · Property Manager Edition"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/">"For Landlords"</a>
                    <a href="/brokers">"For Brokers"</a>
                    <a href="/vendors">"For Vendors"</a>
                    <a href="/cohost-market">"Cohost Network"</a>
                    <a href="/login">"Sign in"</a>
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
