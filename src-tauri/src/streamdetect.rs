use std::sync::atomic::{AtomicBool, Ordering};

pub static STREAM_CAPTURE: AtomicBool = AtomicBool::new(false);

const CAPTURE_PROCESSES: [&str; 16] = [
    "obs64.exe",
    "obs32.exe",
    "streamlabsobs.exe",
    "streamlabs desktop service.exe",
    "streamlabs desktop.exe",
    "xsplit.broadcaster.exe",
    "xsplit.core.exe",
    "prism live studio.exe",
    "vmix64.exe",
    "gamebarftserver.exe",
    "nvidia share.exe",
    "nvidia app.exe",
    "radeonsoftware.exe",
    "bandicam.exe",
    "camtasiarecorder.exe",
    "wirecast.exe",
];

#[cfg(windows)]
pub fn detect() -> bool {
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut found = false;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|c| *c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                )
                .to_ascii_lowercase();
                if CAPTURE_PROCESSES.iter().any(|t| *t == name) {
                    found = true;
                    break;
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
pub fn detect() -> bool {
    false
}

pub fn detect_and_store() -> bool {
    let running = detect();
    STREAM_CAPTURE.store(running, Ordering::Relaxed);
    running
}

pub fn current() -> bool {
    STREAM_CAPTURE.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn stream_capture_running() -> bool {
    detect_and_store()
}

#[tauri::command]
pub fn stream_capture_state() -> bool {
    current()
}
