use crate::api::system_status::{
    HealthStatus, NextStepKind, SystemStatusResponse, get_system_status,
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

fn copy_text(text: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            let _ = window.navigator().clipboard().write_text(text);
        }
    }
    let _ = text;
}

#[component]
pub fn SystemStatusPage() -> impl IntoView {
    let active_tab = RwSignal::new("overview".to_string());
    let refresh = RwSignal::new(0u32);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let snapshot: RwSignal<Option<SystemStatusResponse>> = RwSignal::new(None);
    let loading = RwSignal::new(true);

    // Initial + manual refresh via LocalResource
    let status_res = LocalResource::new(move || {
        let _ = refresh.get();
        async move {
            loading.set(true);
            let res = get_system_status().await;
            loading.set(false);
            match res {
                Ok(s) => {
                    error.set(None);
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

    // Auto-poll every 3s (no enter/exit motion on ticks — high frequency)
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
                        "Deploy-safe runtime health — Environment → Tenant → App → Domain"
                    </p>
                </div>
                <div class="page-actions">
                    <button
                        class="btn btn-ghost btn-sm"
                        style="transition: transform 140ms cubic-bezier(0.23, 1, 0.32, 1);"
                        on:mousedown=move |_| {}
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

            // Keep LocalResource subscribed
            <div style="display:none">{move || status_res.get().map(|_| ())}</div>

            <Show when=move || snapshot.get().is_some() fallback=move || view! {
                <div class="text-sm" style="color:var(--text-muted)">"Loading system status…"</div>
            }>
                {move || snapshot.get().map(|s| view! {
                    <StatusBody status=s active_tab=active_tab />
                })}
            </Show>
        </div>
    }
}

#[component]
fn StatusBody(
    status: SystemStatusResponse,
    active_tab: RwSignal<String>,
) -> impl IntoView {
    let overall = status.overall_status.clone();
    let env = status.environment.clone();
    let version = status.version.clone();
    let collected = status.collected_at.clone();
    let show_local_hint = status.local_dev_hint.is_some();
    let local_hint_text = status.local_dev_hint.clone().unwrap_or_default();
    let for_overview = status.clone();
    let for_resources = status.clone();
    let for_telemetry = status;

    view! {
        // Health strip
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
                    {status_label(&overall)}
                </span>
            </div>
            <div class="text-xs font-mono" style="color:var(--text-muted)">
                {format!("env={env}")}
            </div>
            <div class="text-xs font-mono" style="color:var(--text-muted)">
                {format!(
                    "v{}+{}",
                    version.version,
                    version.build_sha.chars().take(7).collect::<String>()
                )}
            </div>
            <div class="text-xs" style="color:var(--text-muted); margin-left:auto">
                {format!("collected {collected}")}
            </div>
        </div>

        <Show when=move || show_local_hint>
            <div
                class="mb-4 rounded-lg border px-4 py-3 text-sm"
                style="border-color: color-mix(in srgb, var(--cobalt, #2563eb) 30%, transparent); background: color-mix(in srgb, var(--cobalt, #2563eb) 6%, transparent);"
            >
                {local_hint_text.clone()}
            </div>
        </Show>

        <div class="tab-bar mb-4">
            <button
                class=move || if active_tab.get() == "overview" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("overview".into())
            >"Overview"</button>
            <button
                class=move || if active_tab.get() == "resources" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("resources".into())
            >"Resources"</button>
            <button
                class=move || if active_tab.get() == "telemetry" { "tab active" } else { "tab" }
                on:click=move |_| active_tab.set("telemetry".into())
            >"Telemetry"</button>
        </div>

        <div style="transition: opacity 160ms cubic-bezier(0.23, 1, 0.32, 1);">
            <Show when=move || active_tab.get() == "overview">
                <OverviewTab status=for_overview.clone() />
            </Show>
            <Show when=move || active_tab.get() == "resources">
                <ResourcesTab status=for_resources.clone() />
            </Show>
            <Show when=move || active_tab.get() == "telemetry">
                <TelemetryTab status=for_telemetry.clone() />
            </Show>
        </div>
    }
}

#[component]
fn OverviewTab(status: SystemStatusResponse) -> impl IntoView {
    let next_steps = status.next_steps.clone();
    let services = status.platform_services.clone();
    let backend = status.backend_health.clone();
    let tenants = status.tenants.clone();
    let tenants_empty = tenants.is_empty();
    let tenants_list = tenants.clone();

    view! {
        <div class="grid gap-4" style="grid-template-columns: 1fr; max-width: 1100px;">
            // Backend + platform services
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

            // Hierarchy tree
            <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                <h2 class="text-sm font-semibold mb-3">"Tenants → Apps → Domains"</h2>
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

            // Next steps
            <section class="rounded-lg border p-4" style="border-color: var(--outline-variant, #ddd);">
                <h2 class="text-sm font-semibold mb-3">"Next steps"</h2>
                <div class="flex flex-col gap-3">
                    {next_steps.into_iter().map(|step| {
                        let cmd = step.command.clone();
                        let cmd_copy = cmd.clone();
                        let kind_label = match step.kind {
                            NextStepKind::Warning => "warning",
                            NextStepKind::Action => "action",
                            NextStepKind::Info => "info",
                        };
                        view! {
                            <div>
                                <div class="flex items-center justify-between gap-2 mb-1">
                                    <span class="text-sm font-medium">{step.headline.clone()}</span>
                                    <span class="text-[10px] uppercase tracking-wide" style="color:var(--text-muted)">{kind_label}</span>
                                </div>
                                <pre
                                    class="text-xs font-mono p-3 rounded overflow-x-auto whitespace-pre-wrap"
                                    style="background: var(--surface-container-high, #1a1a1a0d); margin:0;"
                                >{cmd}</pre>
                                <button
                                    class="btn btn-ghost btn-sm mt-1 system-status-press"
                                    on:click=move |_| copy_text(&cmd_copy)
                                >"Copy"</button>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </section>
        </div>
    }
}

#[component]
fn ResourcesTab(status: SystemStatusResponse) -> impl IntoView {
    let r = status.resources.clone();
    let db_ver = r.db_version.clone().unwrap_or_else(|| "—".into());
    let db_ver_short = db_ver.chars().take(80).collect::<String>();

    view! {
        <section class="rounded-lg border p-4 max-w-3xl" style="border-color: var(--outline-variant, #ddd);">
            <h2 class="text-sm font-semibold mb-1">"Application capacity"</h2>
            <p class="text-xs mb-4" style="color:var(--text-muted)">
                "Not host CPU/RAM — those stay on atlas-local status / cluster metrics. This is what the app can see."
            </p>
            <div class="grid gap-3" style="grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));">
                <StatCard label="Tenants" value=r.tenant_count.to_string() />
                <StatCard label="App instances" value=r.app_instance_count.to_string() />
                <StatCard label="Domains" value=r.domain_count.to_string() />
                <StatCard label="DB size" value=format_bytes(r.db_size_bytes) />
                <StatCard label="DB sessions" value=r.db_sessions.map(|n| n.to_string()).unwrap_or_else(|| "—".into()) />
                <StatCard label="AI queued" value=r.ai_tasks_queued.to_string() />
                <StatCard label="AI running" value=r.ai_tasks_running.to_string() />
                <StatCard label="AI queue" value=if r.ai_queue_paused { "paused".into() } else { "running".into() } />
            </div>
            <p class="mt-4 text-xs font-mono" style="color:var(--text-muted)">{db_ver_short}</p>
            <div class="mt-4 flex gap-2">
                <a class="btn btn-ghost btn-sm" href="/admin/aitasks" style="text-decoration:none">"AI Task Monitor →"</a>
                <a class="btn btn-ghost btn-sm" href="/logs" style="text-decoration:none">"Audit Logs →"</a>
            </div>
        </section>
    }
}

#[component]
fn TelemetryTab(status: SystemStatusResponse) -> impl IntoView {
    let t = status.telemetry.clone();
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
fn StatCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="rounded-lg p-3" style="background: var(--surface-container-high, #1a1a1a0a);">
            <div class="text-[10px] uppercase tracking-wide mb-1" style="color:var(--text-muted)">{label}</div>
            <div class="text-lg font-semibold font-mono">{value}</div>
        </div>
    }
}
