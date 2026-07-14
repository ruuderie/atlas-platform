//! Nested building systems — `/l/assets/:id/systems`
//! Property-scoped systems list with PropertyTabBar (portfolio `/l/systems` stays).

use crate::atlas_client::{authenticated_get, session_token_from_request};
use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NestedSystemDto {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub condition: Option<String>,
}

#[server(GetNestedBuildingSystems, "/api")]
pub async fn get_nested_building_systems(
    property_id: Uuid,
) -> Result<Vec<NestedSystemDto>, ServerFnError> {
    let token = session_token_from_request().await.map_err(ServerFnError::new)?;
    authenticated_get(
        &format!("/api/folio/assets/{property_id}/systems"),
        &token,
        None,
    )
    .await
    .map_err(ServerFnError::new)
}

#[component]
pub fn PropertySystems() -> impl IntoView {
    let params = use_params_map();
    let asset_id = Memo::new(move |_| {
        params
            .get()
            .get("id")
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or(Uuid::nil())
    });

    let systems = Resource::new(
        move || asset_id.get(),
        |aid| async move {
            if aid.is_nil() {
                return Ok(Vec::new());
            }
            get_nested_building_systems(aid).await
        },
    );

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Building systems".to_string())
                subtitle=Signal::derive(|| "This property — elevators, HVAC, life safety".to_string())
            >
                <a class="folio-btn folio-btn--ghost" href=FolioRoute::LandlordSystems.path()>
                    "Portfolio systems"
                </a>
            </PageHeader>
            {move || {
                let id = asset_id.get();
                view! {
                    <PropertyTabBar
                        asset_id=id
                        active=Signal::derive(|| PropertyTab::Systems)
                    />
                }
            }}
            <Suspense fallback=move || view! { <div class="folio-empty">"Loading systems…"</div> }>
                {move || match systems.get() {
                    Some(Ok(list)) if list.is_empty() => view! {
                        <div class="folio-empty">
                            <p>"No building systems registered on this property."</p>
                            <p class="proj-section__hint">"Add systems from the portfolio Building Systems page or create a project CTA."</p>
                        </div>
                    }.into_any(),
                    Some(Ok(list)) => view! {
                        <section class="proj-section">
                            <For
                                each=move || list.clone()
                                key=|s| s.id
                                children=move |s| {
                                    view! {
                                        <div class="hub-activity-rail__row">
                                            <StatusPill label="System".to_string() tone=StatusPillTone::Info/>
                                            <div class="hub-activity-rail__body">
                                                <p class="hub-activity-rail__row-title">{s.name.clone()}</p>
                                                <p class="hub-activity-rail__row-meta">
                                                    {format!("{} · {}", s.status, s.condition.clone().unwrap_or_else(|| "—".into()))}
                                                </p>
                                            </div>
                                        </div>
                                    }
                                }
                            />
                        </section>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="folio-empty">
                            <p>{format!("Could not load systems: {e}")}</p>
                        </div>
                    }.into_any(),
                    None => view! { <div class="folio-empty">"Loading…"</div> }.into_any(),
                }}
            </Suspense>
        </div>
    }
}
