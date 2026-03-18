use leptos::prelude::*;
use shared_ui::components::card::Card;
use shared_ui::components::ui::button::{Button, ButtonVariant};
use shared_ui::components::ui::input::{Input, InputType};
use shared_ui::components::ui::label::Label;

use crate::api::directories::get_directories;
use crate::api::crm::{get_users, get_deals};
use crate::app::GlobalToast;

#[component]
pub fn Dashboard() -> impl IntoView {
    let users_res = LocalResource::new(|| async move { get_users().await.unwrap_or_default() });
    let dirs_res = LocalResource::new(|| async move { get_directories().await.unwrap_or_default() });
    let deals_res = LocalResource::new(|| async move { get_deals().await.unwrap_or_default() });

    let active_dirs = Signal::derive(move || dirs_res.get().unwrap_or_default().len());
    let total_users = Signal::derive(move || users_res.get().unwrap_or_default().len());
    let deals_pipeline = Signal::derive(move || {
        let sum: f32 = deals_res.get().unwrap_or_default().iter().map(|d| d.amount).sum();
        format!("${:.2}", sum)
    });
    view! {
        <div class="w-full max-w-[1600px] mx-auto space-y-8 p-6">
            <header class="space-y-2">
                <h2 class="text-3xl font-bold tracking-tight">"Platform Overview"</h2>
                <p class="text-lg text-muted-foreground">"High-level metrics and global activity across all directories."</p>
            </header>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <Card class="p-6 bg-card border border-border flex flex-col justify-between space-y-4 shadow-sm".to_string()>
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-muted-foreground">"Active Directories"</span>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M3 9h18v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9Z"/><path d="m3 9 2.45-4.9A2 2 0 0 1 7.24 3h9.52a2 2 0 0 1 1.8 1.1L21 9"/><path d="M12 3v6"/></svg>
                    </div>
                    <div class="space-y-1">
                        <h3 class="text-3xl font-bold tracking-tighter">{move || active_dirs()}</h3>
                        <p class="text-xs text-muted-foreground"><span class="text-emerald-500 font-medium">"Dynamic"</span>" from API"</p>
                    </div>
                </Card>

                <Card class="p-6 bg-card border border-border flex flex-col justify-between space-y-4 shadow-sm".to_string()>
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-muted-foreground">"Total Users"</span>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>
                    </div>
                    <div class="space-y-1">
                        <h3 class="text-3xl font-bold tracking-tighter">{move || total_users()}</h3>
                        <p class="text-xs text-muted-foreground"><span class="text-emerald-500 font-medium">"Dynamic"</span>" from API"</p>
                    </div>
                </Card>

                <Card class="p-6 bg-card border border-border flex flex-col justify-between space-y-4 shadow-sm".to_string()>
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-muted-foreground">"Active Listings"</span>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><rect width="18" height="18" x="3" y="3" rx="2"/><path d="M3 9h18"/><path d="M9 21V9"/></svg>
                    </div>
                    <div class="space-y-1">
                        <h3 class="text-3xl font-bold tracking-tighter">"MVP Placeholder"</h3>
                        <p class="text-xs text-muted-foreground"><span class="text-emerald-500 font-medium">"WIP"</span>" coming soon"</p>
                    </div>
                </Card>

                <Card class="p-6 bg-card border border-border flex flex-col justify-between space-y-4 shadow-sm".to_string()>
                    <div class="flex items-center justify-between">
                        <span class="text-sm font-medium text-muted-foreground">"Deals Pipeline"</span>
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M12 2v20"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>
                    </div>
                    <div class="space-y-1">
                        <h3 class="text-3xl font-bold tracking-tighter">{move || deals_pipeline()}</h3>
                        <p class="text-xs text-muted-foreground"><span class="text-blue-500 font-medium">"Dynamic"</span>" sum mapped"</p>
                    </div>
                </Card>
            </div>

            <div class="grid grid-cols-1 xl:grid-cols-3 gap-6">
                <div class="xl:col-span-2 space-y-4">
                    <h3 class="text-xl font-semibold tracking-tight">"Recent Activity"</h3>
                    <Card class="bg-card border border-border shadow-sm".to_string()>
                        <div class="divide-y divide-border">
                            <div class="p-4 flex items-start space-x-4">
                                <div class="w-2 h-2 mt-2 rounded-full bg-blue-500"></div>
                                <div class="space-y-1 flex-1">
                                    <p class="text-sm font-medium leading-none">"New Deal Closed: Q3 Expansion"</p>
                                    <p class="text-sm text-muted-foreground">"Alice Admin closed a deal worth $150,000 for Acme Corp."</p>
                                </div>
                                <div class="text-xs text-muted-foreground whitespace-nowrap">"2h ago"</div>
                            </div>
                            <div class="p-4 flex items-start space-x-4">
                                <div class="w-2 h-2 mt-2 rounded-full bg-emerald-500"></div>
                                <div class="space-y-1 flex-1">
                                    <p class="text-sm font-medium leading-none">"New Site Deployed: Construction Directory"</p>
                                    <p class="text-sm text-muted-foreground">"System automatically provisioned resources for construction.example.com."</p>
                                </div>
                                <div class="text-xs text-muted-foreground whitespace-nowrap">"5h ago"</div>
                            </div>
                            <div class="p-4 flex items-start space-x-4">
                                <div class="w-2 h-2 mt-2 rounded-full bg-amber-500"></div>
                                <div class="space-y-1 flex-1">
                                    <p class="text-sm font-medium leading-none">"Lead Converted: Stark Industries"</p>
                                    <p class="text-sm text-muted-foreground">"Bob Agent converted Lead L-500 into an Account and created a new Deal."</p>
                                </div>
                                <div class="text-xs text-muted-foreground whitespace-nowrap">"1d ago"</div>
                            </div>
                            <div class="p-4 flex items-start space-x-4">
                                <div class="w-2 h-2 mt-2 rounded-full bg-purple-500"></div>
                                <div class="space-y-1 flex-1">
                                    <p class="text-sm font-medium leading-none">"Article Published: State of Healthcare 2026"</p>
                                    <p class="text-sm text-muted-foreground">"Charlie Editor published a new CMS article to the Healthcare Directory."</p>
                                </div>
                                <div class="text-xs text-muted-foreground whitespace-nowrap">"2d ago"</div>
                            </div>
                        </div>
                    </Card>
                </div>

                <div class="space-y-6">
                    <div class="space-y-4">
                        <h3 class="text-xl font-semibold tracking-tight">"Quick Actions"</h3>
                        <Card class="p-4 bg-card border border-border shadow-sm flex flex-col gap-2".to_string()>
                            <a href="/sites/new" class="block w-full text-left inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors bg-primary text-primary-foreground shadow hover:bg-primary/90 h-9 px-4 py-2">"Register New Site"</a>
                            <a href="/cms?tab=editor" class="block w-full text-left inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2">"Write Article"</a>
                            <a href="/crm/new" class="block w-full text-left inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors border border-input bg-background shadow-sm hover:bg-accent hover:text-accent-foreground h-9 px-4 py-2">"New Lead"</a>
                        </Card>
                    </div>

                    <div class="space-y-4">
                        <h3 class="text-xl font-semibold tracking-tight">"System Alerts"</h3>
                        <Card class="p-4 bg-red-500/10 border-red-500/20 shadow-sm".to_string()>
                            <div class="flex items-start space-x-3">
                                <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-red-500 shrink-0 mt-0.5"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
                                <div>
                                    <h4 class="text-sm font-medium text-red-500 leading-none mb-1">"Healthcare Directory Downtime"</h4>
                                    <p class="text-sm text-red-500/80">"Site marked for maintenance. Scheduled to resume at 04:00 UTC."</p>
                                </div>
                            </div>
                        </Card>
                    </div>
                </div>
            </div>
        </div>
    }
}
