use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

use discord_rich_presence::activity::{Activity, Assets, Button, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use serde::Deserialize;

const DISCORD_APP_ID: &str = "1487515469069029507";
const DOWNLOAD_URL: &str =
    "https://github.com/Wawity/wawity-desktop/releases/download/desktop/WawitySetup-Desktop.exe";
const TELEGRAM_URL: &str = "https://t.me/wawityvpn";

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresencePayload {
    pub enabled: bool,
    pub show_server: bool,
    pub show_subscription: bool,
    pub connected: bool,
    pub server_name: Option<String>,
    pub subscription_name: Option<String>,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub session_start: Option<i64>,
}

pub struct PresenceLink {
    tx: Sender<PresencePayload>,
}

impl PresenceLink {
    pub fn spawn() -> Self {
        let (tx, rx) = channel::<PresencePayload>();
        thread::spawn(move || run_worker(rx));
        Self { tx }
    }

    pub fn push(&self, payload: PresencePayload) {
        let _ = self.tx.send(payload);
    }
}

fn flag_emoji(code: &str) -> String {
    let code = code.trim().to_uppercase();
    if code.len() != 2 || !code.chars().all(|c| c.is_ascii_alphabetic()) {
        return String::new();
    }
    code.chars()
        .filter_map(|c| {
            let c = c as u32;
            char::from_u32(0x1F1E6 + (c - 'A' as u32))
        })
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .concat()
}

fn connected_details(snapshot: &PresencePayload) -> String {
    let ru = snapshot.language == "ru";
    let flag = snapshot
        .country_code
        .as_deref()
        .map(flag_emoji)
        .unwrap_or_default();
    let server_part = match (&snapshot.server_name, snapshot.show_server) {
        (Some(name), true) => {
            let name = name.trim();
            if flag.is_empty() {
                name.to_string()
            } else {
                format!("{flag} {name}")
            }
        }
        _ => String::new(),
    };

    if server_part.is_empty() {
        if ru {
            "Подключено · трафик зашифрован".into()
        } else {
            "Connected · traffic encrypted".into()
        }
    } else if ru {
        format!("Подключено · {server_part}")
    } else {
        format!("Connected · {server_part}")
    }
}

fn connected_state(snapshot: &PresencePayload) -> Option<String> {
    if !snapshot.show_subscription {
        return None;
    }
    snapshot
        .subscription_name
        .clone()
        .filter(|s| !s.trim().is_empty())
}

#[derive(Debug)]
struct PresenceError;

fn run_worker(rx: Receiver<PresencePayload>) {
    let mut client: Option<DiscordIpcClient> = None;
    let mut snapshot = PresencePayload::default();
    let mut broken = false;

    loop {
        match rx.recv_timeout(Duration::from_secs(15)) {
            Ok(next) => snapshot = next,
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        if !snapshot.enabled {
            if let Some(mut c) = client.take() {
                let _ = c.clear_activity();
                let _ = c.close();
            }
            broken = false;
            continue;
        }

        if broken {
            if let Some(mut c) = client.take() {
                let _ = c.close();
            }
            broken = false;
        }

        if client.is_none() {
            if let Ok(mut c) = DiscordIpcClient::new(DISCORD_APP_ID) {
                if c.connect().is_ok() {
                    client = Some(c);
                }
            }
        }

        let Some(c) = client.as_mut() else { continue };

        let ru = snapshot.language == "ru";
        let version = env!("CARGO_PKG_VERSION");

        
        let large_text = format!("wawity v{version}");
        let buttons = vec![
            Button::new("Download wawity", DOWNLOAD_URL),
            Button::new("Telegram", TELEGRAM_URL),
        ];

        let mut activity;

        if snapshot.connected {
            let details = connected_details(&snapshot);
            let state = connected_state(&snapshot);
            let small_text = if ru {
                String::from("Соединение защищено")
            } else {
                String::from("Connection protected")
            };

            let assets = Assets::new()
                .large_image("rpc")
                .large_text(large_text.as_str())
                .small_image("status-on")
                .small_text(small_text.as_str());

            let timestamps = snapshot
                .session_start
                .filter(|secs| *secs > 0)
                .map(|secs| Timestamps::new().start(secs));

            activity = Activity::new().details(details.as_str()).assets(assets);
            if let Some(ref st) = state {
                activity = activity.state(st.as_str());
            }
            if let Some(ts) = timestamps {
                activity = activity.timestamps(ts);
            }
            activity = activity.buttons(buttons.clone());

            let result = c.set_activity(activity);
            if result.is_err() {
                
                let assets = Assets::new()
                    .large_image("rpc")
                    .large_text(large_text.as_str())
                    .small_image("status-on")
                    .small_text(small_text.as_str());
                let mut retry = Activity::new().details(details.as_str()).assets(assets);
                if let Some(ref st) = state {
                    retry = retry.state(st.as_str());
                }
                if let Some(secs) = snapshot.session_start.filter(|secs| *secs > 0) {
                    retry = retry.timestamps(Timestamps::new().start(secs));
                }
                if c.set_activity(retry).is_err() {
                    eprintln!("[presence] set_activity failed");
                    broken = true;
                }
            }
        } else {
            let details = if ru {
                String::from("Не защищён")
            } else {
                String::from("Unprotected")
            };
            let small_text = if ru {
                String::from("VPN выключен")
            } else {
                String::from("VPN off")
            };

            let assets = Assets::new()
                .large_image("rpc-off")
                .large_text(large_text.as_str())
                .small_image("status-off")
                .small_text(small_text.as_str());

            activity = Activity::new()
                .details(details.as_str())
                .assets(assets)
                .buttons(buttons);

            if c.set_activity(activity).is_err() {
                eprintln!("[presence] set_activity failed");
                broken = true;
            }
        }
    }
}

#[tauri::command]
pub fn sync_discord_presence(
    payload: PresencePayload,
    link: tauri::State<'_, PresenceLink>,
) -> Result<(), String> {
    link.push(payload);
    Ok(())
}
