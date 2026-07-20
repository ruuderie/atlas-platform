//! Property-scoped tab bar — Overview | Units | Systems | Portal | Documents.
//! Used by hub, unit, nested systems, documents, and portal stub pages.

use crate::components::nav::FolioRoute;
use leptos::prelude::*;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropertyTab {
    Overview,
    Units,
    Systems,
    Portal,
    Documents,
}

impl PropertyTab {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Units => "Units",
            Self::Systems => "Building Systems",
            Self::Portal => "Tenant portal",
            Self::Documents => "Documents",
        }
    }
}

#[component]
pub fn PropertyTabBar(
    asset_id: Uuid,
    #[prop(into)] active: Signal<PropertyTab>,
    /// When set, Overview/Units are in-page panel toggles (hub).
    #[prop(optional)]
    on_overview: Option<Callback<()>>,
    #[prop(optional)]
    on_units: Option<Callback<()>>,
) -> impl IntoView {
    let id = asset_id;
    let hub = FolioRoute::LandlordAssetDetail.path().replace(":id", &id.to_string());
    let systems = FolioRoute::LandlordAssetSystems.path().replace(":id", &id.to_string());
    let portal = FolioRoute::LandlordAssetPortal.path().replace(":id", &id.to_string());
    let docs = FolioRoute::LandlordAssetDocuments.path().replace(":id", &id.to_string());

    view! {
        <nav class="folio-tab-bar" aria-label="Property sections">
            {if let Some(cb) = on_overview {
                view! {
                    <button
                        type="button"
                        class=move || tab_class(active.get() == PropertyTab::Overview)
                        aria-current=move || aria_current(active.get() == PropertyTab::Overview)
                        on:click=move |_| cb.run(())
                    >
                        {PropertyTab::Overview.label()}
                    </button>
                }.into_any()
            } else {
                view! {
                    <a
                        href=hub.clone()
                        class=move || tab_class(active.get() == PropertyTab::Overview)
                        aria-current=move || aria_current(active.get() == PropertyTab::Overview)
                    >
                        {PropertyTab::Overview.label()}
                    </a>
                }.into_any()
            }}
            {if let Some(cb) = on_units {
                view! {
                    <button
                        type="button"
                        class=move || tab_class(active.get() == PropertyTab::Units)
                        aria-current=move || aria_current(active.get() == PropertyTab::Units)
                        on:click=move |_| cb.run(())
                    >
                        {PropertyTab::Units.label()}
                    </button>
                }.into_any()
            } else {
                view! {
                    <a
                        href=format!("{}?tab=units", hub)
                        class=move || tab_class(active.get() == PropertyTab::Units)
                        aria-current=move || aria_current(active.get() == PropertyTab::Units)
                    >
                        {PropertyTab::Units.label()}
                    </a>
                }.into_any()
            }}
            <a
                href=systems
                class=move || tab_class(active.get() == PropertyTab::Systems)
                aria-current=move || aria_current(active.get() == PropertyTab::Systems)
            >
                {PropertyTab::Systems.label()}
            </a>
            <a
                href=portal
                class=move || tab_class(active.get() == PropertyTab::Portal)
                aria-current=move || aria_current(active.get() == PropertyTab::Portal)
            >
                {PropertyTab::Portal.label()}
            </a>
            <a
                href=docs
                class=move || tab_class(active.get() == PropertyTab::Documents)
                aria-current=move || aria_current(active.get() == PropertyTab::Documents)
            >
                {PropertyTab::Documents.label()}
            </a>
        </nav>
    }
}

fn tab_class(active: bool) -> &'static str {
    if active {
        "folio-tab folio-tab--active"
    } else {
        "folio-tab"
    }
}

fn aria_current(active: bool) -> Option<&'static str> {
    if active {
        Some("page")
    } else {
        None
    }
}
