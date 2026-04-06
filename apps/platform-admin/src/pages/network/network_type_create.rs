use leptos::prelude::*;
use crate::app::GlobalToast;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

#[component]
pub fn NetworkTypeCreate() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<GlobalToast>().expect("GlobalToast not found");

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());

    let submit_action = Action::new_local(move |_: &()| {
        let n = name.get();
        let d = description.get();
        
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
                "description": d
            });

            let client = create_client();
            let url = api_url("/api/admin/network-types");
            let req = client.post(&url).json(&payload);
            let req = with_credentials(req);
            
            match req.send().await {
                Ok(res) if res.status().is_success() => {
                    t.show_toast("Success", "Network Type created.", "success");
                    nav("/network/network-types", leptos_router::NavigateOptions::default());
                }
                Ok(res) => {
                    if let Ok(err) = res.json::<ApiErrorResponse>().await {
                        t.show_toast("Error", &err.message.unwrap_or("Failed".into()), "error");
                    } else {
                        t.show_toast("Error", "Failed to create type", "error");
                    }
                }
                Err(e) => t.show_toast("Error", &format!("Network error: {}", e), "error"),
            }
        }
    });

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/network/network-types" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                <h2 class="text-3xl font-bold tracking-tight">"Create Network Type"</h2>
                <p class="text-muted-foreground mt-2">"Define a new top-level schema classification."</p>
            </header>

            <form class="space-y-6 bg-surface-container rounded-2xl p-8 border border-outline-variant/10 shadow-sm"
                on:submit=move |e| {
                    e.prevent_default();
                    submit_action.dispatch(());
                }
            >
                <div>
                    <label class="block text-sm font-medium text-on-surface mb-2">"Type Name"</label>
                    <input type="text" 
                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                        placeholder="e.g. Real Estate"
                        prop:value=move || name.get()
                        on:input=move |ev| name.set(event_target_value(&ev))
                        required
                    />
                </div>
                <div>
                    <label class="block text-sm font-medium text-on-surface mb-2">"Description"</label>
                    <textarea 
                        class="w-full bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg focus:ring-primary focus:border-primary block p-3"
                        placeholder="Describe the purpose of this network type"
                        rows="3"
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))
                    ></textarea>
                </div>
                
                <div class="flex justify-end gap-4 pt-6 mt-6 border-t border-outline-variant/10">
                    <a href="/network/network-types">
                        <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                    </a>
                    <Button variant=ButtonVariant::Default>"Create Type"</Button>
                </div>
            </form>
        </div>
    }
}
