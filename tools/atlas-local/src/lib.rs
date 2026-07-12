//! Library surface for `atlas-local` — keeps the binary thin and unit-testable.
//!
//! Extension policy: new local automation belongs here (subcommands / modules),
//! not one-off shell scripts. See `docs/architecture/local_development.md`.

pub mod compose;
pub mod db;
pub mod preflight;
pub mod repo;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand, ValueEnum};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "atlas-local",
    version,
    about = "Local Atlas Platform development tooling",
    long_about = "Starts and manages the Docker Compose + Caddy local stack, and optional \
remote DB sandbox pulls. WebAuthn for local is isolated (RP_ID=localhost) and must never \
be copied into K8s overlays.\n\n\
Extension policy: add new local automation as subcommands here — see \
docs/architecture/local_development.md."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the full local stack (Compose + Caddy)
    Up {
        /// Skip Docker image rebuild
        #[arg(long)]
        no_build: bool,
    },
    /// Stop the local stack
    Down,
    /// Show service health without starting
    Status,
    /// Follow or dump Compose logs
    Logs {
        /// Follow log output
        #[arg(short = 'f', long)]
        follow: bool,
        /// Optional service name (postgres, backend, platform-admin, …)
        service: Option<String>,
    },
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
}

#[derive(Subcommand, Debug)]
pub enum DbCommands {
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
    let cli = Cli::parse();
    dispatch(cli)
}

pub fn dispatch(cli: Cli) -> Result<()> {
    let root = repo::find_repo_root().context(
        "Could not find atlas-platform root (looking for docker-compose.yml).\n\
         Fix: cd into atlas-platform/ (or a subdirectory) and re-run.",
    )?;

    match cli.command {
        Commands::Up { no_build } => cmd_up(&root, no_build),
        Commands::Down => {
            preflight::check_docker()?;
            compose::down(&root)
        }
        Commands::Status => {
            preflight::check_docker()?;
            compose::status(&root)
        }
        Commands::Logs { follow, service } => {
            preflight::check_docker()?;
            compose::logs(&root, follow, service.as_deref())
        }
        Commands::ResetDb { yes } => cmd_reset_db(&root, yes),
        Commands::Db {
            command: DbCommands::Pull {
                from,
                dry_run,
                i_understand_pii,
                yes,
            },
        } => db::pull(&root, from, dry_run, i_understand_pii, yes),
    }
}

fn cmd_up(root: &PathBuf, no_build: bool) -> Result<()> {
    preflight::check_docker()?;
    preflight::ensure_env_files(root)?;
    preflight::warn_webauthn_orb_local(root);
    preflight::check_ports(&[80, 8000, 8080, 8081, 3100, 3000])?;

    println!("→ Starting local Atlas stack…");
    compose::up(root, !no_build)?;

    println!("→ Waiting for health (first Rust image builds can take several minutes)…");
    if let Err(e) = compose::wait_healthy(root, 600) {
        let _ = compose::dump_logs_tail(root, "postgres", 40);
        let _ = compose::dump_logs_tail(root, "backend", 40);
        bail!(
            "{e}\n\
             Fix: atlas-local logs -f\n\
             If the database volume is corrupt: atlas-local reset-db\n\
             Check service health: atlas-local status"
        );
    }

    print_url_map();
    Ok(())
}

fn cmd_reset_db(root: &PathBuf, yes: bool) -> Result<()> {
    preflight::check_docker()?;
    if !yes {
        confirm("This will DESTROY the local Postgres volume and all local data. Continue?")?;
    }
    compose::down_volumes(root)?;
    println!("→ Volumes removed. Starting fresh stack…");
    cmd_up(root, false)
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

fn print_url_map() {
    println!();
    println!("Local Atlas is up. Open:");
    println!("  Admin:     http://admin.localhost");
    println!("  API:       http://api.localhost");
    println!("  Network:   http://directory.network.localhost");
    println!("  Folio:     http://folio.localhost   (alias: http://ruuderie.localhost)");
    println!("  Anchor:    http://buildwithruud.localhost  (also: http://oplystusa.localhost)");
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
            Commands::Up { no_build } => assert!(no_build),
            other => panic!("expected Up, got {other:?}"),
        }
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

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("atlas-local-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
