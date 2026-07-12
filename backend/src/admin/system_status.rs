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

impl HealthStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Down => "down",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProbeScheme {
    Http,
    Https,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextStepKind {
    Info,
    Action,
    Warning,
}

// ── Response DTOs ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub environment: String,
    pub overall_status: HealthStatus,
    pub version: VersionBlock,
    pub backend_health: BackendHealthBlock,
    pub platform_services: Vec<PlatformServiceProbe>,
    pub tenants: Vec<TenantStatusNode>,
    pub resources: ResourcesBlock,
    pub telemetry: TelemetryBlock,
    pub next_steps: Vec<NextStep>,
    pub local_dev_hint: Option<String>,
    pub collected_at: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextStep {
    pub kind: NextStepKind,
    pub headline: String,
    pub command: String,
}

pub fn routes_raw() -> Router<DatabaseConnection> {
    Router::new().route("/api/admin/system-status", get(get_system_status))
}

pub async fn get_system_status(
    State(db): State<DatabaseConnection>,
    Extension(_current_user): Extension<user::Model>,
) -> Result<impl IntoResponse, StatusCode> {
    let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
    let version = VersionBlock {
        version: ATLAS_VERSION.to_string(),
        build_sha: ATLAS_BUILD_SHA.to_string(),
        build_date: ATLAS_BUILD_DATE.to_string(),
    };

    let backend_health = check_backend_health(&db).await;
    let platform_services = probe_platform_services(&environment).await;
    let (tenants, domain_count) = build_hierarchy(&db).await?;
    let resources = collect_resources(&db, &tenants, domain_count).await;
    let telemetry = collect_telemetry();
    let overall_status =
        derive_overall(&backend_health, &platform_services, &tenants, &resources);
    let next_steps = build_next_steps(
        &environment,
        overall_status,
        &backend_health,
        &resources,
        &telemetry,
    );
    let local_dev_hint = if environment.eq_ignore_ascii_case("development")
        || environment.eq_ignore_ascii_case("dev")
        || environment.eq_ignore_ascii_case("local")
    {
        Some(
            "Full Compose/Docker view stays on the host: run `atlas-local status` (parity stack). This page is deploy-safe telemetry only."
                .to_string(),
        )
    } else {
        None
    };

    Ok(Json(SystemStatusResponse {
        environment,
        overall_status,
        version,
        backend_health,
        platform_services,
        tenants,
        resources,
        telemetry,
        next_steps,
        local_dev_hint,
        collected_at: chrono::Utc::now().to_rfc3339(),
    }))
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
    let detail = if counters.is_empty() {
        "idle — no labeled counters observed yet".to_string()
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

fn build_next_steps(
    environment: &str,
    overall: HealthStatus,
    backend: &BackendHealthBlock,
    resources: &ResourcesBlock,
    telemetry: &TelemetryBlock,
) -> Vec<NextStep> {
    let mut steps = Vec::new();
    let is_local = environment.eq_ignore_ascii_case("development")
        || environment.eq_ignore_ascii_case("dev")
        || environment.eq_ignore_ascii_case("local");

    match overall {
        HealthStatus::Healthy => {
            steps.push(NextStep {
                kind: NextStepKind::Info,
                headline: "Environment looks healthy".into(),
                command: if is_local {
                    "atlas-local status   # host Compose/Docker detail".into()
                } else {
                    "Re-check after deploys: open System Status or GET /api/admin/system-status"
                        .into()
                },
            });
        }
        HealthStatus::Down | HealthStatus::Degraded | HealthStatus::Unknown => {
            if !backend.database_connected {
                steps.push(NextStep {
                    kind: NextStepKind::Warning,
                    headline: "Database unreachable from backend".into(),
                    command: if is_local {
                        "atlas-local logs -f backend\natlas-local status\n# last resort: atlas-local reset-db".into()
                    } else {
                        "Check Postgres connectivity / Cloud SQL / secrets; inspect backend pod logs"
                            .into()
                    },
                });
            }
            steps.push(NextStep {
                kind: NextStepKind::Action,
                headline: "Inspect live signals".into(),
                command: "Open AI Task Monitor (/admin/aitasks) and Audit Logs (/logs)".into(),
            });
            if is_local {
                steps.push(NextStep {
                    kind: NextStepKind::Action,
                    headline: "Local recovery ladder".into(),
                    command: "atlas-local refresh backend\natlas-local down && atlas-local up\natlas-local reset-db   # destructive".into(),
                });
            }
        }
    }

    if resources.ai_queue_paused {
        steps.push(NextStep {
            kind: NextStepKind::Warning,
            headline: "AI queue is paused".into(),
            command: "Resume from AI Task Monitor → queue controls".into(),
        });
    }

    if !telemetry.metrics_available {
        steps.push(NextStep {
            kind: NextStepKind::Warning,
            headline: "In-process metrics unavailable".into(),
            command: "Confirm metrics registration at backend boot; Prometheus scrape uses METRICS_TOKEN server-side only".into(),
        });
    }

    steps.push(NextStep {
        kind: NextStepKind::Info,
        headline: format!("overall={}", overall.as_str()),
        command: format!(
            "tenants={} apps={} domains={} db_sessions={:?}",
            resources.tenant_count,
            resources.app_instance_count,
            resources.domain_count,
            resources.db_sessions
        ),
    });

    steps
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
