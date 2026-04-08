use leptos::prelude::*;
use leptos::ev;
use shared_ui::components::ui::switch::Switch;
use crate::api::models::{PlatformAppModel, CreateNetwork};
use crate::api::networks::{get_networks, create_network};

#[component]
pub fn Apps() -> impl IntoView {
    let (trigger_fetch, set_trigger_fetch) = signal(0);
    let (is_list_view, set_list_view) = signal(false);
    
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
                        <button class=move || if !is_list_view.get() { "px-3 py-1.5 rounded-md bg-surface-bright text-primary text-xs font-bold transition-all" } else { "px-3 py-1.5 rounded-md text-on-surface-variant text-xs font-bold hover:text-on-surface transition-all" } on:click=move |_| set_list_view.set(false)>"GRID"</button>
                        <button class=move || if is_list_view.get() { "px-3 py-1.5 rounded-md bg-surface-bright text-primary text-xs font-bold transition-all" } else { "px-3 py-1.5 rounded-md text-on-surface-variant text-xs font-bold hover:text-on-surface transition-all" } on:click=move |_| set_list_view.set(true)>"LIST"</button>
                    </div>
                    <a href="/apps/new" class="flex items-center gap-2 btn-primary-gradient text-on-primary px-5 py-2.5 rounded-md font-bold text-sm shadow-xl shadow-primary/10 hover:opacity-90 active:scale-95 transition-all">
                        <span class="material-symbols-outlined text-lg">"add_circle"</span>
                        "New Application"
                    </a>
                </div>
            </div>

            // ── Application Grid ──
            <Suspense fallback=move || view! { <div class="text-on-surface-variant">"Loading applications..."</div> }>
                <div class="space-y-8">
                    {move || networks.get().map(|dirs: Vec<PlatformAppModel>| {
                        let grouped_map = crate::utils::group_apps_by_tenant(dirs);
                        let grouped_vec: Vec<(String, String, Vec<PlatformAppModel>)> = grouped_map
                            .into_iter()
                            .map(|(tid, (name, apps))| (tid, name, apps))
                            .collect();
                            
                        view! {
                            <For
                                each=move || grouped_vec.clone()
                                key=|(tid, _, _)| tid.clone()
                                children=move |(tenant_id, tenant_name, apps)| {
                                    view! {
                                        <div class="bg-surface-container rounded-2xl border border-outline-variant/20 p-6 shadow-sm mb-6">
                                            <div class="flex items-center gap-3 mb-6 border-b border-outline-variant/10 pb-4">
                                                <span class="material-symbols-outlined text-primary text-2xl">"domain"</span>
                                                <h2 class="text-2xl font-bold text-on-surface tracking-tight">{tenant_name.clone()}</h2>
                                                <span class="px-2.5 py-1 text-[10px] font-bold bg-surface-container-high border border-outline-variant/30 rounded-full tracking-wider text-on-surface-variant ml-2 uppercase">"Tenant Group"</span>
                                            </div>
                                            {
                                                let apps = apps.clone();
                                                move || {
                                                    let apps_list = apps.clone();
                                                    let apps_grid = apps.clone();
                                                    if is_list_view.get() {
                                                view! {
                                                    <div class="overflow-x-auto">
                                                        <table class="w-full text-left border-collapse">
                                                            <thead>
                                                                <tr class="text-xs uppercase tracking-wider text-on-surface-variant border-b border-outline-variant/10">
                                                                    <th class="pb-3 px-4 font-medium">"Application"</th>
                                                                    <th class="pb-3 px-4 font-medium">"Type"</th>
                                                                    <th class="pb-3 px-4 font-medium">"Domain"</th>
                                                                    <th class="pb-3 px-4 font-medium">"Status"</th>
                                                                    <th class="pb-3 px-4 font-medium text-right">"Action"</th>
                                                                </tr>
                                                            </thead>
                                                            <tbody>
                                                                <For
                                                                    each=move || apps_list.clone()
                                                                    key=|dir: &PlatformAppModel| dir.instance_id.clone()
                                                                    children=move |dir| {
                                                                        let is_active = dir.site_status.to_lowercase() == "active";
                                                                        let dir_id_manage = dir.instance_id.clone();
                                                                        view! {
                                                                            <tr class="border-b border-outline-variant/5 hover:bg-surface-container-high/50 transition-colors">
                                                                                <td class="py-3 px-4">
                                                                                    <div class="font-bold text-on-surface text-sm">{dir.name.clone()}</div>
                                                                                </td>
                                                                                <td class="py-3 px-4">
                                                                                    <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-primary/10 text-primary uppercase tracking-wider">{dir.app_type.clone()}</span>
                                                                                </td>
                                                                                <td class="py-3 px-4">
                                                                                    <a href=format!("https://{}", dir.domain.clone()) target="_blank" class="text-xs text-on-surface-variant font-mono hover:text-primary transition-colors underline decoration-outline-variant">{dir.domain.clone()}</a>
                                                                                </td>
                                                                                <td class="py-3 px-4">
                                                                                    {if is_active {
                                                                                        view! { <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-tertiary/10 text-tertiary uppercase tracking-wider">"Active"</span> }.into_any()
                                                                                    } else {
                                                                                        view! { <span class="px-2 py-0.5 rounded text-[10px] font-bold bg-error-container text-error uppercase tracking-wider">{dir.site_status.clone()}</span> }.into_any()
                                                                                    }}
                                                                                </td>
                                                                                <td class="py-3 px-4 text-right">
                                                                                    <a href=format!("/apps/{}", dir_id_manage) class="inline-flex items-center gap-1 bg-surface-bright text-on-surface text-[10px] font-bold py-1.5 px-3 rounded-md hover:bg-surface-bright/80 transition-all uppercase tracking-wider">
                                                                                        <span class="material-symbols-outlined text-[14px]">"edit"</span>
                                                                                        "Manage"
                                                                                    </a>
                                                                                </td>
                                                                            </tr>
                                                                        }
                                                                    }
                                                                />
                                                            </tbody>
                                                        </table>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
                                                        <For
                                                            each=move || apps_grid.clone()
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
                                                                            <a href=format!("/apps/{}", dir_id_manage) class="block p-2 bg-surface-container-lowest text-on-surface-variant hover:text-primary rounded-md border border-outline-variant/20 transition-all">
                                                                                <span class="material-symbols-outlined text-sm">"edit"</span>
                                                                            </a>
                                                                            <button class="p-2 bg-surface-container-lowest text-on-surface-variant hover:text-error rounded-md border border-outline-variant/20 transition-all">
                                                                                <span class="material-symbols-outlined text-sm">"delete"</span>
                                                                            </button>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                        <a href="/apps/new" class="bg-surface-container-low border-2 border-dashed border-outline-variant/20 rounded-xl flex flex-col items-center justify-center p-8 min-h-[300px] group hover:border-primary/40 transition-all cursor-pointer">
                                                            <div class="h-16 w-16 rounded-full bg-surface-container-high flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                                                                <span class="material-symbols-outlined text-3xl text-on-surface-variant group-hover:text-primary transition-colors">"add"</span>
                                                            </div>
                                                            <h3 class="text-on-surface font-bold text-lg mb-1">"Scale Tenant Network"</h3>
                                                            <p class="text-on-surface-variant text-sm text-center">"Provision a new application instance within this environment."</p>
                                                        </a>
                                                    </div>
                                                }.into_any()
                                            }
                                            }
                                            }
                                        </div>
                                    }
                                }
                            />
                        }
                    })}
                </div>
            </Suspense>
        </div>
    }
}
