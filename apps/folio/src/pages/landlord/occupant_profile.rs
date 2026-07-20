//! Occupant profile — `/l/leases/:lease_id/occupants/:entry_id`
//!
//! Name, relationship, contact (when available), Depart with confirm.

use crate::components::nav::FolioRoute;
use crate::components::page_header::PageHeader;
use crate::components::status_pill::{StatusPill, StatusPillTone};
use crate::pages::landlord::lease_detail::{get_lease_detail, get_lease_occupants};
use crate::pages::tenant::household::{AdultRelationship, MinorRelationship};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_navigate, use_params_map};
use uuid::Uuid;

#[cfg(feature = "ssr")]
fn extract_token(headers: &axum::http::HeaderMap) -> Option<String> {
    crate::auth::extract_bearer_token(headers)
}

#[server(DepartOccupantProfile, "/api")]
async fn depart_occupant_profile(
    lease_id: Uuid,
    entry_id: Uuid,
) -> Result<(), ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = extract_token(&headers).ok_or_else(|| ServerFnError::new("No session token"))?;
    let _: serde_json::Value = crate::atlas_client::authenticated_post(
        &format!("/api/folio/leases/{lease_id}/occupants/{entry_id}/depart"),
        &token,
        None,
        &serde_json::json!({ "reason": "moved_out" }),
    )
    .await
    .map_err(ServerFnError::new)?;
    Ok(())
}

fn relationship_label(rel: &str) -> String {
    for a in AdultRelationship::ALL {
        if a.as_str() == rel {
            return a.label().to_string();
        }
    }
    for m in MinorRelationship::ALL {
        if m.as_str() == rel {
            return m.label().to_string();
        }
    }
    rel.replace('_', " ")
}

fn initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().filter(|p| !p.is_empty()).collect();
    match parts.as_slice() {
        [] => "?".into(),
        [one] => one
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".into()),
        [a, b, ..] => {
            let ai = a.chars().next().unwrap_or('?');
            let bi = b.chars().next().unwrap_or('?');
            format!(
                "{}{}",
                ai.to_uppercase(),
                bi.to_uppercase()
            )
        }
    }
}

#[component]
pub fn OccupantProfile() -> impl IntoView {
    let params = use_params_map();
    let navigate = use_navigate();

    let lease_id = Signal::derive(move || {
        params
            .get()
            .get("lease_id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });
    let entry_id = Signal::derive(move || {
        params
            .get()
            .get("entry_id")
            .and_then(|s| Uuid::parse_str(&s).ok())
    });

    let refresh = RwSignal::new(0u32);
    let depart_pending = RwSignal::new(false);
    let depart_err = RwSignal::new(None::<String>);

    let lease = Resource::new(
        move || (lease_id.get(), refresh.get()),
        |(maybe, _)| async move {
            match maybe {
                Some(id) => get_lease_detail(id.to_string()).await.ok(),
                None => None,
            }
        },
    );

    let occupant = Resource::new(
        move || (lease_id.get(), entry_id.get(), refresh.get()),
        |(lid, eid, _)| async move {
            match (lid, eid) {
                (Some(lease), Some(entry)) => {
                    let list = get_lease_occupants(lease.to_string()).await.ok()?;
                    list.active.into_iter().find(|o| o.id == entry)
                }
                _ => None,
            }
        },
    );

    view! {
        <div class="main-area">
            <Suspense fallback=|| view! { <div class="folio-empty--compact">"Loading…"</div> }>
                {move || {
                    let Some(oid) = entry_id.get() else {
                        return view! {
                            <div class="folio-empty">
                                <p class="folio-empty__heading">"Occupant not found"</p>
                            </div>
                        }.into_any();
                    };
                    let Some(lid) = lease_id.get() else {
                        return view! {
                            <div class="folio-empty">
                                <p class="folio-empty__heading">"Lease not found"</p>
                            </div>
                        }.into_any();
                    };
                    match occupant.get() {
                        None => view! { <div class="folio-empty--compact">"Loading…"</div> }.into_any(),
                        Some(None) => view! {
                            <div class="folio-empty">
                                <p class="folio-empty__heading">"Occupant not found"</p>
                                <p class="folio-empty__sub">"They may have already departed this lease."</p>
                                <a
                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                    href=FolioRoute::LandlordLeaseDetail.path().replace(":id", &lid.to_string())
                                >
                                    "Back to lease"
                                </a>
                            </div>
                        }.into_any(),
                        Some(Some(o)) => {
                            let name = o.full_name.clone();
                            let rel = relationship_label(&o.relationship);
                            let kind_label = if o.is_minor { "Minor" } else { "Adult" };
                            let avatar = initials(&o.full_name);
                            let lease_href = StoredValue::new(
                                FolioRoute::LandlordLeaseDetail
                                    .path()
                                    .replace(":id", &lid.to_string()),
                            );
                            let name_for_confirm = name.clone();
                            let nav = navigate.clone();
                            view! {
                                <PageHeader
                                    title=Signal::derive({
                                        let name = name.clone();
                                        move || name.clone()
                                    })
                                    subtitle=Signal::derive({
                                        let rel = rel.clone();
                                        let kind_label = kind_label.to_string();
                                        move || format!("{kind_label} · {rel}")
                                    })
                                >
                                    <a
                                        class="folio-btn folio-btn--ghost folio-btn--sm press"
                                        href=lease_href.get_value()
                                    >
                                        "Open lease"
                                    </a>
                                </PageHeader>

                                <div class="folio-detail-split">
                                  <div class="unit-panel" style="display:flex;flex-direction:column;gap:1.25rem;">
                                    <section class="proj-section">
                                        <div style="display:flex;align-items:center;gap:1rem;">
                                            <div class="ld-occupant-avatar" style="width:3.5rem;height:3.5rem;font-size:1rem;">
                                                {avatar}
                                            </div>
                                            <div>
                                                <p class="hub-activity-rail__row-title">{name.clone()}</p>
                                                <div style="display:flex;gap:0.5rem;margin-top:0.35rem;flex-wrap:wrap;">
                                                    <StatusPill
                                                        label=rel.clone()
                                                        tone=StatusPillTone::Info
                                                    />
                                                    <StatusPill
                                                        label=kind_label.to_string()
                                                        tone=if o.is_minor { StatusPillTone::Warn } else { StatusPillTone::Neutral }
                                                    />
                                                </div>
                                            </div>
                                        </div>
                                    </section>

                                    <section class="proj-section">
                                        <h3 class="proj-section__title">"Relationship"</h3>
                                        <p class="hub-activity-rail__row-meta">{rel}</p>
                                        {o.date_of_birth.as_ref().map(|d| view! {
                                            <p class="hub-activity-rail__row-meta" style="margin-top:0.35rem;">
                                                "Date of birth · " {d.clone()}
                                            </p>
                                        })}
                                    </section>

                                    <section class="proj-section">
                                        <h3 class="proj-section__title">"Contact"</h3>
                                        <p class="hub-activity-rail__row-meta">
                                            "Not available — message the primary tenant from the unit."
                                        </p>
                                    </section>

                                    <section class="proj-section" style="border-color:rgba(185,28,28,0.35);">
                                        <h3 class="proj-section__title">"Depart"</h3>
                                        <p class="proj-section__hint">
                                            "Ends this person’s occupancy on the lease. Confirm before saving."
                                        </p>
                                        {move || depart_err.get().map(|e| view! {
                                            <p style="color:#b91c1c;font-size:0.875rem;">{e}</p>
                                        })}
                                        <button
                                            type="button"
                                            class="folio-btn folio-btn--ghost folio-btn--sm press"
                                            style="margin-top:0.75rem;border-color:#fecaca;color:#991b1b;"
                                            disabled=move || depart_pending.get()
                                            on:click=move |_| {
                                                let msg = format!(
                                                    "Depart {} from this lease?",
                                                    name_for_confirm
                                                );
                                                let confirmed = web_sys::window()
                                                    .and_then(|w| w.confirm_with_message(&msg).ok())
                                                    .unwrap_or(false);
                                                if !confirmed {
                                                    return;
                                                }
                                                depart_pending.set(true);
                                                depart_err.set(None);
                                                let nav = nav.clone();
                                                spawn_local(async move {
                                                    match depart_occupant_profile(lid, oid).await {
                                                        Ok(()) => {
                                                            nav(
                                                                &FolioRoute::LandlordLeaseDetail
                                                                    .path()
                                                                    .replace(":id", &lid.to_string()),
                                                                Default::default(),
                                                            );
                                                        }
                                                        Err(e) => {
                                                            depart_err.set(Some(e.to_string()));
                                                            depart_pending.set(false);
                                                        }
                                                    }
                                                });
                                            }
                                        >
                                            {move || if depart_pending.get() { "Departing…" } else { "Depart occupant" }}
                                        </button>
                                    </section>
                                  </div>

                                  <aside class="folio-detail-split__aside unit-panel">
                                    {move || lease.get().flatten().map(|l| {
                                        let rent = l
                                            .recurring_amount_cents
                                            .map(|c| format!("${:.0}/mo", c as f64 / 100.0))
                                            .unwrap_or_else(|| "—".into());
                                        let status = l.status.clone();
                                        let href = lease_href.get_value();
                                        view! {
                                            <section class="proj-section">
                                                <h3 class="proj-section__title">"Lease context"</h3>
                                                <p class="hub-activity-rail__row-meta">
                                                    {format!("{status} · {rent}")}
                                                </p>
                                                <a
                                                    class="folio-btn folio-btn--ghost folio-btn--sm press"
                                                    style="margin-top:0.75rem;"
                                                    href=href
                                                >
                                                    "Open lease"
                                                </a>
                                            </section>
                                        }
                                    })}
                                  </aside>
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </Suspense>
        </div>
    }
}
