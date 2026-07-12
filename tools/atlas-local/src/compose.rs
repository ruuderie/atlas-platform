use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

static HOT_MODE: AtomicBool = AtomicBool::new(false);

/// Enable compose overlay `docker-compose.hot.yml` for this CLI process.
pub fn set_hot_mode(hot: bool) {
    HOT_MODE.store(hot, Ordering::Relaxed);
}

pub fn hot_mode() -> bool {
    HOT_MODE.load(Ordering::Relaxed)
        || std::env::var_os("ATLAS_LOCAL_HOT").is_some()
}

fn compose_base(root: &Path) -> Command {
    let mut cmd = Command::new("docker");
    cmd.arg("compose")
        .current_dir(root)
        .arg("-f")
        .arg("docker-compose.yml");
    if hot_mode() {
        cmd.arg("-f").arg("docker-compose.hot.yml");
    }
    cmd.arg("--env-file").arg(".env");
    if root.join(".env.local").is_file() {
        cmd.arg("--env-file").arg(".env.local");
    }
    cmd
}

fn run_compose(root: &Path, args: &[&str]) -> Result<()> {
    let status = compose_base(root)
        .args(args)
        .status()
        .context("failed to invoke docker compose")?;
    if !status.success() {
        bail!(
            "docker compose {} failed (exit {:?}).\nFix: atlas-local logs -f",
            args.join(" "),
            status.code()
        );
    }
    Ok(())
}

/// Rebuild (optional) and recreate services so containers match the working tree.
pub fn refresh(root: &Path, services: &[&str], build: bool) -> Result<()> {
    wipe_stale_frontend_dist(root);
    maybe_host_build_platform_admin(root, services)?;
    let mut args: Vec<String> = vec!["up".into(), "-d".into(), "--force-recreate".into()];
    if build {
        args.push("--build".into());
    }
    for s in services {
        args.push((*s).into());
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_compose(root, &arg_refs)
}

/// After wiping dist, build platform-admin WASM on the host when `trunk` is
/// available. In-container `rust-lld` often SIGSEGVs on this app; the
/// development entrypoint then serves the host-built dist statically.
fn maybe_host_build_platform_admin(root: &Path, services: &[&str]) -> Result<()> {
    let needs_admin = services.is_empty()
        || services.iter().any(|s| *s == "platform-admin" || *s == "all");
    if !needs_admin {
        return Ok(());
    }
    let admin = root.join("apps/platform-admin");
    if !admin.join("Cargo.toml").is_file() {
        return Ok(());
    }
    let trunk = which_trunk();
    let Some(trunk) = trunk else {
        println!(
            "note: `trunk` not on PATH — platform-admin will try in-container build \
             (slow / may SIGSEGV). Install: cargo install trunk"
        );
        return Ok(());
    };
    println!("→ host trunk build for platform-admin (avoids container rust-lld crashes)…");
    let status = Command::new(&trunk)
        .arg("build")
        .current_dir(&admin)
        .status()
        .context("failed to run trunk build")?;
    if !status.success() {
        bail!(
            "host `trunk build` failed for platform-admin (exit {:?}).\n\
             Fix: cd apps/platform-admin && trunk build",
            status.code()
        );
    }
    println!("✓ platform-admin dist/ rebuilt on host");
    Ok(())
}

fn which_trunk() -> Option<std::path::PathBuf> {
    let out = Command::new("which")
        .arg("trunk")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path.into())
    }
}

pub fn up(root: &Path, build: bool) -> Result<()> {
    wipe_stale_frontend_dist(root);
    // Full stack up includes admin — rebuild WASM on host when possible.
    maybe_host_build_platform_admin(root, &["platform-admin"])?;
    let mut args: Vec<&str> = vec!["up", "-d"];
    if build {
        args.push("--build");
    }
    run_compose(root, &args)
}

/// Remove host-mounted Trunk/cargo-leptos `dist/` folders so Compose never
/// serves a months-old WASM (admin.localhost showed a June login while source
/// matched origin/dev). Safe: `apps/*/dist/` is gitignored.
pub fn wipe_stale_frontend_dist(root: &Path) {
    const APPS: &[&str] = &["platform-admin", "folio", "anchor", "network-instance"];
    for app in APPS {
        let dist = root.join("apps").join(app).join("dist");
        if dist.is_dir() {
            match std::fs::remove_dir_all(&dist) {
                Ok(()) => println!("→ wiped stale apps/{app}/dist (forces fresh frontend build)"),
                Err(e) => eprintln!("warning: could not wipe apps/{app}/dist: {e}"),
            }
        }
    }
}

pub fn down(root: &Path) -> Result<()> {
    println!("→ Stopping local Atlas stack…");
    run_compose(root, &["down"])
}

pub fn down_volumes(root: &Path) -> Result<()> {
    run_compose(root, &["down", "-v"])
}

pub fn status(root: &Path) -> Result<()> {
    run_compose(root, &["ps"])
}

/// Captured `docker compose ps` table (stdout), for the rich status report.
pub fn ps_lines(root: &Path) -> Result<String> {
    let out = compose_base(root)
        .args(["ps", "-a"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to invoke docker compose ps")?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        bail!("{}", err.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Structured container rows for the status dashboard table.
#[derive(Debug, Clone)]
pub struct ContainerRow {
    pub service: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

pub fn ps_rows(root: &Path) -> Result<Vec<ContainerRow>> {
    let out = compose_base(root)
        .args([
            "ps",
            "-a",
            "--format",
            "{{.Service}}\t{{.State}}\t{{.Status}}\t{{.Ports}}",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to invoke docker compose ps")?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        bail!("{}", err.trim());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(4, '\t');
        let service = parts.next().unwrap_or("").to_string();
        let state = parts.next().unwrap_or("").to_string();
        let status = parts.next().unwrap_or("").to_string();
        let ports = parts.next().unwrap_or("").to_string();
        if !service.is_empty() {
            rows.push(ContainerRow {
                service,
                state,
                status,
                ports,
            });
        }
    }
    Ok(rows)
}

/// Run a command inside the Compose `postgres` service.
pub fn exec_postgres(root: &Path, args: &[&str]) -> Result<Output> {
    let mut cmd = compose_base(root);
    cmd.arg("exec").arg("-T").arg("postgres");
    for a in args {
        cmd.arg(a);
    }
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to docker compose exec postgres")
}

pub fn logs(root: &Path, follow: bool, service: Option<&str>) -> Result<()> {
    let mut cmd = compose_base(root);
    cmd.arg("logs");
    if follow {
        cmd.arg("-f");
    }
    cmd.arg("--tail").arg("200");
    if let Some(svc) = service {
        cmd.arg(svc);
    }
    let status = cmd.status().context("docker compose logs failed")?;
    if !status.success() {
        bail!("docker compose logs failed");
    }
    Ok(())
}

pub fn dump_logs_tail(root: &Path, service: &str, lines: usize) -> Result<()> {
    eprintln!("===== logs: {service} (last {lines}) =====");
    let _ = compose_base(root)
        .args(["logs", "--tail", &lines.to_string(), service])
        .status();
    Ok(())
}

/// Foreground Compose Watch — rebuilds/recreates on save for services with `develop.watch`.
pub fn watch(root: &Path) -> Result<()> {
    let status = compose_base(root)
        .args(["watch"])
        .status()
        .context("failed to invoke docker compose watch")?;
    if !status.success() {
        if status.code() == Some(130) || status.code() == Some(143) {
            return Ok(());
        }
        bail!(
            "docker compose watch failed (exit {:?}).\n\
             Fix: ensure Docker Compose v2.22+ is installed, then: atlas-local status",
            status.code()
        );
    }
    Ok(())
}

pub fn wait_healthy(root: &Path, timeout_secs: u64) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut ticks = 0u32;
    let mut last_phase = String::new();

    if hot_mode() {
        println!(
            "→ Hot mode: backend runs `cargo run` (compiles before listen).\n\
               This is slower and less like the server. Default `atlas-local up` uses a baked binary."
        );
    } else {
        println!(
            "→ Parity mode: backend is a baked binary (same shape as K8s). Waiting for /health…"
        );
    }

    while Instant::now() < deadline {
        if curl_ok("http://127.0.0.1:8000/health") {
            println!();
            println!("✓ backend /health is responding");
            if curl_ok("http://127.0.0.1:8081/") {
                println!("✓ platform-admin is responding");
            } else {
                println!(
                    "note: platform-admin not ready yet — check `atlas-local logs -f platform-admin`"
                );
            }
            return Ok(());
        }

        ticks += 1;
        let phase = detect_backend_phase(root);
        if ticks % 6 == 0 || phase != last_phase {
            eprintln!(
                " still waiting for backend:8000/health ({}s) — {phase}",
                ticks * 5
            );
            last_phase = phase;
        } else {
            eprint!(".");
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
        thread::sleep(Duration::from_secs(5));
    }

    let phase = detect_backend_phase(root);
    bail!(
        "Timed out after {timeout_secs}s waiting for backend /health.\n\
         Last observed phase: {phase}\n\
         \n\
         Why this happens locally (and NOT on the server):\n\
         - Server/K8s runs a pre-built `./atlas_backend` image → listen in seconds, then migrations.\n\
         - Local `--hot` (or old default) runs `cargo run` → can compile for many minutes before /health.\n\
         \n\
         Reconcile (recommended):\n\
           atlas-local down\n\
           atlas-local up            # parity mode — baked backend binary\n\
         Hot reload only when you need it:\n\
           atlas-local up --hot\n\
         \n\
         Diagnose: atlas-local logs -f backend\n\
         Corrupt DB volume: atlas-local reset-db"
    )
}

fn detect_backend_phase(root: &Path) -> String {
    let out = compose_base(root)
        .args(["logs", "--tail", "80", "backend"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();
    let Ok(out) = out else {
        return "unknown (could not read backend logs)".into();
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let lower = text.to_lowercase();
    if lower.contains("compiling") || lower.contains("cargo run") {
        return "compiling Rust (cargo) — not an app failure; wait or switch to parity `up`".into();
    }
    if text.contains("error[E") || lower.contains("could not compile") {
        return "compile ERROR — atlas-local logs -f backend".into();
    }
    if lower.contains("migrat") {
        return "running DB migrations".into();
    }
    if lower.contains("listening") || lower.contains("server started") {
        return "process up but /health not ready yet".into();
    }
    if text.trim().is_empty() {
        return "container starting / image still building".into();
    }
    "starting".into()
}

fn curl_ok(url: &str) -> bool {
    Command::new("curl")
        .args(["-sf", "--max-time", "3", url])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
