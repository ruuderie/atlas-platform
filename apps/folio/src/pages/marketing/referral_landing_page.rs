//! ReferralLandingPage — Friends & Family invite landing.
//!
//! Dual acquisition (keep both):
//! - **Marketing landings** (`/`, `/vendors`, `/property-managers`, …) → **waitlist**
//!   for cold traffic (Google, ads). You vet strangers before access.
//! - **Warm `/refer/:code`** → **into the app** for trusted inner-circle testers
//!   (feedback). Attribution via `?ref=`.
//! - **Cold `/refer`** (no code) → send people to marketing waitlist, not open signup.

use crate::components::marketing_nav::{MarketingNav, MarketingNavSectionLink};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::{use_params_map, use_query_map};

fn slugify_refer_code(raw: &str) -> String {
    let slug: String = raw
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    slug.split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .chars()
        .take(48)
        .collect()
}

fn resolve_refer_code(
    params: &leptos_router::params::ParamsMap,
    query: &leptos_router::params::ParamsMap,
) -> String {
    params
        .get("code")
        .filter(|c| !c.trim().is_empty())
        .or_else(|| query.get("ref").filter(|c| !c.trim().is_empty()))
        .map(|c| slugify_refer_code(&c))
        .filter(|c| !c.is_empty())
        .unwrap_or_default()
}

fn path_href(role_path: &str, code: &str) -> String {
    format!("{role_path}?ref={code}")
}

const WARM_LINKS: &[MarketingNavSectionLink] = &[MarketingNavSectionLink {
    label: "Get started",
    href: "#start",
}];

const COLD_LINKS: &[MarketingNavSectionLink] = &[MarketingNavSectionLink {
    label: "Waitlist",
    href: "/#waitlist-wrap",
}];

#[component]
pub fn ReferralLandingPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let refer_code = Memo::new(move |_| resolve_refer_code(&params.get(), &query.get()));
    let is_warm = Memo::new(move |_| !refer_code.get().is_empty());

    view! {
        <Title text="Folio — Friends & Family"/>
        <Meta name="description" content="Invited by a friend? Create your Folio account and get set up. Looking for Folio on your own? Join the waitlist on our marketing pages."/>
        <Meta property="og:title" content="Folio — Friends & Family"/>
        <Meta property="og:description" content="Trusted invites go into the app. Public interest goes on the waitlist."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/refer"/>

        <div class="folio-mktg">
            {move || {
                if is_warm.get() {
                    view! {
                        <MarketingNav
                            section_links=WARM_LINKS
                            cta_label="Get started"
                            cta_href="#start"
                        />
                        <WarmReferHero refer_code=refer_code/>
                        <ReferPaths refer_code=refer_code/>
                        <ReferFooter/>
                    }.into_any()
                } else {
                    view! {
                        <MarketingNav
                            section_links=COLD_LINKS
                            cta_label="Join waitlist"
                            cta_href="/#waitlist-wrap"
                        />
                        <ColdReferHero/>
                        <ReferFooter/>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// No invite code — don’t open the app; send to marketing waitlist for vetting.
#[component]
fn ColdReferHero() -> impl IntoView {
    view! {
        <section class="mktg-hero" style="min-height:58vh;padding-bottom:3rem;">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:680px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"favorite"</span>
                    " Friends & Family"
                </div>
                <h1 class="mktg-hero-h1">
                    "Got an invite link?"
                    <span class="mktg-h1-accent">" Open it to get in."</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:520px;margin:1.25rem auto 0;">
                    "Folio early access for people you trust comes from a personal link. \
                     If you found us on your own — Google, ads, word of mouth without a link — \
                     join the waitlist and we’ll review your spot."
                </p>
                <div style="display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2rem;">
                    <a href="/#waitlist-wrap" class="mktg-btn-accent mktg-btn-lg">"Join the waitlist →"</a>
                    <a href="/login" class="mktg-btn-ghost">"Sign in"</a>
                </div>
                <p style="margin-top:1.5rem;font-size:.8rem;color:#8a97b0;max-width:420px;margin-left:auto;margin-right:auto;">
                    "Already have a link like "
                    <code style="color:#06d6a0;">"/refer/alice"</code>
                    "? Use that URL — it opens setup in the app."
                </p>
            </div>
        </section>
    }
}

#[component]
fn WarmReferHero(refer_code: Memo<String>) -> impl IntoView {
    view! {
        <section class="mktg-hero" style="min-height:58vh;padding-bottom:3rem;">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:680px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"favorite"</span>
                    " Friends & Family · Early access"
                </div>
                <p style="margin:0 auto 1rem;width:fit-content;font-size:.85rem;font-weight:600;color:#06d6a0;display:inline-flex;align-items:center;gap:.35rem;">
                    <span class="material-symbols-outlined" style="font-size:16px;font-variation-settings:'FILL' 1">"favorite"</span>
                    {move || format!("{} invited you", refer_code.get())}
                </p>
                <h1 class="mktg-hero-h1">
                    "You’re in."
                    <span class="mktg-h1-accent">" Create your Folio account."</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:480px;margin:1.25rem auto 0;">
                    "Trusted invites skip the waitlist — jump into the app, try it, and tell us what you think. Your friend gets credit when you join."
                </p>
                <div style="display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2rem;">
                    <a href="#start" class="mktg-btn-accent mktg-btn-lg">"Continue →"</a>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ReferPaths(refer_code: Memo<String>) -> impl IntoView {
    view! {
        <section id="start" class="mktg-section">
            <div class="mktg-section-inner" style="max-width:720px;margin:0 auto;text-align:center;">
                <p class="mktg-section-eyebrow">"Get started"</p>
                <h2 class="mktg-section-h2">"How will you use Folio?"</h2>
                <p class="mktg-section-sub" style="max-width:420px;margin:0 auto;">
                    "Choose the path that fits — we’ll create your account and open the matching setup."
                </p>
                <div style="display:grid;grid-template-columns:1fr;gap:.75rem;margin-top:1.75rem;text-align:left;">
                    <ReferPathCard icon="domain" title="Landlord" desc="Own or manage rentals" href="/onboarding" refer_code=refer_code/>
                    <ReferPathCard icon="handyman" title="Vendor" desc="Trades & service work" href="/onboard/vendor" refer_code=refer_code/>
                    <ReferPathCard icon="apartment" title="Property manager" desc="Run a client book" href="/onboard/pmc" refer_code=refer_code/>
                    <ReferPathCard icon="real_estate_agent" title="Broker" desc="Office & agent roster" href="/onboard/broker" refer_code=refer_code/>
                    <ReferPathCard icon="badge" title="Agent" desc="Listings & clients" href="/onboard/agent" refer_code=refer_code/>
                    <ReferPathCard icon="villa" title="STR host" desc="Short-term stays" href="/onboarding" refer_code=refer_code/>
                </div>
                <p style="margin-top:1.5rem;font-size:.8rem;color:#8a97b0;">
                    "Already have Folio? "
                    <a href="/login" style="color:#ff6b35;">"Sign in"</a>
                    " — we’ll still credit "
                    <strong>{move || refer_code.get()}</strong>
                    " if you came from their link."
                </p>
            </div>
        </section>
    }
}

#[component]
fn ReferPathCard(
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
    href: &'static str,
    refer_code: Memo<String>,
) -> impl IntoView {
    view! {
        <a
            class="mktg-feature-card"
            style="display:flex;align-items:center;gap:1rem;padding:1.15rem 1.2rem;text-decoration:none;color:inherit;"
            href=move || path_href(href, &refer_code.get())
        >
            <span class="material-symbols-outlined" style="color:#ffaa80;font-size:22px;">{icon}</span>
            <span style="flex:1;min-width:0;">
                <span style="display:block;font-weight:700;font-size:.95rem;">{title}</span>
                <span style="display:block;font-size:.78rem;color:#8a97b0;margin-top:.2rem;">{desc}</span>
            </span>
            <span class="material-symbols-outlined" style="opacity:.5;">"chevron_right"</span>
        </a>
    }
}

#[component]
fn ReferFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div><strong>"Folio"</strong>" · Friends & Family"</div>
                <div class="mktg-footer-links">
                    <a href="/">"Landlords"</a>
                    <a href="/vendors">"Vendors"</a>
                    <a href="/property-managers">"PMs"</a>
                    <a href="/#waitlist-wrap">"Waitlist"</a>
                    <a href="/login">"Sign in"</a>
                </div>
            </div>
        </footer>
    }
}
