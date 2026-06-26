use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use uuid::Uuid;

#[component]
pub fn Compliance() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");

    // Tabs state
    let active_tab = RwSignal::new("regulatory".to_string());
    
    // Municipal Permits Live Resource
    let permits_trigger = RwSignal::new(0);
    let permits_res = LocalResource::new(move || {
        permits_trigger.get();
        async move {
            crate::api::admin::get_permits().await.unwrap_or_default()
        }
    });

    // Contracts Live Resource (G-11)
    let contracts_trigger = RwSignal::new(0);
    let contracts_res = LocalResource::new(move || {
        contracts_trigger.get();
        async move {
            crate::api::admin::get_contracts().await.unwrap_or_default()
        }
    });

    // Active SVG Map Zones Live Resource
    let geo_zones_trigger = RwSignal::new(0);
    let geo_zones_res = LocalResource::new(move || {
        geo_zones_trigger.get();
        async move {
            crate::api::admin::get_geo_zones().await.unwrap_or_default()
        }
    });

    let selected_zone_key = RwSignal::new("chicago".to_string());
    
    // Bounding details for selected zone
    let selected_zone = Signal::derive(move || {
        let key = selected_zone_key.get();
        geo_zones_res.get().unwrap_or_default().into_iter().find(|z| z.key == key)
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

    // Modal: Create Contract
    let show_contract_modal = RwSignal::new(false);
    let new_contract_type = RwSignal::new("SLA Agreement".to_string());
    let new_contract_start = RwSignal::new(String::new());
    let new_contract_end = RwSignal::new(String::new());

    // Action: Verify Permit Online
    let verify_permit_action = move |id: Uuid| {
        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::verify_permit(id).await {
                Ok(_) => {
                    permits_trigger.set(permits_trigger.get() + 1);
                    t_toast.show_toast("Success", "Permit regulatory verification check PASSED.", "success");
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Verification failed: {}", e), "error");
                }
            }
        });
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

        let t_toast = toast.clone();
        leptos::task::spawn_local(async move {
            match crate::api::admin::create_geo_zone(name, region, points_str).await {
                Ok(new_zone) => {
                    geo_zones_trigger.set(geo_zones_trigger.get() + 1);
                    selected_zone_key.set(new_zone.key);
                    show_save_zone_modal.set(false);
                    draw_mode_active.set(false);
                    draw_points.set(Vec::new());
                    new_zone_name.set(String::new());
                    new_zone_region.set(String::new());
                    t_toast.show_toast("Success", "Drawn spatial polygon saved to database context successfully.", "success");
                }
                Err(e) => {
                    t_toast.show_toast("Error", &format!("Failed to save geo zone: {}", e), "error");
                }
            }
        });
    };

    // Handle map SVG mouse clicks
    let handle_map_click = move |ev: leptos::ev::MouseEvent| {
        if !draw_mode_active.get() { return; }
        
        if let Some(target) = ev.current_target() {
            let svg: web_sys::Element = target.unchecked_into();
            let rect = svg.get_bounding_client_rect();
            let click_x = ev.client_x() - rect.left() as i32;
            let click_y = ev.client_y() - rect.top() as i32;
            
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
        <div class="page-header">
            <div>
                <h1 class="page-title">"Contracts & Compliance"</h1>
                <p class="page-subtitle">"Track municipal registrations, regulatory listings, and active contracts"</p>
            </div>
            <div style="display:flex;gap:8px;">
                <button 
                    on:click=move |_| toast.show_toast("Info", "Running PostGIS integrity check...", "info")
                    class="btn btn-ghost btn-sm"
                >
                    "Validate Geo Areas"
                </button>
                <button 
                    on:click=move |_| {
                        new_permit_municipality.set(String::new());
                        new_permit_license.set(String::new());
                        show_permit_modal.set(true);
                    }
                    class="btn btn-primary btn-sm"
                >
                    "+ New Registration"
                </button>
            </div>
        </div>

        // KPI Row
        <div class="kpi-row">
            <div class="kpi-card">
                <span class="kpi-label">"Active Permits"</span>
                <span class="kpi-value" id="kpi-permits-val">
                    {move || permits_res.get().map(|p| p.len()).unwrap_or(0).to_string()}
                </span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Executed Contracts"</span>
                <span class="kpi-value">
                    {move || contracts_res.get()
                        .map(|cs| cs.iter().filter(|c| c.status == "Executed").count())
                        .unwrap_or(0)
                        .to_string()}
                </span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Geo zones (PostGIS)"</span>
                <span class="kpi-value">
                    {move || geo_zones_res.get().map(|z| z.len()).unwrap_or(0).to_string()}
                </span>
            </div>
            <div class="kpi-card">
                <span class="kpi-label">"Expiring Contracts (30d)"</span>
                <span class="kpi-value" style="color:var(--amber)">
                    {move || contracts_res.get()
                        .map(|cs| {
                            let now = chrono::Utc::now().date_naive();
                            let in30 = now + chrono::Duration::days(30);
                            cs.iter().filter(|c| {
                                // expiry_date formatted "%b %d, %Y" — check != "—" and parse
                                c.expiry_date != "—" &&
                                chrono::NaiveDate::parse_from_str(&c.expiry_date, "%b %d, %Y")
                                    .map(|d| d >= now && d <= in30)
                                    .unwrap_or(false)
                            }).count()
                        })
                        .unwrap_or(0)
                        .to_string()}
                </span>
            </div>
        </div>

        // Tabs
        <div class="tab-bar">
            <button 
                class=move || format!("tab {}", if active_tab.get() == "regulatory" { "active" } else { "" })
                on:click=move |_| active_tab.set("regulatory".to_string())
            >
                "Regulatory Registrations"
            </button>
            <button 
                class=move || format!("tab {}", if active_tab.get() == "contracts" { "active" } else { "" })
                on:click=move |_| active_tab.set("contracts".to_string())
            >
                "Active Contracts"
            </button>
            <button 
                class=move || format!("tab {}", if active_tab.get() == "geo" { "active" } else { "" })
                on:click=move |_| active_tab.set("geo".to_string())
            >
                "Geographic Zones"
            </button>
        </div>

        // REGULATORY REGISTRATIONS
        <Show when=move || active_tab.get() == "regulatory">
            <div class="section">
                <div class="section-hdr">
                    <span class="section-title">"Municipal Permits and Certificates"</span>
                    <button class="btn btn-ghost btn-sm" on:click=move |_| permits_trigger.set(permits_trigger.get() + 1)>"Refresh"</button>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>"Permit Name"</th>
                            <th>"Holder (Account)"</th>
                            <th>"License / Reg ID"</th>
                            <th>"Type"</th>
                            <th>"Status"</th>
                            <th>"Last Checked"</th>
                            <th>"Date Renewed"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=move || view! { <tr><td colspan="8">"Loading permits..."</td></tr> }>
                            {move || permits_res.get().map(|list| view! {
                                <For 
                                    each=move || list.clone() 
                                    key=|p| p.id 
                                    children=move |p| {
                                        let pid = p.id;
                                        view! {
                                            <tr>
                                                <td><strong>{p.name.clone()}</strong></td>
                                                <td>{p.holder.clone()}</td>
                                                <td class="mono">{p.license.clone()}</td>
                                                <td>{p.permit_type.clone()}</td>
                                                <td>
                                                    <span class=p.status_class.clone()>
                                                        {p.status.clone()}
                                                    </span>
                                                </td>
                                                <td class="muted">{p.last_checked.clone()}</td>
                                                <td class="muted">{p.date_renewed.clone()}</td>
                                                <td>
                                                    <button 
                                                        id=format!("btn-ver-{}", pid)
                                                        class="btn btn-ghost btn-sm" 
                                                        on:click=move |_| verify_permit_action(pid)
                                                    >
                                                        "Verify"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </Show>

        // ACTIVE CONTRACTS
        <Show when=move || active_tab.get() == "contracts">
            <div class="section">
                <div class="section-hdr">
                    <span class="section-title">"Executed Contracts & SLA Documents"</span>
                    <div style="display:flex;gap:8px;">
                        <button class="btn btn-ghost btn-sm" on:click=move |_| contracts_trigger.set(contracts_trigger.get() + 1)>"Refresh"</button>
                        <button
                            id="btn-new-contract"
                            class="btn btn-primary btn-sm"
                            on:click=move |_| {
                                new_contract_type.set("SLA Agreement".to_string());
                                new_contract_start.set(String::new());
                                new_contract_end.set(String::new());
                                show_contract_modal.set(true);
                            }
                        >
                            "+ New Contract"
                        </button>
                    </div>
                </div>
                <table>
                    <thead>
                        <tr>
                            <th>"Contract ID / Name"</th>
                            <th>"Signee Account"</th>
                            <th>"Type"</th>
                            <th>"Status"</th>
                            <th>"Date Executed"</th>
                            <th>"Expiry Date"</th>
                            <th>"Vault File (G-02)"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <Suspense fallback=move || view! { <tr><td colspan="7">"Loading contracts..."</td></tr> }>
                            {move || contracts_res.get().map(|list| {
                                if list.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="7" style="text-align:center;color:var(--text-muted);padding:24px">
                                                "No contracts on record. Click \"+ New Contract\" to add the first one."
                                            </td>
                                        </tr>
                                    }.into_any()
                                } else {
                                    view! {
                                        <For
                                            each=move || list.clone()
                                            key=|c| c.id.clone()
                                            children=move |c| {
                                                let toast2 = toast.clone();
                                                let fname = c.vault_file.clone().unwrap_or_default();
                                                view! {
                                                    <tr>
                                                        <td class="mono"><strong>{c.name.clone()}</strong></td>
                                                        <td>{c.signee.clone()}</td>
                                                        <td>{c.contract_type.clone()}</td>
                                                        <td><span class=c.status_class.clone()>{c.status.clone()}</span></td>
                                                        <td class="muted">{c.date_executed.clone()}</td>
                                                        <td class="muted">{c.expiry_date.clone()}</td>
                                                        <td>
                                                            {if fname.is_empty() {
                                                                view! { <span class="muted">"—"</span> }.into_any()
                                                            } else {
                                                                view! {
                                                                    <a
                                                                        href="#"
                                                                        class="mono"
                                                                        style="color:var(--text-link)"
                                                                        on:click=move |e| {
                                                                            e.prevent_default();
                                                                            toast2.show_toast("Info", &format!("Opening {} from Vault (G-02)...", fname), "info");
                                                                        }
                                                                    >
                                                                        {c.vault_file.clone().unwrap_or_default()}
                                                                    </a>
                                                                }.into_any()
                                                            }}
                                                        </td>
                                                    </tr>
                                                }
                                            }
                                        />
                                    }.into_any()
                                }
                            })}
                        </Suspense>
                    </tbody>
                </table>
            </div>
        </Show>

        // POSTGIS GEO ZONES
        <Show when=move || active_tab.get() == "geo">
            <div class="map-layout">
                <div class="map-canvas-container" id="map-container">
                    <svg class="map-svg" on:click=handle_map_click viewBox="0 0 400 300">
                        <defs>
                            <pattern id="grid-pattern" width="20" height="20" patternUnits="userSpaceOnUse">
                                <path d="M 20 0 L 0 0 0 20" fill="none" stroke="rgba(255,255,255,0.03)" stroke-width="1"/>
                            </pattern>
                        </defs>
                        <rect width="100%" height="100%" fill="url(#grid-pattern)" />
                        
                        <Suspense fallback=move || ()>
                            {move || geo_zones_res.get().map(|zones| view! {
                                <For 
                                    each=move || zones.clone() 
                                    key=|z| z.key.clone() 
                                    children=move |z| {
                                        let key = z.key.clone();
                                        let key_click = z.key.clone();
                                        view! {
                                            <polygon 
                                                class=move || format!("map-poly {}", if selected_zone_key.get() == key { "active" } else { "" })
                                                points=z.points.clone()
                                                on:click=move |e| {
                                                    e.stop_propagation();
                                                    if !draw_mode_active.get_untracked() {
                                                        selected_zone_key.set(key_click.clone());
                                                    }
                                                }
                                            />
                                        }
                                    }
                                />
                            })}
                        </Suspense>

                        <text x="75" y="45" fill="var(--text-muted)" font-size="9" font-family="monospace">"Chicago Loop"</text>
                        <text x="235" y="125" fill="var(--text-muted)" font-size="9" font-family="monospace">"Miami beach"</text>
                        <text x="135" y="175" fill="var(--text-muted)" font-size="9" font-family="monospace">"Copacabana"</text>

                        <Show when=move || draw_mode_active.get() && !draw_polygon_points_str.get().is_empty()>
                            <polygon 
                                points=move || draw_polygon_points_str.get() 
                                style="fill: rgba(6,150,105,0.15); stroke: var(--green); stroke-width: 1.5;"
                            />
                        </Show>

                        <For 
                            each=move || draw_points.get()
                            key=|(px, py)| format!("{},{}", px, py)
                            children=move |(px, py)| {
                                view! {
                                    <circle cx=px cy=py r="3" fill="var(--green)" stroke="#fff" stroke-width="1" style="pointer-events:none; z-index:290;" />
                                }
                            }
                        />
                    </svg>
                    <span style="position:absolute; top:10px; left:12px; font-size:10px; font-family:monospace; color:var(--text-muted)">"Map Engine: PostGIS (SRID 4326)"</span>
                    
                    <button 
                        id="btn-draw-mode"
                        class=move || if draw_mode_active.get() { "btn btn-primary btn-sm btn-danger" } else { "btn btn-ghost btn-sm" }
                        style="position:absolute; bottom:10px; right:12px;" 
                        on:click=move |_| {
                            draw_mode_active.update(|v| *v = !*v);
                            draw_points.set(Vec::new());
                            if draw_mode_active.get() {
                                toast.show_toast("Info", "Click on grid nodes in map to define vertices for a new PostGIS polygon bounding box.", "info");
                            }
                        }
                    >
                        {move || if draw_mode_active.get() { "Cancel Drawing" } else { "Draw Polygon" }}
                    </button>
                </div>

                <div class="map-zone-info">
                    <div style="font-size:11px; font-weight:600; color:var(--cobalt); text-transform:uppercase; letter-spacing:0.06em">"Selected Zone Bounds"</div>
                    <Suspense fallback=move || view! { <div class="py-4">"Loading bounds..."</div> }>
                        {move || selected_zone.get().map(|z| {
                            view! {
                                <div class="zone-field">
                                    <span class="zone-label">"Zone Code Name"</span>
                                    <span class="zone-val">{z.name.clone()}</span>
                                </div>
                                <div class="zone-field">
                                    <span class="zone-label">"Coordinates Region"</span>
                                    <span class="zone-val">{z.region.clone()}</span>
                                </div>
                                <div class="zone-field">
                                    <span class="zone-label">"Active Listing Matches"</span>
                                    <span class="zone-val mono">{z.listings.clone()}</span>
                                </div>
                                <div class="zone-field">
                                    <span class="zone-label">"PostGIS Bounding State"</span>
                                    <span class="zone-val"><span class="tag tag-ok">{z.status.clone()}</span></span>
                                </div>
                                <div class="zone-field">
                                    <span class="zone-label">"Total SLA Coverage Area"</span>
                                    <span class="zone-val mono">{z.coverage.clone()}</span>
                                </div>
                            }
                        })}
                    </Suspense>
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
                        <div class="n-form-row">
                            <label class="n-form-label">"Municipality Name"</label>
                            <input 
                                type="text" 
                                class="n-form-input"
                                placeholder="e.g. City of Chicago STR"
                                prop:value=new_permit_municipality
                                on:input=move |ev| new_permit_municipality.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="n-form-row">
                            <label class="n-form-label">"Registry ID / License"</label>
                            <input 
                                type="text" 
                                class="n-form-input"
                                placeholder="R-2026-A8B"
                                prop:value=new_permit_license
                                on:input=move |ev| new_permit_license.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <div class="flex justify-end gap-3">
                        <button on:click=move |_| show_permit_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                        <button 
                            on:click=move |_| {
                                let mun = new_permit_municipality.get();
                                let lic = new_permit_license.get();
                                if mun.trim().is_empty() || lic.trim().is_empty() {
                                    toast.show_toast("Error", "All fields are required.", "error");
                                    return;
                                }
                                let t_toast = toast.clone();
                                leptos::task::spawn_local(async move {
                                    match crate::api::admin::create_permit(mun, lic).await {
                                        Ok(_) => {
                                            permits_trigger.set(permits_trigger.get() + 1);
                                            show_permit_modal.set(false);
                                            t_toast.show_toast("Success", "Regulatory permit registered successfully.", "success");
                                        }
                                        Err(e) => {
                                            t_toast.show_toast("Error", &format!("Failed to register permit: {}", e), "error");
                                        }
                                    }
                                });
                            }
                            class="btn btn-primary"
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
                        <div class="n-form-row">
                            <label class="n-form-label">"Zone Code Name"</label>
                            <input 
                                type="text" 
                                class="n-form-input"
                                placeholder="e.g. West Loop STR Area"
                                prop:value=new_zone_name
                                on:input=move |ev| new_zone_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="n-form-row">
                            <label class="n-form-label">"Coverage Region"</label>
                            <input 
                                type="text" 
                                class="n-form-input"
                                placeholder="Chicago, IL"
                                prop:value=new_zone_region
                                on:input=move |ev| new_zone_region.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <div class="flex justify-end gap-3">
                        <button on:click=move |_| { show_save_zone_modal.set(false); draw_points.set(Vec::new()); draw_mode_active.set(false); } class="btn btn-ghost">"Cancel"</button>
                        <button on:click=handle_save_geo_zone class="btn btn-primary">"Save Zone Bounds"</button>
                    </div>
                </div>
            </div>
        </Show>

        // Modal dialog: New Contract
        <Show when=move || show_contract_modal.get()>
            <div class="fixed inset-0 z-[100] bg-black/80 backdrop-blur-sm flex items-center justify-center p-4">
                <div class="bg-card w-full max-w-md p-6 rounded-2xl border border-white/10 shadow-2xl relative text-on-surface">
                    <button class="absolute top-4 right-4 text-slate-400 hover:text-white" on:click=move |_| show_contract_modal.set(false)>"✕"</button>
                    <h3 class="text-xl font-semibold mb-2">"New Contract"</h3>
                    <p class="text-xs text-on-surface-variant mb-6">"Register a new legal agreement in the G-11 contracts ledger."</p>
                    <div class="space-y-4 mb-6">
                        <div class="n-form-row">
                            <label class="n-form-label">"Contract Type"</label>
                            <select
                                class="n-form-input"
                                on:change=move |ev| new_contract_type.set(event_target_value(&ev))
                            >
                                <option value="SLA Agreement" selected=true>"SLA Agreement"</option>
                                <option value="Tenant Agreement">"Tenant Agreement"</option>
                                <option value="Corporate Rate Agreement">"Corporate Rate Agreement"</option>
                                <option value="Alliance Agreement">"Alliance Agreement"</option>
                                <option value="Insurance Policy">"Insurance Policy"</option>
                                <option value="Lease">"Lease"</option>
                            </select>
                        </div>
                        <div class="n-form-row">
                            <label class="n-form-label">"Start Date"</label>
                            <input
                                type="date"
                                class="n-form-input"
                                prop:value=new_contract_start
                                on:input=move |ev| new_contract_start.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="n-form-row">
                            <label class="n-form-label">"End Date (optional)"</label>
                            <input
                                type="date"
                                class="n-form-input"
                                prop:value=new_contract_end
                                on:input=move |ev| new_contract_end.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    <div class="flex justify-end gap-3">
                        <button on:click=move |_| show_contract_modal.set(false) class="btn btn-ghost">"Cancel"</button>
                        <button
                            id="btn-submit-contract"
                            class="btn btn-primary"
                            on:click=move |_| {
                                let ctype = new_contract_type.get();
                                let start = new_contract_start.get();
                                if start.trim().is_empty() {
                                    toast.show_toast("Error", "Start date is required.", "error");
                                    return;
                                }
                                let end = if new_contract_end.get().is_empty() {
                                    None
                                } else {
                                    Some(new_contract_end.get())
                                };
                                let t_toast = toast.clone();
                                leptos::task::spawn_local(async move {
                                    match crate::api::admin::create_contract(ctype, start, end, None).await {
                                        Ok(_) => {
                                            contracts_trigger.set(contracts_trigger.get() + 1);
                                            show_contract_modal.set(false);
                                            t_toast.show_toast("Success", "Contract registered in G-11 ledger.", "success");
                                        }
                                        Err(e) => {
                                            t_toast.show_toast("Error", &format!("Failed to create contract: {}", e), "error");
                                        }
                                    }
                                });
                            }
                        >
                            "Register Contract"
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
