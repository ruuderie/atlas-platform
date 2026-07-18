// apps/folio/src/pages/tenant/maintenance_triage.rs
//
// Tenant Maintenance Triage — /t/maintenance/new
//
// 3-step wizard: 1) Category + urgency, 2) Details + description,
// 3) Confirm + submit. POSTs to /api/folio/maintenance.
// ─────────────────────────────────────────────────────────────────────────────

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::get_session;
use crate::pages::landlord::leases::{list_leases, LeaseStatus};

// ── Server function ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub category: String,
    pub urgency: String,
    pub subject: String,
    pub description: String,
    pub unit_access: String,
}

#[derive(Serialize)]
struct CreateTicketBody {
    asset_id: Uuid,
    reported_by_user_id: Uuid,
    category: String,
    description: String,
    is_emergency: bool,
    voice_note_r2_key: Option<String>,
}

#[derive(Deserialize)]
struct IdResp {
    id: Uuid,
}

fn map_maintenance_category(raw: &str) -> &'static str {
    match raw {
        "plumbing" => "plumbing",
        "electrical" => "electrical",
        "hvac" => "hvac",
        "appliance" => "appliance",
        "structural" => "structural",
        "pest" => "pest",
        "roofing" => "roofing",
        // UI-only labels collapse to API `general`
        "locksmith" | "other" | _ => "general",
    }
}

#[server(SubmitMaintenanceRequest, "/api")]
pub async fn submit_maintenance_request(
    req: MaintenanceRequest,
) -> Result<String, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;
    let session = get_session().await?;

    let leases = list_leases().await?;
    let asset_id = leases
        .iter()
        .find(|l| LeaseStatus::from_str(&l.status) == LeaseStatus::Active)
        .and_then(|l| l.asset_id)
        .or_else(|| leases.iter().find_map(|l| l.asset_id))
        .ok_or_else(|| {
            server_fn::error::ServerFnError::new(
                "No unit on your lease — contact your landlord before requesting maintenance.",
            )
        })?;

    let category = map_maintenance_category(&req.category).to_string();
    let is_emergency = req.urgency == "emergency";
    let description = {
        let mut parts = vec![req.subject.clone()];
        if !req.description.trim().is_empty() {
            parts.push(req.description.clone());
        }
        parts.push(format!("Unit access: {}", req.unit_access.replace('_', " ")));
        parts.push(format!("Urgency: {}", req.urgency));
        parts.join("\n\n")
    };

    let body = CreateTicketBody {
        asset_id,
        reported_by_user_id: session.user_id,
        category,
        description,
        is_emergency,
        voice_note_r2_key: None,
    };
    let resp = crate::atlas_client::authenticated_post::<CreateTicketBody, IdResp>(
        "/api/folio/maintenance",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(resp.id.to_string())
}

// ── Category data ─────────────────────────────────────────────────────────────

struct Category {
    id: &'static str,
    icon: &'static str,
    label: &'static str,
}
fn categories() -> Vec<Category> {
    vec![
        Category {
            id: "plumbing",
            icon: "🚰",
            label: "Plumbing",
        },
        Category {
            id: "electrical",
            icon: "⚡",
            label: "Electrical",
        },
        Category {
            id: "hvac",
            icon: "❄️",
            label: "HVAC / Heating",
        },
        Category {
            id: "appliance",
            icon: "🫙",
            label: "Appliance",
        },
        Category {
            id: "structural",
            icon: "🏗",
            label: "Structural",
        },
        Category {
            id: "pest",
            icon: "🐛",
            label: "Pest Control",
        },
        Category {
            id: "locksmith",
            icon: "🔑",
            label: "Locksmith",
        },
        Category {
            id: "other",
            icon: "🔧",
            label: "Other",
        },
    ]
}

// ── Component ─────────────────────────────────────────────────────────────────

#[component]
pub fn TenantMaintenanceTriage() -> impl IntoView {
    let step = RwSignal::new(1u8);
    let category = RwSignal::new(String::new());
    let urgency = RwSignal::new("routine".to_string());
    let subject = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let access = RwSignal::new("yes_with_notice".to_string());
    let submitting = RwSignal::new(false);
    let submitted = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let handle_submit = move |_| {
        submitting.set(true);
        let req = MaintenanceRequest {
            category: category.get(),
            urgency: urgency.get(),
            subject: subject.get(),
            description: description.get(),
            unit_access: access.get(),
        };
        leptos::task::spawn_local(async move {
            match submit_maintenance_request(req).await {
                Ok(_) => {
                    submitted.set(true);
                    submitting.set(false);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="main-area" style="max-width:36rem;">
            <div class="page-header">
                <div>
                    <a href="/t/maintenance" class="back-link">"← Maintenance"</a>
                    <h1 class="page-title">"Request Maintenance"</h1>
                </div>
            </div>

            {move || if submitted.get() {
                view! {
                    <div class="wiz-success-card">
                        <div class="wiz-success-icon">"✓"</div>
                        <div class="wiz-success-title">"Request Submitted"</div>
                        <div class="wiz-success-sub">"Your maintenance request has been received. You'll be notified when it's assigned to a technician."</div>
                        <div class="wiz-success-actions">
                            <a href="/t/maintenance" class="btn btn-primary">"View My Requests"</a>
                            <a href="/t" class="btn btn-ghost">"Dashboard"</a>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    // ── Step indicator ──
                    <div class="wiz-steps">
                        <div class=move || format!("wiz-step {}", if step.get() >= 1 { "wiz-step--active" } else { "" })>"1 Category"</div>
                        <div class="wiz-step-divider"></div>
                        <div class=move || format!("wiz-step {}", if step.get() >= 2 { "wiz-step--active" } else { "" })>"2 Details"</div>
                        <div class="wiz-step-divider"></div>
                        <div class=move || format!("wiz-step {}", if step.get() >= 3 { "wiz-step--active" } else { "" })>"3 Confirm"</div>
                    </div>

                    {move || error.get().map(|e| view! {
                        <div class="wiz-error-banner">"⚠ " {e}</div>
                    })}

                    <div class="wiz-card">
                        // ── Step 1: Category + Urgency ──
                        <Show when=move || step.get() == 1>
                            <div class="wiz-section-title">"What needs attention?"</div>
                            <div class="triage-category-grid">
                                {categories().into_iter().map(|cat| {
                                    let cid   = cat.id;
                                    let icon  = cat.icon;
                                    let label = cat.label;
                                    view! {
                                        <button
                                            class=move || format!("triage-cat-btn {}", if category.get() == cid { "triage-cat-btn--active" } else { "" })
                                            on:click=move |_| { category.set(cid.to_string()); }
                                        >
                                            <span class="triage-cat-icon">{icon}</span>
                                            <span class="triage-cat-label">{label}</span>
                                        </button>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>

                            <div class="form-field" style="margin-top:1.25rem;">
                                <label class="form-label">"Urgency"</label>
                                <div class="triage-urgency-row">
                                    {[("routine","🟢 Routine"),("urgent","🟡 Urgent"),("emergency","🔴 Emergency")].iter().map(|(v,l)| {
                                        let v = *v; let l = *l;
                                        view! {
                                            <button
                                                class=move || format!("triage-urgency-btn {}", if urgency.get() == v { "triage-urgency-btn--active" } else { "" })
                                                on:click=move |_| urgency.set(v.to_string())
                                            >{l}</button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>

                            <div class="wiz-footer">
                                <div></div>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || category.get().is_empty()
                                    on:click=move |_| step.set(2)
                                >"Next →"</button>
                            </div>
                        </Show>

                        // ── Step 2: Details ──
                        <Show when=move || step.get() == 2>
                            <div class="form-field">
                                <label class="form-label">"Subject" <span class="form-required">"*"</span></label>
                                <input type="text" class="form-input" placeholder="e.g. Leaking kitchen faucet"
                                    prop:value=move || subject.get()
                                    on:input=move |ev| subject.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Description"</label>
                                <textarea class="form-input str-listing-textarea" rows="5"
                                    placeholder="Describe the issue in as much detail as possible…"
                                    on:input=move |ev| description.set(event_target_value(&ev))
                                >{move || description.get()}</textarea>
                            </div>
                            <div class="form-field">
                                <label class="form-label">"Unit Entry Permission"</label>
                                <select class="form-select"
                                    on:change=move |ev| access.set(event_target_value(&ev))
                                >
                                    <option value="yes_with_notice">"Yes, with 24h notice"</option>
                                    <option value="yes_anytime">"Yes, anytime"</option>
                                    <option value="schedule_only">"Schedule with me first"</option>
                                </select>
                            </div>
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(1)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || subject.get().trim().is_empty()
                                    on:click=move |_| step.set(3)
                                >"Review →"</button>
                            </div>
                        </Show>

                        // ── Step 3: Confirm ──
                        <Show when=move || step.get() == 3>
                            <div class="wiz-confirm-table">
                                <div class="wiz-confirm-row"><span>"Category"</span><strong>{move || category.get().replace('_', " ")}</strong></div>
                                <div class="wiz-confirm-row"><span>"Urgency"</span><strong>{move || urgency.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Subject"</span><strong>{move || subject.get()}</strong></div>
                                <div class="wiz-confirm-row"><span>"Entry"</span><strong>{move || access.get().replace('_', " ")}</strong></div>
                            </div>
                            {move || if !description.get().is_empty() {
                                view! {
                                    <div class="wiz-confirm-desc">
                                        <span class="form-label">"Description"</span>
                                        <p class="text-sm" style="margin-top:0.25rem;">{description.get()}</p>
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                            <div class="wiz-footer">
                                <button class="btn btn-ghost" on:click=move |_| step.set(2)>"← Back"</button>
                                <button
                                    class="btn btn-primary"
                                    disabled=move || submitting.get()
                                    on:click=handle_submit
                                >{move || if submitting.get() { "Submitting…" } else { "Submit Request" }}</button>
                            </div>
                        </Show>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
