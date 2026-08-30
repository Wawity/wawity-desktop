use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::AppHandle;

static HWID_ENABLED: AtomicBool = AtomicBool::new(true);
static SALT_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));
static CACHED_HWID: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn init(app: &AppHandle) {
    let base = app
        .path_resolver()
        .app_config_dir()
        .unwrap_or_else(std::env::temp_dir);
    let _ = std::fs::create_dir_all(&base);
    if let Ok(mut slot) = SALT_PATH.lock() {
        *slot = Some(base.join("device.salt"));
    }
    apply_headers();
}

#[tauri::command]
pub fn get_hwid() -> String {
    current_hwid()
}

#[tauri::command]
pub fn reset_hwid() -> String {
    let fresh = generate_salt();
    let _ = std::fs::write(salt_path(), &fresh);
    if let Ok(mut slot) = CACHED_HWID.lock() {
        *slot = None;
    }
    let id = current_hwid();
    apply_headers();
    id
}

#[tauri::command]
pub fn set_hwid_enabled(enabled: bool) {
    HWID_ENABLED.store(enabled, Ordering::Relaxed);
    apply_headers();
}

fn apply_headers() {
    if HWID_ENABLED.load(Ordering::Relaxed) {
        wawity_core::engine::set_extra_headers(device_headers());
    } else {
        wawity_core::engine::set_extra_headers(Vec::new());
    }
}

fn device_headers() -> Vec<(String, String)> {
    vec![
        ("x-hwid".to_string(), current_hwid()),
        ("x-device-os".to_string(), os_name()),
        ("x-ver-os".to_string(), os_version()),
        ("x-device-model".to_string(), device_model()),
    ]
}

fn current_hwid() -> String {
    if let Ok(slot) = CACHED_HWID.lock() {
        if let Some(existing) = slot.as_ref() {
            return existing.clone();
        }
    }
    let salt = load_or_create_salt();
    let machine = raw_machine_id();
    let mut hasher = Sha256::new();
    hasher.update(machine.as_bytes());
    hasher.update(b"::");
    hasher.update(salt.as_bytes());
    hasher.update(b"::wawity");
    let digest = hasher.finalize();
    let hex = digest
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>();
    if let Ok(mut slot) = CACHED_HWID.lock() {
        *slot = Some(hex.clone());
    }
    hex
}

fn salt_path() -> PathBuf {
    SALT_PATH
        .lock()
        .ok()
        .and_then(|slot| slot.clone())
        .unwrap_or_else(|| std::env::temp_dir().join("wawity_device.salt"))
}

fn load_or_create_salt() -> String {
    let path = salt_path();
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let trimmed = existing.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let fresh = generate_salt();
    let _ = std::fs::write(&path, &fresh);
    fresh
}

fn generate_salt() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|delta| delta.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id() as u128;
    let mixed = nanos
        .wrapping_mul(6364136223846793005)
        .wrapping_add(pid.wrapping_mul(1442695040888963407))
        .wrapping_add(0x9e3779b97f4a7c15);
    let mut hasher = Sha256::new();
    hasher.update(mixed.to_le_bytes());
    hasher.update(nanos.to_le_bytes());
    hasher.update(pid.to_le_bytes());
    let digest = hasher.finalize();
    digest
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}

fn os_name() -> String {
    match std::env::consts::OS {
        "windows" => "windows".to_string(),
        "macos" => "macos".to_string(),
        "linux" => "linux".to_string(),
        other => other.to_string(),
    }
}

fn os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Some(found) = windows_build() {
            return found;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(found) = command_output("sw_vers", &["-productVersion"]) {
            return found;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(release) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
            let trimmed = release.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    std::env::consts::ARCH.to_string()
}

fn device_model() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Ok(name) = std::env::var("COMPUTERNAME") {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(name) = std::env::var("HOSTNAME") {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Ok(name) = std::fs::read_to_string("/etc/hostname") {
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(found) = command_output("hostname", &[]) {
            return found;
        }
    }
    "Wawity Desktop".to_string()
}

fn raw_machine_id() -> String {
    #[cfg(target_os = "windows")]
    {
        if let Some(found) = windows_machine_guid() {
            return found;
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(found) = macos_platform_uuid() {
            return found;
        }
    }
    #[cfg(target_os = "linux")]
    {
        for path in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
            if let Ok(content) = std::fs::read_to_string(path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    format!("{}-{}", os_name(), device_model())
}

#[cfg(not(target_os = "windows"))]
fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = wawity_core::util::silent_command(program)
        .args(args)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

#[cfg(target_os = "windows")]
fn windows_machine_guid() -> Option<String> {
    let output = wawity_core::util::silent_command("reg")
        .args([
            "query",
            "HKLM\\SOFTWARE\\Microsoft\\Cryptography",
            "/v",
            "MachineGuid",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(idx) = line.find("REG_SZ") {
            let value = line[idx + 6..].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn windows_build() -> Option<String> {
    let output = wawity_core::util::silent_command("reg")
        .args([
            "query",
            "HKLM\\SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion",
            "/v",
            "CurrentBuild",
        ])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if let Some(idx) = line.find("REG_SZ") {
            let value = line[idx + 6..].trim();
            if !value.is_empty() {
                return Some(format!("Windows build {}", value));
            }
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn macos_platform_uuid() -> Option<String> {
    let output = wawity_core::util::silent_command("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains("IOPlatformUUID") {
            if let Some(idx) = line.find('=') {
                let value = line[idx + 1..].trim().trim_matches('"').trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}
