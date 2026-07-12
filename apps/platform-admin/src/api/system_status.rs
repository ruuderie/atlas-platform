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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentId {
    Production,
    Uat,
    #[default]
    Development,
}

impl EnvironmentId {
    pub fn label(self) -> &'static str {
        match self {
            Self::Production => "Production",
            Self::Uat => "UAT",
            Self::Development => "Development",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Uat => "uat",
            Self::Development => "development",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    #[default]
    Warn,
    Bad,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemStatusResponse {
    pub collected_at: String,
    pub fleet: FleetBlock,
    pub environments: Vec<EnvironmentStatusNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetBlock {
    pub capacity: FleetCapacity,
    pub by_environment: Vec<FleetEnvironmentShare>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetCapacity {
    pub tenant_count: u64,
    pub app_instance_count: u64,
    pub domain_count: u64,
    pub db_size_bytes: Option<i64>,
    pub db_sessions: Option<i64>,
    pub ai_tasks_queued: u64,
    pub ai_tasks_running: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FleetEnvironmentShare {
    pub id: EnvironmentId,
    pub label: String,
    pub tenant_count: u64,
    pub share_of_tenants: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Incident {
    pub severity: IncidentSeverity,
    pub title: String,
    pub target: String,
    pub since: String,
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

/// `GET /api/admin/system-status` — PlatformSuperAdmin session required.
pub async fn get_system_status() -> Result<SystemStatusResponse, String> {
    let client = create_client();
    let url = api_url("/api/admin/system-status");
    let req = client.get(&url);
    api_request::<SystemStatusResponse>(req).await
}
