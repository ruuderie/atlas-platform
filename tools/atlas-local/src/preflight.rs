use crate::repo;
use anyhow::{Context, Result, bail};
use std::net::TcpListener;
use std::path::Path;
use std::process::Command;

pub fn check_docker() -> Result<()> {
    if which("docker").is_err() {
        bail!(
            "docker is not installed or not on PATH.\n\
             Fix: install Docker Desktop (macOS/Windows) or Docker Engine + Compose plugin (Linux)."
        );
    }
    let status = Command::new("docker")
        .args(["info"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .context("failed to run `docker info`")?;
    if !status.success() {
        bail!(
            "Docker is installed but the daemon is not running.\n\
             Fix: start Docker Desktop, OrbStack, or `colima start`, then re-run `atlas-local up`."
        );
    }
    // Prefer `docker compose` plugin
    let compose_ok = Command::new("docker")
        .args(["compose", "version"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !compose_ok {
        bail!(
            "`docker compose` plugin is missing.\n\
             Fix: install Docker Compose V2 (bundled with Docker Desktop)."
        );
    }
    Ok(())
}

fn which(bin: &str) -> Result<std::path::PathBuf> {
    let path = env_path_var();
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Ok(candidate);
        }
        #[cfg(windows)]
        {
            let exe = dir.join(format!("{bin}.exe"));
            if exe.is_file() {
                return Ok(exe);
            }
        }
    }
    bail!("{bin} not found on PATH")
}

fn env_path_var() -> String {
    std::env::var_os("PATH")
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default()
}

pub fn ensure_env_files(root: &Path) -> Result<()> {
    repo::require_file(
        &repo::env_path(root),
        "restore the base .env with DB credentials (PGUSER/PGPASSWORD/PGDB).",
    )?;
    let example = repo::env_local_example_path(root);
    repo::require_file(
        &example,
        "repo is incomplete — .env.local.example should be tracked in git.",
    )?;
    let local = repo::env_local_path(root);
    if repo::copy_if_missing(&example, &local)? {
        println!(
            "→ Created {} from .env.local.example (edit WebAuthn/DB pull URLs if needed)",
            local.display()
        );
    }
    Ok(())
}

pub fn warn_webauthn_orb_local(root: &Path) {
    let origin = repo::read_dotenv_value(root, "WEBAUTHN_ORIGIN").unwrap_or_default();
    let rp = repo::read_dotenv_value(root, "RP_ID").unwrap_or_default();
    if looks_like_orb_local_webauthn(&origin, &rp) {
        eprintln!(
            "warning: WEBAUTHN_ORIGIN/RP_ID still reference orb.local.\n\
             Local passkeys will fail on admin.localhost.\n\
             Fix: ensure .env.local overrides to WEBAUTHN_ORIGIN=http://admin.localhost and RP_ID=localhost"
        );
    }
}

/// Pure helper — used by unit tests and `warn_webauthn_orb_local`.
pub fn looks_like_orb_local_webauthn(origin: &str, rp_id: &str) -> bool {
    origin.contains("orb.local") || rp_id.contains("orb.local")
}

/// Best-effort port check. Does not fail if we cannot bind (race); warns when occupied
/// outside our compose project when detectable.
pub fn check_ports(ports: &[u16]) -> Result<()> {
    for &port in ports {
        if TcpListener::bind(("127.0.0.1", port)).is_err() {
            // Port busy — may be our own stack; that's fine for `up` (compose will reuse).
            eprintln!(
                "note: port {port} is in use. If this is not Atlas Compose, free it or run `atlas-local down` first."
            );
        }
    }
    Ok(())
}
