// apps/folio/src/pages/tenant/household.rs
//
// Tenant Household — /t/household
//
// Lets tenants view and manage household members (occupants) and registered
// vehicles on their active lease. All data is lease-scoped via:
//
//   GET  /api/folio/leases                           — resolve active lease ID
//   GET  /api/folio/leases/{id}/occupants            — active + former occupants
//   POST /api/folio/leases/{id}/occupants            — add a household member
//   GET  /api/folio/leases/{id}/vehicles             — registered vehicles
//   POST /api/folio/leases/{id}/vehicles             — add a vehicle
//
// UI:  Two tabs — "People" (occupants) and "Vehicles"
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── API types (mirror backend shapes) ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseSummary {
    pub id: Uuid,
    pub status: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub monthly_rent_cents: Option<i64>,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveOccupant {
    pub id: Uuid,
    pub full_name: String,
    pub relationship: String,
    pub is_minor: bool,
    pub date_of_birth: Option<String>,
    pub id_document_type: Option<String>,
    pub registered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormerOccupant {
    pub id: Uuid,
    pub full_name: String,
    pub relationship: String,
    pub is_minor: bool,
    pub removed_at: String,
    pub removal_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OccupantList {
    pub active: Vec<ActiveOccupant>,
    pub former: Vec<FormerOccupant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleRecord {
    pub id: Uuid,
    pub make: String,
    pub model: String,
    pub year: i32,
    pub color: String,
    pub license_plate: String,
    pub state: String,
    pub country: String,
    pub parking_spot: Option<String>,
    pub registration_expiry: Option<String>,
    pub registered_at: String,
}

// ── Server functions ──────────────────────────────────────────────────────────

/// Fetch the caller's active lease (first active lease found).
#[server(HhFetchLease, "/api")]
pub async fn hh_fetch_lease() -> Result<Option<LeaseSummary>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let leases = crate::atlas_client::authenticated_get::<Vec<LeaseSummary>>(
        "/api/folio/leases",
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(leases
        .into_iter()
        .find(|l| l.status.to_lowercase() == "active"))
}

/// Fetch occupants (active + former) for a lease.
#[server(HhFetchOccupants, "/api")]
pub async fn hh_fetch_occupants(
    lease_id: Uuid,
) -> Result<OccupantList, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<OccupantList>(
        &format!("/api/folio/leases/{lease_id}/occupants"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

/// Fetch registered vehicles for a lease.
#[server(HhFetchVehicles, "/api")]
pub async fn hh_fetch_vehicles(
    lease_id: Uuid,
) -> Result<Vec<VehicleRecord>, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    crate::atlas_client::authenticated_get::<Vec<VehicleRecord>>(
        &format!("/api/folio/leases/{lease_id}/vehicles"),
        &token,
        None,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))
}

/// Register a new occupant on a lease.
#[server(HhAddOccupant, "/api")]
pub async fn hh_add_occupant(
    lease_id: Uuid,
    full_name: String,
    relationship: String,
    is_minor: bool,
    dob: Option<String>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;

    let body = if is_minor {
        serde_json::json!({
            "type": "minor",
            "full_name": full_name,
            "relationship": relationship,
            "date_of_birth": dob
        })
    } else {
        serde_json::json!({
            "type": "adult",
            "full_name": full_name,
            "relationship": relationship
        })
    };

    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &format!("/api/folio/leases/{lease_id}/occupants"),
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Register a new vehicle on a lease.
#[server(HhAddVehicle, "/api")]
pub async fn hh_add_vehicle(
    lease_id: Uuid,
    make: String,
    model: String,
    year: i32,
    color: String,
    plate: String,
    state: String,
    parking: Option<String>,
) -> Result<(), server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;
    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = session_token(&headers)?;
    let body = serde_json::json!({
        "make": make, "model": model, "year": year,
        "color": color,
        "license_plate": { "number": plate, "state": state, "country": "US" },
        "parking_spot": parking
    });
    crate::atlas_client::authenticated_post::<serde_json::Value, serde_json::Value>(
        &format!("/api/folio/leases/{lease_id}/vehicles"),
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn session_token(
    headers: &axum::http::HeaderMap,
) -> Result<String, server_fn::error::ServerFnError> {
    headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            s.split(';').find_map(|p| {
                let p = p.trim();
                p.strip_prefix("session=").map(|t| t.to_string())
            })
        })
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))
}

fn relationship_icon(rel: &str, is_minor: bool) -> &'static str {
    if is_minor {
        return "👶";
    }
    match rel.to_lowercase().as_str() {
        r if r.contains("spouse") || r.contains("partner") => "💑",
        r if r.contains("parent") => "👨‍👩‍👦",
        r if r.contains("sibling") => "👫",
        r if r.contains("room") => "🤝",
        _ => "🧑",
    }
}

fn cents_to_display(cents: i64, currency: &str) -> String {
    let symbol = match currency.to_uppercase().as_str() {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "CAD" => "CA$",
        _ => currency,
    };
    format!("{}{:.2}", symbol, cents as f64 / 100.0)
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantHousehold() -> impl IntoView {
    let refresh = RwSignal::new(0u32);
    let active_tab = RwSignal::new("people"); // "people" | "vehicles"

    // ── Lease ──────────────────────────────────────────────────────────────
    let lease_res = Resource::new(move || refresh.get(), |_| hh_fetch_lease());

    // Derived lease ID signal (used by occupant + vehicle resources)
    let lease_id =
        Signal::derive(move || lease_res.get().and_then(|r| r.ok()).flatten().map(|l| l.id));

    // ── Occupants ──────────────────────────────────────────────────────────
    let occupants_res = Resource::new(
        move || (lease_id.get(), refresh.get()),
        |(lid, _)| async move {
            match lid {
                Some(id) => hh_fetch_occupants(id).await.ok(),
                None => None,
            }
        },
    );

    // ── Vehicles ───────────────────────────────────────────────────────────
    let vehicles_res = Resource::new(
        move || (lease_id.get(), refresh.get()),
        |(lid, _)| async move {
            match lid {
                Some(id) => hh_fetch_vehicles(id).await.ok(),
                None => None,
            }
        },
    );

    // ── Add occupant modal state ────────────────────────────────────────────
    let show_add_person = RwSignal::new(false);
    let new_person_name = RwSignal::new(String::new());
    let new_person_rel = RwSignal::new("Adult — Other".to_string());
    let new_person_minor = RwSignal::new(false);
    let new_person_dob = RwSignal::new(String::new());
    let adding_person = RwSignal::new(false);

    // ── Add vehicle modal state ─────────────────────────────────────────────
    let show_add_vehicle = RwSignal::new(false);
    let new_veh_make = RwSignal::new(String::new());
    let new_veh_model = RwSignal::new(String::new());
    let new_veh_year = RwSignal::new("2020".to_string());
    let new_veh_color = RwSignal::new(String::new());
    let new_veh_plate = RwSignal::new(String::new());
    let new_veh_state = RwSignal::new(String::new());
    let new_veh_parking = RwSignal::new(String::new());
    let adding_vehicle = RwSignal::new(false);

    // ── Handlers ───────────────────────────────────────────────────────────
    let handle_add_person = move |_| {
        let Some(lid) = lease_id.get() else {
            return;
        };
        let name = new_person_name.get();
        if name.trim().is_empty() {
            return;
        }
        let rel = new_person_rel.get();
        let minor = new_person_minor.get();
        let dob_str = new_person_dob.get();
        let dob = if minor && !dob_str.is_empty() {
            Some(dob_str)
        } else {
            None
        };
        adding_person.set(true);
        spawn_local(async move {
            if hh_add_occupant(lid, name, rel, minor, dob).await.is_ok() {
                show_add_person.set(false);
                new_person_name.set(String::new());
                refresh.update(|n| *n += 1);
            }
            adding_person.set(false);
        });
    };

    let handle_add_vehicle = move |_| {
        let Some(lid) = lease_id.get() else {
            return;
        };
        let make = new_veh_make.get();
        let model = new_veh_model.get();
        let plate = new_veh_plate.get();
        if make.trim().is_empty() || model.trim().is_empty() || plate.trim().is_empty() {
            return;
        }
        let year = new_veh_year.get().parse::<i32>().unwrap_or(2020);
        let color = new_veh_color.get();
        let state = new_veh_state.get();
        let parking = {
            let p = new_veh_parking.get();
            if p.trim().is_empty() {
                None
            } else {
                Some(p)
            }
        };
        adding_vehicle.set(true);
        spawn_local(async move {
            if hh_add_vehicle(lid, make, model, year, color, plate, state, parking)
                .await
                .is_ok()
            {
                show_add_vehicle.set(false);
                new_veh_make.set(String::new());
                new_veh_model.set(String::new());
                new_veh_plate.set(String::new());
                refresh.update(|n| *n += 1);
            }
            adding_vehicle.set(false);
        });
    };

    view! {
        <div class="main-area">

            // ── Header ──
            <div class="page-header">
                <div>
                    <h1 class="page-title">"Household"</h1>
                    <p class="page-subtitle">"Manage occupants and registered vehicles on your lease"</p>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        "↻ Refresh"
                    </button>
                </div>
            </div>

            // ── Lease summary card ──
            <Suspense fallback=|| ()>
                {move || lease_res.get().map(|res| {
                    match res {
                        Ok(Some(lease)) => {
                            let rent = lease.monthly_rent_cents.map(|c| cents_to_display(c, &lease.currency)).unwrap_or_else(|| "—".to_string());
                            let start = lease.start_date.clone().unwrap_or_else(|| "—".to_string());
                            let end   = lease.end_date.clone().unwrap_or_else(|| "Month-to-month".to_string());
                            view! {
                                <div class="hh-lease-card">
                                    <div class="hh-lease-field">
                                        <span class="hh-lease-label">"Status"</span>
                                        <span class="hh-lease-value hh-lease-status">{lease.status.clone()}</span>
                                    </div>
                                    <div class="hh-lease-field">
                                        <span class="hh-lease-label">"Monthly Rent"</span>
                                        <span class="hh-lease-value">{rent}</span>
                                    </div>
                                    <div class="hh-lease-field">
                                        <span class="hh-lease-label">"Start Date"</span>
                                        <span class="hh-lease-value">{start}</span>
                                    </div>
                                    <div class="hh-lease-field">
                                        <span class="hh-lease-label">"End / Term"</span>
                                        <span class="hh-lease-value">{end}</span>
                                    </div>
                                </div>
                            }.into_any()
                        }
                        Ok(None) => view! {
                            <div class="hh-no-lease">
                                "No active lease found. Contact your property manager."
                            </div>
                        }.into_any(),
                        Err(_) => view! {
                            <div class="hh-no-lease text-red-400">"Could not load lease information."</div>
                        }.into_any(),
                    }
                })}
            </Suspense>

            // ── Tabs ──
            <div class="hh-tabs">
                <button
                    class=move || format!("hh-tab {}", if active_tab.get() == "people" { "hh-tab--active" } else { "" })
                    on:click=move |_| active_tab.set("people")
                >
                    "👥 People"
                </button>
                <button
                    class=move || format!("hh-tab {}", if active_tab.get() == "vehicles" { "hh-tab--active" } else { "" })
                    on:click=move |_| active_tab.set("vehicles")
                >
                    "🚗 Vehicles"
                </button>
            </div>

            // ── People Tab ──
            <Show when=move || active_tab.get() == "people">
                <div class="hh-section">
                    <div class="hh-section-header">
                        <span class="hh-section-title">"Household Members"</span>
                        <Show when=move || lease_id.get().is_some()>
                            <button class="btn btn-primary btn-sm" on:click=move |_| show_add_person.set(true)>
                                "+ Add Person"
                            </button>
                        </Show>
                    </div>

                    <Suspense fallback=|| view! { <div class="hh-loading">"Loading household…"</div> }>
                        {move || occupants_res.get().map(|opt| {
                            match opt {
                                Some(list) => {
                                    view! {
                                        // Active occupants
                                        {if list.active.is_empty() {
                                            view! {
                                                <div class="hh-empty">"No household members registered yet."</div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="hh-card-grid">
                                                    <For
                                                        each=move || list.active.clone()
                                                        key=|o| o.id
                                                        children=move |occ| {
                                                            let icon = relationship_icon(&occ.relationship, occ.is_minor);
                                                            let rel  = occ.relationship.clone();
                                                            let name = occ.full_name.clone();
                                                            let dob  = occ.date_of_birth.clone();
                                                            let doc  = occ.id_document_type.clone().unwrap_or_else(|| "—".to_string());
                                                            let date = occ.registered_at.chars().take(10).collect::<String>();
                                                            view! {
                                                                <div class="hh-member-card">
                                                                    <div class="hh-member-avatar">{icon}</div>
                                                                    <div class="hh-member-info">
                                                                        <div class="hh-member-name">{name}</div>
                                                                        <div class="hh-member-rel">{rel}</div>
                                                                        {dob.map(|d| view! {
                                                                            <div class="hh-member-meta">"DOB: " {d}</div>
                                                                        })}
                                                                        <div class="hh-member-meta">"ID: " {doc}</div>
                                                                        <div class="hh-member-meta hh-member-date">"Registered " {date}</div>
                                                                    </div>
                                                                    <span class="hh-badge hh-badge--active">"Active"</span>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }.into_any()
                                        }}

                                        // Former occupants (collapsed section)
                                        {if !list.former.is_empty() {
                                            view! {
                                                <details class="hh-former-section">
                                                    <summary class="hh-former-summary">
                                                        "Former occupants (" {list.former.len().to_string()} ")"
                                                    </summary>
                                                    <div class="hh-card-grid mt-3">
                                                        <For
                                                            each=move || list.former.clone()
                                                            key=|o| o.id
                                                            children=move |occ| {
                                                                let name = occ.full_name.clone();
                                                                let rel  = occ.relationship.clone();
                                                                let left = occ.removed_at.chars().take(10).collect::<String>();
                                                                view! {
                                                                    <div class="hh-member-card hh-member-card--former">
                                                                        <div class="hh-member-avatar opacity-40">"🚪"</div>
                                                                        <div class="hh-member-info">
                                                                            <div class="hh-member-name opacity-70">{name}</div>
                                                                            <div class="hh-member-rel">{rel}</div>
                                                                            <div class="hh-member-meta">"Departed " {left}</div>
                                                                        </div>
                                                                        <span class="hh-badge hh-badge--former">"Departed"</span>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                </details>
                                            }.into_any()
                                        } else { ().into_any() }}
                                    }.into_any()
                                }
                                None => view! {
                                    <div class="hh-empty">"Select an active lease to view household members."</div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>
            </Show>

            // ── Vehicles Tab ──
            <Show when=move || active_tab.get() == "vehicles">
                <div class="hh-section">
                    <div class="hh-section-header">
                        <span class="hh-section-title">"Registered Vehicles"</span>
                        <Show when=move || lease_id.get().is_some()>
                            <button class="btn btn-primary btn-sm" on:click=move |_| show_add_vehicle.set(true)>
                                "+ Add Vehicle"
                            </button>
                        </Show>
                    </div>

                    <Suspense fallback=|| view! { <div class="hh-loading">"Loading vehicles…"</div> }>
                        {move || vehicles_res.get().map(|opt| {
                            match opt {
                                Some(vehicles) if !vehicles.is_empty() => view! {
                                    <div class="hh-card-grid">
                                        <For
                                            each=move || vehicles.clone()
                                            key=|v| v.id
                                            children=move |veh| {
                                                let make    = veh.make.clone();
                                                let model   = veh.model.clone();
                                                let year    = veh.year;
                                                let color   = veh.color.clone();
                                                let plate   = veh.license_plate.clone();
                                                let state   = veh.state.clone();
                                                let parking = veh.parking_spot.clone().unwrap_or_else(|| "Not assigned".to_string());
                                                let expiry  = veh.registration_expiry.clone().unwrap_or_else(|| "—".to_string());
                                                view! {
                                                    <div class="hh-vehicle-card">
                                                        <div class="hh-vehicle-icon">"🚗"</div>
                                                        <div class="hh-vehicle-info">
                                                            <div class="hh-vehicle-name">
                                                                {year.to_string()} " " {make} " " {model}
                                                            </div>
                                                            <div class="hh-vehicle-plate">{plate} " · " {state}</div>
                                                            <div class="hh-vehicle-meta">"Color: " {color}</div>
                                                            <div class="hh-vehicle-meta">"Parking: " {parking}</div>
                                                            <div class="hh-vehicle-meta">"Reg. Expiry: " {expiry}</div>
                                                        </div>
                                                        <span class="hh-badge hh-badge--active">"Registered"</span>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }.into_any(),
                                _ => view! {
                                    <div class="hh-empty">"No vehicles registered on this lease."</div>
                                }.into_any(),
                            }
                        })}
                    </Suspense>
                </div>
            </Show>

            // ── Add Person Modal ─────────────────────────────────────────────
            <Show when=move || show_add_person.get()>
                <div class="modal-backdrop">
                    <div class="modal-card">
                        <div class="modal-header">
                            <h3 class="modal-title">"Add Household Member"</h3>
                            <button class="modal-close" on:click=move |_| show_add_person.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body space-y-4">
                            <div class="form-field">
                                <label class="form-label">"Full Name *"</label>
                                <input
                                    type="text"
                                    class="form-input"
                                    placeholder="Jane Doe"
                                    prop:value=new_person_name
                                    on:input=move |ev| new_person_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Relationship"</label>
                                <select
                                    class="form-select"
                                    on:change=move |ev| new_person_rel.set(event_target_value(&ev))
                                >
                                    <option>"Adult — Other"</option>
                                    <option>"Spouse / Partner"</option>
                                    <option>"Parent"</option>
                                    <option>"Sibling"</option>
                                    <option>"Roommate"</option>
                                    <option>"Minor Child"</option>
                                    <option>"Minor — Other"</option>
                                </select>
                            </div>
                            <div class="form-field">
                                <label class="form-label flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        class="form-checkbox"
                                        prop:checked=new_person_minor
                                        on:change=move |ev| {
                                            new_person_minor.set(event_target_checked(&ev));
                                        }
                                    />
                                    "This is a minor (under 18)"
                                </label>
                            </div>
                            <Show when=move || new_person_minor.get()>
                                <div class="form-field">
                                    <label class="form-label">"Date of Birth"</label>
                                    <input
                                        type="date"
                                        class="form-input"
                                        prop:value=new_person_dob
                                        on:input=move |ev| new_person_dob.set(event_target_value(&ev))
                                    />
                                </div>
                            </Show>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_person.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || adding_person.get() || new_person_name.get().trim().is_empty()
                                on:click=handle_add_person
                            >
                                {move || if adding_person.get() { "Adding…" } else { "Add Member" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // ── Add Vehicle Modal ────────────────────────────────────────────
            <Show when=move || show_add_vehicle.get()>
                <div class="modal-backdrop">
                    <div class="modal-card">
                        <div class="modal-header">
                            <h3 class="modal-title">"Register a Vehicle"</h3>
                            <button class="modal-close" on:click=move |_| show_add_vehicle.set(false)>"✕"</button>
                        </div>
                        <div class="modal-body" style="display:grid;grid-template-columns:1fr 1fr;gap:1rem;">
                            <div class="form-field">
                                <label class="form-label">"Make *"</label>
                                <input type="text" class="form-input" placeholder="Toyota"
                                    prop:value=new_veh_make
                                    on:input=move |ev| new_veh_make.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Model *"</label>
                                <input type="text" class="form-input" placeholder="Camry"
                                    prop:value=new_veh_model
                                    on:input=move |ev| new_veh_model.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Year"</label>
                                <input type="number" class="form-input" placeholder="2020"
                                    prop:value=new_veh_year
                                    on:input=move |ev| new_veh_year.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Color"</label>
                                <input type="text" class="form-input" placeholder="Silver"
                                    prop:value=new_veh_color
                                    on:input=move |ev| new_veh_color.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"License Plate *"</label>
                                <input type="text" class="form-input" placeholder="ABC-1234"
                                    prop:value=new_veh_plate
                                    on:input=move |ev| new_veh_plate.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"State / Province"</label>
                                <input type="text" class="form-input" placeholder="CA"
                                    prop:value=new_veh_state
                                    on:input=move |ev| new_veh_state.set(event_target_value(&ev)) />
                            </div>
                            <div class="form-field" style="grid-column:span 2">
                                <label class="form-label">"Parking Spot (optional)"</label>
                                <input type="text" class="form-input" placeholder="Spot A-12"
                                    prop:value=new_veh_parking
                                    on:input=move |ev| new_veh_parking.set(event_target_value(&ev)) />
                            </div>
                        </div>
                        <div class="modal-footer">
                            <button class="btn btn-ghost" on:click=move |_| show_add_vehicle.set(false)>"Cancel"</button>
                            <button
                                class="btn btn-primary"
                                disabled=move || adding_vehicle.get()
                                on:click=handle_add_vehicle
                            >
                                {move || if adding_vehicle.get() { "Registering…" } else { "Register Vehicle" }}
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

        </div>
    }
}
