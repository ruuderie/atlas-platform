// apps/folio/src/pages/str_host/violation_file.rs
//
// STR Violation Filing — /s/violations/new
// POSTs to /api/folio/violations.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::pages::landlord::assets::list_assets;
use crate::pages::landlord::violations::ViolationRecord;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileViolationInput {
    pub subject: String,
    pub description: String,
    pub priority: String,
    pub category: String,
}

#[derive(Serialize)]
struct FileViolationHttpBody {
    asset_id: Uuid,
    contract_id: Option<Uuid>,
    reservation_id: Option<Uuid>,
    category: String,
    subject: String,
    description: String,
    cure_days: Option<u8>,
    evidence_notes: Option<String>,
}

fn map_str_category(ui: &str) -> &'static str {
    match ui {
        "property_damage" => "property_damage",
        "noise_complaint" | "noise" => "noise",
        "unauthorized_guests" => "unauthorized_occupant",
        "party_gathering" => "unauthorized_party",
        "smoking" => "smoking_in_unit",
        "pet_violation" => "unauthorized_pet",
        "over_occupancy" => "over_occupancy",
        "check_out_late" | "other" | _ => "other",
    }
}

#[server(FileStrViolation, "/api")]
pub async fn file_str_violation(
    input: FileViolationInput,
) -> Result<String, server_fn::error::ServerFnError> {
    use axum::http::HeaderMap;
    use leptos_axum::extract;

    let headers = extract::<HeaderMap>().await.unwrap_or_default();
    let token = crate::auth::extract_bearer_token(&headers)
        .ok_or_else(|| server_fn::error::ServerFnError::new("No session token"))?;

    let assets = list_assets().await?;
    let asset_id = assets
        .iter()
        .find(|a| a.str_eligible)
        .or_else(|| assets.iter().find(|a| a.parent_asset_id.is_some()))
        .or_else(|| assets.first())
        .map(|a| a.id)
        .ok_or_else(|| {
            server_fn::error::ServerFnError::new(
                "No property available to file a violation against.",
            )
        })?;

    let category = map_str_category(&input.category).to_string();
    let evidence_notes = {
        let mut notes = Vec::new();
        if !input.priority.is_empty() {
            notes.push(format!("Priority: {}", input.priority));
        }
        if notes.is_empty() {
            None
        } else {
            Some(notes.join("\n"))
        }
    };

    let body = FileViolationHttpBody {
        asset_id,
        contract_id: None,
        reservation_id: None,
        category,
        subject: input.subject,
        description: input.description,
        cure_days: None,
        evidence_notes,
    };
    let resp = crate::atlas_client::authenticated_post::<FileViolationHttpBody, ViolationRecord>(
        "/api/folio/violations",
        &token,
        None,
        &body,
    )
    .await
    .map_err(|e| server_fn::error::ServerFnError::new(e.to_string()))?;
    Ok(resp.id.to_string())
}

#[component]
pub fn StrViolationFiling() -> impl IntoView {
    let subject = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let priority = RwSignal::new("normal".to_string());
    let category = RwSignal::new("property_damage".to_string());
    let submitting = RwSignal::new(false);
    let submitted = RwSignal::new(false);
    let error_msg = RwSignal::new(None::<String>);

    let handle_submit = move |_| {
        if subject.get().trim().is_empty() {
            return;
        }
        submitting.set(true);
        let input = FileViolationInput {
            subject: subject.get(),
            description: description.get(),
            priority: priority.get(),
            category: category.get(),
        };
        leptos::task::spawn_local(async move {
            match file_str_violation(input).await {
                Ok(_) => {
                    submitted.set(true);
                    submitting.set(false);
                }
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                    submitting.set(false);
                }
            }
        });
    };

    view! {
        <div class="main-area" style="max-width:42rem;">
            <div class="page-header">
                <div>
                    <a href="/s/incidents" class="back-link">"← Incidents"</a>
                    <h1 class="page-title">"File a Violation"</h1>
                    <p class="page-subtitle">"Report a guest incident, property damage, or compliance issue"</p>
                </div>
            </div>

            {move || if submitted.get() {
                view! {
                    <div class="wiz-success-card">
                        <div class="wiz-success-icon">"✓"</div>
                        <div class="wiz-success-title">"Violation Filed"</div>
                        <div class="wiz-success-sub">"Your report has been submitted and assigned a case ID. Your property manager will review and action it."</div>
                        <div class="wiz-success-actions">
                            <a href="/s/incidents" class="btn btn-primary">"View Incidents"</a>
                            <a href="/s" class="btn btn-ghost">"Dashboard"</a>
                        </div>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="wiz-card">
                        {move || error_msg.get().map(|e| view! {
                            <div class="wiz-error-banner">"⚠ " {e}</div>
                        })}

                        <div class="form-field">
                            <label class="form-label">"Violation Category" <span class="form-required">"*"</span></label>
                            <select class="form-select"
                                on:change=move |ev| category.set(event_target_value(&ev))
                            >
                                <option value="property_damage">"Property Damage"</option>
                                <option value="noise_complaint">"Noise Complaint"</option>
                                <option value="unauthorized_guests">"Unauthorized Guests"</option>
                                <option value="party_gathering">"Party / Gathering"</option>
                                <option value="smoking">"Smoking"</option>
                                <option value="pet_violation">"Pet Violation"</option>
                                <option value="over_occupancy">"Over Occupancy"</option>
                                <option value="other">"Other"</option>
                            </select>
                        </div>

                        <div class="form-field">
                            <label class="form-label">"Subject / Title" <span class="form-required">"*"</span></label>
                            <input
                                type="text"
                                class="form-input"
                                placeholder="Brief description of the incident"
                                prop:value=move || subject.get()
                                on:input=move |ev| subject.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="form-field">
                            <label class="form-label">"Priority"</label>
                            <select class="form-select"
                                on:change=move |ev| priority.set(event_target_value(&ev))
                            >
                                <option value="low">"Low — Documentation Only"</option>
                                <option value="normal" selected>"Normal — Review Required"</option>
                                <option value="high">"High — Immediate Action"</option>
                                <option value="urgent">"Urgent — Emergency"</option>
                            </select>
                        </div>

                        <div class="form-field">
                            <label class="form-label">"Detailed Description"</label>
                            <textarea
                                class="form-input str-listing-textarea"
                                rows="6"
                                placeholder="Describe what happened, when it occurred, which unit/booking was affected, and any evidence available…"
                                on:input=move |ev| description.set(event_target_value(&ev))
                            ></textarea>
                        </div>

                        <div class="viol-info-banner">
                            <span class="viol-info-icon">"📸"</span>
                            <p class="viol-info-text">"Photo/video uploads will be available in Phase 7. Please keep evidence on file — you may be contacted for it."</p>
                        </div>

                        <div class="wiz-footer">
                            <a href="/s/incidents" class="btn btn-ghost">"Cancel"</a>
                            <button
                                class="btn btn-primary"
                                disabled=move || submitting.get() || subject.get().trim().is_empty()
                                on:click=handle_submit
                            >
                                {move || if submitting.get() { "Submitting…" } else { "File Violation" }}
                            </button>
                        </div>
                    </div>
                }.into_any()
            }}
        </div>
    }
}
