//! BetaProgramPage — curated beta program application page.
//!
//! Served at: `/beta`
//!
//! Strategy:
//!   - Real prices are shown on the marketing pages so the market understands value.
//!   - This page pitches free access during beta in exchange for real usage + feedback.
//!   - An application form screens applicants — only serious, active operators get in.
//!   - Accepted applicants get an invite link (manually reviewed; platform-admin invite flow).
//!
//! Screening criteria (internal — not shown to applicant):
//!   - Active portfolio (has real units/clients right now)
//!   - Currently using a competing tool (switching intent)
//!   - Willing to commit to monthly feedback
//!   - Role + portfolio size matches what we want to test

use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::components::A;

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn BetaProgramPage() -> impl IntoView {
    view! {
        <Title text="Folio Beta Program — Apply for Discounted Early Access"/>
        <Meta name="description" content="Apply to join the Folio beta program. Get discounted access during beta in exchange for real usage and feedback. Limited spots. We review every application."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/beta"/>

        <BetaNav/>
        <BetaHero/>
        <BetaWhatYouGet/>
        <BetaWhatWeLookFor/>
        <BetaApplication/>
        <BetaFooter/>
    }
}

// ── Nav ───────────────────────────────────────────────────────────────────────

#[component]
fn BetaNav() -> impl IntoView {
    let menu_open = RwSignal::new(false);
    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <A href="/" attr:class="mktg-nav-logo">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </A>
                <div class="mktg-nav-links">
                    <a href="#beta-what-you-get">"What you get"</a>
                    <a href="#beta-what-we-look-for">"Who we accept"</a>
                    <a href="#beta-apply">"Apply"</a>
                    <A href="/founding">"Lifetime plans"</A>
                    <a href="/#pricing">"Pricing"</a>
                </div>
                <div class="mktg-nav-actions">
                    <A href="/login" attr:class="mktg-btn-signin" attr:id="beta-nav-signin">
                        <span class="material-symbols-outlined" style="font-size:15px;vertical-align:middle">"login"</span>
                        " Sign in"
                    </A>
                    <a href="#beta-apply" class="mktg-btn-accent" id="beta-nav-cta">"Apply now"</a>
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
            <a href="#beta-what-you-get"    on:click=move |_| menu_open.set(false)>"What you get"</a>
            <a href="#beta-what-we-look-for" on:click=move |_| menu_open.set(false)>"Who we accept"</a>
            <a href="#beta-apply"           on:click=move |_| menu_open.set(false)>"Apply"</a>
            <A href="/founding"             on:click=move |_| menu_open.set(false)>"Lifetime plans"</A>
            <a href="/#pricing"             on:click=move |_| menu_open.set(false)>"Pricing"</a>
            <a href="#beta-apply" on:click=move |_| menu_open.set(false) class="mktg-btn-accent mktg-mobile-nav-cta">"Apply now"</a>
        </div>
    }
}

// ── Hero ──────────────────────────────────────────────────────────────────────

#[component]
fn BetaHero() -> impl IntoView {
    view! {
        <section class="mktg-hero" style="background:linear-gradient(160deg,#0a1628 0%,#0c1a30 50%,#070d18 100%);">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:760px;">
                <div class="mktg-eyebrow" style="color:#06d6a0;">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"science"</span>
                    " Beta Program · Application Required · Limited Spots"
                </div>
                <h1 class="mktg-hero-h1">
                    "Discounted access."
                    <span class="mktg-h1-accent"> " Real feedback."</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:580px;margin:1.5rem auto 0;">
                    "We're opening a curated beta program for active landlords, brokers, \
                     property managers, and vendors. If accepted, you get full access to Folio \
                     at a discounted rate during the beta period — in exchange for real usage and honest feedback."
                </p>

                <div style="display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2rem;">
                    <a href="#beta-apply" class="mktg-btn-accent mktg-btn-lg" id="beta-hero-cta">"Apply for beta →"</a>
                    <a href="/founding" class="mktg-btn-ghost-sm">"See founding member pricing"</a>
                </div>

                <div class="mktg-stats" style="margin-top:3rem;border-top:1px solid var(--mk-border);padding-top:2rem;">
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"Discounted"</div>
                        <div class="mktg-stat-label">"rate during beta"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"Curated"</div>
                        <div class="mktg-stat-label">"application required"</div>
                    </div>
                    <div class="mktg-stat">
                        <div class="mktg-stat-num">"48h"</div>
                        <div class="mktg-stat-label">"decision turnaround"</div>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── What you get ──────────────────────────────────────────────────────────────

#[component]
fn BetaWhatYouGet() -> impl IntoView {
    view! {
        <section id="beta-what-you-get" class="mktg-section">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Beta member perks"</p>
                <h2 class="mktg-section-h2">"What accepted beta members get."</h2>
                <div class="mktg-feature-grid" style="margin-top:2rem;">
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"percent"</span>
                        <h3>"Discounted access during beta"</h3>
                        <p>"Beta members get full platform access at a reduced rate for the entire beta period. Accepted members get priority pricing when we move to general availability."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"key"</span>
                        <h3>"Full platform access, day one"</h3>
                        <p>"Complete access to your role's portal — landlord, broker, PM, or vendor — the moment you're accepted. No waiting list, no drip rollout."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"rocket_launch"</span>
                        <h3>"First access to every new feature"</h3>
                        <p>"Founding members see new portals and capabilities before the public. You're not waiting for the changelog — you're influencing it."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"groups"</span>
                        <h3>"Direct line to the product team"</h3>
                        <p>"Your feedback shapes what we build next. Monthly calls with the founding cohort. Your use cases drive the roadmap — not a ticket queue."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"badge"</span>
                        <h3>"Founding Member badge"</h3>
                        <p>"Your profile carries a permanent Founding Member badge across the platform. First-wave status, recognized forever."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"support_agent"</span>
                        <h3>"Priority support"</h3>
                        <p>"Founding members get direct Slack or email access to the engineering team. You're not filing tickets into a void."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

// ── Who we look for ───────────────────────────────────────────────────────────

#[component]
fn BetaWhatWeLookFor() -> impl IntoView {
    view! {
        <section id="beta-what-we-look-for" class="mktg-section" style="background:rgba(6,214,160,.03);border-top:1px solid rgba(6,214,160,.08);border-bottom:1px solid rgba(6,214,160,.08);">
            <div class="mktg-section-inner">
                <p class="mktg-section-eyebrow">"Who we accept"</p>
                <h2 class="mktg-section-h2">"We're looking for real operators, not curious onlookers."</h2>
                <p class="mktg-section-sub" style="max-width:580px;margin:0 auto 2.5rem;">
                    "Beta feedback is only valuable when it comes from people actively running a real portfolio or business. \
                     Here's what we prioritize."
                </p>
                <div style="display:grid;grid-template-columns:1fr 1fr;gap:1.5rem;max-width:800px;margin:0 auto;">
                    <div class="beta-criteria-card beta-criteria-yes">
                        <div class="beta-criteria-label">
                            <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            "Strong applications"
                        </div>
                        <ul class="beta-criteria-list">
                            <li>"Active portfolio: real units, clients, or jobs right now"</li>
                            <li>"Currently using another tool you want to replace"</li>
                            <li>"Willing to commit to a 30-min call once per month"</li>
                            <li>"Will actually use the platform, not just sign up"</li>
                            <li>"Landlords: 2+ units · Brokers: 1+ active listing · PMs: 1+ owner client"</li>
                        </ul>
                    </div>
                    <div class="beta-criteria-card beta-criteria-no">
                        <div class="beta-criteria-label">
                            <span class="material-symbols-outlined" style="color:#ef4444;font-variation-settings:'FILL' 1">"cancel"</span>
                            "Not a good fit"
                        </div>
                        <ul class="beta-criteria-list">
                            <li>"No active portfolio (planning to buy someday)"</li>
                            <li>"Just curious — no current business need"</li>
                            <li>"Won't have time to give feedback"</li>
                            <li>"Looking for software with no intention to use it seriously"</li>
                        </ul>
                    </div>
                </div>
                <p style="text-align:center;color:var(--mk-muted);font-size:.85rem;margin-top:2rem;">
                    "We review every application personally. No automated filters. We'll respond within 48 hours."
                </p>
            </div>
        </section>
    }
}

// ── Application form ──────────────────────────────────────────────────────────

#[component]
fn BetaApplication() -> impl IntoView {
    // Form state
    let first_name    = RwSignal::new(String::new());
    let last_name     = RwSignal::new(String::new());
    let email         = RwSignal::new(String::new());
    let role          = RwSignal::new(String::new());
    let portfolio_size = RwSignal::new(String::new());
    let current_tool  = RwSignal::new(String::new());
    let pain_point    = RwSignal::new(String::new());
    let is_active     = RwSignal::new(String::new());
    let feedback_call = RwSignal::new(String::new());
    let why_beta      = RwSignal::new(String::new());
    let submitted     = RwSignal::new(false);
    let error_msg     = RwSignal::new(String::new());

    let on_submit = move |_| {
        // Client-side validation
        if email.get().is_empty() || role.get().is_empty() || why_beta.get().len() < 20 {
            error_msg.set("Please fill in all required fields.".to_string());
            return;
        }
        if is_active.get() != "yes" {
            error_msg.set("The beta program is for operators with an active portfolio or client base. Apply again when you're ready to go.".to_string());
            return;
        }
        error_msg.set(String::new());
        submitted.set(true);
        // TODO: POST to /api/pub/beta-applications with all fields
        // Uses atlas_lead source="beta_application" + raw_data JSONB for extended fields
    };

    view! {
        <section id="beta-apply" class="mktg-section">
            <div class="mktg-section-inner" style="max-width:680px;">
                <p class="mktg-section-eyebrow">"The application"</p>
                <h2 class="mktg-section-h2">"Apply for the Folio beta."</h2>
                <p class="mktg-section-sub" style="margin:0 auto 2.5rem;">
                    "Every field helps us understand your business and match you to the right portal. \
                     Takes about 3 minutes."
                </p>

                {move || if submitted.get() {
                    view! {
                        <div class="beta-success-wrap">
                            <span class="material-symbols-outlined" style="font-size:3rem;color:#06d6a0;font-variation-settings:'FILL' 1">"task_alt"</span>
                            <h3>"Application received."</h3>
                            <p>"We review every application personally and respond within 48 hours. \
                               Check your inbox — if accepted, you'll get an invite link with \
                               instructions to set up your account."</p>
                            <p style="color:var(--mk-muted);font-size:.85rem;">"In the meantime, you can also explore our "<A href="/founding" attr:style="color:#06d6a0;">"lifetime founding member plans"</A>" if you'd rather lock in a price now."</p>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <form class="beta-form" on:submit=|e| e.prevent_default()>

                            // ── Name row ────────────────────────────────────
                            <div class="beta-form-row">
                                <div class="beta-field">
                                    <label class="beta-label" for="beta-first-name">"First name"<span class="beta-required">"*"</span></label>
                                    <input
                                        id="beta-first-name" type="text" class="beta-input"
                                        placeholder="Jane"
                                        prop:value=move || first_name.get()
                                        on:input=move |e| first_name.set(event_target_value(&e))
                                    />
                                </div>
                                <div class="beta-field">
                                    <label class="beta-label" for="beta-last-name">"Last name"<span class="beta-required">"*"</span></label>
                                    <input
                                        id="beta-last-name" type="text" class="beta-input"
                                        placeholder="Smith"
                                        prop:value=move || last_name.get()
                                        on:input=move |e| last_name.set(event_target_value(&e))
                                    />
                                </div>
                            </div>

                            // ── Email ────────────────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label" for="beta-email">"Email address"<span class="beta-required">"*"</span></label>
                                <input
                                    id="beta-email" type="email" class="beta-input"
                                    placeholder="you@yourbusiness.com"
                                    prop:value=move || email.get()
                                    on:input=move |e| email.set(event_target_value(&e))
                                />
                            </div>

                            // ── Role ─────────────────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label" for="beta-role">"I am a…"<span class="beta-required">"*"</span></label>
                                <select
                                    id="beta-role" class="beta-input beta-select"
                                    on:change=move |e| role.set(event_target_value(&e))
                                >
                                    <option value="">"Select your role"</option>
                                    <option value="landlord">"Landlord / Real estate investor"</option>
                                    <option value="str-host">"Short-term rental (STR) host"</option>
                                    <option value="broker">"Licensed broker / Real estate agent"</option>
                                    <option value="property-manager">"Property manager / Management company"</option>
                                    <option value="vendor">"Vendor / Contractor / Service provider"</option>
                                </select>
                            </div>

                            // ── Portfolio size ───────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label" for="beta-portfolio">
                                    "How many units / clients / active jobs do you currently manage?"
                                    <span class="beta-required">"*"</span>
                                </label>
                                <select
                                    id="beta-portfolio" class="beta-input beta-select"
                                    on:change=move |e| portfolio_size.set(event_target_value(&e))
                                >
                                    <option value="">"Select a range"</option>
                                    <option value="1-2">"1–2 units / 1 client"</option>
                                    <option value="3-10">"3–10 units / clients"</option>
                                    <option value="11-30">"11–30 units / clients"</option>
                                    <option value="31-100">"31–100 units / clients"</option>
                                    <option value="100+">"100+ units / clients"</option>
                                </select>
                            </div>

                            // ── Current tool ─────────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label" for="beta-current-tool">
                                    "What tool or software do you currently use to manage your portfolio?"
                                </label>
                                <input
                                    id="beta-current-tool" type="text" class="beta-input"
                                    placeholder="e.g. Buildium, TurboTenant, spreadsheets, nothing yet…"
                                    prop:value=move || current_tool.get()
                                    on:input=move |e| current_tool.set(event_target_value(&e))
                                />
                            </div>

                            // ── Pain point ───────────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label" for="beta-pain">
                                    "What's the biggest frustration with your current setup?"
                                </label>
                                <textarea
                                    id="beta-pain" class="beta-input beta-textarea"
                                    placeholder="e.g. Chasing rent payments every month, no unified view of all my properties, lease tracking is a mess…"
                                    prop:value=move || pain_point.get()
                                    on:input=move |e| pain_point.set(event_target_value(&e))
                                ></textarea>
                            </div>

                            // ── Active right now? ────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label">"Do you have an active portfolio or client base right now?"<span class="beta-required">"*"</span></label>
                                <div class="beta-radio-group">
                                    <label class="beta-radio-label">
                                        <input type="radio" name="beta-active" value="yes"
                                            on:change=move |e| is_active.set(event_target_value(&e))/>
                                        " Yes — I'm actively managing units / clients / jobs"
                                    </label>
                                    <label class="beta-radio-label">
                                        <input type="radio" name="beta-active" value="no"
                                            on:change=move |e| is_active.set(event_target_value(&e))/>
                                        " Not yet — I'm still in the planning phase"
                                    </label>
                                </div>
                            </div>

                            // ── Feedback call commitment ──────────────────────
                            <div class="beta-field">
                                <label class="beta-label">"Can you commit to a 30-minute feedback call once per month?"<span class="beta-required">"*"</span></label>
                                <div class="beta-radio-group">
                                    <label class="beta-radio-label">
                                        <input type="radio" name="beta-call" value="yes"
                                            on:change=move |e| feedback_call.set(event_target_value(&e))/>
                                        " Yes — I want to help shape the product"
                                    </label>
                                    <label class="beta-radio-label">
                                        <input type="radio" name="beta-call" value="no"
                                            on:change=move |e| feedback_call.set(event_target_value(&e))/>
                                        " No — I prefer to give feedback asynchronously"
                                    </label>
                                </div>
                            </div>

                            // ── Why beta ─────────────────────────────────────
                            <div class="beta-field">
                                <label class="beta-label" for="beta-why">
                                    "Tell us why you want to be in the beta. What will you actually use Folio for?"
                                    <span class="beta-required">"*"</span>
                                </label>
                                <textarea
                                    id="beta-why" class="beta-input beta-textarea"
                                    placeholder="Be specific. The more detail you give about your real workflow, the better we can evaluate your application — and the better we can onboard you if accepted."
                                    prop:value=move || why_beta.get()
                                    on:input=move |e| why_beta.set(event_target_value(&e))
                                ></textarea>
                                <span class="beta-field-hint">"Minimum 20 characters. Be specific — generic answers are not reviewed."</span>
                            </div>

                            // ── Error message ────────────────────────────────
                            {move || {
                                let msg = error_msg.get();
                                if msg.is_empty() {
                                    view! { <span></span> }.into_any()
                                } else {
                                    view! {
                                        <div class="beta-error-msg">
                                            <span class="material-symbols-outlined" style="font-size:16px;font-variation-settings:'FILL' 1">"error"</span>
                                            {msg}
                                        </div>
                                    }.into_any()
                                }
                            }}

                            // ── Submit ───────────────────────────────────────
                            <button
                                type="button"
                                class="beta-submit-btn"
                                id="beta-submit"
                                on:click=on_submit
                            >
                                "Submit application"
                                <span class="material-symbols-outlined" style="font-size:18px">"send"</span>
                            </button>

                            <p class="beta-form-disclaimer">
                                "We review every application personally. You'll hear back within 48 hours. \
                                 We don't spam — your email is only used for beta onboarding."
                            </p>
                        </form>
                    }.into_any()
                }}
            </div>
        </section>
    }
}

// ── Footer ────────────────────────────────────────────────────────────────────

#[component]
fn BetaFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Modern Landlord OS · Beta Program"</div>
                </div>
                <div class="mktg-footer-links">
                    <A href="/">"For Landlords"</A>
                    <A href="/brokers">"For Brokers"</A>
                    <A href="/property-managers">"For PMs"</A>
                    <A href="/vendors">"For Vendors"</A>
                    <A href="/founding">"Lifetime plans"</A>
                    <A href="/login">"Sign in"</A>
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
