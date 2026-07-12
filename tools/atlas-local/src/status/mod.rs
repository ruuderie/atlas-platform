//! Rich `atlas-local status` — data snapshot + plain text or ratatui dashboard.

mod guidance;
mod plain;
mod resources;
mod tui;

pub use guidance::{Guidance, StackHealth, SuggestedRefresh, mode_label, sync_cookbook};
pub use resources::{TelemetryHistory, format_bytes, sparkline, sparkline_u64};

use crate::compose::{self, ContainerRow};
use crate::db::LocalDbConn;
use crate::repo;
use anyhow::Result;
use resources::{
    ResourceSnapshot, TelemetrySnapshot, collect_resources, collect_telemetry, timed_probe,
};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct StatusSnapshot {
    pub root: PathBuf,
    pub collected_at: SystemTime,
    pub env_ok: bool,
    pub env_local_ok: bool,
    pub environment: String,
    pub rp_id: String,
    pub webauthn_origin: String,
    pub docker: String,
    pub compose: String,
    pub containers: Result<Vec<ContainerRow>, String>,
    pub domains: Vec<DomainEntry>,
    pub probes: Vec<Probe>,
    pub db: DbPanel,
    pub resources: ResourceSnapshot,
    pub telemetry: TelemetrySnapshot,
    pub env_panel: EnvPanel,
}

#[derive(Debug, Clone, Default)]
pub struct EnvPanel {
    pub smtp_mock: bool,
    pub smtp_status: String,
    pub smtp_rows: Vec<(String, String)>,
    pub local_rows: Vec<(String, String)>,
    pub applied_hint: String,
}

#[derive(Debug, Clone)]
pub struct DomainEntry {
    pub kind: String,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct Probe {
    pub label: String,
    pub url: String,
    pub ok: bool,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DbPanel {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub user: String,
    pub password: String,
    pub url: String,
    pub jdbc: String,
    pub state: Result<DbLiveState, String>,
}

#[derive(Debug, Clone)]
pub struct DbLiveState {
    pub ready_line: String,
    pub version: Option<String>,
    pub size: Option<String>,
    pub sessions: Option<u32>,
    pub tenants: Option<u32>,
    pub app_domains: Option<u32>,
    pub sample_domains: Vec<String>,
    pub note: Option<String>,
}

impl StatusSnapshot {
    pub fn collect(root: &Path) -> Self {
        let conn = LocalDbConn::resolve(root);
        let db_state = query_db_state(root, &conn).map_err(|e| e.to_string());

        Self {
            root: root.to_path_buf(),
            collected_at: SystemTime::now(),
            env_ok: root.join(".env").is_file(),
            env_local_ok: root.join(".env.local").is_file(),
            environment: repo::read_dotenv_value(root, "ENVIRONMENT")
                .unwrap_or_else(|| "(unset)".into()),
            rp_id: repo::read_dotenv_value(root, "RP_ID").unwrap_or_else(|| "(unset)".into()),
            webauthn_origin: repo::read_dotenv_value(root, "WEBAUTHN_ORIGIN")
                .unwrap_or_else(|| "(unset)".into()),
            docker: docker_version_line().unwrap_or_else(|| "unavailable".into()),
            compose: compose_version_line().unwrap_or_else(|| "unavailable".into()),
            containers: compose::ps_rows(root).map_err(|e| e.to_string()),
            domains: collect_domains(root),
            probes: collect_probes(),
            db: DbPanel {
                host: conn.host.clone(),
                port: conn.port,
                database: conn.database.clone(),
                user: conn.user.clone(),
                password: conn.password.clone(),
                url: conn.url(),
                jdbc: conn.jdbc_url(),
                state: db_state,
            },
            resources: collect_resources(),
            telemetry: collect_telemetry(root),
            env_panel: collect_env_panel(root),
        }
    }

    pub fn healthy_probe_count(&self) -> (usize, usize) {
        let ok = self.probes.iter().filter(|p| p.ok).count();
        (ok, self.probes.len())
    }

    pub fn container_summary(&self) -> (usize, usize, usize) {
        match &self.containers {
            Ok(rows) => {
                let running = rows
                    .iter()
                    .filter(|r| r.state.eq_ignore_ascii_case("running"))
                    .count();
                let unhealthy = rows
                    .iter()
                    .filter(|r| r.status.to_lowercase().contains("unhealthy"))
                    .count();
                (rows.len(), running, unhealthy)
            }
            Err(_) => (0, 0, 0),
        }
    }

    pub fn backend_latency_ms(&self) -> Option<u64> {
        self.probes
            .iter()
            .find(|p| p.label.starts_with("backend"))
            .and_then(|p| p.latency_ms)
    }
}

/// Entry point for `atlas-local status`.
pub fn print_report(root: &Path, plain: bool) -> Result<()> {
    let snapshot = StatusSnapshot::collect(root);
    let use_tui = !plain && std::io::stdout().is_terminal();
    if use_tui {
        tui::run(root, snapshot)
    } else {
        plain::print(&snapshot);
        Ok(())
    }
}

fn collect_env_panel(root: &Path) -> EnvPanel {
    use crate::env::{SMTP_KEYS, is_secret_key, mask_secret, smtp_is_mock};

    let mut smtp_rows = Vec::new();
    for key in SMTP_KEYS {
        let raw = repo::read_dotenv_value(root, key).unwrap_or_default();
        let display = if raw.is_empty() {
            "(unset)".into()
        } else if is_secret_key(key) {
            mask_secret(&raw)
        } else {
            raw.clone()
        };
        smtp_rows.push(((*key).to_string(), display));
    }
    let server = repo::read_dotenv_value(root, "SMTP_SERVER").unwrap_or_default();
    let smtp_mock = smtp_is_mock(&server);
    let smtp_status = if smtp_mock {
        "MOCK — emails logged only (not delivered)".into()
    } else {
        format!("CONFIGURED → {server} (needs backend recreate after edits)")
    };

    let interesting = [
        "ENVIRONMENT",
        "RP_ID",
        "WEBAUTHN_ORIGIN",
        "METRICS_TOKEN",
        "ADMIN_URL",
        "PUBLIC_API_BASE_URL",
    ];
    let mut local_rows = Vec::new();
    for key in interesting {
        if let Some(raw) = repo::read_dotenv_value(root, key) {
            let display = if is_secret_key(key) {
                mask_secret(&raw)
            } else {
                raw
            };
            local_rows.push((key.to_string(), display));
        }
    }

    EnvPanel {
        smtp_mock,
        smtp_status,
        smtp_rows,
        local_rows,
        applied_hint: "Values in .env.local are injected into backend on recreate — press a to apply"
            .into(),
    }
}

fn collect_domains(root: &Path) -> Vec<DomainEntry> {
    let caddy = root.join("Caddyfile");
    let hosts = match std::fs::read_to_string(&caddy) {
        Ok(text) => parse_caddy_hosts(&text),
        Err(_) => Vec::new(),
    };

    if hosts.is_empty() {
        return FALLBACK_URLS
            .iter()
            .map(|(kind, url)| DomainEntry {
                kind: (*kind).to_string(),
                url: (*url).to_string(),
            })
            .collect();
    }

    hosts
        .into_iter()
        .map(|host| DomainEntry {
            kind: classify_host(&host).to_string(),
            url: format!("http://{host}"),
        })
        .collect()
}

const FALLBACK_URLS: &[(&str, &str)] = &[
    ("Admin", "http://admin.localhost"),
    ("API", "http://api.localhost"),
    ("Network", "http://directory.network.localhost"),
    ("Folio", "http://folio.localhost"),
    ("Anchor", "http://buildwithruud.localhost"),
];

fn classify_host(host: &str) -> &'static str {
    if host.starts_with("api.") {
        "API"
    } else if host.starts_with("admin.") {
        "Admin"
    } else if host.contains("network") {
        "Network"
    } else if host.contains("folio") || host == "ruuderie.localhost" {
        "Folio"
    } else if host.contains("anchor")
        || host == "buildwithruud.localhost"
        || host == "oplystusa.localhost"
    {
        "Anchor"
    } else if host.starts_with("*.") {
        "Wildcard"
    } else {
        "Host"
    }
}

/// Extract `http://…` host tokens from a Caddyfile site address line.
pub fn parse_caddy_hosts(caddyfile: &str) -> Vec<String> {
    let mut hosts = Vec::new();
    for line in caddyfile.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.contains('{') {
            continue;
        }
        let addr = trimmed.split('{').next().unwrap_or("").trim();
        for part in addr.split(',') {
            let part = part.trim();
            if let Some(host) = part.strip_prefix("http://") {
                let host = host.trim();
                if !host.is_empty() && !hosts.iter().any(|h| h == host) {
                    hosts.push(host.to_string());
                }
            }
        }
    }
    hosts
}

fn collect_probes() -> Vec<Probe> {
    const PROBES: &[(&str, &str)] = &[
        ("backend /health", "http://127.0.0.1:8000/health"),
        ("platform-admin", "http://127.0.0.1:8081/"),
        ("network-instance", "http://127.0.0.1:8080/"),
        ("folio", "http://127.0.0.1:3100/"),
        ("anchor", "http://127.0.0.1:3000/"),
        ("Caddy api", "http://api.localhost/health"),
        ("Caddy admin", "http://admin.localhost/"),
    ];
    PROBES
        .iter()
        .map(|(label, url)| {
            let (ok, latency_ms) = timed_probe(url);
            Probe {
                label: (*label).to_string(),
                url: (*url).to_string(),
                ok,
                latency_ms,
            }
        })
        .collect()
}

fn query_db_state(root: &Path, conn: &LocalDbConn) -> Result<DbLiveState> {
    let ready = compose::exec_postgres(
        root,
        &["pg_isready", "-U", &conn.user, "-d", &conn.database],
    )?;
    if !ready.status.success() {
        anyhow::bail!("pg_isready failed");
    }

    let version = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT split_part(version(), ',', 1);",
    )?;
    let size = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT pg_size_pretty(pg_database_size(current_database()));",
    )?;
    let sessions = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT count(*)::text FROM pg_stat_activity WHERE datname = current_database();",
    )?
    .parse()
    .ok();

    let tenants = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT CASE WHEN to_regclass('public.tenant') IS NULL THEN NULL \
         ELSE (SELECT count(*)::text FROM tenant) END;",
    )
    .ok()
    .and_then(|s| if s.is_empty() || s == "NULL" { None } else { s.parse().ok() });

    let app_domains = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT CASE WHEN to_regclass('public.app_domains') IS NULL THEN NULL \
         ELSE (SELECT count(*)::text FROM app_domains) END;",
    )
    .ok()
    .and_then(|s| if s.is_empty() || s == "NULL" { None } else { s.parse().ok() });

    let sample_domains = if app_domains.is_some() {
        sql_lines(
            root,
            &conn.user,
            &conn.database,
            "SELECT domain_name FROM app_domains ORDER BY domain_name LIMIT 20;",
        )
        .unwrap_or_default()
    } else {
        Vec::new()
    };

    let note = if tenants.is_none() {
        Some(
            "schema not ready yet — backend has not finished migrations (parity: baked binary; hot: still compiling)"
                .into(),
        )
    } else {
        None
    };

    Ok(DbLiveState {
        ready_line: "accepting connections".into(),
        version: Some(version),
        size: Some(size),
        sessions,
        tenants,
        app_domains,
        sample_domains,
        note,
    })
}

fn sql_scalar(root: &Path, user: &str, database: &str, sql: &str) -> Result<String> {
    let out = compose::exec_postgres(
        root,
        &[
            "psql",
            "-U",
            user,
            "-d",
            database,
            "-v",
            "ON_ERROR_STOP=1",
            "-tA",
            "-c",
            sql,
        ],
    )?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("{}", err.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn sql_lines(root: &Path, user: &str, database: &str, sql: &str) -> Result<Vec<String>> {
    let raw = sql_scalar(root, user, database, sql)?;
    Ok(raw
        .lines()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

fn docker_version_line() -> Option<String> {
    let out = Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(format!("Engine {v}"))
    }
}

fn compose_version_line() -> Option<String> {
    let out = Command::new("docker")
        .args(["compose", "version", "--short"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_caddy_hosts_from_local_file_shape() {
        let sample = r#"
# comment
http://api.localhost {
	reverse_proxy backend:8000
}

http://*.network.localhost, http://network.localhost, http://directory.network.localhost {
	reverse_proxy network-instance:8080
}
"#;
        let hosts = parse_caddy_hosts(sample);
        assert_eq!(
            hosts,
            vec![
                "api.localhost",
                "*.network.localhost",
                "network.localhost",
                "directory.network.localhost",
            ]
        );
    }
}
