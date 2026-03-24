use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use crate::api::directories::create_directory;
use crate::api::models::{CreateDirectory, DirectoryTypeModel};
use crate::api::directory_types::get_directory_types;

#[component]
pub fn SiteCreate() -> impl IntoView {
    let site_name = RwSignal::new("".to_string());
    let domain = RwSignal::new("".to_string());
    let strategy = RwSignal::new("multi_tenant".to_string());
    
    let (types, set_types) = signal(Vec::<DirectoryTypeModel>::new());
    let (selected_type, set_selected_type) = signal(None::<String>);
    
    let is_submitting = RwSignal::new(false);

    let navigate = leptos_router::hooks::use_navigate();
    let toast = use_context::<crate::app::GlobalToast>().expect("toast");

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            if let Ok(data) = get_directory_types().await {
                set_types.set(data.clone());
                if let Some(first) = data.first() {
                    set_selected_type.set(Some(first.id.clone()));
                }
            }
        });
    });

    let handle_submit = move |_| {
        if is_submitting.get() { return; }
        let n = site_name.get();
        let d = domain.get();
        if n.is_empty() || d.is_empty() {
            toast.message.set(Some("Name and Domain are required.".to_string()));
            return;
        }

        let type_id = selected_type.get().unwrap_or_default();
        if type_id.is_empty() {
            toast.message.set(Some("A directory type must be selected.".to_string()));
            return;
        }

        is_submitting.set(true);
        toast.message.set(Some("Provisioning network tenant...".to_string()));

        let data = CreateDirectory {
            name: n,
            domain: d,
            directory_type_id: type_id,
            description: "New platform tenant".to_string(),
            deployment_strategy: Some(strategy.get()),
        };

        let nav = navigate.clone();
        leptos::task::spawn_local(async move {
            match create_directory(data).await {
                Ok(_) => {
                    toast.message.set(Some("Tenant successfully provisioned!".to_string()));
                    nav("/sites", Default::default());
                }
                Err(e) => {
                    toast.message.set(Some(format!("Error: {}", e)));
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="max-w-3xl mx-auto space-y-6 pt-8">
            <header class="mb-8">
                <a href="/sites" class="text-sm text-muted-foreground hover:text-foreground mb-4 inline-block">"← Back"</a>
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
                    <div class="space-y-2">
                        <Label>"Domain"</Label>
                        <Input r#type=InputType::Text placeholder="e.g. acme.directory.localhost".to_string() bind_value=domain />
                        <p class="text-xs text-muted-foreground">"The hostname users will use to access this directory."</p>
                    </div>
                    
                    <div class="space-y-2 mt-6">
                        <Label>"Directory Network Type"</Label>
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-2">
                            {move || types.get().into_iter().map(|t| {
                                let t_id = t.id.clone();
                                let t_id_2 = t.id.clone();
                                let t_id_3 = t.id.clone();
                                let input_val = t.id.clone();
                                let check_val = input_val.clone();
                                let change_val = input_val.clone();
                                let name = t.name.clone();
                                let desc = t.description.clone();
                                view! {
                                    <label class="flex items-center gap-3 cursor-pointer border p-4 rounded-xl flex-1 transition-all"
                                        class=("border-primary", move || selected_type.get() == Some(t_id.clone()))
                                        class=("bg-primary/5", move || selected_type.get() == Some(t_id_2.clone()))
                                        class=("border-border", move || selected_type.get() != Some(t_id_3.clone()))
                                        on:click=move |_| set_selected_type.set(Some(change_val.clone()))
                                    >
                                        <input type="radio" name="directory_type" value=input_val 
                                            prop:checked=move || selected_type.get() == Some(check_val.clone())
                                            class="hidden"
                                        />
                                        <div class="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                                            <span class="material-symbols-outlined text-primary">"category"</span>
                                        </div>
                                        <div>
                                            <div class="font-bold text-sm text-foreground">{name}</div>
                                            <div class="text-xs text-muted-foreground">{desc}</div>
                                        </div>
                                    </label>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                    
                    <div class="space-y-2 mt-6">
                        <Label>"Deployment Strategy"</Label>
                        <div class="flex flex-col md:flex-row gap-4 mt-2">
                            <label class="flex items-center gap-3 cursor-pointer border p-4 rounded-xl flex-1 transition-all"
                                class=("border-primary", move || strategy.get() == "multi_tenant")
                                class=("bg-primary/5", move || strategy.get() == "multi_tenant")
                                class=("border-border", move || strategy.get() != "multi_tenant")
                                on:click=move |_| strategy.set("multi_tenant".to_string())
                            >
                                <input type="radio" name="strategy" value="multi_tenant" 
                                    prop:checked=move || strategy.get() == "multi_tenant"
                                    class="hidden"
                                />
                                <div class="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                                    <span class="material-symbols-outlined text-primary">"dns"</span>
                                </div>
                                <div>
                                    <div class="font-bold text-sm text-foreground">"Shared Infrastructure"</div>
                                    <div class="text-xs text-muted-foreground">"Instant provisioning on the multi-tenant cluster."</div>
                                </div>
                            </label>
                            
                            <label class="flex items-center gap-3 cursor-pointer border p-4 rounded-xl flex-1 transition-all"
                                class=("border-primary", move || strategy.get() == "dedicated")
                                class=("bg-primary/5", move || strategy.get() == "dedicated")
                                class=("border-border", move || strategy.get() != "dedicated")
                                on:click=move |_| strategy.set("dedicated".to_string())
                            >
                                <input type="radio" name="strategy" value="dedicated" 
                                    prop:checked=move || strategy.get() == "dedicated"
                                    class="hidden"
                                />
                                <div class="w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                                    <span class="material-symbols-outlined text-primary">"deployed_code"</span>
                                </div>
                                <div>
                                    <div class="font-bold text-sm text-foreground">"Dedicated Container"</div>
                                    <div class="text-xs text-muted-foreground">"Spins up an isolated instance (takes 1-2 mins)."</div>
                                </div>
                            </label>
                        </div>
                    </div>
                </div>
                <div class="flex justify-end gap-4 mt-8 pt-6 border-t border-border">
                    <a href="/sites">
                        <Button variant=ButtonVariant::Outline>"Cancel"</Button>
                    </a>
                    <Button variant=ButtonVariant::Default on:click=handle_submit>
                        {move || if is_submitting.get() { "Provisioning..." } else { "Register Site" }}
                    </Button>
                </div>
            </Card>
        </div>
    }
}
