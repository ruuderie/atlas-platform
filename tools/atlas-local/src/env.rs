//! Local env file helpers — `.env.local` is the writable overlay for Compose.

use crate::preflight;
use crate::repo;
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

const SECRET_HINTS: &[&str] = &[
    "TOKEN",
    "PASSWORD",
    "SECRET",
    "PRIVATE",
    "CREDENTIAL",
    "ACCESS_KEY",
];

/// Keys operators commonly need for local SMTP (magic links, invites, OTP).
pub const SMTP_KEYS: &[&str] = &[
    "SMTP_SERVER",
    "SMTP_PORT",
    "SMTP_USERNAME",
    "SMTP_TOKEN",
    "SMTP_FROM",
];

/// Cloudflare R2 keys for Folio vault / PhotoMediaCard uploads.
/// Backend presign requires ACCESS_KEY_ID + ENDPOINT (secret needed for signed PUT).
pub const R2_KEYS: &[&str] = &[
    "R2_ACCESS_KEY_ID",
    "R2_SECRET_ACCESS_KEY",
    "R2_ENDPOINT",
];

pub fn cmd_list(root: &Path, reveal: bool) -> Result<()> {
    preflight::ensure_env_files(root)?;
    let local = repo::env_local_path(root);
    let pairs = read_dotenv_pairs(&local)?;
    if pairs.is_empty() {
        println!("{} is empty (or comments only).", local.display());
        println!("Set values: atlas-local env set KEY=value");
        return Ok(());
    }
    println!("Effective local overlay: {}", local.display());
    println!("(Compose loads .env then .env.local — local wins on duplicate keys.)\n");
    for (k, v) in pairs {
        let display = if reveal || !is_secret_key(&k) {
            v
        } else {
            mask_secret(&v)
        };
        println!("{k}={display}");
    }
    Ok(())
}

pub fn cmd_get(root: &Path, key: &str, reveal: bool) -> Result<()> {
    preflight::ensure_env_files(root)?;
    match repo::read_dotenv_value(root, key) {
        Some(v) => {
            let source = if read_dotenv_pairs(&repo::env_local_path(root))?
                .iter()
                .any(|(k, _)| k == key)
            {
                ".env.local"
            } else {
                ".env"
            };
            let display = if reveal || !is_secret_key(key) {
                v
            } else {
                mask_secret(&v)
            };
            println!("{key}={display}  # from {source}");
            Ok(())
        }
        None => bail!(
            "{key} is not set in .env.local or .env.\n\
             Fix: atlas-local env set {key}=…"
        ),
    }
}

pub fn cmd_set(root: &Path, raw_args: &[String]) -> Result<()> {
    preflight::ensure_env_files(root)?;
    let updates = parse_set_args(raw_args)?;
    if updates.is_empty() {
        bail!(
            "Nothing to set.\n\
             Usage: atlas-local env set KEY=value\n\
                    atlas-local env set KEY value\n\
                    atlas-local env set KEY1=v1 KEY2=v2"
        );
    }
    let path = repo::env_local_path(root);
    upsert_dotenv_file(&path, &updates)?;
    println!("✓ Updated {}:", path.display());
    for (k, v) in &updates {
        let display = if is_secret_key(k) {
            mask_secret(v)
        } else {
            v.clone()
        };
        println!("  {k}={display}");
    }
    println!();
    println!("Apply to running containers:");
    println!("  cargo run -p atlas-local -- refresh backend");
    if updates.iter().any(|(k, _)| k.starts_with("SMTP_")) {
        println!();
        println!("SMTP tip: empty/localhost SMTP_SERVER → backend mocks email (logs only).");
        println!("  Check: atlas-local env smtp");
    }
    if updates.iter().any(|(k, _)| k.starts_with("R2_")) {
        println!();
        println!("R2 tip: vault photo upload needs ACCESS_KEY_ID + ENDPOINT (+ SECRET for PUT).");
        println!("  Check: atlas-local env r2");
    }
    Ok(())
}

pub fn cmd_unset(root: &Path, keys: &[String]) -> Result<()> {
    preflight::ensure_env_files(root)?;
    if keys.is_empty() {
        bail!("Usage: atlas-local env unset KEY [KEY…]");
    }
    let path = repo::env_local_path(root);
    let removed = unset_dotenv_keys(&path, keys)?;
    if removed.is_empty() {
        println!("No matching keys in {}.", path.display());
    } else {
        println!("✓ Removed from {}:", path.display());
        for k in removed {
            println!("  {k}");
        }
        println!();
        println!("Apply: cargo run -p atlas-local -- refresh backend");
    }
    Ok(())
}

pub fn cmd_path(root: &Path) -> Result<()> {
    preflight::ensure_env_files(root)?;
    println!("{}", repo::env_local_path(root).display());
    Ok(())
}

pub fn cmd_edit(root: &Path) -> Result<()> {
    preflight::ensure_env_files(root)?;
    let path = repo::env_local_path(root);
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".into());
    println!("→ Opening {} with {editor}…", path.display());
    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to launch EDITOR={editor}"))?;
    if !status.success() {
        bail!("{editor} exited with {:?}", status.code());
    }
    println!("Saved. Apply with: cargo run -p atlas-local -- refresh backend");
    Ok(())
}

pub fn cmd_smtp(root: &Path, reveal: bool) -> Result<()> {
    preflight::ensure_env_files(root)?;
    println!("Local SMTP (magic links / invites / OTP)\n");
    for key in SMTP_KEYS {
        match repo::read_dotenv_value(root, key) {
            Some(v) => {
                let display = if reveal || !is_secret_key(key) {
                    v.clone()
                } else {
                    mask_secret(&v)
                };
                let src = if key_in_local(root, key) {
                    ".env.local"
                } else {
                    ".env"
                };
                println!("  {key}={display}  # {src}");
            }
            None => println!("  {key}=(unset)"),
        }
    }
    println!();
    let server = repo::read_dotenv_value(root, "SMTP_SERVER").unwrap_or_default();
    if smtp_is_mock(&server) {
        println!("Status: MOCK — emails are logged, not delivered.");
        println!("  SMTP_SERVER is empty or \"localhost\".");
        println!();
        println!("To send real mail locally, set all of:");
        println!("  atlas-local env set SMTP_SERVER=smtp.example.com");
        println!("  atlas-local env set SMTP_PORT=587");
        println!("  atlas-local env set SMTP_USERNAME=your-user");
        println!("  atlas-local env set SMTP_TOKEN=your-password-or-api-token");
        println!("  atlas-local env set SMTP_FROM='Atlas Local <noreply@example.com>'");
        println!("  cargo run -p atlas-local -- refresh backend");
    } else {
        let missing: Vec<_> = SMTP_KEYS
            .iter()
            .copied()
            .filter(|k| {
                *k != "SMTP_SERVER"
                    && repo::read_dotenv_value(root, k)
                        .map(|v| v.trim().is_empty())
                        .unwrap_or(true)
            })
            .collect();
        if missing.is_empty() {
            println!("Status: CONFIGURED — backend will attempt real SMTP to {server}");
            println!("  After edits: cargo run -p atlas-local -- refresh backend");
        } else {
            println!("Status: INCOMPLETE — SMTP_SERVER={server} but missing: {}", missing.join(", "));
            println!("  atlas-local env set KEY=value …");
        }
    }
    Ok(())
}

fn key_in_local(root: &Path, key: &str) -> bool {
    read_dotenv_pairs(&repo::env_local_path(root))
        .ok()
        .map(|p| p.iter().any(|(k, _)| k == key))
        .unwrap_or(false)
}

pub fn smtp_is_mock(server: &str) -> bool {
    let s = server.trim();
    s.is_empty() || s.eq_ignore_ascii_case("localhost") || s == "127.0.0.1"
}

/// R2 readiness for local vault uploads (matches Folio `presign_upload` gate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum R2Readiness {
    /// ACCESS_KEY_ID + ENDPOINT + SECRET all set — presign can succeed.
    Ready,
    /// ACCESS_KEY_ID + ENDPOINT set but SECRET missing — PUT will fail.
    Incomplete,
    /// Presign returns 501 (missing ACCESS_KEY_ID and/or ENDPOINT).
    NotConfigured,
}

pub fn r2_readiness_from_values(access: &str, secret: &str, endpoint: &str) -> R2Readiness {
    let access_ok = !access.trim().is_empty();
    let endpoint_ok = !endpoint.trim().is_empty();
    let secret_ok = !secret.trim().is_empty();
    if access_ok && endpoint_ok && secret_ok {
        R2Readiness::Ready
    } else if access_ok && endpoint_ok {
        R2Readiness::Incomplete
    } else {
        R2Readiness::NotConfigured
    }
}

pub fn r2_readiness(root: &Path) -> R2Readiness {
    r2_readiness_from_values(
        &repo::read_dotenv_value(root, "R2_ACCESS_KEY_ID").unwrap_or_default(),
        &repo::read_dotenv_value(root, "R2_SECRET_ACCESS_KEY").unwrap_or_default(),
        &repo::read_dotenv_value(root, "R2_ENDPOINT").unwrap_or_default(),
    )
}

pub fn r2_status_line(root: &Path) -> String {
    match r2_readiness(root) {
        R2Readiness::Ready => {
            let endpoint = repo::read_dotenv_value(root, "R2_ENDPOINT").unwrap_or_default();
            format!("READY → {endpoint} (vault / PhotoMediaCard can upload)")
        }
        R2Readiness::Incomplete => {
            "INCOMPLETE — ACCESS_KEY_ID + ENDPOINT set, but R2_SECRET_ACCESS_KEY missing".into()
        }
        R2Readiness::NotConfigured => {
            "NOT CONFIGURED — vault presign returns 501 (set R2_ACCESS_KEY_ID + R2_ENDPOINT)".into()
        }
    }
}

pub fn cmd_r2(root: &Path, reveal: bool) -> Result<()> {
    preflight::ensure_env_files(root)?;
    println!("Cloudflare R2 (Folio vault / PhotoMediaCard uploads)\n");
    for key in R2_KEYS {
        match repo::read_dotenv_value(root, key) {
            Some(v) => {
                let display = if reveal || !is_secret_key(key) {
                    v.clone()
                } else {
                    mask_secret(&v)
                };
                let src = if key_in_local(root, key) {
                    ".env.local"
                } else {
                    ".env"
                };
                println!("  {key}={display}  # {src}");
            }
            None => println!("  {key}=(unset)"),
        }
    }
    println!();
    println!("Status: {}", r2_status_line(root));
    println!("  Bucket (hardcoded in backend): atlas-tenant-vault");
    println!();
    match r2_readiness(root) {
        R2Readiness::Ready => {
            println!("Apply after edits:");
            println!("  cargo run -p atlas-local -- refresh backend");
            println!("Smoke: upload a photo on Folio property hub (needs backend recreate).");
        }
        R2Readiness::Incomplete | R2Readiness::NotConfigured => {
            println!("To enable local vault uploads, set:");
            println!("  atlas-local env set R2_ACCESS_KEY_ID=…");
            println!("  atlas-local env set R2_SECRET_ACCESS_KEY=…");
            println!("  atlas-local env set R2_ENDPOINT=https://<accountid>.r2.cloudflarestorage.com");
            println!("  cargo run -p atlas-local -- refresh backend");
            println!();
            println!("Without these, POST /api/folio/vault/presign → 501 Not Implemented.");
        }
    }
    Ok(())
}

pub fn is_secret_key(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    SECRET_HINTS.iter().any(|h| upper.contains(h))
}

pub fn mask_secret(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    if value.len() <= 4 {
        return "****".into();
    }
    format!("{}…{}", &value[..2], &value[value.len().saturating_sub(2)..])
}

pub fn parse_set_args(raw: &[String]) -> Result<Vec<(String, String)>> {
    if raw.is_empty() {
        return Ok(Vec::new());
    }
    // KEY value
    if raw.len() == 2 && !raw[0].contains('=') {
        validate_key(&raw[0])?;
        return Ok(vec![(raw[0].clone(), raw[1].clone())]);
    }
    let mut out = Vec::new();
    for arg in raw {
        let Some((k, v)) = arg.split_once('=') else {
            bail!(
                "Expected KEY=value (got `{arg}`).\n\
                 Or: atlas-local env set KEY value"
            );
        };
        let k = k.trim();
        validate_key(k)?;
        out.push((k.to_string(), v.to_string()));
    }
    Ok(out)
}

fn validate_key(key: &str) -> Result<()> {
    if key.is_empty()
        || !key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        bail!("Invalid env key `{key}` — use A-Z, 0-9, underscore.");
    }
    Ok(())
}

pub fn read_dotenv_pairs(path: &Path) -> Result<Vec<(String, String)>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let mut pairs = Vec::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            pairs.push((
                k.trim().to_string(),
                v.trim().trim_matches('"').to_string(),
            ));
        }
    }
    Ok(pairs)
}

pub fn upsert_dotenv_file(path: &Path, updates: &[(String, String)]) -> Result<()> {
    let mut lines: Vec<String> = if path.is_file() {
        std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?
            .lines()
            .map(str::to_string)
            .collect()
    } else {
        Vec::new()
    };

    for (key, value) in updates {
        let rendered = format_assignment(key, value);
        let mut found = false;
        for line in lines.iter_mut() {
            let trimmed = line.trim_start();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((k, _)) = trimmed.split_once('=') {
                if k.trim() == key {
                    *line = rendered.clone();
                    found = true;
                    break;
                }
            }
        }
        if !found {
            if let Some(last) = lines.last() {
                if !last.is_empty() {
                    lines.push(String::new());
                }
            }
            lines.push(rendered);
        }
    }

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn unset_dotenv_keys(path: &Path, keys: &[String]) -> Result<Vec<String>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    let want: std::collections::HashSet<&str> = keys.iter().map(String::as_str).collect();
    let mut removed = Vec::new();
    let mut kept = Vec::new();
    for line in contents.lines() {
        let trimmed = line.trim_start();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some((k, _)) = trimmed.split_once('=') {
                let k = k.trim();
                if want.contains(k) {
                    removed.push(k.to_string());
                    continue;
                }
            }
        }
        kept.push(line.to_string());
    }
    let mut out = kept.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(path, out).with_context(|| format!("write {}", path.display()))?;
    removed.sort();
    removed.dedup();
    Ok(removed)
}

fn format_assignment(key: &str, value: &str) -> String {
    if value.is_empty()
        || value.chars().any(|c| c.is_whitespace() || c == '#' || c == '"')
    {
        let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
        format!("{key}=\"{escaped}\"")
    } else {
        format!("{key}={value}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_set_key_eq_value() {
        let u = parse_set_args(&[String::from("SMTP_SERVER=smtp.example.com")]).unwrap();
        assert_eq!(u, vec![("SMTP_SERVER".into(), "smtp.example.com".into())]);
    }

    #[test]
    fn parse_set_key_space_value() {
        let u = parse_set_args(&[String::from("SMTP_PORT"), String::from("587")]).unwrap();
        assert_eq!(u, vec![("SMTP_PORT".into(), "587".into())]);
    }

    #[test]
    fn parse_set_multiple() {
        let u = parse_set_args(&[
            String::from("A=1"),
            String::from("B=two words"),
        ])
        .unwrap();
        assert_eq!(u.len(), 2);
        assert_eq!(u[1].1, "two words");
    }

    #[test]
    fn smtp_mock_detection() {
        assert!(smtp_is_mock(""));
        assert!(smtp_is_mock("localhost"));
        assert!(smtp_is_mock("LOCALHOST"));
        assert!(!smtp_is_mock("smtp.gmail.com"));
    }

    #[test]
    fn r2_readiness_states() {
        assert_eq!(
            r2_readiness_from_values("", "", ""),
            R2Readiness::NotConfigured
        );
        assert_eq!(
            r2_readiness_from_values("ak", "sk", ""),
            R2Readiness::NotConfigured
        );
        assert_eq!(
            r2_readiness_from_values("ak", "", "https://example.r2.cloudflarestorage.com"),
            R2Readiness::Incomplete
        );
        assert_eq!(
            r2_readiness_from_values("ak", "sk", "https://example.r2.cloudflarestorage.com"),
            R2Readiness::Ready
        );
    }

    #[test]
    fn access_key_is_masked_as_secret() {
        assert!(is_secret_key("R2_ACCESS_KEY_ID"));
        assert!(is_secret_key("R2_SECRET_ACCESS_KEY"));
    }

    #[test]
    fn upsert_preserves_comments_and_updates() {
        let dir = std::env::temp_dir().join(format!("atlas-env-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env.local");
        fs::write(
            &path,
            "# header\nRP_ID=localhost\nSMTP_SERVER=old\n# trailer\n",
        )
        .unwrap();
        upsert_dotenv_file(
            &path,
            &[
                ("SMTP_SERVER".into(), "smtp.new".into()),
                ("SMTP_PORT".into(), "587".into()),
            ],
        )
        .unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("# header"));
        assert!(body.contains("RP_ID=localhost"));
        assert!(body.contains("SMTP_SERVER=smtp.new"));
        assert!(body.contains("SMTP_PORT=587"));
        assert!(!body.contains("SMTP_SERVER=old"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unset_removes_key() {
        let dir = std::env::temp_dir().join(format!("atlas-env-unset-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(".env.local");
        fs::write(&path, "A=1\nB=2\nC=3\n").unwrap();
        let removed = unset_dotenv_keys(&path, &["B".into()]).unwrap();
        assert_eq!(removed, vec!["B".to_string()]);
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("A=1"));
        assert!(!body.contains("B=2"));
        assert!(body.contains("C=3"));
        let _ = fs::remove_dir_all(&dir);
    }
}
