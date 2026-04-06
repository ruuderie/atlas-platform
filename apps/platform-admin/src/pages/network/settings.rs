use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::switch::Switch;

#[component]
pub fn NetworkSettingsPanel() -> impl IntoView {
    let params = use_params_map();
    let _site_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let domain_override_bind = RwSignal::new("".to_string());
    let auto_approve_bind = RwSignal::new(false);
    let network_identity_bind = RwSignal::new("Directory".to_string());

    let handle_save = move |_| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        toast.message.set(Some("Network structural settings securely applied.".to_string()));
    };

    view! {
        <Card class="bg-card border-border shadow-sm p-6 mb-6".to_string()>
            <h3 class="text-lg font-semibold mb-4 text-primary">"Network Core Configuration"</h3>
            <div class="space-y-4 max-w-lg">
                
                <div class="space-y-4 pb-6 border-b border-border">
                    <div class="space-y-2">
                        <label class="text-sm font-medium leading-none">"Custom Domain Override"</label>
                        <Input r#type=InputType::Text class="font-mono w-full".to_string() bind_value=domain_override_bind placeholder="e.g. directory.example.com".to_string() />
                        <p class="text-xs text-muted-foreground">"The primary public CNAME strictly mapped to this network instance."</p>
                    </div>
                </div>

                <div class="space-y-2 pt-2">
                    <label class="text-sm font-medium leading-none">"Network Identity Mode"</label>
                    <Input r#type=InputType::Text class="w-full".to_string() bind_value=network_identity_bind placeholder="e.g. Standard Directory, B2B Marketplace".to_string() />
                    <p class="text-xs text-muted-foreground">"Dictates core taxonomy structure and UI layout logic."</p>
                </div>
                
                <div class="space-y-2 mt-6 p-4 border border-outline-variant/20 rounded-md bg-surface-container-low transition-colors hover:bg-surface-container">
                    <div class="flex items-center justify-between">
                        <div>
                            <label class="text-sm font-bold leading-none">"Auto-Approve Listings Workflow"</label>
                            <p class="text-xs text-muted-foreground mt-1 max-w-[280px]">"Bypass manual moderation queues for new tenant provisions."</p>
                        </div>
                        <Switch id="auto_approve_toggle".to_string() checked=auto_approve_bind.get() />
                    </div>
                </div>

                <div class="pt-4 border-t border-border mt-6 flex justify-end">
                    <Button variant=ButtonVariant::Default on:click=handle_save>"Update Topology Settings"</Button>
                </div>
            </div>
        </Card>
    }
}
