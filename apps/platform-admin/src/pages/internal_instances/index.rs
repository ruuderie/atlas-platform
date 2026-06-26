/// # Internal Instances — Ops View
///
/// Route: /internal-instances
///
/// Shows app instances that are internally managed by the Atlas Platform team —
/// demo environments, staging, test tenants, and managed services operated on
/// behalf of a client (as distinct from client self-service instances).
///
/// NOTE: Filtering by `mode = InternalOperator` requires a backend schema change
/// to expose the `mode` field from `atlas_app_deployment_config` in the
/// `/api/admin/platform/apps` response. Until then, this page surfaces ALL
/// platform apps with an operational focus (health, provisioning, actions)
/// and documents what to add next.
use leptos::prelude::*;
use crate::api::admin::{get_all_platform_apps, suspend_instance, resume_instance};

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

#[component]
pub fn InternalInstancesPage() -> impl IntoView {
    let apps_res = LocalResource::new(move || async move {
        get_all_platform_apps().await.unwrap_or_default()
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
            </div>

            // ── Ops Context Banner ────────────────────────────────────────────
            <div class="bg-blue-500/5 border border-blue-500/20 rounded-xl px-5 py-4 flex gap-3 text-xs">
                <svg class="w-4 h-4 text-blue-400 shrink-0 mt-0.5" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
                    <circle cx="8" cy="8" r="6"/><line x1="8" y1="5" x2="8" y2="8"/><circle cx="8" cy="11" r="0.5" fill="currentColor"/>
                </svg>
                <div class="text-on-surface-variant/70 leading-relaxed">
                    <span class="font-semibold text-blue-300">"Mode filtering coming soon. "</span>
                    "Currently showing all platform instances. Once the backend exposes "
                    <span class="font-mono text-blue-300/70">"mode = internal_operator"</span>
                    " in the API response, this page will filter to internally managed deployments only. "
                    "Use the search filter or "
                    <a href="/clients" class="text-primary hover:underline">"Clients page"</a>
                    " to distinguish subscriber tenants for now."
                </div>
            </div>

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
            <Suspense fallback=|| view! {
                <div class="text-xs text-on-surface-variant/60 animate-pulse text-center py-8">"Loading instances..."</div>
            }>
                {move || {
                    let apps = apps_res.get().unwrap_or_default();
                    let q = search.get().to_lowercase();

                    let filtered: Vec<_> = apps.into_iter()
                        .filter(|a| {
                            q.is_empty()
                                || a.name.to_lowercase().contains(&q)
                                || a.domain.to_lowercase().contains(&q)
                                || a.app_type.to_lowercase().contains(&q)
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

                                                // Card footer — actions
                                                <div class="px-4 pb-4 flex items-center gap-2">
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
