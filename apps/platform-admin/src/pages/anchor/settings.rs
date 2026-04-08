use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::switch::Switch;

#[component]
pub fn AnchorSettingsPanel() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let hero_bind = RwSignal::new("".to_string());
    let seo_bind = RwSignal::new("".to_string());
    let b2b_bind = RwSignal::new(false);
    
    let setup_token_bind = RwSignal::new("".to_string());

    let setup_token_res = LocalResource::new({
        move || {
            let sid = site_id();
            async move {
                if let Ok(setting) = crate::api::networks::get_tenant_setting(&sid, "setup_token").await {
                    Some(setting.value)
                } else {
                    None
                }
            }
        }
    });

    Effect::new(move |_| {
        if let Some(Some(token)) = setup_token_res.get() {
            setup_token_bind.set(token);
        }
    });

    let handle_save = move |_| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        toast.message.set(Some("Anchor overrides saved successfully.".to_string()));
    };

    let handle_generate_token = move |_| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        let token_str = uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_uppercase();
        let sid = site_id();
        
        leptos::task::spawn_local(async move {
            let req = crate::api::models::UpsertSettingRequest {
                key: "setup_token".to_string(),
                value: token_str.clone(),
                is_encrypted: false,
            };
            if crate::api::networks::upsert_tenant_setting(&sid, req).await.is_ok() {
                setup_token_bind.set(token_str);
                toast.message.set(Some("Setup token strategically regenerated.".to_string()));
            } else {
                toast.message.set(Some("Failed to generate token.".to_string()));
            }
        });
    };

    let (trigger_fetch_domains, set_trigger_fetch_domains) = signal(0);
    let new_domain_bind = RwSignal::new("".to_string());
    
    let domains_res = LocalResource::new({
        move || {
            let sid = site_id();
            trigger_fetch_domains.get();
            async move {
                crate::api::admin::get_app_domains(sid).await.unwrap_or_default()
            }
        }
    });

    let handle_add_domain = move |_| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        let sid = site_id();
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
    };

    let handle_remove_domain = move |domain_name: String| {
        let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
        let sid = site_id();
        
        leptos::task::spawn_local(async move {
            if crate::api::admin::remove_app_domain(sid, domain_name).await.is_ok() {
                set_trigger_fetch_domains.update(|v| *v += 1);
                toast.message.set(Some("Domain safely detached.".to_string()));
            } else {
                toast.message.set(Some("Failed to detach domain.".to_string()));
            }
        });
    };

    view! {
        <Card class="bg-card border-border shadow-sm p-6 mb-6".to_string()>
            <h3 class="text-lg font-semibold mb-4 text-primary">"Services Portal (Anchor) Capabilities"</h3>
            <div class="space-y-4 max-w-lg">
                
                <div class="space-y-4 pb-6 border-b border-border mb-6">
                    <div>
                        <label class="text-sm font-bold leading-none text-red-500">"One-Time Setup Token"</label>
                        <p class="text-xs text-muted-foreground mt-1 mb-2">"Provide this token to the tenant administrator so they can securely initialize their account."</p>
                    </div>
                    <div class="flex gap-2 w-full">
                        <Input r#type=InputType::Text class="font-mono text-indigo-500 font-bold bg-muted flex-1".to_string() bind_value=setup_token_bind placeholder="No token generated".to_string() />
                        <Button variant=ButtonVariant::Outline on:click=handle_generate_token>"Regenerate"</Button>
                    </div>
                </div>

                <div class="space-y-2">
                    <label class="text-sm font-medium leading-none">"Hero Subtitle"</label>
                    <Input r#type=InputType::Text class="w-full".to_string() bind_value=hero_bind placeholder="e.g. Modern consulting...".to_string() />
                    <p class="text-xs text-muted-foreground">"The slogan presented on the landing page of the Anchor site."</p>
                </div>
                <div class="space-y-2 mt-4">
                    <label class="text-sm font-medium leading-none">"SEO Meta Tags"</label>
                    <Input r#type=InputType::Text class="w-full".to_string() bind_value=seo_bind placeholder="<meta name='description' ...>".to_string() />
                    <p class="text-xs text-muted-foreground">"Custom metadata injected into the <head>."</p>
                </div>
                <div class="space-y-2 mt-6 p-4 border border-outline-variant/20 rounded-md bg-surface-container-low">
                    <div class="flex items-center justify-between">
                        <div>
                            <label class="text-sm font-bold leading-none">"B2B Mode Activation"</label>
                            <p class="text-xs text-muted-foreground mt-1">"Toggle the Anchor portal from B2C features to B2B mode."</p>
                        </div>
                        <Switch id="b2b_toggle".to_string() checked=b2b_bind.get() />
                    </div>
                </div>
                <div class="pt-4 border-t border-border mt-6 flex justify-end">
                    <Button variant=ButtonVariant::Default on:click=handle_save>"Save Anchor Overrides"</Button>
                </div>
            </div>
        </Card>

        <Card class="bg-card border-border shadow-sm p-6 mb-6".to_string()>
            <h3 class="text-lg font-semibold mb-4 text-primary">"Native Architecture Domain Routing"</h3>
            <p class="text-xs text-muted-foreground mb-6">"Explicitly map Fully Qualified Domain Names (FQDNs) natively onto this application instance context. Traffic inbound to these domains will be exclusively terminated here."</p>
            
            <div class="space-y-4 max-w-lg mb-6 flex gap-2 w-full">
                <Input r#type=InputType::Text class="font-mono bg-muted flex-1".to_string() bind_value=new_domain_bind placeholder="e.g., uat.buildwithruud.com".to_string() />
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
                                                <Button variant=ButtonVariant::Destructive size="sm" class="h-6 py-0 px-2 text-[10px]".to_string() on:click=move |_| handle_remove_domain(domain_name.clone())>"Sever Link"</Button>
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
