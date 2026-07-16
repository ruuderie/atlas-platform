//! Shared marketing footer — persona + access directory for public LPs only.
//!
//! Authenticated role shells must not use this. CMS footer overrides (when
//! present) replace the "This page" column; Who it's for / Get access always
//! remain so mobile users can find and share other LPs.

use leptos::prelude::*;

/// A single footer hyperlink.
#[derive(Clone, Debug)]
pub struct MarketingFooterLink {
    pub label: &'static str,
    pub href: &'static str,
}

const PERSONA_LINKS: &[MarketingFooterLink] = &[
    MarketingFooterLink {
        label: "Landlords",
        href: "/",
    },
    MarketingFooterLink {
        label: "Property Managers",
        href: "/property-managers",
    },
    MarketingFooterLink {
        label: "Brokers",
        href: "/brokers",
    },
    MarketingFooterLink {
        label: "Vendors",
        href: "/vendors",
    },
    MarketingFooterLink {
        label: "Cohosts",
        href: "/cohost-market",
    },
];

const ACCESS_LINKS: &[MarketingFooterLink] = &[
    MarketingFooterLink {
        label: "Beta",
        href: "/beta",
    },
    MarketingFooterLink {
        label: "Founding",
        href: "/founding",
    },
    MarketingFooterLink {
        label: "Refer",
        href: "/refer",
    },
];

/// Shared marketing-site footer with discoverable persona and access links.
///
/// * `tagline` — brand subtitle under the Folio wordmark.
/// * `show_page_anchors` — when true and no CMS override, show Sign in / Pricing / Features.
/// * `pricing_href` / `features_href` — page-local anchors (defaults `#pricing` / `#features`).
/// * `override_links` — CMS custom links for the third column (empty = use page anchors).
#[component]
pub fn MarketingFooter(
    #[prop(into)] tagline: String,
    #[prop(default = true)] show_page_anchors: bool,
    #[prop(optional)] pricing_href: Option<&'static str>,
    #[prop(optional)] features_href: Option<&'static str>,
    #[prop(default = Vec::new())] override_links: Vec<(String, String)>,
) -> impl IntoView {
    let pricing = pricing_href.unwrap_or("#pricing");
    let features = features_href.unwrap_or("#features");

    let page_links: Vec<(String, String)> = if !override_links.is_empty() {
        override_links
    } else if show_page_anchors {
        vec![
            ("Sign in".into(), "/login".into()),
            ("Pricing".into(), pricing.into()),
            ("Features".into(), features.into()),
        ]
    } else {
        vec![("Sign in".into(), "/login".into())]
    };
    let show_page_col = !page_links.is_empty();

    view! {
        <footer class="mktg-footer">
            <div class="mktg-footer-inner">
                <div class="mktg-footer-brand">
                    <div class="mktg-footer-logo">"Folio"</div>
                    <div class="mktg-footer-tagline">{tagline}</div>
                </div>

                <div class="mktg-footer-dirs">
                    <div class="mktg-footer-col">
                        <div class="mktg-footer-col-title">"Who it's for"</div>
                        <div class="mktg-footer-col-links">
                            {PERSONA_LINKS.iter().map(|l| view! {
                                <a href=l.href rel="external">{l.label}</a>
                            }).collect_view()}
                        </div>
                    </div>

                    <div class="mktg-footer-col">
                        <div class="mktg-footer-col-title">"Get access"</div>
                        <div class="mktg-footer-col-links">
                            {ACCESS_LINKS.iter().map(|l| view! {
                                <a href=l.href rel="external">{l.label}</a>
                            }).collect_view()}
                        </div>
                    </div>

                    <Show when=move || show_page_col>
                        <div class="mktg-footer-col">
                            <div class="mktg-footer-col-title">"This page"</div>
                            <div class="mktg-footer-col-links">
                                {page_links.clone().into_iter().map(|(label, href)| view! {
                                    <a href=href rel="external">{label}</a>
                                }).collect_view()}
                            </div>
                        </div>
                    </Show>
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
