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
/// Purpose is stored in atlas_app_deployment_config.config["purpose"].
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
    let refresh = RwSignal::new(0u32);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let apps_res = LocalResource::new(move || async move {
        let _ = refresh.get();
        match get_all_platform_apps().await {
            Ok(v) => { error_msg.set(None); v }
            Err(e) => { error_msg.set(Some(e)); vec![] }
        }
    });

    let search = RwSignal::new(String::new());

    view! {
        <div class="p-8 max-w-screen-2xl mx-auto space-y-6">

            // ── Header ────────────────────────────────────────────────────────
            <div class="flex items-start justify-between flex-wrap gap-4">
                <div>
                    <h1 class="text-2xl font-extrabold text-on-surface tracking-tight">
                        "Internal Instances"
                    </h1>
                    <p class="text-sm text-on-surface-variant mt-1 max-w-xl">
                        "App instances managed by the Atlas Platform team — demo environments, "
                        "internal tooling, staging deployments, and managed-service arrangements. "
                        "External client deployments are in "
                        <a href="/clients" class="text-primary hover:underline">"Clients"</a>
                        "."
                    </p>
                </div>
                <button
                    class="btn-ghost px-3 py-2 rounded-lg text-xs font-semibold border border-outline-variant/30 flex items-center gap-1.5 hover:bg-surface-bright/20 transition-all active:scale-95 mt-1"
                    on:click=move |_| refresh.update(|n| *n += 1)
                >
                    <svg class="w-3 h-3" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8">
                        <path d="M13.5 8A5.5 5.5 0 1 1 8 2.5M13.5 2.5v3h-3"/>
                    </svg>
                    "Refresh"
                </button>
            </div>

            // ── Ops Context Banner REMOVED — mode filtering is now live ─────────
            // InternalInstancesPage now filters to mode = "internal_operator" from
            // the backend. Standard client deployments are visible in /clients.

            // ── Search ────────────────────────────────────────────────────────
            <div class="flex items-center gap-3">
                <input
                    type="text"
                    placeholder="Search instances, domains..."
                    class="bg-surface-container-low border border-outline-variant/30 text-xs rounded-lg px-3 py-2 focus:border-primary/60 outline-none transition-all text-on-surface w-64"
                    on:input=move |ev| search.set(event_target_value(&ev))
                />
            </div>

            // ── Instance Grid ─────────────────────────────────────────────────
            {move || error_msg.get().map(|e| crate::utils::inline_error(&e))}
            <Suspense fallback=|| view! {
                <div class="text-xs text-on-surface-variant/60 animate-pulse text-center py-8">"Loading instances..."</div>
            }>
                {move || {
                    let apps = apps_res.get().unwrap_or_default();
                    let q = search.get().to_lowercase();

                    // Filter to InternalOperator mode only.
                    // Standard client deployments are in /clients.
                    let filtered: Vec<_> = apps.into_iter()
                        .filter(|a| {
                            let is_internal = a.mode == "internal_operator";
                            let matches = q.is_empty()
                                || a.name.to_lowercase().contains(&q)
                                || a.domain.to_lowercase().contains(&q)
                                || a.app_type.to_lowercase().contains(&q)
                                || a.purpose.as_deref().unwrap_or("").to_lowercase().contains(&q);
                            is_internal && matches
                        })
                        .collect();

                    if filtered.is_empty() {
                        view! {
                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl p-10 text-center">
                                <p class="text-sm text-on-surface-variant/60">"No instances found."</p>
                            </div>
                        }.into_any()
                    } else {
                        let count = filtered.len();
                        view! {
                            <div class="space-y-3">
                                // Count
                                <p class="text-[10px] uppercase tracking-wider text-on-surface-variant/50 font-bold">
                                    {format!("{} instance{}", count, if count == 1 { "" } else { "s" })}
                                </p>

                                // Cards
                                <div class="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-4">
                                    {filtered.into_iter().map(|app| {
                                        let status_str = app.site_status.clone();
                                        let sc = status_class(&status_str).to_string();
                                        let badge = app_badge(&app.app_type).to_string();
                                        let label = app_label(&app.app_type).to_string();
                                        let instance_id = app.instance_id.clone();
                                        let iid2 = instance_id.clone();
                                        let purpose_label = app.purpose.clone();

                                        view! {
                                            <div class="bg-surface-container-low border border-outline-variant/20 rounded-xl overflow-hidden hover:border-outline-variant/50 transition-all">
                                                // Card header
                                                <div class="px-4 py-3 border-b border-outline-variant/15 bg-surface-container-high/20 flex items-center justify-between">
                                                    <div class="flex items-center gap-2">
                                                        <span class=format!("px-2 py-0.5 rounded text-[9px] font-bold uppercase border {}", badge)>
                                                            {label}
                                                        </span>
                                                        <span class=format!("text-[10px] font-semibold {}", sc)>
                                                            {format!("● {}", status_str)}
                                                        </span>
                                                        // Purpose badge — shown when set in config["purpose"]
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

                                                // Card body
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

                                                // Card footer — actions + purpose selector
                                                <div class="px-4 pb-4 space-y-2">
                                                    // Purpose selector
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
                                                    // Action buttons
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
    }
}
