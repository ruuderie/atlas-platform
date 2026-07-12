//! State → concrete `atlas-local …` next steps (problem → fix).

use super::StatusSnapshot;
use crate::compose;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StackHealth {
    Ready,
    Warming,
    Unhealthy,
    Down,
}

#[derive(Debug, Clone)]
pub struct Guidance {
    pub health: StackHealth,
    pub headline: String,
    /// Copy-paste CLI lines (and short notes).
    pub commands: Vec<String>,
}

impl Guidance {
    /// First `atlas-local -- refresh [services…]` from Next steps (skips comments).
    /// Empty slice means bare `refresh` (all app services Compose knows).
    /// `None` means no refresh line (e.g. stack down → use `up`).
    pub fn suggested_refresh_services(&self) -> Option<Vec<String>> {
        for cmd in &self.commands {
            let trimmed = cmd.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("cargo run -p atlas-local -- refresh") {
                let services: Vec<String> = rest
                    .split_whitespace()
                    .map(str::to_string)
                    .collect();
                return Some(services);
            }
        }
        None
    }

    /// One-line UI/help hint for the TUI `x` key (what it will run).
    pub fn x_refresh_hint(&self) -> String {
        match self.suggested_refresh_services() {
            Some(services) if services.is_empty() => {
                "x → refresh all app services  (r only reloads this panel)".into()
            }
            Some(services) => {
                format!(
                    "x → refresh {}  (r only reloads this panel)",
                    services.join(" ")
                )
            }
            None => "x unavailable (no refresh in Next steps) — use up / see commands below".into(),
        }
    }
}

impl StatusSnapshot {
    pub fn stack_health(&self) -> StackHealth {
        let (ok, total) = self.healthy_probe_count();
        let (n, _running, unhealthy) = self.container_summary();
        let db_ok = self.db.state.is_ok();
        let backend_ok = self
            .probes
            .iter()
            .any(|p| p.label.starts_with("backend") && p.ok);

        if n == 0 {
            StackHealth::Down
        } else if db_ok && unhealthy == 0 && ok == total && n > 0 {
            StackHealth::Ready
        } else if backend_ok || (ok > 0 && db_ok) {
            StackHealth::Warming
        } else {
            StackHealth::Unhealthy
        }
    }

    pub fn guidance(&self) -> Guidance {
        let health = self.stack_health();
        let hot = compose::hot_mode();
        let backend_ok = self
            .probes
            .iter()
            .any(|p| p.label.starts_with("backend") && p.ok);
        let schema_ready = matches!(&self.db.state, Ok(s) if s.tenants.is_some());
        let db_up = self.db.state.is_ok();
        let (n, _running, unhealthy) = self.container_summary();
        let failed_services: Vec<String> = self
            .probes
            .iter()
            .filter(|p| !p.ok)
            .filter_map(|p| service_from_probe_label(&p.label))
            .collect();

        let mut commands = Vec::new();
        let headline;

        match health {
            StackHealth::Down => {
                headline = "Stack is down — start it (parity mode matches the server).".into();
                commands.push("cargo run -p atlas-local -- up".into());
                commands.push("# then: cargo run -p atlas-local -- status".into());
            }
            StackHealth::Ready => {
                headline = "Stack looks healthy. After code changes, sync containers.".into();
                if hot {
                    commands.push("cargo run -p atlas-local -- refresh".into());
                    commands.push("# or keep watching: cargo run -p atlas-local -- watch".into());
                } else {
                    commands.push("cargo run -p atlas-local -- refresh backend".into());
                    commands.push("# full app refresh: cargo run -p atlas-local -- refresh".into());
                }
                commands.push("cargo run -p atlas-local -- db info".into());
            }
            StackHealth::Warming | StackHealth::Unhealthy => {
                if !backend_ok {
                    if hot {
                        headline = "Backend not serving /health yet — often still compiling in --hot mode (not a server bug).".into();
                        commands.push("# Prefer server-parity (baked binary):".into());
                        commands.push("cargo run -p atlas-local -- down".into());
                        commands.push("cargo run -p atlas-local -- up".into());
                        commands.push("# Or wait / watch compile:".into());
                        commands.push("cargo run -p atlas-local -- logs -f backend".into());
                    } else {
                        headline = "Backend /health failed — restart the parity backend, then re-check.".into();
                        commands.push("cargo run -p atlas-local -- logs -f backend".into());
                        commands.push("cargo run -p atlas-local -- refresh backend".into());
                        commands.push("cargo run -p atlas-local -- status".into());
                        commands.push("# If still broken after a fix in this branch:".into());
                        commands.push("cargo run -p atlas-local -- down && cargo run -p atlas-local -- up".into());
                    }
                } else if !schema_ready {
                    headline = "Backend is up but schema/migrations not ready yet — wait, then re-check.".into();
                    commands.push("cargo run -p atlas-local -- logs -f backend".into());
                    commands.push("# wait until you see \"Migrations completed\", then:".into());
                    commands.push("cargo run -p atlas-local -- status".into());
                    commands.push("# if schema never appears:".into());
                    commands.push("cargo run -p atlas-local -- refresh backend".into());
                    commands.push("cargo run -p atlas-local -- reset-db".into());
                } else if !db_up {
                    headline = "Postgres unreachable — bring DB back, or wipe the local volume.".into();
                    commands.push("cargo run -p atlas-local -- logs -f postgres".into());
                    commands.push("cargo run -p atlas-local -- up".into());
                    commands.push("# corrupt / empty local DB:".into());
                    commands.push("cargo run -p atlas-local -- reset-db".into());
                } else if unhealthy > 0 || !failed_services.is_empty() {
                    headline = "Some services are unhealthy — refresh them, then re-check status.".into();
                    let targets = if failed_services.is_empty() {
                        "backend platform-admin".into()
                    } else {
                        failed_services.join(" ")
                    };
                    commands.push(format!("cargo run -p atlas-local -- logs -f {targets}"));
                    commands.push(format!("cargo run -p atlas-local -- refresh {targets}"));
                    commands.push("cargo run -p atlas-local -- status".into());
                    commands.push("# still stuck after your fix:".into());
                    commands.push("cargo run -p atlas-local -- down && cargo run -p atlas-local -- up".into());
                } else {
                    headline = "Stack is warming — re-check in a few seconds.".into();
                    commands.push("cargo run -p atlas-local -- status".into());
                    commands.push("cargo run -p atlas-local -- logs -f backend".into());
                }

                if n > 0 && !self.telemetry.metrics_ok
                    && self.telemetry.metrics_note.to_lowercase().contains("unauthorized")
                {
                    commands.push("# metrics scrape: add METRICS_TOKEN=local-dev-metrics to .env.local".into());
                    commands.push("cargo run -p atlas-local -- refresh backend".into());
                }
            }
        }

        // Always offer a recovery ladder footer when not ready.
        if !matches!(health, StackHealth::Ready) {
            commands.push("# Recovery ladder (strongest last):".into());
            commands.push("#   refresh → down&&up → reset-db → up".into());
        }

        Guidance {
            health,
            headline,
            commands,
        }
    }
}

fn service_from_probe_label(label: &str) -> Option<String> {
    let l = label.to_lowercase();
    if l.starts_with("backend") {
        Some("backend".into())
    } else if l.starts_with("platform-admin") {
        Some("platform-admin".into())
    } else if l.starts_with("network") {
        Some("network-instance".into())
    } else if l.starts_with("folio") {
        Some("folio".into())
    } else if l.starts_with("anchor") {
        Some("anchor".into())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::{DbLiveState, DbPanel, EnvPanel, Probe};
    use crate::status::resources::{ResourceSnapshot, TelemetrySnapshot};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn empty_snap() -> StatusSnapshot {
        StatusSnapshot {
            root: PathBuf::from("/tmp"),
            collected_at: SystemTime::now(),
            env_ok: true,
            env_local_ok: true,
            environment: "development".into(),
            rp_id: "localhost".into(),
            webauthn_origin: "http://admin.localhost".into(),
            docker: "Engine".into(),
            compose: "1".into(),
            containers: Ok(vec![]),
            domains: vec![],
            probes: vec![],
            db: DbPanel {
                host: "127.0.0.1".into(),
                port: 5433,
                database: "oplydb".into(),
                user: "postgres".into(),
                password: "postgres".into(),
                url: String::new(),
                jdbc: String::new(),
                state: Err("down".into()),
            },
            resources: ResourceSnapshot::default(),
            telemetry: TelemetrySnapshot::default(),
            env_panel: EnvPanel::default(),
        }
    }

    #[test]
    fn down_stack_suggests_up() {
        let g = empty_snap().guidance();
        assert_eq!(g.health, StackHealth::Down);
        assert!(g.commands.iter().any(|c| c.contains("atlas-local -- up")));
    }

    #[test]
    fn unhealthy_backend_suggests_refresh() {
        let mut snap = empty_snap();
        snap.containers = Ok(vec![crate::compose::ContainerRow {
            service: "backend".into(),
            state: "running".into(),
            status: "Up (unhealthy)".into(),
            ports: String::new(),
        }]);
        snap.probes = vec![Probe {
            label: "backend /health".into(),
            url: "http://127.0.0.1:8000/health".into(),
            ok: false,
            latency_ms: Some(1),
        }];
        snap.db.state = Ok(DbLiveState {
            ready_line: "accepting".into(),
            version: None,
            size: None,
            sessions: None,
            tenants: None,
            app_domains: None,
            sample_domains: vec![],
            note: None,
        });
        let g = snap.guidance();
        assert!(matches!(
            g.health,
            StackHealth::Unhealthy | StackHealth::Warming
        ));
        assert!(g.commands.iter().any(|c| c.contains("refresh") || c.contains("logs")));
    }

    #[test]
    fn suggested_refresh_parses_targets() {
        let g = Guidance {
            health: StackHealth::Unhealthy,
            headline: "x".into(),
            commands: vec![
                "# note".into(),
                "cargo run -p atlas-local -- logs -f network-instance anchor".into(),
                "cargo run -p atlas-local -- refresh network-instance anchor".into(),
            ],
        };
        assert_eq!(
            g.suggested_refresh_services(),
            Some(vec!["network-instance".into(), "anchor".into()])
        );
        let bare = Guidance {
            health: StackHealth::Ready,
            headline: "x".into(),
            commands: vec!["cargo run -p atlas-local -- refresh".into()],
        };
        assert_eq!(bare.suggested_refresh_services(), Some(vec![]));
        let down = empty_snap().guidance();
        assert_eq!(down.suggested_refresh_services(), None);
        assert!(down.x_refresh_hint().contains("unavailable"));
        assert!(g.x_refresh_hint().contains("network-instance"));
        assert!(g.x_refresh_hint().contains("x →"));
    }
}
