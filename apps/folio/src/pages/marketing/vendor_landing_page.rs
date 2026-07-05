//! VendorLandingPage — marketing page targeting vendors, contractors & service providers.
//!
//! Served at: `/vendors`
//!
//! Zero-auth, accessible to any visitor. Independently managed under
//! `app_id = "folio-vendor"` in platform-admin.
//!
//! # Value proposition
//! Vendors (contractors, plumbers, HVAC, cleaners, landscapers) get job dispatch,
//! invoicing, a marketplace profile, and payment — all in one place.
//!
//! # Pricing model
//! Freemium: free marketplace listing + job acceptance; paid tiers unlock priority
//! placement, auto-invoicing, and 0% platform fee.

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn VendorLandingPage() -> impl IntoView {
    view! {
        <Title text="Folio for Vendors – Get Jobs, Get Paid, No Chasing"/>
        <Meta name="description" content="Folio connects vendors and contractors to property managers and landlords. Get dispatched jobs, send invoices, collect payment, and grow your service business — all on one platform."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/vendors"/>

        <VendorNav/>
        <VendorHero/>
        <VendorHow/>
        <VendorFeatures/>
        <VendorPricing/>
        <VendorCta/>
        <VendorFooter/>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn VendorNav() -> impl IntoView {
    let menu_open = RwSignal::new(false);
    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <a href="/" class="mktg-nav-logo">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </a>
                <div class="mktg-nav-links">
                    <a href="#vendor-how">"How it works"</a>
                    <a href="#vendor-features">"Features"</a>
                    <a href="#vendor-pricing">"Pricing"</a>
                    <a href="/" class="mktg-nav-broker-link">"For Landlords"</a>
                    <a href="/property-managers">"For PMs"</a>
                </div>
                <div class="mktg-nav-actions">
                    <a href="/login" class="mktg-btn-signin" id="vendor-nav-signin-btn">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </a>
                    <a href="#vendor-signup" class="mktg-btn-accent">"Join marketplace"</a>
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
            <a href="#vendor-how"          on:click=move |_| menu_open.set(false)>"How it works"</a>
            <a href="#vendor-features"     on:click=move |_| menu_open.set(false)>"Features"</a>
            <a href="#vendor-pricing"      on:click=move |_| menu_open.set(false)>"Pricing"</a>
            <a href="/"                    on:click=move |_| menu_open.set(false)>"For Landlords"</a>
            <a href="/property-managers"   on:click=move |_| menu_open.set(false)>"For PMs"</a>
            <a href="/brokers"             on:click=move |_| menu_open.set(false)>"For Brokers"</a>
            <a href="#vendor-signup"       on:click=move |_| menu_open.set(false) class="mktg-btn-accent mktg-mobile-nav-cta">"Join marketplace"</a>
        </div>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn VendorHero() -> impl IntoView {
    let email     = RwSignal::new(String::new());
    let submitted = RwSignal::new(false);

    view! {
        <section id="vendor-hero" class="mktg-hero" style="background:linear-gradient(160deg,#0a1628 0%,#0d1f3c 50%,#0a1220 100%);">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:800px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"handyman"</span>
                    " Free to join · Get paid instantly · No chasing"
                </div>
                <h1 class="mktg-hero-h1">
                    "Get jobs."
                    <span class="mktg-h1-accent"> " Get paid."</span>
                    <br/>
                    "No chasing."
                </h1>
                <p class="mktg-hero-sub" style="max-width:580px;margin:1.5rem auto 0;">
                    "Join the Folio vendor marketplace and get connected to property managers \
                     and landlords in your area. Jobs dispatched to your phone. \
                     Invoices sent in one tap. Payment in 24 hours."
                </p>

                // ── Lead capture ───────────────────────────────────────────
                <div id="vendor-signup" style="margin-top:2.5rem;" class="mktg-wl-wrap">
                    {move || if submitted.get() {
                        view! {
                            <div class="mktg-success-card">
                                <span class="material-symbols-outlined" style="font-size:2rem;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                                <div>
                                    <div class="mktg-success-h">"You're on the list!"</div>
                                    <div class="mktg-success-sub">"We'll reach out with your marketplace profile link when we launch in your area."</div>
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
                                    id="vendor-hero-email"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))
                                />
                                <button
                                    class="mktg-btn-accent"
                                    id="vendor-hero-cta"
                                    on:click=move |_| {
                                        if !email.get().is_empty() { submitted.set(true); }
                                    }
                                >
                                    "Join marketplace"
                                </button>
                            </div>
                            <p style="font-size:.75rem;color:#6b7280;margin-top:.75rem;">"Free to join. No subscription required to accept jobs."</p>
                        }.into_any()
                    }}
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
                        <div class="mktg-stat-num">"0"</div>
                        <div class="mktg-stat-label">"invoicing or chasing"</div>
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
                <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:2rem;margin-top:2.5rem;max-width:900px;margin-inline:auto;">
                    <div class="mktg-str-card" style="text-align:center;padding:2rem;">
                        <div style="width:48px;height:48px;border-radius:50%;background:rgba(6,214,160,.15);border:2px solid #06d6a0;display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
                            <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"person_add"</span>
                        </div>
                        <h3 style="font-size:1rem;font-weight:600;margin-bottom:.5rem;">"1. Create your profile"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"List your trade, service area, and availability. Free in under 5 minutes."</p>
                    </div>
                    <div class="mktg-str-card" style="text-align:center;padding:2rem;">
                        <div style="width:48px;height:48px;border-radius:50%;background:rgba(245,158,11,.15);border:2px solid #f59e0b;display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
                            <span class="material-symbols-outlined" style="color:#f59e0b;font-variation-settings:'FILL' 1">"notifications"</span>
                        </div>
                        <h3 style="font-size:1rem;font-weight:600;margin-bottom:.5rem;">"2. Get dispatched"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"Receive job requests from landlords and PMs in your area. Accept with one tap."</p>
                    </div>
                    <div class="mktg-str-card" style="text-align:center;padding:2rem;">
                        <div style="width:48px;height:48px;border-radius:50%;background:rgba(255,107,53,.15);border:2px solid #ff6b35;display:flex;align-items:center;justify-content:center;margin:0 auto 1rem;">
                            <span class="material-symbols-outlined" style="color:#ff6b35;font-variation-settings:'FILL' 1">"payments"</span>
                        </div>
                        <h3 style="font-size:1rem;font-weight:600;margin-bottom:.5rem;">"3. Invoice & get paid"</h3>
                        <p style="color:var(--mk-muted);font-size:.9rem;">"Submit your invoice in the app. Payment hits your account within 24 hours."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Features ─────────────────────────────────────────────────────────────────

#[component]
fn VendorFeatures() -> impl IntoView {
    view! {
        <section id="vendor-features" class="mktg-section" style="background:rgba(6,214,160,.03);border-top:1px solid rgba(6,214,160,.1);border-bottom:1px solid rgba(6,214,160,.1);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Features"</p>
                <h2 class="mktg-section-h2">"Built for tradespeople, not accountants."</h2>
                <div class="mktg-feature-grid">
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"search"</span>
                        <h3>"Marketplace listing"</h3>
                        <p>"Your business profile is visible to every landlord and PM on the Folio platform. Free, forever."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"assignment"</span>
                        <h3>"Job dispatch"</h3>
                        <p>"Receive job requests with photos, descriptions, and property details. Accept or decline in one tap."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"receipt"</span>
                        <h3>"Instant invoicing"</h3>
                        <p>"Create and send invoices from your phone in 60 seconds. No spreadsheets. No chasing."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"account_balance_wallet"</span>
                        <h3>"Fast payment"</h3>
                        <p>"Get paid directly via ACH. No more waiting 30+ days for a check in the mail."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"star"</span>
                        <h3>"Reviews & ratings"</h3>
                        <p>"Build your reputation on the platform. Highly-rated vendors get priority placement and more jobs."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"bar_chart"</span>
                        <h3>"Job analytics"</h3>
                        <p>"Track your earnings, completed jobs, average ticket size, and client retention over time."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Pricing ───────────────────────────────────────────────────────────────────

#[component]
fn VendorPricing() -> impl IntoView {
    view! {
        <section id="vendor-pricing" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Pricing"</p>
                <h2 class="mktg-section-h2">"Free to start. Upgrade when you're ready."</h2>
                <p class="mktg-section-sub" style="max-width:520px;margin:0 auto 2.5rem;">"No subscription required to accept jobs. Pay only when you want priority placement and extra features."</p>
                <div class="mktg-pricing-grid" style="grid-template-columns:repeat(3,1fr);">

                    // ── Basic listing — free ───────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Basic"</span>
                        <div class="mktg-pricing-price">"$0"</div>
                        <div class="mktg-pricing-sub">"Free forever"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Marketplace profile"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"Accept & complete jobs"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"In-platform invoicing"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#06d6a0;font-variation-settings:'FILL' 1">"check"</span>"ACH payment"</li>
                        </ul>
                        <a href="#vendor-signup" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="vendor-pricing-basic">"Join free"</a>
                    </div>

                    // ── Pro Vendor (FEATURED) ──────────────────────────────
                    <div class="mktg-pricing-card mktg-pricing-featured">
                        <span class="mktg-pricing-tier">"Pro Vendor"</span>
                        <div class="mktg-pricing-price">"$29"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"Priority placement"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Everything in Basic"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Priority search placement"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Auto-invoicing templates"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Job analytics dashboard"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#ff6b35;font-variation-settings:'FILL' 1">"check"</span>"Verified badge"</li>
                        </ul>
                        <a href="#vendor-signup" class="mktg-pricing-btn mktg-pricing-btn-accent" id="vendor-pricing-pro">"Get early access"</a>
                    </div>

                    // ── Business ───────────────────────────────────────────
                    <div class="mktg-pricing-card">
                        <span class="mktg-pricing-tier">"Business"</span>
                        <div class="mktg-pricing-price">"$79"<span class="mktg-pricing-per">"/mo"</span></div>
                        <div class="mktg-pricing-sub">"0% platform fee"</div>
                        <ul class="mktg-pricing-features">
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Everything in Pro Vendor"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"0% platform fee on jobs"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Multi-tech accounts"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Branded company profile"</li>
                            <li class="mktg-pf"><span class="material-symbols-outlined" style="font-size:16px;color:#f59e0b;font-variation-settings:'FILL' 1">"check"</span>"Dedicated account manager"</li>
                        </ul>
                        <a href="#vendor-signup" class="mktg-pricing-btn mktg-pricing-btn-ghost" id="vendor-pricing-business">"Get early access"</a>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Bottom CTA ────────────────────────────────────────────────────────────────

#[component]
fn VendorCta() -> impl IntoView {
    view! {
        <section class="mktg-cta-section">
            <div class="mktg-section-inner mktg-cta-inner">
                <p class="mktg-section-eyebrow" style="color:#f59e0b;">"Open to all trades"</p>
                <h2 class="mktg-cta-h2">"Stop waiting for referrals. Start getting jobs."</h2>
                <p class="mktg-cta-sub">
                    "Join the Folio vendor marketplace and get connected to property managers \
                     and landlords who need your services — starting today."
                </p>
                <a href="#vendor-signup" class="mktg-btn-accent mktg-btn-lg" id="vendor-cta-btn">"Join the marketplace →"</a>
                <p style="margin-top:16px;font-size:12px;color:#9ca3af;">"Free to join. No credit card required."</p>
            </div>
        </section>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn VendorFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS · Vendor Marketplace"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/">"For Landlords"</a>
                    <a href="/property-managers">"For Property Managers"</a>
                    <a href="/brokers">"For Brokers"</a>
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
