//! ReferralLandingPage — Friends & Family shareable referral capture.
//!
//! Served at: `/refer` and `/refer/:code`
//!
//! Friends/family generate a personal link (`/refer/alice`), share it, and
//! prospects join the landlord waitlist. Attribution lands on
//! `utm_campaign=friends_family` + `referred_by` for the campaign leaderboard.

use crate::components::marketing_nav::{
    MarketingNav, MarketingNavSectionLink, DEFAULT_MARKETING_NAV_CTA,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::{use_params_map, use_query_map};
use shared_ui::marketing::FolioMarketingSlug;

const REFER_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Free forever",
        href: "#refer-free",
    },
    MarketingNavSectionLink {
        label: "Join",
        href: "#refer-waitlist",
    },
    MarketingNavSectionLink {
        label: "Share",
        href: "#refer-share",
    },
];

const UTM_SOURCE: &str = "referral";
const UTM_MEDIUM: &str = "friends_family";
const UTM_CAMPAIGN: &str = "friends_family";
const LEAD_SOURCE: &str = "referral:friends_family";

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

fn resolve_refer_code(params: &leptos_router::params::ParamsMap, query: &leptos_router::params::ParamsMap) -> String {
    params
        .get("code")
        .filter(|c| !c.trim().is_empty())
        .or_else(|| query.get("ref").filter(|c| !c.trim().is_empty()))
        .map(|c| slugify_refer_code(&c))
        .filter(|c| !c.is_empty())
        .unwrap_or_default()
}

#[component]
pub fn ReferralLandingPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let initial_code = resolve_refer_code(&params.get(), &query.get());

    view! {
        <Title text="Folio — Friends & Family Referral"/>
        <Meta name="description" content="Share Folio with landlords you know. 1–2 units are free forever. Join the waitlist through a friend or family referral."/>
        <Meta property="og:title" content="Folio — Friends & Family Referral"/>
        <Meta property="og:description" content="1–2 units free forever. Join Folio through a friend or family referral."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/refer"/>

        <div class="folio-mktg">
            <MarketingNav
                section_links=REFER_SECTION_LINKS
                cta_label=DEFAULT_MARKETING_NAV_CTA
                cta_href="#refer-waitlist"
            />
            <ReferHero referred_by=initial_code.clone()/>
            <ReferFreeTier/>
            <ReferWaitlist referred_by=initial_code.clone()/>
            <ReferShareBlock initial_code=initial_code/>
            <ReferFooter/>
        </div>
    }
}

#[component]
fn ReferHero(referred_by: String) -> impl IntoView {
    let referred_note = if referred_by.is_empty() {
        None
    } else {
        Some(format!("Referred by {referred_by}"))
    };

    view! {
        <section class="mktg-hero" style="min-height:72vh;padding-bottom:3.5rem;">
            <div class="mktg-hero-grid-overlay"></div>
            <div class="mktg-hero-inner" style="text-align:center;max-width:760px;">
                <div class="mktg-eyebrow">
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"favorite"</span>
                    " Friends & Family · Referral"
                </div>
                <h1 class="mktg-hero-h1">
                    "Know a landlord?"
                    <span class="mktg-h1-accent">" Send them Folio."</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:560px;margin:1.5rem auto 0;">
                    "Folio is free forever for 1–2 units — perfect for friends and family who own a property or two. \
                     Share your link, they join the waitlist, and we know who helped introduce them."
                </p>
                {referred_note.map(|note| view! {
                    <p style="margin-top:1.25rem;font-size:.85rem;color:#06d6a0;font-weight:600;">{note}</p>
                })}
                <div style="display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2rem;">
                    <a href="#refer-waitlist" class="mktg-btn-accent mktg-btn-lg">"Join free →"</a>
                    <a href="#refer-share" class="mktg-btn-ghost">"Get my share link"</a>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ReferFreeTier() -> impl IntoView {
    view! {
        <section id="refer-free" class="mktg-section" style="background:rgba(6,214,160,.04);border-top:1px solid rgba(6,214,160,.12);border-bottom:1px solid rgba(6,214,160,.12);">
            <div class="mktg-section-inner" style="text-align:center;">
                <p class="mktg-section-eyebrow">"Why this is easy to share"</p>
                <h2 class="mktg-section-h2">"1–2 units? Free forever."</h2>
                <p class="mktg-section-sub" style="max-width:560px;margin:0 auto 2rem;">
                    "No credit card for the Free plan. Landlords managing their own small portfolio get leases, tenants, and maintenance — at zero cost."
                </p>
                <div class="mktg-feature-grid" style="max-width:900px;margin:0 auto;">
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"home"</span>
                        <h3>"Free · up to 2 units"</h3>
                        <p>"Landlord dashboard, lease management, tenant portal, and maintenance requests — forever free."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"trending_up"</span>
                        <h3>"Grow when they grow"</h3>
                        <p>"More doors? Paid plans start when they need them. No surprise fees on the free tier."</p>
                    </div>
                    <div class="mktg-feature-card">
                        <span class="material-symbols-outlined" style="color:#06d6a0;font-variation-settings:'FILL' 1">"group_add"</span>
                        <h3>"You get credit"</h3>
                        <p>"Every signup through your link is attributed to you in our Friends & Family campaign."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ReferWaitlist(referred_by: String) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let role = RwSignal::new("Landlord".to_string());
    let size = RwSignal::new("1–2 units".to_string());
    let phone = RwSignal::new(String::new());
    let referred = RwSignal::new(referred_by);
    let submitted = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let err_msg = RwSignal::new(String::new());
    let position = RwSignal::new(0u32);

    let submit = move |_| {
        let e = email.get().trim().to_string();
        if loading.get() || e.is_empty() || !e.contains('@') {
            err_msg.set("Please enter a valid email address.".to_string());
            return;
        }
        loading.set(true);
        err_msg.set(String::new());
        let phone_val = phone.get();
        let phone_json = if phone_val.trim().is_empty() {
            serde_json::Value::Null
        } else {
            phone_val.into()
        };
        let ref_code = referred.get();
        let ref_json = if ref_code.is_empty() {
            serde_json::Value::Null
        } else {
            ref_code.clone().into()
        };
        #[cfg(feature = "hydrate")]
        let landing_url = web_sys::window()
            .and_then(|w| w.location().href().ok())
            .unwrap_or_default();
        #[cfg(not(feature = "hydrate"))]
        let landing_url = String::new();
        #[cfg(feature = "hydrate")]
        let referrer_json = web_sys::window()
            .and_then(|w| w.document())
            .map(|d| d.referrer())
            .filter(|r| !r.is_empty())
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null);
        #[cfg(not(feature = "hydrate"))]
        let referrer_json = serde_json::Value::Null;
        let body = serde_json::json!({
            "email": e,
            "role": role.get(),
            "portfolio_size_label": size.get(),
            "phone": phone_json,
            "source": LEAD_SOURCE,
            "utm_source": UTM_SOURCE,
            "utm_medium": UTM_MEDIUM,
            "utm_campaign": UTM_CAMPAIGN,
            "utm_content": ref_json,
            "referred_by": if ref_code.is_empty() { serde_json::Value::Null } else { ref_code.into() },
            "landing_url": landing_url,
            "referrer": referrer_json,
        });
        leptos::task::spawn_local(async move {
            let resp = gloo_net::http::Request::post(&FolioMarketingSlug::Folio.waitlist_path())
                .header("Content-Type", "application/json")
                .body(body.to_string())
                .unwrap()
                .send()
                .await;
            loading.set(false);
            match resp {
                Ok(r) if r.ok() => {
                    if let Ok(v) = r.json::<serde_json::Value>().await {
                        position.set(v.get("position").and_then(|p| p.as_u64()).unwrap_or(0) as u32);
                    }
                    submitted.set(true);
                }
                Ok(_) => err_msg.set("We couldn't join the waitlist. Please try again.".to_string()),
                Err(_) => err_msg.set("Network issue. Please try again in a moment.".to_string()),
            }
        });
    };

    view! {
        <section id="refer-waitlist" class="mktg-section">
            <div class="mktg-section-inner" style="max-width:560px;margin:0 auto;">
                <p class="mktg-section-eyebrow" style="text-align:center;">"Join the waitlist"</p>
                <h2 class="mktg-section-h2" style="text-align:center;">"Claim your free Folio spot."</h2>
                <p class="mktg-section-sub" style="text-align:center;margin-bottom:2rem;">
                    "Tell us a bit about yourself — takes under a minute."
                </p>

                {move || if submitted.get() {
                    view! {
                        <div class="mktg-success-card">
                            <span class="material-symbols-outlined" style="font-size:2rem;color:#06d6a0;font-variation-settings:'FILL' 1">"check_circle"</span>
                            <div>
                                <div class="mktg-success-h">"You're on the list!"</div>
                                <div class="mktg-success-sub">
                                    {move || {
                                        let p = position.get();
                                        if p > 0 {
                                            format!("Position #{p}. We'll reach out before launch.")
                                        } else {
                                            "We'll reach out before launch with early access details.".to_string()
                                        }
                                    }}
                                </div>
                                <a href="#refer-share" style="display:inline-block;margin-top:.75rem;color:#06d6a0;font-size:.85rem;font-weight:600;">
                                    "Share Folio with another landlord →"
                                </a>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="refer-form">
                            <label class="refer-label">"Email"</label>
                            <input
                                type="email"
                                class="mktg-wl-input"
                                placeholder="you@email.com"
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                            <label class="refer-label">"I am a…"</label>
                            <select
                                class="mktg-wl-select"
                                prop:value=move || role.get()
                                on:change=move |ev| role.set(event_target_value(&ev))
                            >
                                <option value="Landlord">"Landlord"</option>
                                <option value="Property Manager">"Property Manager"</option>
                                <option value="STR Host">"STR Host"</option>
                                <option value="Investor">"Investor"</option>
                                <option value="Other">"Other"</option>
                            </select>
                            <label class="refer-label">"Portfolio size"</label>
                            <select
                                class="mktg-wl-select"
                                prop:value=move || size.get()
                                on:change=move |ev| size.set(event_target_value(&ev))
                            >
                                <option value="1–2 units">"1–2 units (free forever)"</option>
                                <option value="1–5 units">"1–5 units"</option>
                                <option value="6–20 units">"6–20 units"</option>
                                <option value="21–100 units">"21–100 units"</option>
                                <option value="100+ units">"100+ units"</option>
                            </select>
                            <label class="refer-label">"Phone (optional)"</label>
                            <input
                                type="tel"
                                class="mktg-wl-input"
                                placeholder="+1 …"
                                prop:value=move || phone.get()
                                on:input=move |ev| phone.set(event_target_value(&ev))
                            />
                            <Show when=move || !referred.get().is_empty() fallback=|| ()>
                                <p style="font-size:.78rem;color:#8a97b0;margin:.5rem 0 0;">
                                    "Referred by "
                                    <strong style="color:#e8edf5;">{move || referred.get()}</strong>
                                </p>
                            </Show>
                            <button
                                class="mktg-btn-accent mktg-btn-lg"
                                style="width:100%;justify-content:center;margin-top:1.25rem;"
                                disabled=move || loading.get()
                                on:click=submit
                            >
                                {move || if loading.get() { "Submitting…".to_string() } else { "Get started".to_string() }}
                            </button>
                            <Show when=move || !err_msg.get().is_empty() fallback=|| ()>
                                <p style="font-size:.78rem;color:#f87171;margin-top:.75rem;">{move || err_msg.get()}</p>
                            </Show>
                            <p style="font-size:.75rem;color:#6b7280;margin-top:.75rem;text-align:center;">
                                "No credit card. Free for 1–2 units."
                            </p>
                        </div>
                    }.into_any()
                }}
            </div>
        </section>
    }
}

#[component]
fn ReferShareBlock(initial_code: String) -> impl IntoView {
    let name = RwSignal::new(if initial_code.is_empty() {
        String::new()
    } else {
        initial_code
    });
    let link = RwSignal::new(String::new());
    let copied = RwSignal::new(false);

    let build_link = move || {
        let slug = slugify_refer_code(&name.get());
        if slug.is_empty() {
            link.set(String::new());
            return;
        }
        #[cfg(feature = "hydrate")]
        {
            let origin = web_sys::window()
                .and_then(|w| w.location().origin().ok())
                .unwrap_or_else(|| "https://folio1.atlas.oply.co".to_string());
            link.set(format!("{origin}/refer/{slug}"));
        }
        #[cfg(not(feature = "hydrate"))]
        {
            link.set(format!("https://folio1.atlas.oply.co/refer/{slug}"));
        }
        copied.set(false);
    };

    let copy_link = move |_| {
        let url = link.get();
        if url.is_empty() {
            return;
        }
        #[cfg(feature = "hydrate")]
        {
            if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                let _ = clipboard.write_text(&url);
                copied.set(true);
            }
        }
    };

    view! {
        <section id="refer-share" class="mktg-section" style="background:rgba(255,107,53,.04);border-top:1px solid rgba(255,107,53,.12);">
            <div class="mktg-section-inner" style="max-width:560px;margin:0 auto;text-align:center;">
                <p class="mktg-section-eyebrow">"For friends & family"</p>
                <h2 class="mktg-section-h2">"Get your personal share link."</h2>
                <p class="mktg-section-sub" style="margin-bottom:1.75rem;">
                    "Enter your name (or a short handle). Anyone who joins through your link counts toward your referrals."
                </p>
                <div class="refer-form" style="text-align:left;">
                    <label class="refer-label">"Your name or handle"</label>
                    <input
                        type="text"
                        class="mktg-wl-input"
                        placeholder="e.g. Alice or maria-chen"
                        prop:value=move || name.get()
                        on:input=move |ev| {
                            name.set(event_target_value(&ev));
                            copied.set(false);
                        }
                    />
                    <button
                        class="mktg-btn-accent"
                        style="width:100%;justify-content:center;margin-top:1rem;"
                        on:click=move |_| build_link()
                    >
                        "Generate link"
                    </button>
                    <Show when=move || !link.get().is_empty() fallback=|| ()>
                        <div style="margin-top:1.25rem;padding:1rem;border-radius:10px;border:1px solid rgba(255,255,255,.12);background:rgba(255,255,255,.04);">
                            <div style="font-size:.72rem;color:#8a97b0;margin-bottom:.35rem;text-transform:uppercase;letter-spacing:.06em;">"Your link"</div>
                            <code style="font-size:.85rem;color:#06d6a0;word-break:break-all;">{move || link.get()}</code>
                            <div style="display:flex;gap:.75rem;margin-top:1rem;flex-wrap:wrap;">
                                <button class="mktg-btn-accent" on:click=copy_link>
                                    {move || if copied.get() { "Copied!".to_string() } else { "Copy link".to_string() }}
                                </button>
                                <a href=move || link.get() class="mktg-btn-ghost" rel="external">"Open link"</a>
                            </div>
                        </div>
                    </Show>
                </div>
            </div>
        </section>
    }
}

#[component]
fn ReferFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Friends & Family Referral"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/" rel="external">"Landlords"</a>
                    <a href="/property-managers" rel="external">"Property Managers"</a>
                    <a href="/founding" rel="external">"Founding"</a>
                    <a href="/beta" rel="external">"Beta"</a>
                    <a href="/login" rel="external">"Sign in"</a>
                </div>
                <div class="mktg-footer-legal">"© 2026 Folio · Atlas Platform"</div>
            </div>
        </footer>
    }
}
