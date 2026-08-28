#![allow(unused_imports)]

pub use wawity_core::engine::*;
use crate::config::{parse_all_from_subscription, parse_subscription, ConfigGenerator};
use wawity_core::engine::SelectorOptions;
use crate::error::VpnError;
use crate::network::{RoutingManager, TunManager};
use crate::process::ProcessManager;
use crate::util::strip_unc_prefix;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
use reqwest::blocking::Client;
use base64::{Engine as _, engine::general_purpose};

#[tauri::command]
pub async fn fetch_subscription(url: String) -> Result<Vec<ParsedServer>, String> {
    tokio::task::spawn_blocking(move || {
        let fetched = fetch_with_fallback(&url)?;
        let proxies = proxies_from(&fetched.body, &fetched.content_type);
        if proxies.is_empty() {
            return Err(format!(
                "No valid proxy servers found. Content-Type: '{}'. First 200 chars: {}",
                fetched.content_type,
                &fetched.body.chars().take(200).collect::<String>()
            ));
        }
        Ok(proxies)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn fetch_rule_list(url: String) -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(move || {
        let fetched = fetch_with_fallback(&url)?;
        let mut out: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for line in fetched.body.lines() {
            let mut entry = line.trim();
            if let Some(idx) = entry.find('#') {
                entry = entry[..idx].trim();
            }
            if entry.is_empty() || entry.starts_with('!') {
                continue;
            }
            let cleaned = entry
                .trim_start_matches("||")
                .trim_start_matches("*.")
                .trim_start_matches('.')
                .trim_end_matches('^')
                .trim()
                .to_ascii_lowercase();
            if cleaned.is_empty() {
                continue;
            }
            if seen.insert(cleaned.clone()) {
                out.push(cleaned);
            }
        }
        if out.is_empty() {
            return Err("No usable entries in the provided list".to_string());
        }
        Ok(out)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn fetch_subscription_raw(url: String) -> Result<Vec<ParsedServer>, String> {
    tokio::task::spawn_blocking(move || {
        use std::collections::HashSet;
        let mut best: Vec<ParsedServer> = Vec::new();
        let mut best_score = (0usize, 0usize, 0usize);
        let mut last_err = String::from("Unknown error");
        let mut last_meta = String::new();
        for ua in CLIENT_USER_AGENTS {
            let fetched = match fetch_with_ua(&url, ua) {
                Ok(f) => f,
                Err(e) => { last_err = e; continue; }
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
            if proxies.is_empty() { continue; }
            let protos: HashSet<String> = proxies.iter().map(|p| p.protocol.clone()).collect();
            let score = (connectable, protos.len(), proxies.len());
            if score > best_score {
                best_score = score;
                best = proxies;
            }
            if best_score.0 == 1 && best_score.1 >= 2 { break; }
        }
        if best.is_empty() {
            return Err(format!("No valid proxy servers found. {} ({})", last_meta, last_err));
        }
        Ok(best)
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn geolocate_servers(hosts: Vec<String>) -> Result<Vec<Option<String>>, String> {
    tokio::task::spawn_blocking(move || geolocate_servers_blocking(hosts))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn ping_servers(targets: Vec<PingTarget>) -> Result<Vec<PingResult>, String> {
    tokio::task::spawn_blocking(move || wawity_core::ops::ping(targets))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_processes() -> Result<Vec<ProcessEntry>, String> {
    tokio::task::spawn_blocking(move || {
        #[cfg(target_os = "windows")]
        {
            let raw = crate::util::enumerate_processes_with_paths()?;
            let mut seen = std::collections::HashSet::new();
            let mut result: Vec<ProcessEntry> = raw
                .into_iter()
                .filter_map(|(pid, process_name, path)| {
                    let normalized = crate::util::normalize_windows_path(&path);
                    if normalized.is_empty() || !seen.insert(normalized.to_lowercase()) { return None; }
                    let name = std::path::Path::new(&normalized)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&process_name)
                        .to_string();
                    Some(ProcessEntry { pid, name, path: normalized })
                })
                .collect();
            result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            Ok(result)
        }
        #[cfg(not(target_os = "windows"))]
        Ok(vec![])
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn measure_tunnel_latency(state: State<'_, AppState>) -> Result<Option<u64>, String> {
    let connected = match state.process_manager.try_lock() {
        Ok(pm) => pm.is_running(),
        Err(_) => return Ok(None),
    };
    if !connected { return Ok(None); }
    tokio::task::spawn_blocking(socks_latency)
        .await
        .map_err(|e| e.to_string())
}

enum NetOp<T> {
    Done(Result<T, String>),
    TimedOut,
}

async fn run_net_op<T, F>(seconds: u64, label: &str, job: F) -> NetOp<T>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    match tokio::time::timeout(
        Duration::from_secs(seconds),
        tokio::task::spawn_blocking(job),
    )
    .await
    {
        Ok(Ok(outcome)) => NetOp::Done(outcome),
        Ok(Err(join)) => NetOp::Done(Err(format!("{} task crashed: {}", label, join))),
        Err(_) => NetOp::TimedOut,
    }
}

fn force_reset(handles: ConnectionHandles, app: AppHandle, stop_process: bool) {
    thread::spawn(move || {
        if stop_process {
            let _ = handles.process_manager.lock().unwrap().stop();
        }
        handles.routing_manager.lock().unwrap().force_cleanup();
        let _ = handles.tun_manager.lock().unwrap().teardown_interface();
        *handles.current_config.lock().unwrap() = None;
        *handles.connected_server_name.lock().unwrap() = None;
        *handles.connected_entry_server_name.lock().unwrap() = None;
        *handles.current_exit_url.lock().unwrap() = None;
        *handles.current_entry_url.lock().unwrap() = None;
        restore_always_on(&handles);
        crate::refresh_tray(&app);
    });
}

#[tauri::command]
pub async fn connect_vpn(
    sub_url: String,
    entry_sub_url: Option<String>,
    server_name: Option<String>,
    entry_server_name: Option<String>,
    kill_switch: bool,
    bypass_apps: Vec<String>,
    quantum_resistant: bool,
    privacy: Option<PrivacyOptions>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let handles = ConnectionHandles::from_state(&state);
    if let Some(p) = privacy {
        *handles.privacy.lock().unwrap() = p;
    }
    let result = match run_net_op(40, "Connect", move || start_session(
        &sub_url, entry_sub_url, server_name, entry_server_name,
        kill_switch, bypass_apps, quantum_resistant, &handles,
    ))
    .await
    {
        NetOp::Done(outcome) => outcome,
        NetOp::TimedOut => {
            force_reset(ConnectionHandles::from_state(&state), app.clone(), true);
            Err("Connection timed out after 40s, network state force-reset for safety".into())
        }
    };
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn disconnect_vpn(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let handles = ConnectionHandles::from_state(&state);
    let result = match run_net_op(30, "Disconnect", move || stop_session(&handles)).await {
        NetOp::Done(outcome) => outcome,
        NetOp::TimedOut => {
            force_reset(ConnectionHandles::from_state(&state), app.clone(), false);
            Err("Disconnect timed out after 30s, firewall force-reset for safety".into())
        }
    };
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn switch_vpn_server(
    sub_url: String,
    entry_sub_url: Option<String>,
    server_name: Option<String>,
    entry_server_name: Option<String>,
    bypass_apps: Vec<String>,
    quantum_resistant: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let handles = ConnectionHandles::from_state(&state);
    let result = match run_net_op(60, "Switch", move || switch_session(
        &sub_url, entry_sub_url, server_name, entry_server_name,
        bypass_apps, quantum_resistant, &handles,
    ))
    .await
    {
        NetOp::Done(outcome) => outcome,
        NetOp::TimedOut => {
            force_reset(ConnectionHandles::from_state(&state), app.clone(), true);
            crate::util::net_debug_log("switch: 60s timeout hit, background force-reset spawned");
            Err("Server switch timed out after 60s, network state force-reset for safety".into())
        }
    };
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn set_always_on(enabled: bool, state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let handles = ConnectionHandles::from_state(&state);
    let outcome = tokio::time::timeout(
        Duration::from_secs(25),
        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let connected = handles.process_manager.lock().unwrap().is_running();
            if enabled {
                let app_exe_path = resolve_own_exe_path()?;
                if connected {
                    let already_locked = handles.routing_manager.lock().unwrap().is_kill_switch_active();
                    if !already_locked {
                        let binary_path = handles.process_manager.lock().unwrap().binary_path_string();
                        let bypass = handles.bypass_apps.lock().unwrap().clone();
                        let mut routing = handles.routing_manager.lock().unwrap();
                        routing.stage_exceptions(&binary_path, &app_exe_path)
                            .map_err(|e| e.to_string())?;
                        if !bypass.is_empty() {
                            if let Err(e) = routing.update_bypass_rules(&bypass, crate::config::SplitMode::parse(&handles.split_rules.lock().unwrap().mode).is_include()) {
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
                    handles.routing_manager.lock().unwrap()
                        .enable_always_on(&app_exe_path)
                        .map_err(|e| e.to_string())?;
                }
                *handles.always_on.lock().unwrap() = true;
                Ok(())
            } else {
                *handles.always_on.lock().unwrap() = false;
                if !connected {
                    handles.routing_manager.lock().unwrap()
                        .disable_always_on()
                        .map_err(|e| e.to_string())?;
                }
                Ok(())
            }
        }),
    ).await;
    let result = match outcome {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => Err(format!("Always-on task crashed: {}", e)),
        Err(_) => Err("Always-on toggle timed out after 25s".into()),
    };
    if result.is_ok() {
        state.persist_always_on_flag(enabled);
    }
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn get_vpn_status(state: State<'_, AppState>) -> Result<VpnStatus, String> {
    let process_manager = Arc::clone(&state.process_manager);
    let routing_manager = Arc::clone(&state.routing_manager);
    let tun_manager = Arc::clone(&state.tun_manager);
    let server_name_handle = Arc::clone(&state.connected_server_name);
    let entry_name_handle = Arc::clone(&state.connected_entry_server_name);
    tokio::task::spawn_blocking(move || {
        let (connected, pid, kill_switch, interface_name, always_on_locked) = {
            let pm = process_manager.lock().unwrap();
            let routing = routing_manager.lock().unwrap();
            let tun = tun_manager.lock().unwrap();
            let connected = pm.is_running();
            let pid = if connected { pm.get_pid() } else { None };
            let kill_switch = connected && routing.is_kill_switch_active();
            let interface_name = tun.get_interface_name().to_string();
            let always_on_locked = !connected && routing.is_always_on_active();
            (connected, pid, kill_switch, interface_name, always_on_locked)
        };
        let server_name = server_name_handle.lock().unwrap().clone();
        let entry_server_name = entry_name_handle.lock().unwrap().clone();
        let interface = if connected { Some(interface_name.clone()) } else { None };
        let (bytes_rx, bytes_tx) = if connected {
            crate::network::read_interface_stats(&interface_name)
        } else {
            (0, 0)
        };
        VpnStatus {
            connected, pid, server: None, kill_switch,
            interface, server_name,
            entry_server_name: if connected { entry_server_name } else { None },
            multihop: connected && entry_name_handle.lock().unwrap().is_some(),
            bytes_rx, bytes_tx,
            speed_rx: 0.0, speed_tx: 0.0,
            always_on_locked,
        }
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_subscription(sub_url: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || parse_subscription(&sub_url).is_ok())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_logs() -> Result<String, String> {
    tokio::task::spawn_blocking(|| {
        let log_path = std::env::temp_dir().join("wawity.log");
        if log_path.exists() {
            fs::read_to_string(&log_path).map_err(|e| e.to_string())
        } else {
            Ok(String::new())
        }
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn get_saved_servers(state: State<'_, AppState>) -> Result<Vec<SavedServer>, String> {
    Ok(state.saved_servers.lock().unwrap().clone())
}

#[tauri::command]
pub fn add_saved_server(id: String, name: String, url: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut servers = state.saved_servers.lock().unwrap();
    if servers.iter().any(|s| s.id == id) {
        return Err("Duplicate ID".into());
    }
    servers.push(SavedServer { id, name, url });
    drop(servers);
    state.save_servers_to_disk().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_saved_server(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut servers = state.saved_servers.lock().unwrap();
    servers.retain(|s| s.id != id);
    drop(servers);
    state.save_servers_to_disk().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_split_rules(
    mode: String,
    processes: Vec<String>,
    domains: Vec<String>,
    ips: Vec<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let incoming = SplitRules { mode, processes, domains, ips };
    {
        let mut slot = state.split_rules.lock().unwrap();
        if *slot == incoming {
            return Ok(());
        }
        *slot = incoming;
    }
    let handles = ConnectionHandles::from_state(&state);
    if !handles.process_manager.lock().unwrap().is_running() {
        return Ok(());
    }
    let paths = handles.bypass_apps.lock().unwrap().clone();
    let result = match run_net_op(35, "Split rules", move || reload_bypass_apps(paths, &handles)).await {
        NetOp::Done(r) => r,
        NetOp::TimedOut => Err("Split rules update timed out after 35s".into()),
    };
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn detect_blocked_services() -> Result<Vec<wawity_core::blockcheck::BlockReport>, String> {
    tokio::task::spawn_blocking(wawity_core::blockcheck::probe_defaults)
        .await
        .map_err(|e| format!("Block detection worker failed: {}", e))
}

#[tauri::command]
pub async fn update_bypass_apps(paths: Vec<String>, state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let handles = ConnectionHandles::from_state(&state);
    let outcome = tokio::time::timeout(
        Duration::from_secs(35),
        tokio::task::spawn_blocking(move || reload_bypass_apps(paths, &handles)),
    ).await;
    let result = match outcome {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => Err(format!("Split tunneling task crashed: {}", e)),
        Err(_) => Err("Split tunneling update timed out after 35s".into()),
    };
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn repair_network(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let handles = ConnectionHandles::from_state(&state);
    let outcome = tokio::time::timeout(
        Duration::from_secs(30),
        tokio::task::spawn_blocking(move || repair_network_sync(&handles)),
    ).await;
    let result = match outcome {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(format!("Repair task crashed: {}", e)),
        Err(_) => Err("Repair timed out after 30s".into()),
    };
    crate::refresh_tray(&app);
    result
}

#[tauri::command]
pub async fn sync_tray_state(
    servers: Vec<TrayServerInput>,
    selected_id: Option<String>,
    kill_switch: bool,
    quantum_resistant: bool,
    bypass_apps: Vec<String>,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let tray_servers = Arc::clone(&state.tray_servers);
    let tray_selected_id = Arc::clone(&state.tray_selected_id);
    let default_kill_switch = Arc::clone(&state.default_kill_switch);
    let default_quantum_resistant = Arc::clone(&state.default_quantum_resistant);
    let process_manager = Arc::clone(&state.process_manager);
    let bypass_handle = Arc::clone(&state.bypass_apps);
    tokio::task::spawn_blocking(move || {
        *tray_servers.lock().unwrap() = servers
            .into_iter()
            .map(|s| TrayServerEntry {
                id: s.id,
                name: s.name,
                url: s.url,
                country_code: s.country_code,
            })
            .collect();
        *tray_selected_id.lock().unwrap() = selected_id;
        *default_kill_switch.lock().unwrap() = kill_switch;
        *default_quantum_resistant.lock().unwrap() = quantum_resistant;
        if let Ok(pm) = process_manager.try_lock() {
            if !pm.is_running() {
                *bypass_handle.lock().unwrap() = bypass_apps;
            }
        }
        crate::refresh_tray(&app);
    })
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_subscription_info(url: String) -> Result<SubscriptionInfo, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("empty subscription url".into());
    }
    let resp = GLOBAL_CLIENT
        .get(trimmed)
        .headers(extra_header_map())
        .send()
        .map_err(|e| format!("subscription request failed: {}", e))?;
    let header_raw = resp
        .headers()
        .iter()
        .find(|(name, _)| name.as_str().eq_ignore_ascii_case("subscription-userinfo"))
        .and_then(|(_, value)| value.to_str().ok())
        .map(|s| s.to_string());
    let mut info = SubscriptionInfo {
        expire: None,
        total: None,
        upload: None,
        download: None,
    };
    if let Some(raw) = header_raw {
        for part in raw.split(';') {
            let part = part.trim();
            let Some((key, value)) = part.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim();
            match key.as_str() {
                "expire" => info.expire = value.parse::<i64>().ok(),
                "total" => info.total = value.parse::<u64>().ok(),
                "upload" => info.upload = value.parse::<u64>().ok(),
                "download" => info.download = value.parse::<u64>().ok(),
                _ => {}
            }
        }
    }
    Ok(info)
}

#[tauri::command]
pub async fn set_start_on_boot(enabled: bool) -> Result<(), String> {
    tokio::task::spawn_blocking(move || -> Result<(), String> {
        let script = if enabled {
            autostart_register_script()?
        } else {
            "Unregister-ScheduledTask -TaskName 'WawityAutostart' -Confirm:$false -ErrorAction SilentlyContinue | Out-Null; Remove-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'Wawity' -ErrorAction SilentlyContinue | Out-Null".to_string()
        };
        let output = crate::util::run_ps_script(&script, Duration::from_secs(20))
            .map_err(|e| format!("task scheduler command failed: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(format!("failed to update autostart task: {}", stderr));
        }
        crate::util::net_debug_log(if enabled { "autostart: task registered" } else { "autostart: task removed" });
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_start_on_boot() -> Result<bool, String> {
    tokio::task::spawn_blocking(|| -> Result<bool, String> {
        let script = "$task = Get-ScheduledTask -TaskName 'WawityAutostart' -ErrorAction SilentlyContinue; $legacy = Get-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'Wawity' -ErrorAction SilentlyContinue; if ($task) { Write-Output 'yes' } elseif ($legacy) { Write-Output 'legacy' } else { Write-Output 'no' }";
        let output = crate::util::run_ps_script(script, Duration::from_secs(10))
            .map_err(|e| format!("task query failed: {}", e))?;
        if !output.status.success() {
            return Ok(false);
        }
        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.eq_ignore_ascii_case("legacy") {
            if let Ok(script) = autostart_register_script() {
                let migrated = crate::util::run_ps_script(&script, Duration::from_secs(20))
                    .map(|o| o.status.success())
                    .unwrap_or(false);
                crate::util::net_debug_log(if migrated { "autostart: legacy run key migrated to task" } else { "autostart: legacy migration failed" });
            }
            return Ok(true);
        }
        Ok(text.eq_ignore_ascii_case("yes"))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn is_first_launch() -> bool {
    first_launch_marker().map(|marker| !marker.exists()).unwrap_or(false)
}

#[tauri::command]
pub fn finish_first_launch() -> Result<(), String> {
    let marker = first_launch_marker()?;
    if let Some(parent) = marker.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("cannot prepare flag directory: {}", e))?;
    }
    std::fs::write(&marker, b"ok").map_err(|e| format!("cannot write first launch flag: {}", e))
}

static SPEED_HALT: Lazy<Arc<std::sync::atomic::AtomicBool>> =
    Lazy::new(|| Arc::new(std::sync::atomic::AtomicBool::new(false)));
static SPEED_BUSY: Lazy<Arc<std::sync::atomic::AtomicBool>> =
    Lazy::new(|| Arc::new(std::sync::atomic::AtomicBool::new(false)));

#[tauri::command]
pub async fn run_speed_test(app: AppHandle) -> Result<wawity_core::netprobe::SpeedResult, String> {
    use std::sync::atomic::Ordering;

    if SPEED_BUSY.swap(true, Ordering::SeqCst) {
        return Err("speed test is already running".to_string());
    }
    SPEED_HALT.store(false, Ordering::SeqCst);

    let halt = Arc::clone(&SPEED_HALT);
    let outcome = tokio::task::spawn_blocking(move || {
        wawity_core::netprobe::run_speed_test(halt, |tick| {
            let _ = app.emit_all("wawity-speed-tick", tick);
        })
    })
    .await;

    SPEED_BUSY.store(false, Ordering::SeqCst);
    SPEED_HALT.store(false, Ordering::SeqCst);

    outcome.map_err(|e| format!("Speed test worker failed: {}", e))
}

#[tauri::command]
pub async fn cancel_speed_test() -> Result<(), String> {
    SPEED_HALT.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub async fn audit_leaks() -> Result<wawity_core::netprobe::LeakAudit, String> {
    tokio::task::spawn_blocking(wawity_core::netprobe::audit_leaks)
        .await
        .map_err(|e| format!("Leak audit worker failed: {}", e))
}

#[tauri::command]
pub async fn probe_reachability(
    targets: Option<Vec<(String, String, String)>>,
) -> Result<Vec<wawity_core::blockcheck::BlockReport>, String> {
    tokio::task::spawn_blocking(move || match targets {
        Some(list) if !list.is_empty() => wawity_core::blockcheck::probe_tagged(list),
        _ => wawity_core::blockcheck::probe_defaults(),
    })
    .await
    .map_err(|e| format!("Reachability worker failed: {}", e))
}

#[tauri::command]
pub async fn probe_servers_deep(
    targets: Vec<wawity_core::smartpick::DeepTarget>,
) -> Result<Vec<wawity_core::smartpick::DeepSample>, String> {
    tokio::task::spawn_blocking(move || wawity_core::smartpick::deep_probe(targets))
        .await
        .map_err(|e| format!("Deep probe worker failed: {}", e))
}
