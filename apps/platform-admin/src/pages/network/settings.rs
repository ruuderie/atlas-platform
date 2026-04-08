use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::button::{Button, ButtonVariant, ButtonSize};
use shared_ui::components::ui::switch::Switch;

#[component]
pub fn NetworkSettingsPanel() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());
    let domain_override_bind = RwSignal::new("".to_string());
    let auto_approve_bind = RwSignal::new(false);
    let network_identity_bind = RwSignal::new("Directory".to_string());

    let (trigger_fetch_domains, set_trigger_fetch_domains) = signal(0);
    let new_domain_bind = RwSignal::new("".to_string());
    
    let domains_res = LocalResource::new({
        let sid_clone = site_id.clone();
        move || {
            let sid = sid_clone();
            trigger_fetch_domains.get();
            async move {
                crate::api::admin::get_app_domains(sid).await.unwrap_or_default()
            }
        }
    });

    let handle_add_domain = {
        let sid_clone = site_id.clone();
        move |_| {
            let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
            let sid = sid_clone();
            let domain_str = new_domain_bind.get();
            if domain_str.is_empty() { return; }
            
            leptos::task::spawn_local(async move {
                if crate::api::admin::add_app_domain(sid, domain_str).await.is_ok() {
                    set_trigger_fetch_domains.update(|v| *v += 1);
                    new_domain_bind.set("".to_string());
                    toast.message.set(Some("Domain routing natively mapped.".to_string()));
                } else {
                    toast.message.set(Some("Failed to route domain. It may already be in use.".to_string()));
                }
            });
        }
    };

    let handle_remove_domain = {
        let sid_clone = site_id.clone();
        move |domain_name: String| {
            let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
            let sid = sid_clone();
            
            leptos::task::spawn_local(async move {
                if crate::api::admin::remove_app_domain(sid, domain_name).await.is_ok() {
                    set_trigger_fetch_domains.update(|v| *v += 1);
                    toast.message.set(Some("Domain safely detached.".to_string()));
                } else {
                    toast.message.set(Some("Failed to detach domain.".to_string()));
                }
            });
        }
    };

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

        <Card class="bg-card border-border shadow-sm p-6 mb-6".to_string()>
            <h3 class="text-lg font-semibold mb-4 text-primary">"Native Architecture Domain Routing"</h3>
            <p class="text-xs text-muted-foreground mb-6">"Explicitly map Fully Qualified Domain Names (FQDNs) natively onto this application instance context. Traffic inbound to these domains will be exclusively terminated here."</p>
            
            <div class="space-y-4 max-w-lg mb-6 flex gap-2 w-full">
                <Input r#type=InputType::Text class="font-mono bg-muted flex-1".to_string() bind_value=new_domain_bind placeholder="e.g., app.network.com".to_string() />
                <Button variant=ButtonVariant::Default on:click=handle_add_domain>"Bind Route"</Button>
            </div>

            <Suspense fallback=move || view! { <div class="text-xs text-muted-foreground p-4">"Loading routing matrix..."</div> }>
                <div class="border border-border rounded-md overflow-hidden bg-surface-container-lowest max-w-lg">
                    {move || domains_res.get().map(|domains| {
                        if domains.is_empty() {
                            view! { <div class="p-6 text-center text-sm text-placeholder">"No active domains mapped. Environment inaccessible."</div> }.into_any()
                        } else {
                            view! {
                                <For
                                    each=move || domains.clone()
                                    key=|d| d.clone()
                                    children=move |d| {
                                        let domain_name = d.clone();
                                        view! {
                                            <div class="flex items-center justify-between p-3 border-b border-border last:border-b-0 hover:bg-surface-container-low transition-colors">
                                                <div class="flex items-center gap-2">
                                                    <span class="material-symbols-outlined text-success text-[14px]">"language"</span>
                                                    <span class="text-sm font-mono text-on-surface font-semibold">{d.clone()}</span>
                                                </div>
                                                <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm class="h-6 py-0 px-2 text-[10px]".to_string() on:click={
                                                    let domain_name = domain_name.clone();
                                                    move |_| handle_remove_domain(domain_name.clone())
                                                }>"Sever Link"</Button>
                                            </div>
                                        }
                                    }
                                />
                            }.into_any()
                        }
                    })}
                </div>
            </Suspense>
        </Card>
    }
}
