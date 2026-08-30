#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::io::Cursor;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use serde::Deserialize;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ};
use winreg::{RegKey, RegValue};
use wry::application::dpi::LogicalSize;
use wry::application::event::{Event, WindowEvent};
use wry::application::event_loop::{ControlFlow, EventLoop, EventLoopProxy};
use wry::application::window::{Window, WindowBuilder};
use wry::webview::{WebContext, WebView, WebViewBuilder};

const UI_HTML: &str = include_str!("../ui/index.html");
const PAYLOAD: &[u8] = include_bytes!("../payload/app.zip");
const WV2_BOOTSTRAP: &[u8] = include_bytes!("../payload/MicrosoftEdgeWebView2Setup.exe");

const APP_NAME: &str = "Wawity";
const APP_EXE: &str = "WawityApp.exe";
const CLI_EXE: &str = "wawity.exe";
const VARIANT: &str = match option_env!("WAWITY_VARIANT") {
    Some(v) => v,
    None => "desktop",
};
const APP_VERSION: &str = "0.2.1";
const UNINSTALL_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Wawity";
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const DATA_DIR_NAME: &str = "com.wawity.vpn";
const FLAG_DIR_NAME: &str = "Wawity";
const NO_WINDOW: u32 = 0x0800_0000;

#[derive(Deserialize)]
struct UiMsg {
    cmd: String,
    #[serde(default)]
    dir: String,
    #[serde(default)]
    desktop: bool,
    #[serde(default)]
    keep: Option<bool>,
    #[serde(default)]
    purge: bool,
    #[serde(default)]
    components: String,
    #[serde(default)]
    cli: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum SetupMode {
    Install,
    Uninstall,
}

struct ExistingSetup {
    install_dir: Option<PathBuf>,
    data_present: bool,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let silent = args.iter().any(|a| a == "--silent");
    let confirmed = args.iter().any(|a| a == "--confirmed");
    let purge = args.iter().any(|a| a == "--purge");

    if server_mode(&args) && !uninstall_mode(&args) {
        std::process::exit(run_server_install(&args));
    }

    if uninstall_mode(&args) {
        if silent || confirmed {
            headless_uninstall(purge);
            return;
        }
        ensure_webview2();
        run_window(SetupMode::Uninstall);
        return;
    }

    ensure_webview2();
    run_window(SetupMode::Install);
}

fn uninstall_mode(args: &[String]) -> bool {
    if args.iter().any(|a| a == "--uninstall") {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|me| me.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .map(|name| name == "uninstall.exe")
        .unwrap_or(false)
}

fn headless_uninstall(purge: bool) {
    if !elevated() {
        let _ = relaunch_elevated(purge, false);
        return;
    }
    perform_uninstall(purge, None);
    spawn_sweep(8);
}

fn run_window(mode: SetupMode) {
    let existing = inspect_machine();
    let suggested_dir = existing
        .install_dir
        .clone()
        .unwrap_or_else(default_install_dir);

    let event_loop: EventLoop<String> = EventLoop::with_user_event();
    let proxy = event_loop.create_proxy();

    let title = match mode {
        SetupMode::Install => "Установка Wawity",
        SetupMode::Uninstall => "Удаление Wawity",
    };

    let window = match WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize::new(460.0, 640.0))
        .with_resizable(false)
        .with_decorations(false)
        .with_transparent(true)
        .build(&event_loop)
    {
        Ok(w) => w,
        Err(e) => fatal(&format!("Не удалось создать окно: {e}")),
    };

    let mut web_context = WebContext::new(Some(setup_data_dir()));

    let webview = match build_ui(window, &mut web_context, proxy.clone(), mode, &existing, &suggested_dir) {
        Ok(wv) => wv,
        Err(e) => {
            if mode == SetupMode::Uninstall {
                fallback_native_uninstall();
                return;
            }
            fatal(&format!(
                "Не удалось запустить WebView2: {e}\n\nПереустановите Microsoft Edge WebView2 Runtime и попробуйте снова."
            ));
        }
    };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(script) => {
                let _ = webview.evaluate_script(&script);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => std::process::exit(0),
            _ => {}
        }
    });
}

fn fallback_native_uninstall() {
    let confirmed = rfd::MessageDialog::new()
        .set_title("Удаление Wawity")
        .set_description("Удалить Wawity с этого компьютера?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    if !confirmed {
        return;
    }
    let purge = rfd::MessageDialog::new()
        .set_title("Удаление Wawity")
        .set_description("Удалить также все данные: настройки, серверы и подписки?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    if elevated() {
        perform_uninstall(purge, None);
        spawn_sweep(6);
    } else {
        let _ = relaunch_elevated(purge, false);
    }
}

fn inspect_machine() -> ExistingSetup {
    let install_dir = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(UNINSTALL_KEY)
        .and_then(|key| key.get_value::<String, _>("InstallLocation"))
        .ok()
        .map(PathBuf::from)
        .filter(|dir| dir.join(APP_EXE).exists() || dir.join(CLI_EXE).exists());
    ExistingSetup {
        install_dir,
        data_present: user_data_present(),
    }
}

fn user_data_dirs() -> Vec<PathBuf> {
    let mut spots = Vec::new();
    if let Ok(roaming) = std::env::var("APPDATA") {
        spots.push(PathBuf::from(&roaming).join(DATA_DIR_NAME));
        spots.push(PathBuf::from(&roaming).join(FLAG_DIR_NAME));
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        spots.push(PathBuf::from(local).join(DATA_DIR_NAME));
    }
    spots
}

fn user_data_present() -> bool {
    user_data_dirs().iter().any(|dir| dir.exists())
}

fn purge_user_data() {
    for dir in user_data_dirs() {
        let _ = fs::remove_dir_all(&dir);
    }
    let temp = std::env::temp_dir();
    for log in ["wawity.log", "wawity_net.log"] {
        let _ = fs::remove_file(temp.join(log));
    }
}

fn build_ui(
    window: Window,
    web_context: &mut WebContext,
    proxy: EventLoopProxy<String>,
    mode: SetupMode,
    existing: &ExistingSetup,
    suggested_dir: &Path,
) -> wry::Result<WebView> {
    let boot = serde_json::json!({
        "mode": match mode {
            SetupMode::Install => "install",
            SetupMode::Uninstall => "uninstall",
        },
        "dir": suggested_dir.to_string_lossy(),
        "version": APP_VERSION,
        "installed": existing.install_dir.is_some(),
        "dataPresent": existing.data_present,
        "variant": VARIANT,
    });

    WebViewBuilder::new(window)?
        .with_web_context(web_context)
        .with_transparent(true)
        .with_initialization_script(&format!("window.__boot = {};", boot))
        .with_html(UI_HTML)?
        .with_ipc_handler(move |win, raw| {
            let parsed: Result<UiMsg, _> = serde_json::from_str(&raw);
            let Ok(msg) = parsed else { return };
            match msg.cmd.as_str() {
                "drag" => {
                    let _ = win.drag_window();
                }
                "minimize" => win.set_minimized(true),
                "close" => std::process::exit(0),
                "browse" => {
                    if let Some(picked) = rfd::FileDialog::new().pick_folder() {
                        let target = if picked.ends_with(APP_NAME) {
                            picked
                        } else {
                            picked.join(APP_NAME)
                        };
                        let _ = proxy.send_event(format!(
                            "setDir({})",
                            serde_json::json!(target.to_string_lossy())
                        ));
                    }
                }
                "install" => {
                    let dir = PathBuf::from(msg.dir.trim());
                    if dir.as_os_str().is_empty() {
                        return;
                    }
                    let px = proxy.clone();
                    let desktop = msg.desktop;
                    let keep = msg.keep.unwrap_or(true);
                    let components = normalize_components(&msg.components);
                    thread::spawn(move || {
                        let ui = px.clone();
                        let report = move |p: f64, label: &str| {
                            let _ = ui.send_event(format!(
                                "setProgress({}, {})",
                                p,
                                serde_json::json!(label)
                            ));
                        };
                        match do_install(&dir, desktop, keep, &components, &report) {
                            Ok(()) => {
                                let _ = px.send_event(format!(
                                    "installDone({})",
                                    serde_json::json!(dir.to_string_lossy())
                                ));
                            }
                            Err(e) => {
                                let _ = px.send_event(format!(
                                    "installFailed({})",
                                    serde_json::json!(e)
                                ));
                            }
                        }
                    });
                }
                "uninstall" => {
                    let px = proxy.clone();
                    let purge = msg.purge;
                    thread::spawn(move || {
                        if elevated() {
                            perform_uninstall(purge, Some(&px));
                            spawn_sweep(6);
                            let _ = px.send_event("uninstallDone()".to_string());
                        } else {
                            let _ = px.send_event(format!(
                                "setProgress(0.35, {})",
                                serde_json::json!("Ожидание прав администратора")
                            ));
                            if relaunch_elevated(purge, true) {
                                let _ = px.send_event("uninstallDone()".to_string());
                            } else {
                                let _ = px.send_event(format!(
                                    "uninstallFailed({})",
                                    serde_json::json!("Не получены права администратора. Удаление отменено.")
                                ));
                            }
                        }
                    });
                }
                "launch" => {
                    let dir = PathBuf::from(msg.dir);
                    if msg.cli {
                        let _ = Command::new("cmd")
                            .args(["/C", "start", "Wawity CLI", "cmd", "/K", CLI_EXE])
                            .current_dir(&dir)
                            .spawn();
                    } else {
                        let _ = Command::new(dir.join(APP_EXE)).current_dir(&dir).spawn();
                    }
                    std::process::exit(0);
                }
                _ => {}
            }
        })
        .build()
}

fn setup_data_dir() -> PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let dir = base.join("WawitySetup").join("WebView2");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn fatal(message: &str) -> ! {
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("Wawity Setup")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
    std::process::exit(1);
}

fn do_install(
    dir: &Path,
    desktop: bool,
    keep_user_data: bool,
    components: &str,
    progress: &dyn Fn(f64, &str),
) -> Result<(), String> {
    let tick = |p: f64, label: &str| progress(p, label);

    tick(0.03, "Завершение процессов");
    quiet(Command::new("taskkill").args(["/F", "/IM", APP_EXE]));
    quiet(Command::new("taskkill").args(["/F", "/IM", CLI_EXE]));
    quiet(Command::new("taskkill").args(["/F", "/IM", "sing-box-x86_64.exe"]));

    if !keep_user_data {
        tick(0.05, "Очистка старых данных");
        purge_user_data();
    }

    tick(0.07, "Подготовка папки");
    fs::create_dir_all(dir).map_err(|e| format!("Не удалось создать папку установки: {e}"))?;

    tick(0.10, "Распаковка файлов");
    let mut bundle = zip::ZipArchive::new(Cursor::new(PAYLOAD))
        .map_err(|e| format!("Повреждён встроенный архив: {e}"))?;
    let total = bundle.len().max(1);
    let mut unpacked: u64 = 0;
    for i in 0..bundle.len() {
        let mut entry = bundle
            .by_index(i)
            .map_err(|e| format!("Ошибка чтения архива: {e}"))?;
        let rel = match entry.enclosed_name() {
            Some(p) => p.to_owned(),
            None => continue,
        };
        let base = rel
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if components == "cli" && base == APP_EXE.to_lowercase() {
            continue;
        }
        if components == "desktop" && base == CLI_EXE.to_lowercase() {
            continue;
        }
        let out = dir.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| format!("Не удалось создать папку: {e}"))?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent).map_err(|e| format!("Не удалось создать папку: {e}"))?;
            }
            let mut file = fs::File::create(&out)
                .map_err(|e| format!("Не удалось записать {}: {e}", out.display()))?;
            unpacked += std::io::copy(&mut entry, &mut file)
                .map_err(|e| format!("Не удалось распаковать {}: {e}", out.display()))?;
        }
        tick(
            0.10 + 0.60 * ((i + 1) as f64 / total as f64),
            "Распаковка файлов",
        );
    }

    if components != "desktop" && !dir.join(CLI_EXE).exists() {
        return Err(format!(
            "В пакете нет {}: соберите CLI через build.bat пункт [2]",
            CLI_EXE
        ));
    }
    if components != "cli" && !dir.join(APP_EXE).exists() {
        return Err(format!(
            "В пакете нет {}: соберите десктоп через build.bat пункт [1]",
            APP_EXE
        ));
    }

    tick(0.74, "Создание деинсталлятора");
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    fs::copy(&me, dir.join("uninstall.exe"))
        .map_err(|e| format!("Не удалось создать uninstall.exe: {e}"))?;

    tick(0.82, "Регистрация в системе");
    write_uninstall_entry(dir, (unpacked / 1024) as u32, components)?;

    tick(0.90, "Создание ярлыков");
    if components != "cli" {
        make_shortcuts(dir, desktop);
    }
    if components != "desktop" {
        make_cli_shortcut(dir);
        tick(0.95, "Добавление wawity в PATH");
        add_to_system_path(dir);
    }

    tick(1.0, "Готово");
    Ok(())
}

fn write_uninstall_entry(dir: &Path, size_kb: u32, components: &str) -> Result<(), String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let (key, _) = hklm
        .create_subkey(UNINSTALL_KEY)
        .map_err(|e| format!("Не удалось записать в реестр: {e}"))?;
    let exe = if components == "cli" {
        dir.join(CLI_EXE)
    } else {
        dir.join(APP_EXE)
    };
    let uninstaller = dir.join("uninstall.exe");
    let display = if components == "cli" {
        "Wawity CLI".to_string()
    } else {
        APP_NAME.to_string()
    };
    let sets: [(&str, String); 8] = [
        ("DisplayName", display),
        ("DisplayVersion", APP_VERSION.to_string()),
        ("Publisher", APP_NAME.to_string()),
        ("DisplayIcon", exe.to_string_lossy().into_owned()),
        ("InstallLocation", dir.to_string_lossy().into_owned()),
        ("UninstallString", format!("\"{}\" --uninstall", uninstaller.display())),
        ("QuietUninstallString", format!("\"{}\" --uninstall --silent", uninstaller.display())),
        ("InstallDate", String::new()),
    ];
    for (name, value) in sets {
        key.set_value(name, &value).map_err(|e| e.to_string())?;
    }
    key.set_value("EstimatedSize", &size_kb).map_err(|e| e.to_string())?;
    key.set_value("NoModify", &1u32).map_err(|e| e.to_string())?;
    key.set_value("NoRepair", &1u32).map_err(|e| e.to_string())?;
    Ok(())
}

fn make_shortcuts(dir: &Path, desktop: bool) {
    let exe = dir.join(APP_EXE).display().to_string();
    let workdir = dir.display().to_string();
    let mut script = format!(
        "$sh = New-Object -ComObject WScript.Shell; $menu = Join-Path $env:ProgramData 'Microsoft\\Windows\\Start Menu\\Programs\\Wawity.lnk'; $lnk = $sh.CreateShortcut($menu); $lnk.TargetPath = '{exe}'; $lnk.WorkingDirectory = '{workdir}'; $lnk.IconLocation = '{exe},0'; $lnk.Save();"
    );
    if desktop {
        script.push_str(&format!(
            " $dt = Join-Path ([Environment]::GetFolderPath('Desktop')) 'Wawity.lnk'; $dl = $sh.CreateShortcut($dt); $dl.TargetPath = '{exe}'; $dl.WorkingDirectory = '{workdir}'; $dl.IconLocation = '{exe},0'; $dl.Save();"
        ));
    }
    run_ps(&script);
}

fn perform_uninstall(purge: bool, px: Option<&EventLoopProxy<String>>) {
    let tick = |p: f64, label: &str| {
        if let Some(proxy) = px {
            let _ = proxy.send_event(format!("setProgress({}, {})", p, serde_json::json!(label)));
        }
    };

    tick(0.1, "Завершение процессов");
    quiet(Command::new("taskkill").args(["/F", "/IM", APP_EXE]));
    quiet(Command::new("taskkill").args(["/F", "/IM", CLI_EXE]));
    quiet(Command::new("taskkill").args(["/F", "/IM", "sing-box-x86_64.exe"]));

    tick(0.3, "Удаление правил брандмауэра");
    run_ps("Get-NetFirewallRule -Name 'WawityFW_*' -ErrorAction SilentlyContinue | Remove-NetFirewallRule -ErrorAction SilentlyContinue");

    tick(0.5, "Удаление ярлыков");
    run_ps("Remove-Item (Join-Path $env:ProgramData 'Microsoft\\Windows\\Start Menu\\Programs\\Wawity.lnk') -Force -ErrorAction SilentlyContinue; Remove-Item (Join-Path $env:ProgramData 'Microsoft\\Windows\\Start Menu\\Programs\\Wawity CLI.lnk') -Force -ErrorAction SilentlyContinue; Remove-Item (Join-Path ([Environment]::GetFolderPath('Desktop')) 'Wawity.lnk') -Force -ErrorAction SilentlyContinue");
    run_ps("Remove-Item (Join-Path $env:LOCALAPPDATA 'WawitySetup') -Recurse -Force -ErrorAction SilentlyContinue");

    tick(0.58, "Очистка PATH");
    if let Ok(dir) = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(UNINSTALL_KEY)
        .and_then(|k| k.get_value::<String, _>("InstallLocation"))
    {
        remove_from_system_path(Path::new(&dir));
    }

    tick(0.65, "Очистка реестра");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let _ = hklm.delete_subkey_all(UNINSTALL_KEY);

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
        let _ = run.delete_value(APP_NAME);
    }

    if purge {
        tick(0.8, "Удаление настроек и серверов");
        purge_user_data();
    }

    tick(0.95, "Удаление файлов приложения");
}

fn spawn_sweep(pings: u32) {
    if let Ok(me) = std::env::current_exe() {
        if let Some(dir) = me.parent() {
            let sweep = format!("ping 127.0.0.1 -n {} > nul & rd /s /q \"{}\"", pings, dir.display());
            let _ = Command::new("cmd")
                .args(["/C", &sweep])
                .current_dir(std::env::temp_dir())
                .creation_flags(NO_WINDOW)
                .spawn();
        }
    }
}

fn elevated() -> bool {
    Command::new("net")
        .args(["session"])
        .creation_flags(NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn relaunch_elevated(purge: bool, wait: bool) -> bool {
    let Ok(me) = std::env::current_exe() else {
        return false;
    };
    let path = me.display().to_string().replace('\'', "''");
    let mut arg_list = String::from("'--uninstall','--confirmed'");
    if purge {
        arg_list.push_str(",'--purge'");
    }
    let wait_flag = if wait { " -Wait" } else { "" };
    let script = format!(
        "Start-Process -FilePath '{path}' -ArgumentList {arg_list} -Verb RunAs{wait_flag}"
    );
    run_ps(&script)
}

fn ensure_webview2() {
    if webview2_present() {
        return;
    }
    let bootstrap = std::env::temp_dir().join("wawity_wv2_bootstrap.exe");
    if fs::write(&bootstrap, WV2_BOOTSTRAP).is_err() {
        return;
    }
    let _ = Command::new(&bootstrap)
        .args(["/silent", "/install"])
        .creation_flags(NO_WINDOW)
        .status();
    let _ = fs::remove_file(&bootstrap);
}

fn webview2_present() -> bool {
    let spots = [
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        ),
        (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}",
        ),
    ];
    spots.iter().any(|(hive, path)| {
        RegKey::predef(*hive)
            .open_subkey(path)
            .and_then(|k| k.get_value::<String, _>("pv"))
            .map(|v| !v.trim().is_empty() && v != "0.0.0.0")
            .unwrap_or(false)
    })
}

fn default_install_dir() -> PathBuf {
    std::env::var("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\Program Files"))
        .join(APP_NAME)
}

fn quiet(cmd: &mut Command) {
    let _ = cmd.creation_flags(NO_WINDOW).status();
}

fn run_ps(script: &str) -> bool {
    Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .creation_flags(NO_WINDOW)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

const ENV_KEY: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

fn normalize_components(raw: &str) -> String {
    match raw {
        "cli" | "desktop" | "both" => raw.to_string(),
        _ => match VARIANT {
            "cli" => "cli".to_string(),
            "full" => "both".to_string(),
            _ => "desktop".to_string(),
        },
    }
}

fn make_cli_shortcut(dir: &Path) {
    if std::env::var("WAWITY_SERVER_INSTALL").is_ok() {
        return;
    }
    let exe = dir.join(CLI_EXE).display().to_string();
    let workdir = dir.display().to_string();
    let script = format!(
        "$sh = New-Object -ComObject WScript.Shell; $menu = Join-Path $env:ProgramData 'Microsoft\\Windows\\Start Menu\\Programs\\Wawity CLI.lnk'; $lnk = $sh.CreateShortcut($menu); $lnk.TargetPath = '{exe}'; $lnk.WorkingDirectory = '{workdir}'; $lnk.IconLocation = '{exe},0'; $lnk.Save();"
    );
    run_ps(&script);
}

fn set_expand_sz(key: &RegKey, name: &str, value: &str) {
    let mut bytes: Vec<u8> = value.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    bytes.extend_from_slice(&[0, 0]);
    let _ = key.set_raw_value(
        name,
        &RegValue {
            bytes,
            vtype: REG_EXPAND_SZ,
        },
    );
}

fn add_to_system_path(dir: &Path) {
    let target = dir.to_string_lossy().trim_end_matches('\\').to_string();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(key) = hklm.open_subkey_with_flags(ENV_KEY, KEY_READ | KEY_SET_VALUE) else {
        return;
    };
    let current: String = key.get_value("Path").unwrap_or_default();
    let present = current
        .split(';')
        .any(|p| p.trim().trim_end_matches('\\').eq_ignore_ascii_case(&target));
    if !present {
        let mut next = current.trim_end_matches(';').to_string();
        if !next.is_empty() {
            next.push(';');
        }
        next.push_str(&target);
        set_expand_sz(&key, "Path", &next);
    }
    quiet(Command::new("setx").args(["WAWITY_HOME", &target, "/M"]));
    broadcast_env_change();
}

fn remove_from_system_path(dir: &Path) {
    let target = dir.to_string_lossy().trim_end_matches('\\').to_string();
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let Ok(key) = hklm.open_subkey_with_flags(ENV_KEY, KEY_READ | KEY_SET_VALUE) else {
        return;
    };
    let current: String = key.get_value("Path").unwrap_or_default();
    let kept: Vec<&str> = current
        .split(';')
        .filter(|p| !p.trim().is_empty())
        .filter(|p| !p.trim().trim_end_matches('\\').eq_ignore_ascii_case(&target))
        .collect();
    let next = kept.join(";");
    if next != current {
        set_expand_sz(&key, "Path", &next);
    }
    let _ = key.delete_value("WAWITY_HOME");
}

#[link(name = "kernel32")]
extern "system" {
    fn AttachConsole(process_id: u32) -> i32;
    fn AllocConsole() -> i32;
    fn SetConsoleOutputCP(page: u32) -> i32;
}

fn attach_console() {
    unsafe {
        if AttachConsole(0xFFFF_FFFF) == 0 {
            AllocConsole();
        }
        SetConsoleOutputCP(65001);
    }
}

fn cprint(line: &str) {
    use std::io::Write;
    if let Ok(mut handle) = fs::OpenOptions::new().write(true).open("CONOUT$") {
        let _ = writeln!(handle, "{}", line);
        let _ = handle.flush();
    } else {
        println!("{}", line);
    }
}

fn server_mode(args: &[String]) -> bool {
    let flagged = args.iter().skip(1).any(|a| {
        let low = a.to_lowercase();
        low == "-serverinstall" || low == "--serverinstall" || low == "/serverinstall"
    });
    if flagged {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|me| me.file_name().map(|n| n.to_string_lossy().to_lowercase()))
        .map(|name| name.contains("serverinstall"))
        .unwrap_or(false)
}

fn arg_value(args: &[String], key: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(item) = iter.next() {
        if item.eq_ignore_ascii_case(key) {
            return iter.next().cloned();
        }
        let prefix = format!("{}=", key);
        if item.to_lowercase().starts_with(&prefix.to_lowercase()) {
            return Some(item[prefix.len()..].to_string());
        }
    }
    None
}

fn run_server_install(args: &[String]) -> i32 {
    attach_console();
    cprint("");
    cprint("  Wawity CLI — server install");
    cprint("  ------------------------------------------------");

    if !elevated() {
        cprint("  [X] Нужны права администратора. Запустите консоль от имени Administrator.");
        cprint("");
        return 1;
    }

    if args.iter().any(|a| {
        let low = a.to_lowercase();
        low == "--help" || low == "-h" || low == "/?"
    }) {
        cprint("  WawitySetup-CLI-serverinstall.exe [--dir <path>] [--purge]");
        cprint("  --dir    каталог установки");
        cprint("  --purge  удалить старые данные");
        cprint("");
        return 0;
    }

    std::env::set_var("WAWITY_SERVER_INSTALL", "1");

    let dir = arg_value(args, "--dir")
        .map(PathBuf::from)
        .unwrap_or_else(default_install_dir);
    let keep = !args.iter().any(|a| a.eq_ignore_ascii_case("--purge"));

    cprint(&format!("  Каталог: {}", dir.display()));
    cprint("");

    let printer = |p: f64, label: &str| {
        cprint(&format!("  [{:>3}%] {}", (p * 100.0).round() as i64, label));
    };

    match do_install(&dir, false, keep, "cli", &printer) {
        Ok(()) => {
            cprint("");
            cprint("  [OK] Wawity CLI установлен");
            cprint(&format!("  Бинарник: {}", dir.join(CLI_EXE).display()));
            cprint("  PATH обновлён, откройте новую консоль и введите: wawity");
            cprint("  Удаление: uninstall.exe --uninstall --silent");
            cprint("");
            0
        }
        Err(e) => {
            cprint("");
            cprint(&format!("  [X] Ошибка: {}", e));
            cprint("");
            1
        }
    }
}

#[link(name = "user32")]
extern "system" {
    fn SendMessageTimeoutW(
        hwnd: isize,
        msg: u32,
        wparam: usize,
        lparam: *const u16,
        flags: u32,
        timeout: u32,
        result: *mut usize,
    ) -> isize;
}

fn broadcast_env_change() {
    let payload: Vec<u16> = "Environment\0".encode_utf16().collect();
    let mut out: usize = 0;
    unsafe {
        SendMessageTimeoutW(0xFFFF, 0x001A, 0, payload.as_ptr(), 0x0002, 3000, &mut out);
    }
}
