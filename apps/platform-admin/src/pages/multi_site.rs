use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::badge::{Badge, BadgeIntent};
use shared_ui::components::ui::switch::Switch;
use shared_ui::components::ui::button::{Button, ButtonVariant, ButtonSize};
use shared_ui::components::ui::dialog::{Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogBody, DialogFooter, DialogClose, DialogAction};
use shared_ui::components::ui::input::Input;
use shared_ui::components::ui::label::Label;

#[component]
pub fn MultiSite() -> impl IntoView {
    view! {
        <div class="max-w-7xl mx-auto space-y-8 p-6">
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
                                    <Input class="col-span-3".to_string() placeholder="e.g. Acme Corp Tenant".to_string() />
                                </div>
                                <div class="grid grid-cols-4 items-center gap-4">
                                    <Label class="text-right".to_string()>"Domain"</Label>
                                    <Input class="col-span-3".to_string() placeholder="acme.example.com".to_string() />
                                </div>
                                <div class="grid grid-cols-4 items-center gap-4">
                                    <Label class="text-right".to_string()>"Theme"</Label>
                                    <select class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 col-span-3">
                                        <option>"Default"</option>
                                        <option>"Professional"</option>
                                        <option>"Dark Mode Only"</option>
                                    </select>
                                </div>
                            </div>
                        </DialogBody>
                        <DialogFooter>
                            <DialogClose class="mt-2 sm:mt-0".to_string()>"Cancel"</DialogClose>
                            <DialogAction>"Create Site"</DialogAction>
                        </DialogFooter>
                    </DialogContent>
                </Dialog>
            </header>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <Card class="p-6 bg-card border border-border flex flex-col h-full".to_string()>
                    <div class="flex flex-col gap-3 mb-4">
                        <div class="flex items-center justify-between">
                            <h3 class="font-semibold text-lg leading-none tracking-tight">"Transportation Directory"</h3>
                            <Badge intent=BadgeIntent::Success>"Active"</Badge>
                        </div>
                        <div class="flex items-center gap-2 flex-wrap">
                            <a href="/sites/transportation">
                                <Button variant=ButtonVariant::Default size=ButtonSize::Sm>"Manage Directory"</Button>
                            </a>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>"Edit"</Button>
                            <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm>"Delete"</Button>
                        </div>
                    </div>
                    <div class="space-y-2 text-sm text-muted-foreground mb-6">
                        <p><strong class="font-medium text-foreground">"Domain:"</strong> " transport.example.com"</p>
                        <p><strong class="font-medium text-foreground">"Theme:"</strong> " default"</p>
                    </div>
                    <div class="mt-auto space-y-4 pt-6 border-t border-border">
                        <h4 class="text-sm font-medium leading-none">"Enabled Modules"</h4>
                        <div class="grid gap-3">
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="t1".to_string() checked=true /> <label for="t1" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Listings"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="t2".to_string() checked=true /> <label for="t2" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Profiles"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="t3".to_string() checked=true /> <label for="t3" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Messaging"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="t4".to_string() checked=false /> <label for="t4" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Payments"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="t5".to_string() checked=true /> <label for="t5" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Analytics"</label>
                            </div>
                        </div>
                    </div>
                </Card>

                <Card class="p-6 bg-card border border-border flex flex-col h-full".to_string()>
                    <div class="flex flex-col gap-3 mb-4">
                        <div class="flex items-center justify-between">
                            <h3 class="font-semibold text-lg leading-none tracking-tight">"Healthcare Directory"</h3>
                            <Badge intent=BadgeIntent::Warning>"Maintenance"</Badge>
                        </div>
                        <div class="flex items-center gap-2 flex-wrap">
                            <a href="/sites/healthcare">
                                <Button variant=ButtonVariant::Default size=ButtonSize::Sm>"Manage Directory"</Button>
                            </a>
                            <Button variant=ButtonVariant::Outline size=ButtonSize::Sm>"Edit"</Button>
                            <Button variant=ButtonVariant::Destructive size=ButtonSize::Sm>"Delete"</Button>
                        </div>
                    </div>
                    <div class="space-y-2 text-sm text-muted-foreground mb-6">
                        <p><strong class="font-medium text-foreground">"Domain:"</strong> " health.example.com"</p>
                        <p><strong class="font-medium text-foreground">"Theme:"</strong> " professional"</p>
                    </div>
                    <div class="mt-auto space-y-4 pt-6 border-t border-border">
                        <h4 class="text-sm font-medium leading-none">"Enabled Modules"</h4>
                        <div class="grid gap-3">
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="h1".to_string() checked=true /> <label for="h1" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Listings"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="h2".to_string() checked=true /> <label for="h2" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Reviews"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="h3".to_string() checked=false /> <label for="h3" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Events"</label>
                            </div>
                            <div class="flex items-center space-x-2">
                                <Switch class="shrink-0".to_string() id="h4".to_string() checked=true /> <label for="h4" class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">"Custom Fields"</label>
                            </div>
                        </div>
                    </div>
                </Card>
            </div>
        </div>
    }
}


