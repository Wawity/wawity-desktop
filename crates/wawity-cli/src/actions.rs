use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use wawity_core::engine::{ParsedServer, PingTarget, PrivacyOptions};
use wawity_core::ops::{self, Session};
use wawity_core::util;

use crate::store::{self, Settings, Subscription};

pub fn privacy_from(settings: &Settings) -> PrivacyOptions {
    PrivacyOptions {
        strict_route: settings.strict_route,
        allow_insecure_tls: settings.allow_insecure_tls,
        tunnel_own_traffic: settings.tunnel_own_traffic,
        dns_leak_guard: settings.dns_leak_guard,
        bootstrap_dns: settings.bootstrap_dns.clone(),
        ..Default::default()
    }
}

pub fn is_pid_alive(pid: u32) -> bool {
    let output = util::silent_command("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH"])
        .output();
    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            text.contains(&pid.to_string())
        }
        Err(_) => false,
    }
}

pub struct ServerEntry {
    pub sub_name: String,
    pub server: ParsedServer,
}

pub fn collect_servers() -> Result<Vec<ServerEntry>, String> {
    let subs = store::load_subscriptions();
    if subs.is_empty() {
        return Err("No subscriptions configured. Add one with: wawity sub add <url>".into());
    }
    let mut all = Vec::new();
    let mut errors = Vec::new();
    for sub in &subs {
        match ops::fetch_subscription_raw(&sub.url) {
            Ok(servers) => {
                for s in servers {
                    all.push(ServerEntry {
                        sub_name: sub.name.clone(),
                        server: s,
                    });
                }
            }
            Err(e) => errors.push(format!("{}: {}", sub.name, e)),
        }
    }
    if all.is_empty() {
        return Err(format!("No servers available. {}", errors.join("; ")));
    }
    Ok(all)
}

pub fn find_server(entries: &[ServerEntry], needle: &str) -> Option<usize> {
    let lower = needle.to_lowercase();
    if let Some(i) = entries.iter().position(|e| e.server.name.to_lowercase() == lower) {
        return Some(i);
    }
    entries
        .iter()
        .position(|e| e.server.name.to_lowercase().contains(&lower))
}

pub fn pick_fastest(entries: &[ServerEntry]) -> Option<usize> {
    let targets: Vec<PingTarget> = entries
        .iter()
        .map(|e| PingTarget {
            host: e.server.server.clone(),
            port: 443,
        })
        .collect();
    let results = ops::ping(targets);
    results
        .iter()
        .enumerate()
        .filter_map(|(i, r)| r.latency_ms.map(|ms| (i, ms)))
        .min_by_key(|(_, ms)| *ms)
        .map(|(i, _)| i)
}

pub struct ConnectRequest {
    pub target: String,
    pub entry: Option<String>,
    pub kill_switch: bool,
    pub quantum_resistant: bool,
    pub foreground: bool,
}

pub fn cmd_connect(req: ConnectRequest, json: bool) -> Result<(), String> {
    let settings = store::load_settings();
    let entries = collect_servers()?;

    let exit_idx = if req.target.eq_ignore_ascii_case("fastest") {
        pick_fastest(&entries).ok_or("No reachable server for fastest selection")?
    } else {
        find_server(&entries, &req.target)
            .ok_or_else(|| format!("Server not found: {}", req.target))?
    };
    let exit = &entries[exit_idx];
    if exit.server.url.trim().is_empty() {
        return Err(format!(
            "Сервер '{}' не содержит прямой ссылки. Обновите подписку: wawity sub refresh",
            exit.server.name
        ));
    }

    let entry_name = req.entry.clone().or_else(|| settings.entry_server.clone());
    let (entry_url, entry_display) = match &entry_name {
        Some(name) if !name.trim().is_empty() => {
            let idx = find_server(&entries, name)
                .ok_or_else(|| format!("Entry server not found: {}", name))?;
            (
                Some(entries[idx].server.url.clone()),
                Some(entries[idx].server.name.clone()),
            )
        }
        _ => (None, None),
    };

    let session = Session::new()?;
    session.set_privacy(privacy_from(&settings));

    session.connect(
        &exit.server.url,
        entry_url,
        Some(exit.server.name.clone()),
        entry_display.clone(),
        req.kill_switch,
        settings.bypass_apps.clone(),
        req.quantum_resistant,
    )?;

    let status = session.status();

    if req.foreground {
        if json {
            println!("{}", serde_json::to_string(&status).unwrap_or_default());
        } else {
            println!("Connected to {} (foreground). Press Ctrl+C to disconnect.", exit.server.name);
            if let Some(entry) = &entry_display {
                println!("Multi-hop entry: {}", entry);
            }
        }
        run_foreground(session, req.kill_switch || settings.kill_switch);
        Ok(())
    } else {
        let pid = session.status().pid;
        let detached = store::DetachedState {
            pid,
            exit_server: Some(exit.server.name.clone()),
            entry_server: entry_display.clone(),
            always_on: req.kill_switch,
            kill_switch: req.kill_switch || settings.kill_switch,
            started_at: chrono::Utc::now().timestamp(),
        };
        store::save_detached(&detached)?;
        std::thread::sleep(Duration::from_secs(4));
        std::mem::forget(session);
        if json {
            println!("{}", serde_json::to_string(&detached).unwrap_or_default());
        } else {
            println!(
                "Connected to {} (detached, pid {}). Use 'wawity disconnect' to stop.",
                exit.server.name,
                pid.map(|p| p.to_string()).unwrap_or_else(|| "?".into())
            );
        }
        Ok(())
    }
}

fn run_foreground(session: Session, _lockdown: bool) {
    let running = Arc::new(AtomicBool::new(true));
    let flag = Arc::clone(&running);
    let _ = ctrlc::set_handler(move || {
        flag.store(false, Ordering::SeqCst);
    });
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(500));
        {
            let mut routing = session.handles().routing_manager.lock().unwrap();
            let _ = routing.verify_and_repair();
        }
        if !session.is_running() {
            eprintln!("sing-box exited unexpectedly.");
            break;
        }
    }
    eprintln!("Disconnecting...");
    if let Err(e) = session.disconnect() {
        eprintln!("Disconnect error: {}", e);
    }
}

pub fn cmd_disconnect(json: bool) -> Result<(), String> {
    let settings = store::load_settings();
    if let Some(detached) = store::load_detached() {
        let errors = ops::force_teardown_detached(detached.pid, detached.always_on);
        store::clear_detached();
        if !errors.is_empty() {
            return Err(errors.join("; "));
        }
        if json {
            println!("{{\"disconnected\":true}}");
        } else {
            println!("Disconnected detached session.");
        }
        return Ok(());
    }
    let session = Session::new()?;
    session.set_always_on_flag(settings.kill_switch);
    session.disconnect()?;
    if json {
        println!("{{\"disconnected\":true}}");
    } else {
        println!("Disconnected.");
    }
    Ok(())
}

pub fn cmd_status(json: bool, watch: bool) -> Result<(), String> {
    let mut meter = RateMeter::new();
    loop {
        let session = Session::new()?;
        let mut status = session.status();
        let detached = store::load_detached();
        if !status.connected {
            if let Some(d) = &detached {
                if let Some(pid) = d.pid {
                    if is_pid_alive(pid) {
                        status.connected = true;
                        status.pid = Some(pid);
                        if status.server_name.is_none() {
                            status.server_name = d.exit_server.clone();
                        }
                        if status.entry_server_name.is_none() {
                            status.entry_server_name = d.entry_server.clone();
                        }
                    } else {
                        store::clear_detached();
                    }
                }
            }
        }
        enrich_status(&mut status);
        let rates = if watch {
            Some(meter.sample(status.bytes_rx, status.bytes_tx))
        } else if status.connected {
            let first = read_traffic(&status);
            std::thread::sleep(Duration::from_millis(700));
            let second = read_traffic(&status);
            if second.0 > 0 || second.1 > 0 {
                status.bytes_rx = second.0;
                status.bytes_tx = second.1;
            }
            Some((
                second.0.saturating_sub(first.0) as f64 / 0.7,
                second.1.saturating_sub(first.1) as f64 / 0.7,
            ))
        } else {
            None
        };
        if json {
            println!("{}", serde_json::to_string(&status).unwrap_or_default());
        } else {
            print_status(&status, rates);
        }
        if !watch {
            return Ok(());
        }
        std::thread::sleep(Duration::from_secs(2));
    }
}

fn print_status(status: &ops::StatusSnapshot, rates: Option<(f64, f64)>) {
    if status.connected {
        println!("Состояние:    ПОДКЛЮЧЕНО");
        if let Some(name) = &status.server_name {
            println!("Сервер:       {}", name);
        }
        if let Some(entry) = &status.entry_server_name {
            println!("Точка входа:  {} (multi-hop)", entry);
        }
        if let Some(pid) = status.pid {
            println!("PID:          {}", pid);
        }
        println!(
            "Интерфейс:    {}",
            status
                .interface
                .clone()
                .unwrap_or_else(|| TUN_ALIAS.to_string())
        );
        println!(
            "Kill-switch:  {}",
            if status.kill_switch {
                "включён"
            } else {
                "выключен"
            }
        );
        println!(
            "Трафик:       вниз {}   вверх {}",
            human_bytes(status.bytes_rx),
            human_bytes(status.bytes_tx)
        );
        if let Some((rx_rate, tx_rate)) = rates {
            println!(
                "Скорость:     вниз {}   вверх {}",
                human_rate(rx_rate),
                human_rate(tx_rate)
            );
        }
    } else if status.always_on_locked {
        println!("Состояние:    ОТКЛЮЧЕНО (always-on lockdown)");
    } else {
        println!("Состояние:    ОТКЛЮЧЕНО");
    }
}

pub const TUN_ALIAS: &str = "wawity-tun0";

pub struct RateMeter {
    last: Option<(Instant, u64, u64)>,
}

impl RateMeter {
    pub fn new() -> Self {
        RateMeter { last: None }
    }

    pub fn sample(&mut self, rx: u64, tx: u64) -> (f64, f64) {
        let now = Instant::now();
        let result = match self.last {
            Some((prev_time, prev_rx, prev_tx)) => {
                let elapsed = now.duration_since(prev_time).as_secs_f64();
                if elapsed < 0.2 || rx < prev_rx || tx < prev_tx {
                    (0.0, 0.0)
                } else {
                    (
                        (rx - prev_rx) as f64 / elapsed,
                        (tx - prev_tx) as f64 / elapsed,
                    )
                }
            }
            None => (0.0, 0.0),
        };
        self.last = Some((now, rx, tx));
        result
    }
}

impl Default for RateMeter {
    fn default() -> Self {
        RateMeter::new()
    }
}

pub fn enrich_status(status: &mut ops::StatusSnapshot) {
    let detached = store::load_detached();
    if !status.connected {
        if let Some(d) = &detached {
            if let Some(pid) = d.pid {
                if is_pid_alive(pid) {
                    status.connected = true;
                    status.pid = Some(pid);
                    if status.server_name.is_none() {
                        status.server_name = d.exit_server.clone();
                    }
                    if status.entry_server_name.is_none() {
                        status.entry_server_name = d.entry_server.clone();
                    }
                }
            }
        }
    }
    if !status.connected {
        return;
    }
    if status.interface.is_none() {
        status.interface = Some(TUN_ALIAS.to_string());
    }
    if status.bytes_rx == 0 && status.bytes_tx == 0 {
        let alias = status
            .interface
            .clone()
            .unwrap_or_else(|| TUN_ALIAS.to_string());
        let (rx, tx) = wawity_core::network::read_interface_stats(&alias);
        status.bytes_rx = rx;
        status.bytes_tx = tx;
    }
    if !status.kill_switch {
        status.kill_switch = match &detached {
            Some(d) => d.kill_switch || d.always_on,
            None => store::load_settings().kill_switch,
        };
    }
    if !status.multihop {
        status.multihop = status.entry_server_name.is_some();
    }
}

pub fn read_traffic(status: &ops::StatusSnapshot) -> (u64, u64) {
    if !status.connected {
        return (0, 0);
    }
    let alias = status
        .interface
        .clone()
        .unwrap_or_else(|| TUN_ALIAS.to_string());
    wawity_core::network::read_interface_stats(&alias)
}

pub fn human_rate(v: f64) -> String {
    const UNITS: [&str; 5] = ["Б", "КБ", "МБ", "ГБ", "ТБ"];
    if v < 1.0 {
        return "0 Б/с".to_string();
    }
    let mut val = v;
    let mut idx = 0;
    while val >= 1024.0 && idx < UNITS.len() - 1 {
        val /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{:.0} {}/с", val, UNITS[idx])
    } else {
        format!("{:.1} {}/с", val, UNITS[idx])
    }
}

pub fn human_bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut val = n as f64;
    let mut idx = 0;
    while val >= 1024.0 && idx < UNITS.len() - 1 {
        val /= 1024.0;
        idx += 1;
    }
    format!("{:.1} {}", val, UNITS[idx])
}

pub fn cmd_servers(ping: bool, json: bool) -> Result<(), String> {
    let entries = collect_servers()?;
    let latencies: Vec<Option<u64>> = if ping {
        let targets: Vec<PingTarget> = entries
            .iter()
            .map(|e| PingTarget {
                host: e.server.server.clone(),
                port: 443,
            })
            .collect();
        ops::ping(targets).into_iter().map(|r| r.latency_ms).collect()
    } else {
        vec![None; entries.len()]
    };
    if json {
        let rows: Vec<_> = entries
            .iter()
            .zip(latencies.iter())
            .map(|(e, lat)| {
                serde_json::json!({
                    "name": e.server.name,
                    "protocol": e.server.protocol,
                    "host": e.server.server,
                    "subscription": e.sub_name,
                    "latency_ms": lat,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
    } else {
        for (e, lat) in entries.iter().zip(latencies.iter()) {
            let lat_str = match lat {
                Some(ms) => format!("{:>4}ms", ms),
                None => "   -  ".to_string(),
            };
            println!(
                "{}  [{}]  {:<28} {}",
                lat_str, e.server.protocol, e.server.name, e.sub_name
            );
        }
    }
    Ok(())
}

pub fn cmd_repair(json: bool) -> Result<(), String> {
    let session = Session::new()?;
    wawity_core::engine::repair_network_sync(session.handles());
    if json {
        println!("{{\"repaired\":true}}");
    } else {
        println!("Network repair completed.");
    }
    Ok(())
}

pub fn cmd_sub_add(url: String, name: Option<String>, json: bool) -> Result<(), String> {
    let servers = ops::fetch_subscription_raw(&url)?;
    let mut subs = store::load_subscriptions();
    let display = name.unwrap_or_else(|| format!("Subscription {}", subs.len() + 1));
    let sub = Subscription {
        id: store::new_id(),
        name: display.clone(),
        url,
    };
    subs.push(sub.clone());
    store::save_subscriptions(&subs)?;
    if json {
        println!(
            "{}",
            serde_json::json!({ "id": sub.id, "name": sub.name, "servers": servers.len() })
        );
    } else {
        println!("Added '{}' ({} servers). id={}", display, servers.len(), sub.id);
    }
    Ok(())
}

pub fn cmd_sub_list(json: bool) -> Result<(), String> {
    let subs = store::load_subscriptions();
    if json {
        println!("{}", serde_json::to_string_pretty(&subs).unwrap_or_default());
    } else if subs.is_empty() {
        println!("No subscriptions.");
    } else {
        for s in &subs {
            println!("{}  {}", s.id, s.name);
        }
    }
    Ok(())
}

pub fn cmd_sub_refresh(id: Option<String>, json: bool) -> Result<(), String> {
    let subs = store::load_subscriptions();
    let mut report = Vec::new();
    for s in &subs {
        if let Some(filter) = &id {
            if &s.id != filter {
                continue;
            }
        }
        match ops::fetch_subscription_raw(&s.url) {
            Ok(servers) => report.push((s.name.clone(), Ok(servers.len()))),
            Err(e) => report.push((s.name.clone(), Err(e))),
        }
    }
    if json {
        let rows: Vec<_> = report
            .iter()
            .map(|(name, res)| match res {
                Ok(n) => serde_json::json!({ "name": name, "servers": n }),
                Err(e) => serde_json::json!({ "name": name, "error": e }),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&rows).unwrap_or_default());
    } else {
        for (name, res) in &report {
            match res {
                Ok(n) => println!("{}: {} servers", name, n),
                Err(e) => println!("{}: error {}", name, e),
            }
        }
    }
    Ok(())
}

pub fn cmd_sub_rm(id: String, json: bool) -> Result<(), String> {
    let mut subs = store::load_subscriptions();
    let before = subs.len();
    subs.retain(|s| s.id != id);
    store::save_subscriptions(&subs)?;
    let removed = before - subs.len();
    if json {
        println!("{{\"removed\":{}}}", removed);
    } else if removed > 0 {
        println!("Removed {} subscription(s).", removed);
    } else {
        println!("No subscription with id {}.", id);
    }
    Ok(())
}
