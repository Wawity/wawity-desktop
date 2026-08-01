use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub kill_switch: bool,
    pub quantum_resistant: bool,
    pub bootstrap_dns: String,
    pub strict_route: bool,
    pub allow_insecure_tls: bool,
    pub tunnel_own_traffic: bool,
    pub dns_leak_guard: bool,
    pub bypass_apps: Vec<String>,
    pub entry_server: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            kill_switch: true,
            quantum_resistant: true,
            bootstrap_dns: "cloudflare".to_string(),
            strict_route: true,
            allow_insecure_tls: false,
            tunnel_own_traffic: true,
            dns_leak_guard: true,
            bypass_apps: Vec::new(),
            entry_server: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachedState {
    pub pid: Option<u32>,
    #[serde(default)]
    pub kill_switch: bool,
    pub exit_server: Option<String>,
    pub entry_server: Option<String>,
    pub always_on: bool,
    pub started_at: i64,
}

pub fn config_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_else(|_| PathBuf::from("."))
        });
    base.join("Wawity").join("cli")
}

fn ensure_dir() -> PathBuf {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    dir
}

fn read_json<T: for<'de> Deserialize<'de>>(path: PathBuf) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_subscriptions() -> Vec<Subscription> {
    let dir = ensure_dir();
    read_json(dir.join("subscriptions.json")).unwrap_or_default()
}

pub fn save_subscriptions(subs: &[Subscription]) -> Result<(), String> {
    let dir = ensure_dir();
    write_json(dir.join("subscriptions.json"), &subs.to_vec())
}

pub fn load_settings() -> Settings {
    let dir = ensure_dir();
    read_json(dir.join("settings.json")).unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let dir = ensure_dir();
    write_json(dir.join("settings.json"), settings)
}

pub fn load_detached() -> Option<DetachedState> {
    let dir = ensure_dir();
    read_json(dir.join("state.json"))
}

pub fn save_detached(state: &DetachedState) -> Result<(), String> {
    let dir = ensure_dir();
    write_json(dir.join("state.json"), state)
}

pub fn clear_detached() {
    let dir = ensure_dir();
    let _ = fs::remove_file(dir.join("state.json"));
}

pub fn new_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("sub_{:x}", now)
}
