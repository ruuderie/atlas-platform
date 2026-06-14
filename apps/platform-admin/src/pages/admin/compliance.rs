use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
pub struct MockPermit {
    pub key: &'static str,
    pub name: String,
    pub holder: String,
    pub license: String,
    pub permit_type: String,
    pub status: RwSignal<String>,
    pub status_class: RwSignal<&'static str>,
    pub last_checked: RwSignal<String>,
    pub date_renewed: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockGeoZone {
    pub key: String,
    pub name: String,
    pub region: String,
    pub listings: String,
    pub status: String,
    pub status_class: &'static str,
    pub coverage: String,
    pub points: String,
}

#[component]
pub fn Compliance() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Tabs state
    let active_tab = RwSignal::new("regulatory".to_string());
    
    // Municipal Permits State
    let permits = RwSignal::new(vec![
        MockPermit {
            key: "reg-1",
            name: "Chicago STR Operator License".to_string(),
            holder: "Biscayne STR Co.".to_string(),
            license: "R-2026-CH04".to_string(),
            permit_type: "Short-term Rental".to_string(),
            status: RwSignal::new("✓ Active".to_string()),
            status_class: RwSignal::new("bg-emerald-500/10 border-emerald-500/30 text-emerald-400"),
            last_checked: RwSignal::new("2 days ago".to_string()),
            date_renewed: "Jan 12, 2026".to_string(),
        },
        MockPermit {
            key: "reg-2",
            name: "Miami STR Lodging Permit".to_string(),
            holder: "Biscayne STR Co.".to_string(),
            license: "FL-MIA-8819A".to_string(),
            permit_type: "Short-term Rental".to_string(),
            status: RwSignal::new("✓ Active".to_string()),
            status_class: RwSignal::new("bg-emerald-500/10 border-emerald-500/30 text-emerald-400"),
            last_checked: RwSignal::new("Yesterday".to_string()),
            date_renewed: "May 03, 2025".to_string(),
        },
        MockPermit {
            key: "reg-3",
            name: "Condominio Legal License (BR)".to_string(),
            holder: "Rio Verde PMC".to_string(),
            license: "BR-CNPJ-04981".to_string(),
            permit_type: "Condominio Code".to_string(),
            status: RwSignal::new("⚠ In Review".to_string()),
            status_class: RwSignal::new("bg-amber-500/10 border-amber-500/30 text-amber-400"),
            last_checked: RwSignal::new("1 week ago".to_string()),
            date_renewed: "Dec 14, 2024".to_string(),
        },
    ]);

    // Active SVG Map Zones
    let geo_zones = RwSignal::new(vec![
        MockGeoZone {
            key: "chicago".to_string(),
            name: "Chicago Loop Service Area".to_string(),
            region: "Chicago, IL, USA".to_string(),
            listings: "1,240 listings".to_string(),
            status: "SRID 4326 (Valid)".to_string(),
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
            coverage: "42.8 sq km".to_string(),
            points: "60,60 140,50 160,130 90,140".to_string(),
        },
        MockGeoZone {
            key: "miami".to_string(),
            name: "Miami STR Corridor".to_string(),
            region: "Miami, FL, USA".to_string(),
            listings: "146 listings".to_string(),
            status: "SRID 4326 (Valid)".to_string(),
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
            coverage: "18.4 sq km".to_string(),
            points: "220,160 310,130 330,220 250,230".to_string(),
        },
        MockGeoZone {
            key: "rio".to_string(),
            name: "Rio de Janeiro Copacabana".to_string(),
            region: "Rio de Janeiro, Brazil".to_string(),
            listings: "324 listings".to_string(),
            status: "SRID 4326 (Valid)".to_string(),
            status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
            coverage: "8.2 sq km".to_string(),
            points: "120,180 200,210 170,270 90,240".to_string(),
        },
    ]);

    let selected_zone_key = RwSignal::new("chicago".to_string());
    
    // Bounding details for selected zone
    let selected_zone = Signal::derive(move || {
        let key = selected_zone_key.get();
        geo_zones.get().iter().find(|z| z.key == key).cloned()
    });

    // Drawing mode status
    let draw_mode_active = RwSignal::new(false);
    let draw_points = RwSignal::new(Vec::<(i32, i32)>::new());
    
    // Modal states
    let show_permit_modal = RwSignal::new(false);
    let show_save_zone_modal = RwSignal::new(false);
    let new_permit_municipality = RwSignal::new(String::new());
    let new_permit_license = RwSignal::new(String::new());
    let new_zone_name = RwSignal::new(String::new());
    let new_zone_region = RwSignal::new(String::new());

    // Action: Verify Permit Online
    let verify_permit = move |key: &'static str| {
        let items = permits.get();
        if let Some(p) = items.iter().find(|permit| permit.key == key) {
            let p_status = p.status;
            let p_class = p.status_class;
            let p_checked = p.last_checked;
            let t_toast = toast.clone();
            
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(800).await;
                p_status.set("✓ Verified".to_string());
                p_class.set("bg-emerald-500/10 border-emerald-500/30 text-emerald-400");
                p_checked.set("Just now".to_string());
                t_toast.show_toast("Success", "Permit regulatory verification check PASSED.", "success");
            });
        }
    };

    // Action: Save drawn geo zone
    let handle_save_geo_zone = move |_| {
        let name = new_zone_name.get();
        let region = new_zone_region.get();
        
        if name.trim().is_empty() || region.trim().is_empty() {
            toast.show_toast("Error", "Name and region are required.", "error");
            return;
        }

        let points_vec = draw_points.get();
        let points_str = points_vec.iter()
            .map(|(px, py)| format!("{},{}", px, py))
            .collect::<Vec<String>>()
            .join(" ");

        let key = format!("drawn_{}", name.to_lowercase().replace(" ", "_"));
        let key_select = key.clone();

        geo_zones.update(|list| {
            list.push(MockGeoZone {
                key,
                name: name.clone(),
                region: region.clone(),
                listings: "12 listings".to_string(),
                status: "SRID 4326 (Valid)".to_string(),
                status_class: "bg-emerald-500/10 border-emerald-500/30 text-emerald-400",
                coverage: "2.4 sq km".to_string(),
                points: points_str,
            });
        });

        // Highlight new zone
        selected_zone_key.set(key_select);
        
        // Reset states
        show_save_zone_modal.set(false);
        draw_mode_active.set(false);
        draw_points.set(Vec::new());
        new_zone_name.set(String::new());
        new_zone_region.set(String::new());
        
        toast.show_toast("Success", "Drawn spatial polygon saved to database context successfully.", "success");
    };

    // Handle map SVG mouse clicks
    let handle_map_click = move |ev: leptos::ev::MouseEvent| {
        if !draw_mode_active.get() { return; }
        
        // Get target element coordinate mapping offsets
        if let Some(target) = ev.current_target() {
            let svg: web_sys::Element = target.unchecked_into();
            let rect = svg.get_bounding_client_rect();
            let click_x = ev.client_x() - rect.left() as i32;
            let click_y = ev.client_y() - rect.top() as i32;
            
            // Map click to local 400x300 viewBox coordinates
            let mapped_x = (click_x as f64 * (400.0 / rect.width())) as i32;
            let mapped_y = (click_y as f64 * (300.0 / rect.height())) as i32;
            
            draw_points.update(|pts| pts.push((mapped_x, mapped_y)));
            
            if draw_points.get().len() == 4 {
                show_save_zone_modal.set(true);
            }
        }
    };

    let draw_polygon_points_str = Signal::derive(move || {
        draw_points.get().iter()
            .map(|(px, py)| format!("{},{}", px, py))
            .collect::<Vec<String>>()
            .join(" ")
    });

    view! {
        <div class="max-w-6xl mx-auto space-y-8 animate-in slide-in-from-bottom-4 duration-500 ease-out fade-in">
            // Page Header
            <header class="flex justify-between items-center bg-surface-container border border-outline-variant/10 p-6 rounded-2xl shadow-sm">
                <div>
                    <h1 class="text-3xl font-light tracking-tight text-on-surface mb-2 font-headline">"Contracts & Compliance"</h1>
                    <p class="text-on-surface-variant text-sm tracking-wide">"Track municipal registrations, regulatory listings, and active contracts"</p>
                </div>
                <div class="flex gap-3">
                    <button 
                        on:click=move |_| toast.show_toast("Info", "Running PostGIS integrity check...", "info")
                        class="px-4 py-2 text-sm font-semibold rounded-lg bg-[#05183c] border border-outline-variant/30 text-[#91aaeb] hover:bg-[#05183c]/60 active:scale-95 transition-all shadow-sm"
                    >
                        "Validate Geo Areas"
                    </button>
                    <button 
                        on:click=move |_| {
                            new_permit_municipality.set(String::new());
                            new_permit_license.set(String::new());
                            show_permit_modal.set(true);
                        }
                        class="btn-primary-gradient px-4 py-2 rounded-lg text-sm font-bold text-on-primary shadow-lg shadow-primary/20 hover:scale-105 active:scale-95 transition-all"
                    >
                        "+ New Registration"
                    </button>
                </div>
            </header>

            // KPI stats row
            <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Active Permits"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"18"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Executed Contracts"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">"34"</span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Geo zones (PostGIS)"</span>
                    <span class="text-3xl font-bold font-mono text-on-surface">
                        {move || geo_zones.get().len().to_string()}
                    </span>
                </div>
                <div class="bg-surface-container border border-outline-variant/10 p-4 rounded-xl flex flex-col gap-2">
                    <span class="text-[10px] font-bold text-on-surface-variant uppercase tracking-widest">"Expiring Contracts (30d)"</span>
                    <span class="text-3xl font-bold font-mono text-amber-500">"2"</span>
                </div>
            </div>

            // Tabs
            <div class="flex border-b border-outline-variant/15 flex-shrink-0">
                {
                    let tab_btn = move |id: &'static str, label: &'static str| {
                        let id_str = id.to_string();
                        let active_id = id_str.clone();
                        let click_id = id_str.clone();
                        view! {
                            <button 
                                class=move || if active_tab.get() == active_id { "px-4 py-2 text-sm font-semibold border-b-2 border-primary text-on-surface transition-all" } else { "px-4 py-2 text-sm text-on-surface-variant hover:text-on-surface transition-all" }
                                on:click=move |_| active_tab.set(click_id.clone())
                            >
                                {label}
                            </button>
                        }
                    };
                    view! {
                        {tab_btn("regulatory", "Regulatory Registrations (G-16)")}
                        {tab_btn("contracts", "Active Contracts (G-11)")}
                        {tab_btn("geo", "PostGIS Geo Zones (G-01)")}
                    }
                }
            </div>

            // Tab content: Regulatory Registrations
            <Show when=move || active_tab.get() == "regulatory">
                <div class="bg-surface border border-outline-variant/10 rounded-xl overflow-hidden shadow-sm">
                    <div class="px-5 py-4 bg-surface-container/30 border-b border-outline-variant/10 flex justify-between items-center">
                        <span class="font-semibold text-sm">"Municipal Permits and Certificates"</span>
                    </div>
                    <div class="overflow-x-auto w-full">
                        <table class="w-full text-left text-sm whitespace-nowrap">
                            <thead class="bg-surface-container-highest/60 text-[#91aaeb] text-xs font-medium uppercase tracking-wider">
                                <tr>
                                    <th class="px-6 py-4">"Permit Name"</th>
                                    <th class="px-6 py-4">"Holder (Account)"</th>
                                    <th class="px-6 py-4">"License / Reg ID"</th>
                                    <th class="px-6 py-4">"Type"</th>
                                    <th class="px-6 py-4">"Status"</th>
                                    <th class="px-6 py-4">"Last Checked"</th>
                                    <th class="px-6 py-4">"Date Renewed"</th>
                                    <th class="px-6 py-4"></th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                                <For 
                                    each=move || permits.get() 
                                    key=|p| p.key 
                                    children=move |p| {
                                        let pkey = p.key;
                                        view! {
                                            <tr class="hover:bg-surface-bright/5">
                                                <td class="px-6 py-4 font-bold">{p.name.clone()}</td>
                                                <td class="px-6 py-4">{p.holder.clone()}</td>
                                                <td class="px-6 py-4 font-mono text-xs text-primary">{p.license.clone()}</td>
                                                <td class="px-6 py-4 text-xs">{p.permit_type.clone()}</td>
                                                <td class="px-6 py-4">
                                                    <span class=move || format!("px-2 py-0.5 rounded text-[10px] uppercase font-bold border {}", p.status_class.get())>
                                                        {move || p.status.get()}
                                                    </span>
                                                </td>
                                                <td class="px-6 py-4 text-xs text-on-surface-variant">{move || p.last_checked.get()}</td>
                                                <td class="px-6 py-4 text-xs text-on-surface-variant">{p.date_renewed.clone()}</td>
                                                <td class="px-6 py-4 text-right">
                                                    <button 
                                                        on:click=move |_| verify_permit(pkey)
                                                        class="px-2.5 py-1 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all"
                                                    >
                                                        "Verify"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            // Tab content: Active Contracts
            <Show when=move || active_tab.get() == "contracts">
                <div class="bg-surface border border-outline-variant/10 rounded-xl overflow-hidden shadow-sm">
                    <div class="px-5 py-4 bg-surface-container/30 border-b border-outline-variant/10 flex justify-between items-center">
                        <span class="font-semibold text-sm">"Executed Contracts & SLA Documents"</span>
                    </div>
                    <div class="overflow-x-auto w-full">
                        <table class="w-full text-left text-sm whitespace-nowrap">
                            <thead class="bg-surface-container-highest/60 text-[#91aaeb] text-xs font-medium uppercase tracking-wider">
                                <tr>
                                    <th class="px-6 py-4">"Contract ID / Name"</th>
                                    <th class="px-6 py-4">"Signee Account"</th>
                                    <th class="px-6 py-4">"Type"</th>
                                    <th class="px-6 py-4">"Status"</th>
                                    <th class="px-6 py-4">"Date Executed"</th>
                                    <th class="px-6 py-4">"Expiry Date"</th>
                                    <th class="px-6 py-4">"Vault File (G-02)"</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-outline-variant/10 text-on-surface">
                                <tr class="hover:bg-surface-bright/5">
                                    <td class="px-6 py-4 font-mono text-xs">"ct_nexus_sla_2026"</td>
                                    <td class="px-6 py-4 font-semibold">"Nexus Property Group"</td>
                                    <td class="px-6 py-4 text-xs">"SLA Agreement"</td>
                                    <td class="px-6 py-4">
                                        <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold border bg-emerald-500/10 border-emerald-500/30 text-emerald-400">"Executed"</span>
                                    </td>
                                    <td class="px-6 py-4 text-xs text-on-surface-variant">"Jan 01, 2026"</td>
                                    <td class="px-6 py-4 text-xs text-on-surface-variant">"Dec 31, 2026"</td>
                                    <td class="px-6 py-4">
                                        <button on:click=move |_| toast.show_toast("Info", "Opening SLA PDF from Cloudflare R2...", "info") class="font-mono text-xs text-primary hover:underline bg-transparent border-none p-0 cursor-pointer">"nexus_sla.pdf"</button>
                                    </td>
                                </tr>
                                <tr class="hover:bg-surface-bright/5">
                                    <td class="px-6 py-4 font-mono text-xs">"ct_biscayne_str_pro"</td>
                                    <td class="px-6 py-4 font-semibold">"Biscayne STR Co."</td>
                                    <td class="px-6 py-4 text-xs">"Tenant Agreement"</td>
                                    <td class="px-6 py-4">
                                        <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold border bg-emerald-500/10 border-emerald-500/30 text-emerald-400">"Executed"</span>
                                    </td>
                                    <td class="px-6 py-4 text-xs text-on-surface-variant">"May 15, 2025"</td>
                                    <td class="px-6 py-4 text-xs text-on-surface-variant">"May 14, 2028"</td>
                                    <td class="px-6 py-4">
                                        <button on:click=move |_| toast.show_toast("Info", "Opening Agreement PDF...", "info") class="font-mono text-xs text-primary hover:underline bg-transparent border-none p-0 cursor-pointer">"biscayne_ag.pdf"</button>
                                    </td>
                                </tr>
                                <tr class="hover:bg-surface-bright/5">
                                    <td class="px-6 py-4 font-mono text-xs">"ct_harbor_media_cr"</td>
                                    <td class="px-6 py-4 font-semibold">"Harbor Media"</td>
                                    <td class="px-6 py-4 text-xs">"Tenant Agreement"</td>
                                    <td class="px-6 py-4">
                                        <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold border bg-amber-500/10 border-amber-500/30 text-amber-400">"In Review"</span>
                                    </td>
                                    <td class="px-6 py-4 text-xs text-on-surface-variant">"Draft"</td>
                                    <td class="px-6 py-4 text-xs text-on-surface-variant">"—"</td>
                                    <td class="px-6 py-4">
                                        <button on:click=move |_| toast.show_toast("Info", "Opening Draft PDF...", "info") class="font-mono text-xs text-primary hover:underline bg-transparent border-none p-0 cursor-pointer">"harbor_draft.pdf"</button>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </Show>

            // Tab content: PostGIS Geo Zones
            <Show when=move || active_tab.get() == "geo">
                <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    // Map Panel SVG canvas
                    <div class="lg:col-span-2 bg-[#090B11] border border-outline-variant/10 rounded-xl relative p-4 flex flex-col items-center justify-center min-h-[360px]">
                        <span class="absolute top-3 left-4 text-xs font-mono text-on-surface-variant">"Map Engine: PostGIS (SRID 4326)"</span>
                        
                        <div class="w-full flex-1 flex items-center justify-center cursor-crosshair">
                            <svg 
                                on:click=handle_map_click 
                                class="w-full max-h-[300px] border border-white/5 rounded bg-surface/25" 
                                viewBox="0 0 400 300"
                            >
                                <defs>
                                    <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
                                        <path d="M 20 0 L 0 0 0 20" fill="none" stroke="rgba(255,255,255,0.03)" stroke-width="1" />
                                    </pattern>
                                </defs>
                                <rect width="100%" height="100%" fill="url(#grid)" />
                                
                                // Render existing polygons
                                <For 
                                    each=move || geo_zones.get() 
                                    key=|z| z.key.clone() 
                                    children=move |z| {
                                        let key = z.key.clone();
                                        let fill = if selected_zone_key.get() == key { "rgba(10,132,255,0.35)" } else { "rgba(10,132,255,0.15)" };
                                        let stroke_width = if selected_zone_key.get() == key { "2.5" } else { "1.5" };
                                        let stroke_dash = if selected_zone_key.get() == key { "none" } else { "4" };
                                        
                                        view! {
                                            <polygon 
                                                on:click=move |e| { e.stop_propagation(); if !draw_mode_active.get_untracked() { selected_zone_key.set(key.clone()); } }
                                                points=z.points.clone()
                                                style=format!("fill: {}; stroke: #0A84FF; stroke-width: {}; stroke-dasharray: {}; cursor: pointer; transition: all 0.15s;", fill, stroke_width, stroke_dash)
                                            />
                                        }
                                    }
                                />

                                // Render live polygon drawing preview
                                <Show when=move || draw_mode_active.get() && !draw_polygon_points_str.get().is_empty()>
                                    <polygon 
                                        points=move || draw_polygon_points_str.get() 
                                        style="fill: rgba(6,150,105,0.15); stroke: #069669; stroke-width: 1.5;"
                                    />
                                </Show>
                            </svg>
                        </div>

                        // Draw mode controller
                        <div class="absolute bottom-3 right-4">
                            <button 
                                on:click=move |_| {
                                    draw_mode_active.update(|v| *v = !*v);
                                    draw_points.set(Vec::new());
                                    if draw_mode_active.get() {
                                        toast.show_toast("Info", "Click on grid nodes in map to define vertices for a new PostGIS polygon bounding box.", "info");
                                    }
                                }
                                class=move || if draw_mode_active.get() { "px-3 py-1.5 text-xs font-semibold bg-red-600/20 border border-red-500/30 text-red-400 rounded-lg hover:bg-red-600/30 transition-colors" } else { "px-3 py-1.5 text-xs bg-surface-container border border-outline-variant/30 hover:bg-surface-container-high/40 rounded transition-all" }
                            >
                                {move || if draw_mode_active.get() { "Cancel Drawing" } else { "Draw Polygon" }}
                            </button>
                        </div>
                    </div>

                    // Sidebar Detail Inspector panel
                    <div class="bg-surface-container border border-outline-variant/10 rounded-xl p-5 flex flex-col gap-5">
                        <span class="text-[10px] font-bold text-primary uppercase tracking-widest">"Selected Zone Bounds"</span>
                        {move || selected_zone.get().map(|z| {
                            view! {
                                <div class="space-y-1">
                                    <span class="text-[9px] font-semibold text-on-surface-variant uppercase tracking-wider">"Zone Code Name"</span>
                                    <p class="text-sm font-semibold text-on-surface">{z.name.clone()}</p>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-[9px] font-semibold text-on-surface-variant uppercase tracking-wider">"Coordinates Region"</span>
                                    <p class="text-sm text-on-surface">{z.region.clone()}</p>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-[9px] font-semibold text-on-surface-variant uppercase tracking-wider">"Active Listing Matches"</span>
                                    <p class="text-sm font-mono text-on-surface">{z.listings.clone()}</p>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-[9px] font-semibold text-on-surface-variant uppercase tracking-wider">"PostGIS Bounding State"</span>
                                    <div>
                                        <span class="px-2 py-0.5 rounded text-[10px] uppercase font-bold bg-emerald-500/10 border border-emerald-500/30 text-emerald-400">
                                            {z.status.clone()}
                                        </span>
                                    </div>
                                </div>
                                <div class="space-y-1">
                                    <span class="text-[9px] font-semibold text-on-surface-variant uppercase tracking-wider">"Total SLA Coverage Area"</span>
                                    <p class="text-sm font-mono text-on-surface">{z.coverage.clone()}</p>
                                </div>
                            }
                        })}
                    </div>
                </div>
            </Show>

            // Modal dialog: Create Municipal Permit
            <Show when=move || show_permit_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_permit_modal.set(false)>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Register Regulatory Permit"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Identify municipality guidelines and submit active licensing code details."</p>
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Municipality Name"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="e.g. City of Chicago STR"
                                    prop:value=new_permit_municipality
                                    on:input=move |ev| new_permit_municipality.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Registry ID / License"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="e.g. R-2026-A8B"
                                    prop:value=new_permit_license
                                    on:input=move |ev| new_permit_license.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| show_permit_modal.set(false) class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button 
                                on:click=move |_| {
                                    let mun = new_permit_municipality.get();
                                    let lic = new_permit_license.get();
                                    if mun.trim().is_empty() || lic.trim().is_empty() {
                                        toast.show_toast("Error", "All fields are required.", "error");
                                        return;
                                    }
                                    permits.update(|list| {
                                        list.push(MockPermit {
                                            key: "new-permit",
                                            name: mun,
                                            holder: "Biscayne STR Co.".to_string(),
                                            license: lic,
                                            permit_type: "Short-term Rental".to_string(),
                                            status: RwSignal::new("✓ Active".to_string()),
                                            status_class: RwSignal::new("bg-emerald-500/10 border-emerald-500/30 text-emerald-400"),
                                            last_checked: RwSignal::new("Just now".to_string()),
                                            date_renewed: "Jun 2026".to_string(),
                                        });
                                    });
                                    show_permit_modal.set(false);
                                    toast.show_toast("Success", "Regulatory permit registered successfully.", "success");
                                }
                                class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary"
                            >
                                "Upload"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>

            // Modal dialog: Save drawn geo zone
            <Show when=move || show_save_zone_modal.get()>
                <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                    <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                        <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| { show_save_zone_modal.set(false); draw_points.set(Vec::new()); draw_mode_active.set(false); }>"✕"</button>
                        <h3 class="text-xl font-semibold mb-2">"Save New PostGIS Bounding Zone"</h3>
                        <p class="text-xs text-on-surface-variant mb-6">"Identify geographic zone boundaries details."</p>
                        <div class="space-y-4 mb-6">
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Zone Code Name"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="e.g. West Loop STR Area"
                                    prop:value=new_zone_name
                                    on:input=move |ev| new_zone_name.set(event_target_value(&ev))
                                />
                            </div>
                            <div class="flex flex-col gap-1.5">
                                <label class="text-xs font-medium text-on-surface-variant">"Coverage Region"</label>
                                <input 
                                    type="text" 
                                    class="bg-surface-container-highest border border-outline/20 text-on-surface text-sm rounded-lg p-2.5 w-full focus:ring-primary focus:border-primary"
                                    placeholder="e.g. Chicago, IL"
                                    prop:value=new_zone_region
                                    on:input=move |ev| new_zone_region.set(event_target_value(&ev))
                                />
                            </div>
                        </div>
                        <div class="flex justify-end gap-3">
                            <button on:click=move |_| { show_save_zone_modal.set(false); draw_points.set(Vec::new()); draw_mode_active.set(false); } class="px-4 py-2 bg-surface-container-highest border border-outline-variant/30 rounded-lg text-xs font-bold text-on-surface">"Cancel"</button>
                            <button on:click=handle_save_geo_zone class="btn-primary-gradient px-4 py-2 rounded-lg text-xs font-bold text-on-primary">"Save Zone Bounds"</button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
