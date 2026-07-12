use crate::RemoteEnv;
use crate::compose;
use crate::confirm_prompt;
use crate::preflight;
use crate::prod_pull_allowed;
use crate::repo;
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::{Command, Stdio};

/// SQL re-applied after a remote dump so Caddy `*.localhost` hosts resolve.
const LOCAL_ALIAS_SQL: &str = r#"
DO $$
DECLARE
    v_tenant_id UUID;
    v_instance_id UUID;
BEGIN
    SELECT id INTO v_tenant_id FROM tenant WHERE name = 'buildwithruud' LIMIT 1;
    IF v_tenant_id IS NOT NULL THEN
        SELECT id INTO v_instance_id FROM app_instances
          WHERE tenant_id = v_tenant_id AND app_type = 'anchor' LIMIT 1;
        IF v_instance_id IS NOT NULL THEN
            IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'buildwithruud.localhost') THEN
                INSERT INTO app_domains (id, app_instance_id, domain_name)
                VALUES (gen_random_uuid(), v_instance_id, 'buildwithruud.localhost');
            END IF;
            IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'anchor.localhost') THEN
                INSERT INTO app_domains (id, app_instance_id, domain_name)
                VALUES (gen_random_uuid(), v_instance_id, 'anchor.localhost');
            END IF;
        END IF;
    END IF;

    SELECT id INTO v_tenant_id FROM tenant WHERE name = 'oplystusa' LIMIT 1;
    IF v_tenant_id IS NOT NULL THEN
        SELECT id INTO v_instance_id FROM app_instances
          WHERE tenant_id = v_tenant_id AND app_type = 'anchor' LIMIT 1;
        IF v_instance_id IS NOT NULL THEN
            IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'oplystusa.localhost') THEN
                INSERT INTO app_domains (id, app_instance_id, domain_name)
                VALUES (gen_random_uuid(), v_instance_id, 'oplystusa.localhost');
            END IF;
        END IF;
    END IF;

    SELECT id INTO v_tenant_id FROM tenant WHERE name = 'ruuderie' LIMIT 1;
    IF v_tenant_id IS NOT NULL THEN
        SELECT id INTO v_instance_id FROM app_instances
          WHERE tenant_id = v_tenant_id AND app_type = 'property_management' LIMIT 1;
        IF v_instance_id IS NOT NULL THEN
            IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'ruuderie.localhost') THEN
                INSERT INTO app_domains (id, app_instance_id, domain_name)
                VALUES (gen_random_uuid(), v_instance_id, 'ruuderie.localhost');
            END IF;
            IF NOT EXISTS (SELECT 1 FROM app_domains WHERE domain_name = 'folio.localhost') THEN
                INSERT INTO app_domains (id, app_instance_id, domain_name)
                VALUES (gen_random_uuid(), v_instance_id, 'folio.localhost');
            END IF;
        END IF;
    END IF;
END $$;
"#;

pub fn pull(
    root: &Path,
    from: RemoteEnv,
    dry_run: bool,
    i_understand_pii: bool,
    yes: bool,
) -> Result<()> {
    if !prod_pull_allowed(from, i_understand_pii) {
        bail!(
            "Refusing to pull production without --i-understand-pii.\n\
             Prod dumps contain PII and secrets. Prefer --from dev.\n\
             See: atlas-local db pull --help"
        );
    }

    preflight::check_docker()?;
    preflight::ensure_env_files(root)?;

    let remote_url = std::env::var(from.env_var())
        .ok()
        .or_else(|| repo::read_dotenv_value(root, from.env_var()))
        .with_context(|| {
            format!(
                "{} is not set.\n\
                 Fix: export {}='postgresql://…' (SSH tunnel to NixForge) or add it to .env.local.\n\
                 Docs: docs/architecture/local_development.md",
                from.env_var(),
                from.env_var()
            )
        })?;

    let local_url = local_database_url(root)?;

    println!("Plan:");
    println!("  source: {} ({})", from.label(), from.env_var());
    println!("  target: local Compose Postgres");
    println!("  action: wipe local DB and restore dump");
    if dry_run {
        println!("dry-run: no changes made.");
        return Ok(());
    }

    if !yes {
        confirm_prompt(&format!(
            "This will WIPE your local database and replace it with a snapshot of {}. Continue?",
            from.label()
        ))?;
    }

    // Ensure postgres is up (backend can be down during restore)
    println!("→ Ensuring local postgres is running…");
    let status = Command::new("docker")
        .current_dir(root)
        .args([
            "compose",
            "--env-file",
            ".env",
            "up",
            "-d",
            "postgres",
        ])
        .status()
        .context("failed to start postgres")?;
    if !status.success() {
        bail!("Could not start postgres.\nFix: atlas-local status / atlas-local logs postgres");
    }

    // Wait briefly for readiness
    for _ in 0..30 {
        let ok = Command::new("docker")
            .args([
                "exec",
                "atlas-platform-db",
                "pg_isready",
                "-U",
                "postgres",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    require_bin("pg_dump")?;
    require_bin("pg_restore").or_else(|_| require_bin("psql"))?;

    let dump_path = std::env::temp_dir().join(format!("atlas-local-{}.dump", from.label()));
    println!("→ Dumping {} → {}", from.label(), dump_path.display());
    let dump_status = Command::new("pg_dump")
        .args(["-Fc", "--no-owner", "--no-acl", "-f"])
        .arg(&dump_path)
        .arg(&remote_url)
        .status()
        .context("pg_dump failed to start")?;
    if !dump_status.success() {
        bail!(
            "pg_dump from {} failed.\n\
             Fix: check {}, network/SSH tunnel, and that pg_dump matches the server major version.",
            from.label(),
            from.env_var()
        );
    }

    println!("→ Restoring into local database (this replaces local data)…");
    // Drop/recreate public schema via psql on local URL when possible
    let drop_status = Command::new("psql")
        .arg(&local_url)
        .args([
            "-v",
            "ON_ERROR_STOP=1",
            "-c",
            "DROP SCHEMA public CASCADE; CREATE SCHEMA public;",
        ])
        .status();
    match drop_status {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!(
                "warning: could not reset schema via psql; attempting pg_restore --clean anyway"
            );
        }
    }

    let restore = Command::new("pg_restore")
        .args(["--no-owner", "--no-acl", "--clean", "--if-exists", "-d"])
        .arg(&local_url)
        .arg(&dump_path)
        .status()
        .context("pg_restore failed to start")?;
    if !restore.success() {
        // pg_restore often exits non-zero with warnings; continue but warn
        eprintln!(
            "warning: pg_restore exited with {:?}. Check data manually if apps misbehave.",
            restore.code()
        );
    }

    println!("→ Re-applying local *.localhost domain aliases…");
    let alias = Command::new("psql")
        .arg(&local_url)
        .args(["-v", "ON_ERROR_STOP=1", "-c", LOCAL_ALIAS_SQL])
        .status();
    if !matches!(alias, Ok(s) if s.success()) {
        eprintln!(
            "warning: could not re-apply localhost aliases. Browse via server hostnames may fail;\n\
             Fix: restart backend so migrations run, or re-run after `atlas-local up`."
        );
    }

    let _ = std::fs::remove_file(&dump_path);

    println!();
    println!("Sandbox restore complete from {}.", from.label());
    println!("  Remote passkeys will NOT work on localhost (RP_ID mismatch).");
    println!("  Register a new local passkey or use magic-link/password.");
    println!("  Prefer ENVIRONMENT=development so outbox/email/Stripe stay stubbed.");
    println!("  Restart apps if they were already running: atlas-local down && atlas-local up");
    let _ = compose::status(root);
    Ok(())
}

/// Host-side connection details for Compose Postgres (port published as 5433).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDbConn {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl LocalDbConn {
    /// Resolve from `.env` / `.env.local`. Prefer `LOCAL_DATABASE_URL` (host → container
    /// mapping). Fall back to `PG*` vars with host defaults suitable for DBeaver on the Mac.
    pub fn resolve(root: &Path) -> Self {
        if let Some(url) = repo::read_dotenv_value(root, "LOCAL_DATABASE_URL") {
            if let Some(parsed) = parse_postgres_url(&url) {
                return parsed.for_host_client();
            }
        }

        let user = repo::read_dotenv_value(root, "PGUSER").unwrap_or_else(|| "postgres".into());
        let password =
            repo::read_dotenv_value(root, "PGPASSWORD").unwrap_or_else(|| "postgres".into());
        let database = repo::read_dotenv_value(root, "PGDB").unwrap_or_else(|| "oplydb".into());

        Self {
            host: "127.0.0.1".into(),
            port: 5433,
            user,
            password,
            database,
        }
        .for_host_client()
    }

    /// Rewrite Compose-internal hostnames so GUI tools on the host work.
    pub fn for_host_client(mut self) -> Self {
        if self.host == "postgres" || self.host == "localhost" {
            self.host = "127.0.0.1".into();
        }
        // Inside Compose the DB listens on 5432; on the host it is published as 5433.
        if self.port == 5432 && (self.host == "127.0.0.1" || self.host == "localhost") {
            self.port = 5433;
        }
        self
    }

    pub fn url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.database
        )
    }

    pub fn jdbc_url(&self) -> String {
        format!(
            "jdbc:postgresql://{}:{}/{}",
            self.host, self.port, self.database
        )
    }

    pub fn print_guide(&self) {
        println!("Database (Compose Postgres — connect from DBeaver / TablePlus / psql):");
        println!("  Host:     {}", self.host);
        println!("  Port:     {}", self.port);
        println!("  Database: {}", self.database);
        println!("  User:     {}", self.user);
        println!("  Password: {}", self.password);
        println!("  URL:      {}", self.url());
        println!("  JDBC:     {}", self.jdbc_url());
        println!();
        println!("  DBeaver: Database → New Connection → PostgreSQL → fill Host/Port/Database/User/Password above.");
        println!("  Tip: SSL mode = disable (local Compose has no TLS).");
        println!("  Re-print anytime: atlas-local db info");
    }
}

/// Print connection info for GUI clients (no Docker required).
pub fn info(root: &Path) -> Result<()> {
    let conn = LocalDbConn::resolve(root);
    println!();
    conn.print_guide();
    println!();
    println!("Stack must be up (`atlas-local up`) for the port to accept connections.");
    Ok(())
}

fn local_database_url(root: &Path) -> Result<String> {
    Ok(LocalDbConn::resolve(root).url())
}

/// Minimal `postgresql://user:pass@host:port/db` parser (no extra deps).
pub fn parse_postgres_url(url: &str) -> Option<LocalDbConn> {
    let rest = url
        .strip_prefix("postgresql://")
        .or_else(|| url.strip_prefix("postgres://"))?;

    let (creds, host_part) = rest.split_once('@')?;
    let (user, password) = match creds.split_once(':') {
        Some((u, p)) => (u.to_string(), p.to_string()),
        None => (creds.to_string(), String::new()),
    };

    let (host_port, database) = match host_part.split_once('/') {
        Some((hp, db)) => (hp, db.split('?').next().unwrap_or(db).to_string()),
        None => (host_part, String::new()),
    };

    let (host, port) = if let Some((h, p)) = host_port.rsplit_once(':') {
        let port: u16 = p.parse().ok()?;
        // IPv6 in brackets not supported — fine for local Compose.
        (h.to_string(), port)
    } else {
        (host_port.to_string(), 5432)
    };

    if user.is_empty() || database.is_empty() {
        return None;
    }

    Some(LocalDbConn {
        host,
        port,
        user,
        password,
        database,
    })
}

fn require_bin(name: &str) -> Result<()> {
    let ok = Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Ok(())
    } else {
        bail!(
            "`{name}` is not installed or not on PATH.\n\
             Fix: install PostgreSQL client tools (pg_dump / pg_restore / psql)."
        )
    }
}
