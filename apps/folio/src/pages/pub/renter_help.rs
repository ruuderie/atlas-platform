//! Public renter help page — zero-auth, cold-traffic entry point.
//!
//! Route: GET /help
//!
//! # Entry points
//! - `/help`                          — generic landing (hero → category → vendors → form)
//! - `/help?vendor_id=<uuid>`         — deep-link from vendor website/QR/business card
//!                                      skips hero + category, jumps straight to form
//! - `/help?trade=electrical`         — deep-link from Google / vendor category page
//!                                      skips hero, goes to pre-filtered vendor list
//! - `/help?vendor_id=X&utm_source=Y` — UTM tracking for growth analytics
//!
//! # Flow
//! Hero → Category picker → Vendor list (sidebar + card grid) →
//! Request form (split-panel) → Confirmation + landlord lead-gen
//!
//! # SEO
//! Fully rendered server-side. Meta title + description set per step via
//! Leptos `<Title>` / `<Meta>` components.

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use uuid::Uuid;
use std::str::FromStr;

// ── Step enum ────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum HelpStep {
    /// Hero + category picker — shown when no query params present.
    Hero,
    /// Vendor list — filtered by trade. Entry from ?trade= or after category pick.
    Vendors,
    /// Request form — vendor selected. Entry from ?vendor_id= or after vendor pick.
    Form,
    /// Confirmation + landlord lead-gen.
    Done,
}

// ── Vendor data model (matches /api/pub/vendors response) ────────────────────

#[derive(Clone, Debug, PartialEq)]
struct VendorItem {
    id:            String,
    business_name: String,
    trade_type:    Option<String>,
    avg_score:     Option<f64>,
    review_count:  i64,
    bio:           Option<String>,
    verified:      bool,
    initials:      String,
    color:         &'static str,
}

static AVATAR_COLORS: &[&str] = &[
    "#0d1421", "#0284c7", "#7c3aed", "#059669", "#dc2626", "#d97706",
];

fn make_initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.len() {
        0 => "?".to_string(),
        1 => words[0].chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default(),
        _ => {
            let a = words[0].chars().next().unwrap_or('?').to_uppercase().to_string();
            let b = words[1].chars().next().unwrap_or('?').to_uppercase().to_string();
            format!("{a}{b}")
        }
    }
}

fn pick_color(idx: usize) -> &'static str {
    AVATAR_COLORS[idx % AVATAR_COLORS.len()]
}

// ── Trade categories ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct TradeCategory {
    id:    &'static str,
    label: &'static str,
    icon:  &'static str,
    color: &'static str,
    bg:    &'static str,
}

static CATEGORIES: &[TradeCategory] = &[
    TradeCategory { id: "electrical",    label: "Electrical",    icon: "bolt",              color: "#d97706", bg: "#fef3c7" },
    TradeCategory { id: "plumbing",      label: "Plumbing",      icon: "water_drop",        color: "#2563eb", bg: "#dbeafe" },
    TradeCategory { id: "hvac",          label: "HVAC",          icon: "ac_unit",           color: "#db2777", bg: "#fce7f3" },
    TradeCategory { id: "roofing",       label: "Roofing",       icon: "roofing",           color: "#059669", bg: "#d1fae5" },
    TradeCategory { id: "pest_control",  label: "Pest Control",  icon: "pest_control",      color: "#7c3aed", bg: "#ede9fe" },
    TradeCategory { id: "general",       label: "General",       icon: "handyman",          color: "#ea580c", bg: "#ffedd5" },
    TradeCategory { id: "landscaping",   label: "Landscaping",   icon: "yard",              color: "#16a34a", bg: "#f0fdf4" },
    TradeCategory { id: "cleaning",      label: "Cleaning",      icon: "cleaning_services", color: "#475569", bg: "#f1f5f9" },
];

// ── Page component ───────────────────────────────────────────────────────────

/// Public renter help page. Zero-auth, SEO-indexed.
#[component]
pub fn RenterHelpPage() -> impl IntoView {
    let query = use_query_map();

    // Derive initial step from query params
    let initial_step = {
        let q = query.get();
        let has_vendor = q.get("vendor_id").is_some();
        let has_trade  = q.get("trade").is_some();
        if has_vendor       { HelpStep::Form }
        else if has_trade   { HelpStep::Vendors }
        else                { HelpStep::Hero }
    };
    let pre_vendor_id = {
        let q = query.get();
        q.get("vendor_id").and_then(|v| Uuid::from_str(&v).ok())
    };
    let pre_trade = {
        let q = query.get();
        q.get("trade")
    };
    // utm_source stored as a signal so it's Copy-safe across Fn closures
    let utm_source = {
        let q = query.get();
        q.get("utm_source")
    };
    let utm = RwSignal::new(utm_source);

    let (step, set_step)               = signal(initial_step);
    let (selected_trade, set_trade)    = signal(pre_trade.unwrap_or_else(|| "electrical".to_string()));
    let (selected_vendor, set_vendor)  = signal(Option::<VendorItem>::None);
    let (urgency, set_urgency)         = signal("this_week".to_string());
    let (description, set_description) = signal(String::new());
    let (address, set_address)         = signal(String::new());
    let (renter_name, set_renter_name) = signal(String::new());
    let (renter_email, set_email)      = signal(String::new());
    let (renter_phone, set_phone)      = signal(String::new());
    let (landlord_email, set_ll_email) = signal(String::new());
    let (invite_sent, set_invite_sent) = signal(false);
    let (request_id, set_request_id)   = signal(Option::<String>::None);
    // Stub vendor data — in production this comes from /api/pub/vendors
    let stub_vendors: Vec<VendorItem> = vec![
        VendorItem { id: "aaaaaaaa-0000-0000-0000-000000000001".to_string(), business_name: "Rivera & West Electric".to_string(), trade_type: Some("electrical".to_string()), avg_score: Some(4.9), review_count: 127, bio: Some("Licensed since 2014. Residential & commercial.".to_string()), verified: true, initials: "RW".to_string(), color: "#0d1421" },
        VendorItem { id: "aaaaaaaa-0000-0000-0000-000000000002".to_string(), business_name: "Metro Power Solutions".to_string(), trade_type: Some("electrical".to_string()), avg_score: Some(4.6), review_count: 84, bio: None, verified: true, initials: "MP".to_string(), color: "#0284c7" },
        VendorItem { id: "aaaaaaaa-0000-0000-0000-000000000003".to_string(), business_name: "Circuit Works LLC".to_string(), trade_type: Some("electrical".to_string()), avg_score: Some(4.4), review_count: 39, bio: Some("Panel upgrades & EV charging specialists.".to_string()), verified: false, initials: "CW".to_string(), color: "#7c3aed" },
    ];
    let (vendors, _set_vendors) = signal(stub_vendors);


    view! {
        // SEO meta
        <leptos_meta::Title text="Get Help with Your Rental | Folio"/>
        <leptos_meta::Meta name="description" content="Find vetted contractors for your rental — no landlord required. Browse the Folio vendor network and submit a service request for free."/>

        <div class="pub-renter-shell">

            // ── Header ───────────────────────────────────────────────────────
            <header class="pub-renter-header" id="renter-header">
                <div class="pub-renter-logo">
                    <div class="pub-renter-logo-mark">
                        <span class="ms msf" style="font-size:16px;color:#fff">"apartment"</span>
                    </div>
                    <span class="pub-renter-wordmark">"Folio"</span>
                </div>
                <nav class="pub-renter-nav">
                    <a href="#" class="pub-renter-nav-link" id="renter-nav-how-it-works">"How it works"</a>
                    <a href="#" class="pub-renter-nav-link" id="renter-nav-vendors">"For vendors"</a>
                    <a href="/register?role=landlord" class="pub-renter-nav-cta" id="renter-nav-landlord-signup">
                        "Landlord sign-up →"
                    </a>
                </nav>
            </header>

            // ── STEP: HERO ────────────────────────────────────────────────────
            <Show when=move || step.get() == HelpStep::Hero>
                <section class="renter-hero" id="renter-step-hero">
                    <div class="renter-hero-copy">
                        <div class="renter-hero-eyebrow">
                            <span class="ms msf" style="font-size:14px">"apartment"</span>
                            " Renter Help Center"
                        </div>
                        <h1 class="renter-hero-title">
                            "Something broken"<br/>
                            "in your rental?"<br/>
                            <em>"We got you."</em>
                        </h1>
                        <p class="renter-hero-sub">
                            "Connect directly with vetted contractors in the Folio network — "
                            "no landlord needed to get the ball rolling."
                        </p>
                        <div class="renter-trust-row">
                            <span class="renter-trust-badge">
                                <span class="ms msf">"verified"</span>"Vetted vendors"
                            </span>
                            <span class="renter-trust-badge">
                                <span class="ms msf">"star"</span>"Real reviews"
                            </span>
                            <span class="renter-trust-badge">
                                <span class="ms msf">"lock"</span>"No account needed"
                            </span>
                        </div>
                    </div>
                    <div class="renter-hero-card" id="renter-hero-card">
                        <h3 class="renter-hero-card-title">"What do you need fixed?"</h3>
                        <div class="renter-search-wrap">
                            <span class="ms" style="color:#94a3b8;font-size:20px">"search"</span>
                            <input type="text" placeholder="e.g. leaky faucet, HVAC, broken lock…"
                                id="renter-hero-search-input"
                                class="renter-search-input"/>
                        </div>
                        <div class="renter-cat-grid" id="renter-category-grid">
                            {CATEGORIES.iter().map(|cat| {
                                let cat_id = cat.id;
                                let cat_label = cat.label;
                                let cat_icon  = cat.icon;
                                let cat_color = cat.color;
                                let cat_bg    = cat.bg;
                                let is_sel    = move || selected_trade.get() == cat_id;
                                view! {
                                    <div
                                        class=move || if is_sel() { "renter-cat-card renter-cat-card--sel" } else { "renter-cat-card" }
                                        id=format!("renter-cat-{cat_id}")
                                        on:click=move |_| set_trade.set(cat_id.to_string())
                                    >
                                        <div class="renter-cat-icon" style=format!("background:{cat_bg}")>
                                            <span class="ms msf" style=format!("color:{cat_color};font-size:20px")>
                                                {cat_icon}
                                            </span>
                                        </div>
                                        <span class="renter-cat-label">{cat_label}</span>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <button
                            id="renter-hero-find-btn"
                            type="button"
                            class="renter-search-btn"
                            on:click=move |_| set_step.set(HelpStep::Vendors)
                        >
                            <span class="ms msf" style="font-size:18px">"arrow_forward"</span>
                            "Find Available Vendors"
                        </button>
                    </div>
                </section>
            </Show>

            // ── STEP: VENDOR LIST ─────────────────────────────────────────────
            <Show when=move || step.get() == HelpStep::Vendors>
                <div class="renter-workspace" id="renter-step-vendors">
                    // Sidebar filters
                    <aside class="renter-sidebar" id="renter-sidebar">
                        <div class="renter-sidebar-section">
                            <div class="renter-sidebar-title">"Trade Type"</div>
                            <ul class="renter-cat-nav" id="renter-trade-nav">
                                {CATEGORIES.iter().map(|cat| {
                                    let cat_id    = cat.id;
                                    let cat_label = cat.label;
                                    let cat_icon  = cat.icon;
                                    let cat_color = cat.color;
                                    let is_sel    = move || selected_trade.get() == cat_id;
                                    view! {
                                        <li
                                            class=move || if is_sel() { "renter-cat-nav-item renter-cat-nav-item--sel" } else { "renter-cat-nav-item" }
                                            id=format!("renter-nav-trade-{cat_id}")
                                            on:click=move |_| set_trade.set(cat_id.to_string())
                                        >
                                            <span class="ms msf" style=format!("color:{cat_color}")>{cat_icon}</span>
                                            {cat_label}
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </div>
                        <div class="renter-sidebar-divider"></div>
                        <div class="renter-sidebar-section">
                            <div class="renter-sidebar-title">"Rating"</div>
                            <div class="renter-filter-chips">
                                <span class="renter-filter-chip renter-filter-chip--sel">"4.5+ ★"</span>
                                <span class="renter-filter-chip">"4.0+ ★"</span>
                                <span class="renter-filter-chip">"Any"</span>
                            </div>
                        </div>
                        <div class="renter-sidebar-section">
                            <div class="renter-sidebar-title">"Availability"</div>
                            <div class="renter-filter-chips">
                                <span class="renter-filter-chip renter-filter-chip--sel">"Any time"</span>
                                <span class="renter-filter-chip">"This week"</span>
                                <span class="renter-filter-chip">"Emergency"</span>
                            </div>
                        </div>
                        <div class="renter-info-box">
                            <div class="renter-info-box-title">"💡 No account needed"</div>
                            <div class="renter-info-box-sub">"Send requests directly to vendors. They'll contact you to schedule."</div>
                        </div>
                    </aside>
                    // Vendor card grid
                    <div class="renter-main" id="renter-vendor-list">
                        <div class="renter-list-header">
                            <div>
                                <h2 class="renter-list-title">
                                    {move || format!("{} Vendors Near You", selected_trade.get().replace('_', " ").split_whitespace().map(|w| {
                                        let mut c = w.chars();
                                        match c.next() {
                                            None => String::new(),
                                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                                        }
                                    }).collect::<Vec<_>>().join(" "))}
                                </h2>
                                <p class="renter-list-sub">"Vetted vendors available · Browse and send a free request"</p>
                            </div>
                        </div>
                        <div class="renter-vendor-grid" id="renter-vendor-grid">
                            {move || vendors.get().into_iter().enumerate().map(|(idx, v)| {
                                let v_clone   = v.clone();
                                let v_for_sel = v.clone();
                                let set_step2 = set_step;
                                let set_vendor2 = set_vendor;
                                let color  = pick_color(idx);
                                let initials = make_initials(&v.business_name);
                                let avg = v.avg_score.map(|s| format!("{:.1}", s)).unwrap_or_else(|| "New".to_string());
                                let stars = v.avg_score.map(|s| "★".repeat(s.round() as usize)).unwrap_or_else(|| "★★★★★".to_string());
                                view! {
                                    <div class="renter-vendor-card" id=format!("renter-vendor-{idx}")>
                                        <div class="rvc-top">
                                            <div class="rvc-avatar" style=format!("background:{color}")>
                                                {initials.clone()}
                                            </div>
                                            <div class="rvc-body">
                                                <div class="rvc-name">{v.business_name.clone()}</div>
                                                <div class="rvc-trade">
                                                    {v.trade_type.clone().unwrap_or_else(|| "General".to_string())}
                                                </div>
                                                <div class="rvc-rating">
                                                    <span class="rvc-stars">{stars}</span>
                                                    <span class="rvc-count">{format!("{avg} · {} reviews", v.review_count)}</span>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="rvc-tags">
                                            {if v.verified { Some(view! {
                                                <span class="rvc-tag rvc-tag--verified">
                                                    <span class="ms msf" style="font-size:10px">"verified"</span>
                                                    "Verified"
                                                </span>
                                            }) } else { None }}
                                        </div>
                                        <div class="rvc-footer">
                                            <button
                                                type="button"
                                                class="rvc-btn-primary"
                                                id=format!("renter-vendor-select-{idx}")
                                                on:click=move |_| {
                                                    set_vendor2.set(Some(v_for_sel.clone()));
                                                    set_step2.set(HelpStep::Form);
                                                }
                                            >
                                                "Send Request"
                                            </button>
                                            <button type="button" class="rvc-btn-ghost">"View Profile"</button>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    </div>
                </div>
            </Show>

            // ── STEP: REQUEST FORM ────────────────────────────────────────────
            <Show when=move || step.get() == HelpStep::Form>
                <div class="renter-form-split" id="renter-step-form">
                    // Left — info panel
                    <div class="renter-form-left">
                        <button
                            type="button"
                            class="renter-back-btn"
                            id="renter-form-back"
                            on:click=move |_| set_step.set(HelpStep::Vendors)
                        >
                            <span class="ms" style="font-size:18px">"arrow_back"</span>
                            "Back"
                        </button>
                        <div class="renter-form-left-title">"Send your request"</div>
                        <div class="renter-form-left-sub">
                            "Tell the vendor what you need. They'll reach out within a few hours to schedule."
                        </div>
                        // Selected vendor chip
                        {move || selected_vendor.get().map(|v| view! {
                            <div class="renter-vendor-chip">
                                <div class="rvc-avatar" style=format!("background:{};width:44px;height:44px;border-radius:11px;font-size:15px", v.color)>
                                    {v.initials.clone()}
                                </div>
                                <div>
                                    <div class="rvc-chip-name">{v.business_name.clone()}</div>
                                    <div class="rvc-chip-trade">{v.trade_type.unwrap_or_else(|| "General".to_string())}</div>
                                </div>
                                <button type="button" class="rvc-change-btn"
                                    on:click=move |_| set_step.set(HelpStep::Vendors)>
                                    "Change"
                                </button>
                            </div>
                        })}
                        // How it works
                        <div class="renter-hiw">
                            <div class="renter-hiw-title">"How it works"</div>
                            <div class="renter-hiw-item">
                                <div class="renter-hiw-num">"1"</div>
                                <div><strong>"Submit your request"</strong><br/>"Describe the issue and share your contact info."</div>
                            </div>
                            <div class="renter-hiw-item">
                                <div class="renter-hiw-num">"2"</div>
                                <div><strong>"Vendor responds"</strong><br/>"They'll call or email you within a few hours."</div>
                            </div>
                            <div class="renter-hiw-item">
                                <div class="renter-hiw-num">"3"</div>
                                <div><strong>"Get it fixed"</strong><br/>"Schedule the job — no platform fees for renters."</div>
                            </div>
                        </div>
                    </div>
                    // Right — form
                    <div class="renter-form-right">
                        <div class="renter-form-card" id="renter-form-card">
                            <div class="renter-form-title">"Request details"</div>
                            <div class="renter-form-sub">
                                "All fields are shared only with the vendor you selected."
                            </div>
                            // Urgency
                            <div class="renter-form-group">
                                <label class="renter-flabel">"How urgent is this?"</label>
                                <div class="renter-urgency-row">
                                    {[("not_urgent","🟢","Not urgent"),("this_week","🟡","This week"),("emergency","🔴","Emergency")].iter().map(|(id, emoji, label)| {
                                        let id_str = *id;
                                        let is_sel = move || urgency.get() == id_str;
                                        view! {
                                            <div
                                                class=move || if is_sel() { "renter-urgency-btn renter-urgency-btn--sel" } else { "renter-urgency-btn" }
                                                id=format!("renter-urgency-{id_str}")
                                                on:click=move |_| set_urgency.set(id_str.to_string())
                                            >
                                                <span style="font-size:22px">{*emoji}</span>
                                                <span class="renter-urgency-label">{*label}</span>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            </div>
                            // Description
                            <div class="renter-form-group">
                                <label class="renter-flabel" for="renter-description">"What's the issue?"</label>
                                <textarea
                                    id="renter-description"
                                    class="renter-fi"
                                    rows="4"
                                    placeholder="e.g. The circuit breaker in the kitchen keeps tripping…"
                                    on:input=move |e| set_description.set(event_target_value(&e))
                                />
                            </div>
                            // Address
                            <div class="renter-form-group">
                                <label class="renter-flabel" for="renter-address">"Rental address"</label>
                                <input
                                    id="renter-address"
                                    type="text"
                                    class="renter-fi"
                                    placeholder="123 Main St, Apt 4B, Miami FL 33101"
                                    on:input=move |e| set_address.set(event_target_value(&e))
                                />
                            </div>
                            <div class="renter-form-divider"></div>
                            // Contact info
                            <div class="renter-form-group">
                                <label class="renter-flabel">"Your contact info"</label>
                                <div class="renter-input-row" style="margin-bottom:10px">
                                    <input id="renter-name" type="text" class="renter-fi" placeholder="Your name"
                                        on:input=move |e| set_renter_name.set(event_target_value(&e))/>
                                    <input id="renter-phone" type="tel" class="renter-fi" placeholder="Phone number"
                                        on:input=move |e| set_phone.set(event_target_value(&e))/>
                                </div>
                                <input id="renter-email" type="email" class="renter-fi" placeholder="Email (for confirmation)"
                                    on:input=move |e| set_email.set(event_target_value(&e))/>
                            </div>
                            // Submit
                            <button
                                id="renter-submit-btn"
                                type="button"
                                class="renter-submit-btn"
                                on:click=move |_| {
                                    // TODO: POST /api/pub/service-requests
                                    let _payload = (
                                        selected_vendor.get().map(|v| v.id),
                                        description.get(),
                                        urgency.get(),
                                        address.get(),
                                        renter_name.get(),
                                        renter_email.get(),
                                        renter_phone.get(),
                                        utm.get(),
                                    );
                                    set_request_id.set(Some("req-preview".to_string()));
                                    set_step.set(HelpStep::Done);
                                }
                            >
                                <span class="ms msf" style="font-size:20px">"send"</span>
                                "Send Request to Vendor"
                            </button>
                            <p class="renter-form-note">
                                "Free to send · No account required · Vendor contacts you directly"
                            </p>
                        </div>
                    </div>
                </div>
            </Show>

            // ── STEP: DONE ────────────────────────────────────────────────────
            <Show when=move || step.get() == HelpStep::Done>
                <div class="renter-confirm-layout" id="renter-step-confirm">

                    // Left — confirmation + timeline
                    <div class="renter-confirm-main">
                        <div class="renter-confirm-icon">
                            <span class="ms msf" style="font-size:32px;color:#10b981">"check_circle"</span>
                        </div>
                        <div class="renter-confirm-title">"Request sent! 🎉"</div>
                        <div class="renter-confirm-sub">
                            {move || selected_vendor.get().map(|v| v.business_name.clone()).unwrap_or_else(|| "The vendor".to_string())}
                            " received your request and will be in touch within a few hours."
                        </div>
                        <div class="renter-confirm-divider"></div>
                        <div class="renter-tl-label-header">"What happens next"</div>
                        <div class="renter-tl">
                            <div class="renter-tl-item">
                                <div class="renter-tl-dot renter-tl-dot--done">
                                    <span class="ms msf" style="font-size:14px;color:#fff">"check"</span>
                                </div>
                                <div>
                                    <div class="renter-tl-label">"Request sent"</div>
                                    <div class="renter-tl-sub">"Just now"</div>
                                </div>
                            </div>
                            <div class="renter-tl-item">
                                <div class="renter-tl-dot renter-tl-dot--pulse">
                                    <span class="ms msf" style="font-size:14px;color:#fff">"schedule"</span>
                                </div>
                                <div>
                                    <div class="renter-tl-label">"Vendor reviews your request"</div>
                                    <div class="renter-tl-sub">"Usually within 2–4 hours"</div>
                                </div>
                            </div>
                            <div class="renter-tl-item">
                                <div class="renter-tl-dot renter-tl-dot--idle"></div>
                                <div>
                                    <div class="renter-tl-label">"They contact you to schedule"</div>
                                    <div class="renter-tl-sub">"Via phone or email you provided"</div>
                                </div>
                            </div>
                            <div class="renter-tl-item">
                                <div class="renter-tl-dot renter-tl-dot--idle"></div>
                                <div>
                                    <div class="renter-tl-label">"Leave a review after the job"</div>
                                    <div class="renter-tl-sub">"Help other renters find trusted vendors"</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    // Right — lead gen panels
                    <div class="renter-lead-panel">

                        // Landlord invite
                        <div class="renter-landlord-card" id="renter-landlord-invite-card">
                            <div class="renter-ll-eyebrow">"💡 While you wait"</div>
                            <div class="renter-ll-title">"Does your landlord know about Folio?"</div>
                            <div class="renter-ll-sub">
                                "Invite them to join — they can handle maintenance requests properly, "
                                "track your lease, and communicate with you all in one place."
                            </div>
                            <ul class="renter-ll-benefits">
                                <li><span class="ms msf" style="color:#34d399">"check_circle"</span>" Free to start — no credit card"</li>
                                <li><span class="ms msf" style="color:#34d399">"check_circle"</span>" They can see your requests and respond"</li>
                                <li><span class="ms msf" style="color:#34d399">"check_circle"</span>" Digital lease and payment tracking"</li>
                            </ul>
                            <Show when=move || !invite_sent.get()>
                                <input
                                    id="renter-landlord-email"
                                    type="email"
                                    class="renter-ll-input"
                                    placeholder="Landlord's email address…"
                                    on:input=move |e| set_ll_email.set(event_target_value(&e))
                                />
                                <button
                                    id="renter-landlord-invite-btn"
                                    type="button"
                                    class="renter-ll-btn"
                                    on:click=move |_| {
                                        // TODO: POST /api/pub/landlord-invites { email, request_id }
                                        let _ = landlord_email.get();
                                        set_invite_sent.set(true);
                                    }
                                >
                                    "Send Landlord Invite →"
                                </button>
                            </Show>
                            <Show when=move || invite_sent.get()>
                                <div class="renter-ll-sent">
                                    <span class="ms msf" style="font-size:20px;color:#34d399">"check_circle"</span>
                                    "Invite sent! We'll follow up with your landlord."
                                </div>
                            </Show>
                            <a class="renter-skip-link" href="#" id="renter-skip-invite">"Skip for now"</a>
                        </div>

                        // Create renter account nudge
                        <div class="renter-account-card" id="renter-account-nudge">
                            <div class="renter-account-title">"Track your request"</div>
                            <div class="renter-account-sub">
                                "Create a free renter account to track this request, "
                                "store your lease documents, and submit future requests faster."
                            </div>
                            <a href="/register?role=tenant" class="renter-account-btn" id="renter-create-account-btn">
                                "Create a free renter account →"
                            </a>
                            <p class="renter-account-note">"Takes 30 seconds · No credit card"</p>
                        </div>

                    </div>
                </div>
            </Show>

        </div>
    }
}
