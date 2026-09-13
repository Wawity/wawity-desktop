use std::io::{self, Stdout};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use crate::actions::{self, human_bytes, ServerEntry};
use crate::store;

enum Job {
    Connect { index: usize },
    Disconnect,
    Ping,
    Refresh,
}

enum Msg {
    Log(String),
    Servers(Vec<ServerLine>),
    Status(wawity_core::ops::StatusSnapshot),
}

#[derive(Clone)]
struct ServerLine {
    name: String,
    protocol: String,
    host: String,
    sub: String,
    latency: Option<u64>,
    url: String,
}

pub struct App {
    servers: Vec<ServerLine>,
    list_state: ListState,
    logs: Vec<String>,
    status: Option<wawity_core::ops::StatusSnapshot>,
    input: String,
    input_mode: bool,
    entry_server: Option<String>,
    should_quit: bool,
    last_status_poll: Instant,
}

impl App {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let settings = store::load_settings();
        Self {
            servers: Vec::new(),
            list_state,
            logs: vec!["Wawity console ready. Loading servers...".into()],
            status: None,
            input: String::new(),
            input_mode: false,
            entry_server: settings.entry_server,
            should_quit: false,
            last_status_poll: Instant::now() - Duration::from_secs(10),
        }
    }

    fn log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 200 {
            let overflow = self.logs.len() - 200;
            self.logs.drain(0..overflow);
        }
    }

    fn selected(&self) -> Option<usize> {
        self.list_state.selected()
    }

    fn move_selection(&mut self, delta: i32) {
        if self.servers.is_empty() {
            return;
        }
        let len = self.servers.len() as i32;
        let cur = self.list_state.selected().unwrap_or(0) as i32;
        let next = ((cur + delta) % len + len) % len;
        self.list_state.select(Some(next as usize));
    }
}

pub fn run() -> Result<(), String> {
    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(|e| e.to_string())?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| e.to_string())?;

    let result = event_loop(&mut terminal);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();
    result
}

fn event_loop(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), String> {
    let mut app = App::new();
    let (tx, rx): (Sender<Msg>, Receiver<Msg>) = mpsc::channel();
    let (job_tx, job_rx): (Sender<Job>, Receiver<Job>) = mpsc::channel();

    spawn_worker(tx.clone(), job_rx);
    let _ = job_tx.send(Job::Refresh);

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Log(l) => app.log(l),
                Msg::Servers(list) => {
                    app.servers = list;
                    if app.list_state.selected().unwrap_or(0) >= app.servers.len() {
                        app.list_state.select(Some(0));
                    }
                    app.log(format!("Loaded {} servers.", app.servers.len()));
                }
                Msg::Status(s) => app.status = Some(s),
            }
        }

        if app.last_status_poll.elapsed() >= Duration::from_secs(2) {
            app.last_status_poll = Instant::now();
            let _ = job_tx.send(Job::Refresh);
        }

        terminal
            .draw(|f| draw(f, &mut app))
            .map_err(|e| e.to_string())?;

        if event::poll(Duration::from_millis(200)).map_err(|e| e.to_string())? {
            if let Event::Key(key) = event::read().map_err(|e| e.to_string())? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if app.input_mode {
                    handle_input_key(&mut app, &job_tx, key.code);
                } else {
                    handle_key(&mut app, &job_tx, key.code, key.modifiers);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

fn handle_key(
    app: &mut App,
    job_tx: &Sender<Job>,
    code: KeyCode,
    mods: KeyModifiers,
) {
    match code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => app.move_selection(-1),
        KeyCode::Down | KeyCode::Char('j') => app.move_selection(1),
        KeyCode::Enter => {
            if let Some(idx) = app.selected() {
                if idx < app.servers.len() {
                    app.log(format!("Connecting to {}...", app.servers[idx].name));
                    let _ = job_tx.send(Job::Connect { index: idx });
                }
            }
        }
        KeyCode::Char('d') => {
            app.log("Disconnecting...");
            let _ = job_tx.send(Job::Disconnect);
        }
        KeyCode::Char('p') => {
            app.log("Pinging servers...");
            let _ = job_tx.send(Job::Ping);
        }
        KeyCode::Char('r') => {
            app.log("Refreshing subscriptions...");
            let _ = job_tx.send(Job::Refresh);
        }
        KeyCode::Char('e') => {
            if let Some(idx) = app.selected() {
                if idx < app.servers.len() {
                    let name = app.servers[idx].name.clone();
                    app.entry_server = Some(name.clone());
                    let mut settings = store::load_settings();
                    settings.entry_server = Some(name.clone());
                    let _ = store::save_settings(&settings);
                    app.log(format!("Entry (multi-hop) server set: {}", name));
                }
            }
        }
        KeyCode::Char('x') => {
            app.entry_server = None;
            let mut settings = store::load_settings();
            settings.entry_server = None;
            let _ = store::save_settings(&settings);
            app.log("Entry server cleared.");
        }
        KeyCode::Char('a') => {
            app.input_mode = true;
            app.input.clear();
            app.log("Enter subscription URL, then press Enter (Esc to cancel).");
        }
        _ => {}
    }
}

fn handle_input_key(app: &mut App, job_tx: &Sender<Job>, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.input_mode = false;
            app.input.clear();
            app.log("Cancelled.");
        }
        KeyCode::Enter => {
            let url = app.input.trim().to_string();
            app.input_mode = false;
            app.input.clear();
            if url.is_empty() {
                app.log("Empty URL ignored.");
                return;
            }
            match actions::cmd_sub_add(url, None, false) {
                Ok(()) => {
                    app.log("Subscription added.");
                    let _ = job_tx.send(Job::Refresh);
                }
                Err(e) => app.log(format!("Add failed: {}", e)),
            }
        }
        KeyCode::Backspace => {
            app.input.pop();
        }
        KeyCode::Char(c) => app.input.push(c),
        _ => {}
    }
}

fn spawn_worker(tx: Sender<Msg>, job_rx: Receiver<Job>) {
    std::thread::spawn(move || {
        let mut cache: Vec<ServerLine> = Vec::new();
        for job in job_rx {
            match job {
                Job::Refresh => {
                    if let Ok(session) = wawity_core::ops::Session::new() {
                        let mut status = session.status();
                        if !status.connected {
                            if let Some(d) = store::load_detached() {
                                if let Some(pid) = d.pid {
                                    if actions::is_pid_alive(pid) {
                                        status.connected = true;
                                        status.pid = Some(pid);
                                        status.server_name = d.exit_server.clone();
                                        status.entry_server_name = d.entry_server.clone();
                                    }
                                }
                            }
                        }
                        let _ = tx.send(Msg::Status(status));
                    }
                    if cache.is_empty() {
                        match actions::collect_servers() {
                            Ok(entries) => {
                                cache = entries.iter().map(to_line).collect();
                                let _ = tx.send(Msg::Servers(cache.clone()));
                            }
                            Err(e) => {
                                let _ = tx.send(Msg::Log(format!("Server load failed: {}", e)));
                            }
                        }
                    }
                }
                Job::Ping => match actions::collect_servers() {
                    Ok(entries) => {
                        let targets: Vec<_> = entries
                            .iter()
                            .map(|e| wawity_core::engine::PingTarget {
                                host: e.server.server.clone(),
                                port: 443,
                            })
                            .collect();
                        let results = wawity_core::ops::ping(targets);
                        cache = entries
                            .iter()
                            .zip(results.iter())
                            .map(|(e, r)| {
                                let mut line = to_line(e);
                                line.latency = r.latency_ms;
                                line
                            })
                            .collect();
                        let _ = tx.send(Msg::Servers(cache.clone()));
                        let _ = tx.send(Msg::Log("Ping complete.".into()));
                    }
                    Err(e) => {
                        let _ = tx.send(Msg::Log(format!("Ping failed: {}", e)));
                    }
                },
                Job::Connect { index } => {
                    let settings = store::load_settings();
                    let line = match cache.get(index) {
                        Some(l) => l.clone(),
                        None => {
                            let _ = tx.send(Msg::Log("Invalid selection.".into()));
                            continue;
                        }
                    };
                    let entry = settings.entry_server.clone();
                    let entry_pair = match &entry {
                        Some(name) if !name.trim().is_empty() => cache
                            .iter()
                            .find(|l| l.name.eq_ignore_ascii_case(name))
                            .map(|l| (l.url.clone(), l.name.clone())),
                        _ => None,
                    };
                    let session = match wawity_core::ops::Session::new() {
                        Ok(s) => s,
                        Err(e) => {
                            let _ = tx.send(Msg::Log(format!("Session error: {}", e)));
                            continue;
                        }
                    };
                    session.set_privacy(actions::privacy_from(&settings));
                    let (entry_url, entry_name) = match entry_pair {
                        Some((u, n)) => (Some(u), Some(n)),
                        None => (None, None),
                    };
                    match session.connect(
                        &line.url,
                        entry_url,
                        Some(line.name.clone()),
                        entry_name.clone(),
                        settings.kill_switch,
                        settings.bypass_apps.clone(),
                        settings.quantum_resistant,
                    ) {
                        Ok(()) => {
                            let pid = session.status().pid;
                            let detached = store::DetachedState {
                                pid,
                                exit_server: Some(line.name.clone()),
                                entry_server: entry_name.clone(),
                                always_on: settings.kill_switch,
                                kill_switch: settings.kill_switch,
                                started_at: chrono::Utc::now().timestamp(),
                            };
                            let _ = store::save_detached(&detached);
                            std::thread::sleep(Duration::from_secs(3));
                            std::mem::forget(session);
                            let _ = tx.send(Msg::Log(format!(
                                "Connected to {} (pid {}).",
                                line.name,
                                pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into())
                            )));
                        }
                        Err(e) => {
                            let _ = tx.send(Msg::Log(format!("Connect failed: {}", e)));
                        }
                    }
                }
                Job::Disconnect => match actions::cmd_disconnect(false) {
                    Ok(()) => {
                        let _ = tx.send(Msg::Log("Disconnected.".into()));
                    }
                    Err(e) => {
                        let _ = tx.send(Msg::Log(format!("Disconnect failed: {}", e)));
                    }
                },
            }
        }
    });
}

fn to_line(e: &ServerEntry) -> ServerLine {
    ServerLine {
        name: e.server.name.clone(),
        protocol: e.server.protocol.clone(),
        host: e.server.server.clone(),
        sub: e.sub_name.clone(),
        latency: None,
        url: e.server.url.clone(),
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(f.size());

    draw_header(f, app, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    draw_servers(f, app, body[0]);
    draw_logs(f, app, body[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let (state_span, detail) = match &app.status {
        Some(s) if s.connected => {
            let name = s.server_name.clone().unwrap_or_else(|| "unknown".into());
            let entry = s
                .entry_server_name
                .as_ref()
                .map(|e| format!(" via {}", e))
                .unwrap_or_default();
            (
                Span::styled(
                    "CONNECTED",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                ),
                format!(
                    "  {}{}   down {} / up {}   killswitch:{}",
                    name,
                    entry,
                    human_bytes(s.bytes_rx),
                    human_bytes(s.bytes_tx),
                    if s.kill_switch { "on" } else { "off" }
                ),
            )
        }
        Some(s) if s.always_on_locked => (
            Span::styled(
                "LOCKED",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            "  always-on lockdown active (no tunnel)".to_string(),
        ),
        _ => (
            Span::styled(
                "DISCONNECTED",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            String::new(),
        ),
    };
    let entry_line = match &app.entry_server {
        Some(e) => format!("Multi-hop entry: {}", e),
        None => "Multi-hop entry: (none)".to_string(),
    };
    let lines = vec![
        Line::from(vec![
            Span::styled("Wawity", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("  VPN console"),
        ]),
        Line::from(vec![state_span, Span::raw(detail)]),
        Line::from(Span::styled(entry_line, Style::default().fg(Color::DarkGray))),
    ];
    let block = Block::default().borders(Borders::ALL);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_servers(f: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .servers
        .iter()
        .map(|s| {
            let lat = match s.latency {
                Some(ms) => format!("{:>4}ms", ms),
                None => "   -  ".into(),
            };
            let lat_color = match s.latency {
                Some(ms) if ms < 100 => Color::Green,
                Some(ms) if ms < 250 => Color::Yellow,
                Some(_) => Color::Red,
                None => Color::DarkGray,
            };
            Line::from(vec![
                Span::styled(lat, Style::default().fg(lat_color)),
                Span::raw("  "),
                Span::styled(format!("[{}] ", s.protocol), Style::default().fg(Color::Blue)),
                Span::raw(s.name.clone()),
                Span::styled(format!("  ({})", s.sub), Style::default().fg(Color::DarkGray)),
            ])
            .into()
        })
        .collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Servers "))
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let height = area.height.saturating_sub(2) as usize;
    let start = app.logs.len().saturating_sub(height);
    let text: Vec<Line> = app.logs[start..]
        .iter()
        .map(|l| Line::from(Span::raw(l.clone())))
        .collect();
    let block = Block::default().borders(Borders::ALL).title(" Log ");
    f.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: true }), area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let content = if app.input_mode {
        Line::from(vec![
            Span::styled("URL> ", Style::default().fg(Color::Yellow)),
            Span::raw(app.input.clone()),
        ])
    } else {
        Line::from(Span::styled(
            "[Enter] connect  [d] disconnect  [p] ping  [r] refresh  [e] set entry  [x] clear entry  [a] add sub  [q] quit",
            Style::default().fg(Color::DarkGray),
        ))
    };
    let block = Block::default().borders(Borders::ALL);
    f.render_widget(Paragraph::new(content).block(block), area);
}
