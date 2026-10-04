use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager};

const POLL: Duration = Duration::from_millis(1800);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchState {
    pub armed: bool,
    pub process: String,
    pub running: bool,
}

struct Slot {
    process: String,
    stop: Arc<AtomicBool>,
    running: Arc<AtomicBool>,
}

static SLOT: Lazy<Mutex<Option<Slot>>> = Lazy::new(|| Mutex::new(None));

fn leaf_of(raw: &str) -> String {
    let flat = raw.trim().replace('/', "\\").to_lowercase();
    flat.rsplit('\\').next().unwrap_or("").to_string()
}

fn hits(needle: &str, name: &str, path: &str) -> bool {
    let want = needle.trim().replace('/', "\\").to_lowercase();
    if want.is_empty() {
        return false;
    }
    let leaf = leaf_of(needle);
    if leaf.is_empty() {
        return false;
    }
    let name_low = name.trim().to_lowercase();
    let path_low = path.trim().replace('/', "\\").to_lowercase();

    if !path_low.is_empty() && (path_low == want || path_low.ends_with(&format!("\\{}", leaf))) {
        return true;
    }
    name_low == leaf || leaf_of(name) == leaf
}

fn alive(needle: &str) -> bool {
    match crate::util::enumerate_processes_with_paths() {
        Ok(list) => list.iter().any(|(_, name, path)| hits(needle, name, path)),
        Err(_) => false,
    }
}

fn tear_down() {
    if let Ok(mut slot) = SLOT.lock() {
        if let Some(prev) = slot.take() {
            prev.stop.store(true, Ordering::Relaxed);
        }
    }
}

#[tauri::command]
pub async fn arm_app_watch(app: AppHandle, process: String) -> Result<WatchState, String> {
    let target = process.trim().to_string();
    if target.is_empty() {
        return Err("process name is empty".to_string());
    }

    tear_down();

    let stop = Arc::new(AtomicBool::new(false));
    let running = Arc::new(AtomicBool::new(false));

    {
        let mut slot = SLOT.lock().map_err(|e| e.to_string())?;
        *slot = Some(Slot {
            process: target.clone(),
            stop: Arc::clone(&stop),
            running: Arc::clone(&running),
        });
    }

    let watched = target.clone();
    let halt = Arc::clone(&stop);
    let flag = Arc::clone(&running);

    std::thread::spawn(move || {
        let mut appeared = false;
        loop {
            if halt.load(Ordering::Relaxed) {
                return;
            }

            let up = alive(&watched);
            flag.store(up, Ordering::Relaxed);

            if up {
                appeared = true;
            } else if appeared {
                if let Ok(mut slot) = SLOT.lock() {
                    let mine = slot
                        .as_ref()
                        .map(|s| s.process == watched && !s.stop.load(Ordering::Relaxed))
                        .unwrap_or(false);
                    if mine {
                        *slot = None;
                    } else {
                        return;
                    }
                }
                let _ = app.emit_all("wawity-watched-app-closed", watched.clone());
                return;
            }

            std::thread::sleep(POLL);
        }
    });

    Ok(WatchState {
        armed: true,
        process: target,
        running: false,
    })
}

#[tauri::command]
pub async fn disarm_app_watch() -> Result<WatchState, String> {
    tear_down();
    Ok(WatchState {
        armed: false,
        process: String::new(),
        running: false,
    })
}

#[tauri::command]
pub async fn app_watch_state() -> Result<WatchState, String> {
    let slot = SLOT.lock().map_err(|e| e.to_string())?;
    Ok(match slot.as_ref() {
        Some(active) => WatchState {
            armed: true,
            process: active.process.clone(),
            running: active.running.load(Ordering::Relaxed),
        },
        None => WatchState {
            armed: false,
            process: String::new(),
            running: false,
        },
    })
}
