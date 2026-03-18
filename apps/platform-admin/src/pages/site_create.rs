use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

#[component]
pub fn SiteCreate() -> impl IntoView {
    let site_name = RwSignal::new("".to_string());
    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    let handle_submit = move |_| {
        toast.message.set(Some("Provisioning network tenant...".to_string()));
        navigate("/sites", Default::default());
    };

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
                <h2 class="text-3xl font-bold tracking-tight">"Register New Tenant"</h2>
                <p class="text-muted-foreground mt-2">"Configure a brand new directory site within your platform network."</p>
            </header>
            
            <Card class="p-8 bg-card border border-border shadow-sm".to_string()>
                <div class="space-y-6">
                    <div class="space-y-2">
                        <Label>"Site Name"</Label>
                        <Input r#type=InputType::Text placeholder="e.g. Acme Corp Directory".to_string() bind_value=site_name />
                        <p class="text-xs text-muted-foreground">"This will be the primary identifier for your tenant platform."</p>
                    </div>
                </div>
                <div class="flex justify-end gap-4 mt-8 pt-6 border-t border-border">
                    <a href="/sites">
                        <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                    </a>
                    <Button variant=ButtonVariant::Default on:click=handle_submit>"Register Site"</Button>
                </div>
            </Card>
        </div>
    }
}
