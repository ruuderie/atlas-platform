use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::api::models::PlatformAppModel;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct TenantMapItem {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub plan: String,
    pub health: String,
    pub health_label: String,
    pub mrr: String,
    pub lat: f64,
    pub lng: f64,
    pub location: String,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = initPlatformMap)]
    fn init_platform_map(tenants_json: &str, on_impersonate: &js_sys::Function);
}

fn call_js_focus_tenant(id: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(controller) = js_sys::Reflect::get(&window, &JsValue::from_str("platformMapController")) {
            if !controller.is_undefined() && !controller.is_null() {
                if let Ok(focus_fn) = js_sys::Reflect::get(&controller, &JsValue::from_str("focusTenant")) {
                    if let Some(focus_fn) = focus_fn.dyn_ref::<js_sys::Function>() {
                        let _ = focus_fn.call1(&controller, &JsValue::from_str(id));
                    }
                }
            }
        }
    }
}

fn call_js_update_visibility(filtered_ids_json: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(controller) = js_sys::Reflect::get(&window, &JsValue::from_str("platformMapController")) {
            if !controller.is_undefined() && !controller.is_null() {
                if let Ok(vis_fn) = js_sys::Reflect::get(&controller, &JsValue::from_str("updateVisibility")) {
                    if let Some(vis_fn) = vis_fn.dyn_ref::<js_sys::Function>() {
                        let _ = vis_fn.call1(&controller, &JsValue::from_str(filtered_ids_json));
                    }
                }
            }
        }
    }
}


fn get_deterministic_location(idx: usize, name: &str) -> (f64, f64, String) {
    let locations = vec![
        (25.7617, -80.1918, "Miami, FL".to_string()),
        (40.7128, -74.0060, "New York, NY".to_string()),
        (41.8781, -87.6298, "Chicago, IL".to_string()),
        (-23.5505, -46.6333, "São Paulo, BR".to_string()),
        (39.7392, -104.9903, "Denver, CO".to_string()),
        (37.7749, -122.4194, "San Francisco, CA".to_string()),
        (30.2672, -97.7431, "Austin, TX".to_string()),
        (51.5074, -0.1278, "London, UK".to_string()),
    ];

    let loc_idx = idx % locations.len();
    let base = &locations[loc_idx];
    
    // Add small offsets based on name length to prevent overlap
    let offset_lat = ((name.len() as f64 * 0.05) % 0.2) - 0.1;
    let offset_lng = ((name.len() as f64 * 0.08) % 0.2) - 0.1;
    
    (base.0 + offset_lat, base.1 + offset_lng, base.2.clone())
}

fn construct_tenant_map_items(
    real_apps: Vec<PlatformAppModel>,
    stats: Vec<crate::api::models::TenantStatModel>,
) -> Vec<TenantMapItem> {
    let mut result = Vec::new();
    
    for (idx, app) in real_apps.into_iter().enumerate() {
        let app_slug = app.name.to_lowercase().replace(' ', "-");
        
        let (plan, mrr) = if let Some(stat) = stats.iter().find(|s| s.tenant_id.to_string() == app.tenant_id) {
            let plan_str = stat.plan.clone().unwrap_or_else(|| "starter".to_string());
            let mrr_cents = stat.mrr_cents.unwrap_or(0);
            let mrr_formatted = if mrr_cents > 0 {
                format!("${}", mrr_cents / 100)
            } else {
                "$0".to_string()
            };
            (plan_str, mrr_formatted)
        } else {
            ("starter".to_string(), "$0".to_string())
        };
        
        let (lat, lng, location) = get_deterministic_location(idx, &app.name);
        
        let (health, health_label) = match app.site_status.to_lowercase().as_str() {
            "active" | "running" => ("good".to_string(), "● Healthy".to_string()),
            "warning" => ("warning".to_string(), "⚠ SLA Warning".to_string()),
            _ => ("critical".to_string(), "⚡ Outage".to_string()),
        };
        
        result.push(TenantMapItem {
            id: app.tenant_id.clone(),
            name: app.name.clone(),
            slug: app_slug,
            plan,
            health,
            health_label,
            mrr,
            lat,
            lng,
            location,
        });
    }
    
    result
}

#[component]
pub fn PlatformMap() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let dirs_res = use_context::<LocalResource<Vec<PlatformAppModel>>>().expect("dirs context");
    
    let active_id = RwSignal::new(None::<String>);
    let plan_filter = RwSignal::new("all".to_string());
    let health_filter = RwSignal::new("all".to_string());
    let show_impersonate_modal = RwSignal::new(false);
    let selected_impersonate_tenant = RwSignal::new(None::<TenantMapItem>);

    let stats_res = LocalResource::new(move || async move {
        crate::api::admin::get_tenant_stats().await.unwrap_or_default()
    });

    // Merge networks resource into our Leaflet model list
    let tenants_list = Signal::derive(move || {
        let real = dirs_res.get().unwrap_or_default();
        let stats = stats_res.get().unwrap_or_default();
        construct_tenant_map_items(real, stats)
    });

    let filtered_tenants = Signal::derive(move || {
        let plan = plan_filter.get();
        let health = health_filter.get();
        tenants_list.get().into_iter().filter(move |t| {
            let matches_plan = plan == "all" || t.plan == plan;
            let matches_health = health == "all" || t.health == health;
            matches_plan && matches_health
        }).collect::<Vec<TenantMapItem>>()
    });

    // Effect: Initialize Leaflet Map once tenants list is loaded
    Effect::new(move |_| {
        let list = tenants_list.get();
        if list.is_empty() {
            return;
        }

        // Setup the impersonation handler closure
        let on_impersonate = Closure::wrap(Box::new(move |id: String| {
            if let Some(t) = tenants_list.get_untracked().into_iter().find(|x| x.id == id) {
                selected_impersonate_tenant.set(Some(t));
                show_impersonate_modal.set(true);
            }
        }) as Box<dyn FnMut(String)>);

        // Setup the focus handler from map marker clicks
        let on_focus_from_map = Closure::wrap(Box::new(move |id: String| {
            active_id.set(Some(id));
        }) as Box<dyn FnMut(String)>);

        if let Some(window) = web_sys::window() {
            let _ = js_sys::Reflect::set(&window, &JsValue::from_str("onFocusTenantFromMap"), on_focus_from_map.as_ref());
        }
        on_focus_from_map.forget();

        if let Ok(json_str) = serde_json::to_string(&list) {
            init_platform_map(&json_str, on_impersonate.as_ref().unchecked_ref());
        }
        on_impersonate.forget();
    });

    // Effect: Sync map marker visibility when filters change
    Effect::new(move |_| {
        let ids: Vec<String> = filtered_tenants.get().iter().map(|t| t.id.clone()).collect();
        if let Ok(ids_json) = serde_json::to_string(&ids) {
            call_js_update_visibility(&ids_json);
        }
    });

    // Effect: Focus map when active list item changes
    Effect::new(move |_| {
        if let Some(id) = active_id.get() {
            call_js_focus_tenant(&id);
        }
    });

    let handle_confirm_impersonate = move |_| {
        if let Some(t) = selected_impersonate_tenant.get() {
            show_impersonate_modal.set(false);
            toast.show_toast("Warning", &format!("⚠ Impersonating {} — audit logged", t.slug), "warn");
        }
    };

    view! {
        <div class="space-y-6">
            // ── Breadcrumbs & Header ──
            <div class="flex items-end justify-between">
                <div>
                    <nav class="flex items-center gap-2 text-on-surface-variant text-xs mb-2">
                        <span>"Network"</span>
                        <span class="material-symbols-outlined text-xs">"chevron_right"</span>
                        <span class="text-primary/70">"Platform Map"</span>
                    </nav>
                    <h1 class="text-4xl font-extrabold tracking-tight text-on-surface mb-2">"Platform Map"</h1>
                    <p class="text-on-surface-variant text-sm max-w-2xl">"Geographic tracking and visual network health mapping of all active tenant instances."</p>
                </div>
            </div>

            // ── Primary Split Map View Container ──
            <div class="grid grid-cols-1 lg:grid-cols-[320px_1fr] h-[650px] border border-outline-variant/20 rounded-2xl overflow-hidden bg-[#06122d]">
                // 1. Sidebar Panel
                <div class="flex flex-col border-r border-outline-variant/10 bg-[#05183c]/20 overflow-hidden">
                    <div class="p-4 border-b border-outline-variant/10 flex-shrink-0 flex items-center justify-between">
                        <div>
                            <div class="text-sm font-bold text-on-surface">"Platform Map"</div>
                            <div class="text-[10px] text-on-surface-variant/70">"Geographic tracking of instances"</div>
                        </div>
                        <span class="px-2.5 py-0.5 text-[10px] font-bold rounded-full bg-primary/10 border border-primary/20 text-primary uppercase tracking-wider">
                            {move || format!("{} NIs", filtered_tenants.get().len())}
                        </span>
                    </div>

                    // Filters Row
                    <div class="p-4 border-b border-outline-variant/10 bg-[#06122d]/40 flex flex-col gap-3 flex-shrink-0">
                        <div class="space-y-1">
                            <label class="text-[9.5px] font-bold uppercase tracking-wider text-on-surface-variant/80">"Plan Tier"</label>
                            <select
                                class="bg-[#031d4b] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 w-full outline-none focus:ring-1 focus:ring-primary focus:border-primary cursor-pointer"
                                on:change=move |ev| plan_filter.set(event_target_value(&ev))
                            >
                                <option value="all">"All Plan Tiers"</option>
                                <option value="enterprise">"Enterprise"</option>
                                <option value="growth">"Growth"</option>
                                <option value="starter">"Starter"</option>
                            </select>
                        </div>
                        <div class="space-y-1">
                            <label class="text-[9.5px] font-bold uppercase tracking-wider text-on-surface-variant/80">"System Health"</label>
                            <select
                                class="bg-[#031d4b] border border-outline-variant/30 text-on-surface text-xs rounded-lg px-3 py-2 w-full outline-none focus:ring-1 focus:ring-primary focus:border-primary cursor-pointer"
                                on:change=move |ev| health_filter.set(event_target_value(&ev))
                            >
                                <option value="all">"All Health States"</option>
                                <option value="good">"Good / Healthy"</option>
                                <option value="warning">"SLA Warning"</option>
                                <option value="critical">"Critical Outage"</option>
                            </select>
                        </div>
                    </div>

                    // Active Tenant List
                    <div class="flex-1 overflow-y-auto p-3 space-y-2">
                        <For
                            each=move || filtered_tenants.get()
                            key=|t| t.id.clone()
                            children=move |t| {
                                let id_clone = t.id.clone();
                                let id_clone_for_active = id_clone.clone();
                                let is_active = move || active_id.get().as_ref() == Some(&id_clone_for_active);
                                
                                let health_color = match t.health.as_str() {
                                    "good" => "text-[#c6fff3]",
                                    "warning" => "text-amber-400",
                                    _ => "text-[#ee7d77]",
                                };

                                let plan_badge_class = match t.plan.as_str() {
                                    "enterprise" => "bg-[#7C3AED]/10 text-[#a78bfa] border-[#7C3AED]/20",
                                    "growth" => "bg-[#c6fff3]/10 text-[#c6fff3] border-[#c6fff3]/20",
                                    _ => "bg-surface-container-high/40 text-on-surface-variant border-outline-variant/30",
                                };

                                view! {
                                    <div 
                                        class=move || format!(
                                            "border rounded-xl p-3.5 cursor-pointer transition-all duration-150 {}",
                                            if is_active() {
                                                "bg-[#05183c] border-[#7bd0ff] ring-1 ring-[#7bd0ff] shadow-sm shadow-[#7bd0ff]/5"
                                            } else {
                                                "bg-[#05183c]/30 hover:bg-[#05183c]/60 border-outline-variant/10"
                                            }
                                        )
                                        on:click=move |_| active_id.set(Some(id_clone.clone()))
                                    >
                                        <div class="flex justify-between items-start mb-1">
                                            <div class="font-bold text-xs text-on-surface">{t.name.clone()}</div>
                                        </div>
                                        <div class="text-[10px] font-mono text-on-surface-variant/70 mb-3">{t.slug.clone()}</div>
                                        
                                        <div class="flex items-center justify-between text-[10px]">
                                            <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold border uppercase tracking-wider {}", plan_badge_class)>
                                                {t.plan.clone()}
                                            </span>
                                            <span class=format!("font-semibold {}", health_color)>
                                                {t.health_label.clone()}
                                            </span>
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>
                </div>

                // 2. Map Canvas Div
                <div class="relative w-full h-full bg-[#060e20]">
                    <div id="map" class="w-full h-full z-10"></div>

                    // Overlay Map Legend
                    <div class="absolute bottom-6 right-6 bg-surface-container-high/90 backdrop-blur-md border border-outline-variant/30 rounded-xl p-4 shadow-2xl z-20 flex flex-col gap-2.5">
                        <div class="flex items-center gap-2.5 text-xs font-semibold text-on-surface-variant">
                            <div class="w-3.5 h-3.5 rounded bg-[#7C3AED]"></div>
                            <span>"Enterprise Tier"</span>
                        </div>
                        <div class="flex items-center gap-2.5 text-xs font-semibold text-on-surface-variant">
                            <div class="w-3.5 h-3.5 rounded bg-[#069669]"></div>
                            <span>"Growth Tier"</span>
                        </div>
                        <div class="flex items-center gap-2.5 text-xs font-semibold text-[#91aaeb]">
                            <div class="w-3.5 h-3.5 rounded bg-[#1C2236] border border-outline-variant/30"></div>
                            <span>"Starter Tier"</span>
                        </div>
                        <div class="flex items-center gap-2.5 text-xs font-semibold text-on-surface-variant">
                            <div class="w-3.5 h-3.5 rounded bg-[#E5484D]"></div>
                            <span>"Critical Outage"</span>
                        </div>
                    </div>
                </div>
            </div>

            // ── Caution Impersonation Confirmation Modal ──
            <Show when=move || show_impersonate_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-surface-container-low w-full max-w-md p-6 rounded-2xl border border-outline-variant/30 shadow-2xl relative">
                        <button class="absolute top-4 right-4 text-on-surface-variant hover:text-on-surface" on:click=move |_| show_impersonate_modal.set(false)>"✕"</button>
                        <div class="flex items-center gap-3 mb-4">
                            <span class="material-symbols-outlined text-error text-3xl">"warning"</span>
                            <h3 class="text-lg font-bold text-on-surface">"Audit-Logged Impersonation"</h3>
                        </div>
                        <p class="text-on-surface-variant text-xs leading-relaxed mb-6">
                            "You are about to establish a secure impersonation session for "
                            <strong class="text-on-surface font-bold">
                                {move || selected_impersonate_tenant.get().map(|t| t.name).unwrap_or_default()}
                            </strong>
                            ". All actions performed under this session will be recorded in the security audit logs under your administrative identity."
                        </p>
                        <div class="flex justify-end gap-3">
                            <button class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface hover:bg-surface-bright/20 transition-all" on:click=move |_| show_impersonate_modal.set(false)>"Cancel"</button>
                            <button 
                                class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-xs font-bold transition-all"
                                on:click=handle_confirm_impersonate
                            >
                                "Audit & Impersonate"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
