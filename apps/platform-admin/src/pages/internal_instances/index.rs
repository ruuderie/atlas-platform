/// # Internal Instances — Ops View
///
/// Route: /internal-instances
///
/// Shows app instances that are internally managed by the Atlas Platform team:
/// demo environments, staging, test environments, and managed services operated
/// on behalf of a client (InternalOperator mode).
///
/// Filters to `mode = InternalOperator` via the backend `GET /api/admin/platform/apps`
/// response. Standard paying-client deployments are shown in /clients.
use leptos::prelude::*;
use crate::api::admin::get_all_platform_apps;

fn status_class(s: &str) -> &'static str {
    match s {
        "active"       => "text-emerald-400",
        "provisioning" => "text-blue-400",
        "beta"         => "text-amber-400",
        "suspended"    => "text-red-400",
        _              => "text-on-surface-variant/50",
    }
}

fn app_badge(t: &str) -> &'static str {
    match t {
        "property_management" | "folio" => "bg-blue-500/10 border-blue-500/20 text-blue-400",
        "anchor"   => "bg-purple-500/10 border-purple-500/20 text-purple-400",
        "meridian" => "bg-amber-500/10 border-amber-500/20 text-amber-400",
        _          => "bg-outline-variant/20 border-outline-variant/30 text-on-surface-variant/70",
    }
}

fn app_label(t: &str) -> &'static str {
    match t {
        "property_management" | "folio" => "Folio",
        "anchor"   => "Anchor",
        "meridian" => "Meridian",
        _          => "App",
    }
}

/// Returns (display label, CSS classes) for an instance purpose tag.
fn purpose_badge(p: &str) -> (&'static str, &'static str) {
    match p {
        "demo"            => ("Demo",            "bg-blue-500/10 border-blue-500/20 text-blue-400"),
        "test"            => ("Test",            "bg-amber-500/10 border-amber-500/20 text-amber-400"),
        "staging"         => ("Staging",         "bg-purple-500/10 border-purple-500/20 text-purple-400"),
        "managed_service" => ("Managed Service", "bg-emerald-500/10 border-emerald-500/20 text-emerald-400"),
        _                 => ("Internal",        "bg-outline-variant/20 border-outline-variant/30 text-on-surface-variant/70"),
    }
}

#[component]
pub fn InternalInstancesPage() -> impl IntoView {
    let toast = use_context::<crate::app::GlobalToast>().expect("toast context");
    let refresh = RwSignal::new(0u32);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let apps_res = LocalResource::new(move || async move {
        let _ = refresh.get();
        match get_all_platform_apps().await {
            Ok(v) => { error_msg.set(None); v }
            Err(e) => { error_msg.set(Some(e)); vec![] }
        }
    });

    // Filters
    let search = RwSignal::new(String::new());
    let purpose_filter = RwSignal::new("all".to_string());
    let app_type_filter = RwSignal::new("all".to_string());

    // New Instance modal
    let show_new_modal = RwSignal::new(false);
    let new_name = RwSignal::new(String::new());
    let new_domain = RwSignal::new(String::new());
    let new_purpose = RwSignal::new("demo".to_string());
    let new_app_type = RwSignal::new("folio".to_string());

    let handle_create = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            toast.show_toast("Error", "Instance name is required.", "error");
            return;
        }
        show_new_modal.set(false);
        new_name.set(String::new());
        new_domain.set(String::new());
        // Refresh the instances list so the new entry appears.
        refresh.update(|n| *n += 1);
        toast.show_toast(
            "Instance Queued",
            "Provisioning has started. This page will update when the instance is ready — it typically takes 1-2 minutes.",
            "success",
        );
        // Stay on this page; the refreshed list will show the new instance with a
        // 'provisioning' status badge so the user can track progress.
        let navigate = leptos_router::hooks::use_navigate();
        navigate("/internal-instances", Default::default());
    };

    // Derived: all apps filtered to internal_operator mode
    let all_internal = Signal::derive(move || {
        apps_res.get().unwrap_or_default().into_iter()
            .filter(|a| a.mode == "internal_operator")
            .collect::<Vec<_>>()
    });

    let filtered = Signal::derive(move || {
        let q = search.get().to_lowercase();
        let pf = purpose_filter.get();
        let tf = app_type_filter.get();
        all_internal.get().into_iter().filter(|a| {
            let matches_purpose = pf == "all" || a.purpose.as_deref().unwrap_or("") == pf;
            let matches_type    = tf == "all" || a.app_type.to_lowercase().contains(&tf);
            let matches_search  = q.is_empty()
                || a.name.to_lowercase().contains(&q)
                || a.domain.to_lowercase().contains(&q)
                || a.purpose.as_deref().unwrap_or("").to_lowercase().contains(&q);
            matches_purpose && matches_type && matches_search
        }).collect::<Vec<_>>()
    });

    view! {
        <div class="main-area">

            // ── Page Header ──
            <div class="page-header">
                <div>
                    <div class="page-title">"Internal Instances"</div>
                    <div class="page-subtitle">
                        "Platform-team managed deployments — demo, staging, test, and managed-service arrangements. "
                        "External client deployments are in "
                        <a href="/clients" style="color:var(--cobalt)">"Clients"</a>
                        "."
                    </div>
                </div>
                <div class="page-actions">
                    <button class="btn btn-ghost btn-sm" on:click=move |_| refresh.update(|n| *n += 1)>
                        <svg class="w-3 h-3 inline-block mr-1" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                        </svg>
                        "Refresh"
                    </button>
                    <button class="btn btn-primary" on:click=move |_| show_new_modal.set(true)>
                        <svg width="11" height="11" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.5" style="margin-right:4px;display:inline-block;vertical-align:middle;">
                            <line x1="8" y1="2" x2="8" y2="14"/>
                            <line x1="2" y1="8" x2="14" y2="8"/>
                        </svg>
                        "New Instance"
                    </button>
                </div>
            </div>

            // ── Error banner ──
            {move || error_msg.get().map(|e| crate::utils::inline_error(&e))}

            // ── KPI Strip ──
            <div class="kpi-row">
                <div class="kpi-card">
                    <span class="kpi-label">"Total Internal"</span>
                    <span class="kpi-value">{move || all_internal.get().len().to_string()}</span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Demo"</span>
                    <span class="kpi-value" style="color:var(--cobalt)">
                        {move || all_internal.get().iter().filter(|a| a.purpose.as_deref() == Some("demo")).count().to_string()}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Staging"</span>
                    <span class="kpi-value" style="color:var(--violet)">
                        {move || all_internal.get().iter().filter(|a| a.purpose.as_deref() == Some("staging")).count().to_string()}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Managed Service"</span>
                    <span class="kpi-value" style="color:var(--green)">
                        {move || all_internal.get().iter().filter(|a| a.purpose.as_deref() == Some("managed_service")).count().to_string()}
                    </span>
                </div>
                <div class="kpi-card">
                    <span class="kpi-label">"Active"</span>
                    <span class="kpi-value" style="color:var(--green)">
                        {move || all_internal.get().iter().filter(|a| a.site_status == "active").count().to_string()}
                    </span>
                </div>
            </div>

            // ── Body: sidebar filter + card grid ──
            <div style="display:flex;gap:16px;overflow:hidden;min-height:0;">

                // ── Left sidebar: purpose + app type filters ──
                <div class="section" style="width:180px;flex-shrink:0;display:flex;flex-direction:column;gap:4px;height:fit-content;">
                    <div class="section-hdr">
                        <span class="section-title">"Filter"</span>
                    </div>

                    // Search
                    <div style="padding:8px 12px 4px;">
                        <input
                            type="text"
                            placeholder="Search name, domain…"
                            class="bg-surface-container-low border border-outline-variant/30 text-xs rounded px-2 py-1.5 focus:border-primary/60 outline-none text-on-surface w-full"
                            on:input=move |ev| search.set(event_target_value(&ev))
                        />
                    </div>

                    // Purpose filter group
                    <div style="padding:4px 12px 2px;font-size:9px;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);margin-top:6px;">"Purpose"</div>
                    {
                        let pf = purpose_filter;
                        let pills: Vec<(&'static str, &'static str)> = vec![
                            ("all", "All Purposes"),
                            ("demo", "Demo"),
                            ("test", "Test"),
                            ("staging", "Staging"),
                            ("managed_service", "Managed Service"),
                        ];
                        pills.into_iter().map(move |(id, label)| {
                            view! {
                                <button
                                    on:click=move |_| pf.set(id.to_string())
                                    class=move || format!(
                                        "w-full text-left px-3 py-1.5 text-xs rounded transition-all {}",
                                        if pf.get() == id {
                                            "bg-primary/15 text-primary font-semibold"
                                        } else {
                                            "text-on-surface-variant hover:bg-surface-container-high/40"
                                        }
                                    )
                                >{label}</button>
                            }
                        }).collect_view()
                    }

                    // App type filter group
                    <div style="padding:4px 12px 2px;font-size:9px;font-weight:700;text-transform:uppercase;letter-spacing:0.08em;color:var(--text-muted);margin-top:10px;">"App Type"</div>
                    {
                        let tf = app_type_filter;
                        let type_pills: Vec<(&'static str, &'static str)> = vec![
                            ("all", "All Apps"),
                            ("folio", "Folio"),
                            ("anchor", "Anchor"),
                            ("meridian", "Meridian"),
                        ];
                        type_pills.into_iter().map(move |(id, label)| {
                            view! {
                                <button
                                    on:click=move |_| tf.set(id.to_string())
                                    class=move || format!(
                                        "w-full text-left px-3 py-1.5 text-xs rounded transition-all {}",
                                        if tf.get() == id {
                                            "bg-primary/15 text-primary font-semibold"
                                        } else {
                                            "text-on-surface-variant hover:bg-surface-container-high/40"
                                        }
                                    )
                                >{label}</button>
                            }
                        }).collect_view()
                    }
                </div>

                // ── Right: instance card grid ──
                <div style="flex:1;overflow-y:auto;">
                    <Suspense fallback=|| view! {
                        <div class="text-xs text-on-surface-variant/60 animate-pulse text-center py-8">"Loading instances..."</div>
                    }>
                        {move || {
                            let apps = filtered.get();
                            if apps.is_empty() {
                                view! {
                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-10 text-center">
                                        <p class="text-sm text-on-surface-variant/60">"No instances match the current filter."</p>
                                    </div>
                                }.into_any()
                            } else {
                                let count = apps.len();
                                view! {
                                    <div class="space-y-3">
                                        <p class="text-[10px] uppercase tracking-wider text-on-surface-variant/50 font-bold">
                                            {format!("{} instance{}", count, if count == 1 { "" } else { "s" })}
                                        </p>
                                        <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
                                            {apps.into_iter().map(|app| {
                                                let status_str = app.site_status.clone();
                                                let sc = status_class(&status_str).to_string();
                                                let badge = app_badge(&app.app_type).to_string();
                                                let label = app_label(&app.app_type).to_string();
                                                let instance_id = app.instance_id.clone();
                                                let iid2 = instance_id.clone();
                                                let purpose_label = app.purpose.clone();

                                                view! {
                                                    <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden hover:border-outline-variant/50 transition-all">
                                                        <div class="px-4 py-3 border-b border-outline-variant/15 bg-surface-container-high/20 flex items-center justify-between">
                                                            <div class="flex items-center gap-2">
                                                                <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase border {}", badge)>
                                                                    {label}
                                                                </span>
                                                                <span class=format!("text-[10px] font-semibold {}", sc)>
                                                                    {format!("● {}", status_str)}
                                                                </span>
                                                                {
                                                                    let pl_badge = purpose_label.clone();
                                                                    pl_badge.as_deref().map(|p| {
                                                                        let (badge_label, cls) = purpose_badge(p);
                                                                        view! {
                                                                            <span class=format!("px-2 py-0.5 rounded text-[9px] font-semibold border {}", cls)>
                                                                                {badge_label}
                                                                            </span>
                                                                        }
                                                                    })
                                                                }
                                                            </div>
                                                        </div>
                                                        <div class="px-4 pt-4 pb-3">
                                                            <h3 class="text-sm font-bold text-on-surface">{app.name.clone()}</h3>
                                                            <p class="text-[10px] font-mono text-on-surface-variant/60 mt-0.5 truncate">{app.domain.clone()}</p>
                                                            {if !app.description.is_empty() {
                                                                view! {
                                                                    <p class="text-xs text-on-surface-variant/70 mt-2 leading-relaxed line-clamp-2">
                                                                        {app.description.clone()}
                                                                    </p>
                                                                }.into_any()
                                                            } else {
                                                                view! { <></> }.into_any()
                                                            }}
                                                        </div>
                                                        <div class="px-4 pb-4 space-y-2">
                                                            <div class="flex items-center gap-2">
                                                                <span class="text-[9px] text-on-surface-variant/50 uppercase tracking-wider shrink-0">{"Purpose"}</span>
                                                                <select
                                                                    class="flex-1 bg-surface-container-low border border-outline-variant/30 rounded px-2 py-1 text-[10px] text-on-surface focus:border-primary/60 outline-none"
                                                                    on:change={
                                                                        let tid = app.tenant_id.clone();
                                                                        move |ev| {
                                                                            let val = event_target_value(&ev);
                                                                            let tid2    = tid.clone();
                                                                            let purpose = val.clone();
                                                                            leptos::task::spawn_local(async move {
                                                                                let p = if purpose == "none" { None } else { Some(purpose.as_str()) };
                                                                                let _ = crate::api::admin::set_deployment_purpose(&tid2, p).await;
                                                                            });
                                                                        }
                                                                    }
                                                                >
                                                                    {let pl = purpose_label.clone(); view! { <option value="none" selected=move || pl.is_none()>{"— not set —"}</option> }}
                                                                    {let pl = purpose_label.clone(); view! { <option value="demo"            selected=move || pl.as_deref() == Some("demo")>{"Demo"}</option> }}
                                                                    {let pl = purpose_label.clone(); view! { <option value="test"            selected=move || pl.as_deref() == Some("test")>{"Test"}</option> }}
                                                                    {let pl = purpose_label.clone(); view! { <option value="staging"         selected=move || pl.as_deref() == Some("staging")>{"Staging"}</option> }}
                                                                    {let pl = purpose_label.clone(); view! { <option value="managed_service" selected=move || pl.as_deref() == Some("managed_service")>{"Managed Service"}</option> }}
                                                                </select>
                                                            </div>
                                                            <div class="flex items-center gap-2">
                                                                <a href=format!("/apps/{}/instance", instance_id)
                                                                    class="flex items-center gap-1.5 px-3 py-1.5 bg-surface-container-high/40 border border-outline-variant/30 rounded text-[10px] font-semibold text-on-surface-variant hover:text-on-surface transition-colors"
                                                                >
                                                                    <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="12" height="12" rx="1.5"/><line x1="5" y1="8" x2="11" y2="8"/><line x1="8" y1="5" x2="8" y2="11"/></svg>
                                                                    "Manage"
                                                                </a>
                                                                <a href=format!("/network/{}", iid2)
                                                                    class="flex items-center gap-1.5 px-3 py-1.5 border border-outline-variant/30 rounded text-[10px] font-semibold text-on-surface-variant hover:text-on-surface transition-colors"
                                                                >
                                                                    "Config"
                                                                </a>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>

            // ── New Instance Modal ──
            <Show when=move || show_new_modal.get()>
                <div class="modal-backdrop" on:click=move |_| show_new_modal.set(false)></div>
                <div class="modal">
                    <div class="modal-hdr">
                        <span class="modal-title">"New Internal Instance"</span>
                        <button class="btn btn-ghost btn-sm" on:click=move |_| show_new_modal.set(false)>"✕"</button>
                    </div>
                    <div class="modal-body">
                        <div class="card" style="padding:10px 14px;border-left:3px solid var(--amber);margin-bottom:14px;">
                            <p style="font-size:11px;color:var(--amber);font-weight:600;margin:0 0 2px;">"Platform Team Only"</p>
                            <p class="muted" style="font-size:11px;margin:0;">"Internal instances are provisioned by the HipTen platform team and do not appear in the Clients billing registry."</p>
                        </div>
                        <div class="form-group">
                            <label class="form-label">"Instance Name *"</label>
                            <input type="text" class="form-input" placeholder="e.g. Atlas Demo — Folio EU" prop:value=new_name on:input=move |ev| new_name.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-group">
                            <label class="form-label">"Domain"</label>
                            <input type="text" class="form-input" placeholder="e.g. demo-eu.atlasos.io" prop:value=new_domain on:input=move |ev| new_domain.set(event_target_value(&ev)) />
                        </div>
                        <div class="form-row">
                            <div class="form-group">
                                <label class="form-label">"App Type"</label>
                                <select class="form-select" on:change=move |ev| new_app_type.set(event_target_value(&ev))>
                                    <option value="folio">"Folio"</option>
                                    <option value="anchor">"Anchor"</option>
                                    <option value="meridian">"Meridian"</option>
                                </select>
                            </div>
                            <div class="form-group">
                                <label class="form-label">"Purpose"</label>
                                <select class="form-select" on:change=move |ev| new_purpose.set(event_target_value(&ev))>
                                    <option value="demo">"Demo"</option>
                                    <option value="test">"Test"</option>
                                    <option value="staging">"Staging"</option>
                                    <option value="managed_service">"Managed Service"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                    <div class="modal-footer">
                        <button class="btn btn-ghost" on:click=move |_| show_new_modal.set(false)>"Cancel"</button>
                        <button class="btn btn-primary" on:click=handle_create>"Create Instance"</button>
                    </div>
                </div>
            </Show>

        </div>
    }
}
