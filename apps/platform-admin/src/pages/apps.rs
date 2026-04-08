use leptos::prelude::*;
use leptos::ev;
use shared_ui::components::ui::switch::Switch;
use crate::api::models::{PlatformAppModel, CreateNetwork};
use crate::api::networks::{get_networks, create_network};

#[component]
pub fn Apps() -> impl IntoView {
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    
    let networks = LocalResource::new(
        move || { 
            trigger_fetch.get();
            async move { get_networks().await.unwrap_or_default() }
        }
    );

    view! {
        <div class="space-y-8">
            // ── Header ──
            <div class="flex flex-col md:flex-row md:items-end justify-between gap-4">
                <div>
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <span>"Network"</span>
                        <span class="material-symbols-outlined text-xs">"chevron_right"</span>
                        <span class="text-primary/70">"Application Registry"</span>
                    </nav>
                    <h1 class="text-4xl font-extrabold tracking-tight text-on-surface mb-2">"Platform Applications"</h1>
                    <p class="text-on-surface-variant text-sm max-w-2xl">"Manage application instances, boundaries, and integrations across the multi-tenant infrastructure."</p>
                </div>
                <div class="flex items-center gap-3">
                    <div class="flex bg-surface-container-high rounded-lg p-1">
                        <button class="px-3 py-1.5 rounded-md bg-surface-bright text-primary text-xs font-bold transition-all">"GRID"</button>
                        <button class="px-3 py-1.5 rounded-md text-on-surface-variant text-xs font-bold hover:text-on-surface transition-all">"LIST"</button>
                    </div>
                    <a href="/apps/new" class="flex items-center gap-2 btn-primary-gradient text-on-primary px-5 py-2.5 rounded-md font-bold text-sm shadow-xl shadow-primary/10 hover:opacity-90 active:scale-95 transition-all">
                        <span class="material-symbols-outlined text-lg">"add_circle"</span>
                        "New Application"
                    </a>
                </div>
            </div>

            // ── Application Grid ──
            <Suspense fallback=move || view! { <div class="text-on-surface-variant">"Loading applications..."</div> }>
                <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                    {move || networks.get().map(|dirs: Vec<PlatformAppModel>| view! {
                        <For
                            each=move || dirs.clone()
                            key=|dir: &PlatformAppModel| dir.instance_id.clone()
                            children=move |dir| {
                                let status = dir.site_status.clone();
                                let is_active = status.to_lowercase() == "active";
                                let status_display = status.clone();
                                let dir_id_manage = dir.instance_id.clone();
                                let is_anchor = dir.app_type == "Services" || dir.app_type.to_lowercase() == "anchor";
                                let label_app_type = if is_anchor { "Services / Anchor" } else { "Network" };
                                
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
                                                <div class="flex items-center gap-2">
                                                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary border border-primary/20 uppercase tracking-wider">{label_app_type}</span>
                                                    <a href=format!("https://{}", dir.domain.clone()) target="_blank" rel="noopener noreferrer" class="text-xs text-on-surface-variant font-mono hover:text-primary transition-colors underline decoration-outline-variant hover:decoration-primary">{dir.domain.clone()}</a>
                                                </div>
                                            </div>
                                            <div class="h-10 w-10 bg-surface-container-lowest rounded-lg flex items-center justify-center border border-outline-variant/20">
                                                <span class="material-symbols-outlined text-primary-dim">"corporate_fare"</span>
                                            </div>
                                        </div>
                                        // Stats
                                        <div class="grid grid-cols-2 gap-4 mb-8">
                                            <div class="bg-surface-container-lowest/50 rounded-lg p-3 min-w-0 inline-block w-full">
                                                <span class="block text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"Theme"</span>
                                                <span class="block text-sm font-medium text-on-surface truncate" title="Default Theme">"Default"</span>
                                            </div>
                                            <div class="bg-surface-container-lowest/50 rounded-lg p-3">
                                                <span class="block text-[10px] font-bold text-secondary uppercase tracking-widest mb-1">"App Type"</span>
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm font-medium text-on-surface">{dir.app_type.clone()}</span>
                                                </div>
                                            </div>
                                        </div>
                                        // Active Modules
                                        <div class="space-y-4 mb-8">
                                            <h4 class="text-[10px] font-bold text-secondary uppercase tracking-widest border-b border-outline-variant/10 pb-2">"Active Modules"</h4>
                                            {if is_anchor {
                                                view! {
                                                    <div class="flex items-center justify-between">
                                                        <div class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined text-on-surface-variant text-lg">"fingerprint"</span>
                                                            <span class="text-sm font-medium text-on-surface">"Identities"</span>
                                                        </div>
                                                        <Switch class="shrink-0".to_string() id=format!("a1_{}", dir.instance_id) checked=true />
                                                    </div>
                                                    <div class="flex items-center justify-between">
                                                        <div class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined text-on-surface-variant text-lg">"room_service"</span>
                                                            <span class="text-sm font-medium text-on-surface">"Service Offerings"</span>
                                                        </div>
                                                        <Switch class="shrink-0".to_string() id=format!("a2_{}", dir.instance_id) checked=true />
                                                    </div>
                                                    <div class="flex items-center justify-between opacity-60">
                                                        <div class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined text-on-surface-variant text-lg">"content_paste"</span>
                                                            <span class="text-sm font-medium text-on-surface">"Content Matrix"</span>
                                                        </div>
                                                        <Switch class="shrink-0".to_string() id=format!("a3_{}", dir.instance_id) checked=false />
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="flex items-center justify-between">
                                                        <div class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined text-on-surface-variant text-lg">"list_alt"</span>
                                                            <span class="text-sm font-medium text-on-surface">"Listings"</span>
                                                        </div>
                                                        <Switch class="shrink-0".to_string() id=format!("t1_{}", dir.instance_id) checked=true />
                                                    </div>
                                                    <div class="flex items-center justify-between">
                                                        <div class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined text-on-surface-variant text-lg">"search"</span>
                                                            <span class="text-sm font-medium text-on-surface">"Search Provider"</span>
                                                        </div>
                                                        <Switch class="shrink-0".to_string() id=format!("t2_{}", dir.instance_id) checked=true />
                                                    </div>
                                                    <div class="flex items-center justify-between opacity-60">
                                                        <div class="flex items-center gap-3">
                                                            <span class="material-symbols-outlined text-on-surface-variant text-lg">"payments"</span>
                                                            <span class="text-sm font-medium text-on-surface">"Payments"</span>
                                                        </div>
                                                        <Switch class="shrink-0".to_string() id=format!("t3_{}", dir.instance_id) checked=false />
                                                    </div>
                                                }.into_any()
                                            }}
                                        </div>
                                        // Actions
                                        <div class="flex items-center gap-2 pt-4 border-t border-outline-variant/10">
                                            <a href=format!("/apps/{}", dir_id_manage) class="flex-1 bg-surface-bright text-on-surface text-xs font-bold py-2 rounded-md hover:bg-surface-bright/80 transition-all uppercase tracking-wider text-center">"Manage App"</a>
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
                    <a href="/apps/new" class="bg-surface-container-low border-2 border-dashed border-outline-variant/20 rounded-xl flex flex-col items-center justify-center p-8 group hover:border-primary/40 transition-all cursor-pointer">
                        <div class="h-16 w-16 rounded-full bg-surface-container-high flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                            <span class="material-symbols-outlined text-3xl text-on-surface-variant group-hover:text-primary transition-colors">"add"</span>
                        </div>
                        <h3 class="text-on-surface font-bold text-lg mb-1">"Scale Network"</h3>
                        <p class="text-on-surface-variant text-sm text-center">"Provision a new application instance within this environment."</p>
                    </a>
                </div>
            </Suspense>
        </div>
    }
}
