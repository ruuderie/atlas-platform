use crate::api::system_status::{
    EnvironmentId, EnvironmentStatusNode, FleetBlock, HealthStatus, IncidentSeverity,
    SystemStatusResponse, get_system_status,
};
use leptos::prelude::*;

fn status_label(s: &HealthStatus) -> &'static str {
    match s {
        HealthStatus::Healthy => "healthy",
        HealthStatus::Degraded => "degraded",
        HealthStatus::Down => "down",
        HealthStatus::Unknown => "unknown",
    }
}

fn status_color(s: &HealthStatus) -> &'static str {
    match s {
        HealthStatus::Healthy => "var(--green, #1b7f4e)",
        HealthStatus::Degraded => "var(--amber, #b86e00)",
        HealthStatus::Down => "var(--red, #c0392b)",
        HealthStatus::Unknown => "var(--text-muted)",
    }
}

fn format_bytes(bytes: Option<i64>) -> String {
    match bytes {
        Some(b) if b >= 1_073_741_824 => format!("{:.1} GiB", b as f64 / 1_073_741_824.0),
        Some(b) if b >= 1_048_576 => format!("{:.1} MiB", b as f64 / 1_048_576.0),
        Some(b) if b >= 1024 => format!("{:.1} KiB", b as f64 / 1024.0),
        Some(b) => format!("{b} B"),
        None => "—".into(),
    }
}

fn error_rate(env: &EnvironmentStatusNode) -> String {
    let req = env
        .telemetry
        .counters
        .iter()
        .find(|c| c.name == "http_requests_total")
        .map(|c| c.value);
    let err = env
        .telemetry
        .counters
        .iter()
        .find(|c| c.name == "http_request_errors_total")
        .map(|c| c.value);
    match (req, err) {
        (Some(r), Some(e)) if r > 0.0 => format!("{:.3}%", (e / r) * 100.0),
        _ => "—".into(),
    }
}

fn probe_counts(env: &EnvironmentStatusNode) -> (usize, usize) {
    let total = env.platform_services.len();
    let ok = env
        .platform_services
        .iter()
        .filter(|p| p.status == HealthStatus::Healthy)
        .count();
    (ok, total)
}

#[component]
pub fn SystemStatusPage() -> impl IntoView {
    let active_tab = RwSignal::new("overview".to_string());
    let selected_env = RwSignal::new(EnvironmentId::Development);
    let refresh = RwSignal::new(0u32);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let snapshot: RwSignal<Option<SystemStatusResponse>> = RwSignal::new(None);
    let loading = RwSignal::new(true);

    let status_res = LocalResource::new(move || {
        let _ = refresh.get();
        async move {
            loading.set(true);
            let res = get_system_status().await;
            loading.set(false);
            match res {
                Ok(s) => {
                    error.set(None);
                    if let Some(first) = s.environments.first() {
                        let current = selected_env.get_untracked();
                        if !s.environments.iter().any(|e| e.id == current) {
                            selected_env.set(first.id);
                        }
                    }
                    snapshot.set(Some(s.clone()));
                    Some(s)
                }
                Err(e) => {
                    error.set(Some(e));
                    None
                }
            }
        }
    });

    Effect::new(move |_| {
        leptos::task::spawn_local(async move {
            loop {
                gloo_timers::future::TimeoutFuture::new(3000).await;
                refresh.update(|n| *n = n.wrapping_add(1));
            }
        });
    });

    view! {
        <div class="main-canvas">
            <div class="page-header">
                <div>
                    <h1 class="page-title">"System Status"</h1>
                    <p class="page-subtitle">
                        "Fleet health by environment. Each rollup is independent — Dev downtime never paints Production red."
                    </p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-ghost btn-sm"
                        style="transition: transform 140ms cubic-bezier(0.23, 1, 0.32, 1);"
                        on:click=move |_| refresh.update(|n| *n = n.wrapping_add(1))
                    >
                        {move || if loading.get() { "Refreshing…" } else { "Refresh" }}
                    </button>
                </div>
            </div>

            <Show when=move || error.get().is_some()>
                <div
                    class="mb-4 rounded-lg border px-4 py-3 text-sm"
                    style="border-color: color-mix(in srgb, var(--red, #c0392b) 35%, transparent); background: color-mix(in srgb, var(--red, #c0392b) 8%, transparent);"
                >
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            <div style="display:none">{move || status_res.get().map(|_| ())}</div>

            <Show when=move || snapshot.get().is_some() fallback=move || view! {
                <div class="text-sm" style="color:var(--text-muted)">"Loading system status…"</div>
            }>
                {move || snapshot.get().map(|s| view! {
                    <StatusBody
                        status=s
                        active_tab=active_tab
                        selected_env=selected_env
                    />
                })}
            </Show>
        </div>
    }
}

#[component]
fn StatusBody(
    status: SystemStatusResponse,
    active_tab: RwSignal<String>,
    selected_env: RwSignal<EnvironmentId>,
) -> impl IntoView {
    let envs_store = StoredValue::new(status.environments.clone());
    let fleet_store = StoredValue::new(status.fleet.clone());
    let collected = status.collected_at.clone();
    let envs_for_cards = status.environments.clone();
    let envs_for_versions = status.environments;

    view! {
        <div
            class="mb-4 grid gap-2.5"
            style="grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));"
        >
            {envs_for_cards.into_iter().map(|env| {
                let id = env.id;
                let overall = env.overall_status.clone();
                let label = env.label.clone();
                let version = env.version.clone();
                let (ok, total) = probe_counts(&env);
                let tenant_n = env.tenants.len();
                view! {
                    <button
                        type="button"
                        class="rounded-lg border px-3.5 py-3 text-left"
                        style=move || {
                            let active = selected_env.get() == id;
                            if active {
                                "border-color: var(--cobalt, #2563eb); background: color-mix(in srgb, var(--cobalt, #2563eb) 8%, transparent); cursor:pointer; font:inherit; color:inherit;"
                            } else {
                                "border-color: var(--outline-variant, #ddd); background: var(--surface-container-low, transparent); cursor:pointer; font:inherit; color:inherit;"
                            }
                        }
                        on:click=move |_| selected_env.set(id)
                    >
                        <div class="flex items-center justify-between gap-2 mb-1">
                            <span class="text-sm font-semibold">{label.clone()}</span>
                            <span
                                class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-wide px-2 py-0.5 rounded border"
                                style=format!(
                                    "color:{}; border-color:{};",
                                    status_color(&overall),
                                    status_color(&overall)
                                )
                            >
                                <span
                                    class="inline-block h-2 w-2 rounded-full"
                                    style=format!("background:{}", status_color(&overall))
                                ></span>
                                {status_label(&overall)}
                            </span>
                        </div>
                        <div class="text-[11px] font-mono mb-2" style="color:var(--text-muted)">
                            {format!(
                                "v{}+{}",
                                version.version,
                                version.build_sha.chars().take(7).collect::<String>()
                            )}
                        </div>
                        <div class="flex flex-wrap gap-x-3 gap-y-1 text-[11px]" style="color:var(--text-secondary, var(--text-muted))">
                            <span>{format!("{ok}/{total} probes ok")}</span>
                            <span>{format!("{tenant_n} tenants")}</span>
                        </div>
                    </button>
                }
            }).collect_view()}
        </div>

        {move || {
            let id = selected_env.get();
            let env = envs_store.with_value(|envs| {
                envs.iter()
                    .find(|e| e.id == id)
                    .cloned()
                    .or_else(|| envs.first().cloned())
            });
            env.map(|env| {
                let overall = env.overall_status.clone();
                let label = env.label.clone();
                let version = env.version.clone();
                let collected_env = env.collected_at.clone();
                let (ok, total) = probe_counts(&env);
                let err = error_rate(&env);
                let is_dev = matches!(env.id, EnvironmentId::Development);
                view! {
                    <div
                        class="mb-4 flex flex-wrap items-center gap-4 rounded-lg border px-4 py-3"
                        style="border-color: var(--outline-variant, #ddd); background: var(--surface-container-low, transparent);"
                    >
                        <div class="flex items-center gap-2">
                            <span
                                class="inline-block h-2.5 w-2.5 rounded-full"
                                style=format!("background:{}", status_color(&overall))
                            ></span>
                            <span class="text-sm font-semibold" style=format!("color:{}", status_color(&overall))>
                                {format!("{} · {}", label, status_label(&overall))}
                            </span>
                        </div>
                        <div class="text-xs font-mono" style="color:var(--text-muted)">
                            {format!("probes {ok}/{total} healthy")}
                        </div>
                        <div class="text-xs font-mono" style="color:var(--text-muted)">
                            {format!("err rate {err}")}
                        </div>
                        <div class="text-xs font-mono" style="color:var(--text-muted)">
                            {format!(
                                "v{}+{}",
                                version.version,
                                version.build_sha.chars().take(7).collect::<String>()
                            )}
                        </div>
                        <div class="text-xs" style="color:var(--text-muted); margin-left:auto">
                            {format!("collected {collected_env}")}
                        </div>
                    </div>

                    <Show when=move || is_dev>
                        <div
                            class="mb-4 rounded-lg border px-4 py-3 text-sm"
                            style="border-color: color-mix(in srgb, var(--cobalt, #2563eb) 30%, transparent); background: color-mix(in srgb, var(--cobalt, #2563eb) 6%, transparent);"
                        >
                            "Full Compose/Docker view stays on the host: run "
                            <code class="font-mono text-xs">"atlas-local status"</code>
                            " (parity stack). This page is deploy-safe telemetry only."
                        </div>
                    </Show>
                }
            })
        }}

        <div class="tab-bar mb-4">
            <button
                class=move || if active_tab.get() == "overview" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("overview".into())
            >"Overview"</button>
            <button
                class=move || if active_tab.get() == "resources" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("resources".into())
            >"Capacity"</button>
            <button
                class=move || if active_tab.get() == "telemetry" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("telemetry".into())
            >"Telemetry"</button>
            <button
                class=move || if active_tab.get() == "versions" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("versions".into())
            >"Versions"</button>
        </div>

        <div style="transition: opacity 160ms cubic-bezier(0.23, 1, 0.32, 1);">
            {move || {
                let tab = active_tab.get();
                let id = selected_env.get();
                let env = envs_store.with_value(|envs| {
                    envs.iter()
                        .find(|e| e.id == id)
                        .cloned()
                        .or_else(|| envs.first().cloned())
                });
                match (tab.as_str(), env) {
                    ("overview", Some(env)) => view! { <OverviewTab env=env /> }.into_any(),
                    ("resources", Some(env)) => {
                        let fleet = fleet_store.with_value(|f| f.clone());
                        view! { <CapacityTab fleet=fleet env=env /> }.into_any()
                    }
                    ("telemetry", Some(env)) => view! { <TelemetryTab env=env /> }.into_any(),
                    ("versions", _) => view! {
                        <VersionsTab envs=envs_for_versions.clone() collected=collected.clone() />
                    }
                    .into_any(),
                    _ => view! { <div></div> }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
fn OverviewTab(env: EnvironmentStatusNode) -> impl IntoView {
    let services = env.platform_services.clone();
    let backend = env.backend_health.clone();
    let tenants = env.tenants.clone();
    let tenants_empty = tenants.is_empty();
    let tenants_list = tenants.clone();
    let incidents = env.incidents.clone();
    let incidents_empty = incidents.is_empty();
    let incidents_list = incidents.clone();
    let incident_sub = if incidents_empty {
        "All clear".to_string()
    } else {
        format!("{} open", incidents.len())
    };

    view! {
        <div class="grid gap-4" style="grid-template-columns: minmax(0,1fr) minmax(0,1.1fr); max-width: 1100px;">
            <div class="flex flex-col gap-4 min-w-0">
                <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                    <div class="flex items-center justify-between gap-3 mb-3">
                        <h2 class="text-sm font-semibold">"Incidents in this environment"</h2>
                        <span class="text-xs" style="color:var(--text-muted)">{incident_sub}</span>
                    </div>
                    <Show when=move || incidents_empty fallback=move || view! {
                        <div class="flex flex-col gap-2">
                            {incidents_list.clone().into_iter().map(|inc| {
                                let color = match inc.severity {
                                    IncidentSeverity::Bad => status_color(&HealthStatus::Down),
                                    IncidentSeverity::Warn => status_color(&HealthStatus::Degraded),
                                };
                                view! {
                                    <div class="flex items-start gap-2.5 py-2 border-b" style="border-color: var(--outline-variant, #eee);">
                                        <span class="inline-block h-2 w-2 rounded-full mt-1.5" style=format!("background:{color}")></span>
                                        <div class="min-w-0">
                                            <div class="text-sm font-semibold">{inc.title.clone()}</div>
                                            <div class="text-[11px] font-mono mt-0.5" style="color:var(--text-muted)">
                                                {format!("{} · since {}", inc.target, inc.since)}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }>
                        <p class="text-sm" style="color:var(--text-muted)">"No failing probes in this environment."</p>
                    </Show>
                </section>

                <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                    <h2 class="text-sm font-semibold mb-3">"Platform services"</h2>
                    <div class="flex flex-col gap-2 text-sm">
                        <div class="flex items-center gap-2">
                            <span class="inline-block h-2 w-2 rounded-full" style=format!("background:{}", status_color(&backend.status))></span>
                            <span class="font-mono text-xs">"backend / DB"</span>
                            <span style="color:var(--text-muted)" class="text-xs">
                                {format!("{} · {}ms · {}", status_label(&backend.status), backend.check_latency_ms, backend.message)}
                            </span>
                        </div>
                        {services.into_iter().map(|svc| {
                            let color = status_color(&svc.status);
                            let label = status_label(&svc.status);
                            view! {
                                <div class="flex flex-wrap items-center gap-2">
                                    <span class="inline-block h-2 w-2 rounded-full" style=format!("background:{color}")></span>
                                    <span class="font-mono text-xs">{svc.name.clone()}</span>
                                    <span class="text-xs" style="color:var(--text-muted)">{format!("{label} · {}", svc.detail)}</span>
                                    {svc.latency_ms.map(|ms| view! {
                                        <span class="text-xs font-mono" style="color:var(--text-muted)">{format!("{ms}ms")}</span>
                                    })}
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </section>
            </div>

            <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                <div class="flex items-center justify-between gap-3 mb-3">
                    <h2 class="text-sm font-semibold">"Tenants → Apps → Domains"</h2>
                    <span class="text-xs" style="color:var(--text-muted)">{format!("{} tenants", tenants.len())}</span>
                </div>
                <Show when=move || tenants_empty fallback=move || view! {
                    <div class="flex flex-col gap-3">
                        {tenants_list.clone().into_iter().map(|tenant| {
                            let t_color = status_color(&tenant.status);
                            view! {
                                <div class="border-l-2 pl-3" style=format!("border-color:{t_color}")>
                                    <div class="flex items-center gap-2 text-sm font-medium">
                                        <span class="inline-block h-2 w-2 rounded-full" style=format!("background:{t_color}")></span>
                                        {tenant.name.clone()}
                                        <span class="text-xs font-mono" style="color:var(--text-muted)">
                                            {format!("{} · {}", tenant.site_status, status_label(&tenant.status))}
                                        </span>
                                    </div>
                                    <div class="mt-2 ml-2 flex flex-col gap-2">
                                        {tenant.apps.into_iter().map(|app| {
                                            let a_color = status_color(&app.status);
                                            view! {
                                                <div>
                                                    <div class="flex items-center gap-2 text-xs">
                                                        <span class="inline-block h-1.5 w-1.5 rounded-full" style=format!("background:{a_color}")></span>
                                                        <span class="font-semibold">{app.app_type.clone()}</span>
                                                        <span style="color:var(--text-muted)">{status_label(&app.status)}</span>
                                                    </div>
                                                    <ul class="mt-1 ml-4 list-none flex flex-col gap-1">
                                                        {app.domains.into_iter().map(|d| {
                                                            let d_color = status_color(&d.status);
                                                            let latency = d.latency_ms.map(|ms| format!(" · {ms}ms")).unwrap_or_default();
                                                            view! {
                                                                <li class="font-mono text-[11px] flex items-center gap-2" style="color:var(--text-muted)">
                                                                    <span class="inline-block h-1.5 w-1.5 rounded-full" style=format!("background:{d_color}")></span>
                                                                    <span style="color:var(--on-surface, inherit)">{d.domain_name.clone()}</span>
                                                                    <span>{format!("{}{}", status_label(&d.status), latency)}</span>
                                                                </li>
                                                            }
                                                        }).collect_view()}
                                                    </ul>
                                                </div>
                                            }
                                        }).collect_view()}
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                }>
                    <p class="text-sm" style="color:var(--text-muted)">"No tenants provisioned yet."</p>
                </Show>
            </section>
        </div>
    }
}

#[component]
fn CapacityTab(fleet: FleetBlock, env: EnvironmentStatusNode) -> impl IntoView {
    let r = env.resources.clone();
    let label = env.label.clone();
    let db_ver = r.db_version.clone().unwrap_or_default();
    let db_ver_short = db_ver.chars().take(80).collect::<String>();
    let err = error_rate(&env);
    let fc = fleet.capacity.clone();
    let shares = fleet.by_environment.clone();

    view! {
        <div class="flex flex-col gap-4 max-w-3xl">
            <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                <h2 class="text-sm font-semibold mb-1">"Fleet totals"</h2>
                <p class="text-xs mb-4" style="color:var(--text-muted)">
                    "Sum across all reported environments — does not change when you select an env."
                </p>
                <div class="grid gap-3 mb-4" style="grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));">
                    <StatCard label="Tenants" value=fc.tenant_count.to_string() />
                    <StatCard label="App instances" value=fc.app_instance_count.to_string() />
                    <StatCard label="Domains" value=fc.domain_count.to_string() />
                    <StatCard label="DB size" value=format_bytes(fc.db_size_bytes) />
                    <StatCard label="DB sessions" value=fc.db_sessions.map(|n| n.to_string()).unwrap_or_else(|| "—".into()) />
                    <StatCard label="AI queued" value=fc.ai_tasks_queued.to_string() />
                    <StatCard label="AI running" value=fc.ai_tasks_running.to_string() />
                </div>
                <div class="flex flex-col gap-2">
                    {shares.into_iter().map(|s| {
                        let pct = (s.share_of_tenants * 100.0).round() as i32;
                        let width = format!("{pct}%");
                        view! {
                            <div class="grid gap-2.5 items-center text-xs" style="grid-template-columns: 88px minmax(0,1fr) 48px;">
                                <span class="font-semibold" style="color:var(--text-secondary, var(--text-muted))">{s.label.clone()}</span>
                                <div class="h-1.5 rounded overflow-hidden" style="background: var(--surface-container-high, #1a1a1a0a); border: 1px solid var(--outline-variant, #eee);">
                                    <div class="h-full rounded" style=format!("width:{width}; background: var(--cobalt, #2563eb);")></div>
                                </div>
                                <span class="font-mono text-right" style="color:var(--text-muted)">{format!("{pct}%")}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </section>

            <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                <h2 class="text-sm font-semibold mb-1">"Selected environment"</h2>
                <p class="text-xs mb-4" style="color:var(--text-muted)">
                    {format!("{label} · database / workers for this environment only")}
                </p>
                <div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));">
                    <StatCard label="Tenants" value=r.tenant_count.to_string() />
                    <StatCard label="App instances" value=r.app_instance_count.to_string() />
                    <StatCard label="Domains" value=r.domain_count.to_string() />
                    <StatCard label="DB size" value=format_bytes(r.db_size_bytes) />
                    <StatCard label="DB sessions" value=r.db_sessions.map(|n| n.to_string()).unwrap_or_else(|| "—".into()) />
                    <StatCard label="AI queued" value=r.ai_tasks_queued.to_string() />
                    <StatCard label="AI running" value=r.ai_tasks_running.to_string() />
                    <StatCard label="AI queue" value=if r.ai_queue_paused { "paused".into() } else { "running".into() } />
                    <StatCard label="HTTP err rate" value=err />
                </div>
                <p class="mt-4 text-xs font-mono" style="color:var(--text-muted)">{db_ver_short}</p>
                <div class="mt-4 flex gap-2">
                    <a class="btn btn-ghost btn-sm" href="/admin/aitasks" style="text-decoration:none">"AI Task Monitor →"</a>
                    <a class="btn btn-ghost btn-sm" href="/logs" style="text-decoration:none">"Audit Logs →"</a>
                    <a class="btn btn-ghost btn-sm" href="/admin/integrations" style="text-decoration:none">"Integrations →"</a>
                </div>
            </section>
        </div>
    }
}

#[component]
fn TelemetryTab(env: EnvironmentStatusNode) -> impl IntoView {
    let t = env.telemetry.clone();
    let counters_empty = t.counters.is_empty();
    let counters = t.counters.clone();
    let metrics_available = t.metrics_available;
    let detail = t.detail.clone();
    view! {
        <section class="rounded-lg border p-4 max-w-3xl" style="border-color: var(--outline-variant, #ddd);">
            <h2 class="text-sm font-semibold mb-1">"Sanitized counters"</h2>
            <p class="text-xs mb-4" style="color:var(--text-muted)">
                {format!(
                    "{detail} — aggregated server-side from the in-process registry (no METRICS_TOKEN in the browser)."
                )}
            </p>
            <Show when=move || counters_empty fallback=move || view! {
                <div class="flex flex-col gap-2">
                    {counters.clone().into_iter().map(|c| {
                        view! {
                            <div class="flex justify-between gap-4 text-sm font-mono border-b py-2" style="border-color: var(--outline-variant, #eee);">
                                <span>{c.name.clone()}</span>
                                <span class="font-semibold">{format!("{:.0}", c.value)}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }>
                <p class="text-sm" style="color:var(--text-muted)">
                    {if metrics_available {
                        "No labeled counters observed yet (quiet process)."
                    } else {
                        "Metrics unavailable."
                    }}
                </p>
            </Show>
        </section>
    }
}

#[component]
fn VersionsTab(envs: Vec<EnvironmentStatusNode>, collected: String) -> impl IntoView {
    let _ = collected;
    view! {
        <section class="rounded-lg border p-4 max-w-3xl" style="border-color: var(--outline-variant, #ddd);">
            <h2 class="text-sm font-semibold mb-1">"Build drift across environments"</h2>
            <p class="text-xs mb-4" style="color:var(--text-muted)">
                "Compare SHA / version so UAT lag vs Production is obvious."
            </p>
            <div class="flex flex-col gap-2">
                {envs.into_iter().map(|env| {
                    let color = status_color(&env.overall_status);
                    view! {
                        <div class="flex justify-between gap-3 py-2 border-b text-sm" style="border-color: var(--outline-variant, #eee);">
                            <span class="font-semibold flex items-center gap-2">
                                <span class="inline-block h-2 w-2 rounded-full" style=format!("background:{color}")></span>
                                {env.label.clone()}
                            </span>
                            <span class="font-mono text-xs" style="color:var(--text-muted)">
                                {format!(
                                    "v{} · {} · {}",
                                    env.version.version,
                                    env.version.build_sha.chars().take(7).collect::<String>(),
                                    env.version.build_date
                                )}
                            </span>
                        </div>
                    }
                }).collect_view()}
            </div>
            <p class="mt-4 text-xs" style="color:var(--text-muted)">
                "Useful when diagnosing “works in UAT, broken in prod” — same binary or not."
            </p>
        </section>
    }
}

#[component]
fn StatCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="rounded-lg p-3" style="background: var(--surface-container-high, #1a1a1a0a);">
            <div class="text-[10px] uppercase tracking-wide mb-1" style="color:var(--text-muted)">{label}</div>
            <div class="text-lg font-semibold font-mono">{value}</div>
        </div>
    }
}
