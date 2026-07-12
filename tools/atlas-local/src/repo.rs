use anyhow::{Context, Result, bail};
use std::env;
use std::path::{Path, PathBuf};

/// Compose services that are infrastructure (excluded from default `refresh`).
pub const INFRA_SERVICES: &[&str] = &["postgres", "proxy"];

/// Walk from cwd (then from this crate's manifest dir) until `docker-compose.yml` is found.
pub fn find_repo_root() -> Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(parent) = manifest.parent().and_then(|p| p.parent()) {
        candidates.push(parent.to_path_buf());
    }
    candidates.push(manifest);

    for start in candidates {
        let mut dir = start;
        loop {
            if dir.join("docker-compose.yml").is_file() {
                return Ok(dir);
            }
            if !dir.pop() {
                break;
            }
        }
    }
    bail!("docker-compose.yml not found")
}

pub fn compose_file(root: &Path) -> PathBuf {
    root.join("docker-compose.yml")
}

/// Service names under `services:` in `docker-compose.yml` (source of truth for the CLI).
pub fn list_compose_services(root: &Path) -> Result<Vec<String>> {
    let path = compose_file(root);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    parse_compose_service_names(&text)
}

/// Minimal YAML scrape: top-level `services:` then 2-space-indented keys.
/// Avoids a serde_yaml dependency; matches our Compose file style.
pub fn parse_compose_service_names(yaml: &str) -> Result<Vec<String>> {
    let mut in_services = false;
    let mut names = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if !in_services {
            if line.starts_with("services:") {
                in_services = true;
            }
            continue;
        }

        // Left the services block (next top-level key at column 0)
        if !line.starts_with(' ') && !line.starts_with('\t') {
            break;
        }

        // Service keys are indented exactly two spaces: `  backend:`
        if let Some(rest) = line.strip_prefix("  ") {
            if rest.starts_with(' ') || rest.starts_with('\t') || rest.starts_with('#') {
                continue; // nested property
            }
            let key = rest.split(':').next().unwrap_or("").trim();
            if !key.is_empty() && !key.starts_with('-') {
                names.push(key.to_string());
            }
        }
    }

    if names.is_empty() {
        bail!(
            "No services found under `services:` in docker-compose.yml.\n\
             Fix: check the Compose file at the atlas-platform root."
        );
    }
    Ok(names)
}

/// Default targets for `atlas-local refresh` with no args: all Compose services except infra.
pub fn default_refresh_services(all: &[String]) -> Vec<String> {
    all.iter()
        .filter(|s| !INFRA_SERVICES.contains(&s.as_str()))
        .cloned()
        .collect()
}

/// Help footer listing services discovered from Compose (+ which apps/ dirs exist).
pub fn format_services_help(root: &Path, services: &[String]) -> String {
    let mut lines = vec![
        "Modes:  atlas-local up          (parity — baked backend ≈ K8s, preferred)".to_string(),
        "        atlas-local up --hot    (cargo run + mounts; slow cold /health)".to_string(),
        "Stuck?  atlas-local status   (press x = refresh affected apps from Next steps)".to_string(),
        String::new(),
        "Available Compose services (from docker-compose.yml):".to_string(),
        format!("  {}", services.join(", ")),
        String::new(),
        "Default `atlas-local refresh` targets (excludes infra):".to_string(),
        format!("  {}", default_refresh_services(services).join(", ")),
    ];

    let apps_dir = root.join("apps");
    if apps_dir.is_dir() {
        let mut app_dirs: Vec<String> = std::fs::read_dir(&apps_dir)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| !n.starts_with('.') && n != "shared-ui" && n != "target")
            .collect();
        app_dirs.sort();
        if !app_dirs.is_empty() {
            lines.push(String::new());
            lines.push("Apps under apps/ (may map 1:1 to a Compose service):".to_string());
            lines.push(format!("  {}", app_dirs.join(", ")));
        }
    }

    lines.push(String::new());
    lines.push("List anytime: atlas-local services".to_string());
    lines.push("Docs: docs/architecture/local_development.md".to_string());
    lines.join("\n")
}

pub fn env_path(root: &Path) -> PathBuf {
    root.join(".env")
}

pub fn env_local_path(root: &Path) -> PathBuf {
    root.join(".env.local")
}

pub fn env_local_example_path(root: &Path) -> PathBuf {
    root.join(".env.local.example")
}

pub fn read_dotenv_value(root: &Path, key: &str) -> Option<String> {
    for file in [env_local_path(root), env_path(root)] {
        if let Ok(contents) = std::fs::read_to_string(&file) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    if k.trim() == key {
                        return Some(v.trim().trim_matches('"').to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn require_file(path: &Path, hint: &str) -> Result<()> {
    if path.is_file() {
        Ok(())
    } else {
        bail!("{} is missing.\nFix: {hint}", path.display())
    }
}

pub fn copy_if_missing(from: &Path, to: &Path) -> Result<bool> {
    if to.exists() {
        return Ok(false);
    }
    std::fs::copy(from, to)
        .with_context(|| format!("copy {} → {}", from.display(), to.display()))?;
    Ok(true)
}
