//! Library surface for `atlas-local` — keeps the binary thin and unit-testable.
//!
//! Extension policy: new local automation belongs here (subcommands / modules),
//! not one-off shell scripts. See `docs/architecture/local_development.md`.

pub mod compose;
pub mod db;
pub mod env;
pub mod preflight;
pub mod repo;
pub mod status;

use anyhow::{Context, Result, bail};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "atlas-local",
    version,
    about = "Local Atlas Platform tooling — parity by default (matches server), optional --hot",
    long_about = "Manages the Docker Compose + Caddy local stack for Atlas Platform.\n\n\
PARITY (default `up`): baked backend binary — same runtime shape as K8s. Prefer this \
when you want confidence that matches deployed envs.\n\
HOT (`up --hot` / `watch`): volume mounts + cargo run — faster iteration, slower cold \
boot; can look \"broken\" while still compiling.\n\n\
After failures, run `atlas-local status` — Overview Next steps shows what to run.\n\
In the TUI press `x` to execute the first suggested `refresh <services>` \
(affected apps only). `r` only reloads the status panel.\n\n\
WebAuthn is isolated (RP_ID=localhost in .env.local); never copy into K8s overlays.\n\
Docs: docs/architecture/local_development.md"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the stack (parity by default — baked backend ≈ K8s)
    #[command(
        long_about = "Start Compose + Caddy.\n\n\
Default is PARITY mode: backend is a baked binary (like the server). First image \
build can take a while; later boots reach /health in seconds.\n\n\
--hot applies docker-compose.hot.yml (cargo run + mounts). Use only when you need \
a volume-mounted Rust loop. Cold compile can take many minutes before /health.\n\n\
If up times out: atlas-local status  (Next steps)  or  logs -f backend"
    )]
    Up {
        /// Skip Docker image rebuild
        #[arg(long)]
        no_build: bool,
        /// Hot mode: mounts + cargo run (diverges from server; slow first /health)
        #[arg(long)]
        hot: bool,
    },
    /// Stop the local stack
    Down,
    /// Live dashboard — press ? for sync guide, x to refresh from Next steps
    #[command(
        long_about = "Ratatui dashboard (TTY) or plain text (--plain / pipes).\n\n\
Tabs: 1 Overview · 2 Resources · 3 Telemetry · 4 Env (SMTP / .env.local).\n\n\
Overview → Next steps lists state-aware copy-paste commands.\n\
  ?  sync guide — after a Rust/UI/.env change, which command updates the running app\n\
     (parity refresh vs hot watch vs --no-build vs trunk for platform-admin)\n\
  x  run the first `refresh …` from Next steps (honors --no-build on that line)\n\
  r  reload this status panel only (does NOT recreate containers)\n\
  a  (Env tab) apply .env.local to backend (recreate backend)\n\
  s/e (Env tab) set SMTP / open editor\n\
  q  quit · tab / 1–4 switch tabs · auto-refresh panel every 3s\n\n\
Modes: PARITY (`up`) baked binaries ≈ server — edit then `refresh`.\n\
       HOT (`up --hot` / `watch`) live mounts — use for tight Rust loops only.\n\n\
CLI equivalent of x: cargo run -p atlas-local -- refresh <services…>"
    )]
    Status {
        /// Skip the TUI; print plain text (also used when stdout is not a TTY)
        #[arg(long)]
        plain: bool,
    },
    /// Follow or dump Compose logs
    Logs {
        /// Follow log output
        #[arg(short = 'f', long)]
        follow: bool,
        /// Optional service name (postgres, backend, platform-admin, …)
        service: Option<String>,
    },
    /// Recreate app containers so they match your latest saves (one-shot)
    #[command(
        long_about = "Force-recreate (and optionally rebuild) services so containers match \
your working tree. Omit SERVICE to refresh all app services (excludes postgres/proxy).\n\n\
Default includes Docker `--build` (parity backend Rust needs this).\n\
Pass `--no-build` for env-only recreate or when HOT mounts already have sources.\n\n\
platform-admin: always wipes dist/ and runs host `trunk build` when that service is targeted \
(WASM — often the slow step).\n\n\
Live alternative (HOT only): `atlas-local watch`.\n\
See also: `atlas-local status` then press `?` for the sync cookbook."
    )]
    Refresh {
        /// Compose service name(s); omit for all app services (see `atlas-local services`)
        #[arg(value_name = "SERVICE")]
        services: Vec<String>,
        /// Skip image rebuild (faster when only volume-mounted services changed)
        #[arg(long)]
        no_build: bool,
    },
    /// Compose Watch on save (implies hot mode)
    #[command(
        long_about = "Foreground `docker compose watch`. Implies hot mode \
(docker-compose.hot.yml). Prefer parity `up` + `refresh` when you want server-like confidence."
    )]
    Watch,
    /// List Compose services discovered from docker-compose.yml (+ apps/ dirs)
    Services,
    /// Destroy local Postgres volume and recreate (confirm required)
    #[command(name = "reset-db")]
    ResetDb {
        /// Skip interactive confirmation
        #[arg(long)]
        yes: bool,
    },
    /// Database sandbox commands
    Db {
        #[command(subcommand)]
        command: DbCommands,
    },
    /// Manage local `.env.local` (SMTP, WebAuthn, tokens, …)
    #[command(
        long_about = "Read/write the gitignored .env.local overlay used by Docker Compose.\n\n\
Never commit secrets. After set/unset/edit: refresh backend so containers reload env.\n\n\
SMTP: empty or localhost SMTP_SERVER → backend mocks email (logs only).\n\
  atlas-local env smtp          # status + template\n\
  atlas-local env set SMTP_SERVER=smtp.example.com\n\
  atlas-local env set SMTP_TOKEN=…"
    )]
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum EnvCommands {
    /// List keys in `.env.local` (secrets masked unless --reveal)
    List {
        /// Show secret values in full
        #[arg(long)]
        reveal: bool,
    },
    /// Get one key (effective: .env.local then .env)
    Get {
        key: String,
        #[arg(long)]
        reveal: bool,
    },
    /// Create or update KEY in `.env.local`
    #[command(
        long_about = "Upsert into .env.local.\n\
  atlas-local env set SMTP_SERVER=smtp.example.com\n\
  atlas-local env set SMTP_PORT 587\n\
  atlas-local env set A=1 B=2"
    )]
    Set {
        /// KEY=value and/or KEY value
        #[arg(required = true)]
        args: Vec<String>,
    },
    /// Remove KEY(s) from `.env.local`
    Unset {
        #[arg(required = true)]
        keys: Vec<String>,
    },
    /// Open `.env.local` in $EDITOR
    Edit,
    /// Print path to `.env.local`
    Path,
    /// SMTP status + how to configure real delivery
    Smtp {
        #[arg(long)]
        reveal: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum DbCommands {
    /// Show local Postgres connection details (DBeaver / TablePlus / psql)
    #[command(
        long_about = "Print Host/Port/Database/User/Password/URL/JDBC for GUI clients.\n\
Host tools use 127.0.0.1:5433 (Compose publishes 5433→5432). SSL = disable locally."
    )]
    Info,
    /// Pull a remote environment database into local Postgres (wipes local data)
    Pull {
        /// Source environment
        #[arg(long, value_enum)]
        from: RemoteEnv,
        /// Show connection target / plan without writing
        #[arg(long)]
        dry_run: bool,
        /// Required when --from prod (PII risk)
        #[arg(long)]
        i_understand_pii: bool,
        /// Skip interactive confirmation
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum RemoteEnv {
    Dev,
    Uat,
    Prod,
}

impl RemoteEnv {
    pub fn env_var(self) -> &'static str {
        match self {
            Self::Dev => "ATLAS_DEV_DATABASE_URL",
            Self::Uat => "ATLAS_UAT_DATABASE_URL",
            Self::Prod => "ATLAS_PROD_DATABASE_URL",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dev => "atlas_dev",
            Self::Uat => "atlas_uat",
            Self::Prod => "atlas_prod",
        }
    }
}

/// Pure gate used by `db pull` — unit-tested so prod never becomes accidental.
pub fn prod_pull_allowed(from: RemoteEnv, i_understand_pii: bool) -> bool {
    !matches!(from, RemoteEnv::Prod) || i_understand_pii
}

pub fn run() -> Result<()> {
    let root_for_help = repo::find_repo_root().ok();
    let services_help = root_for_help
        .as_ref()
        .and_then(|r| {
            repo::list_compose_services(r)
                .ok()
                .map(|svcs| repo::format_services_help(r, &svcs))
        })
        .unwrap_or_else(|| {
            "Available services: run from atlas-platform/ (needs docker-compose.yml),\n\
             or: atlas-local services"
                .to_string()
        });

    let mut cmd = Cli::command().after_help(services_help.clone());
    cmd = cmd.mut_subcommand("refresh", |c| c.after_help(services_help.clone()));
    cmd = cmd.mut_subcommand("services", |c| c.after_help(services_help.clone()));

    let matches = cmd.get_matches();
    let cli = Cli::from_arg_matches(&matches).map_err(|e| anyhow::anyhow!("{e}"))?;
    dispatch(cli)
}

pub fn dispatch(cli: Cli) -> Result<()> {
    let root = repo::find_repo_root().context(
        "Could not find atlas-platform root (looking for docker-compose.yml).\n\
         Fix: cd into atlas-platform/ (or a subdirectory) and re-run.",
    )?;

    match cli.command {
        Commands::Up { no_build, hot } => cmd_up(&root, no_build, hot),
        Commands::Down => {
            detect_and_set_hot_mode(&root);
            preflight::check_docker()?;
            compose::down(&root)
        }
        Commands::Status { plain } => {
            detect_and_set_hot_mode(&root);
            preflight::check_docker()?;
            status::print_report(&root, plain)
        }
        Commands::Logs { follow, service } => {
            detect_and_set_hot_mode(&root);
            preflight::check_docker()?;
            compose::logs(&root, follow, service.as_deref())
        }
        Commands::Refresh { services, no_build } => {
            detect_and_set_hot_mode(&root);
            cmd_refresh(&root, &services, !no_build)
        }
        Commands::Watch => {
            compose::set_hot_mode(true);
            persist_mode(&root, true)?;
            cmd_watch(&root)
        }
        Commands::Services => cmd_services(&root),
        Commands::ResetDb { yes } => {
            detect_and_set_hot_mode(&root);
            cmd_reset_db(&root, yes)
        },
        Commands::Db {
            command: DbCommands::Info,
        } => db::info(&root),
        Commands::Db {
            command: DbCommands::Pull {
                from,
                dry_run,
                i_understand_pii,
                yes,
            },
        } => db::pull(&root, from, dry_run, i_understand_pii, yes),
        Commands::Env { command } => match command {
            EnvCommands::List { reveal } => env::cmd_list(&root, reveal),
            EnvCommands::Get { key, reveal } => env::cmd_get(&root, &key, reveal),
            EnvCommands::Set { args } => env::cmd_set(&root, &args),
            EnvCommands::Unset { keys } => env::cmd_unset(&root, &keys),
            EnvCommands::Edit => env::cmd_edit(&root),
            EnvCommands::Path => env::cmd_path(&root),
            EnvCommands::Smtp { reveal } => env::cmd_smtp(&root, reveal),
        },
    }
}

fn cmd_up(root: &PathBuf, no_build: bool, hot: bool) -> Result<()> {
    compose::set_hot_mode(hot);
    persist_mode(root, hot)?;

    preflight::check_docker()?;
    preflight::ensure_env_files(root)?;
    preflight::warn_webauthn_orb_local(root);
    preflight::check_ports(&[80, 8000, 8080, 8081, 3100, 3000])?;

    if hot {
        println!("→ Starting local Atlas stack in HOT mode (cargo run — diverges from server)…");
        println!("  Prefer `atlas-local up` (parity) when you want server-like confidence.");
    } else {
        println!("→ Starting local Atlas stack in PARITY mode (baked backend binary ≈ K8s)…");
    }
    compose::up(root, !no_build)?;

    // Parity boots fast; hot first compile can take a long time.
    let timeout = if hot { 900 } else { 300 };
    println!("→ Waiting for health (timeout {timeout}s)…");
    if let Err(e) = compose::wait_healthy(root, timeout) {
        let _ = compose::dump_logs_tail(root, "postgres", 40);
        let _ = compose::dump_logs_tail(root, "backend", 40);
        let snap = status::StatusSnapshot::collect(root);
        let guide = snap.guidance();
        eprintln!();
        eprintln!("Next steps — {}", guide.headline);
        for line in &guide.commands {
            eprintln!("  {line}");
        }
        bail!(
            "{e}\n\
             Re-check after trying the commands above: atlas-local status\n\
             Fix: atlas-local logs -f\n\
             If the database volume is corrupt: atlas-local reset-db"
        );
    }

    print_url_map(root);
    println!();
    if hot {
        println!("Sync: use `atlas-local watch` or `atlas-local refresh` after saves.");
    } else {
        println!("Sync: code changes need `atlas-local refresh` (rebuilds the parity image).");
        println!("  For cargo/trunk volume mounts: atlas-local up --hot");
    }
    Ok(())
}

fn mode_file(root: &Path) -> PathBuf {
    root.join(".atlas-local-mode")
}

fn persist_mode(root: &Path, hot: bool) -> Result<()> {
    let label = if hot { "hot" } else { "parity" };
    std::fs::write(mode_file(root), format!("{label}\n"))
        .with_context(|| format!("write {}", mode_file(root).display()))?;
    Ok(())
}

fn detect_and_set_hot_mode(root: &Path) {
    let hot = std::fs::read_to_string(mode_file(root))
        .map(|s| s.trim() == "hot")
        .unwrap_or(false);
    compose::set_hot_mode(hot);
}

fn cmd_refresh(root: &PathBuf, services: &[String], build: bool) -> Result<()> {
    preflight::check_docker()?;
    preflight::ensure_env_files(root)?;

    let available = repo::list_compose_services(root)?;
    let selected_owned: Vec<String> = if services.is_empty() {
        let defaults = repo::default_refresh_services(&available);
        if defaults.is_empty() {
            bail!(
                "No refreshable app services found in docker-compose.yml.\n\
                 Fix: atlas-local services"
            );
        }
        defaults
    } else {
        for s in services {
            if !available.iter().any(|a| a == s) {
                bail!(
                    "Unknown service '{s}'.\n\
                     Available (from docker-compose.yml): {}\n\
                     Fix: atlas-local services",
                    available.join(", ")
                );
            }
        }
        services.to_vec()
    };
    let selected: Vec<&str> = selected_owned.iter().map(String::as_str).collect();

    println!(
        "→ Refreshing {} (build={}) so containers match your working tree…",
        selected.join(", "),
        if build { "yes" } else { "no" }
    );
    compose::refresh(root, &selected, build)?;

    if selected.contains(&"backend") {
        let _ = compose::wait_healthy(root, 180);
    }

    println!();
    print_in_sync_banner(&selected);
    Ok(())
}

fn cmd_services(root: &PathBuf) -> Result<()> {
    let services = repo::list_compose_services(root)?;
    println!("{}", repo::format_services_help(root, &services));
    Ok(())
}

fn cmd_watch(root: &PathBuf) -> Result<()> {
    preflight::check_docker()?;
    preflight::ensure_env_files(root)?;

    let available = repo::list_compose_services(root).unwrap_or_default();
    let apps = repo::default_refresh_services(&available);

    println!("→ Compose Watch started (foreground).");
    println!("  Save a file under a watched path and Compose will rebuild/recreate.");
    println!("  When that finishes, the stack matches your latest saves.");
    println!();
    if !apps.is_empty() {
        println!("  App services in this repo: {}", apps.join(", "));
    }
    println!("  Leave this running; Ctrl-C to stop watching (stack keeps running).");
    println!();
    compose::watch(root)?;
    println!();
    println!("Watch stopped. Stack is still up — use `atlas-local refresh` after further edits.");
    Ok(())
}

fn print_in_sync_banner(services: &[&str]) {
    println!("✓ In sync with your latest saves");
    println!(
        "  Recreated: {}",
        services.join(", ")
    );
    println!("  Refresh the browser if a WASM/SSR app was included.");
}

fn cmd_reset_db(root: &PathBuf, yes: bool) -> Result<()> {
    preflight::check_docker()?;
    if !yes {
        confirm("This will DESTROY the local Postgres volume and all local data. Continue?")?;
    }
    compose::down_volumes(root)?;
    println!("→ Volumes removed. Starting fresh stack…");
    cmd_up(root, false, compose::hot_mode())
}

fn confirm(prompt: &str) -> Result<()> {
    eprint!("{prompt} [y/N] ");
    let _ = io::stderr().flush();
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    let ok = matches!(line.trim().to_lowercase().as_str(), "y" | "yes");
    if !ok {
        bail!("Aborted.");
    }
    Ok(())
}

pub fn confirm_prompt(prompt: &str) -> Result<()> {
    confirm(prompt)
}

fn print_url_map(root: &PathBuf) {
    println!();
    println!("Local Atlas is up. Open:");
    println!("  Admin:     http://admin.localhost");
    println!("  API:       http://api.localhost");
    println!("  Network:   http://directory.network.localhost");
    println!("  Folio:     http://folio.localhost   (alias: http://ruuderie.localhost)");
    println!("  Anchor:    http://buildwithruud.localhost  (also: http://oplystusa.localhost)");
    println!();
    db::LocalDbConn::resolve(root).print_guide();
    println!();
    println!("Passkeys: register a NEW local passkey (RP_ID=localhost).");
    println!("  Server passkeys will not work here — that is intentional.");
    println!();
    println!("New tenants: provision with {{slug}}.anchor.localhost / .network.localhost / .folio.localhost");
    println!("Sandbox pull: atlas-local db pull --from dev --help");
    println!("Docs: docs/architecture/local_development.md");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn cli_help_parses_as_error_display() {
        // clap --help exits via Error::DisplayHelp; ensure the parser is wired.
        let err = Cli::try_parse_from(["atlas-local", "--help"]).unwrap_err();
        assert!(err.to_string().contains("Local Atlas") || err.kind() == clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn cli_parses_up_no_build() {
        let cli = Cli::try_parse_from(["atlas-local", "up", "--no-build"]).unwrap();
        match cli.command {
            Commands::Up { no_build, hot } => {
                assert!(no_build);
                assert!(!hot);
            }
            other => panic!("expected Up, got {other:?}"),
        }
        let cli = Cli::try_parse_from(["atlas-local", "up", "--hot"]).unwrap();
        match cli.command {
            Commands::Up { hot, .. } => assert!(hot),
            other => panic!("expected Up, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_status_plain() {
        let cli = Cli::try_parse_from(["atlas-local", "status", "--plain"]).unwrap();
        match cli.command {
            Commands::Status { plain } => assert!(plain),
            other => panic!("expected Status, got {other:?}"),
        }
    }

    #[test]
    fn cli_parses_db_info() {
        let cli = Cli::try_parse_from(["atlas-local", "db", "info"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Db {
                command: DbCommands::Info
            }
        ));
    }

    #[test]
    fn cli_parses_db_pull_dev_dry_run() {
        let cli = Cli::try_parse_from([
            "atlas-local",
            "db",
            "pull",
            "--from",
            "dev",
            "--dry-run",
        ])
        .unwrap();
        match cli.command {
            Commands::Db {
                command:
                    DbCommands::Pull {
                        from,
                        dry_run,
                        i_understand_pii,
                        yes,
                    },
            } => {
                assert_eq!(from, RemoteEnv::Dev);
                assert!(dry_run);
                assert!(!i_understand_pii);
                assert!(!yes);
            }
            other => panic!("expected Db::Pull, got {other:?}"),
        }
    }

    #[test]
    fn parse_local_database_url_for_dbeaver() {
        let conn = db::parse_postgres_url(
            "postgresql://postgres:postgres@localhost:5433/oplydb",
        )
        .unwrap()
        .for_host_client();
        assert_eq!(conn.host, "127.0.0.1");
        assert_eq!(conn.port, 5433);
        assert_eq!(conn.user, "postgres");
        assert_eq!(conn.password, "postgres");
        assert_eq!(conn.database, "oplydb");
        assert_eq!(
            conn.jdbc_url(),
            "jdbc:postgresql://127.0.0.1:5433/oplydb"
        );
    }

    #[test]
    fn cli_parses_refresh_and_watch() {
        let cli = Cli::try_parse_from(["atlas-local", "refresh", "backend", "--no-build"]).unwrap();
        match cli.command {
            Commands::Refresh { services, no_build } => {
                assert_eq!(services, vec!["backend"]);
                assert!(no_build);
            }
            other => panic!("expected Refresh, got {other:?}"),
        }
        let cli = Cli::try_parse_from(["atlas-local", "watch"]).unwrap();
        assert!(matches!(cli.command, Commands::Watch));
    }

    #[test]
    fn cli_parses_reset_db() {
        let cli = Cli::try_parse_from(["atlas-local", "reset-db", "--yes"]).unwrap();
        match cli.command {
            Commands::ResetDb { yes } => assert!(yes),
            other => panic!("expected ResetDb, got {other:?}"),
        }
    }

    #[test]
    fn remote_env_labels_and_vars() {
        assert_eq!(RemoteEnv::Dev.label(), "atlas_dev");
        assert_eq!(RemoteEnv::Dev.env_var(), "ATLAS_DEV_DATABASE_URL");
        assert_eq!(RemoteEnv::Uat.env_var(), "ATLAS_UAT_DATABASE_URL");
        assert_eq!(RemoteEnv::Prod.env_var(), "ATLAS_PROD_DATABASE_URL");
    }

    #[test]
    fn prod_pull_requires_pii_flag() {
        assert!(prod_pull_allowed(RemoteEnv::Dev, false));
        assert!(prod_pull_allowed(RemoteEnv::Uat, false));
        assert!(!prod_pull_allowed(RemoteEnv::Prod, false));
        assert!(prod_pull_allowed(RemoteEnv::Prod, true));
    }

    #[test]
    fn find_repo_root_from_crate_manifest() {
        let root = repo::find_repo_root().expect("repo root");
        assert!(
            root.join("docker-compose.yml").is_file(),
            "expected docker-compose.yml under {}",
            root.display()
        );
        assert!(
            root.join(".env.local.example").is_file(),
            "expected .env.local.example under {}",
            root.display()
        );
    }

    #[test]
    fn parse_dotenv_line_prefers_local_file() {
        let tmp = tempfile_dir();
        std::fs::write(tmp.join(".env"), "RP_ID=from-env\nWEBAUTHN_ORIGIN=http://a\n").unwrap();
        std::fs::write(
            tmp.join(".env.local"),
            "RP_ID=localhost\nWEBAUTHN_ORIGIN=http://admin.localhost\n",
        )
        .unwrap();
        assert_eq!(
            repo::read_dotenv_value(&tmp, "RP_ID").as_deref(),
            Some("localhost")
        );
        assert_eq!(
            repo::read_dotenv_value(&tmp, "WEBAUTHN_ORIGIN").as_deref(),
            Some("http://admin.localhost")
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn webauthn_orb_local_detected() {
        assert!(preflight::looks_like_orb_local_webauthn(
            "https://platform-admin.atlas-platform.orb.local",
            "atlas-platform.orb.local"
        ));
        assert!(!preflight::looks_like_orb_local_webauthn(
            "http://admin.localhost",
            "localhost"
        ));
    }

    #[test]
    fn parse_compose_services_from_sample() {
        let yaml = r#"
services:
  postgres:
    image: postgres:15
  backend:
    build: ./backend
  platform-admin:
    build: ./apps/platform-admin

networks:
  app-network:
"#;
        let names = repo::parse_compose_service_names(yaml).unwrap();
        assert_eq!(
            names,
            vec!["postgres", "backend", "platform-admin"]
        );
        assert_eq!(
            repo::default_refresh_services(&names),
            vec!["backend", "platform-admin"]
        );
    }

    #[test]
    fn list_compose_services_from_repo() {
        let root = repo::find_repo_root().unwrap();
        let names = repo::list_compose_services(&root).unwrap();
        assert!(names.contains(&"backend".into()));
        assert!(names.contains(&"platform-admin".into()));
        assert!(names.contains(&"folio".into()));
        assert!(names.contains(&"anchor".into()));
        let help = repo::format_services_help(&root, &names);
        assert!(help.contains("docker-compose.yml"));
        assert!(help.contains("backend"));
    }

    #[test]
    fn cli_parses_services_command() {
        let cli = Cli::try_parse_from(["atlas-local", "services"]).unwrap();
        assert!(matches!(cli.command, Commands::Services));
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("atlas-local-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
