//! Deploy-safe System Status for platform-admin.
//!
//! Assembles environment health, Tenant → App → Domain hierarchy, application
//! capacity signals, and sanitized Prometheus aggregates. Never exposes Docker,
//! `METRICS_TOKEN`, DB passwords, or raw `/metrics` text to the browser.
//!
//! Metrics are read from the in-process Prometheus registry (same data as
//! `/metrics`), so no bearer token loops through HTTP.

use axum::{
    Json, Router,
    extract::{Extension, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseBackend, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Statement,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::entities::{app_domain, app_instance, atlas_ai_task, atlas_app_deployment_config, tenant, user};
use crate::handlers::version::{ATLAS_BUILD_DATE, ATLAS_BUILD_SHA, ATLAS_VERSION};
use crate::metrics::REGISTRY;
use crate::services::ai_task_service::AiTaskService;

// ── Enums (wire as snake_case strings) ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeScheme {
    Http,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentId {
    Production,
    Uat,
    Development,
}

impl EnvironmentId {
    fn from_env_var(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            "uat" | "staging" => Self::Uat,
            "development" | "dev" | "local" => Self::Development,
            _ => Self::Development,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Production => "Production",
            Self::Uat => "UAT",
            Self::Development => "Development",
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Uat => "uat",
            Self::Development => "development",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    Warn,
    Bad,
}

// ── Response DTOs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub collected_at: String,
    pub fleet: FleetBlock,
    pub environments: Vec<EnvironmentStatusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetBlock {
    pub capacity: FleetCapacity,
    pub by_environment: Vec<FleetEnvironmentShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetCapacity {
    pub tenant_count: u64,
    pub app_instance_count: u64,
    pub domain_count: u64,
    pub db_size_bytes: Option<i64>,
    pub db_sessions: Option<i64>,
    pub ai_tasks_queued: u64,
    pub ai_tasks_running: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetEnvironmentShare {
    pub id: EnvironmentId,
    pub label: String,
    pub tenant_count: u64,
    pub share_of_tenants: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentStatusNode {
    pub id: EnvironmentId,
    pub label: String,
    pub overall_status: HealthStatus,
    pub reachable: bool,
    pub version: VersionBlock,
    pub collected_at: String,
    pub backend_health: BackendHealthBlock,
    pub platform_services: Vec<PlatformServiceProbe>,
    pub tenants: Vec<TenantStatusNode>,
    pub resources: ResourcesBlock,
    pub telemetry: TelemetryBlock,
    pub incidents: Vec<Incident>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Incident {
    pub severity: IncidentSeverity,
    pub title: String,
    pub target: String,
    pub since: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionBlock {
    pub version: String,
    pub build_sha: String,
    pub build_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendHealthBlock {
    pub status: HealthStatus,
    pub database_connected: bool,
    pub check_latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformServiceProbe {
    pub name: String,
    pub url: String,
    pub status: HealthStatus,
    pub http_status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStatusNode {
    pub tenant_id: String,
    pub name: String,
    pub site_status: String,
    pub status: HealthStatus,
    pub apps: Vec<AppInstanceStatusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInstanceStatusNode {
    pub instance_id: String,
    pub app_type: String,
    pub site_status: String,
    pub status: HealthStatus,
    pub domains: Vec<DomainStatusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainStatusNode {
    pub domain_name: String,
    pub scheme: ProbeScheme,
    pub status: HealthStatus,
    pub http_status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesBlock {
    pub db_version: Option<String>,
    pub db_size_bytes: Option<i64>,
    pub db_sessions: Option<i64>,
    pub tenant_count: u64,
    pub app_instance_count: u64,
    pub domain_count: u64,
    pub ai_queue_paused: bool,
    pub ai_tasks_queued: u64,
    pub ai_tasks_running: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryBlock {
    pub metrics_available: bool,
    pub detail: String,
    pub counters: Vec<MetricCounter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricCounter {
    pub name: String,
    pub value: f64,
}

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/admin/system-status", get(get_system_status))
}

pub async fn get_system_status(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let env_raw = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
    let env_id = EnvironmentId::from_env_var(&env_raw);
    let collected_at = chrono::Utc::now().to_rfc3339();
    let version = VersionBlock {
        version: ATLAS_VERSION.to_string(),
        build_sha: ATLAS_BUILD_SHA.to_string(),
        build_date: ATLAS_BUILD_DATE.to_string(),
    };

    let backend_health = check_backend_health(&db).await;
    let platform_services = probe_platform_services(env_id.as_str()).await;
    let (tenants, domain_count) = build_hierarchy(&db).await?;
    let resources = collect_resources(&db, &tenants, domain_count).await;
    let telemetry = collect_telemetry();
    let overall_status =
        derive_overall(&backend_health, &platform_services, &tenants, &resources);
    let incidents = collect_incidents(&platform_services, &tenants);

    let env_node = EnvironmentStatusNode {
        id: env_id,
        label: env_id.label().to_string(),
        overall_status,
        reachable: true,
        version,
        collected_at: collected_at.clone(),
        backend_health,
        platform_services,
        tenants,
        resources: resources.clone(),
        telemetry,
        incidents,
    };

    let fleet = fleet_from_environments(std::slice::from_ref(&env_node));

    Ok(Json(SystemStatusResponse {
        collected_at,
        fleet,
        environments: vec![env_node],
    }))
}

fn fleet_from_environments(envs: &[EnvironmentStatusNode]) -> FleetBlock {
    let mut tenant_count = 0u64;
    let mut app_instance_count = 0u64;
    let mut domain_count = 0u64;
    let mut db_size_bytes: Option<i64> = None;
    let mut db_sessions: Option<i64> = None;
    let mut ai_tasks_queued = 0u64;
    let mut ai_tasks_running = 0u64;

    for env in envs {
        let r = &env.resources;
        tenant_count = tenant_count.saturating_add(r.tenant_count);
        app_instance_count = app_instance_count.saturating_add(r.app_instance_count);
        domain_count = domain_count.saturating_add(r.domain_count);
        ai_tasks_queued = ai_tasks_queued.saturating_add(r.ai_tasks_queued);
        ai_tasks_running = ai_tasks_running.saturating_add(r.ai_tasks_running);
        if let Some(sz) = r.db_size_bytes {
            db_size_bytes = Some(db_size_bytes.unwrap_or(0).saturating_add(sz));
        }
        if let Some(sess) = r.db_sessions {
            db_sessions = Some(db_sessions.unwrap_or(0).saturating_add(sess));
        }
    }

    let by_environment = envs
        .iter()
        .map(|env| {
            let share = if tenant_count == 0 {
                0.0
            } else {
                env.resources.tenant_count as f64 / tenant_count as f64
            };
            FleetEnvironmentShare {
                id: env.id,
                label: env.label.clone(),
                tenant_count: env.resources.tenant_count,
                share_of_tenants: share,
            }
        })
        .collect();

    FleetBlock {
        capacity: FleetCapacity {
            tenant_count,
            app_instance_count,
            domain_count,
            db_size_bytes,
            db_sessions,
            ai_tasks_queued,
            ai_tasks_running,
        },
        by_environment,
    }
}

fn collect_incidents(
    services: &[PlatformServiceProbe],
    tenants: &[TenantStatusNode],
) -> Vec<Incident> {
    let mut incidents = Vec::new();
    for svc in services {
        match svc.status {
            HealthStatus::Down => incidents.push(Incident {
                severity: IncidentSeverity::Bad,
                title: format!("{} unreachable", svc.name),
                target: svc.name.clone(),
                since: "recent".into(),
            }),
            HealthStatus::Degraded => incidents.push(Incident {
                severity: IncidentSeverity::Warn,
                title: format!("{} degraded", svc.name),
                target: svc.name.clone(),
                since: "recent".into(),
            }),
            HealthStatus::Healthy | HealthStatus::Unknown => {}
        }
    }
    for tenant in tenants {
        for app in &tenant.apps {
            for domain in &app.domains {
                match domain.status {
                    HealthStatus::Down => incidents.push(Incident {
                        severity: IncidentSeverity::Bad,
                        title: format!("{} down", domain.domain_name),
                        target: "domain".into(),
                        since: "recent".into(),
                    }),
                    HealthStatus::Degraded => incidents.push(Incident {
                        severity: IncidentSeverity::Warn,
                        title: format!("{} degraded", domain.domain_name),
                        target: "domain".into(),
                        since: "recent".into(),
                    }),
                    HealthStatus::Healthy | HealthStatus::Unknown => {}
                }
            }
        }
    }
    incidents
}

async fn check_backend_health(db: &DatabaseConnection) -> BackendHealthBlock {
    let started = Instant::now();
    match db
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT 1".to_string(),
        ))
        .await
    {
        Ok(_) => BackendHealthBlock {
            status: HealthStatus::Healthy,
            database_connected: true,
            check_latency_ms: started.elapsed().as_millis() as u64,
            message: "database connected".into(),
        },
        Err(e) => BackendHealthBlock {
            status: HealthStatus::Down,
            database_connected: false,
            check_latency_ms: started.elapsed().as_millis() as u64,
            message: format!("database check failed: {e}"),
        },
    }
}

async fn probe_platform_services(environment: &str) -> Vec<PlatformServiceProbe> {
    // Self-relative URLs work in-cluster and via ingress when the admin SPA
    // talks to the same API host. Absolute public hosts are environment-specific;
    // we probe the process-local health path via loopback when possible.
    let mut targets: Vec<(&str, String)> = vec![
        ("backend_health", "http://127.0.0.1:8000/health".into()),
        ("api_version", "http://127.0.0.1:8000/api/version".into()),
    ];

    if environment.eq_ignore_ascii_case("development")
        || environment.eq_ignore_ascii_case("dev")
        || environment.eq_ignore_ascii_case("local")
    {
        targets.push(("admin_localhost", "http://admin.localhost/".into()));
        targets.push(("api_localhost", "http://api.localhost/health".into()));
    }

    let mut out = Vec::with_capacity(targets.len());
    for (name, url) in targets {
        out.push(probe_url(name, &url).await);
    }
    out
}

async fn probe_url(name: &str, url: &str) -> PlatformServiceProbe {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(1500))
        .redirect(reqwest::redirect::Policy::limited(2))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return PlatformServiceProbe {
                name: name.into(),
                url: url.into(),
                status: HealthStatus::Unknown,
                http_status: None,
                latency_ms: None,
                detail: format!("client build failed: {e}"),
            };
        }
    };

    let started = Instant::now();
    match client.get(url).send().await {
        Ok(res) => {
            let code = res.status().as_u16();
            let latency_ms = Some(started.elapsed().as_millis() as u64);
            let status = if res.status().is_success() {
                HealthStatus::Healthy
            } else if res.status().is_server_error() {
                HealthStatus::Down
            } else {
                HealthStatus::Degraded
            };
            PlatformServiceProbe {
                name: name.into(),
                url: url.into(),
                status,
                http_status: Some(code),
                latency_ms,
                detail: format!("HTTP {code}"),
            }
        }
        Err(e) => PlatformServiceProbe {
            name: name.into(),
            url: url.into(),
            status: HealthStatus::Down,
            http_status: None,
            latency_ms: Some(started.elapsed().as_millis() as u64),
            detail: e.to_string(),
        },
    }
}

async fn build_hierarchy(
    db: &DatabaseConnection,
) -> Result<(Vec<TenantStatusNode>, u64), StatusCode> {
    let instances = app_instance::Entity::find()
        .find_also_related(tenant::Entity)
        .all(db)
        .await
        .map_err(|e| {
            tracing::error!(event = "system_status.instances.failed", error = %e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let all_domains = app_domain::Entity::find().all(db).await.map_err(|e| {
        tracing::error!(event = "system_status.domains.failed", error = %e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let domain_count = all_domains.len() as u64;

    let mut domains_by_instance: BTreeMap<Uuid, Vec<app_domain::Model>> = BTreeMap::new();
    for d in all_domains {
        domains_by_instance
            .entry(d.app_instance_id)
            .or_default()
            .push(d);
    }

    // Group instances by tenant
    let mut by_tenant: BTreeMap<Uuid, (tenant::Model, Vec<app_instance::Model>)> = BTreeMap::new();
    for (instance, tenant_opt) in instances {
        let Some(t) = tenant_opt else { continue };
        if t.id == Uuid::nil() {
            continue;
        }
        by_tenant
            .entry(t.id)
            .or_insert_with(|| (t.clone(), Vec::new()))
            .1
            .push(instance);
    }

    // Collect domain probe targets (cap to keep latency bounded)
    const MAX_DOMAIN_PROBES: usize = 24;
    let mut probe_targets: Vec<(Uuid, String, ProbeScheme)> = Vec::new();
    'collect: for (_tid, (_t, apps)) in &by_tenant {
        for app in apps {
            if let Some(domains) = domains_by_instance.get(&app.id) {
                for d in domains {
                    if probe_targets.len() >= MAX_DOMAIN_PROBES {
                        break 'collect;
                    }
                    probe_targets.push((
                        app.id,
                        d.domain_name.clone(),
                        domain_scheme(&d.domain_name),
                    ));
                }
            }
        }
    }

    let probe_by_key = {
        let results = probe_domains_with_instance(probe_targets).await;
        let mut map: BTreeMap<(Uuid, String), DomainStatusNode> = BTreeMap::new();
        for (iid, node) in results {
            map.insert((iid, node.domain_name.clone()), node);
        }
        map
    };

    let mut tenants_out = Vec::new();
    for (_tid, (t, apps)) in by_tenant {
        let mut app_nodes = Vec::new();
        for app in apps {
            let deployment = atlas_app_deployment_config::Entity::find()
                .filter(atlas_app_deployment_config::Column::TenantId.eq(t.id))
                .filter(atlas_app_deployment_config::Column::AppSlug.eq(&app.app_type))
                .one(db)
                .await
                .unwrap_or(None);

            let site_status = deployment
                .as_ref()
                .map(|d| match d.instance_status {
                    atlas_app_deployment_config::AppInstanceStatus::Active => "active",
                    atlas_app_deployment_config::AppInstanceStatus::Suspended => "suspended",
                    atlas_app_deployment_config::AppInstanceStatus::Archived => "archived",
                })
                .unwrap_or(t.site_status.as_str())
                .to_string();

            let domain_models = domains_by_instance
                .get(&app.id)
                .cloned()
                .unwrap_or_default();

            let mut domain_nodes = Vec::new();
            for d in domain_models {
                if let Some(probed) = probe_by_key.get(&(app.id, d.domain_name.clone())) {
                    domain_nodes.push(probed.clone());
                } else {
                    let scheme = domain_scheme(&d.domain_name);
                    domain_nodes.push(DomainStatusNode {
                        domain_name: d.domain_name,
                        scheme,
                        status: HealthStatus::Unknown,
                        http_status: None,
                        latency_ms: None,
                        detail: "not probed (cap reached)".into(),
                    });
                }
            }

            let app_status = rollup_domain_status(&domain_nodes, &site_status);
            app_nodes.push(AppInstanceStatusNode {
                instance_id: app.id.to_string(),
                app_type: app.app_type,
                site_status,
                status: app_status,
                domains: domain_nodes,
            });
        }

        let tenant_status = rollup_app_status(&app_nodes, &t.site_status);
        tenants_out.push(TenantStatusNode {
            tenant_id: t.id.to_string(),
            name: t.name,
            site_status: t.site_status,
            status: tenant_status,
            apps: app_nodes,
        });
    }

    tenants_out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok((tenants_out, domain_count))
}

fn domain_scheme(domain: &str) -> ProbeScheme {
    if domain.ends_with(".localhost") || domain == "localhost" || domain.starts_with("127.") {
        ProbeScheme::Http
    } else {
        ProbeScheme::Https
    }
}

fn domain_url(domain: &str, scheme: ProbeScheme) -> String {
    match scheme {
        ProbeScheme::Http => format!("http://{domain}/"),
        ProbeScheme::Https => format!("https://{domain}/"),
    }
}

async fn probe_domains_with_instance(
    targets: Vec<(Uuid, String, ProbeScheme)>,
) -> Vec<(Uuid, DomainStatusNode)> {
    let mut handles = Vec::with_capacity(targets.len());
    for (instance_id, domain, scheme) in targets {
        handles.push(async move {
            let url = domain_url(&domain, scheme);
            let probe = probe_url(&domain, &url).await;
            (
                instance_id,
                DomainStatusNode {
                    domain_name: domain,
                    scheme,
                    status: probe.status,
                    http_status: probe.http_status,
                    latency_ms: probe.latency_ms,
                    detail: probe.detail,
                },
            )
        });
    }
    futures::future::join_all(handles).await
}

fn rollup_domain_status(domains: &[DomainStatusNode], site_status: &str) -> HealthStatus {
    if site_status.eq_ignore_ascii_case("suspended") || site_status.eq_ignore_ascii_case("archived")
    {
        return HealthStatus::Degraded;
    }
    if domains.is_empty() {
        return HealthStatus::Unknown;
    }
    if domains.iter().any(|d| d.status == HealthStatus::Down) {
        return HealthStatus::Down;
    }
    if domains
        .iter()
        .any(|d| matches!(d.status, HealthStatus::Degraded | HealthStatus::Unknown))
    {
        return HealthStatus::Degraded;
    }
    HealthStatus::Healthy
}

fn rollup_app_status(apps: &[AppInstanceStatusNode], site_status: &str) -> HealthStatus {
    if site_status.eq_ignore_ascii_case("suspended") {
        return HealthStatus::Degraded;
    }
    if apps.is_empty() {
        return HealthStatus::Unknown;
    }
    if apps.iter().any(|a| a.status == HealthStatus::Down) {
        return HealthStatus::Down;
    }
    if apps
        .iter()
        .any(|a| matches!(a.status, HealthStatus::Degraded | HealthStatus::Unknown))
    {
        return HealthStatus::Degraded;
    }
    HealthStatus::Healthy
}

async fn collect_resources(
    db: &DatabaseConnection,
    tenants: &[TenantStatusNode],
    domain_count: u64,
) -> ResourcesBlock {
    let db_version = db
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT version()".to_string(),
        ))
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<String>("", "version").ok());

    let db_size_bytes = db
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT pg_database_size(current_database())::bigint AS size".to_string(),
        ))
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<i64>("", "size").ok());

    let db_sessions = db
        .query_one(Statement::from_string(
            DatabaseBackend::Postgres,
            "SELECT count(*)::bigint AS sessions FROM pg_stat_activity".to_string(),
        ))
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<i64>("", "sessions").ok());

    let ai_tasks_queued = atlas_ai_task::Entity::find()
        .filter(
            Condition::any()
                .add(atlas_ai_task::Column::Status.eq("queued"))
                .add(atlas_ai_task::Column::Status.eq("Pending")),
        )
        .count(db)
        .await
        .unwrap_or(0);

    let ai_tasks_running = atlas_ai_task::Entity::find()
        .filter(
            Condition::any()
                .add(atlas_ai_task::Column::Status.eq("running"))
                .add(atlas_ai_task::Column::Status.eq("Running")),
        )
        .count(db)
        .await
        .unwrap_or(0);

    let app_instance_count = tenants.iter().map(|t| t.apps.len() as u64).sum();

    ResourcesBlock {
        db_version,
        db_size_bytes,
        db_sessions,
        tenant_count: tenants.len() as u64,
        app_instance_count,
        domain_count,
        ai_queue_paused: AiTaskService::is_queue_paused(),
        ai_tasks_queued,
        ai_tasks_running,
    }
}

fn collect_telemetry() -> TelemetryBlock {
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    if encoder.encode(&REGISTRY.gather(), &mut buffer).is_err() {
        return TelemetryBlock {
            metrics_available: false,
            detail: "failed to encode in-process metrics".into(),
            counters: Vec::new(),
        };
    }
    let body = String::from_utf8(buffer).unwrap_or_default();
    let interesting = [
        "outbox_jobs_processed_total",
        "outbox_job_failures_total",
        "magic_link_requests_total",
        "passkey_auth_success_total",
        "passkey_registration_success_total",
        "auth_requests_total",
        "frontend_hydration_panics_total",
    ];
    let mut counters = Vec::new();
    for name in interesting {
        if let Some(v) = sum_prometheus_counter(&body, name) {
            counters.push(MetricCounter {
                name: name.to_string(),
                value: v,
            });
        }
    }
    // Surface magic-link outcomes (esp. user_not_found) so operators see why
    // "email sent" UI produced no outbox / SMTP activity.
    for (status, v) in prometheus_counter_by_label(&body, "magic_link_requests_total", "status") {
        counters.push(MetricCounter {
            name: format!("magic_link_requests[{status}]"),
            value: v,
        });
    }
    let detail = if counters.is_empty() {
        "idle — no labeled counters observed yet".to_string()
    } else if counters.iter().any(|c| c.name.contains("user_not_found")) {
        "magic-link: unknown email(s) — check magic_link_requests[user_not_found] / backend logs"
            .to_string()
    } else {
        format!("{} counter families aggregated", counters.len())
    };
    TelemetryBlock {
        metrics_available: true,
        detail,
        counters,
    }
}

fn sum_prometheus_counter(body: &str, metric: &str) -> Option<f64> {
    let mut total = 0.0;
    let mut found = false;
    for line in body.lines() {
        if line.starts_with('#') {
            continue;
        }
        let (name_part, rest) = match line.split_once(' ') {
            Some(p) => p,
            None => continue,
        };
        let base = name_part.split('{').next().unwrap_or(name_part);
        if base != metric {
            continue;
        }
        if let Ok(v) = rest.trim().parse::<f64>() {
            total += v;
            found = true;
        }
    }
    found.then_some(total)
}

/// Sum a CounterVec by one label value (e.g. status=user_not_found).
fn prometheus_counter_by_label(
    body: &str,
    metric: &str,
    label: &str,
) -> Vec<(String, f64)> {
    use std::collections::BTreeMap;
    let needle = format!("{label}=\"");
    let mut map: BTreeMap<String, f64> = BTreeMap::new();
    for line in body.lines() {
        if line.starts_with('#') {
            continue;
        }
        let (name_part, rest) = match line.split_once(' ') {
            Some(p) => p,
            None => continue,
        };
        let base = name_part.split('{').next().unwrap_or(name_part);
        if base != metric {
            continue;
        }
        let Ok(v) = rest.trim().parse::<f64>() else {
            continue;
        };
        let Some(labels) = name_part.strip_prefix(metric).and_then(|s| {
            s.strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
        }) else {
            continue;
        };
        if let Some(idx) = labels.find(&needle) {
            let after = &labels[idx + needle.len()..];
            if let Some(end) = after.find('"') {
                let key = after[..end].to_string();
                *map.entry(key).or_insert(0.0) += v;
            }
        }
    }
    map.into_iter().collect()
}

fn derive_overall(
    backend: &BackendHealthBlock,
    services: &[PlatformServiceProbe],
    tenants: &[TenantStatusNode],
    resources: &ResourcesBlock,
) -> HealthStatus {
    if backend.status == HealthStatus::Down {
        return HealthStatus::Down;
    }
    if services.iter().any(|s| s.status == HealthStatus::Down) {
        return HealthStatus::Degraded;
    }
    if tenants.iter().any(|t| t.status == HealthStatus::Down) {
        return HealthStatus::Degraded;
    }
    if resources.ai_queue_paused {
        return HealthStatus::Degraded;
    }
    if backend.status == HealthStatus::Healthy
        && services
            .iter()
            .all(|s| matches!(s.status, HealthStatus::Healthy | HealthStatus::Unknown))
    {
        return HealthStatus::Healthy;
    }
    HealthStatus::Degraded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_serializes_snake_case() {
        let json = serde_json::to_string(&HealthStatus::Degraded).unwrap();
        assert_eq!(json, "\"degraded\"");
    }

    #[test]
    fn environment_id_from_env_var() {
        assert_eq!(
            EnvironmentId::from_env_var("production"),
            EnvironmentId::Production
        );
        assert_eq!(EnvironmentId::from_env_var("UAT"), EnvironmentId::Uat);
        assert_eq!(
            EnvironmentId::from_env_var("dev"),
            EnvironmentId::Development
        );
    }

    #[test]
    fn fleet_share_sums_single_env() {
        let resources = ResourcesBlock {
            db_version: None,
            db_size_bytes: Some(100),
            db_sessions: Some(3),
            tenant_count: 5,
            app_instance_count: 10,
            domain_count: 7,
            ai_queue_paused: false,
            ai_tasks_queued: 1,
            ai_tasks_running: 0,
        };
        let node = EnvironmentStatusNode {
            id: EnvironmentId::Development,
            label: "Development".into(),
            overall_status: HealthStatus::Healthy,
            reachable: true,
            version: VersionBlock {
                version: "0.1.0".into(),
                build_sha: "abc".into(),
                build_date: "2026-07-12".into(),
            },
            collected_at: "now".into(),
            backend_health: BackendHealthBlock {
                status: HealthStatus::Healthy,
                database_connected: true,
                check_latency_ms: 1,
                message: "ok".into(),
            },
            platform_services: vec![],
            tenants: vec![],
            resources,
            telemetry: TelemetryBlock {
                metrics_available: true,
                detail: "ok".into(),
                counters: vec![],
            },
            incidents: vec![],
        };
        let fleet = fleet_from_environments(&[node]);
        assert_eq!(fleet.capacity.tenant_count, 5);
        assert_eq!(fleet.by_environment.len(), 1);
        assert!((fleet.by_environment[0].share_of_tenants - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn sum_prometheus_counter_aggregates_labels() {
        let body = r#"
# HELP auth_requests_total Total
# TYPE auth_requests_total counter
auth_requests_total{tenant_id="a",status="ok"} 2
auth_requests_total{tenant_id="b",status="ok"} 3
"#;
        assert_eq!(sum_prometheus_counter(body, "auth_requests_total"), Some(5.0));
    }

    #[test]
    fn domain_scheme_localhost_is_http() {
        assert_eq!(domain_scheme("admin.localhost"), ProbeScheme::Http);
        assert_eq!(domain_scheme("example.com"), ProbeScheme::Https);
    }
}
