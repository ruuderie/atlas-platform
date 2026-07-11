use crate::api::crm::create_lead;
use crate::api::models::CreateLead;
use crate::components::dynamic_form::{
    DynamicField, DynamicFieldType, DynamicForm, DynamicSelectOption,
};
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub fn CrmCreate() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    let layout = vec![
        DynamicField {
            id: "lead_name".to_string(),
            name: "lead_name".to_string(),
            label: "Full Name".to_string(),
            field_type: DynamicFieldType::Text,
            required: true,
            placeholder: Some("e.g. Jane Doe".to_string()),
            default_value: None,
            options: None,
        },
        DynamicField {
            id: "lead_email".to_string(),
            name: "lead_email".to_string(),
            label: "Email Address".to_string(),
            field_type: DynamicFieldType::Email,
            required: true,
            placeholder: Some("jane@example.com".to_string()),
            default_value: None,
            options: None,
        },
        DynamicField {
            id: "lead_source".to_string(),
            name: "lead_source".to_string(),
            label: "Lead Source".to_string(),
            field_type: DynamicFieldType::Select,
            required: false,
            placeholder: Some("Select source...".to_string()),
            default_value: None,
            options: Some(vec![
                DynamicSelectOption {
                    label: "Website".to_string(),
                    value: "website".to_string(),
                },
                DynamicSelectOption {
                    label: "Referral".to_string(),
                    value: "referral".to_string(),
                },
                DynamicSelectOption {
                    label: "Cold Call".to_string(),
                    value: "cold_call".to_string(),
                },
            ]),
        },
        DynamicField {
            id: "newsletter".to_string(),
            name: "newsletter".to_string(),
            label: "Subscribe to Newsletter".to_string(),
            field_type: DynamicFieldType::Checkbox,
            required: false,
            placeholder: None,
            default_value: Some("true".to_string()),
            options: None,
        },
    ];

    let handle_submit = move |data: HashMap<String, String>| {
        let name = data.get("lead_name").cloned().unwrap_or_default();
        let email = data.get("lead_email").cloned().unwrap_or_default();
        if name.trim().is_empty() || email.trim().is_empty() {
            toast.show_toast("Validation", "Name and email are required.", "error");
            return;
        }
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            let payload = CreateLead {
                name: name.trim().to_string(),
                email: Some(email.trim().to_string()),
            };
            match create_lead(payload).await {
                Ok(lead) => {
                    toast.show_toast(
                        "CRM",
                        &format!("Lead '{}' ingested successfully.", lead.name),
                        "success",
                    );
                    navigate("/crm", Default::default());
                }
                Err(e) => {
                    toast.show_toast("Error", &format!("Failed to create lead: {}", e), "error")
                }
            }
        });
    };

    view! {
        <div class="main-canvas">
            // ── Page Header ──
            <div class="page-header">
                <div>
                    <a href="/crm" style="font-size:12px;color:var(--text-muted);display:inline-block;margin-bottom:4px;">"← Back to CRM"</a>
                    <h1 class="page-title">"New Lead"</h1>
                    <p class="page-subtitle">"Ingest a new prospect or lead directly into the CRM tracking database."</p>
                </div>
            </div>

            <div class="section">
                <DynamicForm layout=layout on_submit=handle_submit class="".to_string()>
                    <div style="display:flex;justify-content:flex-end;gap:10px;margin-top:24px;padding-top:16px;border-top:1px solid var(--border-subtle);">
                        <a href="/crm"><button class="btn btn-ghost">"Cancel"</button></a>
                        <button class="btn btn-primary" type="submit">"Save Lead"</button>
                    </div>
                </DynamicForm>
            </div>
        </div>
    }
}
