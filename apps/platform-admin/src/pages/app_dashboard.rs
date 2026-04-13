use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use shared_ui::components::card::Card;
use shared_ui::components::ui::tabs::{Tabs, TabsList, TabsTrigger, TabsContent};
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::table::{
    Table as DataTable, TableBody as DataTableBody, TableCell as DataTableCell,
    TableHead as DataTableHead, TableHeader as DataTableHeader, TableRow as DataTableRow,
};
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use shared_ui::components::ui::related_list::RelatedList;

use crate::components::upsell_banner::UpsellBanner;

#[component]
pub fn AppDashboard() -> impl IntoView {
    let params = use_params_map();
    let site_id = move || params.with(|p| p.get("id").unwrap_or_default());

    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    
    let (show_add_listing, set_show_add_listing) = signal(false);
    let (show_add_category, set_show_add_category) = signal(false);
    let (show_add_template, set_show_add_template) = signal(false);
    
    let (editing_listing_name, set_editing_listing_name) = signal(None::<String>);
    let (managing_user_name, set_managing_user_name) = signal(None::<String>);

    let dirs = use_context::<LocalResource<Vec<crate::api::models::PlatformAppModel>>>().expect("dirs context");
    let domain_bind = RwSignal::new(String::new());
    
    Effect::new(move |_| {
        let current_id = site_id();
        if let Some(d) = dirs.get() {
            if let Some(dir) = d.into_iter().find(|dir| dir.instance_id.to_string() == current_id) {
                domain_bind.set(dir.domain.clone());
            } else {
                domain_bind.set(format!("{}.example.com", current_id));
            }
        }
    });
    
    let site_id_str = site_id().to_string();
    let listings_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::listings::get_listings(&sid).await.unwrap_or_default() }
        }
    });

    let profiles_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::admin::get_users(uuid::Uuid::parse_str(&sid).ok()).await.unwrap_or_default() }
        }
    });

    let categories_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::categories::get_categories(Some(sid)).await.unwrap_or_default() }
        }
    });

    let templates_res = LocalResource::new(move || async move {
        crate::api::templates::get_templates().await.unwrap_or_default()
    });

    let domains_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::admin::get_app_domains(sid).await.unwrap_or_default() }
        }
    });

    let (show_domain_modal, set_show_domain_modal) = signal(false);
    let new_domain_input = RwSignal::new(String::new());
    
    let add_domain_action = Action::new({
        let toast = toast.clone();
        let sid = site_id_str.clone();
        move |domain: &String| {
            let domain = domain.clone();
            let sid = sid.clone();
            let toast = toast.clone();
            async move {
                match crate::api::admin::add_app_domain(sid, domain).await {
                    Ok(_) => { toast.message.set(Some("Domain securely attached.".to_string())); }
                    Err(e) => { toast.message.set(Some(format!("Error adding domain: {}", e))); }
                }
            }
        }
    });

    // Local database resources automatically populate children panes

    let app_manifest = Signal::derive(move || {
        let current_id = site_id();
        let app_type_str = if let Some(d) = dirs.get() {
            if let Some(dir) = d.into_iter().find(|dir| dir.instance_id.to_string() == current_id) {
                dir.app_type.clone()
            } else {
                "network".to_string()
            }
        } else {
            "network".to_string()
        };
        crate::components::app_manifest::get_manifest_for_app_type(&app_type_str)
    });

    view! {
        <Show 
            when=move || dirs.get().is_some() 
            fallback=|| view! { 
                <div class="p-8 text-center text-on-surface-variant flex flex-col items-center justify-center min-h-[400px]">
                    <div class="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full mb-4"></div>
                    "Loading Application Workspace..."
                </div> 
            }
        >
            <div class="w-full max-w-[1600px] mx-auto space-y-6 p-6">
                <header class="flex flex-col md:flex-row justify-between items-start md:items-end gap-4 border-b border-border pb-4">
                    <div>
                        <div class="flex items-center space-x-3 mb-2">
                            <Button variant=ButtonVariant::Outline class="h-8 px-2".to_string() on:click=move |_| {
                                let window = web_sys::window().unwrap();
                                let _ = window.history().unwrap().back();
                            }>
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1"><path d="m15 18-6-6 6-6"/></svg>
                                "Back to Registry"
                            </Button>
                            <Badge intent=BadgeIntent::Success>"Active"</Badge>
                        </div>
                        <h2 class="text-3xl font-bold tracking-tight">"Application: " {site_id}</h2>
                        <p class="text-muted-foreground mt-1">"Manage application resources, users, and configuration."</p>
                    </div>
                    <div class="flex space-x-2">
                        <a href=move || {
                            let d = domain_bind.get();
                            if d.starts_with("http") { d } else if !d.is_empty() { format!("https://{}", d) } else { "#".to_string() }
                        } target="_blank" rel="noopener noreferrer">
                            <Button variant=ButtonVariant::Outline class="bg-background".to_string()>"View Live App"</Button>
                        </a>
                        <Button variant=ButtonVariant::Default>"App Settings"</Button>
                    </div>
                </header>
                
                <Show when=move || listings_res.get().map(|lst| lst.is_empty()).unwrap_or(false)>
                    <UpsellBanner 
                        title="Supercharge your new application!".to_string()
                        description="Jumpstart your marketplace with pre-populated leads and premium business listings."
                            .to_string()
                        cta_text="Get 100 Premium Listings - $49".to_string()
                        on_click=Callback::new(move |_| {
                            leptos::logging::log!("Upsell Clicked: Application Injection on Dashboard");
                        })
                    />
                </Show>

                <Tabs default_value="settings".to_string() class="w-full relative z-0 mt-6">
                    <TabsList class="flex w-full max-w-md mb-6 bg-muted p-1 rounded-md overflow-x-auto">
                    {move || app_manifest.get().panels.into_iter().map(|panel| {
                        view! {
                            <TabsTrigger value=panel.id.clone()>{panel.title.clone()}</TabsTrigger>
                        }
                    }).collect_view()}
                    <TabsTrigger value="domains".to_string()>"Routing & Domains"</TabsTrigger>
                </TabsList>

                {move || app_manifest.get().panels.into_iter().map(|panel| {
                    view! {
                        <TabsContent value=panel.id.clone() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                            <crate::pages::dynamic_panel::DynamicPanel panel_id=panel.id.clone() />
                        </TabsContent>
                    }
                }).collect_view()}
                <TabsContent value="domains".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <div class="space-y-6">
                        <div class="flex justify-between items-center bg-card p-6 rounded-xl border border-border shadow-sm">
                            <div>
                                <h3 class="text-lg font-medium">"Custom Hostnames"</h3>
                                <p class="text-sm text-muted-foreground">"Manage DNS routing for this application instance. Tenant traffic routes here natively."</p>
                            </div>
                            <Button variant=ButtonVariant::Default on:click=move |_| set_show_domain_modal.set(true)>
                                "Add Domain"
                            </Button>
                        </div>
                        
                        <div class="bg-card border border-border rounded-xl shadow-sm overflow-hidden">
                            <table class="w-full text-left border-collapse">
                                <thead>
                                    <tr class="bg-muted/50 border-b border-border text-xs tracking-wider uppercase text-muted-foreground">
                                        <th class="px-6 py-4 font-medium">"Domain Name"</th>
                                        <th class="px-6 py-4 font-medium">"Edge SSL Status"</th>
                                        <th class="px-6 py-4 font-medium text-right">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody class="divide-y divide-border">
                                    <Suspense fallback=move || view! { <tr><td colspan="3" class="p-6 text-center text-muted-foreground">"Loading connected routes..."</td></tr> }>
                                        {move || {
                                            match domains_res.get() {
                                                Some(domains) if domains.is_empty() => {
                                                    view! {
                                                        <tr>
                                                            <td colspan="3" class="px-6 py-8 text-center text-muted-foreground">
                                                                "No custom domains attached. Traffic uses primary wildcard via instance ID."
                                                            </td>
                                                        </tr>
                                                    }.into_any()
                                                },
                                                Some(domains) => {
                                                    domains.into_iter().map(|domain| {
                                                        let d_clone = domain.clone();
                                                        let sid = site_id().to_string();
                                                        let t = toast.clone();
                                                        view! {
                                                            <tr class="hover:bg-muted/30 transition-colors">
                                                                <td class="px-6 py-4 font-mono text-sm text-primary">
                                                                    {domain.clone()}
                                                                </td>
                                                                <td class="px-6 py-4">
                                                                    <Badge intent=BadgeIntent::Success>"Active / Managed"</Badge>
                                                                </td>
                                                                <td class="px-6 py-4 text-right">
                                                                    <button class="text-destructive hover:underline text-xs font-bold uppercase tracking-widest" >
                                                                        "DELETE"
                                                                    </button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view().into_any()
                                                },
                                                None => view! { <tr></tr> }.into_any()
                                            }
                                        }}
                                    </Suspense>
                                </tbody>
                            </table>
                        </div>
                    </div>
                </TabsContent>
            </Tabs>

            <Show when=move || show_add_listing.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_listing.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Register Business"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Add a new commercial entity to this active network."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Business Name"</Label>
                                <Input r#type=InputType::Text placeholder="e.g. Acme Corp".to_string() bind_value=RwSignal::new("".to_string()) />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_add_listing.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                toast.message.set(Some("Listing securely registered.".to_string()));
                                set_show_add_listing.set(false);
                            }>"Save Listing"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || editing_listing_name.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_editing_listing_name.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">{move || format!("Edit {}", editing_listing_name.get().unwrap_or_default())}</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Update metadata properties."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Organization Alias"</Label>
                                <Input r#type=InputType::Text bind_value=RwSignal::new(editing_listing_name.get().unwrap_or_default()) />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_editing_listing_name.set(None)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                toast.message.set(Some("Metadata updated successfully.".to_string()));
                                set_editing_listing_name.set(None);
                            }>"Apply Changes"</Button>
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || managing_user_name.get().is_some()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_managing_user_name.set(None)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">{move || format!("Manage {}", managing_user_name.get().unwrap_or_default())}</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Configure robust access and permissions."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_managing_user_name.set(None)>"Close"</Button>
                            <Button variant=ButtonVariant::Destructive on:click=move |_| {
                                toast.message.set(Some("User access rescinded.".to_string()));
                                set_managing_user_name.set(None);
                            }>"Revoke Access"</Button>
                        </div>
                    </div>
                </div>
            </Show>
            
            <Show when=move || show_add_category.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_category.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Add Category"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Define a new taxonomy level for listings."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_add_category.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                toast.message.set(Some("Category configured.".to_string()));
                                set_show_add_category.set(false);
                            }>"Save"</Button>
                        </div>
                    </div>
                </div>
            </Show>
            
            <Show when=move || show_add_template.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_template.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Assign Template"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Link a structural template to format listings here."</p>
                        <div class="flex justify-end gap-3 mt-8">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_add_template.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                toast.message.set(Some("Template assigned.".to_string()));
                                set_show_add_template.set(false);
                            }>"Save"</Button>
                        </div>
                    </div>
                </div>
            </Show>
            <Show when=move || show_domain_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_domain_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Attach Domain"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Provision a new hostname. A Cloudflare SSL certificate will be automatically requested."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Hostname (e.g. dev.buildwithruud.com)"</Label>
                                <Input r#type=InputType::Text bind_value=new_domain_input />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_domain_modal.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                let d = new_domain_input.get();
                                add_domain_action.dispatch(d);
                                set_show_domain_modal.set(false);
                            }>"Provision Pipeline"</Button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
        </Show>
    }
}
