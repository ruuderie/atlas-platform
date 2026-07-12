//! Ratatui status dashboard with Overview / Capacity / Telemetry / Env tabs.
//!
//! Design: purpose-first hierarchy, instant key response, no decorative motion.
//! Telemetry "streams" via a 3s poll ring-buffer (no SSE exists in the platform).
//! Env tab writes `.env.local`; **a** recreates backend so Compose injects those values.
//! Capacity KPIs (tenants / domains / DB / sessions) match Platform Admin System Status.

use super::{
    StatusSnapshot, TelemetryHistory, format_bytes, sparkline, sparkline_u64,
};
use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap};
use std::io::{self, Stdout, Write};
use std::path::Path;
use std::time::{Duration, Instant};

const FG: Color = Color::Rgb(235, 235, 240);
const MUTED: Color = Color::Rgb(140, 140, 150);
const DIM: Color = Color::Rgb(90, 90, 100);
const ACCENT: Color = Color::Rgb(10, 132, 255);
const OK: Color = Color::Rgb(50, 215, 75);
const WARN: Color = Color::Rgb(255, 159, 10);
const ERR: Color = Color::Rgb(255, 69, 58);
const BORDER: Color = Color::Rgb(58, 58, 68);
const TITLE: Color = Color::Rgb(255, 255, 255);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview = 0,
    Resources = 1,
    Telemetry = 2,
    Env = 3,
}

impl Tab {
    fn next(self) -> Self {
        match self {
            Self::Overview => Self::Resources,
            Self::Resources => Self::Telemetry,
            Self::Telemetry => Self::Env,
            Self::Env => Self::Overview,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Overview => Self::Env,
            Self::Resources => Self::Overview,
            Self::Telemetry => Self::Resources,
            Self::Env => Self::Telemetry,
        }
    }
}

const SMTP_FIELD_KEYS: &[&str] = &[
    "SMTP_SERVER",
    "SMTP_PORT",
    "SMTP_USERNAME",
    "SMTP_TOKEN",
    "SMTP_FROM",
];

#[derive(Clone)]
enum Overlay {
    None,
    SmtpForm {
        fields: [String; 5],
        focus: usize,
        notice: Option<String>,
    },
    Banner(String),
}

pub fn run(root: &Path, initial: StatusSnapshot) -> Result<()> {
    let _guard = TerminalGuard;
    let mut terminal = setup()?;
    event_loop(&mut terminal, root, initial)
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore();
    }
}

fn setup() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alternate screen")?;
    Terminal::new(CrosstermBackend::new(stdout)).context("create terminal")
}

fn restore() -> Result<()> {
    disable_raw_mode().ok();
    execute!(io::stdout(), LeaveAlternateScreen).ok();
    Ok(())
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    root: &Path,
    mut snap: StatusSnapshot,
) -> Result<()> {
    let mut last_refresh = Instant::now();
    let refresh_every = Duration::from_secs(3);
    let mut refreshing = false;
    let mut tab = Tab::Overview;
    let mut overlay = Overlay::None;
    let mut history = TelemetryHistory::default();
    ingest_history(&mut history, &snap);

    loop {
        terminal.draw(|frame| draw(frame, &snap, &history, tab, refreshing, &overlay))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match &mut overlay {
                    Overlay::SmtpForm {
                        fields,
                        focus,
                        notice,
                    } => match key.code {
                        KeyCode::Esc => overlay = Overlay::None,
                        KeyCode::Tab | KeyCode::Down => {
                            *focus = (*focus + 1) % fields.len();
                            *notice = None;
                        }
                        KeyCode::BackTab | KeyCode::Up => {
                            *focus = (*focus + fields.len() - 1) % fields.len();
                            *notice = None;
                        }
                        KeyCode::Enter => {
                            let _ = crate::preflight::ensure_env_files(root);
                            let updates: Vec<(String, String)> = SMTP_FIELD_KEYS
                                .iter()
                                .zip(fields.iter())
                                .map(|(k, v)| ((*k).to_string(), v.clone()))
                                .collect();
                            match crate::env::upsert_dotenv_file(
                                &crate::repo::env_local_path(root),
                                &updates,
                            ) {
                                Ok(()) => {
                                    snap = StatusSnapshot::collect(root);
                                    ingest_history(&mut history, &snap);
                                    overlay = Overlay::Banner(
                                        "Saved to .env.local — press a to apply (recreate backend)"
                                            .into(),
                                    );
                                }
                                Err(e) => *notice = Some(format!("Save failed: {e}")),
                            }
                        }
                        KeyCode::Backspace => {
                            fields[*focus].pop();
                            *notice = None;
                        }
                        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                            fields[*focus].push(c);
                            *notice = None;
                        }
                        _ => {}
                    },
                    Overlay::Banner(_) => match key.code {
                        KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                            overlay = Overlay::None;
                        }
                        KeyCode::Char('a') if tab == Tab::Env => {
                            match apply_backend_env(terminal, root) {
                                Ok(msg) => {
                                    snap = StatusSnapshot::collect(root);
                                    ingest_history(&mut history, &snap);
                                    last_refresh = Instant::now();
                                    overlay = Overlay::Banner(msg);
                                }
                                Err(e) => {
                                    overlay = Overlay::Banner(format!("Apply failed: {e:#}"));
                                }
                            }
                        }
                        KeyCode::Char('q') => break,
                        _ => {}
                    },
                    Overlay::None => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('r') => {
                            refreshing = true;
                            terminal.draw(|frame| {
                                draw(frame, &snap, &history, tab, refreshing, &overlay)
                            })?;
                            snap = StatusSnapshot::collect(root);
                            ingest_history(&mut history, &snap);
                            last_refresh = Instant::now();
                            refreshing = false;
                        }
                        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => tab = tab.next(),
                        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => tab = tab.prev(),
                        KeyCode::Char('1') => tab = Tab::Overview,
                        KeyCode::Char('2') => tab = Tab::Resources,
                        KeyCode::Char('3') => tab = Tab::Telemetry,
                        KeyCode::Char('4') => tab = Tab::Env,
                        KeyCode::Char('s') if tab == Tab::Env => {
                            overlay = smtp_form_from_snap(&snap);
                        }
                        KeyCode::Char('a') if tab == Tab::Env => {
                            match apply_backend_env(terminal, root) {
                                Ok(msg) => {
                                    snap = StatusSnapshot::collect(root);
                                    ingest_history(&mut history, &snap);
                                    last_refresh = Instant::now();
                                    overlay = Overlay::Banner(msg);
                                }
                                Err(e) => {
                                    overlay = Overlay::Banner(format!("Apply failed: {e:#}"));
                                }
                            }
                        }
                        KeyCode::Char('e') if tab == Tab::Env => {
                            match open_env_editor(terminal, root) {
                                Ok(msg) => {
                                    snap = StatusSnapshot::collect(root);
                                    ingest_history(&mut history, &snap);
                                    last_refresh = Instant::now();
                                    overlay = Overlay::Banner(msg);
                                }
                                Err(e) => {
                                    overlay = Overlay::Banner(format!("Edit failed: {e:#}"));
                                }
                            }
                        }
                        KeyCode::Char('x') => match run_suggested_refresh(terminal, root, &snap) {
                            Ok(msg) => {
                                snap = StatusSnapshot::collect(root);
                                ingest_history(&mut history, &snap);
                                last_refresh = Instant::now();
                                overlay = Overlay::Banner(msg);
                            }
                            Err(e) => {
                                overlay = Overlay::Banner(format!("{e:#}"));
                            }
                        },
                        _ => {}
                    },
                }
            }
        }

        // Pause auto-refresh while editing SMTP so typed values aren't stomped.
        if matches!(overlay, Overlay::None)
            && last_refresh.elapsed() >= refresh_every
        {
            snap = StatusSnapshot::collect(root);
            ingest_history(&mut history, &snap);
            last_refresh = Instant::now();
        }
    }
    Ok(())
}

fn smtp_form_from_snap(snap: &StatusSnapshot) -> Overlay {
    let mut fields = [
        String::new(),
        "587".into(),
        String::new(),
        String::new(),
        String::new(),
    ];
    for (i, key) in SMTP_FIELD_KEYS.iter().enumerate() {
        if let Some(raw) = crate::repo::read_dotenv_value(&snap.root, key) {
            fields[i] = raw;
        }
    }
    Overlay::SmtpForm {
        fields,
        focus: 0,
        notice: None,
    }
}

/// Leave the TUI, recreate backend so Compose injects `.env.local`, then resume.
fn apply_backend_env(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    root: &Path,
) -> Result<String> {
    restore()?;
    println!();
    println!("→ Recreating backend so SMTP / .env.local values take effect…");
    let _ = io::stdout().flush();
    let result = crate::compose::refresh(root, &["backend"], false);
    println!();
    println!("Press Enter to return to status…");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    *terminal = setup()?;
    result?;
    Ok("Backend recreated — running process now has .env.local SMTP_* values".into())
}

/// Run the first Next-steps `refresh …` (failed/unhealthy apps, or default sync).
fn run_suggested_refresh(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    root: &Path,
    snap: &StatusSnapshot,
) -> Result<String> {
    let guide = snap.guidance();
    let Some(services) = guide.suggested_refresh_services() else {
        anyhow::bail!(
            "No refresh in Next steps (stack may be down). Start with: atlas-local -- up"
        );
    };
    let label = if services.is_empty() {
        "all services".into()
    } else {
        services.join(" ")
    };
    let service_refs: Vec<&str> = services.iter().map(String::as_str).collect();

    restore()?;
    println!();
    println!("→ Running suggested refresh: {label}");
    println!("  (same as Next steps — only listed services, not the whole stack)");
    let _ = io::stdout().flush();
    let result = crate::compose::refresh(root, &service_refs, false);
    println!();
    println!("Press Enter to return to status…");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    *terminal = setup()?;
    result?;
    Ok(format!("Refreshed {label}"))
}

fn open_env_editor(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    root: &Path,
) -> Result<String> {
    restore()?;
    crate::env::cmd_edit(root)?;
    println!();
    println!("Press Enter to return to status…");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    *terminal = setup()?;
    Ok("Editor closed — press a to apply changes to backend".into())
}

fn ingest_history(history: &mut TelemetryHistory, snap: &StatusSnapshot) {
    history.push_sample(
        snap.backend_latency_ms(),
        snap.resources.totals.cpu_pct,
        snap.resources.totals.mem_used_mib,
        &snap.telemetry.feed_lines,
    );
}

fn draw(
    frame: &mut Frame,
    snap: &StatusSnapshot,
    history: &TelemetryHistory,
    tab: Tab,
    refreshing: bool,
    overlay: &Overlay,
) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // header (wraps)
            Constraint::Length(3), // tabs
            Constraint::Min(10),   // body
            Constraint::Length(2), // footer (wraps)
        ])
        .split(area);

    draw_header(frame, root[0], snap, history, refreshing);
    draw_tabs(frame, root[1], tab);

    match tab {
        Tab::Overview => draw_overview(frame, root[2], snap),
        Tab::Resources => draw_resources(frame, root[2], snap),
        Tab::Telemetry => draw_telemetry(frame, root[2], snap, history),
        Tab::Env => draw_env(frame, root[2], snap),
    }

    draw_footer(frame, root[3], tab, overlay);
    draw_overlay(frame, area, overlay);
}

fn panel(title: impl Into<String>) -> Block<'static> {
    let title = title.into();
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .border_set(symbols::border::ROUNDED)
        .title(Span::styled(
            format!(" {title} "),
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD),
        ))
}

fn draw_header(
    frame: &mut Frame,
    area: Rect,
    snap: &StatusSnapshot,
    history: &TelemetryHistory,
    refreshing: bool,
) {
    let (ok, total) = snap.healthy_probe_count();
    let (n, running, unhealthy) = snap.container_summary();
    let db_ok = snap.db.state.is_ok();
    let overall = if db_ok && unhealthy == 0 && ok == total && n > 0 {
        ("Ready", OK)
    } else if n == 0 {
        ("Down", ERR)
    } else if ok > 0 || db_ok {
        ("Warming", WARN)
    } else {
        ("Unhealthy", ERR)
    };

    let lat = snap
        .backend_latency_ms()
        .map(|ms| format!("{ms}ms"))
        .unwrap_or_else(|| "—".into());
    let spark_w = (area.width as usize).saturating_sub(40).clamp(8, 24);
    let spark = sparkline_u64(
        &history.backend_latency_ms.iter().copied().collect::<Vec<_>>(),
        spark_w,
    );

    let refresh = if refreshing {
        Span::styled(" · refreshing…", Style::default().fg(ACCENT))
    } else {
        Span::raw("")
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(
                " Atlas ",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled("local", Style::default().fg(FG).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled(
                format!(" ● {} ", overall.0),
                Style::default().fg(overall.1).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" · {}", snap.environment),
                Style::default().fg(MUTED),
            ),
            refresh,
        ]),
        Line::from(vec![Span::styled(
            format!(
                "{running}/{n} up · probes {ok}/{total} · CPU {:.0}% · RAM {:.0}MiB · /health {lat} {spark}",
                snap.resources.totals.cpu_pct,
                snap.resources.totals.mem_used_mib
            ),
            Style::default().fg(MUTED),
        )]),
    ];

    let block = panel("Overview");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_tabs(frame: &mut Frame, area: Rect, tab: Tab) {
    let titles = ["1 Overview", "2 Capacity", "3 Telemetry", "4 Env"]
        .iter()
        .map(|t| {
            Line::from(Span::styled(
                format!(" {t} "),
                Style::default().fg(MUTED),
            ))
        })
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .border_set(symbols::border::ROUNDED),
        )
        .select(tab as usize)
        .highlight_style(
            Style::default()
                .fg(TITLE)
                .bg(Color::Rgb(28, 28, 35))
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled("│", Style::default().fg(DIM)));

    frame.render_widget(tabs, area);
}

fn draw_overview(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    // Next steps get a full-width band so long CLI commands can wrap.
    let bands = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(12)])
        .split(area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(bands[0]);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(3),
            Constraint::Length(6),
        ])
        .split(cols[0]);

    draw_system(frame, left[0], snap);
    draw_domains(frame, left[1], snap);
    draw_database(frame, left[2], snap);
    draw_probes(frame, cols[1], snap);
    draw_next_steps(frame, bands[1], snap);
}

fn draw_next_steps(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let guide = snap.guidance();
    let border = match guide.health {
        crate::status::StackHealth::Ready => OK,
        crate::status::StackHealth::Warming => WARN,
        crate::status::StackHealth::Unhealthy | crate::status::StackHealth::Down => ERR,
    };
    let x_hint = guide.x_refresh_hint();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border))
        .border_set(symbols::border::ROUNDED)
        .title(Span::styled(
            " Next steps ",
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD),
        ))
        .title_bottom(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("x", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(
                " = run first refresh below (affected apps only) · ",
                Style::default().fg(MUTED),
            ),
            Span::styled("r", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" = reload panel ", Style::default().fg(MUTED)),
        ]));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            guide.headline.clone(),
            Style::default().fg(FG),
        )),
        Line::from(Span::styled(
            x_hint,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
    ];
    for cmd in &guide.commands {
        let style = if cmd.starts_with('#') {
            Style::default().fg(DIM)
        } else {
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
        };
        lines.push(Line::from(Span::styled(cmd.clone(), style)));
    }
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }),
        inner,
    );
}

fn draw_system(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let env_mark = |ok: bool| {
        if ok {
            Span::styled("✓", Style::default().fg(OK))
        } else {
            Span::styled("✗", Style::default().fg(ERR))
        }
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("root  ", Style::default().fg(MUTED)),
            Span::styled(
                snap.root.display().to_string(),
                Style::default().fg(FG),
            ),
        ]),
        Line::from(vec![
            Span::styled("env   ", Style::default().fg(MUTED)),
            env_mark(snap.env_ok),
            Span::raw(" .env  "),
            env_mark(snap.env_local_ok),
            Span::raw(" .env.local  "),
            Span::styled(snap.environment.clone(), Style::default().fg(ACCENT)),
        ]),
        Line::from(vec![
            Span::styled("auth  ", Style::default().fg(MUTED)),
            Span::styled(
                format!("RP_ID={}  ORIGIN={}", snap.rp_id, snap.webauthn_origin),
                Style::default().fg(FG),
            ),
        ]),
    ];
    let block = panel("System");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_domains(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let block = panel("Domains");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let lines: Vec<Line> = snap
        .domains
        .iter()
        .map(|d| {
            Line::from(vec![
                Span::styled(format!("{:<9}", d.kind), Style::default().fg(ACCENT)),
                Span::styled(d.url.clone(), Style::default().fg(FG)),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_probes(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let (ok, total) = snap.healthy_probe_count();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .border_set(symbols::border::ROUNDED)
        .title(Span::styled(
            format!(" HTTP latency  {ok}/{total} "),
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD),
        ));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines: Vec<Line> = snap
        .probes
        .iter()
        .map(|p| {
            let (mark, color) = if p.ok { ("●", OK) } else { ("○", ERR) };
            let lat = p
                .latency_ms
                .map(|ms| format!("{ms:>5}ms"))
                .unwrap_or_else(|| "   —  ".into());
            let lat_color = match p.latency_ms {
                Some(ms) if ms < 100 => OK,
                Some(ms) if ms < 500 => WARN,
                Some(_) => ERR,
                None => DIM,
            };
            Line::from(vec![
                Span::styled(format!("{mark} "), Style::default().fg(color)),
                Span::styled(format!("{lat}  "), Style::default().fg(lat_color)),
                Span::styled(
                    format!("{:<16}", p.label),
                    Style::default().fg(if p.ok { FG } else { MUTED }),
                ),
            ])
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_database(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let db_ok = snap.db.state.is_ok();
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(BORDER))
        .border_set(symbols::border::ROUNDED)
        .title(Span::styled(
            " Database ",
            Style::default().fg(TITLE).add_modifier(Modifier::BOLD),
        ))
        .title(
            Line::from(Span::styled(
                if db_ok {
                    " ● connected "
                } else {
                    " ○ unreachable "
                },
                Style::default().fg(if db_ok { OK } else { ERR }),
            ))
            .right_aligned(),
        );
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines = vec![Line::from(vec![
        Span::styled("DBeaver  ", Style::default().fg(MUTED)),
        Span::styled(
            format!(
                "{}:{}/{}  user={}  ssl=off",
                snap.db.host, snap.db.port, snap.db.database, snap.db.user
            ),
            Style::default().fg(FG),
        ),
    ])];
    match &snap.db.state {
        Ok(s) => {
            lines.push(Line::from(Span::styled(
                format!(
                    "size {} · sessions {} · tenants {}",
                    s.size.as_deref().unwrap_or("—"),
                    s.sessions.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                    s.tenants.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                ),
                Style::default().fg(MUTED),
            )));
            if let Some(ref note) = s.note {
                lines.push(Line::from(Span::styled(
                    note.clone(),
                    Style::default().fg(WARN),
                )));
            }
        }
        Err(e) => lines.push(Line::from(Span::styled(e.clone(), Style::default().fg(ERR)))),
    }
    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
}

fn draw_resources(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let rows_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(7),
        ])
        .split(area);

    // Application capacity — same KPI vocabulary as Platform Admin System Status
    let (tenants, domains, sessions, size) = match &snap.db.state {
        Ok(state) => (
            state
                .tenants
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            state
                .app_domains
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            state
                .sessions
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".into()),
            state.size.clone().unwrap_or_else(|| "—".into()),
        ),
        Err(_) => ("—".into(), "—".into(), "—".into(), "—".into()),
    };
    let app_cap = Line::from(vec![
        Span::styled("Application  ", Style::default().fg(MUTED)),
        Span::styled(format!("tenants {tenants}"), Style::default().fg(FG)),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled(format!("domains {domains}"), Style::default().fg(FG)),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled(format!("DB {size}"), Style::default().fg(FG)),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled(format!("sessions {sessions}"), Style::default().fg(FG)),
    ]);
    let block = panel("Application capacity");
    let inner = block.inner(rows_layout[0]);
    frame.render_widget(block, rows_layout[0]);
    frame.render_widget(
        Paragraph::new(vec![
            app_cap,
            Line::from(Span::styled(
                format!(
                    "env={} · matches System Status Capacity KPIs (host CPU stays below)",
                    snap.environment
                ),
                Style::default().fg(DIM),
            )),
        ])
        .wrap(Wrap { trim: true }),
        inner,
    );

    // Host / Docker stack load
    let totals = &snap.resources.totals;
    let summary = Line::from(vec![
        Span::styled("Stack load  ", Style::default().fg(MUTED)),
        Span::styled(
            format!("CPU {:.1}%", totals.cpu_pct),
            Style::default().fg(if totals.cpu_pct > 80.0 { WARN } else { FG }),
        ),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled(
            format!("RAM {:.0} MiB", totals.mem_used_mib),
            Style::default().fg(FG),
        ),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled(
            format!("images {}", format_bytes(totals.images_bytes)),
            Style::default().fg(FG),
        ),
        Span::styled("  ·  ", Style::default().fg(DIM)),
        Span::styled(
            "dev volumes inflate disk (target cache)",
            Style::default().fg(DIM),
        ),
    ]);
    let block = panel("Stack load");
    let inner = block.inner(rows_layout[1]);
    frame.render_widget(block, rows_layout[1]);
    frame.render_widget(
        Paragraph::new(summary).wrap(Wrap { trim: true }),
        inner,
    );

    // CPU/RAM table
    let block = panel("Containers · CPU / RAM / I/O");
    let inner = block.inner(rows_layout[2]);
    frame.render_widget(block, rows_layout[2]);

    let header = Row::new(vec!["SERVICE", "CPU", "MEM", "MEM%", "NET I/O", "BLOCK I/O"])
        .style(Style::default().fg(MUTED).add_modifier(Modifier::BOLD));
    let table_rows = snap.resources.stats.iter().map(|s| {
        let cpu_c = if s.cpu_pct > 80.0 {
            WARN
        } else if s.cpu_pct > 20.0 {
            ACCENT
        } else {
            FG
        };
        Row::new(vec![
            Cell::from(s.service.clone()).style(Style::default().fg(FG)),
            Cell::from(format!("{:.1}%", s.cpu_pct)).style(Style::default().fg(cpu_c)),
            Cell::from(s.mem_used.split('/').next().unwrap_or("").trim().to_string())
                .style(Style::default().fg(FG)),
            Cell::from(format!("{:.1}%", s.mem_pct)).style(Style::default().fg(MUTED)),
            Cell::from(s.net_io.clone()).style(Style::default().fg(DIM)),
            Cell::from(s.block_io.clone()).style(Style::default().fg(DIM)),
        ])
    });
    frame.render_widget(
        Table::new(
            table_rows,
            [
                Constraint::Length(18),
                Constraint::Length(8),
                Constraint::Length(12),
                Constraint::Length(7),
                Constraint::Min(14),
                Constraint::Min(14),
            ],
        )
        .header(header)
        .column_spacing(1),
        inner,
    );

    // Images + binaries + volumes
    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ])
        .split(rows_layout[3]);

    draw_list_panel(
        frame,
        bottom[0],
        "Images (disk)",
        &snap
            .resources
            .images
            .iter()
            .map(|i| format!("{}  {}", i.repository, i.size))
            .collect::<Vec<_>>(),
    );
    draw_list_panel(
        frame,
        bottom[1],
        "Binaries",
        &{
            let mut lines: Vec<String> = snap
                .resources
                .binaries
                .iter()
                .map(|b| format!("{}  {}  {}", b.service, b.path, b.size))
                .collect();
            if lines.is_empty() {
                lines.push("runner images only (anchor/folio)".into());
                lines.push("dev services use cargo/trunk in-container".into());
            }
            lines
        },
    );
    draw_list_panel(
        frame,
        bottom[2],
        "Volumes",
        &snap
            .resources
            .volumes
            .iter()
            .map(|v| format!("{}  {}", short_name(&v.name), v.size))
            .collect::<Vec<_>>(),
    );
}

fn draw_list_panel(frame: &mut Frame, area: Rect, title: &str, lines: &[String]) {
    let block = panel(title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let para_lines: Vec<Line> = lines
        .iter()
        .map(|l| Line::from(Span::styled(l.clone(), Style::default().fg(FG))))
        .collect();
    frame.render_widget(
        Paragraph::new(para_lines).wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_telemetry(
    frame: &mut Frame,
    area: Rect,
    snap: &StatusSnapshot,
    history: &TelemetryHistory,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Min(6),
        ])
        .split(area);

    // Sparklines
    let lat = sparkline_u64(
        &history.backend_latency_ms.iter().copied().collect::<Vec<_>>(),
        (chunks[0].width.saturating_sub(24)) as usize,
    );
    let cpu = sparkline(
        &history.total_cpu_pct.iter().copied().collect::<Vec<_>>(),
        (chunks[0].width.saturating_sub(24)) as usize,
    );
    let mem = sparkline(
        &history.total_mem_mib.iter().copied().collect::<Vec<_>>(),
        (chunks[0].width.saturating_sub(24)) as usize,
    );
    let spark_lines = vec![
        Line::from(vec![
            Span::styled("/health ms   ", Style::default().fg(MUTED)),
            Span::styled(lat, Style::default().fg(ACCENT)),
            Span::styled(
                format!(
                    "  last {}",
                    snap.backend_latency_ms()
                        .map(|m| format!("{m}ms"))
                        .unwrap_or_else(|| "—".into())
                ),
                Style::default().fg(DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("CPU %        ", Style::default().fg(MUTED)),
            Span::styled(cpu, Style::default().fg(WARN)),
            Span::styled(
                format!("  now {:.1}%", snap.resources.totals.cpu_pct),
                Style::default().fg(DIM),
            ),
        ]),
        Line::from(vec![
            Span::styled("RAM MiB      ", Style::default().fg(MUTED)),
            Span::styled(mem, Style::default().fg(OK)),
            Span::styled(
                format!("  now {:.0}", snap.resources.totals.mem_used_mib),
                Style::default().fg(DIM),
            ),
        ]),
    ];
    let block = panel("Live trends (polled 3s — no SSE in platform)");
    let inner = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);
    frame.render_widget(Paragraph::new(spark_lines), inner);

    // Prometheus + DB aggregates
    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let prom_title = if snap.telemetry.metrics_ok {
        format!("Prometheus  {}", snap.telemetry.metrics_note)
    } else {
        "Prometheus".into()
    };
    let mut prom_lines: Vec<String> = snap.telemetry.prometheus_lines.clone();
    if !snap.telemetry.metrics_ok {
        prom_lines.insert(0, snap.telemetry.metrics_note.clone());
    }
    draw_list_panel(frame, mid[0], &prom_title, &prom_lines);

    let mut db_lines = Vec::new();
    if let Some(n) = snap.telemetry.unprocessed_events {
        db_lines.push(format!("telemetry_events unprocessed: {n}"));
    }
    for l in &snap.telemetry.daily_metrics {
        db_lines.push(format!("daily  {l}"));
    }
    for l in snap.telemetry.recent_events.iter().take(4) {
        db_lines.push(format!("event  {l}"));
    }
    if db_lines.is_empty() {
        db_lines.push("no telemetry rows yet (schema warming or idle)".into());
    }
    draw_list_panel(frame, mid[1], "DB telemetry", &db_lines);

    // Streaming feed
    let feed: Vec<String> = history.feed.iter().rev().take(40).cloned().collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let block = panel("Stream  (request_log · telemetry_events · /metrics)");
    let inner = block.inner(chunks[2]);
    frame.render_widget(block, chunks[2]);
    let lines: Vec<Line> = feed
        .iter()
        .map(|l| {
            let color = if l.contains(" prom ") {
                ACCENT
            } else if l.contains(" http ") {
                FG
            } else if l.contains(" event ") {
                OK
            } else {
                MUTED
            };
            Line::from(Span::styled(l.clone(), Style::default().fg(color)))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_footer(frame: &mut Frame, area: Rect, tab: Tab, overlay: &Overlay) {
    let (tab_hint, detail) = match (tab, overlay) {
        (_, Overlay::SmtpForm { .. }) => (
            "smtp form",
            "↑↓ fields · enter save · esc cancel — save alone does not reload backend",
        ),
        (_, Overlay::Banner(_)) => ("notice", "enter/esc dismiss · a apply if prompted"),
        (Tab::Overview, _) => (
            "overview",
            "x runs the first refresh in Next steps (e.g. network-instance anchor) — not a full stack rebuild",
        ),
        (Tab::Resources, _) => (
            "capacity",
            "app KPIs + stack load · x = Next-steps refresh · r = reload this panel only",
        ),
        (Tab::Telemetry, _) => (
            "telemetry",
            "x = Next-steps refresh (affected apps) · r = reload this panel only",
        ),
        (Tab::Env, _) => (
            "env / smtp",
            "s SMTP · a apply .env to backend · e editor · x = Next-steps app refresh",
        ),
    };
    let lines = vec![
        Line::from(vec![
            Span::styled(" q", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" quit", Style::default().fg(MUTED)),
            Span::styled("  r", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" reload panel", Style::default().fg(MUTED)),
            Span::styled("  x", Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled(" refresh apps (Next steps)", Style::default().fg(MUTED)),
            Span::styled(
                "  tab/←→/1234",
                Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!(" {tab_hint}"), Style::default().fg(MUTED)),
        ]),
        Line::from(Span::styled(detail, Style::default().fg(DIM))),
    ];
    frame.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }),
        area,
    );
}

fn draw_env(frame: &mut Frame, area: Rect, snap: &StatusSnapshot) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);
    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(6)])
        .split(cols[0]);

    let status_color = if snap.env_panel.smtp_mock { WARN } else { OK };
    let status_lines = vec![
        Line::from(Span::styled(
            snap.env_panel.smtp_status.clone(),
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            snap.env_panel.applied_hint.clone(),
            Style::default().fg(MUTED),
        )),
        Line::from(Span::styled(
            "Set writes .env.local only. Apply recreates backend so the app process sees it.",
            Style::default().fg(DIM),
        )),
    ];
    frame.render_widget(
        Paragraph::new(status_lines)
            .block(panel("SMTP status"))
            .wrap(Wrap { trim: true }),
        left[0],
    );

    let smtp_rows: Vec<Row> = snap
        .env_panel
        .smtp_rows
        .iter()
        .map(|(k, v)| {
            Row::new(vec![
                Cell::from(Span::styled(k.clone(), Style::default().fg(MUTED))),
                Cell::from(Span::styled(v.clone(), Style::default().fg(FG))),
            ])
        })
        .collect();
    frame.render_widget(
        Table::new(
            smtp_rows,
            [Constraint::Length(16), Constraint::Percentage(100)],
        )
        .block(panel("SMTP keys (.env.local / .env)"))
        .column_spacing(2),
        left[1],
    );

    let mut local_lines: Vec<Line> = snap
        .env_panel
        .local_rows
        .iter()
        .map(|(k, v)| {
            Line::from(vec![
                Span::styled(format!("{k}="), Style::default().fg(MUTED)),
                Span::styled(v.clone(), Style::default().fg(FG)),
            ])
        })
        .collect();
    if local_lines.is_empty() {
        local_lines.push(Line::from(Span::styled(
            "(no common keys set yet)",
            Style::default().fg(DIM),
        )));
    }
    local_lines.push(Line::from(""));
    local_lines.push(Line::from(Span::styled(
        "Keys: s SMTP form · a apply · e editor",
        Style::default().fg(ACCENT),
    )));
    frame.render_widget(
        Paragraph::new(local_lines)
            .block(panel("Other local overlay"))
            .wrap(Wrap { trim: true }),
        cols[1],
    );
}

fn draw_overlay(frame: &mut Frame, area: Rect, overlay: &Overlay) {
    match overlay {
        Overlay::None => {}
        Overlay::Banner(msg) => {
            let width = (area.width.saturating_sub(10)).min(72).max(30);
            let height = 5u16;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let y = area.y + (area.height.saturating_sub(height)) / 2;
            let rect = Rect::new(x, y, width, height);
            frame.render_widget(Clear, rect);
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        msg.clone(),
                        Style::default().fg(FG).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "enter / esc to dismiss",
                        Style::default().fg(MUTED),
                    )),
                ])
                .block(panel("Env"))
                .wrap(Wrap { trim: true })
                .alignment(Alignment::Center),
                rect,
            );
        }
        Overlay::SmtpForm {
            fields,
            focus,
            notice,
        } => {
            let width = (area.width.saturating_sub(8)).min(78).max(40);
            let height = 14u16;
            let x = area.x + (area.width.saturating_sub(width)) / 2;
            let y = area.y + (area.height.saturating_sub(height)) / 2;
            let rect = Rect::new(x, y, width, height);
            frame.render_widget(Clear, rect);

            let mut lines = vec![
                Line::from(Span::styled(
                    "Configure real SMTP (empty/localhost = mock)",
                    Style::default().fg(MUTED),
                )),
                Line::from(""),
            ];
            for (i, key) in SMTP_FIELD_KEYS.iter().enumerate() {
                let selected = i == *focus;
                let style = if selected {
                    Style::default().fg(TITLE).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(MUTED)
                };
                let marker = if selected { "›" } else { " " };
                let mut value = fields[i].clone();
                if crate::env::is_secret_key(key) && !selected && !value.is_empty() {
                    value = crate::env::mask_secret(&value);
                }
                if selected {
                    value.push('▌');
                }
                lines.push(Line::from(Span::styled(
                    format!("{marker} {key:<14} {value}"),
                    style,
                )));
            }
            lines.push(Line::from(""));
            if let Some(n) = notice {
                lines.push(Line::from(Span::styled(
                    n.clone(),
                    Style::default().fg(ERR),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "enter save · esc cancel",
                    Style::default().fg(DIM),
                )));
            }
            frame.render_widget(
                Paragraph::new(lines)
                    .block(panel("Set SMTP"))
                    .wrap(Wrap { trim: true }),
                rect,
            );
        }
    }
}

fn short_name(name: &str) -> &str {
    name.strip_prefix("atlas-platform_")
        .or_else(|| name.strip_prefix("atlas-platform-"))
        .unwrap_or(name)
}
