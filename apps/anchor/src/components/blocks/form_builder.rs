use leptos::*;
use serde::{Deserialize, Serialize};
use leptos_router::ActionForm;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Text,
    Email,
    TextArea,
    Select,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub options: Vec<String>, // For Select fields
    #[serde(default)]
    pub placeholder: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FormBuilderData {
    pub form_id: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub submit_button_text: Option<String>,
    #[serde(default)]
    pub fields: Vec<FormField>,
}

#[server(SubmitDynamicForm, "/api")]
pub async fn submit_dynamic_form(
    form_id: String,
    payload: String,
) -> Result<(), ServerFnError> {
    use axum::Extension;
    use leptos_axum::extract;
    let Extension(state) = extract::<Extension<crate::state::AppState>>().await?;
    let Extension(tenant) = extract::<Extension<crate::state::TenantContext>>().await?;
    let tenant_id = tenant.0.unwrap_or_default();
    
    // Parse payload as JSON map
    let parsed_payload: std::collections::HashMap<String, String> = serde_json::from_str(&payload).unwrap_or_default();
    
    // Fetch tenant admin email from site settings
    let admin_email: String = sqlx::query_scalar("SELECT value FROM site_settings WHERE key = 'admin_email' AND tenant_id = $1")
        .bind(tenant_id)
        .fetch_optional(&state.pool)
        .await?
        .unwrap_or_default();
        
    if !admin_email.is_empty() {
        let subject = format!("New Form Submission: {}", form_id);
        
        let mut body = format!("<h3>New submission for form: {}</h3><br/><ul>", form_id);
        for (k, v) in parsed_payload.iter() {
            body.push_str(&format!("<li><strong>{}</strong>: {}</li>", k, v));
        }
        body.push_str("</ul>");
        
        let _ = crate::email::send_email(admin_email, subject, body).await;
    }
    
    Ok(())
}

#[component]
pub fn FormBuilderBlock(data: FormBuilderData) -> impl IntoView {
    let submit_action = create_server_action::<SubmitDynamicForm>();
    let value = submit_action.value();
    let pending = submit_action.pending();
    
    let btn_text = data.submit_button_text.clone().unwrap_or_else(|| "Submit".to_string());
    
    view! {
        <section class="py-16 bg-surface-container-low w-full">
            <div class="container mx-auto px-4 max-w-3xl">
                <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 md:p-12 shadow-2xl relative overflow-hidden">
                    <div class="absolute top-0 left-0 w-full h-2 bg-gradient-to-r from-primary to-secondary"></div>
                    
                    <h2 class="text-3xl font-black text-on-surface mb-4 text-center tracking-tight">
                        {data.title.clone()}
                    </h2>
                    {if data.description.is_some() && !data.description.clone().unwrap_or_default().is_empty() {
                        view!{ <p class="text-center text-on-surface-variant text-lg mb-10 font-light">
                            {data.description.clone().unwrap()}
                        </p> }.into_view()
                    } else { view!{}.into_view() }}
                    
                    <Show
                        when=move || value.with(|v| v.is_some())
                        fallback={
                            let data_for_fallback = data.clone();
                            let btn_text_for_fallback = btn_text.clone();
                            move || {
                                let data = data_for_fallback.clone();
                                let btn_text = btn_text_for_fallback.clone();
                                
                                let (payload_map, set_payload_map) = create_signal(std::collections::HashMap::<String, String>::new());
                                
                                view! {
                                    <ActionForm action=submit_action class="flex flex-col gap-6">
                                        <input type="hidden" name="form_id" value=data.form_id.clone() />
                                        <input type="hidden" name="payload" value=move || serde_json::to_string(&payload_map.get()).unwrap_or_default() />
                                        
                                        {data.fields.clone().into_iter().map(|field| {
                                            let field_name_clone = field.name.clone();
                                            let field_name_input = field.name.clone();
                                            let req = field.required;
                                        
                                        let on_input = move |ev: leptos::ev::Event| {
                                            let val = event_target_value(&ev);
                                            set_payload_map.update(|m| {
                                                m.insert(field_name_input.clone(), val);
                                            });
                                        };
                                        
                                        view! {
                                            <div class="flex flex-col gap-2">
                                                <label class="text-sm font-bold text-on-surface uppercase tracking-wider">
                                                    {field.label.clone()} {if req { view!{<span class="text-error ml-1">"*"</span>}.into_view() } else { view!{}.into_view() }}
                                                </label>
                                                {match field.field_type {
                                                    FieldType::Text => view! {
                                                        <input type="text" on:input=on_input.clone() required=req placeholder=field.placeholder.clone().unwrap_or_default() class="w-full bg-surface-container px-4 py-3 rounded-xl border border-outline-variant focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all" />
                                                    }.into_view(),
                                                    FieldType::Email => view! {
                                                        <input type="email" on:input=on_input.clone() required=req placeholder=field.placeholder.clone().unwrap_or_default() class="w-full bg-surface-container px-4 py-3 rounded-xl border border-outline-variant focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all" />
                                                    }.into_view(),
                                                    FieldType::TextArea => view! {
                                                        <textarea on:input=on_input.clone() required=req placeholder=field.placeholder.clone().unwrap_or_default() rows="4" class="w-full bg-surface-container px-4 py-3 rounded-xl border border-outline-variant focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all resize-y"></textarea>
                                                    }.into_view(),
                                                    FieldType::Select => {
                                                        let options = field.options.clone();
                                                        let on_change = on_input.clone();
                                                        view! {
                                                            <select on:change=on_change required=req class="w-full bg-surface-container px-4 py-3 rounded-xl border border-outline-variant focus:border-primary focus:ring-1 focus:ring-primary outline-none transition-all appearance-none">
                                                                <option value="" disabled selected>"Select an option"</option>
                                                                {options.into_iter().map(|opt| view! { <option value=opt.clone()>{opt}</option> }).collect_view()}
                                                            </select>
                                                        }.into_view()
                                                    }
                                                }}
                                            </div>
                                        }
                                    }).collect_view()}
                                    
                                    <button type="submit" disabled=move || pending.get() class="mt-4 w-full py-4 rounded-xl font-bold text-lg bg-primary text-on-primary hover:bg-primary/90 transition-colors shadow-md disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2">
                                        <Show when=move || pending.get() fallback=|| view! { <span></span> }>
                                            <span class="material-symbols-outlined animate-spin">"progress_activity"</span>
                                        </Show>
                                        {btn_text.clone()}
                                    </button>
                                </ActionForm>
                            }.into_view()
                        }}
                    >
                        <div class="flex flex-col items-center justify-center py-12 px-6 text-center bg-primary-container/30 rounded-2xl border border-primary/20">
                            <span class="material-symbols-outlined text-[4rem] text-primary mb-4">"check_circle"</span>
                            <h3 class="text-2xl font-bold text-on-surface mb-2">"Form Submitted Successfully!"</h3>
                            <p class="text-on-surface-variant">"Thank you for reaching out. We have received your details and will get back to you shortly."</p>
                        </div>
                    </Show>
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_builder_data_deserialization() {
        let json = r#"{
            "form_id": "test_form",
            "title": "Contact Us",
            "description": "Please fill out this form.",
            "submit_button_text": "Send",
            "fields": [
                {
                    "name": "email",
                    "label": "Email Address",
                    "field_type": "email",
                    "required": true,
                    "placeholder": "jane@example.com"
                },
                {
                    "name": "interest",
                    "label": "Interest",
                    "field_type": "select",
                    "required": false,
                    "options": ["Passive", "Active"]
                }
            ]
        }"#;

        let data: FormBuilderData = serde_json::from_str(json).unwrap();
        assert_eq!(data.form_id, "test_form");
        assert_eq!(data.title, "Contact Us");
        assert_eq!(data.submit_button_text.unwrap(), "Send");
        assert_eq!(data.fields.len(), 2);
        
        assert_eq!(data.fields[0].name, "email");
        assert_eq!(data.fields[0].field_type, FieldType::Email);
        assert!(data.fields[0].required);
        
        assert_eq!(data.fields[1].name, "interest");
        assert_eq!(data.fields[1].field_type, FieldType::Select);
        assert!(!data.fields[1].required);
        assert_eq!(data.fields[1].options.len(), 2);
    }
}
