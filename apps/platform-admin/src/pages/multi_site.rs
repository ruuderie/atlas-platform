use leptos::prelude::*;
use leptos::ev;
use shared_ui::components::ui::switch::Switch;
use crate::api::models::{DirectoryModel, CreateDirectory};
use crate::api::directories::{get_directories, create_directory};

#[component]
pub fn MultiSite() -> impl IntoView {
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
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
    
    let handle_create_site = move |_: ev::MouseEvent| {
        is_submitting.set(true);
        let data = CreateDirectory {
            name: site_name.get(),
            domain: domain.get(),
            directory_type_id: "00000000-0000-0000-0000-000000000000".to_string(),
            description: format!("Created with theme: {}", theme.get()),
        };

        leptos::task::spawn_local(async move {
            match create_directory(data).await {
                Ok(_) => {
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
        <div class="space-y-8">
            // ── Header ──
            <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
                <div>
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <span>"Network"</span>
                        <span class="material-symbols-outlined text-xs">"chevron_right"</span>
                        <span class="text-primary/70">"Registry"</span>
                    </nav>
                    <h1 class="text-4xl font-extrabold tracking-tight text-on-surface mb-2">"Network Directories"</h1>
                    <p class="text-on-surface-variant text-sm max-w-2xl">"Manage multi-site orchestration, domain routing, and feature modules across the global intelligence infrastructure."</p>
                </div>
                <div class="flex items-center gap-3">
                    <div class="flex bg-surface-container-high rounded-lg p-1">
                        <button class="px-3 py-1.5 rounded-md bg-surface-bright text-primary text-xs font-bold transition-all">"GRID"</button>
                        <button class="px-3 py-1.5 rounded-md text-on-surface-variant text-xs font-bold hover:text-on-surface transition-all">"LIST"</button>
                    </div>
                    <a href="/sites/new" class="flex items-center gap-2 btn-primary-gradient text-on-primary px-5 py-2.5 rounded-md font-bold text-sm shadow-xl shadow-primary/10 hover:opacity-90 active:scale-95 transition-all">
                        <span class="material-symbols-outlined text-lg">"add_circle"</span>
                        "New Site"
                    </a>
                </div>
            </div>

            // ── Directory Grid ──
            <Suspense fallback=move || view! { <div class="text-on-surface-variant">"Loading directories..."</div> }>
                <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                    {move || directories.get().map(|dirs| view! {
                        <For
                            each=move || dirs.clone()
                            key=|dir: &DirectoryModel| dir.id.clone()
                            children=move |dir| {
                                let status = dir.site_status.clone();
                                let is_active = status == "active";
                                let status_display = status.clone();
                                let dir_id_manage = dir.id.clone();
                                
                                view! {
                                    <div class="bg-surface-container-high rounded-xl p-6 relative group border-t border-white/5 overflow-hidden">
                                        <div class="absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full -mr-16 -mt-16 blur-3xl"></div>
                                        // Header
                                        <div class="flex justify-between items-start mb-6">
                                            <div class="flex flex-col">
                                                <div class="flex items-center gap-2 mb-1">
                                                    <h3 class="text-xl font-bold text-on-surface tracking-tight">{dir.name.clone()}</h3>
                                                    {if is_active {
                                                        view! { <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-tertiary/10 text-tertiary border border-tertiary/20 uppercase tracking-wider">"Active"</span> }.into_any()
                                                    } else {
                                                        view! { <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-error-container text-error border border-error/20 uppercase tracking-wider">{status_display}</span> }.into_any()
                                                    }}
                                                </div>
                                                <span class="text-xs text-on-surface-variant font-mono">{dir.domain.clone()}</span>
                                            </div>
                                            <div class="h-10 w-10 bg-surface-container-lowest rounded-lg flex items-center justify-center border border-outline-variant/20">
                                                <span class="material-symbols-outlined text-primary-dim">"corporate_fare"</span>
                                            </div>
                                        </div>
                                        // Stats
                                        <div class="grid grid-cols-2 gap-4 mb-8">
                                            <div class="bg-surface-container-lowest/50 rounded-lg p-3">
                                                <span class="block text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Theme"</span>
                                                <span class="text-sm font-medium text-on-surface">{dir.theme.clone().unwrap_or_else(|| "Default".to_string())}</span>
                                            </div>
                                            <div class="bg-surface-container-lowest/50 rounded-lg p-3">
                                                <span class="block text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Modules"</span>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm font-medium text-on-surface">{dir.enabled_modules}</span>
                                                    <span class="text-[10px] text-tertiary font-bold">"active"</span>
                                                </div>
                                            </div>
                                        </div>
                                        // Active Modules
                                        <div class="space-y-4 mb-8">
                                            <h4 class="text-[10px] font-bold text-secondary uppercase tracking-widest border-b border-outline-variant/10 pb-2">"Active Modules"</h4>
                                            <div class="flex items-center justify-between">
                                                <div class="flex items-center gap-3">
                                                    <span class="material-symbols-outlined text-on-surface-variant text-lg">"list_alt"</span>
                                                    <span class="text-sm font-medium text-on-surface">"Listings"</span>
                                                </div>
                                                <Switch class="shrink-0".to_string() id=format!("t1_{}", dir.id) checked=true />
                                            </div>
                                            <div class="flex items-center justify-between">
                                                <div class="flex items-center gap-3">
                                                    <span class="material-symbols-outlined text-on-surface-variant text-lg">"group"</span>
                                                    <span class="text-sm font-medium text-on-surface">"CRM"</span>
                                                </div>
                                                <Switch class="shrink-0".to_string() id=format!("t2_{}", dir.id) checked=true />
                                            </div>
                                            <div class="flex items-center justify-between opacity-60">
                                                <div class="flex items-center gap-3">
                                                    <span class="material-symbols-outlined text-on-surface-variant text-lg">"payments"</span>
                                                    <span class="text-sm font-medium text-on-surface">"Payments"</span>
                                                </div>
                                                <Switch class="shrink-0".to_string() id=format!("t3_{}", dir.id) checked=false />
                                            </div>
                                        </div>
                                        // Actions
                                        <div class="flex items-center gap-2 pt-4 border-t border-outline-variant/10">
                                            <a href=format!("/sites/{}", dir_id_manage) class="flex-1 bg-surface-bright text-on-surface text-xs font-bold py-2 rounded-md hover:bg-surface-bright/80 transition-all uppercase tracking-wider text-center">"Manage"</a>
                                            <button class="p-2 bg-surface-container-lowest text-on-surface-variant hover:text-primary rounded-md border border-outline-variant/20 transition-all">
                                                <span class="material-symbols-outlined text-sm">"edit"</span>
                                            </button>
                                            <button class="p-2 bg-surface-container-lowest text-on-surface-variant hover:text-error rounded-md border border-outline-variant/20 transition-all">
                                                <span class="material-symbols-outlined text-sm">"delete"</span>
                                            </button>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    })}
                    // Empty State / Add Placeholder
                    <a href="/sites/new" class="bg-surface-container-low border-2 border-dashed border-outline-variant/20 rounded-xl flex flex-col items-center justify-center p-8 group hover:border-primary/40 transition-all cursor-pointer">
                        <div class="h-16 w-16 rounded-full bg-surface-container-high flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                            <span class="material-symbols-outlined text-3xl text-on-surface-variant group-hover:text-primary transition-colors">"add"</span>
                        </div>
                        <h3 class="text-on-surface font-bold text-lg mb-1">"Scale Network"</h3>
                        <p class="text-on-surface-variant text-sm text-center">"Provision a new directory instance within this environment."</p>
                    </a>
                </div>
            </Suspense>
        </div>
    }
}
