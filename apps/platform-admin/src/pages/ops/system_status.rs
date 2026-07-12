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

/// Tone class shared with stitch: ok | warn | bad | unk
fn status_tone(s: &HealthStatus) -> &'static str {
    match s {
        HealthStatus::Healthy => "ok",
        HealthStatus::Degraded => "warn",
        HealthStatus::Down => "bad",
        HealthStatus::Unknown => "unk",
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

struct ProbeCounts {
    ok: usize,
    warn: usize,
    bad: usize,
    total: usize,
}

fn probe_counts(env: &EnvironmentStatusNode) -> ProbeCounts {
    let total = env.platform_services.len();
    let ok = env
        .platform_services
        .iter()
        .filter(|p| p.status == HealthStatus::Healthy)
        .count();
    let warn = env
        .platform_services
        .iter()
        .filter(|p| p.status == HealthStatus::Degraded)
        .count();
    let bad = env
        .platform_services
        .iter()
        .filter(|p| p.status == HealthStatus::Down)
        .count();
    ProbeCounts {
        ok,
        warn,
        bad,
        total,
    }
}

fn sha7(sha: &str) -> String {
    sha.chars().take(7).collect()
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
        <div class="main-area no-pad">
            <div class="ss-page">
                <div style="display:none">{move || status_res.get().map(|_| ())}</div>

                <Show when=move || error.get().is_some()>
                    <div class="ss-chrome">
                        <div class="ss-header">
                            <div>
                                <div class="ss-title">"System Status"</div>
                                <div class="ss-sub">"Fleet health by environment."</div>
                            </div>
                            <div class="ss-actions">
                                <button
                                    class="btn btn-ghost btn-sm"
                                    type="button"
                                    on:click=move |_| refresh.update(|n| *n = n.wrapping_add(1))
                                >
                                    "Retry"
                                </button>
                            </div>
                        </div>
                        <div class="ss-error" style="margin:0 24px 16px">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </div>
                </Show>

                <Show when=move || snapshot.get().is_some() fallback=move || view! {
                    <div class="ss-chrome">
                        <div class="ss-header">
                            <div>
                                <div class="ss-title">"System Status"</div>
                                <div class="ss-sub">"Loading fleet status…"</div>
                            </div>
                            <div class="ss-actions">
                                <button class="btn btn-ghost btn-sm" type="button" disabled>"Refreshing…"</button>
                            </div>
                        </div>
                    </div>
                }>
                    {move || snapshot.get().map(|s| view! {
                        <StatusBody
                            status=s
                            active_tab=active_tab
                            selected_env=selected_env
                            loading=loading
                            refresh=refresh
                        />
                    })}
                </Show>
            </div>
        </div>
    }
}

#[component]
fn StatusBody(
    status: SystemStatusResponse,
    active_tab: RwSignal<String>,
    selected_env: RwSignal<EnvironmentId>,
    loading: RwSignal<bool>,
    refresh: RwSignal<u32>,
) -> impl IntoView {
    let envs_store = StoredValue::new(status.environments.clone());
    let fleet_store = StoredValue::new(status.fleet.clone());
    let envs_for_cards = status.environments.clone();
    let envs_for_versions = status.environments.clone();

    view! {
        <div class="ss-chrome">
            <div class="ss-header">
                <div>
                    <div class="ss-title">"System Status"</div>
                    <div class="ss-sub">
                        "Fleet health by environment. Pick Production, UAT, or Development — each rollup is independent."
                    </div>
                </div>
                <div class="ss-actions">
                    <button
                        class="btn btn-ghost btn-sm"
                        type="button"
                        on:click=move |_| refresh.update(|n| *n = n.wrapping_add(1))
                    >
                        {move || if loading.get() { "Refreshing…" } else { "Refresh" }}
                    </button>
                </div>
            </div>

            <div class="ss-fleet" role="tablist" aria-label="Environments">
                {envs_for_cards.into_iter().map(|env| {
                    let id = env.id;
                    let overall = env.overall_status.clone();
                    let tone = status_tone(&overall);
                    let label = env.label.clone();
                    let version = env.version.clone();
                    let counts = probe_counts(&env);
                    let tenant_n = env.tenants.len();
                    view! {
                        <button
                            type="button"
                            role="tab"
                            class=move || {
                                if selected_env.get() == id {
                                    "ss-env-card active"
                                } else {
                                    "ss-env-card"
                                }
                            }
                            prop:aria-selected=move || selected_env.get() == id
                            on:click=move |_| selected_env.set(id)
                        >
                            <div class="ss-env-top">
                                <span class="ss-env-name">{label.clone()}</span>
                                <span class=format!("ss-env-pill {tone}")>
                                    <span class=format!("ss-dot {tone}")></span>
                                    {status_label(&overall)}
                                </span>
                            </div>
                            <div class="ss-env-meta">
                                {format!("v{}+{}", version.version, sha7(&version.build_sha))}
                            </div>
                            <div class="ss-env-counts">
                                <span><b>{counts.ok}</b>" ok"</span>
                                <span><b>{counts.warn}</b>" degraded"</span>
                                <span><b>{counts.bad}</b>" down"</span>
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
                    let tone = status_tone(&overall);
                    let label = env.label.clone();
                    let version = env.version.clone();
                    let collected_env = env.collected_at.clone();
                    let counts = probe_counts(&env);
                    let err = error_rate(&env);
                    let is_dev = matches!(env.id, EnvironmentId::Development);
                    view! {
                        <div class="ss-health">
                            <div class=format!("ss-pill {tone}")>
                                <span class=format!("ss-dot {tone}")></span>
                                {format!("{} · {}", label, status_label(&overall))}
                            </div>
                            <span class="ss-meta">{format!("probes {}/{} healthy", counts.ok, counts.total)}</span>
                            <span class="ss-meta">{format!("err rate {err}")}</span>
                            <span class="ss-meta">
                                {format!("v{}+{}", version.version, sha7(&version.build_sha))}
                            </span>
                            <span class="ss-meta-right">{format!("collected {collected_env} · auto 3s")}</span>
                        </div>

                        <Show when=move || is_dev>
                            <div style="padding:10px 24px 0">
                                <div class="ss-live">
                                    <strong>"Local stack."</strong>
                                    " Full Compose/Docker view stays on the host: "
                                    <code>"atlas-local status"</code>
                                    " (parity). This page is deploy-safe telemetry only."
                                </div>
                            </div>
                        </Show>
                    }
                })
            }}

            <div class="ss-tabs" role="tablist">
                <button
                    type="button"
                    class=move || if active_tab.get() == "overview" { "ss-tab active" } else { "ss-tab" }
                    on:click=move |_| active_tab.set("overview".into())
                >"Overview"</button>
                <button
                    type="button"
                    class=move || if active_tab.get() == "resources" { "ss-tab active" } else { "ss-tab" }
                    on:click=move |_| active_tab.set("resources".into())
                >"Capacity"</button>
                <button
                    type="button"
                    class=move || if active_tab.get() == "telemetry" { "ss-tab active" } else { "ss-tab" }
                    on:click=move |_| active_tab.set("telemetry".into())
                >"Telemetry"</button>
                <button
                    type="button"
                    class=move || if active_tab.get() == "versions" { "ss-tab active" } else { "ss-tab" }
                    on:click=move |_| active_tab.set("versions".into())
                >"Versions"</button>
            </div>
        </div>

        <div class="ss-canvas">
            <div class="ss-live">
                <strong>"Live contract."</strong>
                " Status pills are computed "
                <em>"inside"</em>
                " each environment node. Dev downtime never paints Production red. Aggregate via "
                <code>"GET /api/admin/system-status"</code>
                " → "
                <code>"fleet"</code>
                " + "
                <code>"environments[]"</code>
                "."
            </div>

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
                    ("overview", Some(env)) => view! {
                        <div class="ss-pane active"><OverviewTab env=env /></div>
                    }
                    .into_any(),
                    ("resources", Some(env)) => {
                        let fleet = fleet_store.with_value(|f| f.clone());
                        view! {
                            <div class="ss-pane active"><CapacityTab fleet=fleet env=env /></div>
                        }
                        .into_any()
                    }
                    ("telemetry", Some(env)) => view! {
                        <div class="ss-pane active"><TelemetryTab env=env /></div>
                    }
                    .into_any(),
                    ("versions", _) => view! {
                        <div class="ss-pane active">
                            <VersionsTab envs=envs_for_versions.clone() />
                        </div>
                    }
                    .into_any(),
                    _ => view! { <div class="ss-pane active"></div> }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
fn OverviewTab(env: EnvironmentStatusNode) -> impl IntoView {
    let services = env.platform_services.clone();
    let tenants = env.tenants.clone();
    let tenants_empty = tenants.is_empty();
    let tenants_list = tenants.clone();
    let tenant_count = tenants.len();
    let incidents = env.incidents.clone();
    let incidents_empty = incidents.is_empty();
    let incidents_list = incidents.clone();
    let incident_sub = if incidents_empty {
        "All clear".to_string()
    } else {
        format!("{} open", incidents.len())
    };

    view! {
        <div class="ss-grid">
            <div class="ss-col">
                <section class="ss-section">
                    <div class="ss-section-hd">
                        <div class="ss-section-title">"Incidents in this environment"</div>
                        <div class="ss-section-sub">{incident_sub}</div>
                    </div>
                    <div class="ss-section-bd">
                        <Show when=move || incidents_empty fallback=move || view! {
                            {incidents_list.clone().into_iter().map(|inc| {
                                let tone = match inc.severity {
                                    IncidentSeverity::Bad => "bad",
                                    IncidentSeverity::Warn => "warn",
                                };
                                view! {
                                    <div class="ss-incident">
                                        <span class=format!("ss-dot {tone}")></span>
                                        <div class="ss-incident-body">
                                            <div class="ss-incident-title">{inc.title.clone()}</div>
                                            <div class="ss-incident-meta">
                                                {format!("{} · since {}", inc.target, inc.since)}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        }>
                            <div class="ss-empty">"No failing probes in this environment."</div>
                        </Show>
                    </div>
                </section>

                <section class="ss-section">
                    <div class="ss-section-hd">
                        <div class="ss-section-title">"Platform services"</div>
                        <div class="ss-section-sub">"HTTP probes · this env only"</div>
                    </div>
                    <div class="ss-section-bd">
                        {services.into_iter().map(|svc| {
                            let tone = status_tone(&svc.status);
                            let lat = svc
                                .latency_ms
                                .map(|ms| format!("{ms}ms"))
                                .unwrap_or_else(|| "—".into());
                            view! {
                                <div class="ss-svc">
                                    <span class=format!("ss-dot {tone}")></span>
                                    <div>
                                        <div class="ss-svc-name">{svc.name.clone()}</div>
                                        <div class="ss-svc-detail">{svc.detail.clone()}</div>
                                    </div>
                                    <div class="ss-svc-lat">{lat}</div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </section>
            </div>

            <section class="ss-section">
                <div class="ss-section-hd">
                    <div class="ss-section-title">"Tenants → Apps → Domains"</div>
                    <div class="ss-section-sub">{format!("{tenant_count} tenants")}</div>
                </div>
                <div class="ss-section-bd gap-lg">
                    <Show when=move || tenants_empty fallback=move || view! {
                        {tenants_list.clone().into_iter().map(|tenant| {
                            let tone = status_tone(&tenant.status);
                            let tenant_class = match tone {
                                "ok" => "ss-tenant".to_string(),
                                t => format!("ss-tenant {t}"),
                            };
                            view! {
                                <div class=tenant_class>
                                    <div class="ss-tenant-hd">
                                        <span class=format!("ss-dot {tone}")></span>
                                        {tenant.name.clone()}
                                        <span class="ss-tenant-meta">
                                            {format!("{} · {}", tenant.site_status, status_label(&tenant.status))}
                                        </span>
                                    </div>
                                    {tenant.apps.into_iter().map(|app| {
                                        let a_tone = status_tone(&app.status);
                                        view! {
                                            <div class="ss-app">
                                                <div class="ss-app-hd">
                                                    <span class=format!("ss-dot sm {a_tone}")></span>
                                                    <strong>{app.app_type.clone()}</strong>
                                                    <span class="ss-tenant-meta">{status_label(&app.status)}</span>
                                                </div>
                                                <ul class="ss-domains">
                                                    {app.domains.into_iter().map(|d| {
                                                        let d_tone = status_tone(&d.status);
                                                        let lat = d
                                                            .latency_ms
                                                            .map(|ms| format!("{} · {ms}ms", status_label(&d.status)))
                                                            .unwrap_or_else(|| status_label(&d.status).to_string());
                                                        view! {
                                                            <li>
                                                                <span class=format!("ss-dot sm {d_tone}")></span>
                                                                <span class="name">{d.domain_name.clone()}</span>
                                                                <span>{lat}</span>
                                                            </li>
                                                        }
                                                    }).collect_view()}
                                                </ul>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }
                        }).collect_view()}
                    }>
                        <div class="ss-empty">"No tenants in this environment."</div>
                    </Show>
                </div>
            </section>
        </div>
    }
}

#[component]
fn CapacityTab(fleet: FleetBlock, env: EnvironmentStatusNode) -> impl IntoView {
    let r = env.resources.clone();
    let label = env.label.clone();
    let db_ver = r.db_version.clone().unwrap_or_default();
    let err = error_rate(&env);
    let fc = fleet.capacity.clone();
    let shares = fleet.by_environment.clone();
    let ai_queue = if r.ai_queue_paused {
        "paused".to_string()
    } else {
        "running".to_string()
    };

    view! {
        <section class="ss-section">
            <div class="ss-section-hd">
                <div>
                    <div class="ss-section-title">"Fleet totals"</div>
                    <div class="ss-section-sub" style="margin-top:2px">
                        "Sum across all reported environments — does not change when you select an env"
                    </div>
                </div>
            </div>
            <div class="ss-section-bd ss-fleet-cap">
                <div class="ss-stats">
                    <StatCard label="Tenants" value=fc.tenant_count.to_string() />
                    <StatCard label="App instances" value=fc.app_instance_count.to_string() />
                    <StatCard label="Domains" value=fc.domain_count.to_string() />
                    <StatCard label="DB size" value=format_bytes(fc.db_size_bytes) />
                    <StatCard
                        label="DB sessions"
                        value=fc.db_sessions.map(|n| n.to_string()).unwrap_or_else(|| "—".into())
                    />
                    <StatCard label="AI queued" value=fc.ai_tasks_queued.to_string() />
                    <StatCard label="AI running" value=fc.ai_tasks_running.to_string() />
                </div>
                <div class="ss-share">
                    {shares.into_iter().map(|s| {
                        let pct = (s.share_of_tenants * 100.0).round() as i32;
                        let width = format!("{pct}%");
                        view! {
                            <div class="ss-share-row">
                                <span class="ss-share-lbl">{s.label.clone()}</span>
                                <div class="ss-share-track">
                                    <div class="ss-share-fill" style=format!("width:{width}")></div>
                                </div>
                                <span class="ss-share-pct">{format!("{pct}%")}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>
        </section>

        <section class="ss-section">
            <div class="ss-section-hd">
                <div>
                    <div class="ss-section-title">"Selected environment"</div>
                    <div class="ss-section-sub" style="margin-top:2px">
                        {format!("{label} · database / workers for this environment only")}
                    </div>
                </div>
            </div>
            <div class="ss-section-bd">
                <div class="ss-stats">
                    <StatCard label="Tenants" value=r.tenant_count.to_string() />
                    <StatCard label="App instances" value=r.app_instance_count.to_string() />
                    <StatCard label="Domains" value=r.domain_count.to_string() />
                    <StatCard label="DB size" value=format_bytes(r.db_size_bytes) />
                    <StatCard
                        label="DB sessions"
                        value=r.db_sessions.map(|n| n.to_string()).unwrap_or_else(|| "—".into())
                    />
                    <StatCard label="AI queued" value=r.ai_tasks_queued.to_string() />
                    <StatCard label="AI running" value=r.ai_tasks_running.to_string() />
                    <StatCard label="AI queue" value=ai_queue />
                    <StatCard label="HTTP err rate" value=err />
                </div>
                <p class="ss-note">{db_ver}</p>
                <div class="ss-links">
                    <a class="btn btn-ghost btn-sm" href="/admin/aitasks">"AI Task Monitor →"</a>
                    <a class="btn btn-ghost btn-sm" href="/logs">"Audit Logs →"</a>
                    <a class="btn btn-ghost btn-sm" href="/admin/integrations">"Integrations →"</a>
                </div>
            </div>
        </section>
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
        <section class="ss-section">
            <div class="ss-section-hd">
                <div>
                    <div class="ss-section-title">"Sanitized counters"</div>
                    <div class="ss-section-sub" style="margin-top:2px">
                        "Server-side aggregates for this env — no METRICS_TOKEN in the browser"
                    </div>
                </div>
            </div>
            <div class="ss-section-bd">
                <p class="ss-note">{detail}</p>
                <Show when=move || counters_empty fallback=move || view! {
                    {counters.clone().into_iter().map(|c| {
                        view! {
                            <div class="ss-counter">
                                <span class="ss-counter-name">{c.name.clone()}</span>
                                <span class="ss-counter-val">{format!("{:.0}", c.value)}</span>
                            </div>
                        }
                    }).collect_view()}
                }>
                    <div class="ss-empty">
                        {if metrics_available {
                            "No labeled counters observed yet (quiet process)."
                        } else {
                            "Metrics unavailable for this environment."
                        }}
                    </div>
                </Show>
            </div>
        </section>
    }
}

#[component]
fn VersionsTab(envs: Vec<EnvironmentStatusNode>) -> impl IntoView {
    view! {
        <section class="ss-section">
            <div class="ss-section-hd">
                <div>
                    <div class="ss-section-title">"Build drift across environments"</div>
                    <div class="ss-section-sub" style="margin-top:2px">
                        "Compare SHA / version so UAT lag vs Production is obvious"
                    </div>
                </div>
            </div>
            <div class="ss-section-bd">
                <div class="ss-drift">
                    {envs.into_iter().map(|env| {
                        let tone = status_tone(&env.overall_status);
                        view! {
                            <div class="ss-drift-row">
                                <span class="ss-drift-env">
                                    <span class=format!("ss-dot {tone}")></span>
                                    {env.label.clone()}
                                </span>
                                <span class="ss-drift-sha">
                                    {format!(
                                        "v{} · {} · {}",
                                        env.version.version,
                                        sha7(&env.version.build_sha),
                                        env.version.build_date
                                    )}
                                </span>
                            </div>
                        }
                    }).collect_view()}
                </div>
                <p class="ss-note">
                    "Useful when diagnosing “works in UAT, broken in prod” — same binary or not."
                </p>
            </div>
        </section>
    }
}

#[component]
fn StatCard(label: &'static str, value: String) -> impl IntoView {
    let small = value.len() > 8;
    view! {
        <div class="ss-stat">
            <div class="ss-stat-lbl">{label}</div>
            <div class=if small { "ss-stat-val sm" } else { "ss-stat-val" }>{value}</div>
        </div>
    }
}
