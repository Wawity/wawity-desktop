#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dependency_on_unit_never_type_fallback)]
mod appicons;
mod appstats;
mod appwatch;
mod commands;
mod coreinfo;
mod firewall;
mod migrate;
pub use wawity_core::{config, constants, error, network, process, util};
mod games;
mod hotkeys;
mod hwid;
mod installed;
mod presence;
mod streamdetect;
mod telemetry;

use commands::{
    add_saved_server, connect_vpn, disconnect_vpn,
    fetch_rule_list, fetch_subscription, fetch_subscription_raw,
    get_logs, get_saved_servers, get_start_on_boot, get_subscription_info, get_vpn_status,
    geolocate_servers, list_processes, measure_tunnel_latency, start_session, stop_session,
    switch_session, ping_servers, remove_saved_server, repair_network, resolve_own_exe_path,
    detect_blocked_services, set_always_on, set_split_rules, set_start_on_boot, switch_vpn_server,
    sync_tray_state,
    update_bypass_apps,
    validate_subscription, AppState, ConnectionHandles,
};
use games::scan_installed_games;
use installed::list_installed_apps;
use util::silent_command;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Manager, RunEvent, SystemTray, SystemTrayEvent, WindowBuilder,
    WindowEvent, WindowUrl,
};
static TRAY_RU: AtomicBool = AtomicBool::new(false);
static LAST_TOOLTIP: Mutex<String> = Mutex::new(String::new());
fn tray_ru() -> bool {
    TRAY_RU.load(Ordering::Relaxed)
}
#[tauri::command]
fn sync_hotkeys(combo: Option<String>, panic_combo: Option<String>, app: AppHandle) -> Result<(), String> {
    hotkeys::apply(&app, combo, panic_combo)
}
#[tauri::command]
fn set_app_language(language: String, app: AppHandle) {
    TRAY_RU.store(language == "ru", Ordering::Relaxed);
    refresh_tray(&app);
}

#[tauri::command]
fn tray_connect_server(server_id: String, app: AppHandle) {
    spawn_background_connect_specific(app, server_id);
}

#[tauri::command]
fn tray_toggle_connection(app: AppHandle) {
    spawn_background_toggle(app, false);
}
#[tauri::command]
fn tray_open_main(app: AppHandle) {
    if let Some(t) = app.get_window("tray") {
        let _ = t.hide();
    }
    if let Some(w) = app.get_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn tray_quit(app: AppHandle) {
    if let Some(t) = app.get_window("tray") {
        let _ = t.hide();
    }
    graceful_shutdown_and_exit(&app);
}

#[tauri::command]
fn tray_reconnect(app: AppHandle) {
    spawn_background_reconnect(app);
}

#[tauri::command]
fn tray_repair(app: AppHandle) {
    spawn_background_repair(app);
}

fn toggle_tray_popup(app: &AppHandle, position: tauri::PhysicalPosition<f64>) {
    let Some(w) = app.get_window("tray") else { return };
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
        return;
    }
    let size = w.outer_size().unwrap_or(tauri::PhysicalSize { width: 340, height: 500 });
    let mut x = position.x as i32 - size.width as i32 + 24;
    let mut y = position.y as i32 - size.height as i32 - 12;
    if let Ok(Some(mon)) = w.current_monitor() {
        let mp = mon.position();
        let ms = mon.size();
        let max_x = mp.x + ms.width as i32 - size.width as i32 - 8;
        x = x.clamp(mp.x + 8, max_x.max(mp.x + 8));
        if y < mp.y + 8 {
            y = position.y as i32 + 12;
        }
    }
    let _ = w.set_position(tauri::PhysicalPosition { x, y });
    let _ = w.emit("tray-popup-shown", ());
    let _ = w.show();
    let _ = w.set_focus();
}
fn position_notify_window(w: &tauri::Window) {
    let size = w.outer_size().unwrap_or(tauri::PhysicalSize { width: 380, height: 320 });
    let mon = w
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| w.current_monitor().ok().flatten());
    if let Some(mon) = mon {
        let mp = mon.position();
        let ms = mon.size();
        let x = mp.x + ms.width as i32 - size.width as i32 - 16;
        let y = mp.y + ms.height as i32 - size.height as i32 - 72;
        let _ = w.set_position(tauri::PhysicalPosition { x, y });
    }
}

#[derive(Clone, serde::Serialize)]
struct NotifyPayload {
    title: String,
    body: Option<String>,
    variant: String,
}
static NOTIFY_READY: AtomicBool = AtomicBool::new(false);
static NOTIFY_QUEUE: Mutex<Vec<NotifyPayload>> = Mutex::new(Vec::new());

fn deliver_notification(app: &AppHandle, payload: NotifyPayload) {
    let Some(w) = app.get_window("notify") else { return };
    position_notify_window(&w);
    if !w.is_visible().unwrap_or(false) {
        let _ = w.show();
    }
    let _ = w.emit("wawity-notify", payload);
}

fn push_notification(app: &AppHandle, title: String, body: Option<String>, variant: &str) {
    let payload = NotifyPayload {
        title,
        body,
        variant: variant.to_string(),
    };
    if !NOTIFY_READY.load(Ordering::SeqCst) {
        NOTIFY_QUEUE.lock().unwrap().push(payload);
        return;
    }
    deliver_notification(app, payload);
}

#[tauri::command]
fn notify_ready(app: AppHandle) {
    NOTIFY_READY.store(true, Ordering::SeqCst);
    let pending: Vec<NotifyPayload> = std::mem::take(&mut *NOTIFY_QUEUE.lock().unwrap());
    for payload in pending {
        deliver_notification(&app, payload);
    }
}

#[tauri::command]
fn show_notification(title: String, body: Option<String>, variant: Option<String>, app: AppHandle) {
    push_notification(&app, title, body, &variant.unwrap_or_else(|| "info".to_string()));
}
fn format_speed(bps: f64) -> String {
    if bps >= 1_000_000.0 {
        format!("{:.1} MB/s", bps / 1_000_000.0)
    } else if bps >= 1_000.0 {
        format!("{:.0} KB/s", bps / 1_000.0)
    } else {
        format!("{:.0} B/s", bps)
    }
}

pub fn refresh_tray(app: &AppHandle) {
    let state: tauri::State<AppState> = app.state();

    let (connected, always_on_active, server_name, iface_name) = {
        let Ok(pm) = state.process_manager.try_lock() else { return };
        let Ok(routing) = state.routing_manager.try_lock() else { return };
        let Ok(tun) = state.tun_manager.try_lock() else { return };

        let connected = pm.is_running();
        let always_on_active = routing.is_always_on_active();
        let server_name = state.connected_server_name.lock().unwrap().clone();
        let iface_name = tun.get_interface_name().to_string();

        (connected, always_on_active, server_name, iface_name)
    };

    let (bytes_rx, bytes_tx) = if connected {
        crate::network::read_interface_stats(&iface_name)
    } else {
        (0, 0)
    };

    let (speed_rx, speed_tx) = {
        let mut prev = state.tray_prev_stats.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(prev.2).as_secs_f64();
        let speed = if connected && prev.0 > 0 && bytes_rx >= prev.0 && elapsed > 0.05 {
            (
                (bytes_rx - prev.0) as f64 / elapsed,
                bytes_tx.saturating_sub(prev.1) as f64 / elapsed,
            )
        } else {
            (0.0, 0.0)
        };
        *prev = (bytes_rx, bytes_tx, now);
        speed
    };

    let ru = tray_ru();
    let tooltip = if connected {
        let fallback = if ru { "Подключено" } else { "Connected" };
        format!(
            "Wawity — {} | ↓{} ↑{}",
            server_name.unwrap_or_else(|| fallback.into()),
            format_speed(speed_rx),
            format_speed(speed_tx)
        )
    } else if always_on_active {
        if ru { "Wawity — Блокировка".to_string() } else { "Wawity — Locked down".to_string() }
    } else {
        "Wawity VPN".to_string()
    };

    let mut last = LAST_TOOLTIP.lock().unwrap();
    if *last != tooltip {
        *last = tooltip.clone();
        let _ = app.tray_handle().set_tooltip(&tooltip);
    }
}

fn graceful_shutdown_and_exit(app: &AppHandle) {
    let state: tauri::State<AppState> = app.state();
    let handles = ConnectionHandles::from_state(&state);

    thread::spawn(|| {
        thread::sleep(Duration::from_secs(15));
        std::process::exit(0);
    });

    thread::spawn(move || {
        crate::util::net_debug_log("exit: graceful shutdown started");
        if let Err(e) = stop_session(&handles) {
            crate::util::net_debug_log(&format!("exit: disconnect path failed ({}), forcing shutdown", e));
            let always_on_flag = *handles.always_on.lock().unwrap();
            let _ = handles.process_manager.lock().unwrap().stop();
            handles.routing_manager.lock().unwrap().disable_dns_leak_guard();
            let _ = handles.routing_manager.lock().unwrap().end_connection(always_on_flag);
            let _ = handles.tun_manager.lock().unwrap().teardown_interface();
        }
        handles.process_manager.lock().unwrap().kill_orphans();
        crate::util::net_debug_log("exit: shutdown complete");
        std::process::exit(0);
    });
}

fn spawn_background_toggle(app: AppHandle, announce: bool) {
    thread::spawn(move || {
        let state: tauri::State<AppState> = app.state();
        let ru = tray_ru();
        let was_connected = state.current_exit_url.lock().unwrap().is_some();

        if announce && was_connected {
            let title = if ru { "Отключение…" } else { "Disconnecting…" };
            push_notification(&app, title.to_string(), None, "warning");
        }

        let handles = ConnectionHandles::from_state(&state);
        let running = state.process_manager.lock().unwrap().is_running();

        if running {
            if announce && !was_connected {
                let title = if ru { "Отключение…" } else { "Disconnecting…" };
                push_notification(&app, title.to_string(), None, "warning");
            }
            let _ = stop_session(&handles);
        } else {
            let target = {
                let servers = state.tray_servers.lock().unwrap();
                let selected = state.tray_selected_id.lock().unwrap().clone();
                selected
                    .and_then(|id| servers.iter().find(|s| s.id == id).cloned())
                    .or_else(|| servers.first().cloned())
            };

            if let Some(srv) = target {
                if announce {
                    let title = if ru { "Подключение…" } else { "Connecting…" };
                    push_notification(&app, title.to_string(), Some(srv.name.clone()), "info");
                }
                let bypass = state.bypass_apps.lock().unwrap().clone();
                let kill_switch = *state.default_kill_switch.lock().unwrap();
                let quantum_resistant = *state.default_quantum_resistant.lock().unwrap();
                let _ = start_session(
                    &srv.url,
                    None,
                    Some(srv.name.clone()),
                    None,
                    kill_switch,
                    bypass,
                    quantum_resistant,
                    &handles,
                );
            } else if announce {
                let title = if ru { "Нет серверов" } else { "No servers" };
                let body = if ru { "Добавьте подписку в Wawity" } else { "Add a subscription in Wawity" };
                push_notification(&app, title.to_string(), Some(body.to_string()), "error");
            }
        }

        refresh_tray(&app);
        if let Some(w) = app.get_window("main") {
            let _ = w.emit("vpn-status-changed", ());
        }
    });
}

fn spawn_background_connect_specific(app: AppHandle, server_id: String) {
    thread::spawn(move || {
        let state: tauri::State<AppState> = app.state();
        let handles = ConnectionHandles::from_state(&state);

        let server = state
            .tray_servers
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.id == server_id)
            .cloned();

        if let Some(srv) = server {
            let bypass = state.bypass_apps.lock().unwrap().clone();
            let kill_switch = *state.default_kill_switch.lock().unwrap();
            let quantum_resistant = *state.default_quantum_resistant.lock().unwrap();
            let running = state.process_manager.lock().unwrap().is_running();
            let same_server = state.current_exit_url.lock().unwrap().as_deref() == Some(srv.url.as_str());

            if running && !same_server {
                let _ = switch_session(
                    &srv.url, None, Some(srv.name.clone()), None,
                    bypass, quantum_resistant, &handles,
                );
            } else if !running {
                let _ = start_session(
                    &srv.url, None, Some(srv.name.clone()), None,
                    kill_switch, bypass, quantum_resistant, &handles,
                );
            }

            *state.tray_selected_id.lock().unwrap() = Some(server_id);
        }

        refresh_tray(&app);
        if let Some(w) = app.get_window("main") {
            let _ = w.emit("vpn-status-changed", ());
        }
    });
}

fn spawn_background_reconnect(app: AppHandle) {
    thread::spawn(move || {
        let state: tauri::State<AppState> = app.state();
        let handles = ConnectionHandles::from_state(&state);

        let exit_url = state.current_exit_url.lock().unwrap().clone();
        let entry_url = state.current_entry_url.lock().unwrap().clone();
        let server_name = state.connected_server_name.lock().unwrap().clone();
        let entry_name = state.connected_entry_server_name.lock().unwrap().clone();
        let bypass = state.bypass_apps.lock().unwrap().clone();
        let kill_switch = state.routing_manager.lock().unwrap().is_kill_switch_active();
        let quantum_resistant = *state.current_quantum_resistant.lock().unwrap();

        if let Some(url) = exit_url {
            let _ = stop_session(&handles);
            thread::sleep(Duration::from_millis(400));
            let _ = start_session(
                &url, entry_url, server_name, entry_name,
                kill_switch, bypass, quantum_resistant, &handles,
            );
        }

        refresh_tray(&app);
        if let Some(w) = app.get_window("main") {
            let _ = w.emit("vpn-status-changed", ());
        }
    });
}

fn spawn_background_repair(app: AppHandle) {
    thread::spawn(move || {
        let state: tauri::State<AppState> = app.state();
        let handles = ConnectionHandles::from_state(&state);
        let pm = handles.process_manager.lock().unwrap();
        pm.kill_orphans();
        let _ = pm.stop();
        drop(pm);
        handles.routing_manager.lock().unwrap().force_cleanup();
        let _ = handles.tun_manager.lock().unwrap().teardown_interface();
        *handles.current_config.lock().unwrap() = None;
        *handles.connected_server_name.lock().unwrap() = None;
        *handles.connected_entry_server_name.lock().unwrap() = None;
        *handles.current_exit_url.lock().unwrap() = None;
        *handles.current_entry_url.lock().unwrap() = None;

        refresh_tray(&app);
        if let Some(w) = app.get_window("main") {
            let _ = w.emit("vpn-status-changed", ());
        }
    });
}

fn on_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { position, .. } => {
            toggle_tray_popup(app, position);
        }

        SystemTrayEvent::DoubleClick { .. } => {
            if let Some(w) = app.get_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }

        SystemTrayEvent::RightClick { position, .. } => {
            toggle_tray_popup(app, position);
        }

        _ => {}
    }
}

#[cfg(target_os = "windows")]
fn apply_window_effects(window: &tauri::Window) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
    };
    let hwnd = window.hwnd().unwrap();
    let hwnd_val = HWND(hwnd.0 as _);
    unsafe {
        let corner_pref = DWMWCP_ROUND;
        let _ = DwmSetWindowAttribute(
            hwnd_val,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_pref as *const _ as *const _,
            std::mem::size_of_val(&corner_pref) as u32,
        );
    }
}

#[cfg(not(target_os = "windows"))]
fn apply_window_effects(_window: &tauri::Window) {}

#[cfg(target_os = "windows")]
fn check_admin() -> bool {
    silent_command("net")
        .args(&["session"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(target_os = "windows")]
fn show_error_box(title: &str, message: &str) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(msg_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn close_splash_window(app: &AppHandle) {
    if let Some(splash) = app.get_window("splash") {
        let _ = splash.close();
    }
}

#[tauri::command]
fn show_main_window(app: AppHandle) {
    if let Some(w) = app.get_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
    close_splash_window(&app);
}

#[tauri::command]
fn close_splash_only(app: AppHandle) {
    close_splash_window(&app);
}

#[tauri::command]
fn is_launched_hidden(state: tauri::State<'_, AppState>) -> bool {
    state.launched_hidden
}

fn main() {
    let launched_hidden = std::env::args().any(|a| a.eq_ignore_ascii_case("--autostart"));

    #[cfg(target_os = "windows")]
    {
        if !check_admin() {
            if let Ok(exe) = std::env::current_exe() {
                let exe_str = exe.display().to_string().replace('\'', "''");
                let forwarded_args: Vec<String> = std::env::args().skip(1).collect();

                let cmd = if forwarded_args.is_empty() {
                    format!("Start-Process -FilePath '{}' -Verb RunAs -WindowStyle Hidden", exe_str)
                } else {
                    let arg_list = forwarded_args
                        .iter()
                        .map(|a| format!("'{}'", a.replace('\'', "''")))
                        .collect::<Vec<_>>()
                        .join(",");
                    format!(
                        "Start-Process -FilePath '{}' -ArgumentList {} -Verb RunAs -WindowStyle Hidden",
                        exe_str, arg_list
                    )
                };

                let _ = silent_command("powershell")
                    .args(&["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &cmd])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn();
                std::process::exit(0);
            }
        }
    }

    let mut state = match AppState::new() {
        Ok(s) => s,
        Err(e) => {
            #[cfg(target_os = "windows")]
            show_error_box(
                "Wawity VPN - Initialization Error",
                &format!(
                    "Failed to initialize:\n\n{}\n\nRun as administrator.",
                    e
                ),
            );
            #[cfg(not(target_os = "windows"))]
            eprintln!("Init failed: {}", e);
            std::process::exit(1);
        }
    };

    state.launched_hidden = launched_hidden;

    telemetry::install_panic_hook();

    let app = tauri::Builder::default()
        .manage(state)
        .manage(appstats::AppTrafficState::default())
    .manage(presence::PresenceLink::spawn())
        .invoke_handler(tauri::generate_handler![
            connect_vpn,
            disconnect_vpn,
            switch_vpn_server,
            set_always_on,
            get_vpn_status,
            appstats::get_app_traffic,
            validate_subscription,
            get_logs,
            get_saved_servers,
            add_saved_server,
            remove_saved_server,
            update_bypass_apps,
            set_split_rules,
            detect_blocked_services,
            fetch_subscription,
            fetch_subscription_raw,
            fetch_rule_list,
            geolocate_servers,
            ping_servers,
            list_processes,
            repair_network,
            measure_tunnel_latency,
            sync_tray_state,
            get_subscription_info,
            show_main_window,
            close_splash_only,
            is_launched_hidden,
            set_start_on_boot,
            get_start_on_boot,
            scan_installed_games,
            list_installed_apps,
            presence::sync_discord_presence,
            set_app_language,
            sync_hotkeys,
            show_notification,
            notify_ready,
            tray_connect_server,
            tray_toggle_connection,
            tray_open_main,
            tray_quit,
            tray_reconnect,
            tray_repair,
            appicons::collect_app_icons,
            commands::is_first_launch,
            commands::finish_first_launch,
            commands::run_speed_test,
            commands::cancel_speed_test,
            commands::audit_leaks,
            commands::probe_reachability,
            commands::probe_servers_deep,
            appwatch::arm_app_watch,
            appwatch::disarm_app_watch,
            appwatch::app_watch_state,
            hwid::get_hwid,
            hwid::reset_hwid,
            hwid::set_hwid_enabled,
            telemetry::set_telemetry_enabled,
            telemetry::track_event,
            telemetry::report_error,
            coreinfo::core_info,
            streamdetect::stream_capture_running,
            streamdetect::stream_capture_state,
            firewall::firewall_wawity_rules,
            migrate::scan_foreign_clients,
        ])
        .system_tray(SystemTray::new())
        .on_system_tray_event(on_tray_event)
        .setup(|app| {
            hwid::init(&app.handle());
            let win = app.get_window("main").unwrap();

            let tray_popup = WindowBuilder::new(
                app,
                "tray",
                WindowUrl::App("index.html?tray=1".into()),
            )
            .title("Wawity")
            .inner_size(330.0, 480.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .focused(false)
            .build()?;

            let tray_popup_ref = tray_popup.clone();
            tray_popup.on_window_event(move |ev| {
                if let WindowEvent::Focused(false) = ev {
                    let _ = tray_popup_ref.hide();
                }
            });
            let notify_popup = WindowBuilder::new(
                app,
                "notify",
                WindowUrl::App("index.html?notify=1".into()),
            )
            .title("Wawity")
            .inner_size(380.0, 320.0)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(true)
            .focused(false)
            .build()?;
            let _ = notify_popup.set_ignore_cursor_events(true);
            position_notify_window(&notify_popup);
            apply_window_effects(&win);

            let hidden_launch = app.state::<AppState>().launched_hidden;

            if !hidden_launch {
                if let Some(splash) = app.get_window("splash") {
                    let _ = splash.show();
                    let _ = splash.set_focus();
                }
            }

            let w = win.clone();
            win.on_window_event(move |ev| {
                if let WindowEvent::CloseRequested { api, .. } = ev {
                    let _ = w.hide();
                    api.prevent_close();
                }
            });

            refresh_tray(&app.app_handle());

            let fallback_app = app.app_handle();
            thread::spawn(move || {
                thread::sleep(Duration::from_secs(10));
                let hidden = fallback_app.state::<AppState>().launched_hidden;
                if let Some(main_win) = fallback_app.get_window("main") {
                    if hidden {
                        close_splash_window(&fallback_app);
                    } else if let Ok(false) = main_win.is_visible() {
                        log::warn!("frontend never signalled ready, force-showing window");
                        let _ = main_win.show();
                        let _ = main_win.set_focus();
                        close_splash_window(&fallback_app);
                    }
                }
            });

            let startup_app = app.app_handle();
            thread::spawn(move || {
                let state: tauri::State<AppState> = startup_app.state();
                state.process_manager.lock().unwrap().kill_orphans();

                let persisted_always_on = *state.always_on.lock().unwrap();
                if persisted_always_on {
                    match resolve_own_exe_path() {
                        Ok(app_exe_path) => {
                            if let Err(e) = state.routing_manager.lock().unwrap().enable_always_on(&app_exe_path) {
                                log::error!("failed to re-engage persisted always-on lockdown at startup: {}", e);
                                state.routing_manager.lock().unwrap().force_cleanup();
                                *state.always_on.lock().unwrap() = false;
                                state.persist_always_on_flag(false);
                            }
                        }
                        Err(e) => {
                            log::error!("cannot resolve own exe path for always-on re-engage: {}", e);
                            state.routing_manager.lock().unwrap().force_cleanup();
                        }
                    }
                } else {
                    state.routing_manager.lock().unwrap().force_cleanup();
                }

                refresh_tray(&startup_app);
                if let Some(w) = startup_app.get_window("main") {
                    let _ = w.emit("vpn-status-changed", ());
                }
            });

            let watchdog_app = app.app_handle();
            thread::spawn(move || {
                let mut last_notified: Option<Instant> = None;
                loop {
                    thread::sleep(Duration::from_secs(10));
                    let state: tauri::State<AppState> = watchdog_app.state();
                    let repaired = state.routing_manager.lock().unwrap().verify_and_repair();
                    match repaired {
                        Ok(true) => {
                            log::warn!("firewall lockdown drift detected and repaired");
                            refresh_tray(&watchdog_app);
                            let should_notify = last_notified
                                .map(|t| t.elapsed() >= Duration::from_secs(30))
                                .unwrap_or(true);
                            if should_notify {
                                last_notified = Some(Instant::now());
                                if let Some(w) = watchdog_app.get_window("main") {
                                    let _ = w.emit("lockdown-repaired", ());
                                }
                            }
                        }
                        Ok(false) => {}
                        Err(e) => log::error!("lockdown watchdog repair failed: {}", e),
                    }
                }
            });

            let bg_app = app.app_handle();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(1500));
                refresh_tray(&bg_app);
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Failed to build Tauri app");

    app.run(move |app_handle, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
            api.prevent_exit();
            graceful_shutdown_and_exit(app_handle);
        }
    });
}
