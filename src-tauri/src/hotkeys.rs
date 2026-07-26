use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, PostThreadMessageW, MSG, WM_APP, WM_HOTKEY};

const HOTKEY_ID: i32 = 1;

static LOOP_STARTED: AtomicBool = AtomicBool::new(false);
static LOOP_THREAD_ID: AtomicU32 = AtomicU32::new(0);
static PENDING: Mutex<Option<Option<(u32, u32)>>> = Mutex::new(None);
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

pub fn apply(app: &AppHandle, combo: Option<String>) -> Result<(), String> {
    let parsed = match combo {
        Some(c) if !c.trim().is_empty() => Some(parse_combo(c.trim())?),
        _ => None,
    };
    *APP_HANDLE.lock().unwrap() = Some(app.clone());
    *PENDING.lock().unwrap() = Some(parsed);
    if !LOOP_STARTED.swap(true, Ordering::SeqCst) {
        thread::spawn(hotkey_loop);
        return Ok(());
    }
    let mut waited = 0;
    let mut tid = LOOP_THREAD_ID.load(Ordering::SeqCst);
    while tid == 0 && waited < 100 {
        thread::sleep(Duration::from_millis(5));
        waited += 1;
        tid = LOOP_THREAD_ID.load(Ordering::SeqCst);
    }
    if tid != 0 {
        unsafe {
            let _ = PostThreadMessageW(tid, WM_APP, WPARAM(0), LPARAM(0));
        }
    }
    Ok(())
}

fn hotkey_loop() {
    unsafe {
        LOOP_THREAD_ID.store(GetCurrentThreadId(), Ordering::SeqCst);
        let mut registered = false;
        loop {
            let next = PENDING.lock().unwrap().take();
            if let Some(next) = next {
                if registered {
                    let _ = UnregisterHotKey(HWND(0), HOTKEY_ID);
                    registered = false;
                }
                if let Some((mods, vk)) = next {
                    match RegisterHotKey(
                        HWND(0),
                        HOTKEY_ID,
                        HOT_KEY_MODIFIERS(mods | MOD_NOREPEAT.0),
                        vk,
                    ) {
                        Ok(()) => registered = true,
                        Err(e) => notify_error(e.to_string()),
                    }
                }
            }
            let mut msg = MSG::default();
            let res = GetMessageW(&mut msg, HWND(0), 0, 0);
            if res.0 <= 0 {
                break;
            }
            if msg.message == WM_HOTKEY {
                trigger();
            }
        }
        LOOP_THREAD_ID.store(0, Ordering::SeqCst);
        LOOP_STARTED.store(false, Ordering::SeqCst);
    }
}

fn trigger() {
    
    let app = APP_HANDLE.lock().unwrap().clone();
    if let Some(app) = app {
        let _ = app.emit_all("wawity-hotkey-toggle", ());
        crate::spawn_background_toggle(app, true);
    }
}

fn notify_error(message: String) {
    let app = APP_HANDLE.lock().unwrap().clone();
    if let Some(app) = app {
        let _ = app.emit_all("wawity-hotkey-error", message);
    }
}

fn parse_combo(combo: &str) -> Result<(u32, u32), String> {
    let mut mods: u32 = 0;
    let mut key: Option<u32> = None;
    for part in combo.split('+') {
        let token = part.trim().to_ascii_uppercase();
        match token.as_str() {
            "CTRL" | "CONTROL" | "COMMANDORCONTROL" | "CMDORCTRL" => mods |= MOD_CONTROL.0,
            "SHIFT" => mods |= MOD_SHIFT.0,
            "ALT" | "OPTION" => mods |= MOD_ALT.0,
            "SUPER" | "WIN" | "META" | "CMD" | "COMMAND" => mods |= MOD_WIN.0,
            other => key = Some(parse_key(other)?),
        }
    }
    match key {
        Some(vk) => Ok((mods, vk)),
        None => Err(format!("no key in combo: {combo}")),
    }
}

fn parse_key(token: &str) -> Result<u32, String> {
    if token.len() == 1 {
        let ch = token.chars().next().unwrap();
        let vk = match ch {
            'A'..='Z' | '0'..='9' => ch as u32,
            '`' => 0xC0,
            '-' => 0xBD,
            '=' => 0xBB,
            '[' => 0xDB,
            ']' => 0xDD,
            '\\' => 0xDC,
            ';' => 0xBA,
            '\'' => 0xDE,
            ',' => 0xBC,
            '.' => 0xBE,
            '/' => 0xBF,
            _ => return Err(format!("unsupported key: {token}")),
        };
        return Ok(vk);
    }
    if let Some(num) = token.strip_prefix('F') {
        if let Ok(n) = num.parse::<u32>() {
            if (1..=24).contains(&n) {
                return Ok(0x6F + n);
            }
        }
    }
    let vk = match token {
        "SPACE" => 0x20,
        "TAB" => 0x09,
        "HOME" => 0x24,
        "END" => 0x23,
        "PAGEUP" => 0x21,
        "PAGEDOWN" => 0x22,
        "INSERT" => 0x2D,
        "DELETE" => 0x2E,
        "UP" => 0x26,
        "DOWN" => 0x28,
        "LEFT" => 0x25,
        "RIGHT" => 0x27,
        _ => return Err(format!("unsupported key: {token}")),
    };
    Ok(vk)
}