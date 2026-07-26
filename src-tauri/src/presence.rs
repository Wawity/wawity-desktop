use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use discord_rich_presence::activity::{Activity, Assets, Timestamps};
use discord_rich_presence::{DiscordIpc, DiscordIpcClient};
use serde::Deserialize;

const DISCORD_APP_ID: &str = "1487515469069029507";

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

fn run_worker(rx: Receiver<PresencePayload>) {
    let launched_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

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
        let details = if snapshot.connected {
            match (&snapshot.server_name, snapshot.show_server) {
                (Some(name), true) => {
                    if ru {
                        format!("Подключено · {}", name)
                    } else {
                        format!("Connected · {}", name)
                    }
                }
                _ => {
                    if ru {
                        "Подключено".to_string()
                    } else {
                        "Connected".to_string()
                    }
                }
            }
        } else if ru {
            "Отключено".to_string()
        } else {
            "Disconnected".to_string()
        };

        let mut activity = Activity::new()
            .details(&details)
            .assets(Assets::new().large_image("rpc").large_text("Wawity VPN"))
            .timestamps(Timestamps::new().start(launched_at));

        let sub = if snapshot.show_subscription {
            snapshot.subscription_name.clone()
        } else {
            None
        };
        if let Some(ref s) = sub {
            activity = activity.state(s);
        }

        if c.set_activity(activity).is_err() {
            broken = true;
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