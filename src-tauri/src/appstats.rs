
use crate::etwstats;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPTABLE_OWNER_MODULE, TCP_TABLE_OWNER_MODULE_ALL,
};
use windows::Win32::Networking::WinSock::AF_INET;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION,
};

#[repr(C)]
struct RawTcpRow {
    state: u32,
    local_addr: u32,
    local_port: u32,
    remote_addr: u32,
    remote_port: u32,
}

#[repr(C)]
struct EstatsDataPathRow {
    data_bytes_in: u64,
    data_bytes_out: u64,
}

type GetPerTcpConnectionETypeFn = unsafe extern "system" fn(
    *const RawTcpRow,
    u32,
    *mut core::ffi::c_void,
    u32,
    u32,
) -> u32;

fn get_per_tcp_connection_etype() -> Option<GetPerTcpConnectionETypeFn> {
    static FN: OnceLock<Option<GetPerTcpConnectionETypeFn>> = OnceLock::new();
    *FN.get_or_init(|| unsafe {
        use windows::core::{s, w};
        use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
        let module = LoadLibraryW(w!("iphlpapi.dll")).ok()?;
        match GetProcAddress(module, s!("GetPerTcpConnectionEType")) {
            Some(f) => Some(core::mem::transmute::<
                unsafe extern "system" fn() -> isize,
                GetPerTcpConnectionETypeFn,
            >(f)),
            None => None,
        }
    })
}

const TCP_ESTATS_DATA_PATH_TCP_ROW_V0: u32 = 0;

/// Per-PID totals only ever grew during a long session. Past this size we
/// keep the heaviest talkers and drop the rest.
const MAX_TRACKED_PIDS: usize = 512;
const KEEP_TRACKED_PIDS: usize = 256;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTrafficEntry {
    pub name: String,
    pub rx: u64,
    pub tx: u64,
}

type ConnKey = (u32, [u8; 4], u16, [u8; 4], u16);

#[derive(Default)]
pub struct AppTrafficState {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    prev: HashMap<ConnKey, (u64, u64)>,
    names: HashMap<u32, String>,
    /// Accumulated while ETW is not running. When ETW is live the byte totals
    /// come from the kernel network provider instead, and this stays empty so
    /// the two sources never double count.
    totals: HashMap<u32, (u64, u64)>,
}

fn addr_bytes(v: u32) -> [u8; 4] {
    v.to_ne_bytes()
}

fn port_host(v: u32) -> u16 {
    ((v >> 8) | ((v & 0xFF) << 8)) as u16
}

fn process_name(pid: u32, cache: &mut HashMap<u32, String>) -> String {
    if let Some(n) = cache.get(&pid) {
        return n.clone();
    }
    let fallback = format!("pid {}", pid);
    let name = unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)
            .unwrap_or(HANDLE::default());
        if handle.is_invalid() {
            cache.insert(pid, fallback.clone());
            return fallback;
        }
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if ok && len > 0 {
            let full = String::from_utf16_lossy(&buf[..len as usize]);
            let short = full.rsplit('\\').next().unwrap_or(&full).to_string();
            if short.is_empty() {
                fallback.clone()
            } else {
                short
            }
        } else {
            fallback.clone()
        }
    };
    cache.insert(pid, name.clone());
    name
}

fn sample_connections() -> HashMap<ConnKey, (u64, u64)> {
    let mut out = HashMap::new();
    let Some(get_etype) = get_per_tcp_connection_etype() else {
        return out;
    };
    unsafe {
        let mut size: u32 = 0;
        let _ = GetExtendedTcpTable(
            None,
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        );
        if size == 0 {
            return out;
        }
        let mut buf = vec![0u8; size as usize];
        let rc = GetExtendedTcpTable(
            Some(buf.as_mut_ptr().cast()),
            &mut size,
            false,
            AF_INET.0 as u32,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        );
        if rc != 0 {
            return out;
        }
        let table = buf.as_ptr().cast::<MIB_TCPTABLE_OWNER_MODULE>();
        let count = (*table).dwNumEntries as usize;
        let rows = std::slice::from_raw_parts((*table).table.as_ptr(), count);

        for row in rows {
            if row.dwOwningPid == 0 {
                continue;
            }
            let raw = RawTcpRow {
                state: row.dwState,
                local_addr: row.dwLocalAddr,
                local_port: row.dwLocalPort,
                remote_addr: row.dwRemoteAddr,
                remote_port: row.dwRemotePort,
            };
            let mut stats = EstatsDataPathRow { data_bytes_in: 0, data_bytes_out: 0 };
            let rc = get_etype(
                &raw,
                TCP_ESTATS_DATA_PATH_TCP_ROW_V0,
                &mut stats as *mut _ as *mut core::ffi::c_void,
                0,
                std::mem::size_of::<EstatsDataPathRow>() as u32,
            );
            if rc != 0 {
                continue;
            }
            let key: ConnKey = (
                row.dwOwningPid,
                addr_bytes(row.dwLocalAddr),
                port_host(row.dwLocalPort),
                addr_bytes(row.dwRemoteAddr),
                port_host(row.dwRemotePort),
            );
            out.insert(key, (stats.data_bytes_in, stats.data_bytes_out));
        }
    }
    out
}

pub fn take_snapshot(state: &AppTrafficState) -> Vec<AppTrafficEntry> {
    // With ETW live the kernel provider already feeds the totals, and running
    // the TCP table walk on top of it would double count the IPv4 TCP flows.
    let etw_live = etwstats::state() == etwstats::CaptureState::Running;
    if !etw_live {
        sample_legacy_into_totals(state);
    }

    let mut per_pid: Vec<(u32, (u64, u64))> = etwstats::take_totals()
        .into_iter()
        .filter(|(_, (rx, tx))| *rx > 0 || *tx > 0)
        .collect();
    per_pid.sort_by(|a, b| (b.1 .0 + b.1 .1).cmp(&(a.1 .0 + a.1 .1)));
    per_pid.truncate(12);

    let mut guard = state.inner.lock().unwrap();
    let inner = &mut *guard;
    let tracked: HashSet<u32> = per_pid.iter().map(|(pid, _)| *pid).collect();
    inner.names.retain(|pid, _| tracked.contains(pid));
    if inner.totals.len() > MAX_TRACKED_PIDS {
        let mut ranked: Vec<(u32, (u64, u64))> =
            inner.totals.iter().map(|(pid, v)| (*pid, *v)).collect();
        ranked.sort_by(|a, b| (b.1 .0 + b.1 .1).cmp(&(a.1 .0 + a.1 .1)));
        ranked.truncate(KEEP_TRACKED_PIDS);
        inner.totals = ranked.into_iter().collect();
    }

    per_pid
        .into_iter()
        .map(|(pid, (rx, tx))| AppTrafficEntry {
            name: process_name(pid, &mut inner.names),
            rx,
            tx,
        })
        .collect()
}

/// The original IPv4-TCP sampler, kept as the fallback for when the kernel
/// logger cannot run (no administrator rights, or a session already in use).
/// It walks the TCP table, diffs against the previous walk and feeds the same
/// totals map ETW uses.
fn sample_legacy_into_totals(state: &AppTrafficState) {
    // Walk the TCP table before taking the lock.
    let current = sample_connections();

    let mut guard = state.inner.lock().unwrap();
    let inner = &mut *guard;

    for (key, (bytes_in, bytes_out)) in &current {
        let (d_in, d_out) = match inner.prev.get(key) {
            Some((pin, pout)) => (bytes_in.saturating_sub(*pin), bytes_out.saturating_sub(*pout)),
            // A connection seen for the first time has no baseline, so
            // attributing its whole lifetime here would be a lie.
            None => (0, 0),
        };
        if d_in == 0 && d_out == 0 {
            continue;
        }
        etwstats::add(key.0, d_in, d_out);
        let totals = inner.totals.entry(key.0).or_insert((0, 0));
        totals.0 = totals.0.saturating_add(d_in);
        totals.1 = totals.1.saturating_add(d_out);
    }

    inner.prev = current;
}

/// Starts counting every protocol: TCP and UDP, IPv4 and IPv6.
///
/// `false` means the kernel logger refused. The reason is kept in
/// [`etwstats::state`] so the interface can tell the user that the numbers
/// are TCP/IPv4 only instead of silently showing a short list.
pub fn start_capture() -> bool {
    ETW_SESSION.with(|s| {
        if s.borrow().is_some() {
            return true;
        }
        match etwstats::start() {
            Some(session) => {
                *s.borrow_mut() = Some(session);
                true
            }
            None => false,
        }
    })
}

pub fn stop_capture() {
    ETW_SESSION.with(|s| {
        s.borrow_mut().take();
    });
}

thread_local! {
    static ETW_SESSION: std::cell::RefCell<Option<etwstats::Session>> =
        const { std::cell::RefCell::new(None) };
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppTrafficStatus {
    /// One of "running" | "idle" | "needsAdmin" | "unavailable".
    pub mode: String,
    /// True when the numbers include UDP and IPv6, false for the TCP-only
    /// fallback.
    pub full: bool,
}

pub fn capture_status() -> AppTrafficStatus {
    let mode = match etwstats::state() {
        etwstats::CaptureState::Running => "running",
        etwstats::CaptureState::NeedsAdmin => "needsAdmin",
        etwstats::CaptureState::Unavailable => "unavailable",
        _ => "idle",
    };
    AppTrafficStatus {
        mode: mode.to_string(),
        full: mode == "running",
    }
}

#[tauri::command]
pub async fn set_app_traffic_capture(enabled: bool) -> Result<AppTrafficStatus, String> {
    if enabled {
        start_capture();
    } else {
        stop_capture();
    }
    Ok(capture_status())
}

#[tauri::command]
pub async fn get_app_traffic_status() -> Result<AppTrafficStatus, String> {
    Ok(capture_status())
}

// Async + worker thread: the TCP table walk and OpenProcess calls used to run
// on the thread that pumps every window.
#[tauri::command]
pub async fn get_app_traffic(
    state: tauri::State<'_, AppTrafficState>,
) -> Result<Vec<AppTrafficEntry>, String> {
    let snapshot_state = AppTrafficState {
        inner: Arc::clone(&state.inner),
    };
    tokio::task::spawn_blocking(move || take_snapshot(&snapshot_state))
        .await
        .map_err(|e| e.to_string())
}
