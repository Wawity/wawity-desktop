
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

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
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    prev: HashMap<ConnKey, (u64, u64)>,
    names: HashMap<u32, String>,
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
            let Some(get_etype) = get_per_tcp_connection_etype() else {
                return out;
            };
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
    let mut inner = state.inner.lock().unwrap();

    let current = sample_connections();

    for (key, (bytes_in, bytes_out)) in &current {
        let (d_in, d_out) = match inner.prev.get(key) {
            Some((pin, pout)) => (
                bytes_in.saturating_sub(*pin),
                bytes_out.saturating_sub(*pout),
            ),
            None => (0, 0),
        };
        let totals = inner.totals.entry(key.0).or_insert((0, 0));
        totals.0 += d_in;
        totals.1 += d_out;
    }

    inner.prev = current;

    let mut per_pid: Vec<(u32, (u64, u64))> = inner
        .totals
        .iter()
        .filter(|(_, (rx, tx))| *rx > 0 || *tx > 0)
        .map(|(pid, v)| (*pid, *v))
        .collect();

    per_pid.sort_by(|a, b| (b.1 .0 + b.1 .1).cmp(&(a.1 .0 + a.1 .1)));
    per_pid.truncate(12);

    let entries = per_pid
        .into_iter()
        .map(|(pid, (rx, tx))| AppTrafficEntry {
            name: process_name(pid, &mut inner.names),
            rx,
            tx,
        })
        .collect();

    entries
}

#[tauri::command]
pub fn get_app_traffic(state: tauri::State<'_, AppTrafficState>) -> Vec<AppTrafficEntry> {
    take_snapshot(&state)
}
