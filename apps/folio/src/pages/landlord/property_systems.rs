//! Nested building systems — `/l/assets/:id/systems`
//! Property-scoped systems list with PropertyTabBar (portfolio `/l/systems` stays).

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::property_tab_bar::{PropertyTab, PropertyTabBar};
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::building_systems::{create_building_system, BuildingSystemTypeOpt};
use leptos::prelude::*;
use leptos::task::spawn_local;
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

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(GetNestedBuildingSystems, "/api")]
pub async fn get_nested_building_systems(
    property_id: Uuid,
) -> Result<Vec<NestedSystemDto>, ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers)
        .ok_or_else(|| ServerFnError::new("No session token"))?;
    crate::atlas_client::authenticated_get(
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

    let refresh = RwSignal::new(0u32);
    let show_add = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let new_type = RwSignal::new(BuildingSystemTypeOpt::Elevator.as_str().to_string());
    let creating = RwSignal::new(false);
    let create_err = RwSignal::new(None::<String>);

    let systems = Resource::new(
        move || (asset_id.get(), refresh.get()),
        |(aid, _)| async move {
            if aid.is_nil() {
                return Ok(Vec::new());
            }
            get_nested_building_systems(aid).await
        },
    );

    let on_create = move |_| {
        let pid = asset_id.get();
        if pid.is_nil() {
            create_err.set(Some("Invalid property.".into()));
            return;
        }
        let name = new_name.get().trim().to_string();
        let system_type = new_type.get();
        if name.is_empty() {
            create_err.set(Some("Name is required.".into()));
            return;
        }
        creating.set(true);
        create_err.set(None);
        spawn_local(async move {
            match create_building_system(pid.to_string(), name, system_type).await {
                Ok(_) => {
                    show_add.set(false);
                    new_name.set(String::new());
                    refresh.update(|n| *n += 1);
                }
                Err(e) => create_err.set(Some(e.to_string())),
            }
            creating.set(false);
        });
    };

    view! {
        <div class="landlord-list-page">
            <PageHeader
                title=Signal::derive(|| "Building systems".to_string())
                subtitle=Signal::derive(|| "This property — elevators, HVAC, life safety".to_string())
            >
                <a class="folio-btn folio-btn--ghost" href=FolioRoute::LandlordSystems.path()>
                    "Portfolio systems"
                </a>
                <button
                    type="button"
                    class="folio-btn folio-btn--primary press"
                    on:click=move |_| {
                        create_err.set(None);
                        show_add.set(true);
                    }
                >
                    "+ Add System"
                </button>
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
                            <p>"No building systems on this property yet."</p>
                            <p class="proj-section__hint">
                                "Register elevators, HVAC, roof, and life-safety gear here — you do not need the portfolio Systems page."
                            </p>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary press"
                                style="margin-top:1rem;"
                                on:click=move |_| {
                                    create_err.set(None);
                                    show_add.set(true);
                                }
                            >
                                "Add system"
                            </button>
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

            <Show when=move || show_add.get()>
                <div class="modal-backdrop">
                    <div class="modal-card" style="max-width:28rem;">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add Building System"</h3>
                            <button type="button" class="modal-close" on:click=move |_| show_add.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Name *"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="Main Elevator"
                                    prop:value=new_name
                                    on:input=move |ev| new_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"System Type *"</label>
                                <select class="form-select" on:change=move |ev| new_type.set(event_target_value(&ev))>
                                    {BuildingSystemTypeOpt::ALL.iter().copied().map(|t| {
                                        view! { <option value=t.as_str()>{t.label()}</option> }
                                    }).collect_view()}
                                </select>
                            </div>
                            {move || create_err.get().map(|e| view! {
                                <p class="folio-empty__sub" style="color:#b91c1c;">{e}</p>
                            })}
                        </div>
                        <div class="modal-footer">
                            <button type="button" class="folio-btn folio-btn--ghost" on:click=move |_| show_add.set(false)>"Cancel"</button>
                            <button
                                type="button"
                                class="folio-btn folio-btn--primary"
                                disabled=move || creating.get() || new_name.get().trim().is_empty()
                                on:click=on_create
                            >
                                {move || if creating.get() { "Saving…" } else { "Add System" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
