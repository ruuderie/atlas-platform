use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::switch::Switch;
use shared_ui::components::ui::button::{Button, ButtonVariant, ButtonSize};
use shared_ui::components::ui::dialog::{Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter, DialogClose, DialogAction};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;
use crate::api::models::{DirectoryModel, CreateDirectory};
use crate::api::directories::{get_directories, create_directory};

#[component]
pub fn MultiSite() -> impl IntoView {
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
    // Resource to fetch directories
    let directories = LocalResource::new(
        move || { 
            trigger_fetch.get();
            async move { get_directories().await.unwrap_or_default() }
        }
    );

    let site_name = RwSignal::new("".to_string());
    let domain = RwSignal::new("".to_string());
    let theme = RwSignal::new("default".to_string());
    let is_submitting = RwSignal::new(false);
    
    let handle_create_site = move |_| {
        is_submitting.set(true);
        let data = CreateDirectory {
            name: site_name.get(),
            domain: domain.get(),
            // Provide a dummy UUID for the directory type if not selected, or just a default
            directory_type_id: "00000000-0000-0000-0000-000000000000".to_string(),
            description: format!("Created with theme: {}", theme.get()),
        };

        leptos::task::spawn_local(async move {
            match create_directory(data).await {
                Ok(_) => {
                    // Refresh data
                    set_trigger_fetch.update(|v| *v += 1);
                    site_name.set("".to_string());
                    domain.set("".to_string());
                }
                Err(e) => {
                    leptos::logging::log!("Failed to create directory: {}", e);
                }
            }
            is_submitting.set(false);
        });
    };

    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-8 p-6">
            <header class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-8">
                <div class="space-y-2">
                    <h2 class="text-3xl font-bold tracking-tight">"Site Registry & Configuration"</h2>
                    <p class="text-muted-foreground text-lg">"Manage tenants, themes, and feature flags across the network."</p>
                </div>
                
                <Dialog>
                    <DialogTrigger class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2".to_string()>
                        "+ New Site"
                    </DialogTrigger>
                    <DialogContent class="sm:max-w-[425px]".to_string()>
                        <DialogHeader>
                            <DialogTitle>"Register New Tenant"</DialogTitle>
                            <DialogDescription>"Configure the domain and initial theme settings."</DialogDescription>
                        </DialogHeader>
                        <DialogBody>
                            <div class="grid gap-4 py-4">
                                <div class="grid grid-cols-4 items-center gap-4">
                                    <Label class="text-right".to_string()>"Site Name"</Label>
                                    <Input class="col-span-3".to_string() placeholder="e.g. Acme Corp Tenant".to_string() bind_value=site_name />
                                </div>
                                <div class="grid grid-cols-4 items-center gap-4">
                                    <Label class="text-right".to_string()>"Domain"</Label>
                                    <Input class="col-span-3".to_string() placeholder="acme.example.com".to_string() bind_value=domain />
                                </div>
                                <div class="grid grid-cols-4 items-center gap-4">
                                    <Label class="text-right".to_string()>"Theme"</Label>
                                    <select 
                                        class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 col-span-3"
                                        on:change=move |ev| theme.set(event_target_value(&ev))
                                    >
                                        <option value="default">"Default"</option>
                                        <option value="professional">"Professional"</option>
                                        <option value="dark">"Dark Mode Only"</option>
                                    </select>
                                </div>
                            </div>
                        </DialogBody>
                        <DialogFooter>
                            <DialogClose class="mt-2 sm:mt-0".to_string()>"Cancel"</DialogClose>
                            <Button on:click=handle_create_site>"Create Site"</Button>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </header>

            <Suspense fallback=move || view! { <div class="text-muted-foreground">"Loading directories..."</div> }>
                <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {move || directories.get().map(|dirs| view! {
                        <For
                            each=move || dirs.clone()
                            key=|dir: &DirectoryModel| dir.id.clone()
                            children=move |dir| {
                                view! {
                                    <Card class="p-6 bg-card border border-border flex flex-col h-full".to_string()>
                                        <div class="flex flex-col gap-3 mb-4">
                                            <div class="flex items-center justify-between">
                                                <h3 class="font-semibold text-lg leading-none tracking-tight">{dir.name.clone()}</h3>
                                                <Badge intent=if dir.site_status == "active" { BadgeIntent::Success } else { BadgeIntent::Warning }>{dir.site_status.clone()}</Badge>
                                            </div>
                                            <div class="flex items-center gap-2 flex-wrap">
                                                <a href=format!("/sites/{}", dir.id)>
                                                    <Button variant=ButtonVariant::Default size=ButtonSize::Sm>"Manage Directory"</Button>
                                                </a>
                                                <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>"Edit"</Button>
                                                <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm>"Delete"</Button>
                                            </div>
                                        </div>
                                        <div class="space-y-2 text-sm text-muted-foreground mb-6">
                                            <p><strong class="font-medium text-foreground">"Domain:"</strong> " " {dir.domain.clone()}</p>
                                            <p><strong class="font-medium text-foreground">"Theme:"</strong> " " {dir.theme.clone().unwrap_or_else(|| "default".to_string())}</p>
                                        </div>
                                        <div class="mt-auto space-y-4 pt-6 border-t border-border">
                                            <h4 class="text-sm font-medium leading-none mb-4">"Active Feature Flags (Status: " {dir.enabled_modules} ")"</h4>
                                            // In a real app we would decode the u32 bitmask and toggle real modules
                                            <div class="grid gap-3">
                                                <div class="flex items-center justify-between">
                                                    <label for=format!("t1_{}", dir.id) class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Listings Module"</label>
                                                    <Switch class="shrink-0".to_string() id=format!("t1_{}", dir.id) checked=true /> 
                                                </div>
                                                <div class="flex items-center justify-between">
                                                    <label for=format!("t2_{}", dir.id) class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"CRM Pipeline Tools"</label>
                                                    <Switch class="shrink-0".to_string() id=format!("t2_{}", dir.id) checked=true /> 
                                                </div>
                                                <div class="flex items-center justify-between">
                                                    <label for=format!("t3_{}", dir.id) class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Payments Engine"</label>
                                                    <Switch class="shrink-0".to_string() id=format!("t3_{}", dir.id) checked=false /> 
                                                </div>
                                                <div class="flex items-center justify-between">
                                                    <label for=format!("t4_{}", dir.id) class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"User Profiles"</label>
                                                    <Switch class="shrink-0".to_string() id=format!("t4_{}", dir.id) checked=true /> 
                                                </div>
                                                <div class="flex items-center justify-between">
                                                    <label for=format!("t5_{}", dir.id) class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Advanced Analytics"</label>
                                                    <Switch class="shrink-0".to_string() id=format!("t5_{}", dir.id) checked=false /> 
                                                </div>
                                            </div>
                                        </div>
                                    </Card>
                                }
                            }
                        />
                    })}
                </div>
            </Suspense>
        </div>
    }
}
