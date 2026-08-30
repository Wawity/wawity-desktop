#![allow(dead_code)]

use crate::config::{parse_all_from_subscription, parse_subscription, ConfigGenerator, RouteRuleSpec};
use crate::error::VpnError;
use crate::network::{RoutingManager, TunManager};
use crate::process::ProcessManager;
use crate::util::strip_unc_prefix;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use reqwest::blocking::Client;
use base64::{Engine as _, engine::general_purpose};

pub static GLOBAL_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .user_agent("v2rayN/6.23")
        .redirect(reqwest::redirect::Policy::limited(10))
        .connection_verbose(false)
        .tcp_keepalive(Duration::from_secs(90))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .build()
        .expect("Failed to build global HTTP client")
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VpnStatus {
    pub connected: bool,
    pub pid: Option<u32>,
    pub server: Option<String>,
    pub kill_switch: bool,
    pub interface: Option<String>,
    pub server_name: Option<String>,
    pub entry_server_name: Option<String>,
    pub multihop: bool,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
    pub speed_rx: f64,
    pub speed_tx: f64,
    pub always_on_locked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SavedServer {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedServer {
    pub name: String,
    pub url: String,
    pub protocol: String,
    pub server: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingTarget {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    pub host: String,
    pub port: u16,
    pub latency_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct TrayServerEntry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub country_code: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayServerInput {
    pub id: String,
    pub name: String,
    pub url: String,
    pub country_code: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SplitRules {
    pub mode: String,
    pub processes: Vec<String>,
    pub domains: Vec<String>,
    pub ips: Vec<String>,
}

fn split_config_for(handles: &ConnectionHandles, apps: Vec<String>) -> crate::config::SplitConfig {
    let rules = handles.split_rules.lock().unwrap().clone();
    crate::config::SplitConfig {
        apps,
        processes: rules.processes,
        domains: rules.domains,
        ips: rules.ips,
        mode: Some(crate::config::SplitMode::parse(&rules.mode)),
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct SelectorOptions {
    
    pub strategy: String,
    
    pub urls: Vec<String>,
}

impl Default for SelectorOptions {
    fn default() -> Self {
        Self { strategy: "select".to_string(), urls: Vec::new() }
    }
}

impl SelectorOptions {
    pub fn is_empty(&self) -> bool {
        self.urls.is_empty()
    }
}

pub struct AppState {
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub tun_manager: Arc<Mutex<TunManager>>,
    pub routing_manager: Arc<Mutex<RoutingManager>>,
    pub current_config: Arc<Mutex<Option<String>>>,
    pub saved_servers: Arc<Mutex<Vec<SavedServer>>>,
    pub servers_file: PathBuf,
    pub always_on_flag_file: PathBuf,
    pub connected_server_name: Arc<Mutex<Option<String>>>,
    pub connected_entry_server_name: Arc<Mutex<Option<String>>>,
    pub bypass_apps: Arc<Mutex<Vec<String>>>,
    pub always_on: Arc<Mutex<bool>>,
    pub current_exit_url: Arc<Mutex<Option<String>>>,
    pub current_entry_url: Arc<Mutex<Option<String>>>,
    pub current_quantum_resistant: Arc<Mutex<bool>>,
    pub privacy: Arc<Mutex<PrivacyOptions>>,
    pub split_rules: Arc<Mutex<SplitRules>>,
    pub selector: Arc<Mutex<SelectorOptions>>,
    pub tray_servers: Arc<Mutex<Vec<TrayServerEntry>>>,
    pub tray_selected_id: Arc<Mutex<Option<String>>>,
    pub default_kill_switch: Arc<Mutex<bool>>,
    pub default_quantum_resistant: Arc<Mutex<bool>>,
    pub tray_prev_stats: Arc<Mutex<(u64, u64, Instant)>>,
    pub launched_hidden: bool,
}

impl AppState {
    pub fn new() -> Result<Self, VpnError> {
        let process_manager = ProcessManager::new()?;
        let tun_manager = TunManager::new("wawity-tun0");
        let routing_manager = RoutingManager::new();
        let exe_dir = std::env::current_exe()
            .map_err(|e| VpnError::IoError(e.to_string()))?
            .parent()
            .ok_or_else(|| VpnError::Internal("Cannot get exe directory".into()))?
            .to_path_buf();
        let data_dir = exe_dir.join("data");
        fs::create_dir_all(&data_dir)?;
        let servers_file = data_dir.join("servers.json");
        let always_on_flag_file = data_dir.join("always_on.flag");
        let saved_servers = if servers_file.exists() {
            let content = fs::read_to_string(&servers_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };
        let persisted_always_on = always_on_flag_file.exists();
        Ok(Self {
            process_manager: Arc::new(Mutex::new(process_manager)),
            tun_manager: Arc::new(Mutex::new(tun_manager)),
            routing_manager: Arc::new(Mutex::new(routing_manager)),
            current_config: Arc::new(Mutex::new(None)),
            saved_servers: Arc::new(Mutex::new(saved_servers)),
            servers_file,
            always_on_flag_file,
            connected_server_name: Arc::new(Mutex::new(None)),
            connected_entry_server_name: Arc::new(Mutex::new(None)),
            bypass_apps: Arc::new(Mutex::new(Vec::new())),
            always_on: Arc::new(Mutex::new(persisted_always_on)),
            current_exit_url: Arc::new(Mutex::new(None)),
            current_entry_url: Arc::new(Mutex::new(None)),
            current_quantum_resistant: Arc::new(Mutex::new(false)),
            privacy: Arc::new(Mutex::new(PrivacyOptions::default())),
            split_rules: Arc::new(Mutex::new(SplitRules::default())),
        selector: Arc::new(Mutex::new(SelectorOptions::default())),
            tray_servers: Arc::new(Mutex::new(Vec::new())),
            tray_selected_id: Arc::new(Mutex::new(None)),
            default_kill_switch: Arc::new(Mutex::new(true)),
            default_quantum_resistant: Arc::new(Mutex::new(true)),
            tray_prev_stats: Arc::new(Mutex::new((0, 0, Instant::now()))),
            launched_hidden: false,
        })
    }

    pub fn save_servers_to_disk(&self) -> Result<(), VpnError> {
        let servers = self.saved_servers.lock().unwrap().clone();
        let json = serde_json::to_string_pretty(&servers)
            .map_err(|e| VpnError::JsonError(e.to_string()))?;
        fs::write(&self.servers_file, json)?;
        Ok(())
    }

    pub fn persist_always_on_flag(&self, enabled: bool) {
        if enabled {
            let _ = fs::write(&self.always_on_flag_file, b"1");
        } else {
            let _ = fs::remove_file(&self.always_on_flag_file);
        }
    }
}

#[derive(Clone)]
pub struct ConnectionHandles {
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub tun_manager: Arc<Mutex<TunManager>>,
    pub routing_manager: Arc<Mutex<RoutingManager>>,
    pub current_config: Arc<Mutex<Option<String>>>,
    pub connected_server_name: Arc<Mutex<Option<String>>>,
    pub connected_entry_server_name: Arc<Mutex<Option<String>>>,
    pub bypass_apps: Arc<Mutex<Vec<String>>>,
    pub always_on: Arc<Mutex<bool>>,
    pub current_exit_url: Arc<Mutex<Option<String>>>,
    pub current_entry_url: Arc<Mutex<Option<String>>>,
    pub current_quantum_resistant: Arc<Mutex<bool>>,
    pub privacy: Arc<Mutex<PrivacyOptions>>,
    pub split_rules: Arc<Mutex<SplitRules>>,
    pub selector: Arc<Mutex<SelectorOptions>>,
}

impl ConnectionHandles {
    pub fn from_state(state: &AppState) -> Self {
        Self {
            process_manager: Arc::clone(&state.process_manager),
            tun_manager: Arc::clone(&state.tun_manager),
            routing_manager: Arc::clone(&state.routing_manager),
            current_config: Arc::clone(&state.current_config),
            connected_server_name: Arc::clone(&state.connected_server_name),
            connected_entry_server_name: Arc::clone(&state.connected_entry_server_name),
            bypass_apps: Arc::clone(&state.bypass_apps),
            always_on: Arc::clone(&state.always_on),
            current_exit_url: Arc::clone(&state.current_exit_url),
            current_entry_url: Arc::clone(&state.current_entry_url),
            current_quantum_resistant: Arc::clone(&state.current_quantum_resistant),
            privacy: Arc::clone(&state.privacy),
            split_rules: Arc::clone(&state.split_rules),
            selector: Arc::clone(&state.selector),
        }
    }
}

pub static CLIENT_USER_AGENTS: &[&str] = &[
    "v2rayN/6.23",
    "clash-verge/1.6.6",
    "sing-box/1.8.0",
    "ClashX/1.95.1",
    "Shadowrocket/2086 CFNetwork/1474 Darwin/23.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
];

pub static EXTRA_HEADERS: Lazy<RwLock<Vec<(String, String)>>> =
    Lazy::new(|| RwLock::new(Vec::new()));

pub fn set_extra_headers(headers: Vec<(String, String)>) {
    if let Ok(mut guard) = EXTRA_HEADERS.write() {
        *guard = headers;
    }
}

pub fn extra_header_map() -> reqwest::header::HeaderMap {
    let mut map = reqwest::header::HeaderMap::new();
    if let Ok(guard) = EXTRA_HEADERS.read() {
        for (name, value) in guard.iter() {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_bytes(name.as_bytes()),
                reqwest::header::HeaderValue::from_str(value),
            ) {
                map.insert(header_name, header_value);
            }
        }
    }
    map
}

pub fn make_client_with_ua(timeout_secs: u64, ua: &str) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(ua)
        .default_headers(extra_header_map())
        .redirect(reqwest::redirect::Policy::limited(10))
        .connection_verbose(false)
        .tcp_keepalive(Duration::from_secs(90))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(10)
        .gzip(true)
        .deflate(true)
        .brotli(true)
        .build()
        .map_err(|e| e.to_string())
}

pub fn make_client(timeout_secs: u64) -> Result<Client, String> {
    make_client_with_ua(timeout_secs, CLIENT_USER_AGENTS[0])
}

pub struct FetchResult {
    pub body: String,
    pub content_type: String,
}

pub fn fetch_with_ua(url: &str, ua: &str) -> Result<FetchResult, String> {
    let client = make_client_with_ua(20, ua)?;
    let result = client
        .get(url)
        .header("Accept", "*/*")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .header("Connection", "keep-alive")
        .send();
    match result {
        Ok(resp) => {
            let status = resp.status();
            let content_type = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            if !status.is_success() {
                return Err(format!("HTTP {}", status.as_u16()));
            }
            match resp.text() {
                Ok(body) => Ok(FetchResult { body, content_type }),
                Err(e) => Err(format!("Read body: {}", e)),
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

pub fn fetch_with_fallback(url: &str) -> Result<FetchResult, String> {
    let mut last_err = String::from("Unknown error");
    for ua in CLIENT_USER_AGENTS {
        match fetch_with_ua(url, ua) {
            Ok(r) => return Ok(r),
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PrivacyOptions {
    pub strict_route: bool,
    pub allow_insecure_tls: bool,
    pub tunnel_own_traffic: bool,
    pub dns_leak_guard: bool,
    pub bootstrap_dns: String,
    pub dpi_profile: String,
    pub route_rules: Vec<RouteRuleSpec>,
    pub route_all: bool,
    
    pub dns_remote: String,
    
    pub dns_custom_doh: Option<String>,
    
    pub dns_block_ads: bool,
    pub dns_block_trackers: bool,
}

impl Default for PrivacyOptions {
    fn default() -> Self {
        Self {
            strict_route: true,
            allow_insecure_tls: false,
            tunnel_own_traffic: true,
            dns_leak_guard: true,
            bootstrap_dns: "cloudflare".to_string(),
            dpi_profile: "off".to_string(),
            route_rules: Vec::new(),
            route_all: true,
            dns_remote: "cloudflare".to_string(),
            dns_custom_doh: None,
            dns_block_ads: true,
            dns_block_trackers: true,
        }
    }
}

pub fn resolve_server_ips_bootstrap(host: &str, resolver: &str) -> Result<Vec<String>, String> {
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Ok(vec![host.to_string()]);
    }
    let preferred: &[&str] = match resolver {
        "google" => &["https://8.8.8.8/resolve", "https://8.8.4.4/resolve"],
        "quad9" => &["https://9.9.9.9:5053/dns-query", "https://149.112.112.112:5053/dns-query"],
        _ => &["https://1.1.1.1/dns-query", "https://1.0.0.1/dns-query"],
    };
    let mut endpoints: Vec<&str> = preferred.to_vec();
    for extra in [
        "https://1.1.1.1/dns-query",
        "https://8.8.8.8/resolve",
        "https://9.9.9.9:5053/dns-query",
    ] {
        if !endpoints.contains(&extra) {
            endpoints.push(extra);
        }
    }
    let client = make_client(5)?;
    let mut last_err = String::from("resolve failed");
    for ep in endpoints {
        let target = format!("{}?name={}&type=A", ep, host);
        match client.get(&target).header("Accept", "application/dns-json").send() {
            Ok(resp) if resp.status().is_success() => match resp.json::<serde_json::Value>() {
                Ok(v) => {
                    let ips: Vec<String> = v["Answer"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter(|a| a["type"].as_i64() == Some(1))
                                .filter_map(|a| a["data"].as_str().map(|d| d.to_string()))
                                .filter(|d| d.parse::<std::net::Ipv4Addr>().is_ok())
                                .collect()
                        })
                        .unwrap_or_default();
                    if !ips.is_empty() {
                        return Ok(ips);
                    }
                    last_err = format!("no A records for {}", host);
                }
                Err(e) => last_err = format!("dns json: {}", e),
            },
            Ok(resp) => last_err = format!("dns http {}", resp.status().as_u16()),
            Err(e) => last_err = format!("dns request: {}", e),
        }
    }
    match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:443", host)) {
        Ok(addrs) => {
            let mut ips: Vec<String> = Vec::new();
            for addr in addrs {
                if let std::net::IpAddr::V4(v4) = addr.ip() {
                    let value = v4.to_string();
                    if !ips.contains(&value) {
                        ips.push(value);
                    }
                }
            }
            if !ips.is_empty() {
                log::info!("bootstrap dns fell back to system resolver for {}", host);
                return Ok(ips);
            }
        }
        Err(e) => last_err = format!("{}; system resolver: {}", last_err, e),
    }
    Err(last_err)
}

pub fn pad_base64(s: &str) -> String {
    let rem = s.len() % 4;
    if rem == 0 { s.to_string() } else { format!("{}{}", s, "=".repeat(4 - rem)) }
}

pub fn try_decode_base64_variants(input: &str) -> Option<String> {
    let stripped = input.trim().replace(['\n', '\r', ' '], "");
    let padded = pad_base64(&stripped);
    let candidates: &[&dyn Fn(&str) -> Result<Vec<u8>, _>] = &[
        &|s: &str| general_purpose::STANDARD.decode(s),
        &|s: &str| general_purpose::URL_SAFE.decode(s),
        &|s: &str| general_purpose::URL_SAFE_NO_PAD.decode(s),
        &|s: &str| general_purpose::STANDARD_NO_PAD.decode(s),
    ];
    for decode_fn in candidates {
        if let Ok(bytes) = decode_fn(&padded) {
            if let Ok(text) = String::from_utf8(bytes) {
                if text.lines().any(|l| is_proxy_line(l.trim())) {
                    return Some(text);
                }
            }
        }
        if let Ok(bytes) = decode_fn(input.trim()) {
            if let Ok(text) = String::from_utf8(bytes) {
                if text.lines().any(|l| is_proxy_line(l.trim())) {
                    return Some(text);
                }
            }
        }
    }
    None
}

pub fn is_proxy_line(s: &str) -> bool {
    matches!(
        s.split("://").next().unwrap_or(""),
        "vless" | "vmess" | "trojan" | "ss" | "hysteria2" | "hy2" | "tuic"
    )
}

pub fn extract_fragment(url_str: &str) -> String {
    if let Some(pos) = url_str.rfind('#') {
        let frag = &url_str[pos + 1..];
        urlencoding::decode(frag)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| frag.to_string())
    } else {
        String::new()
    }
}

pub fn extract_proxies_from_clash_yaml(yaml: &str) -> Vec<ParsedServer> {
    let mut results = Vec::new();
    let mut in_proxies = false;
    let mut current: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed == "proxies:" {
            in_proxies = true;
            continue;
        }
        if in_proxies && trimmed.starts_with("- name:") {
            if !current.is_empty() {
                if let Some(srv) = clash_entry_to_parsed(&current) {
                    results.push(srv);
                }
            }
            current.clear();
            let val = trimmed.trim_start_matches("- name:").trim().trim_matches('"').to_string();
            current.insert("name".to_string(), val);
            continue;
        }
        if in_proxies && !trimmed.is_empty() && !trimmed.starts_with('-') {
            if let Some((k, v)) = trimmed.split_once(':') {
                current.insert(
                    k.trim().to_string(),
                    v.trim().trim_matches('"').to_string(),
                );
            }
            continue;
        }
        if in_proxies && trimmed.ends_with(':') && !trimmed.starts_with(' ') && trimmed != "proxies:" {
            if !current.is_empty() {
                if let Some(srv) = clash_entry_to_parsed(&current) {
                    results.push(srv);
                }
                current.clear();
            }
            in_proxies = false;
        }
    }
    if !current.is_empty() {
        if let Some(srv) = clash_entry_to_parsed(&current) {
            results.push(srv);
        }
    }
    results
}

pub fn clash_entry_to_parsed(m: &std::collections::HashMap<String, String>) -> Option<ParsedServer> {
    let proxy_type = m.get("type")?.to_lowercase();
    let server = m.get("server")?.clone();
    let name = m.get("name").cloned().unwrap_or_else(|| server.clone());
    let protocol = match proxy_type.as_str() {
        "vless" => "vless",
        "vmess" => "vmess",
        "trojan" => "trojan",
        "ss" | "shadowsocks" => "shadowsocks",
        "hysteria2" | "hy2" => "hysteria2",
        "tuic" => "tuic",
        _ => return None,
    };
    Some(ParsedServer {
        name,
        url: String::new(),
        protocol: protocol.to_string(),
        server,
    })
}

pub fn extract_proxies_from_singbox_json(json_str: &str) -> Vec<ParsedServer> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) else {
        return vec![];
    };
    let Some(outbounds) = val.get("outbounds").and_then(|v| v.as_array()) else {
        return vec![];
    };
    let skip_types = ["direct", "block", "dns", "selector", "urltest"];
    outbounds
        .iter()
        .filter_map(|ob| {
            let t = ob.get("type")?.as_str()?;
            if skip_types.contains(&t) { return None; }
            let server = ob.get("server")?.as_str()?.to_string();
            let tag = ob.get("tag").and_then(|v| v.as_str()).unwrap_or(&server).to_string();
            Some(ParsedServer {
                name: tag,
                url: String::new(),
                protocol: t.to_string(),
                server,
            })
        })
        .collect()
}

pub fn proxies_from(body: &str, content_type: &str) -> Vec<ParsedServer> {
    let trimmed = body.trim();
    if trimmed.is_empty() { return vec![]; }
    if is_proxy_line(trimmed.lines().next().unwrap_or("").trim()) {
        let entries = parse_all_from_subscription(trimmed);
        if !entries.is_empty() {
            return entries.into_iter().map(|(cfg, name)| ParsedServer {
                name: if name.is_empty() {
                    format!("{} — {}", cfg.protocol_name().to_uppercase(), cfg.server_host())
                } else { name },
                url: String::new(),
                protocol: cfg.protocol_name().to_string(),
                server: cfg.server_host().to_string(),
            }).collect();
        }
    }
    if content_type.contains("json") || trimmed.starts_with('{') {
        let r = extract_proxies_from_singbox_json(trimmed);
        if !r.is_empty() { return r; }
    }
    if content_type.contains("yaml") || content_type.contains("yml")
        || trimmed.contains("proxies:")
        || trimmed.starts_with("mixed-port:") || trimmed.starts_with("port:")
    {
        let r = extract_proxies_from_clash_yaml(trimmed);
        if !r.is_empty() { return r; }
    }
    if let Some(decoded) = try_decode_base64_variants(trimmed) {
        if is_proxy_line(decoded.lines().next().unwrap_or("").trim()) {
            let entries = parse_all_from_subscription(&decoded);
            if !entries.is_empty() {
                return entries.into_iter().map(|(cfg, name)| ParsedServer {
                    name: if name.is_empty() {
                        format!("{} — {}", cfg.protocol_name().to_uppercase(), cfg.server_host())
                    } else { name },
                    url: String::new(),
                    protocol: cfg.protocol_name().to_string(),
                    server: cfg.server_host().to_string(),
                }).collect();
            }
        }
        if decoded.contains("proxies:") {
            let r = extract_proxies_from_clash_yaml(&decoded);
            if !r.is_empty() { return r; }
        }
        if decoded.trim_start().starts_with('{') {
            let r = extract_proxies_from_singbox_json(&decoded);
            if !r.is_empty() { return r; }
        }
    }
    vec![]
}

pub fn raw_proxies_from(body: &str) -> Vec<ParsedServer> {
    let trimmed = body.trim();
    if trimmed.is_empty() { return vec![]; }
    let mut content = trimmed.to_string();
    if !is_proxy_line(trimmed.lines().next().unwrap_or("").trim()) {
        if let Some(decoded) = try_decode_base64_variants(trimmed) {
            content = decoded;
        }
    }
    let mut results = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || !is_proxy_line(line) { continue; }
        if let Ok(cfg) = parse_subscription(line) {
            let raw_name = extract_fragment(line);
            let name = if raw_name.is_empty() {
                format!("{} — {}", cfg.protocol_name().to_uppercase(), cfg.server_host())
            } else { raw_name };
            results.push(ParsedServer {
                name,
                url: line.to_string(),
                protocol: cfg.protocol_name().to_string(),
                server: cfg.server_host().to_string(),
            });
        }
    }
    results
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IpApiEntry {
    query: String,
    status: String,
    #[serde(rename = "countryCode")]
    country_code: Option<String>,
}

pub fn is_ip(s: &str) -> bool {
    s.parse::<std::net::IpAddr>().is_ok()
}

pub fn geolocate_servers_blocking(hosts: Vec<String>) -> Result<Vec<Option<String>>, String> {
    if hosts.is_empty() { return Ok(vec![]); }
    let resolved: Vec<String> = hosts.iter().map(|h| {
        if is_ip(h) { h.clone() } else {
            std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:443", h) as &str)
                .ok()
                .and_then(|mut it| it.next())
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| h.clone())
        }
    }).collect();
    let mut results: Vec<Option<String>> = Vec::with_capacity(resolved.len());
    for chunk in resolved.chunks(100) {
        let payload: Vec<serde_json::Value> = chunk
            .iter()
            .map(|h| serde_json::json!({ "query": h }))
            .collect();
        match GLOBAL_CLIENT
            .post("http://ip-api.com/batch?fields=status,countryCode,query")
            .json(&payload)
            .send()
        {
            Ok(r) if r.status().is_success() => {
                match r.json::<Vec<IpApiEntry>>() {
                    Ok(entries) => {
                        for entry in &entries {
                            results.push(if entry.status == "success" {
                                entry.country_code.as_ref().map(|c| c.to_lowercase())
                            } else { None });
                        }
                    }
                    Err(_) => { for _ in chunk { results.push(None); } }
                }
            }
            _ => { for _ in chunk { results.push(None); } }
        }
    }
    Ok(results)
}

pub fn resolve_ruleset_paths() -> (Option<PathBuf>, Option<PathBuf>) {
    let exe_dir = match std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
        Some(d) => d,
        None => return (None, None),
    };
    let candidate_dirs = [
        exe_dir.join("rulesets"),
        exe_dir.join("resources").join("rulesets"),
    ];
    let mut ads_path = None;
    let mut private_path = None;
    for dir in &candidate_dirs {
        if ads_path.is_none() {
            let p = dir.join("geosite-category-ads-all.srs");
            if p.exists() { ads_path = Some(p); }
        }
        if private_path.is_none() {
            let p = dir.join("geosite-private.srs");
            if p.exists() { private_path = Some(p); }
        }
    }
    (ads_path, private_path)
}

pub fn resolve_own_exe_path() -> Result<String, String> {
    let raw = std::env::current_exe()
        .map_err(|e| format!("cannot resolve own executable path: {}", e))?;
    let canonical = raw.canonicalize().unwrap_or(raw);
    Ok(strip_unc_prefix(canonical).to_string_lossy().to_string())
}

pub fn survived_startup(handles: &ConnectionHandles, timeout_ms: u64) -> bool {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        thread::sleep(Duration::from_millis(80));
        if !handles.process_manager.lock().unwrap().is_running() {
            return false;
        }
        if Instant::now() >= deadline {
            return true;
        }
    }
}

pub fn pin_dns_to_tunnel() {
    thread::spawn(|| {
        let mut last_error = String::from("tunnel adapter never appeared");
        let mut native_failure_logged = false;
        for attempt in 0..24 {
            match crate::network::netinfo::set_interface_dns("wawity-tun0", "172.19.0.2") {
                Ok(()) => {
                    crate::util::net_debug_log(&format!(
                        "dns pinned to 172.19.0.2 natively, attempt {}",
                        attempt + 1
                    ));
                    return;
                }
                Err(e) => {
                    last_error = e.to_string();
                    let adapter_present = !last_error.contains("not found");
                    if adapter_present {
                        if !native_failure_logged {
                            native_failure_logged = true;
                            crate::util::net_debug_log(&format!(
                                "native dns pin failed, trying netsh fallback: {}",
                                last_error
                            ));
                        }
                        let outcome = crate::util::silent_command("netsh")
                            .args([
                                "interface",
                                "ipv4",
                                "set",
                                "dnsservers",
                                "name=wawity-tun0",
                                "source=static",
                                "address=172.19.0.2",
                                "register=none",
                                "validate=no",
                            ])
                            .status();
                        if outcome.as_ref().map(|s| s.success()).unwrap_or(false) {
                            crate::util::net_debug_log(&format!(
                                "dns pinned to 172.19.0.2 via netsh, attempt {}",
                                attempt + 1
                            ));
                            return;
                        }
                        crate::util::net_debug_log(&format!(
                            "netsh dns fallback attempt {} failed: {:?}",
                            attempt + 1,
                            outcome
                        ));
                    }
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
        crate::util::net_debug_log(&format!("dns pin gave up: {}", last_error));
    });
}

pub fn resolve_isp_dns(default_iface: &Option<String>) -> Option<String> {
    let found = default_iface.as_ref().and_then(|alias| {
        crate::network::netinfo::physical_dns_servers(alias)
            .into_iter()
            .next()
    });
    match &found {
        Some(ip) => crate::util::net_debug_log(&format!("bootstrap resolver: isp dns {}", ip)),
        None => crate::util::net_debug_log(
            "bootstrap resolver: isp dns not found, falling back to doh bootstrap",
        ),
    }
    found
}

pub fn verify_tunnel_ready(handles: &ConnectionHandles, total: Duration) -> Result<Option<u64>, String> {
    let deadline = Instant::now() + total;
    thread::sleep(Duration::from_millis(60));
    if !handles.process_manager.lock().unwrap().is_running() {
        return Err("sing-box exited right after start".into());
    }
    let mut adapter_seen = false;
    while Instant::now() < deadline {
        if crate::network::netinfo::wintun_adapter_present() {
            adapter_seen = true;
            break;
        }
        if !handles.process_manager.lock().unwrap().is_running() {
            return Err("sing-box died before the tun adapter appeared".into());
        }
        thread::sleep(Duration::from_millis(50));
    }
    if !adapter_seen {
        return Err(format!(
            "tun adapter never appeared, up interfaces: {}",
            crate::network::netinfo::adapter_summary()
        ));
    }
    let mut last_probe_error = String::new();
    let mut backoff = Duration::from_millis(80);
    loop {
        if !handles.process_manager.lock().unwrap().is_running() {
            return Err("sing-box died during traffic verification".into());
        }
        match probe_socks() {
            Ok(latency) => return Ok(Some(latency)),
            Err(reason) => {
                if reason != last_probe_error {
                    crate::util::net_debug_log(&format!("verify: probe failed: {}", reason));
                    last_probe_error = reason;
                }
            }
        }
        if Instant::now() >= deadline {
            crate::util::net_debug_log(
                "verify: process alive and adapter up, probe inconclusive, accepting session",
            );
            return Ok(None);
        }
        thread::sleep(backoff);
        backoff = (backoff * 2).min(Duration::from_millis(500));
    }
}

pub fn full_teardown(handles: &ConnectionHandles, keep_locked_down: bool) {
    handles.routing_manager.lock().unwrap().disable_dns_leak_guard();
    let _ = handles.process_manager.lock().unwrap().stop();
    let _ = handles.routing_manager.lock().unwrap().end_connection(keep_locked_down);
    let _ = handles.tun_manager.lock().unwrap().teardown_interface();
    *handles.current_config.lock().unwrap() = None;
    *handles.connected_server_name.lock().unwrap() = None;
    *handles.connected_entry_server_name.lock().unwrap() = None;
    *handles.current_exit_url.lock().unwrap() = None;
    *handles.current_entry_url.lock().unwrap() = None;
}

pub static NETWORK_OP_LOCK: Mutex<()> = Mutex::new(());

pub fn acquire_network_op(op: &str) -> Result<std::sync::MutexGuard<'static, ()>, String> {
    use std::sync::TryLockError;
    match NETWORK_OP_LOCK.try_lock() {
        Ok(guard) => Ok(guard),
        Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
        Err(TryLockError::WouldBlock) => {
            crate::util::net_debug_log(&format!(
                "{}: rejected, another network operation is still running",
                op
            ));
            Err("Another network operation is still running, wait a few seconds and retry".into())
        }
    }
}

pub fn restore_always_on(handles: &ConnectionHandles) {
    if *handles.always_on.lock().unwrap() {
        if let Ok(app_exe_path) = resolve_own_exe_path() {
            let _ = handles.routing_manager.lock().unwrap().enable_always_on(&app_exe_path);
        }
    }
}

pub fn start_session(
    sub_url: &str,
    entry_sub_url: Option<String>,
    server_name: Option<String>,
    entry_server_name: Option<String>,
    kill_switch: bool,
    bypass_apps: Vec<String>,
    quantum_resistant: bool,
    handles: &ConnectionHandles,
) -> Result<(), String> {
    let _network_op_guard = acquire_network_op("connect")?;
    if handles.process_manager.lock().unwrap().is_running() {
        return Err("VPN already connected, disconnect first".into());
    }
    let exit_config = parse_subscription(sub_url)
        .map_err(|e| format!("Exit config parse: {}", e))?;
    let entry_config = match &entry_sub_url {
        Some(url) if !url.trim().is_empty() => {
            Some(parse_subscription(url).map_err(|e| format!("Entry config parse: {}", e))?)
        }
        _ => None,
    };
    if let Some(entry) = &entry_config {
        if entry.server_host() == exit_config.server_host() {
            return Err("Entry and exit servers must be different for multi-hop".into());
        }
    }
    let binary_path = handles.process_manager.lock().unwrap().binary_path_string();
    let app_exe_path = resolve_own_exe_path()?;
    let (ads_ruleset_path, private_ruleset_path) = resolve_ruleset_paths();
    let always_on_flag = *handles.always_on.lock().unwrap();
    let want_lockdown = kill_switch || always_on_flag;
    let default_iface = crate::util::get_default_physical_interface();
    let privacy = handles.privacy.lock().unwrap().clone();
    let generator = ConfigGenerator::new(
        want_lockdown,
        split_config_for(handles, bypass_apps.clone()),
        quantum_resistant,
        binary_path.clone(),
        app_exe_path.clone(),
    )
    .with_local_rulesets(ads_ruleset_path, private_ruleset_path)
    .with_default_interface(default_iface.clone())
    .with_padding(true)
    .with_dpi(&privacy.dpi_profile)
    .with_privacy(
        privacy.strict_route,
        privacy.allow_insecure_tls,
        privacy.tunnel_own_traffic,
        &privacy.bootstrap_dns,
    )
    .with_dns_center(&privacy.dns_remote, privacy.dns_custom_doh.as_deref(), privacy.dns_block_ads || privacy.dns_block_trackers)
    .with_routing(privacy.route_rules.clone(), privacy.route_all)
    .with_system_dns(resolve_isp_dns(&default_iface))
    .with_main_selector(handles.selector.lock().unwrap().clone());
    let config_json = generator
        .to_json(&exit_config, entry_config.as_ref())
        .map_err(|e| format!("Config generate: {}", e))?;
    if let Err(e) = handles.tun_manager.lock().unwrap().setup_interface() {
        return Err(format!("TUN setup failed: {}", e));
    }
    let staged_server_ips = if want_lockdown {
        let direct_host = if let Some(entry) = &entry_config {
            entry.server_host()
        } else {
            exit_config.server_host()
        };
        match resolve_server_ips_bootstrap(&direct_host, &privacy.bootstrap_dns) {
            Ok(ips) => Some(ips),
            Err(e) => {
                let _ = handles.tun_manager.lock().unwrap().teardown_interface();
                return Err(format!("Server address resolve failed: {}", e));
            }
        }
    } else {
        None
    };
    let was_always_on_idle = handles.routing_manager.lock().unwrap().is_always_on_active()
        && !handles.routing_manager.lock().unwrap().is_kill_switch_active();
    if want_lockdown {
        if let Err(e) = handles.routing_manager.lock().unwrap()
            .stage_exceptions(&binary_path, &app_exe_path)
        {
            let _ = handles.tun_manager.lock().unwrap().teardown_interface();
            return Err(format!("Firewall rule setup failed: {}", e));
        }
        if !bypass_apps.is_empty() {
            if let Err(e) = handles.routing_manager.lock().unwrap()
                .update_bypass_rules(&bypass_apps, crate::config::SplitMode::parse(&handles.split_rules.lock().unwrap().mode).is_include())
            {
                handles.routing_manager.lock().unwrap().abort_staged_connection(was_always_on_idle);
                let _ = handles.tun_manager.lock().unwrap().teardown_interface();
                return Err(format!("Bypass firewall rules failed: {}", e));
            }
        }
        if let Err(e) = handles.routing_manager.lock().unwrap()
            .allow_server_ips(staged_server_ips.as_deref().unwrap_or(&[]))
        {
            handles.routing_manager.lock().unwrap().abort_staged_connection(was_always_on_idle);
            let _ = handles.tun_manager.lock().unwrap().teardown_interface();
            return Err(format!("Server firewall rule failed: {}", e));
        }
    }
    *handles.connected_server_name.lock().unwrap() = server_name;
    *handles.connected_entry_server_name.lock().unwrap() =
        if entry_config.is_some() { entry_server_name } else { None };
    if let Err(e) = handles.process_manager.lock().unwrap().start(&config_json) {
        if want_lockdown {
            handles.routing_manager.lock().unwrap().abort_staged_connection(was_always_on_idle);
        }
        let _ = handles.tun_manager.lock().unwrap().teardown_interface();
        *handles.connected_server_name.lock().unwrap() = None;
        *handles.connected_entry_server_name.lock().unwrap() = None;
        return Err(format!("Failed to start sing-box: {}", e));
    }
    if !survived_startup(handles, 1200) {
        if want_lockdown {
            handles.routing_manager.lock().unwrap().abort_staged_connection(was_always_on_idle);
        }
        let _ = handles.tun_manager.lock().unwrap().teardown_interface();
        *handles.connected_server_name.lock().unwrap() = None;
        *handles.connected_entry_server_name.lock().unwrap() = None;
        let captured = handles.process_manager.lock().unwrap().recent_output(80);
        let log_tail = fs::read_to_string(std::env::temp_dir().join("wawity.log"))
            .map(|c| c.lines().rev().take(20).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("\n"))
            .unwrap_or_default();
        let diagnostic = if !captured.trim().is_empty() { captured }
        else if !log_tail.trim().is_empty() { log_tail }
        else { "sing-box produced no output. Check binary permissions and wintun.dll.".to_string() };
        return Err(format!("sing-box terminated during startup.\n{}", diagnostic));
    }
    pin_dns_to_tunnel();
    if privacy.dns_leak_guard {
        if let Err(e) = handles.routing_manager.lock().unwrap().enable_dns_leak_guard() {
            log::warn!("dns leak guard failed: {}", e);
        }
    }
    if want_lockdown {
        if let Err(e) = handles.routing_manager.lock().unwrap().commit_connection() {
            let _ = handles.process_manager.lock().unwrap().stop();
            handles.routing_manager.lock().unwrap().abort_staged_connection(was_always_on_idle);
            let _ = handles.tun_manager.lock().unwrap().teardown_interface();
            *handles.connected_server_name.lock().unwrap() = None;
            *handles.connected_entry_server_name.lock().unwrap() = None;
            return Err(format!("Firewall lockdown activation failed: {}", e));
        }
    }
    *handles.current_config.lock().unwrap() = Some(config_json);
    *handles.bypass_apps.lock().unwrap() = bypass_apps;
    *handles.current_exit_url.lock().unwrap() = Some(sub_url.to_string());
    *handles.current_entry_url.lock().unwrap() = entry_sub_url;
    *handles.current_quantum_resistant.lock().unwrap() = quantum_resistant;
    Ok(())
}

pub fn stop_session(handles: &ConnectionHandles) -> Result<(), String> {
    let _network_op_guard = acquire_network_op("disconnect")?;
    handles.routing_manager.lock().unwrap().disable_dns_leak_guard();
    let mut errors = Vec::new();
    match handles.process_manager.lock().unwrap().stop() {
        Ok(()) | Err(VpnError::NotRunning) => {}
        Err(e) => errors.push(format!("process stop: {}", e)),
    }
    let always_on_flag = *handles.always_on.lock().unwrap();
    if let Err(e) = handles.routing_manager.lock().unwrap().end_connection(always_on_flag) {
        errors.push(format!("firewall teardown: {}", e));
    }
    restore_always_on(handles);
    if let Err(e) = handles.tun_manager.lock().unwrap().teardown_interface() {
        errors.push(format!("tun teardown: {}", e));
    }
    *handles.current_config.lock().unwrap() = None;
    *handles.connected_server_name.lock().unwrap() = None;
    *handles.connected_entry_server_name.lock().unwrap() = None;
    *handles.current_exit_url.lock().unwrap() = None;
    *handles.current_entry_url.lock().unwrap() = None;
    if errors.is_empty() { Ok(()) } else { Err(errors.join("; ")) }
}

pub fn switch_session(
    new_sub_url: &str,
    entry_sub_url: Option<String>,
    new_server_name: Option<String>,
    entry_server_name: Option<String>,
    bypass_apps: Vec<String>,
    quantum_resistant: bool,
    handles: &ConnectionHandles,
) -> Result<(), String> {
    let _network_op_guard = acquire_network_op("switch")?;
    crate::util::net_debug_log("switch: requested");
    if !handles.process_manager.lock().unwrap().is_running() {
        return Err("VPN is not connected".into());
    }
    let exit_config = parse_subscription(new_sub_url)
        .map_err(|e| format!("Exit config parse: {}", e))?;
    let entry_config = match &entry_sub_url {
        Some(url) if !url.trim().is_empty() => {
            Some(parse_subscription(url).map_err(|e| format!("Entry config parse: {}", e))?)
        }
        _ => None,
    };
    if let Some(entry) = &entry_config {
        if entry.server_host() == exit_config.server_host() {
            return Err("Entry and exit servers must be different for multi-hop".into());
        }
    }
    let is_locked_down = handles.routing_manager.lock().unwrap().is_kill_switch_active();
    let always_on_flag = *handles.always_on.lock().unwrap();
    let binary_path = handles.process_manager.lock().unwrap().binary_path_string();
    let app_exe_path = resolve_own_exe_path()?;
    let (ads_ruleset_path, private_ruleset_path) = resolve_ruleset_paths();
    let privacy = handles.privacy.lock().unwrap().clone();
    let direct_host = if let Some(entry) = &entry_config {
        entry.server_host()
    } else {
        exit_config.server_host()
    };
    let default_iface = crate::util::get_default_physical_interface();
    crate::util::net_debug_log(&format!(
        "switch: default interface = {}",
        default_iface.clone().unwrap_or_else(|| "none, sing-box auto-detect".into())
    ));
    let config_json = ConfigGenerator::new(
        is_locked_down,
        split_config_for(handles, bypass_apps.clone()),
        quantum_resistant,
        binary_path.clone(),
        app_exe_path.clone(),
    )
    .with_local_rulesets(ads_ruleset_path, private_ruleset_path)
    .with_default_interface(default_iface.clone())
    .with_padding(true)
    .with_dpi(&privacy.dpi_profile)
    .with_privacy(
        privacy.strict_route,
        privacy.allow_insecure_tls,
        privacy.tunnel_own_traffic,
        &privacy.bootstrap_dns,
    )
    .with_dns_center(&privacy.dns_remote, privacy.dns_custom_doh.as_deref(), privacy.dns_block_ads || privacy.dns_block_trackers)
    .with_routing(privacy.route_rules.clone(), privacy.route_all)
    .with_system_dns(resolve_isp_dns(&default_iface))
    .with_main_selector(handles.selector.lock().unwrap().clone())
    .to_json(&exit_config, entry_config.as_ref())
    .map_err(|e| {
        crate::util::net_debug_log(&format!("switch: config generate failed: {}", e));
        format!("Config generate failed, current session kept intact: {}", e)
    })?;
    if is_locked_down {
        if let Err(e) = handles.routing_manager.lock().unwrap()
            .allow_server_endpoint(&direct_host, &privacy.bootstrap_dns)
        {
            return Err(format!(
                "Server firewall rule failed, keeping current session: {}",
                e
            ));
        }
    } else {
        let mut routing = handles.routing_manager.lock().unwrap();
        if let Err(e) = routing.stage_exceptions(&binary_path, &app_exe_path) {
            // Kill switch is off: a broken firewall (e.g. service disabled) must
            // not block switching — proceed without the mid-switch leak guard,
            // same tolerance as the connect path.
            log::warn!("switch leak guard unavailable, continuing without it: {}", e);
            crate::util::net_debug_log(&format!(
                "switch guard: stage failed, continuing without leak guard: {}",
                e
            ));
        } else {
            if let Err(e) = routing.allow_server_endpoint(&direct_host, &privacy.bootstrap_dns) {
                crate::util::net_debug_log(&format!(
                    "switch guard: server endpoint rule skipped: {}",
                    e
                ));
            }
            if let Err(e) = routing.commit_connection() {
                routing.abort_staged_connection(false);
                log::warn!("switch leak guard lock failed, continuing without it: {}", e);
                crate::util::net_debug_log(&format!(
                    "switch guard: commit failed, continuing without leak guard: {}",
                    e
                ));
            } else {
                crate::util::net_debug_log("switch guard: leak protection engaged for the switch window");
            }
        }
        drop(routing);
    }
    let old_server_name = handles.connected_server_name.lock().unwrap().clone();
    let old_entry_name = handles.connected_entry_server_name.lock().unwrap().clone();
    crate::util::net_debug_log("switch: stopping current session");
    if let Err(e) = handles.process_manager.lock().unwrap().stop_fast() {
        log::warn!("stop during switch: {}", e);
        crate::util::net_debug_log(&format!("switch: stop reported error: {}", e));
    }
    if !crate::util::wait_for_wintun_teardown(Duration::from_secs(8)) {
        crate::util::net_debug_log(&format!(
            "switch: old tun adapter stuck, up interfaces: {}",
            crate::network::netinfo::adapter_summary()
        ));
        handles.process_manager.lock().unwrap().kill_orphans();
        let _ = handles.tun_manager.lock().unwrap().teardown_interface();
        if !crate::util::wait_for_wintun_teardown(Duration::from_secs(4)) {
            crate::util::net_debug_log("switch: old tun adapter still present, continuing anyway");
        }
    }
    crate::util::net_debug_log("switch: old session down, starting new one");
    *handles.connected_server_name.lock().unwrap() = new_server_name;
    *handles.connected_entry_server_name.lock().unwrap() =
        if entry_config.is_some() { entry_server_name } else { None };
    let mut attempt = 1u32;
    loop {
        crate::util::net_debug_log(&format!("switch: starting sing-box, attempt {}", attempt));
        if let Err(e) = handles.process_manager.lock().unwrap().start(&config_json) {
            crate::util::net_debug_log(&format!("switch: sing-box start failed: {}", e));
            *handles.connected_server_name.lock().unwrap() = old_server_name.clone();
            *handles.connected_entry_server_name.lock().unwrap() = old_entry_name.clone();
            full_teardown(handles, always_on_flag);
            restore_always_on(handles);
            return Err(format!(
                "Failed to start sing-box with new server, session terminated for safety: {}",
                e
            ));
        }
        pin_dns_to_tunnel();
        match verify_tunnel_ready(handles, Duration::from_secs(8)) {
            Ok(Some(latency)) => {
                crate::util::net_debug_log(&format!(
                    "switch: tunnel verified on attempt {}, first probe {} ms",
                    attempt, latency
                ));
                break;
            }
            Ok(None) => {
                crate::util::net_debug_log(&format!(
                    "switch: tunnel up on attempt {}, probe inconclusive, keeping session",
                    attempt
                ));
                break;
            }
            Err(reason) => {
                let captured = handles.process_manager.lock().unwrap().recent_output(80);
                crate::util::net_debug_log(&format!(
                    "switch: verification failed on attempt {}: {}",
                    attempt, reason
                ));
                let _ = handles.process_manager.lock().unwrap().stop();
                let _ = crate::util::wait_for_wintun_teardown(Duration::from_secs(6));
                if attempt >= 2 {
                    *handles.connected_server_name.lock().unwrap() = old_server_name.clone();
                    *handles.connected_entry_server_name.lock().unwrap() = old_entry_name.clone();
                    full_teardown(handles, always_on_flag);
                    restore_always_on(handles);
                    return Err(format!(
                        "New server failed live tunnel verification twice: {}. Session closed for safety.\n{}",
                        reason, captured
                    ));
                }
                attempt += 1;
                thread::sleep(Duration::from_millis(150));
            }
        }
    }
    if privacy.dns_leak_guard {
        if let Err(e) = handles.routing_manager.lock().unwrap().enable_dns_leak_guard() {
            log::warn!("dns leak guard failed: {}", e);
        }
    }
    if is_locked_down {
        if let Err(e) = handles.routing_manager.lock().unwrap().update_bypass_rules(&bypass_apps, crate::config::SplitMode::parse(&handles.split_rules.lock().unwrap().mode).is_include()) {
            log::warn!("Bypass firewall rule refresh during switch failed: {}", e);
        }
    } else {
        if let Err(e) = handles.routing_manager.lock().unwrap().end_connection(false) {
            log::warn!("switch guard release failed: {}", e);
        }
        crate::util::net_debug_log("switch guard: leak protection released, tunnel is live");
    }
    *handles.current_config.lock().unwrap() = Some(config_json);
    *handles.bypass_apps.lock().unwrap() = bypass_apps;
    *handles.current_exit_url.lock().unwrap() = Some(new_sub_url.to_string());
    *handles.current_entry_url.lock().unwrap() = entry_sub_url;
    *handles.current_quantum_resistant.lock().unwrap() = quantum_resistant;
    crate::util::net_debug_log("switch: completed successfully");
    Ok(())
}

pub fn reload_bypass_apps(
    new_paths: Vec<String>,
    handles: &ConnectionHandles,
) -> Result<(), String> {
    if !handles.process_manager.lock().unwrap().is_running() {
        *handles.bypass_apps.lock().unwrap() = new_paths;
        return Ok(());
    }
    let is_locked_down = handles.routing_manager.lock().unwrap().is_kill_switch_active();
    let always_on_flag = *handles.always_on.lock().unwrap();
    let server_name = handles.connected_server_name.lock().unwrap().clone();
    let entry_server_name = handles.connected_entry_server_name.lock().unwrap().clone();
    let exit_url = handles.current_exit_url.lock().unwrap().clone()
        .ok_or_else(|| "No active exit server cached, reconnect required".to_string())?;
    let entry_url = handles.current_entry_url.lock().unwrap().clone();
    let quantum_resistant = *handles.current_quantum_resistant.lock().unwrap();
    let exit_config = parse_subscription(&exit_url)
        .map_err(|e| format!("Cannot re-parse exit config: {}", e))?;
    let entry_config = match &entry_url {
        Some(url) if !url.trim().is_empty() => parse_subscription(url).ok(),
        _ => None,
    };
    let binary_path = handles.process_manager.lock().unwrap().binary_path_string();
    let app_exe_path = resolve_own_exe_path()?;
    let (ads_ruleset_path, private_ruleset_path) = resolve_ruleset_paths();
    let privacy = handles.privacy.lock().unwrap().clone();
    if !is_locked_down {
        let direct_host = if let Some(entry) = &entry_config {
            entry.server_host()
        } else {
            exit_config.server_host()
        };
        let mut routing = handles.routing_manager.lock().unwrap();
        if let Err(e) = routing.stage_exceptions(&binary_path, &app_exe_path) {
            // Kill switch is off: tolerate a broken firewall and continue
            // without the leak guard, same as the connect path.
            log::warn!("leak guard unavailable, continuing without it: {}", e);
        } else {
            if let Err(e) = routing.allow_server_endpoint(&direct_host, &privacy.bootstrap_dns) {
                log::warn!("server endpoint rule during bypass update skipped: {}", e);
            }
            if let Err(e) = routing.commit_connection() {
                routing.abort_staged_connection(false);
                log::warn!("leak guard lock failed, continuing without it: {}", e);
            }
        }
    }
    if let Err(e) = handles.process_manager.lock().unwrap().stop() {
        log::warn!("stop during bypass update: {}", e);
    }
    if !crate::util::wait_for_wintun_teardown(Duration::from_secs(6)) {
        log::warn!("previous tun adapter still present after 6s, applying bypass anyway");
    }
    thread::sleep(Duration::from_millis(300));
    let default_iface = crate::util::get_default_physical_interface();
    let generator = ConfigGenerator::new(
        is_locked_down,
        split_config_for(handles, new_paths.clone()),
        quantum_resistant,
        binary_path.clone(),
        app_exe_path.clone(),
    )
    .with_local_rulesets(ads_ruleset_path, private_ruleset_path)
    .with_default_interface(default_iface.clone())
    .with_padding(true)
    .with_dpi(&privacy.dpi_profile)
    .with_privacy(
        privacy.strict_route,
        privacy.allow_insecure_tls,
        privacy.tunnel_own_traffic,
        &privacy.bootstrap_dns,
    )
    .with_dns_center(&privacy.dns_remote, privacy.dns_custom_doh.as_deref(), privacy.dns_block_ads || privacy.dns_block_trackers)
    .with_routing(privacy.route_rules.clone(), privacy.route_all)
    .with_system_dns(resolve_isp_dns(&default_iface))
    .with_main_selector(handles.selector.lock().unwrap().clone());
    let new_config_json = match generator.to_json(&exit_config, entry_config.as_ref()) {
        Ok(json) => json,
        Err(e) => {
            full_teardown(handles, always_on_flag);
            restore_always_on(handles);
            return Err(format!(
                "Config rebuild failed, VPN disconnected for safety: {}",
                e
            ));
        }
    };
    if is_locked_down {
        if let Err(e) = handles.routing_manager.lock().unwrap().update_bypass_rules(&new_paths, crate::config::SplitMode::parse(&handles.split_rules.lock().unwrap().mode).is_include()) {
            log::warn!("Bypass firewall rule update failed: {}", e);
        }
    }
    let mut start_result = handles.process_manager.lock().unwrap().start(&new_config_json);
    if start_result.is_err() {
        thread::sleep(Duration::from_millis(500));
        start_result = handles.process_manager.lock().unwrap().start(&new_config_json);
    }
    if let Err(e) = start_result {
        full_teardown(handles, always_on_flag);
        restore_always_on(handles);
        return Err(format!(
            "Failed to apply split tunneling change, VPN disconnected for safety: {}",
            e
        ));
    }
    if !survived_startup(handles, 3000) {
        full_teardown(handles, always_on_flag);
        restore_always_on(handles);
        return Err(
            "sing-box died after applying split tunneling change, VPN disconnected for safety".into()
        );
    }
    pin_dns_to_tunnel();
    if privacy.dns_leak_guard {
        if let Err(e) = handles.routing_manager.lock().unwrap().enable_dns_leak_guard() {
            log::warn!("dns leak guard failed: {}", e);
        }
    }
    if !is_locked_down {
        if let Err(e) = handles.routing_manager.lock().unwrap().end_connection(false) {
            log::warn!("leak guard release after bypass update failed: {}", e);
        }
    }
    *handles.current_config.lock().unwrap() = Some(new_config_json);
    *handles.bypass_apps.lock().unwrap() = new_paths;
    *handles.connected_server_name.lock().unwrap() = server_name;
    *handles.connected_entry_server_name.lock().unwrap() = entry_server_name;
    Ok(())
}

pub fn socks_latency() -> Option<u64> {
    probe_socks().ok()
}

pub fn probe_socks() -> Result<u64, String> {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", crate::constants::LOCAL_PROXY_PORT)
        .parse()
        .map_err(|_| "local proxy address parse".to_string())?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(2))
        .map_err(|e| format!("local proxy connect: {}", e))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| format!("socket setup: {}", e))?;
    stream.set_write_timeout(Some(Duration::from_secs(5))).map_err(|e| format!("socket setup: {}", e))?;
    stream.set_nodelay(true).map_err(|e| format!("socket setup: {}", e))?;
    stream.write_all(&[0x05, 0x01, 0x00]).map_err(|e| format!("socks greeting write: {}", e))?;
    let mut greet = [0u8; 2];
    stream.read_exact(&mut greet).map_err(|e| format!("socks greeting read: {}", e))?;
    if greet[0] != 0x05 || greet[1] != 0x00 {
        return Err(format!("socks greeting rejected: {:02x} {:02x}", greet[0], greet[1]));
    }
    let host = b"cp.cloudflare.com";
    let mut req = vec![0x05, 0x01, 0x00, 0x03, host.len() as u8];
    req.extend_from_slice(host);
    req.extend_from_slice(&80u16.to_be_bytes());
    stream.write_all(&req).map_err(|e| format!("socks connect write: {}", e))?;
    let mut head = [0u8; 4];
    stream.read_exact(&mut head).map_err(|e| format!("socks connect read: {}", e))?;
    if head[0] != 0x05 || head[1] != 0x00 {
        return Err(format!("socks connect refused, reply code {:#04x}", head[1]));
    }
    let skip = match head[3] {
        0x01 => 6usize,
        0x03 => {
            let mut l = [0u8; 1];
            stream.read_exact(&mut l).map_err(|e| format!("socks address read: {}", e))?;
            l[0] as usize + 2
        }
        0x04 => 18usize,
        _ => return Err("socks reply address type unknown".to_string()),
    };
    let mut rest = vec![0u8; skip];
    stream.read_exact(&mut rest).map_err(|e| format!("socks address read: {}", e))?;
    let request = b"HEAD /generate_204 HTTP/1.1\r\nHost: cp.cloudflare.com\r\nConnection: keep-alive\r\n\r\n";
    let mut scratch = [0u8; 512];
    stream.write_all(request).map_err(|e| format!("http request write: {}", e))?;
    drain_http_head(&mut stream, &mut scratch).map_err(|e| format!("no http response: {}", e))?;
    let mut best = u128::MAX;
    for _ in 0..3 {
        let start = Instant::now();
        if stream.write_all(request).is_err() {
            break;
        }
        if drain_http_head(&mut stream, &mut scratch).is_err() {
            break;
        }
        best = best.min(start.elapsed().as_micros());
    }
    if best == u128::MAX {
        return Err("tunnel probe got no keep-alive response".to_string());
    }
    Ok((((best as u64) + 999) / 1000).max(1))
}

fn drain_http_head(stream: &mut std::net::TcpStream, scratch: &mut [u8]) -> Result<(), String> {
    use std::io::Read;
    let terminator = b"\r\n\r\n";
    let mut matched = 0usize;
    loop {
        let read = stream.read(scratch).map_err(|e| format!("http read: {}", e))?;
        if read == 0 {
            return Err("proxy closed the probe connection".to_string());
        }
        for byte in &scratch[..read] {
            if *byte == terminator[matched] {
                matched += 1;
                if matched == terminator.len() {
                    return Ok(());
                }
            } else {
                matched = usize::from(*byte == terminator[0]);
            }
        }
    }
}

pub fn repair_network_sync(handles: &ConnectionHandles) {
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
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SubscriptionInfo {
    pub expire: Option<i64>,
    pub total: Option<u64>,
    pub upload: Option<u64>,
    pub download: Option<u64>,
}

pub fn autostart_register_script() -> Result<String, String> {
    let exe_path = resolve_own_exe_path()?;
    let escaped_path = exe_path.replace('\'', "''");
    Ok(format!(
        "$who = [System.Security.Principal.WindowsIdentity]::GetCurrent().Name; $act = New-ScheduledTaskAction -Execute '{}' -Argument '--autostart'; $trig = New-ScheduledTaskTrigger -AtLogOn -User $who; $trig.Delay = 'PT3S'; $prin = New-ScheduledTaskPrincipal -UserId $who -RunLevel Highest -LogonType Interactive; $conf = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable; $conf.ExecutionTimeLimit = 'PT0S'; Register-ScheduledTask -TaskName 'WawityAutostart' -Action $act -Trigger $trig -Principal $prin -Settings $conf -Force -ErrorAction Stop | Out-Null; Remove-ItemProperty -Path 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run' -Name 'Wawity' -ErrorAction SilentlyContinue | Out-Null",
        escaped_path
    ))
}

pub fn first_launch_marker() -> Result<std::path::PathBuf, String> {
    let base = std::env::var("APPDATA").map_err(|_| "APPDATA is not available".to_string())?;
    Ok(std::path::PathBuf::from(base).join("Wawity").join("setup-complete.flag"))
}
