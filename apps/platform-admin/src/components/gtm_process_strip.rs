//! Shared six-stage Go-to-Market process strip.

use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GtmStage {
    Products,
    LandingPages,
    Campaigns,
    Programs,
    Syndication,
    Verification,
}

impl GtmStage {
    pub fn id(self) -> &'static str {
        match self {
            Self::Products => "products",
            Self::LandingPages => "landing_pages",
            Self::Campaigns => "campaigns",
            Self::Programs => "programs",
            Self::Syndication => "syndication",
            Self::Verification => "verification",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Products => "Products",
            Self::LandingPages => "Landing Pages",
            Self::Campaigns => "Campaigns",
            Self::Programs => "Programs",
            Self::Syndication => "Syndication",
            Self::Verification => "Verification",
        }
    }

    pub fn href(self) -> &'static str {
        match self {
            Self::Products => "/products",
            Self::LandingPages => "/landing-pages",
            Self::Campaigns => "/campaigns",
            Self::Programs => "/programs",
            Self::Syndication => "/syndication/offers",
            Self::Verification => "/verification",
        }
    }

    pub fn all() -> [GtmStage; 6] {
        [
            Self::Products,
            Self::LandingPages,
            Self::Campaigns,
            Self::Programs,
            Self::Syndication,
            Self::Verification,
        ]
    }
}

#[component]
pub fn GtmProcessStrip(
    active: GtmStage,
    #[prop(into, optional)] subtitle: Option<String>,
) -> impl IntoView {
    view! {
        <nav class="gtm-process" aria-label="Go-to-Market stages">
            {GtmStage::all().into_iter().enumerate().map(|(i, stage)| {
                let is_active = stage == active;
                let href = stage.href();
                let label = stage.label();
                view! {
                    {(i > 0).then(|| view! { <span class="gtm-step-sep" aria-hidden="true"></span> })}
                    <a
                        class=if is_active { "gtm-step active" } else { "gtm-step" }
                        href=href
                    >
                        <span class="gtm-step-num">{(i + 1).to_string()}</span>
                        <span class="gtm-step-label">{label}</span>
                    </a>
                }
            }).collect_view()}
            {subtitle.map(|s| view! { <p class="gtm-process-sub">{s}</p> })}
        </nav>
    }
}
