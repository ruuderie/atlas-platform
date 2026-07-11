//! ReferralVendorLandingPage — Friends & Family Vendors shareable referral capture.
//!
//! Served at: `/refer/vendors` and `/refer/vendors/:code`
//!
//! Must be registered **before** `/refer/:code` so `vendors` is not captured as a code.
//! Attribution: `utm_campaign=friends_family_vendors`, `role: Vendor`, waitlist → folio-vendor.

use crate::components::marketing_nav::{
    MarketingNav, MarketingNavSectionLink, DEFAULT_MARKETING_NAV_CTA,
};
use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::{use_params_map, use_query_map};
use shared_ui::marketing::FolioMarketingSlug;

const REFER_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Why Folio",
        href: "#refer-why",
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
const UTM_MEDIUM: &str = "friends_family_vendors";
const UTM_CAMPAIGN: &str = "friends_family_vendors";
const LEAD_SOURCE: &str = "referral:friends_family_vendors";

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

#[component]
pub fn ReferralVendorLandingPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let initial_code = resolve_refer_code(&params.get(), &query.get());

    view! {
        <Title text="Folio — Friends & Family Vendors"/>
        <Meta name="description" content="Share Folio with vendors and contractors you trust. Join the vendor waitlist through a Friends & Family referral."/>
        <Meta property="og:title" content="Folio — Friends & Family Vendors"/>
        <Meta property="og:description" content="Get on the Folio vendor marketplace through a friend or family referral."/>
        <Link rel="canonical" href="https://folio1.atlas.oply.co/refer/vendors"/>

        <div class="folio-mktg">
            <MarketingNav
                section_links=REFER_SECTION_LINKS
                cta_label=DEFAULT_MARKETING_NAV_CTA
                cta_href="#refer-waitlist"
            />
            <VendorReferHero referred_by=initial_code.clone()/>
            <VendorReferWhy/>
            <VendorReferWaitlist referred_by=initial_code.clone()/>
            <VendorReferShareBlock initial_code=initial_code/>
            <VendorReferFooter/>
        </div>
    }
}

#[component]
fn VendorReferHero(referred_by: String) -> impl IntoView {
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
                    <span class="material-symbols-outlined" style="font-size:14px;font-variation-settings:'FILL' 1">"handshake"</span>
                    " Friends & Family · Vendors"
                </div>
                <h1 class="mktg-hero-h1">
                    "Know a great vendor?"
                    <span class="mktg-h1-accent">" Send them Folio."</span>
                </h1>
                <p class="mktg-hero-sub" style="max-width:560px;margin:1.5rem auto 0;">
                    "Share your link — they join the vendor waitlist, and you get credit in the Friends & Family Vendors campaign."
                </p>
                {referred_note.map(|note| view! {
                    <p style="margin-top:1rem;font-size:.85rem;color:#06d6a0;font-weight:600;">{note}</p>
                })}
                <div style="display:flex;gap:1rem;justify-content:center;flex-wrap:wrap;margin-top:2rem;">
                    <a href="#refer-waitlist" class="mktg-btn-accent mktg-btn-lg">"Join as a vendor →"</a>
                    <a href="#refer-share" class="mktg-btn-ghost">"Get my share link"</a>
                </div>
            </div>
        </section>
    }
}

#[component]
fn VendorReferWhy() -> impl IntoView {
    view! {
        <section id="refer-why" class="mktg-section">
            <div class="mktg-section-inner" style="max-width:900px;margin:0 auto;text-align:center;">
                <p class="mktg-section-eyebrow">"Why Folio for vendors"</p>
                <h2 class="mktg-section-h2">"Jobs, invoices, and payments in one place."</h2>
                <p class="mktg-section-sub" style="margin-bottom:2rem;">
                    "Marketplace visibility, dispatched work, and in-platform invoicing — without chasing landlords for checks."
                </p>
                <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:1rem;text-align:left;">
                    <div class="mktg-feature-card" style="padding:1.25rem;">
                        <span class="material-symbols-outlined" style="color:#ff6b35;">"work"</span>
                        <h3 style="font-size:.95rem;margin:.5rem 0;">"Dispatched jobs"</h3>
                        <p style="font-size:.8rem;color:#8a97b0;">"Get matched to landlords who need your trade."</p>
                    </div>
                    <div class="mktg-feature-card" style="padding:1.25rem;">
                        <span class="material-symbols-outlined" style="color:#ff6b35;">"receipt_long"</span>
                        <h3 style="font-size:.95rem;margin:.5rem 0;">"Invoicing"</h3>
                        <p style="font-size:.8rem;color:#8a97b0;">"Send invoices in-app and track payment status."</p>
                    </div>
                    <div class="mktg-feature-card" style="padding:1.25rem;">
                        <span class="material-symbols-outlined" style="color:#ff6b35;">"payments"</span>
                        <h3 style="font-size:.95rem;margin:.5rem 0;">"Get paid"</h3>
                        <p style="font-size:.8rem;color:#8a97b0;">"Platform payment rails — less chase, more cash."</p>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[component]
fn VendorReferWaitlist(referred_by: String) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let trade = RwSignal::new("General Contractor".to_string());
    let phone = RwSignal::new(String::new());
    let referred = RwSignal::new(referred_by);
    let loading = RwSignal::new(false);
    let submitted = RwSignal::new(false);
    let position = RwSignal::new(0u32);
    let err_msg = RwSignal::new(String::new());

    let submit = move |_| {
        let e = email.get().trim().to_string();
        if e.is_empty() || !e.contains('@') {
            err_msg.set("Enter a valid email.".into());
            return;
        }
        loading.set(true);
        err_msg.set(String::new());
        let ref_code = referred.get();
        let ref_json = if ref_code.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::Value::String(ref_code.clone())
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
            "role": "Vendor",
            "portfolio_size_label": trade.get(),
            "phone": if phone.get().trim().is_empty() {
                serde_json::Value::Null
            } else {
                serde_json::Value::String(phone.get().trim().to_string())
            },
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
            let resp = gloo_net::http::Request::post(&FolioMarketingSlug::FolioVendor.waitlist_path())
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
                <p class="mktg-section-eyebrow" style="text-align:center;">"Join the vendor waitlist"</p>
                <h2 class="mktg-section-h2" style="text-align:center;">"Get early marketplace access."</h2>
                <p class="mktg-section-sub" style="text-align:center;margin-bottom:2rem;">
                    "Takes under a minute — we'll reach out before launch."
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
                                    "Share Folio with another vendor →"
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
                                placeholder="you@company.com"
                                prop:value=move || email.get()
                                on:input=move |ev| email.set(event_target_value(&ev))
                            />
                            <label class="refer-label">"Primary trade"</label>
                            <select
                                class="mktg-wl-select"
                                prop:value=move || trade.get()
                                on:change=move |ev| trade.set(event_target_value(&ev))
                            >
                                <option value="General Contractor">"General Contractor"</option>
                                <option value="Plumbing">"Plumbing"</option>
                                <option value="Electrical">"Electrical"</option>
                                <option value="HVAC">"HVAC"</option>
                                <option value="Cleaning">"Cleaning"</option>
                                <option value="Landscaping">"Landscaping"</option>
                                <option value="Other">"Other"</option>
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
                                {move || if loading.get() { "Submitting…".to_string() } else { "Join vendor waitlist".to_string() }}
                            </button>
                            <Show when=move || !err_msg.get().is_empty() fallback=|| ()>
                                <p style="font-size:.78rem;color:#f87171;margin-top:.75rem;">{move || err_msg.get()}</p>
                            </Show>
                        </div>
                    }.into_any()
                }}
            </div>
        </section>
    }
}

#[component]
fn VendorReferShareBlock(initial_code: String) -> impl IntoView {
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
            link.set(format!("{origin}/refer/vendors/{slug}"));
        }
        #[cfg(not(feature = "hydrate"))]
        {
            link.set(format!("https://folio1.atlas.oply.co/refer/vendors/{slug}"));
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
                <h2 class="mktg-section-h2">"Get your vendor share link."</h2>
                <p class="mktg-section-sub" style="margin-bottom:1.75rem;">
                    "Anyone who joins the vendor waitlist through your link counts toward your referrals."
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
fn VendorReferFooter() -> impl IntoView {
    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div>
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">"Friends & Family · Vendors"</div>
                </div>
                <div class="mktg-footer-links">
                    <a href="/vendors" rel="external">"Vendors"</a>
                    <a href="/refer" rel="external">"Landlord referrals"</a>
                    <a href="/founding" rel="external">"Founding"</a>
                    <a href="/login" rel="external">"Sign in"</a>
                </div>
                <div class="mktg-footer-legal">"© 2026 Folio · Atlas Platform"</div>
            </div>
        </footer>
    }
}
