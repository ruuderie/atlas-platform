//! Tenant / guest portal CMS stub — `/l/assets/:id/portal`

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use uuid::Uuid;

#[component]
pub fn TenantPortalContent() -> impl IntoView {
    let params = use_params_map();
    let asset_id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or(Uuid::nil())
    });

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Tenant portal".to_string())
                subtitle=Signal::derive(|| "Content management — coming later".to_string())
            />
            {move || {
                let id = asset_id.get();
                view! {
                    <PropertyTabBar
                        asset_id=id
                        active=Signal::derive(|| PropertyTab::Portal)
                    />
                }
            }}
            <div class="folio-empty">
                <p>"Portal CMS is out of scope for this release."</p>
                <p class="proj-section__hint">"Use Documents and Activity for day-to-day ops."</p>
                <a class="folio-btn folio-btn--ghost" href=FolioRoute::LandlordAssets.path()>"Back to assets"</a>
            </div>
        </div>
    }
}
