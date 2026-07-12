use super::client::{api_request, api_url, create_client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Down,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProbeScheme {
    Http,
    #[default]
    Https,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NextStepKind {
    #[default]
    Info,
    Action,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionBlock {
    pub version: String,
    pub build_sha: String,
    pub build_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendHealthBlock {
    pub status: HealthStatus,
    pub database_connected: bool,
    pub check_latency_ms: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformServiceProbe {
    pub name: String,
    pub url: String,
    pub status: HealthStatus,
    pub http_status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantStatusNode {
    pub tenant_id: String,
    pub name: String,
    pub site_status: String,
    pub status: HealthStatus,
    pub apps: Vec<AppInstanceStatusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppInstanceStatusNode {
    pub instance_id: String,
    pub app_type: String,
    pub site_status: String,
    pub status: HealthStatus,
    pub domains: Vec<DomainStatusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainStatusNode {
    pub domain_name: String,
    pub scheme: ProbeScheme,
    pub status: HealthStatus,
    pub http_status: Option<u16>,
    pub latency_ms: Option<u64>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TelemetryBlock {
    pub metrics_available: bool,
    pub detail: String,
    pub counters: Vec<MetricCounter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MetricCounter {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NextStep {
    pub kind: NextStepKind,
    pub headline: String,
    pub command: String,
}

/// `GET /api/admin/system-status` — PlatformSuperAdmin session required.
pub async fn get_system_status() -> Result<SystemStatusResponse, String> {
    let client = create_client();
    let url = api_url("/api/admin/system-status");
    let req = client.get(&url);
    api_request::<SystemStatusResponse>(req).await
}
