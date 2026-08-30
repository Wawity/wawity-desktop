use std::io::{self, Write};

use wawity_core::engine::PingTarget;
use wawity_core::ops::{self, Session, StatusSnapshot};

use crate::actions::{self, ConnectRequest, ServerEntry};
use crate::brand;
use crate::store::{self, Settings};
use crate::tui;

const PAGE: usize = 12;

struct Shell {
    servers: Vec<ServerEntry>,
    latency: Vec<Option<u64>>,
    order: Vec<usize>,
    filter: String,
    sort_by_ping: bool,
    page: usize,
    settings: Settings,
    admin: bool,
    rate: actions::RateMeter,
}

pub fn run() -> Result<(), String> {
    brand::init();
    brand::clear();
    brand::intro();

    let mut shell = Shell {
        servers: Vec::new(),
        latency: Vec::new(),
        order: Vec::new(),
        filter: String::new(),
        sort_by_ping: false,
        page: 0,
        settings: store::load_settings(),
        admin: has_admin_rights(),
        rate: actions::RateMeter::new(),
    };

    shell.status_line();
    if !shell.admin {
        brand::warn("\u{41d}\u{435}\u{442} \u{43f}\u{440}\u{430}\u{432} \u{430}\u{434}\u{43c}\u{438}\u{43d}\u{438}\u{441}\u{442}\u{440}\u{430}\u{442}\u{43e}\u{440}\u{430}: \u{43f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{435}\u{43d}\u{438}\u{435} \u{43d}\u{435} \u{437}\u{430}\u{440}\u{430}\u{431}\u{43e}\u{442}\u{430}\u{435}\u{442}. \u{417}\u{430}\u{43f}\u{443}\u{441}\u{442}\u{438}\u{442}\u{435} \u{442}\u{435}\u{440}\u{43c}\u{438}\u{43d}\u{430}\u{43b} \u{43e}\u{442} \u{438}\u{43c}\u{435}\u{43d}\u{438} \u{430}\u{434}\u{43c}\u{438}\u{43d}\u{438}\u{441}\u{442}\u{440}\u{430}\u{442}\u{43e}\u{440}\u{430}.");
    }
    shell.menu();

    loop {
        let raw = read_line("wawity");
        let line = normalize_input(&raw);
        if line.is_empty() {
            continue;
        }
        let lower = line.to_lowercase();
        let (head, rest) = match line.split_once(' ') {
            Some((h, r)) => (h.to_lowercase(), r.trim().to_string()),
            None => (lower.clone(), String::new()),
        };

        if matches!(head.as_str(), "0" | "q" | "quit" | "exit") {
            break;
        }

        shell.frame();

        match head.as_str() {
            "1" | "c" | "connect" => {
                if rest.is_empty() {
                    shell.flow_connect();
                } else {
                    shell.connect_token(&rest);
                }
            }
            "2" | "f" | "fast" | "fastest" => shell.connect_token("fastest"),
            "3" | "d" | "disconnect" | "off" => shell.flow_disconnect(),
            "4" | "s" | "servers" | "list" => shell.flow_servers(),
            "5" | "st" | "status" => shell.flow_status(),
            "6" | "sub" | "subs" => match rest.split_once(' ') {
                Some(("add", url)) => shell.sub_add(url),
                _ => {
                    if rest == "refresh" {
                        shell.sub_refresh();
                    } else {
                        shell.flow_subs();
                    }
                }
            },
            "7" | "set" | "settings" => shell.flow_settings(),
            "8" | "repair" | "fix" => shell.flow_repair(),
            "9" | "dash" | "panel" | "tui" => shell.flow_dash(),
            "m" | "menu" => shell.menu(),
            "r" | "reload" | "refresh" => {
                let _ = shell.load_servers(true);
            }
            "cls" | "clear" => shell.menu(),
            "h" | "help" | "?" => help(),
            _ => brand::err(&format!(
                "\u{41d}\u{435}\u{438}\u{437}\u{432}\u{435}\u{441}\u{442}\u{43d}\u{430}\u{44f} \u{43a}\u{43e}\u{43c}\u{430}\u{43d}\u{434}\u{430}: {}. \u{412}\u{432}\u{435}\u{434}\u{438}\u{442}\u{435} h \u{434}\u{43b}\u{44f} \u{441}\u{43f}\u{440}\u{430}\u{432}\u{43a}\u{438}.",
                truncate(&line, 60)
            )),
        }
    }

    brand::clear();
    brand::outro();
    Ok(())
}

impl Shell {
    fn frame(&mut self) {
        brand::clear();
        brand::banner();
        self.status_line();
        self.strip();
    }

    fn strip(&self) {
        println!(
            "  {}",
            brand::dim("1 \u{43f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{438}\u{442}\u{44c} \u{b7} 2 \u{431}\u{44b}\u{441}\u{442}\u{440}\u{43e} \u{b7} 3 \u{43e}\u{442}\u{43a}\u{43b}\u{44e}\u{447}\u{438}\u{442}\u{44c} \u{b7} 4 \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{44b} \u{b7} 5 \u{441}\u{442}\u{430}\u{442}\u{443}\u{441} \u{b7} 6 \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{438} \u{b7} 7 \u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{438} \u{b7} 8 \u{441}\u{435}\u{442}\u{44c} \u{b7} 9 \u{43f}\u{430}\u{43d}\u{435}\u{43b}\u{44c} \u{b7} m \u{43c}\u{435}\u{43d}\u{44e} \u{b7} 0 \u{432}\u{44b}\u{445}\u{43e}\u{434}")
        );
    }

    fn menu(&self) {
        brand::heading("\u{413}\u{43b}\u{430}\u{432}\u{43d}\u{43e}\u{435} \u{43c}\u{435}\u{43d}\u{44e}");
        let items = [
            ("1", "\u{41f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{438}\u{442}\u{44c}\u{441}\u{44f}", "\u{432}\u{44b}\u{431}\u{43e}\u{440} \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{430} \u{43f}\u{43e} \u{43d}\u{43e}\u{43c}\u{435}\u{440}\u{443}"),
            ("2", "\u{411}\u{44b}\u{441}\u{442}\u{440}\u{43e}\u{435} \u{43f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{435}\u{43d}\u{438}\u{435}", "\u{441}\u{430}\u{43c}\u{44b}\u{439} \u{43d}\u{438}\u{437}\u{43a}\u{438}\u{439} \u{43f}\u{438}\u{43d}\u{433}"),
            ("3", "\u{41e}\u{442}\u{43a}\u{43b}\u{44e}\u{447}\u{438}\u{442}\u{44c}\u{441}\u{44f}", "\u{440}\u{430}\u{437}\u{43e}\u{440}\u{432}\u{430}\u{442}\u{44c} \u{442}\u{443}\u{43d}\u{43d}\u{435}\u{43b}\u{44c}"),
            ("4", "\u{421}\u{435}\u{440}\u{432}\u{435}\u{440}\u{44b} \u{438} \u{43f}\u{438}\u{43d}\u{433}", "\u{441}\u{43f}\u{438}\u{441}\u{43e}\u{43a}, \u{444}\u{438}\u{43b}\u{44c}\u{442}\u{440}, \u{437}\u{430}\u{43c}\u{435}\u{440}"),
            ("5", "\u{421}\u{442}\u{430}\u{442}\u{443}\u{441}", "\u{441}\u{43e}\u{441}\u{442}\u{43e}\u{44f}\u{43d}\u{438}\u{435} \u{438} \u{442}\u{440}\u{430}\u{444}\u{438}\u{43a}"),
            ("6", "\u{41f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{438}", "\u{434}\u{43e}\u{431}\u{430}\u{432}\u{438}\u{442}\u{44c} / \u{43e}\u{431}\u{43d}\u{43e}\u{432}\u{438}\u{442}\u{44c} / \u{443}\u{434}\u{430}\u{43b}\u{438}\u{442}\u{44c}"),
            ("7", "\u{41d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{438}", "kill-switch, multi-hop, DNS"),
            ("8", "\u{412}\u{43e}\u{441}\u{441}\u{442}\u{430}\u{43d}\u{43e}\u{432}\u{438}\u{442}\u{44c} \u{441}\u{435}\u{442}\u{44c}", "\u{441}\u{431}\u{440}\u{43e}\u{441} \u{43f}\u{440}\u{430}\u{432}\u{438}\u{43b} \u{438} DNS"),
            ("9", "\u{41f}\u{430}\u{43d}\u{435}\u{43b}\u{44c} \u{43c}\u{43e}\u{43d}\u{438}\u{442}\u{43e}\u{440}\u{438}\u{43d}\u{433}\u{430}", "\u{436}\u{438}\u{432}\u{43e}\u{439} TUI-\u{434}\u{430}\u{448}\u{431}\u{43e}\u{440}\u{434}"),
            ("0", "\u{412}\u{44b}\u{445}\u{43e}\u{434}", "\u{437}\u{430}\u{43a}\u{440}\u{44b}\u{442}\u{44c} \u{43a}\u{43e}\u{43d}\u{441}\u{43e}\u{43b}\u{44c}"),
        ];
        for (key, title, hint) in items {
            println!(
                "  {}  {:<26} {}",
                brand::accent(key),
                title,
                brand::dim(hint)
            );
        }
        println!(
            "\n  {}",
            brand::dim("h \u{2014} \u{441}\u{43f}\u{440}\u{430}\u{432}\u{43a}\u{430}   r \u{2014} \u{43e}\u{431}\u{43d}\u{43e}\u{432}\u{438}\u{442}\u{44c} \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{44b}   sub add <url> \u{2014} \u{434}\u{43e}\u{431}\u{430}\u{432}\u{438}\u{442}\u{44c} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{443}")
        );
    }

    fn status_line(&mut self) {
        let snap = snapshot();
        let connected = snap.as_ref().map(|s| s.connected).unwrap_or(false);
        let (bytes_rx, bytes_tx) = match &snap {
            Some(s) if s.connected => (s.bytes_rx, s.bytes_tx),
            _ => (0, 0),
        };
        let (rx_rate, tx_rate) = self.rate.sample(bytes_rx, bytes_tx);
        let (dot, text) = if connected {
            let s = snap.as_ref().unwrap();
            (
                brand::fg(brand::GREEN, "\u{25cf}"),
                format!(
                    "{}  {}",
                    brand::bold(
                        s.server_name
                            .clone()
                            .unwrap_or_else(|| "\u{43f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{435}\u{43d}\u{43e}".into())
                            .as_str()
                    ),
                    brand::dim(&format!(
                        "\u{2193} {} ({})   \u{2191} {} ({})",
                        actions::human_bytes(bytes_rx),
                        actions::human_rate(rx_rate),
                        actions::human_bytes(bytes_tx),
                        actions::human_rate(tx_rate)
                    ))
                ),
            )
        } else {
            (
                brand::fg(brand::GREY, "\u{25cb}"),
                brand::dim("\u{43d}\u{435}\u{442} \u{441}\u{43e}\u{435}\u{434}\u{438}\u{43d}\u{435}\u{43d}\u{438}\u{44f}"),
            )
        };
        let mut flags: Vec<String> = Vec::new();
        let ks_live = snap.as_ref().map(|s| s.kill_switch).unwrap_or(false);
        if ks_live {
            flags.push(brand::fg(brand::GREEN, "kill-switch: \u{430}\u{43a}\u{442}\u{438}\u{432}\u{435}\u{43d}"));
        } else if self.settings.kill_switch {
            flags.push(brand::fg(brand::AMBER, "kill-switch: \u{432}\u{43a}\u{43b} (\u{43f}\u{440}\u{438} \u{43f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{435}\u{43d}\u{438}\u{438})"));
        } else {
            flags.push(brand::dim("kill-switch: \u{432}\u{44b}\u{43a}\u{43b}"));
        }
        if self.settings.quantum_resistant {
            flags.push(brand::dim("post-quantum"));
        }
        if let Some(entry) = self.settings.entry_server.clone() {
            if !entry.trim().is_empty() {
                flags.push(brand::dim(&format!("multi-hop \u{2192} {}", entry)));
            }
        }
        if let Some(pid) = snap.as_ref().and_then(|s| s.pid) {
            flags.push(brand::dim(&format!("pid {}", pid)));
        }
        if !self.servers.is_empty() {
            flags.push(brand::dim(&format!(
                "\u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{43e}\u{432}: {}",
                self.servers.len()
            )));
        }
        println!("  {} {}", dot, text);
        if !flags.is_empty() {
            println!("  {}", flags.join(brand::dim("   ").as_str()));
        }
        brand::rule();
    }

    fn load_servers(&mut self, force: bool) -> Result<(), String> {
        if !self.servers.is_empty() && !force {
            return Ok(());
        }
        let spinner = brand::Spinner::start(
            "\u{417}\u{430}\u{433}\u{440}\u{443}\u{437}\u{43a}\u{430} \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{43e}\u{432} \u{438}\u{437} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43e}\u{43a}",
        );
        match actions::collect_servers() {
            Ok(list) => {
                let count = list.len();
                self.latency = vec![None; count];
                self.servers = list;
                self.page = 0;
                self.rebuild_order();
                spinner.done(&format!(
                    "\u{421}\u{435}\u{440}\u{432}\u{435}\u{440}\u{43e}\u{432} \u{434}\u{43e}\u{441}\u{442}\u{443}\u{43f}\u{43d}\u{43e}: {}",
                    count
                ));
                Ok(())
            }
            Err(e) => {
                spinner.fail(&e);
                Err(e)
            }
        }
    }

    fn rebuild_order(&mut self) {
        let needle = self.filter.to_lowercase();
        let mut idx: Vec<usize> = (0..self.servers.len())
            .filter(|i| {
                if needle.is_empty() {
                    return true;
                }
                let entry = &self.servers[*i];
                entry.server.name.to_lowercase().contains(&needle)
                    || entry.server.server.to_lowercase().contains(&needle)
                    || entry.server.protocol.to_lowercase().contains(&needle)
                    || entry.sub_name.to_lowercase().contains(&needle)
            })
            .collect();
        if self.sort_by_ping {
            idx.sort_by_key(|i| self.latency[*i].unwrap_or(u64::MAX));
        }
        self.order = idx;
        let pages = self.page_count();
        if self.page >= pages {
            self.page = pages.saturating_sub(1);
        }
    }

    fn page_count(&self) -> usize {
        if self.order.is_empty() {
            1
        } else {
            (self.order.len() + PAGE - 1) / PAGE
        }
    }

    fn render_servers(&self) {
        brand::heading("\u{421}\u{435}\u{440}\u{432}\u{435}\u{440}\u{44b}");
        if self.order.is_empty() {
            brand::warn("\u{421}\u{43f}\u{438}\u{441}\u{43e}\u{43a} \u{43f}\u{443}\u{441}\u{442}. \u{414}\u{43e}\u{431}\u{430}\u{432}\u{44c}\u{442}\u{435} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{443} (\u{43f}\u{443}\u{43d}\u{43a}\u{442} 6).");
            return;
        }
        println!(
            "  {}",
            brand::dim(&format!(
                "{:>4}  {:<30} {:<10} {:>8}  {}",
                "#", "\u{421}\u{415}\u{420}\u{412}\u{415}\u{420}", "\u{41f}\u{420}\u{41e}\u{422}\u{41e}\u{41a}\u{41e}\u{41b}", "\u{41f}\u{418}\u{41d}\u{413}", "\u{41f}\u{41e}\u{414}\u{41f}\u{418}\u{421}\u{41a}\u{410}"
            ))
        );
        let start = self.page * PAGE;
        let end = (start + PAGE).min(self.order.len());
        let last = store::load_detached().and_then(|d| d.exit_server);
        for slot in start..end {
            let i = self.order[slot];
            let entry = &self.servers[i];
            let mark = match &last {
                Some(name) if *name == entry.server.name => brand::fg(brand::AMBER, "\u{2605}"),
                _ => " ".to_string(),
            };
            let ping = match self.latency[i] {
                Some(ms) => {
                    let color = if ms < 80 {
                        brand::GREEN
                    } else if ms < 180 {
                        brand::AMBER
                    } else {
                        brand::RED
                    };
                    brand::fg(color, &format!("{} ms", ms))
                }
                None => brand::dim("\u{2014}"),
            };
            println!(
                "  {}{:>3}  {:<30} {:<10} {:>17}  {}",
                mark,
                brand::accent(&(slot + 1).to_string()),
                truncate(&entry.server.name, 30),
                brand::dim(&entry.server.protocol),
                ping,
                brand::dim(&truncate(&entry.sub_name, 18))
            );
        }
        println!(
            "\n  {}",
            brand::dim(&format!(
                "\u{441}\u{442}\u{440}\u{430}\u{43d}\u{438}\u{446}\u{430} {}/{}   \u{432}\u{441}\u{435}\u{433}\u{43e}: {}{}",
                self.page + 1,
                self.page_count(),
                self.order.len(),
                if self.filter.is_empty() {
                    String::new()
                } else {
                    format!("   \u{444}\u{438}\u{43b}\u{44c}\u{442}\u{440}: {}", self.filter)
                }
            ))
        );
    }

    fn ping_all(&mut self) {
        if self.servers.is_empty() {
            return;
        }
        let targets: Vec<PingTarget> = self
            .servers
            .iter()
            .map(|e| PingTarget {
                host: e.server.server.clone(),
                port: 443,
            })
            .collect();
        let spinner = brand::Spinner::start(
            "\u{417}\u{430}\u{43c}\u{435}\u{440} \u{437}\u{430}\u{434}\u{435}\u{440}\u{436}\u{43a}\u{438}",
        );
        let results = ops::ping(targets);
        for (i, r) in results.iter().enumerate() {
            if i < self.latency.len() {
                self.latency[i] = r.latency_ms;
            }
        }
        let alive = self.latency.iter().filter(|v| v.is_some()).count();
        spinner.done(&format!(
            "\u{414}\u{43e}\u{441}\u{442}\u{443}\u{43f}\u{43d}\u{43e} \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{43e}\u{432}: {}",
            alive
        ));
        self.rebuild_order();
    }

    fn pick_server(&mut self, title: &str) -> Option<usize> {
        loop {
            self.frame();
            self.render_servers();
            println!(
                "  {}",
                brand::dim("n \u{2014} \u{434}\u{430}\u{43b}\u{44c}\u{448}\u{435}   b \u{2014} \u{43d}\u{430}\u{437}\u{430}\u{434}   p \u{2014} \u{43f}\u{438}\u{43d}\u{433}   s \u{2014} \u{441}\u{43e}\u{440}\u{442} \u{43f}\u{43e} \u{43f}\u{438}\u{43d}\u{433}\u{443}   f <\u{442}\u{435}\u{43a}\u{441}\u{442}> \u{2014} \u{444}\u{438}\u{43b}\u{44c}\u{442}\u{440}   a \u{2014} \u{441}\u{431}\u{440}\u{43e}\u{441}   0 \u{2014} \u{43d}\u{430}\u{437}\u{430}\u{434}")
            );
            let raw = read_line(title);
            let line = raw.trim().to_lowercase();
            if line.is_empty() {
                continue;
            }
            if line == "0" {
                return None;
            }
            if let Ok(num) = line.parse::<usize>() {
                if num >= 1 && num <= self.order.len() {
                    return Some(self.order[num - 1]);
                }
                self.frame();
                brand::err("\u{41d}\u{43e}\u{43c}\u{435}\u{440} \u{432}\u{43d}\u{435} \u{441}\u{43f}\u{438}\u{441}\u{43a}\u{430}");
                continue;
            }
            match line.as_str() {
                "n" => {
                    if self.page + 1 < self.page_count() {
                        self.page += 1;
                    }
                }
                "b" => {
                    self.page = self.page.saturating_sub(1);
                }
                "p" => self.ping_all(),
                "s" => {
                    self.sort_by_ping = !self.sort_by_ping;
                    self.rebuild_order();
                }
                "a" => {
                    self.filter.clear();
                    self.page = 0;
                    self.rebuild_order();
                }
                _ => {
                    if let Some(text) = line.strip_prefix("f ") {
                        self.filter = text.trim().to_string();
                        self.page = 0;
                        self.rebuild_order();
                    }
                }
            }
        }
    }

    fn flow_connect(&mut self) {
        if self.load_servers(false).is_err() {
            return;
        }
        if let Some(idx) = self.pick_server("\u{441}\u{435}\u{440}\u{432}\u{435}\u{440}") {
            let name = self.servers[idx].server.name.clone();
            self.frame();
            self.connect_token(&name);
        } else {
            self.frame();
            self.menu();
        }
    }

    fn connect_token(&mut self, target: &str) {
        if !self.admin {
            brand::warn("\u{422}\u{440}\u{435}\u{431}\u{443}\u{44e}\u{442}\u{441}\u{44f} \u{43f}\u{440}\u{430}\u{432}\u{430} \u{430}\u{434}\u{43c}\u{438}\u{43d}\u{438}\u{441}\u{442}\u{440}\u{430}\u{442}\u{43e}\u{440}\u{430}");
        }
        let resolved = match target.parse::<usize>() {
            Ok(num) if num >= 1 && num <= self.order.len() => {
                self.servers[self.order[num - 1]].server.name.clone()
            }
            _ => target.to_string(),
        };
        brand::info(&format!(
            "\u{41f}\u{43e}\u{434}\u{43a}\u{43b}\u{44e}\u{447}\u{435}\u{43d}\u{438}\u{435}: {}",
            brand::bold(&resolved)
        ));
        let request = ConnectRequest {
            target: resolved,
            entry: self.settings.entry_server.clone(),
            kill_switch: self.settings.kill_switch,
            quantum_resistant: self.settings.quantum_resistant,
            foreground: false,
        };
        match actions::cmd_connect(request, false) {
            Ok(()) => brand::ok("\u{422}\u{443}\u{43d}\u{43d}\u{435}\u{43b}\u{44c} \u{43f}\u{43e}\u{434}\u{43d}\u{44f}\u{442}"),
            Err(e) => brand::err(&e),
        }
    }

    fn flow_disconnect(&mut self) {
        match actions::cmd_disconnect(false) {
            Ok(()) => brand::ok("\u{422}\u{443}\u{43d}\u{43d}\u{435}\u{43b}\u{44c} \u{437}\u{430}\u{43a}\u{440}\u{44b}\u{442}"),
            Err(e) => brand::err(&e),
        }
    }

    fn flow_servers(&mut self) {
        if self.load_servers(false).is_err() {
            return;
        }
        self.ping_all();
        self.frame();
        self.render_servers();
    }

    fn flow_status(&mut self) {
        brand::heading("\u{421}\u{442}\u{430}\u{442}\u{443}\u{441}");
        if let Err(e) = actions::cmd_status(false, false) {
            brand::err(&e);
        }
    }

    fn flow_repair(&mut self) {
        brand::heading("\u{412}\u{43e}\u{441}\u{441}\u{442}\u{430}\u{43d}\u{43e}\u{432}\u{43b}\u{435}\u{43d}\u{438}\u{435} \u{441}\u{435}\u{442}\u{438}");
        match actions::cmd_repair(false) {
            Ok(()) => brand::ok("\u{421}\u{435}\u{442}\u{44c} \u{432}\u{43e}\u{441}\u{441}\u{442}\u{430}\u{43d}\u{43e}\u{432}\u{43b}\u{435}\u{43d}\u{430}"),
            Err(e) => brand::err(&e),
        }
    }

    fn flow_dash(&mut self) {
        if let Err(e) = tui::run() {
            brand::err(&e);
        }
        self.frame();
        self.menu();
    }

    fn sub_add(&mut self, url: &str) {
        let clean = clean_url(url);
        if clean.is_empty() {
            brand::err("\u{41f}\u{443}\u{441}\u{442}\u{43e}\u{439} URL");
            return;
        }
        brand::heading("\u{414}\u{43e}\u{431}\u{430}\u{432}\u{43b}\u{435}\u{43d}\u{438}\u{435} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{438}");
        brand::info(&truncate(&clean, 70));
        match actions::cmd_sub_add(clean, None, false) {
            Ok(()) => {
                let _ = self.load_servers(true);
            }
            Err(e) => brand::err(&e),
        }
    }

    fn sub_refresh(&mut self) {
        brand::heading("\u{41e}\u{431}\u{43d}\u{43e}\u{432}\u{43b}\u{435}\u{43d}\u{438}\u{435} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43e}\u{43a}");
        match actions::cmd_sub_refresh(None, false) {
            Ok(()) => {
                let _ = self.load_servers(true);
            }
            Err(e) => brand::err(&e),
        }
    }

    fn flow_subs(&mut self) {
        loop {
            self.frame();
            brand::heading("\u{41f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{438}");
            let subs = store::load_subscriptions();
            if subs.is_empty() {
                brand::warn("\u{41f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43e}\u{43a} \u{43d}\u{435}\u{442}");
            } else {
                for (i, s) in subs.iter().enumerate() {
                    println!(
                        "  {:>3}  {:<24} {}",
                        brand::accent(&(i + 1).to_string()),
                        truncate(&s.name, 24),
                        brand::dim(&truncate(&s.url, 46))
                    );
                }
            }
            println!();
            println!("  {}  \u{414}\u{43e}\u{431}\u{430}\u{432}\u{438}\u{442}\u{44c}", brand::accent("1"));
            println!("  {}  \u{41e}\u{431}\u{43d}\u{43e}\u{432}\u{438}\u{442}\u{44c} \u{432}\u{441}\u{435}", brand::accent("2"));
            println!("  {}  \u{423}\u{434}\u{430}\u{43b}\u{438}\u{442}\u{44c} \u{43f}\u{43e} \u{43d}\u{43e}\u{43c}\u{435}\u{440}\u{443}", brand::accent("3"));
            println!("  {}  \u{41d}\u{430}\u{437}\u{430}\u{434}", brand::accent("0"));
            let choice = read_line("\u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{438}");
            match choice.trim() {
                "1" => {
                    let url = clean_url(&read_line("url"));
                    if url.is_empty() {
                        continue;
                    }
                    let name = read_line("\u{438}\u{43c}\u{44f}").trim().to_string();
                    let named = if name.is_empty() { None } else { Some(name) };
                    self.frame();
                    brand::heading("\u{414}\u{43e}\u{431}\u{430}\u{432}\u{43b}\u{435}\u{43d}\u{438}\u{435} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{438}");
                    match actions::cmd_sub_add(url, named, false) {
                        Ok(()) => {
                            let _ = self.load_servers(true);
                            wait_key();
                        }
                        Err(e) => {
                            brand::err(&e);
                            wait_key();
                        }
                    }
                }
                "2" => {
                    self.frame();
                    brand::heading("\u{41e}\u{431}\u{43d}\u{43e}\u{432}\u{43b}\u{435}\u{43d}\u{438}\u{435}");
                    match actions::cmd_sub_refresh(None, false) {
                        Ok(()) => {
                            let _ = self.load_servers(true);
                        }
                        Err(e) => brand::err(&e),
                    }
                    wait_key();
                }
                "3" => {
                    let num = read_line("\u{43d}\u{43e}\u{43c}\u{435}\u{440}").trim().parse::<usize>();
                    match num {
                        Ok(n) if n >= 1 && n <= subs.len() => {
                            let id = subs[n - 1].id.clone();
                            match actions::cmd_sub_rm(id, false) {
                                Ok(()) => {
                                    let _ = self.load_servers(true);
                                }
                                Err(e) => brand::err(&e),
                            }
                        }
                        _ => {
                            brand::err("\u{41d}\u{435}\u{432}\u{435}\u{440}\u{43d}\u{44b}\u{439} \u{43d}\u{43e}\u{43c}\u{435}\u{440}");
                            wait_key();
                        }
                    }
                }
                "0" | "" => break,
                _ => {}
            }
        }
        self.frame();
        self.menu();
    }

    fn flow_settings(&mut self) {
        loop {
            self.frame();
            brand::heading("\u{41d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{438}");
            let entry = self
                .settings
                .entry_server
                .clone()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "\u{43d}\u{435}\u{442}".to_string());
            let rows = [
                ("1", "Kill-switch", flag(self.settings.kill_switch)),
                ("2", "\u{41f}\u{43e}\u{441}\u{442}-\u{43a}\u{432}\u{430}\u{43d}\u{442}\u{43e}\u{432}\u{43e}\u{435} \u{448}\u{438}\u{444}\u{440}\u{43e}\u{432}\u{430}\u{43d}\u{438}\u{435}", flag(self.settings.quantum_resistant)),
                ("3", "\u{422}\u{43e}\u{447}\u{43a}\u{430} \u{432}\u{445}\u{43e}\u{434}\u{430} (multi-hop)", entry.clone()),
                ("4", "\u{421}\u{442}\u{440}\u{43e}\u{433}\u{430}\u{44f} \u{43c}\u{430}\u{440}\u{448}\u{440}\u{443}\u{442}\u{438}\u{437}\u{430}\u{446}\u{438}\u{44f}", flag(self.settings.strict_route)),
                ("5", "\u{417}\u{430}\u{449}\u{438}\u{442}\u{430} \u{43e}\u{442} DNS-\u{443}\u{442}\u{435}\u{447}\u{435}\u{43a}", flag(self.settings.dns_leak_guard)),
                ("6", "Bootstrap DNS", self.settings.bootstrap_dns.clone()),
            ];
            for (key, title, value) in rows {
                println!(
                    "  {}  {:<34} {}",
                    brand::accent(key),
                    title,
                    brand::fg(brand::CYAN, &value)
                );
            }
            println!("  {}  \u{41d}\u{430}\u{437}\u{430}\u{434}", brand::accent("0"));
            let choice = read_line("\u{43d}\u{430}\u{441}\u{442}\u{440}\u{43e}\u{439}\u{43a}\u{438}");
            match choice.trim() {
                "1" => self.settings.kill_switch = !self.settings.kill_switch,
                "2" => self.settings.quantum_resistant = !self.settings.quantum_resistant,
                "3" => {
                    if self.load_servers(false).is_ok() {
                        match self.pick_server("\u{442}\u{43e}\u{447}\u{43a}\u{430} \u{432}\u{445}\u{43e}\u{434}\u{430}") {
                            Some(idx) => {
                                self.settings.entry_server =
                                    Some(self.servers[idx].server.name.clone())
                            }
                            None => self.settings.entry_server = None,
                        }
                    }
                }
                "4" => self.settings.strict_route = !self.settings.strict_route,
                "5" => self.settings.dns_leak_guard = !self.settings.dns_leak_guard,
                "6" => {
                    let value = read_line("dns").trim().to_string();
                    if !value.is_empty() {
                        self.settings.bootstrap_dns = value;
                    }
                }
                "0" | "" => break,
                _ => {}
            }
            if let Err(e) = store::save_settings(&self.settings) {
                brand::err(&e);
                wait_key();
            }
        }
        self.frame();
        self.menu();
    }
}

fn flag(value: bool) -> String {
    if value {
        "\u{432}\u{43a}\u{43b}".to_string()
    } else {
        "\u{432}\u{44b}\u{43a}\u{43b}".to_string()
    }
}

fn truncate(text: &str, max: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        return text.to_string();
    }
    let mut out: String = chars[..max.saturating_sub(1)].iter().collect();
    out.push('\u{2026}');
    out
}

fn normalize_input(raw: &str) -> String {
    let mut line = raw.trim().to_string();
    for prefix in ["wawity.exe ", "wawity "] {
        if line.to_lowercase().starts_with(prefix) {
            line = line[prefix.len()..].trim().to_string();
            break;
        }
    }
    line
}

fn clean_url(raw: &str) -> String {
    let mut text = raw.trim().to_string();
    if let Some(open) = text.find("](") {
        if text.starts_with('[') && text.ends_with(')') {
            text = text[open + 2..text.len() - 1].to_string();
        }
    }
    text = text
        .trim_matches(|c| matches!(c, '<' | '>' | '[' | ']' | '(' | ')' | '"' | '\'' | ' '))
        .to_string();
    text
}

fn read_line(label: &str) -> String {
    print!(
        "\n  {} {} ",
        brand::accent(label),
        brand::fg(brand::CYAN, "\u{203a}")
    );
    let _ = io::stdout().flush();
    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(0) => "0".to_string(),
        Ok(_) => buffer,
        Err(_) => "0".to_string(),
    }
}

fn wait_key() {
    print!(
        "\n  {} ",
        brand::dim("Enter \u{2014} \u{43f}\u{440}\u{43e}\u{434}\u{43e}\u{43b}\u{436}\u{438}\u{442}\u{44c}")
    );
    let _ = io::stdout().flush();
    let mut sink = String::new();
    let _ = io::stdin().read_line(&mut sink);
}

fn snapshot() -> Option<StatusSnapshot> {
    let session = Session::new().ok()?;
    let mut status = session.status();
    actions::enrich_status(&mut status);
    if status.connected {
        let (rx, tx) = actions::read_traffic(&status);
        if rx > 0 || tx > 0 {
            status.bytes_rx = rx;
            status.bytes_tx = tx;
        }
    }
    Some(status)
}

#[cfg(windows)]
fn has_admin_rights() -> bool {
    use std::os::windows::process::CommandExt;
    std::process::Command::new("net")
        .args(["session"])
        .creation_flags(0x0800_0000)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn has_admin_rights() -> bool {
    true
}

fn help() {
    brand::heading("\u{421}\u{43f}\u{440}\u{430}\u{432}\u{43a}\u{430}");
    let rows = [
        ("1 | connect [\u{43d}\u{43e}\u{43c}\u{435}\u{440}]", "\u{432}\u{44b}\u{431}\u{43e}\u{440} \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}\u{430} \u{43f}\u{43e} \u{43d}\u{43e}\u{43c}\u{435}\u{440}\u{443}"),
        ("2 | fast", "\u{441}\u{430}\u{43c}\u{44b}\u{439} \u{431}\u{44b}\u{441}\u{442}\u{440}\u{44b}\u{439} \u{441}\u{435}\u{440}\u{432}\u{435}\u{440}"),
        ("3 | disconnect", "\u{440}\u{430}\u{437}\u{440}\u{44b}\u{432} \u{442}\u{443}\u{43d}\u{43d}\u{435}\u{43b}\u{44f}"),
        ("4 | servers", "\u{441}\u{43f}\u{438}\u{441}\u{43e}\u{43a} \u{441} \u{43f}\u{438}\u{43d}\u{433}\u{43e}\u{43c}"),
        ("5 | status", "\u{442}\u{435}\u{43a}\u{443}\u{449}\u{435}\u{435} \u{441}\u{43e}\u{441}\u{442}\u{43e}\u{44f}\u{43d}\u{438}\u{435}"),
        ("6 | subs", "\u{43c}\u{435}\u{43d}\u{44e} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43e}\u{43a}"),
        ("sub add <url>", "\u{434}\u{43e}\u{431}\u{430}\u{432}\u{438}\u{442}\u{44c} \u{43f}\u{43e}\u{434}\u{43f}\u{438}\u{441}\u{43a}\u{443} \u{441}\u{440}\u{430}\u{437}\u{443}"),
        ("7 | settings", "kill-switch, multi-hop, DNS"),
        ("8 | repair", "\u{441}\u{431}\u{440}\u{43e}\u{441} \u{441}\u{435}\u{442}\u{435}\u{432}\u{44b}\u{445} \u{43f}\u{440}\u{430}\u{432}\u{438}\u{43b}"),
        ("9 | dash", "\u{436}\u{438}\u{432}\u{43e}\u{439} \u{434}\u{430}\u{448}\u{431}\u{43e}\u{440}\u{434}"),
        ("m | menu", "\u{43f}\u{43e}\u{43b}\u{43d}\u{43e}\u{435} \u{43c}\u{435}\u{43d}\u{44e}"),
        ("0 | exit", "\u{432}\u{44b}\u{445}\u{43e}\u{434}"),
    ];
    for (cmd, hint) in rows {
        println!("  {:<26} {}", brand::fg(brand::CYAN, cmd), brand::dim(hint));
    }
}
