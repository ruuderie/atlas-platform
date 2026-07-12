//! Host / container resource + telemetry collectors for `atlas-local status`.

use crate::compose;
use crate::db::LocalDbConn;
use crate::repo;
use anyhow::Result;
use std::collections::VecDeque;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

const HISTORY: usize = 48;

#[derive(Debug, Clone, Default)]
pub struct ResourceSnapshot {
    pub stats: Vec<ContainerStat>,
    pub images: Vec<ImageSize>,
    pub volumes: Vec<VolumeSize>,
    pub totals: ResourceTotals,
    pub binaries: Vec<BinarySize>,
}

#[derive(Debug, Clone)]
pub struct ContainerStat {
    pub service: String,
    pub name: String,
    pub cpu_pct: f64,
    pub mem_used: String,
    pub mem_pct: f64,
    pub net_io: String,
    pub block_io: String,
}

#[derive(Debug, Clone)]
pub struct ImageSize {
    pub repository: String,
    pub size: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct VolumeSize {
    pub name: String,
    pub size: String,
}

#[derive(Debug, Clone)]
pub struct BinarySize {
    pub service: String,
    pub path: String,
    pub size: String,
}

#[derive(Debug, Clone, Default)]
pub struct ResourceTotals {
    pub cpu_pct: f64,
    pub mem_used_mib: f64,
    pub images_bytes: u64,
    pub volumes_hint: String,
}

#[derive(Debug, Clone, Default)]
pub struct TelemetrySnapshot {
    pub metrics_ok: bool,
    pub metrics_note: String,
    pub prometheus_lines: Vec<String>,
    pub recent_requests: Vec<String>,
    pub recent_events: Vec<String>,
    pub unprocessed_events: Option<u32>,
    pub daily_metrics: Vec<String>,
    pub feed_lines: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TelemetryHistory {
    pub backend_latency_ms: VecDeque<u64>,
    pub total_cpu_pct: VecDeque<f64>,
    pub total_mem_mib: VecDeque<f64>,
    pub feed: VecDeque<String>,
}

impl TelemetryHistory {
    pub fn push_sample(&mut self, latency_ms: Option<u64>, cpu: f64, mem_mib: f64, feed: &[String]) {
        if let Some(ms) = latency_ms {
            push_capped(&mut self.backend_latency_ms, ms, HISTORY);
        }
        push_capped(&mut self.total_cpu_pct, cpu, HISTORY);
        push_capped(&mut self.total_mem_mib, mem_mib, HISTORY);
        for line in feed {
            push_capped(&mut self.feed, line.clone(), 80);
        }
    }
}

fn push_capped<T>(q: &mut VecDeque<T>, value: T, max: usize) {
    q.push_back(value);
    while q.len() > max {
        q.pop_front();
    }
}

pub fn collect_resources() -> ResourceSnapshot {
    let stats = docker_stats();
    let images = docker_images();
    let volumes = docker_volumes();
    let binaries = collect_binaries();

    let cpu_pct: f64 = stats.iter().map(|s| s.cpu_pct).sum();
    let mem_used_mib: f64 = stats.iter().map(|s| parse_mem_mib(&s.mem_used)).sum();
    let images_bytes: u64 = images.iter().map(|i| i.size_bytes).sum();
    let volumes_hint = if volumes.is_empty() {
        "n/a".into()
    } else {
        volumes
            .iter()
            .map(|v| format!("{}={}", short_vol(&v.name), v.size))
            .collect::<Vec<_>>()
            .join(" · ")
    };

    ResourceSnapshot {
        stats,
        images,
        volumes,
        totals: ResourceTotals {
            cpu_pct,
            mem_used_mib,
            images_bytes,
            volumes_hint,
        },
        binaries,
    }
}

pub fn collect_telemetry(root: &Path) -> TelemetrySnapshot {
    let token = repo::read_dotenv_value(root, "METRICS_TOKEN")
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "local-dev-metrics".into());

    let (metrics_ok, metrics_note, prometheus_lines) = scrape_prometheus(&token);
    let conn = LocalDbConn::resolve(root);
    let recent_requests = query_recent_requests(root, &conn);
    let (recent_events, unprocessed_events) = query_telemetry_events(root, &conn);
    let daily_metrics = query_daily_metrics(root, &conn);

    let mut feed_lines = Vec::new();
    let ts = chrono_ish_now();
    if metrics_ok {
        for line in prometheus_lines.iter().take(8) {
            feed_lines.push(format!("{ts}  prom  {line}"));
        }
    } else {
        feed_lines.push(format!("{ts}  prom  {metrics_note}"));
    }
    for line in &recent_requests {
        feed_lines.push(format!("{ts}  http  {line}"));
    }
    for line in &recent_events {
        feed_lines.push(format!("{ts}  event {line}"));
    }

    TelemetrySnapshot {
        metrics_ok,
        metrics_note,
        prometheus_lines,
        recent_requests,
        recent_events,
        unprocessed_events,
        daily_metrics,
        feed_lines,
    }
}

/// Probe URL with HTTP code + total time (ms). Uses curl `-w`.
pub fn timed_probe(url: &str) -> (bool, Option<u64>) {
    let out = Command::new("curl")
        .args([
            "-sS",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code} %{time_total}",
            "--max-time",
            "3",
            url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else {
        return (false, None);
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.split_whitespace();
    let code = parts.next().unwrap_or("000");
    let secs: f64 = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let ms = (secs * 1000.0).round() as u64;
    let ok = code.starts_with('2') || code.starts_with('3');
    (ok, Some(ms))
}

fn docker_stats() -> Vec<ContainerStat> {
    let out = Command::new("docker")
        .args([
            "stats",
            "--no-stream",
            "--format",
            "{{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut p = line.split('\t');
        let name = p.next().unwrap_or("").trim().to_string();
        if name.is_empty() || !name.contains("atlas-platform") {
            continue;
        }
        let cpu = parse_pct(p.next().unwrap_or("0%"));
        let mem_used = p.next().unwrap_or("").to_string();
        let mem_pct = parse_pct(p.next().unwrap_or("0%"));
        let net_io = p.next().unwrap_or("").to_string();
        let block_io = p.next().unwrap_or("").to_string();
        rows.push(ContainerStat {
            service: service_from_container(&name),
            name,
            cpu_pct: cpu,
            mem_used,
            mem_pct,
            net_io,
            block_io,
        });
    }
    rows.sort_by(|a, b| a.service.cmp(&b.service));
    rows
}

fn docker_images() -> Vec<ImageSize> {
    let out = Command::new("docker")
        .args([
            "images",
            "--format",
            "{{.Repository}}\t{{.Size}}\t{{.ID}}",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut p = line.split('\t');
        let repo = p.next().unwrap_or("").to_string();
        if !(repo.starts_with("atlas-platform") || repo == "postgres" || repo == "caddy") {
            continue;
        }
        let size = p.next().unwrap_or("").to_string();
        rows.push(ImageSize {
            size_bytes: parse_size_bytes(&size),
            repository: repo,
            size,
        });
    }
    rows.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    rows
}

fn docker_volumes() -> Vec<VolumeSize> {
    // `docker system df -v` is heavy; use `docker volume ls` + inspect Size when available.
    // Fallback: parse `docker system df --format` for volumes line, plus known atlas volumes.
    let out = Command::new("docker")
        .args(["volume", "ls", "--format", "{{.Name}}"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for name in String::from_utf8_lossy(&out.stdout).lines() {
        let name = name.trim();
        if !name.contains("atlas-platform") {
            continue;
        }
        let size = volume_size(name).unwrap_or_else(|| "?".into());
        rows.push(VolumeSize {
            name: name.to_string(),
            size,
        });
    }
    rows
}

fn volume_size(name: &str) -> Option<String> {
    // Docker Desktop / Engine may expose Size via system df -v
    let out = Command::new("docker")
        .args(["system", "df", "-v"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut in_volumes = false;
    for line in text.lines() {
        if line.starts_with("Local Volumes") {
            in_volumes = true;
            continue;
        }
        if in_volumes && (line.starts_with("Build cache") || line.starts_with("Containers")) {
            break;
        }
        if in_volumes && line.starts_with(name) {
            // VOLUME NAME  LINKS  SIZE
            let parts: Vec<_> = line.split_whitespace().collect();
            if let Some(size) = parts.last() {
                return Some((*size).to_string());
            }
        }
    }
    None
}

fn collect_binaries() -> Vec<BinarySize> {
    let candidates = [
        ("anchor", "atlas-platform-anchor", "/app/anchor"),
        ("folio", "atlas-platform-folio", "/app/folio"),
    ];
    let mut out = Vec::new();
    for (service, container, path) in candidates {
        if let Some(size) = exec_ls_size(container, path) {
            out.push(BinarySize {
                service: service.into(),
                path: path.into(),
                size,
            });
        }
    }
    out
}

fn exec_ls_size(container: &str, path: &str) -> Option<String> {
    let out = Command::new("docker")
        .args(["exec", container, "ls", "-lh", path])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    // -rwxr-xr-x 1 root root 42M ... /app/anchor
    let line = String::from_utf8_lossy(&out.stdout);
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() >= 5 {
        Some(parts[4].to_string())
    } else {
        None
    }
}

fn scrape_prometheus(token: &str) -> (bool, String, Vec<String>) {
    let started = Instant::now();
    let out = Command::new("curl")
        .args([
            "-sS",
            "-o",
            "/tmp/atlas-local-metrics.prom",
            "-w",
            "%{http_code}",
            "-H",
            &format!("Authorization: Bearer {token}"),
            "--max-time",
            "3",
            "http://127.0.0.1:8000/metrics",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else {
        return (
            false,
            "curl failed reaching :8000/metrics".into(),
            Vec::new(),
        );
    };
    let code = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let body = std::fs::read_to_string("/tmp/atlas-local-metrics.prom").unwrap_or_default();
    let _ = std::fs::remove_file("/tmp/atlas-local-metrics.prom");

    if code == "401" || code == "403" {
        return (
            false,
            "unauthorized — add METRICS_TOKEN=local-dev-metrics to .env.local, then atlas-local refresh backend"
                .into(),
            Vec::new(),
        );
    }
    if code != "200" {
        return (
            false,
            format!("HTTP {code} from /metrics — is backend up?"),
            Vec::new(),
        );
    }

    let interesting = [
        "outbox_jobs_processed_total",
        "outbox_job_failures_total",
        "magic_link_requests_total",
        "passkey_auth_success_total",
        "passkey_registration_success_total",
        "auth_requests_total",
        "frontend_hydration_panics_total",
    ];
    let mut lines = Vec::new();
    for name in interesting {
        if let Some(v) = sum_prometheus_counter(&body, name) {
            lines.push(format!("{name}={v}"));
        }
    }
    for name in ["outbox_job_latency_seconds", "auth_request_duration_seconds"] {
        if let Some(count) = prometheus_label_value(&body, &format!("{name}_count")) {
            lines.push(format!("{name}_count={count}"));
        }
    }
    if lines.is_empty() {
        // CounterVec families stay absent until first observation — normal on a quiet local stack.
        lines.push("idle — no labeled counters observed yet (passkey/outbox/magic-link)".into());
        if !body.is_empty() {
            let preview: Vec<_> = body
                .lines()
                .filter(|l| !l.starts_with('#') && !l.is_empty())
                .take(6)
                .map(str::to_string)
                .collect();
            lines.extend(preview);
        }
    }
    (
        true,
        format!("scraped in {}ms", started.elapsed().as_millis()),
        lines,
    )
}

fn sum_prometheus_counter(body: &str, metric: &str) -> Option<f64> {
    let mut total = 0.0;
    let mut found = false;
    for line in body.lines() {
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with(metric)
            && (line[metric.len()..].starts_with('{') || line[metric.len()..].starts_with(' '))
        {
            if let Some(val) = line
                .rsplit_once(' ')
                .and_then(|(_, v)| v.parse::<f64>().ok())
            {
                total += val;
                found = true;
            }
        }
    }
    found.then_some(total)
}

fn prometheus_label_value(body: &str, metric: &str) -> Option<f64> {
    sum_prometheus_counter(body, metric)
}

fn query_recent_requests(root: &Path, conn: &LocalDbConn) -> Vec<String> {
    let exists = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT to_regclass('public.request_log') IS NOT NULL;",
    )
    .ok();
    if exists.as_deref() != Some("t") && exists.as_deref() != Some("true") {
        return Vec::new();
    }
    let sql = "SELECT method || ' ' || path || ' → ' || status_code::text \
               FROM request_log ORDER BY created_at DESC LIMIT 8;";
    sql_lines(root, &conn.user, &conn.database, sql).unwrap_or_default()
}

fn query_telemetry_events(root: &Path, conn: &LocalDbConn) -> (Vec<String>, Option<u32>) {
    let exists = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT to_regclass('public.telemetry_events') IS NOT NULL;",
    )
    .ok();
    if exists.as_deref() != Some("t") && exists.as_deref() != Some("true") {
        return (Vec::new(), None);
    }
    let events = sql_lines(
        root,
        &conn.user,
        &conn.database,
        "SELECT event_source || '/' || event_type || ' @ ' || to_char(timestamp, 'HH24:MI:SS') \
         FROM telemetry_events ORDER BY timestamp DESC LIMIT 8;",
    )
    .unwrap_or_default();
    let unprocessed = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT count(*)::text FROM telemetry_events WHERE processed = false;",
    )
    .ok()
    .and_then(|s| s.parse().ok());
    (events, unprocessed)
}

fn query_daily_metrics(root: &Path, conn: &LocalDbConn) -> Vec<String> {
    let exists = sql_scalar(
        root,
        &conn.user,
        &conn.database,
        "SELECT to_regclass('public.platform_metrics_daily') IS NOT NULL;",
    )
    .ok();
    if exists.as_deref() != Some("t") && exists.as_deref() != Some("true") {
        return Vec::new();
    }
    sql_lines(
        root,
        &conn.user,
        &conn.database,
        "SELECT metric_key || '=' || metric_value::text \
         FROM platform_metrics_daily WHERE date = CURRENT_DATE \
         ORDER BY metric_value DESC NULLS LAST LIMIT 8;",
    )
    .unwrap_or_default()
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
        anyhow::bail!("{}", String::from_utf8_lossy(&out.stderr).trim());
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

fn service_from_container(name: &str) -> String {
    let rest = name
        .strip_prefix("atlas-platform-")
        .unwrap_or(name);
    match rest {
        "db" => "postgres".into(),
        "admin" => "platform-admin".into(),
        "instance" => "network-instance".into(),
        other => other.to_string(),
    }
}

fn parse_pct(s: &str) -> f64 {
    s.trim()
        .trim_end_matches('%')
        .parse()
        .unwrap_or(0.0)
}

fn parse_mem_mib(s: &str) -> f64 {
    // "4.12GiB / 23.5GiB" or "64.12MiB / 23.5GiB"
    let used = s.split('/').next().unwrap_or("").trim();
    parse_size_to_mib(used)
}

fn parse_size_to_mib(s: &str) -> f64 {
    let s = s.trim();
    let lower = s.to_lowercase();
    if let Some(n) = lower.strip_suffix("gib") {
        return n.trim().parse::<f64>().unwrap_or(0.0) * 1024.0;
    }
    if let Some(n) = lower.strip_suffix("gb") {
        return n.trim().parse::<f64>().unwrap_or(0.0) * 1024.0;
    }
    if let Some(n) = lower.strip_suffix("mib") {
        return n.trim().parse::<f64>().unwrap_or(0.0);
    }
    if let Some(n) = lower.strip_suffix("mb") {
        return n.trim().parse::<f64>().unwrap_or(0.0);
    }
    if let Some(n) = lower.strip_suffix("kib") {
        return n.trim().parse::<f64>().unwrap_or(0.0) / 1024.0;
    }
    if let Some(n) = lower.strip_suffix('b') {
        return n.trim().parse::<f64>().unwrap_or(0.0) / (1024.0 * 1024.0);
    }
    0.0
}

fn parse_size_bytes(s: &str) -> u64 {
    let s = s.trim();
    let lower = s.to_lowercase();
    let (num, mult) = if let Some(n) = lower.strip_suffix("gb") {
        (n, 1_000_000_000f64)
    } else if let Some(n) = lower.strip_suffix("mb") {
        (n, 1_000_000f64)
    } else if let Some(n) = lower.strip_suffix("kb") {
        (n, 1_000f64)
    } else if let Some(n) = lower.strip_suffix('b') {
        (n, 1f64)
    } else {
        (s, 1f64)
    };
    (num.trim().parse::<f64>().unwrap_or(0.0) * mult) as u64
}

fn short_vol(name: &str) -> &str {
    name.rsplit('_').next().unwrap_or(name)
}

fn chrono_ish_now() -> String {
    // Avoid chrono dep: local HH:MM:SS via `date`
    Command::new("date")
        .args(["+%H:%M:%S"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "--:--:--".into())
}

/// Unicode sparkline from samples (Emil: tiny, purposeful state indication).
pub fn sparkline(values: &[f64], width: usize) -> String {
    const BARS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() || width == 0 {
        return "·".repeat(width.max(1));
    }
    let slice = if values.len() > width {
        &values[values.len() - width..]
    } else {
        values
    };
    let max = slice.iter().cloned().fold(0.0_f64, f64::max).max(1e-9);
    let mut out = String::new();
    for _ in 0..(width.saturating_sub(slice.len())) {
        out.push('·');
    }
    for v in slice {
        let idx = ((v / max) * (BARS.len() as f64 - 1.0)).round() as usize;
        out.push(BARS[idx.min(BARS.len() - 1)]);
    }
    out
}

pub fn sparkline_u64(values: &[u64], width: usize) -> String {
    let f: Vec<f64> = values.iter().map(|v| *v as f64).collect();
    sparkline(&f, width)
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1000.0;
    const MB: f64 = KB * 1000.0;
    const GB: f64 = MB * 1000.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2}GB", b / GB)
    } else if b >= MB {
        format!("{:.0}MB", b / MB)
    } else if b >= KB {
        format!("{:.0}KB", b / KB)
    } else {
        format!("{bytes}B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_non_empty() {
        let s = sparkline(&[1.0, 2.0, 5.0, 3.0], 8);
        assert_eq!(s.chars().count(), 8);
    }

    #[test]
    fn parse_mem_gib() {
        assert!((parse_mem_mib("4.12GiB / 23.5GiB") - 4.12 * 1024.0).abs() < 1.0);
    }

    #[test]
    fn service_name_map() {
        assert_eq!(service_from_container("atlas-platform-db"), "postgres");
        assert_eq!(
            service_from_container("atlas-platform-admin"),
            "platform-admin"
        );
    }
}
