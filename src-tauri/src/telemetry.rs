use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

const SENTRY_DSN: &str = "https://bb87803130401b2a4da17f73cb08e715@o4511764824719360.ingest.de.sentry.io/4511764827013200";
const APTABASE_APP_KEY: &str = "";
const STATS_RELAY_URL: &str = "https://wawity.dass-awesome.workers.dev/e/k3v9x2m8qwe4rty7asd1fgh5jkl0zxc6vbn3mqp8";

static ENABLED: AtomicBool = AtomicBool::new(true);

fn session_id() -> &'static str {
    static SESSION: OnceLock<String> = OnceLock::new();
    SESSION.get_or_init(|| {
        format!("{}-{}", chrono::Utc::now().timestamp_millis(), std::process::id())
    })
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

fn allowed() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

fn post_json(url: String, headers: Vec<(String, String)>, body: serde_json::Value, wait: bool) {
    let task = move || {
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(6))
            .build()
        {
            Ok(c) => c,
            Err(_) => return,
        };
        let mut req = client.post(&url).json(&body);
        for (name, value) in &headers {
            req = req.header(name.as_str(), value.as_str());
        }
        let _ = req.send();
    };
    if wait {
        let _ = std::thread::spawn(task).join();
    } else {
        std::thread::spawn(task);
    }
}

fn sentry_bits() -> Option<(String, String)> {
    if SENTRY_DSN.is_empty() {
        return None;
    }
    let rest = SENTRY_DSN.strip_prefix("https://")?;
    let (key, tail) = rest.split_once('@')?;
    let (host, project) = tail.rsplit_once('/')?;
    Some((
        "https://".to_string() + host + "/api/" + project + "/store/",
        format!(
            "Sentry sentry_version=7, sentry_key={}, sentry_client=wawity/{}",
            key,
            env!("CARGO_PKG_VERSION")
        ),
    ))
}

pub fn report(level: &str, message: String, stack: Option<String>, wait: bool) {
    if !allowed() {
        return;
    }
    let bits = match sentry_bits() {
        Some(v) => v,
        None => return,
    };
    let mut event = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "platform": "other",
        "level": level,
        "release": format!("wawity@{}", env!("CARGO_PKG_VERSION")),
        "message": { "formatted": message },
        "tags": { "os": std::env::consts::OS }
    });
    if let Some(stack) = stack {
        event["extra"] = serde_json::json!({ "stack": stack });
    }
    post_json(
        bits.0,
        vec![
            ("X-Sentry-Auth".to_string(), bits.1),
            ("Content-Type".to_string(), "application/json".to_string()),
        ],
        event,
        wait,
    );
}

fn aptabase_host() -> &'static str {
    if APTABASE_APP_KEY.starts_with("A-EU-") {
        "https://eu.aptabase.com"
    } else if APTABASE_APP_KEY.starts_with("A-US-") {
        "https://us.aptabase.com"
    } else {
        "https://api.aptabase.com"
    }
}

pub fn track(name: String, props: serde_json::Value) {
    if !allowed() {
        return;
    }
    if !STATS_RELAY_URL.is_empty() {
        let body = serde_json::json!({
            "event": name,
            "sessionId": session_id(),
            "appVersion": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "props": props
        });
        post_json(STATS_RELAY_URL.to_string(), Vec::new(), body, false);
        return;
    }
    if APTABASE_APP_KEY.is_empty() {
        return;
    }
    let body = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "sessionId": session_id(),
        "eventName": name,
        "systemProps": {
            "isDebug": cfg!(debug_assertions),
            "locale": "",
            "osName": std::env::consts::OS,
            "osVersion": "",
            "appVersion": env!("CARGO_PKG_VERSION"),
            "sdkVersion": "wawity-telemetry@1"
        },
        "props": props
    });
    post_json(
        format!("{}/api/v0/event", aptabase_host()),
        vec![("App-Key".to_string(), APTABASE_APP_KEY.to_string())],
        body,
        false,
    );
}

pub fn install_panic_hook() {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "panic".to_string()
        };
        let loc = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        report("fatal", format!("panic: {} ({})", msg, loc), None, true);
        prev(info);
    }));
}

#[tauri::command]
pub fn set_telemetry_enabled(enabled: bool) {
    set_enabled(enabled);
}

#[tauri::command]
pub fn track_event(name: String, props: Option<serde_json::Value>) {
    track(name, props.unwrap_or_else(|| serde_json::json!({})));
}

#[tauri::command]
pub fn report_error(message: String, stack: Option<String>) {
    report("error", message, stack, false);
}
