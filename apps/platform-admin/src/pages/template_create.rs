use leptos::prelude::*;
use crate::app::GlobalToast;
use shared_ui::components::ui::button::{Button, ButtonVariant};

#[component]
pub fn TemplateCreate() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let template_type = RwSignal::new("General".to_string());
    let is_active = RwSignal::new(true);

    let submit_action = Action::new_local(move |_: &()| {
        let n = name.get();
        let d = description.get();
        let tt = template_type.get();
        let a = is_active.get();
        
        let t = toast.clone();
        let nav = navigate.clone();

        async move {
            if n.is_empty() {
                t.show_toast("Error", "Name is required.", "error");
                return;
            }

            use crate::api::client::{api_url, create_client, with_credentials, ApiErrorResponse};
            use serde_json::json;

            let payload = json!({
                "name": n,
                "description": d,
                "template_type": tt,
                "is_active": a,
                "category_id": "00000000-0000-0000-0000-000000000000", // MVP hardcoded
                "directory_id": "00000000-0000-0000-0000-000000000000"
            });

            let client = create_client();
            let url = api_url("/api/admin/templates");
            let req = client.post(&url).json(&payload);
            let req = with_credentials(req);
            
            match req.send().await {
                Ok(res) if res.status().is_success() => {
                    t.show_toast("Success", "Template created.", "success");
                    nav("/templates", leptos_router::NavigateOptions::default());
                }
                Ok(res) => {
                    if let Ok(err) = res.json::<ApiErrorResponse>().await {
                        t.show_toast("Error", &err.message.unwrap_or("Failed".into()), "error");
                    } else {
                        t.show_toast("Error", "Failed to create template", "error");
                    }
                }
                Err(e) => t.show_toast("Error", &format!("Network error: {}", e), "error"),
            }
        }
    });

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/templates" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                <h2 class="text-3xl font-bold tracking-tight">"Create Template"</h2>
                <p class="text-muted-foreground mt-2">"Define a new data schema template."</p>
            </header>

            <form class="space-y-6 bg-surface-container rounded-2xl p-8 border border-outline-variant/10 shadow-sm"
                on:submit=move |e| {
                    e.prevent_default();
                    submit_action.dispatch(());
                }
            >
                <div>
                    <label class="block text-sm font-medium text-on-surface mb-2">"Template Name"</label>
                    <input type="text" 
                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                        placeholder="e.g. BlogPost"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-on-surface mb-2">"Description"</label>
                    <textarea 
                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                        placeholder="Describe the usage of this template"
                        rows="3"
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))
                    ></textarea>
                </div>
                <div>
                    <label class="block text-sm font-medium text-on-surface mb-2">"Type"</label>
                    <select 
                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary block p-3"
                        prop:value=move || template_type.get()
                        on:change=move |ev| template_type.set(event_target_value(&ev))
                    >
                        <option value="General">"General"</option>
                        <option value="Listing">"Listing"</option>
                        <option value="Profile">"Profile"</option>
                        <option value="Event">"Event"</option>
                    </select>
                </div>
                <div class="flex items-center gap-3 mt-4">
                    <input type="checkbox" 
                        id="is_active"
                        prop:checked=move || is_active.get()
                        on:change=move |ev| is_active.set(event_target_checked(&ev))
                        class="w-4 h-4 text-primary bg-surface-container-highest border-outline/20 rounded focus:ring-primary"
                    />
                    <label for="is_active" class="text-sm font-medium text-on-surface">"Active"</label>
                </div>

                <div class="flex justify-end gap-4 pt-6 mt-6 border-t border-outline-variant/10">
                    <a href="/templates">
                        <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                    </a>
                    <Button variant=ButtonVariant::Default>"Create Template"</Button>
                </div>
            </form>
        </div>
    }
}
