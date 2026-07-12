//! Plain-text status fallback (pipes, CI, `--plain`).

use super::{StatusSnapshot, format_bytes};

pub fn print(snap: &StatusSnapshot) {
    let guide = snap.guidance();
    let hot = crate::compose::hot_mode();
    let mode = super::mode_label(hot);

    println!();
    println!("═══ Atlas local status ═══");
    println!();
    println!("Mode: {mode}");
    println!();
    println!("Next steps — {}", guide.headline);
    println!("  {}", guide.x_refresh_hint());
    println!("  (TUI: press ? for sync guide · x to run that refresh · r only reloads the panel)");
    for line in &guide.commands {
        println!("  {line}");
    }
    println!();
    println!("Sync after code changes (same as TUI ?)");
    for line in super::sync_cookbook(hot) {
        println!("  {line}");
    }
    println!();

    println!("System");
    println!("  Root:          {}", snap.root.display());
    println!("  Mode:          {mode}");
    println!(
        "  Env files:     .env {}   .env.local {}",
        if snap.env_ok { "✓" } else { "✗" },
        if snap.env_local_ok {
            "✓"
        } else {
            "missing"
        }
    );
    println!("  ENVIRONMENT:   {}", snap.environment);
    println!("  WebAuthn:      RP_ID={}", snap.rp_id);
    println!("                 ORIGIN={}", snap.webauthn_origin);
    println!("  Docker:        {}", snap.docker);
    println!("  Compose:       {}", snap.compose);
    println!();

    println!("Application capacity");
    match &snap.db.state {
        Ok(state) => {
            println!(
                "  tenants {}   domains {}   DB {}   sessions {}",
                state
                    .tenants
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "—".into()),
                state
                    .app_domains
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "—".into()),
                state.size.as_deref().unwrap_or("—"),
                state
                    .sessions
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| "—".into()),
            );
        }
        Err(e) => println!("  (db unreachable: {e})"),
    }
    println!();

    println!("Stack load");
    println!(
        "  CPU {:.1}%   RAM {:.0} MiB   images {}",
        snap.resources.totals.cpu_pct,
        snap.resources.totals.mem_used_mib,
        format_bytes(snap.resources.totals.images_bytes)
    );
    println!();

    println!("Containers (CPU / RAM)");
    if snap.resources.stats.is_empty() {
        println!("  (none — atlas-local up)");
    } else {
        for s in &snap.resources.stats {
            println!(
                "  {:<18} CPU {:>6.1}%  MEM {:<14} ({:.1}%)  net {}",
                s.service,
                s.cpu_pct,
                s.mem_used.split('/').next().unwrap_or("").trim(),
                s.mem_pct,
                s.net_io
            );
        }
    }
    println!();

    println!("Images");
    for i in &snap.resources.images {
        println!("  {:<32} {}", i.repository, i.size);
    }
    println!();

    if !snap.resources.binaries.is_empty() {
        println!("Binaries");
        for b in &snap.resources.binaries {
            println!("  {}  {}  {}", b.service, b.path, b.size);
        }
        println!();
    }

    if !snap.resources.volumes.is_empty() {
        println!("Volumes");
        for v in &snap.resources.volumes {
            println!("  {}  {}", v.name, v.size);
        }
        println!();
    }

    println!("Domains / URLs");
    for d in &snap.domains {
        println!("  {:<12} {}", d.kind, d.url);
    }
    println!();

    println!("HTTP probes (latency)");
    for p in &snap.probes {
        let lat = p
            .latency_ms
            .map(|ms| format!("{ms}ms"))
            .unwrap_or_else(|| "—".into());
        println!(
            "  {}  {:>6}  {:<18} {}",
            if p.ok { "✓" } else { "✗" },
            lat,
            p.label,
            p.url
        );
    }
    println!();

    println!("Database");
    println!("  Host:     {}", snap.db.host);
    println!("  Port:     {}", snap.db.port);
    println!("  Database: {}", snap.db.database);
    println!("  User:     {}", snap.db.user);
    println!("  Password: {}", snap.db.password);
    println!("  URL:      {}", snap.db.url);
    match &snap.db.state {
        Ok(state) => {
            println!("  State:    {}", state.ready_line);
            if let Some(ref v) = state.version {
                println!("  Version:  {v}");
            }
            if let Some(ref s) = state.size {
                println!("  Size:     {s}");
            }
            if let Some(n) = state.sessions {
                println!("  Sessions: {n}");
            }
            if let Some(ref note) = state.note {
                println!("  Note:     {note}");
            }
        }
        Err(e) => println!("  State:    unreachable ({e})"),
    }
    println!();

    println!("Telemetry");
    println!("  Prometheus: {}", snap.telemetry.metrics_note);
    for line in &snap.telemetry.prometheus_lines {
        println!("    {line}");
    }
    if let Some(n) = snap.telemetry.unprocessed_events {
        println!("  Unprocessed telemetry_events: {n}");
    }
    for line in &snap.telemetry.recent_requests {
        println!("  req   {line}");
    }
    for line in &snap.telemetry.recent_events {
        println!("  event {line}");
    }
    println!();

    println!("Env / SMTP");
    println!("  {}", snap.env_panel.smtp_status);
    for (k, v) in &snap.env_panel.smtp_rows {
        println!("  {k}={v}");
    }
    println!("  {}", snap.env_panel.applied_hint);
    println!();
    println!("Next steps (again) — {}", guide.headline);
    for line in &guide.commands {
        if !line.starts_with('#') {
            println!("  {line}");
        }
    }
}
