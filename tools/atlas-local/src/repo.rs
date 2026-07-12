use anyhow::{Context, Result, bail};
use std::env;
use std::path::{Path, PathBuf};

/// Walk from cwd (then from this crate's manifest dir) until `docker-compose.yml` is found.
pub fn find_repo_root() -> Result<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        candidates.push(cwd);
    }
    // When run via `cargo run -p atlas-local`, CARGO_MANIFEST_DIR is tools/atlas-local
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
