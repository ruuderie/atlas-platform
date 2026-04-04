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
    let (show_invite, set_show_invite) = signal(false);
    let (show_add_category, set_show_add_category) = signal(false);
    let (show_add_template, set_show_add_template) = signal(false);
    
    let (editing_listing_name, set_editing_listing_name) = signal(None::<String>);
    let (managing_user_name, set_managing_user_name) = signal(None::<String>);

    let dirs = use_context::<LocalResource<Vec<crate::api::models::DirectoryModel>>>().expect("dirs context");
    let domain_bind = RwSignal::new(String::new());
    
    Effect::new(move |_| {
        let current_id = site_id();
        if let Some(d) = dirs.get() {
            if let Some(dir) = d.into_iter().find(|dir| dir.id.to_string() == current_id) {
                domain_bind.set(dir.domain.clone());
            } else {
                domain_bind.set(format!("{}.example.com", current_id));
            }
        }
    });
    
    let invite_email = RwSignal::new("".to_string());
    let setup_token_bind = RwSignal::new("".to_string());

    let handle_save_domain = move |_| {
        toast.message.set(Some("Domain configuration updated dynamically.".to_string()));
    };

    let handle_invite = move |_| {
        if !invite_email.get().is_empty() {
            toast.message.set(Some(format!("Invited {} to collaborate.", invite_email.get())));
            invite_email.set("".to_string());
        }
    };

    let site_id_str = site_id().to_string();
    let listings_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move { crate::api::listings::get_listings(&sid).await.unwrap_or_default() }
        }
    });

    let setup_token_res = LocalResource::new({
        let sid = site_id_str.clone();
        move || {
            let sid = sid.clone();
            async move {
                if let Ok(setting) = crate::api::directories::get_tenant_setting(&sid, "setup_token").await {
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

    let handle_generate_token = {
        let toast = toast.clone();
        move |_| {
            let token_str = uuid::Uuid::new_v4().to_string().replace("-", "")[..12].to_uppercase();
            let sid = site_id().to_string();
            let toast = toast.clone();
            leptos::task::spawn_local(async move {
                let req = crate::api::models::UpsertSettingRequest {
                    key: "setup_token".to_string(),
                    value: token_str.clone(),
                    is_encrypted: false,
                };
                if crate::api::directories::upsert_tenant_setting(&sid, req).await.is_ok() {
                    setup_token_bind.set(token_str);
                    toast.message.set(Some("Setup token strategically regenerated.".to_string()));
                } else {
                    toast.message.set(Some("Failed to generate token.".to_string()));
                }
            });
        }
    };

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

    let mock_profiles = vec![
        ("usr_8821", "Alice Admin", "alice@example.com", "Site Admin"),
        ("usr_3194", "Bob Driver", "bob@example.com", "Contributor"),
        ("usr_5561", "Charlie Dispatch", "charlie@example.com", "Editor"),
    ];

    let mock_categories = vec![
        ("C-10", "Auto & Transport", "Active"),
        ("C-11", "Home Services", "Active"),
        ("C-12", "Professional Services", "Active"),
    ];
    
    let mock_templates = vec![
        ("T-01", "Standard Business", "v1.2", "Active"),
        ("T-02", "Premium Listing", "v2.0", "Active"),
    ];

    let app_manifest = Signal::derive(move || {
        let current_id = site_id();
        let app_type_str = if let Some(d) = dirs.get() {
            if let Some(dir) = d.into_iter().find(|dir| dir.id.to_string() == current_id) {
                dir.directory_type_id.clone()
            } else {
                "directory".to_string()
            }
        } else {
            "directory".to_string()
        };
        crate::components::app_manifest::get_manifest_for_app_type(&app_type_str)
    });

    view! {
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
                </TabsList>

                <TabsContent value="listings".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <RelatedList
                        title="Business Listings".to_string()
                        description="Businesses registered in this specific directory network.".to_string()
                        icon="store".to_string()
                        action_label="Add Listing".to_string()
                        on_action=Callback::new(move |_| set_show_add_listing.set(true))
                        count=4
                    >
                        <DataTable class="w-full text-sm">
                            <DataTableHeader class="bg-muted/50 border-b border-border">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Listing ID"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Business Name"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Category"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border">
                                <Suspense fallback=move || view! { <div class="p-4 text-sm text-muted-foreground">"Loading listings..."</div> }>
                                    {move || listings_res.get().map(|listings| view! {
                                        <For each=move || listings.clone() key=|l| l.id.clone() children=move |l| {
                                            let badge_intent = match l.status {
                                                crate::api::models::ListingStatus::Active => BadgeIntent::Success,
                                                crate::api::models::ListingStatus::Pending => BadgeIntent::Warning,
                                                _ => BadgeIntent::Default,
                                            };
                                            let title1 = l.title.clone();
                                            let title2 = l.title.clone();
                                            view! {
                                                <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                                    <DataTableCell class="p-4 align-middle font-medium">{l.id.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">{title1}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-muted-foreground">{l.category_id.clone().unwrap_or_else(|| "Uncategorized".into())}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">
                                                        <Badge intent=badge_intent>{format!("{:?}", l.status)}</Badge>
                                                    </DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right">
                                                        <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string() on:click=move |_| set_editing_listing_name.set(Some(title2.clone()))>"Edit"</Button>
                                                    </DataTableCell>
                                                </DataTableRow>
                                            }
                                        }/>
                                    })}
                                </Suspense>
                            </DataTableBody>
                        </DataTable>
                    </RelatedList>
                </TabsContent>

                <TabsContent value="profiles".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <RelatedList
                        title="Directory Profiles".to_string()
                        description="Users who have registered accounts specifically within this site.".to_string()
                        icon="group".to_string()
                        action_label="Invite User".to_string()
                        on_action=Callback::new(move |_| set_show_invite.set(true))
                        count=3
                    >
                        <DataTable class="w-full text-sm">
                            <DataTableHeader class="bg-muted/50 border-b border-border">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"User ID"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Email"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Role"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border">
                                <Suspense fallback=move || view! { <div class="p-4 text-sm text-muted-foreground">"Loading profiles..."</div> }>
                                    {move || profiles_res.get().map(|profiles| view! {
                                        <For each=move || profiles.clone() key=|p| p.id.clone() children=move |p| {
                                            let role_badge = if p.is_admin { BadgeIntent::Error } else { BadgeIntent::Default };
                                            let role_str = if p.is_admin { "Admin" } else { "User" };
                                            let p_name1 = p.username.clone();
                                            let p_name2 = p.username.clone();
                                            view! {
                                                <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                                    <DataTableCell class="p-4 align-middle font-medium">{p.id.to_string()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">{p_name1}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-muted-foreground">{p.email.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">
                                                        <Badge intent=role_badge>{role_str.to_string()}</Badge>
                                                    </DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right">
                                                        <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string() on:click=move |_| set_managing_user_name.set(Some(p_name2.clone()))>"Manage"</Button>
                                                    </DataTableCell>
                                                </DataTableRow>
                                            }
                                        }/>
                                    })}
                                </Suspense>
                            </DataTableBody>
                        </DataTable>
                    </RelatedList>
                </TabsContent>
                
                <TabsContent value="categories".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <RelatedList
                        title="Directory Categories".to_string()
                        description="Categories mapped specifically to structure this directory's listings.".to_string()
                        icon="category".to_string()
                        action_label="Add Category".to_string()
                        on_action=Callback::new(move |_| set_show_add_category.set(true))
                        count=3
                    >
                        <DataTable class="w-full text-sm">
                            <DataTableHeader class="bg-muted/50 border-b border-border">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Category ID"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border">
                                <Suspense fallback=move || view! { <div class="p-4 text-sm text-muted-foreground">"Loading categories..."</div> }>
                                    {move || categories_res.get().map(|categories| view! {
                                        <For each=move || categories.clone() key=|c| c.id.clone() children=move |c| {
                                            view! {
                                                <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                                    <DataTableCell class="p-4 align-middle font-medium">{c.id.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">{c.name.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">
                                                        <Badge intent=if c.is_active { BadgeIntent::Success } else { BadgeIntent::Warning }>{if c.is_active { "Active" } else { "Inactive" }.to_string()}</Badge>
                                                    </DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right">
                                                        <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Edit"</Button>
                                                    </DataTableCell>
                                                </DataTableRow>
                                            }
                                        }/>
                                    })}
                                </Suspense>
                            </DataTableBody>
                        </DataTable>
                    </RelatedList>
                </TabsContent>
                
                <TabsContent value="templates".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <RelatedList
                        title="Assigned Templates".to_string()
                        description="Templates orchestrating the data structure for listings in this network.".to_string()
                        icon="draw".to_string()
                        action_label="Assign Template".to_string()
                        on_action=Callback::new(move |_| set_show_add_template.set(true))
                        count=2
                    >
                        <DataTable class="w-full text-sm">
                            <DataTableHeader class="bg-muted/50 border-b border-border">
                                <DataTableRow class="hover:bg-transparent">
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Template ID"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Name"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Version"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-left align-middle font-medium text-muted-foreground">"Status"</DataTableHead>
                                    <DataTableHead class="h-10 px-4 text-right align-middle font-medium text-muted-foreground">"Actions"</DataTableHead>
                                </DataTableRow>
                            </DataTableHeader>
                            <DataTableBody class="divide-y divide-border">
                                <Suspense fallback=move || view! { <div class="p-4 text-sm text-muted-foreground">"Loading templates..."</div> }>
                                    {move || templates_res.get().map(|templates| view! {
                                        <For each=move || templates.clone() key=|t| t.id.clone() children=move |t| {
                                            view! {
                                                <DataTableRow class="transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
                                                    <DataTableCell class="p-4 align-middle font-medium">{t.id.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">{t.name.clone()}</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-muted-foreground">"v1.0"</DataTableCell>
                                                    <DataTableCell class="p-4 align-middle">
                                                        <Badge intent=if t.is_active { BadgeIntent::Success } else { BadgeIntent::Warning }>{if t.is_active { "Active" } else { "Inactive" }.to_string()}</Badge>
                                                    </DataTableCell>
                                                    <DataTableCell class="p-4 align-middle text-right">
                                                        <Button variant=ButtonVariant::Ghost class="h-8 px-2 text-primary".to_string()>"Manage"</Button>
                                                    </DataTableCell>
                                                </DataTableRow>
                                            }
                                        }/>
                                    })}
                                </Suspense>
                            </DataTableBody>
                        </DataTable>
                    </RelatedList>
                </TabsContent>

                <TabsContent value="settings".to_string() class="mt-0 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2">
                    <Card class="bg-card border-border shadow-sm p-6".to_string()>
                        <h3 class="text-lg font-semibold mb-4">"Application Configuration"</h3>
                        <div class="space-y-4 max-w-lg">
                            <div class="space-y-2">
                                <label class="text-sm font-medium leading-none">"Custom Domain"</label>
                                <div class="flex items-center space-x-2">
                                    <Input r#type=InputType::Text class="w-full".to_string() bind_value=domain_bind placeholder="www.example.com".to_string() />
                                    <Button variant=ButtonVariant::Default on:click=handle_save_domain>"Save"</Button>
                                </div>
                                <p class="text-xs text-muted-foreground">"Configure DNS CNAME record pointing to proxy.foundry.local"</p>
                            </div>
                            
                            <div class="space-y-2 mt-6">
                                <label class="text-sm font-medium leading-none">"Tenant Setup Token"</label>
                                <div class="flex items-center space-x-2">
                                    <Input r#type=InputType::Text class="w-full text-mono font-bold".to_string() bind_value=setup_token_bind disabled=true />
                                    <Button variant=ButtonVariant::Outline on:click=handle_generate_token>"Regenerate"</Button>
                                </div>
                                <p class="text-xs text-muted-foreground">"One-time setup token required by the application administrator during initial initialization."</p>
                            </div>
                            <div class="pt-4 border-t border-border mt-6">
                                <Button variant=ButtonVariant::Destructive>"Deactivate Application"</Button>
                            </div>
                        </div>
                    </Card>
                    <Show when=move || app_manifest.get().app_type_id == "anchor_app">
                        <crate::components::anchor_settings::AnchorSettingsPanel />
                    </Show>
                </TabsContent>
            </Tabs>

            <Show when=move || show_add_listing.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_add_listing.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Register Business"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Add a new commercial entity to this active directory."</p>
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

            <Show when=move || show_invite.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| set_show_invite.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2 text-foreground">"Invite Team Member"</h3>
                        <p class="text-muted-foreground text-sm mb-6">"Send an invitation email to grant access."</p>
                        <div class="space-y-4 mb-6">
                            <div class="grid gap-2 text-left">
                                <Label>"Email Address"</Label>
                                <Input r#type=InputType::Email placeholder="user@example.com".to_string() bind_value=invite_email />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <Button variant=ButtonVariant::Outline on:click=move |_| set_show_invite.set(false)>"Cancel"</Button>
                            <Button variant=ButtonVariant::Default on:click=move |_| {
                                handle_invite(());
                                set_show_invite.set(false);
                            }>"Send Invite"</Button>
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
        </div>
    }
}
