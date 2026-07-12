use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

fn compose_base(root: &Path) -> Command {
    let mut cmd = Command::new("docker");
    cmd.arg("compose")
        .current_dir(root)
        .arg("--env-file")
        .arg(".env");
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

pub fn up(root: &Path, build: bool) -> Result<()> {
    let mut args: Vec<&str> = vec!["up", "-d"];
    if build {
        args.push("--build");
    }
    run_compose(root, &args)
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

pub fn wait_healthy(root: &Path, timeout_secs: u64) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut ticks = 0u32;

    while Instant::now() < deadline {
        if curl_ok("http://127.0.0.1:8000/health") {
            println!();
            println!("✓ backend /health is responding");
            // Admin is nice-to-have; don't block forever if CSR image is still building
            if curl_ok("http://127.0.0.1:8081/") {
                println!("✓ platform-admin is responding");
            } else {
                println!("note: platform-admin not ready yet — check `atlas-local logs -f platform-admin`");
            }
            return Ok(());
        }

        ticks += 1;
        if ticks % 6 == 0 {
            eprintln!(
                " still waiting for backend:8000/health ({}s elapsed)…",
                ticks * 5
            );
        } else {
            eprint!(".");
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
        let _ = root; // compose project root reserved for richer probes
        thread::sleep(Duration::from_secs(5));
    }

    bail!(
        "Timed out after {timeout_secs}s waiting for the stack to become healthy.\n\
         First builds of Rust images can take a long time — re-run `atlas-local up` once images exist.\n\
         Fix: atlas-local logs -f backend\n\
         If postgres never became ready: atlas-local reset-db"
    )
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
