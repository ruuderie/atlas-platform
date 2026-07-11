//! Shared top navigation for Folio marketing pages.
//!
//! Canonical role dropdown, section links, and CTAs — one component used by
//! every marketing landing page so labels and role coverage stay consistent.

use leptos::prelude::*;

use crate::components::lang::{get_current_lang, LanguageSwitcher};

// ── Role enum ─────────────────────────────────────────────────────────────────

/// Which marketing role page is currently active (highlights the role panel item).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarketingNavRole {
    Landlords,
    PropertyManagers,
    Brokers,
    Vendors,
    Cohosts,
    #[default]
    None,
}

impl MarketingNavRole {
    /// Roles shown in the dropdown / mobile drawer (excludes [`Self::None`]).
    pub const ALL: [Self; 5] = [
        Self::Landlords,
        Self::PropertyManagers,
        Self::Brokers,
        Self::Vendors,
        Self::Cohosts,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Landlords => "landlords",
            Self::PropertyManagers => "property_managers",
            Self::Brokers => "brokers",
            Self::Vendors => "vendors",
            Self::Cohosts => "cohosts",
            Self::None => "none",
        }
    }

    pub const fn href(self) -> &'static str {
        match self {
            Self::Landlords => "/",
            Self::PropertyManagers => "/property-managers",
            Self::Brokers => "/brokers",
            Self::Vendors => "/vendors",
            Self::Cohosts => "/cohost-market",
            Self::None => "/",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Landlords => "Landlords",
            Self::PropertyManagers => "Property Managers",
            Self::Brokers => "Brokers",
            Self::Vendors => "Vendors",
            Self::Cohosts => "Cohost Network",
            Self::None => "",
        }
    }

    pub const fn subtitle(self) -> &'static str {
        match self {
            Self::Landlords => "Own and manage your properties",
            Self::PropertyManagers => "Manage portfolios for clients",
            Self::Brokers => "Represent buyers and sellers",
            Self::Vendors => "Get jobs from PMs and landlords",
            Self::Cohosts => "Co-manage short-term rentals",
            Self::None => "",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::Landlords => "🏠",
            Self::PropertyManagers => "🏢",
            Self::Brokers => "🤝",
            Self::Vendors => "🔧",
            Self::Cohosts => "🌐",
            Self::None => "",
        }
    }
}

impl std::fmt::Display for MarketingNavRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ── Section link ──────────────────────────────────────────────────────────────

/// Desktop / mobile in-page (or cross-page) nav link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MarketingNavSectionLink {
    pub label: &'static str,
    pub href: &'static str,
}

/// Default section links when a page does not pass `section_links`
/// (points at the landlord marketing homepage anchors).
pub const DEFAULT_MARKETING_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Features",
        href: "/#features",
    },
    MarketingNavSectionLink {
        label: "How it works",
        href: "/#app-preview",
    },
    MarketingNavSectionLink {
        label: "Pricing",
        href: "/#pricing",
    },
];

/// Section links for the landlord homepage (`/` / `/lp`) — same-page hashes.
pub const HOME_MARKETING_SECTION_LINKS: &[MarketingNavSectionLink] = &[
    MarketingNavSectionLink {
        label: "Features",
        href: "#features",
    },
    MarketingNavSectionLink {
        label: "How it works",
        href: "#app-preview",
    },
    MarketingNavSectionLink {
        label: "Pricing",
        href: "#pricing",
    },
];

/// Canonical primary CTA across all Folio marketing pages.
/// Keep this in sync with stitch `pub_marketing_*` nav buttons.
pub const DEFAULT_MARKETING_NAV_CTA: &str = "Get started";

/// Map CMS / page-specific CTA aliases onto the shared nav label so role pages
/// do not drift ("Join waitlist" vs "Join Folio" vs "Get early access").
pub fn resolve_marketing_nav_cta(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return DEFAULT_MARKETING_NAV_CTA.to_string();
    }
    let key = trimmed
        .trim_end_matches(['→', '›', '»'])
        .trim()
        .to_ascii_lowercase();
    match key.as_str() {
        "join waitlist"
        | "join the waitlist"
        | "get early access"
        | "join folio"
        | "join marketplace"
        | "get started"
        | "get started free" => DEFAULT_MARKETING_NAV_CTA.to_string(),
        _ => trimmed.to_string(),
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Shared Folio marketing top nav + mobile drawer.
#[component]
pub fn MarketingNav(
    /// Highlighted role in the "Who it's for" panel.
    #[prop(default = MarketingNavRole::None)]
    active: MarketingNavRole,
    /// Desktop / mobile section links. Defaults to [`DEFAULT_MARKETING_SECTION_LINKS`].
    #[prop(optional)]
    section_links: Option<&'static [MarketingNavSectionLink]>,
    /// Runtime CMS section links. When present, these replace `section_links`.
    #[prop(default = None)]
    section_link_overrides: Option<Vec<(String, String)>>,
    /// Primary CTA label. Defaults to [`DEFAULT_MARKETING_NAV_CTA`].
    #[prop(default = DEFAULT_MARKETING_NAV_CTA.to_string(), into)]
    cta_label: String,
    /// Primary CTA href. Defaults to `"/#waitlist-wrap"`.
    #[prop(default = "/#waitlist-wrap")]
    cta_href: &'static str,
) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let lang_res = Resource::new(|| (), |_| get_current_lang());
    let links = section_links.unwrap_or(DEFAULT_MARKETING_SECTION_LINKS);
    let nav_links = section_link_overrides
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| {
            links
                .iter()
                .map(|link| (link.label.to_string(), link.href.to_string()))
                .collect()
        });

    view! {
        <nav id="mktg-nav" class="mktg-nav">
            <div class="mktg-nav-inner">
                <a href="/" class="mktg-nav-logo" rel="external">
                    <span class="mktg-logo-mark">"F"</span>
                    "Folio"
                </a>
                <div class="mktg-nav-links">
                    {nav_links
                        .iter()
                        .cloned()
                        .map(|(label, href)| view! {
                            <a href=href>{label}</a>
                        })
                        .collect_view()}
                    <details class="mktg-nav-role-dropdown">
                        <summary aria-label="Who it's for">
                            "Who it's for"
                            <span class="mktg-nav-role-arrow">
                                <span class="material-symbols-outlined" style="font-size:15px">
                                    "expand_more"
                                </span>
                            </span>
                        </summary>
                        <div class="mktg-nav-role-panel" role="menu">
                            {MarketingNavRole::ALL
                                .into_iter()
                                .map(|role| {
                                    let class = if active == role {
                                        "mktg-nav-role-item mktg-nav-role-item--active"
                                    } else {
                                        "mktg-nav-role-item"
                                    };
                                    view! {
                                        <a href=role.href() class=class rel="external" role="menuitem">
                                            <span class="mktg-nav-role-icon" aria-hidden="true">{role.icon()}</span>
                                            <span class="mktg-nav-role-copy">
                                                <span class="mktg-nav-role-title">{role.label()}</span>
                                                <span class="mktg-nav-role-sub">{role.subtitle()}</span>
                                            </span>
                                        </a>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </details>
                </div>
                <div class="mktg-nav-actions">
                    <Suspense fallback=|| ()>
                        {move || {
                            lang_res.get().and_then(|r| r.ok()).map(|code| {
                                view! { <LanguageSwitcher current_lang=code/> }
                            })
                        }}
                    </Suspense>
                    <a href="/login" class="mktg-btn-signin" id="nav-signin-btn" rel="external">
                        <span
                            class="material-symbols-outlined"
                            style="font-size:15px;vertical-align:middle"
                        >
                            "login"
                        </span>
                        " Sign in"
                    </a>
                    <a href="/founding" class="mktg-btn-founders" id="nav-founders-btn" rel="external">
                        "Founders ✦"
                    </a>
                    <a href=cta_href class="mktg-btn-accent" id="nav-waitlist-btn">
                        {cta_label.clone()}
                    </a>
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
        <div class=move || {
            if menu_open.get() {
                "mktg-mobile-nav mktg-mobile-nav--open"
            } else {
                "mktg-mobile-nav"
            }
        }>
            {nav_links
                .iter()
                .cloned()
                .map(|(label, href)| view! {
                    <a href=href on:click=move |_| menu_open.set(false)>
                        {label}
                    </a>
                })
                .collect_view()}
            {MarketingNavRole::ALL
                .into_iter()
                .map(|role| {
                    view! {
                        <a
                            href=role.href()
                            on:click=move |_| menu_open.set(false)
                            rel="external"
                        >
                            {role.label()}
                        </a>
                    }
                })
                .collect_view()}
            <a href="/login" on:click=move |_| menu_open.set(false) rel="external">
                "Sign in"
            </a>
            <a href="/founding" on:click=move |_| menu_open.set(false) rel="external">
                "Founders ✦"
            </a>
            <a href=cta_href on:click=move |_| menu_open.set(false)>
                {cta_label.clone()}
            </a>
        </div>
    }
}
