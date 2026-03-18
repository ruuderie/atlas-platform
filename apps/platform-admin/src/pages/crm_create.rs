use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

#[component]
pub fn CrmCreate() -> impl IntoView {
    let lead_email = RwSignal::new("".to_string());
    let lead_name = RwSignal::new("".to_string());
    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    let handle_submit = move |_| {
        toast.message.set(Some("Lead ingested successfully.".to_string()));
        navigate("/crm", Default::default());
    };

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                <h2 class="text-3xl font-bold tracking-tight">"New Lead"</h2>
                <p class="text-muted-foreground mt-2">"Ingest a new prospect or lead directly into the CRM tracking database."</p>
            </header>
            
            <Card class="p-8 bg-card border border-border shadow-sm".to_string()>
                <div class="space-y-6">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div class="space-y-2">
                            <Label>"Full Name"</Label>
                            <Input r#type=InputType::Text placeholder="e.g. Jane Doe".to_string() bind_value=lead_name />
                        </div>
                        <div class="space-y-2">
                            <Label>"Email Address"</Label>
                            <Input r#type=InputType::Email placeholder="jane@example.com".to_string() bind_value=lead_email />
                        </div>
                    </div>
                </div>
                <div class="flex justify-end gap-4 mt-8 pt-6 border-t border-border">
                    <a href="/crm">
                        <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                    </a>
                    <Button variant=ButtonVariant::Default on:click=handle_submit>"Save Lead"</Button>
                </div>
            </Card>
        </div>
    }
}
