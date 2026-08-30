#![allow(dead_code)]

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;

use crate::engine::{
    fetch_with_fallback, fetch_with_ua, geolocate_servers_blocking, make_client_with_ua,
    start_session, stop_session, switch_session, proxies_from,
    raw_proxies_from, resolve_own_exe_path, AppState, ConnectionHandles, ParsedServer,
    PingResult, PingTarget, PrivacyOptions, SubscriptionInfo, CLIENT_USER_AGENTS,
};
use crate::network::{RoutingManager, TunManager};
use crate::util;

#[derive(Debug, Clone, Serialize)]
pub struct StatusSnapshot {
    pub connected: bool,
    pub pid: Option<u32>,
    pub kill_switch: bool,
    pub always_on_locked: bool,
    pub multihop: bool,
    pub interface: Option<String>,
    pub server_name: Option<String>,
    pub entry_server_name: Option<String>,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
}

pub struct Session {
    state: AppState,
    handles: ConnectionHandles,
}

impl Session {
    pub fn new() -> Result<Self, String> {
        let state = AppState::new().map_err(|e| e.to_string())?;
        let handles = ConnectionHandles::from_state(&state);
        Ok(Self { state, handles })
    }

    pub fn handles(&self) -> &ConnectionHandles {
        &self.handles
    }

    pub fn set_privacy(&self, privacy: PrivacyOptions) {
        *self.handles.privacy.lock().unwrap() = privacy;
    }

    pub fn set_always_on_flag(&self, enabled: bool) {
        *self.handles.always_on.lock().unwrap() = enabled;
    }

    pub fn is_running(&self) -> bool {
        self.handles.process_manager.lock().unwrap().is_running()
    }

    pub fn connect(
        &self,
        exit_url: &str,
        entry_url: Option<String>,
        server_name: Option<String>,
        entry_server_name: Option<String>,
        kill_switch: bool,
        bypass_apps: Vec<String>,
        quantum_resistant: bool,
    ) -> Result<(), String> {
        start_session(
            exit_url,
            entry_url,
            server_name,
            entry_server_name,
            kill_switch,
            bypass_apps,
            quantum_resistant,
            &self.handles,
        )
    }

    pub fn switch(
        &self,
        exit_url: &str,
        entry_url: Option<String>,
        server_name: Option<String>,
        entry_server_name: Option<String>,
        bypass_apps: Vec<String>,
        quantum_resistant: bool,
    ) -> Result<(), String> {
        switch_session(
            exit_url,
            entry_url,
            server_name,
            entry_server_name,
            bypass_apps,
            quantum_resistant,
            &self.handles,
        )
    }

    pub fn disconnect(&self) -> Result<(), String> {
        stop_session(&self.handles)
    }

    pub fn status(&self) -> StatusSnapshot {
        let (connected, pid, kill_switch, interface_name, always_on_locked) = {
            let pm = self.handles.process_manager.lock().unwrap();
            let routing = self.handles.routing_manager.lock().unwrap();
            let tun = self.handles.tun_manager.lock().unwrap();
            let connected = pm.is_running();
            let pid = if connected { pm.get_pid() } else { None };
            let kill_switch = connected && routing.is_kill_switch_active();
            let interface_name = tun.get_interface_name().to_string();
            let always_on_locked = !connected && routing.is_always_on_active();
            (connected, pid, kill_switch, interface_name, always_on_locked)
        };
        let server_name = self.handles.connected_server_name.lock().unwrap().clone();
        let entry_server_name = self.handles.connected_entry_server_name.lock().unwrap().clone();
        let multihop = connected && entry_server_name.is_some();
        let (bytes_rx, bytes_tx) = if connected {
            crate::network::read_interface_stats(&interface_name)
        } else {
            (0, 0)
        };
        StatusSnapshot {
            connected,
            pid,
            kill_switch,
            always_on_locked,
            multihop,
            interface: if connected { Some(interface_name) } else { None },
            server_name,
            entry_server_name: if connected { entry_server_name } else { None },
            bytes_rx,
            bytes_tx,
        }
    }

    pub fn set_always_on(&self, enabled: bool) -> Result<(), String> {
        let connected = self.handles.process_manager.lock().unwrap().is_running();
        if enabled {
            let app_exe_path = resolve_own_exe_path()?;
            if connected {
                let already_locked =
                    self.handles.routing_manager.lock().unwrap().is_kill_switch_active();
                if !already_locked {
                    let binary_path =
                        self.handles.process_manager.lock().unwrap().binary_path_string();
                    let bypass = self.handles.bypass_apps.lock().unwrap().clone();
                    let mut routing = self.handles.routing_manager.lock().unwrap();
                    routing
                        .stage_exceptions(&binary_path, &app_exe_path)
                        .map_err(|e| e.to_string())?;
                    if !bypass.is_empty() {
                        let include_mode = crate::config::SplitMode::parse(
                            &self.handles.split_rules.lock().unwrap().mode,
                        ).is_include();
                        if let Err(e) = routing.update_bypass_rules(&bypass, include_mode) {
                            routing.abort_staged_connection(false);
                            return Err(e.to_string());
                        }
                    }
                    if let Err(e) = routing.commit_connection() {
                        routing.abort_staged_connection(false);
                        return Err(e.to_string());
                    }
                }
            } else {
                self.handles
                    .routing_manager
                    .lock()
                    .unwrap()
                    .enable_always_on(&app_exe_path)
                    .map_err(|e| e.to_string())?;
            }
            *self.handles.always_on.lock().unwrap() = true;
        } else {
            *self.handles.always_on.lock().unwrap() = false;
            if !connected {
                self.handles
                    .routing_manager
                    .lock()
                    .unwrap()
                    .disable_always_on()
                    .map_err(|e| e.to_string())?;
            }
        }
        self.state.persist_always_on_flag(enabled);
        Ok(())
    }

    pub fn detach_process(&self) -> Option<u32> {
        self.handles.process_manager.lock().unwrap().detach()
    }

    pub fn recent_output(&self, lines: usize) -> String {
        self.handles.process_manager.lock().unwrap().recent_output(lines)
    }
}

pub fn fetch_subscription(url: &str) -> Result<Vec<ParsedServer>, String> {
    let fetched = fetch_with_fallback(url)?;
    let proxies = proxies_from(&fetched.body, &fetched.content_type);
    if proxies.is_empty() {
        return Err(format!(
            "No valid proxy servers found. Content-Type: '{}'. First 200 chars: {}",
            fetched.content_type,
            fetched.body.chars().take(200).collect::<String>()
        ));
    }
    Ok(proxies)
}

pub fn fetch_subscription_raw(url: &str) -> Result<Vec<ParsedServer>, String> {
    use std::collections::HashSet;
    let mut best: Vec<ParsedServer> = Vec::new();
    let mut best_score = (0usize, 0usize, 0usize);
    let mut last_err = String::from("Unknown error");
    let mut last_meta = String::new();
    for ua in CLIENT_USER_AGENTS {
        let fetched = match fetch_with_ua(url, ua) {
            Ok(f) => f,
            Err(e) => {
                last_err = e;
                continue;
            }
        };
        last_meta = format!(
            "Content-Type: '{}'. First 200 chars: {}",
            fetched.content_type,
            fetched.body.chars().take(200).collect::<String>()
        );
        let mut proxies = raw_proxies_from(&fetched.body);
        let connectable = if proxies.is_empty() {
            proxies = proxies_from(&fetched.body, &fetched.content_type);
            0usize
        } else {
            1usize
        };
        if proxies.is_empty() {
            continue;
        }
        let protos: HashSet<String> = proxies.iter().map(|p| p.protocol.clone()).collect();
        let score = (connectable, protos.len(), proxies.len());
        if score > best_score {
            best_score = score;
            best = proxies;
        }
        if best_score.0 == 1 && best_score.1 >= 2 {
            break;
        }
    }
    if best.is_empty() {
        return Err(format!("No valid proxy servers found. {} ({})", last_meta, last_err));
    }
    Ok(best)
}

pub fn geolocate(hosts: Vec<String>) -> Result<Vec<Option<String>>, String> {
    geolocate_servers_blocking(hosts)
}

const PING_WORKERS: usize = 24;
const PING_CONNECT_TIMEOUT: Duration = Duration::from_secs(3);
const PING_ECHO_TIMEOUT: Duration = Duration::from_millis(2500);

fn round_up_millis(elapsed: Duration) -> u64 {
    ((elapsed.as_micros() as u64 + 999) / 1000).max(1)
}

fn endpoint_rtt(host: &str, port: u16) -> Option<u64> {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Instant;

    let addr = format!("{}:{}", host, port).to_socket_addrs().ok()?.next()?;
    let dialing = Instant::now();
    let mut stream = TcpStream::connect_timeout(&addr, PING_CONNECT_TIMEOUT).ok()?;
    let handshake = dialing.elapsed();
    if stream.set_nodelay(true).is_err()
        || stream.set_read_timeout(Some(PING_ECHO_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(PING_ECHO_TIMEOUT)).is_err()
    {
        return Some(round_up_millis(handshake));
    }

    let probe = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nUser-Agent: Mozilla/5.0\r\nConnection: close\r\n\r\n",
        host
    );
    let sent = Instant::now();
    if stream.write_all(probe.as_bytes()).is_err() {
        return Some(round_up_millis(handshake));
    }
    let mut sink = [0u8; 1];
    match stream.read(&mut sink) {
        Ok(_) => Some(round_up_millis(sent.elapsed())),
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut
            || e.kind() == std::io::ErrorKind::WouldBlock =>
        {
            Some(round_up_millis(handshake))
        }
        Err(_) => Some(round_up_millis(sent.elapsed())),
    }
}

pub fn ping(targets: Vec<PingTarget>) -> Vec<PingResult> {
    if targets.is_empty() {
        return Vec::new();
    }
    let workers = PING_WORKERS.min(targets.len());
    let batch_size = (targets.len() + workers - 1) / workers;
    let mut crew = Vec::with_capacity(workers);
    for slice in targets.chunks(batch_size) {
        let batch: Vec<(String, u16)> = slice.iter().map(|t| (t.host.clone(), t.port)).collect();
        crew.push(std::thread::spawn(move || {
            batch
                .into_iter()
                .map(|(host, port)| PingResult {
                    latency_ms: endpoint_rtt(&host, port),
                    host,
                    port,
                })
                .collect::<Vec<PingResult>>()
        }));
    }
    let mut measured = Vec::with_capacity(targets.len());
    for worker in crew {
        if let Ok(part) = worker.join() {
            measured.extend(part);
        }
    }
    measured
}

pub fn subscription_info(url: &str) -> Result<SubscriptionInfo, String> {
    let client = make_client_with_ua(15, CLIENT_USER_AGENTS[0])?;
    let resp = client
        .get(url)
        .header("Accept", "*/*")
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    let header = resp
        .headers()
        .get("subscription-userinfo")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let mut info = SubscriptionInfo {
        expire: None,
        total: None,
        upload: None,
        download: None,
    };
    if let Some(header) = header {
        for part in header.split(';') {
            let part = part.trim();
            let mut kv = part.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim();
            let val = kv.next().unwrap_or("").trim();
            match key {
                "upload" => info.upload = val.parse().ok(),
                "download" => info.download = val.parse().ok(),
                "total" => info.total = val.parse().ok(),
                "expire" => info.expire = val.parse().ok(),
                _ => {}
            }
        }
    }
    Ok(info)
}

pub fn force_teardown_detached(pid: Option<u32>, was_always_on: bool) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(pid) = pid {
        let output = util::silent_command("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .output();
        if let Err(e) = output {
            errors.push(format!("taskkill: {}", e));
        }
    }
    let mut routing = RoutingManager::new();
    routing.force_cleanup();
    let tun = TunManager::new("wawity-tun0");
    if let Err(e) = tun.teardown_interface() {
        errors.push(format!("tun teardown: {}", e));
    }
    if was_always_on {
        if let Ok(app_exe_path) = resolve_own_exe_path() {
            let mut routing = RoutingManager::new();
            if let Err(e) = routing.enable_always_on(&app_exe_path) {
                errors.push(format!("reengage always-on: {}", e));
            }
        }
    }
    errors
}

pub fn register_autostart() -> Result<(), String> {
    let script = crate::engine::autostart_register_script()?;
    util::run_ps_script(&script, Duration::from_secs(20)).map(|_| ())
}

pub fn _arc_touch(_: Arc<()>) {}
