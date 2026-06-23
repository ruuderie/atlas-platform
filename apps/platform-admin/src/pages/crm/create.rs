use leptos::prelude::*;
use std::collections::HashMap;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use crate::components::dynamic_form::{DynamicForm, DynamicField, DynamicFieldType, DynamicSelectOption};
use crate::api::crm::create_lead;
use crate::api::models::CreateLead;

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
                DynamicSelectOption { label: "Website".to_string(), value: "website".to_string() },
                DynamicSelectOption { label: "Referral".to_string(), value: "referral".to_string() },
                DynamicSelectOption { label: "Cold Call".to_string(), value: "cold_call".to_string() },
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
        let name  = data.get("lead_name").cloned().unwrap_or_default();
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
                    toast.show_toast("CRM", &format!("Lead '{}' ingested successfully.", lead.name), "success");
                    navigate("/crm", Default::default());
                }
                Err(e) => toast.show_toast("Error", &format!("Failed to create lead: {}", e), "error"),
            }
        });
    };

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/crm" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                <h2 class="text-3xl font-bold tracking-tight">"New Lead"</h2>
                <p class="text-muted-foreground mt-2">"Ingest a new prospect or lead directly into the CRM tracking database."</p>
                <div class="mt-4 px-3 py-1 bg-primary/10 text-primary text-xs font-mono rounded inline-block">"Server-Driven UI Enabled"</div>
            </header>
            
            <Card class="p-8 bg-card border border-border shadow-sm".to_string()>
                <DynamicForm layout=layout on_submit=handle_submit class="space-y-6".to_string()>
                    <div class="flex justify-end gap-4 mt-8 pt-6 border-t border-border">
                        <a href="/crm">
                            <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                        </a>
                        <Button variant=ButtonVariant::Default>"Save Lead"</Button>
                    </div>
                </DynamicForm>
            </Card>
        </div>
    }
}
