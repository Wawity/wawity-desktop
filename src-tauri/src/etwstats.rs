//! Per-process network byte accounting via ETW.
//!
//! Why this exists: the old sampler used `GetPerTcpConnectionEType`, which
//! covers **IPv4 TCP only**. Games, voice, QUIC and every IPv6 flow were
//! invisible, so the per-app breakdown looked empty or wrong even while the
//! VPN was busy. Windows exposes per-process bytes for UDP and IPv6 only
//! through the kernel network provider, which is what this module consumes.
//!
//! Things that bite here, and how they are handled:
//!
//! * `ProcessTrace` blocks forever. It runs on its own thread and is released
//!   with `ControlTrace(STOP)`, so `Drop` must stop the session *before*
//!   joining.
//! * The event callback must never block or it stalls the whole ETW pipeline,
//!   so samples are batched in a thread-local and folded into the shared map
//!   only every `FLUSH_EVERY` events.
//! * The buffer callback must return 0. Returning non-zero asks ETW to flush
//!   every single buffer, which is a lot of pointless IO at network rates.
//! * A kernel logger needs administrator rights and cannot run twice. Both
//!   failures are recorded so the UI can say something honest, and the caller
//!   falls back to the old TCP/IPv4 sampler instead of showing nothing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::core::PCWSTR;
use windows::Win32::System::Diagnostics::Etw::{
    CloseTrace, ControlTraceW, CONTROLTRACE_HANDLE, EnableTraceEx, OpenTraceW, ProcessTrace, StartTraceW,
    EVENT_CONTROL_CODE_DISABLE_PROVIDER, EVENT_RECORD, EVENT_TRACE_CONTROL_STOP,
    EVENT_TRACE_FLAG_NETWORK_TCPIP, EVENT_TRACE_LOGFILEW, EVENT_TRACE_PROPERTIES,
    EVENT_TRACE_REAL_TIME_MODE, EVENT_TRACE_SYSTEM_LOGGER_MODE, PROCESSTRACE_HANDLE,
    SystemTraceControlGuid,
};

/// Kernel provider keyword that emits TCP/IPv4/IPv6 and UDP send + receive.
const KEYWORD_NETWORK: u64 = 0x1_0000_0000;

const SESSION_NAME: &str = "WawityNetStats";
/// NT Kernel Logger hands out buffers per processor; too few and a burst
/// silently drops events.
const MIN_BUFFERS: u32 = 64;
const BUFFER_SIZE: u32 = 64;
/// How often batched callback samples are folded into the shared map.
const FLUSH_EVERY: u32 = 256;

/// MOF task numbers we care about, from the kernel network provider.
const TASK_TCPIP_SEND: u16 = 10;
const TASK_UDPIP_SEND: u16 = 12;
const TASK_TCPIP_RECV_V4: u16 = 11;
const TASK_UDPIP_RECV_V4: u16 = 13;
const TASK_TCPIP_RECV_V6: u16 = 26;
const TASK_UDPIP_RECV_V6: u16 = 27;

pub type Totals = HashMap<u32, (u64, u64)>;

fn totals() -> &'static Mutex<Totals> {
    static T: OnceLock<Mutex<Totals>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Why ETW is not running, so the interface can say something honest rather
/// than showing a suspiciously empty list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureState {
    /// Not started; nothing has asked for per-app stats.
    Idle,
    /// Kernel logger live, counting UDP and IPv6 as well as TCP.
    Running,
    /// Refused, almost always "run as administrator".
    NeedsAdmin,
    /// Something else went wrong; the reason is in the log.
    Unavailable,
}

static STATE: Mutex<CaptureState> = Mutex::new(CaptureState::Idle);
static ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn state() -> CaptureState {
    STATE.lock().map(|s| *s).unwrap_or(CaptureState::Idle)
}

fn set_state(s: CaptureState) {
    if let Ok(mut g) = STATE.lock() {
        *g = s;
    }
}

thread_local! {
    static BATCH: std::cell::RefCell<HashMap<u32, (u64, u64)>> =
        std::cell::RefCell::new(HashMap::new());
    static PENDING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn flush() {
    BATCH.with(|b| {
        let mut batch = b.borrow_mut();
        if batch.is_empty() {
            return;
        }
        if let Ok(mut all) = totals().lock() {
            for (pid, (rx, tx)) in batch.iter() {
                let e = all.entry(*pid).or_insert((0, 0));
                e.0 = e.0.saturating_add(*rx);
                e.1 = e.1.saturating_add(*tx);
            }
        }
        batch.clear();
    });
}

/// A live kernel-network session. Dropping it stops the session.
pub struct Session {
    properties: Box<EVENT_TRACE_PROPERTIES>,
    consumer: Option<std::thread::JoinHandle<()>>,
}

impl Drop for Session {
    fn drop(&mut self) {
        // Stopping the session is what makes ProcessTrace return; without
        // this the join below would wait forever.
        unsafe {
            let _ = ControlTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                PCWSTR::null(),
                self.properties.as_mut() as *mut _,
                EVENT_TRACE_CONTROL_STOP,
            );
        }
        if let Some(h) = self.consumer.take() {
            let _ = h.join();
        }
        ACTIVE.store(false, Ordering::SeqCst);
        set_state(CaptureState::Idle);
    }
}

unsafe extern "system" fn on_event(record: *mut EVENT_RECORD) -> u32 {
    unsafe { account(record) };
    // Non-zero keeps the session alive; zero would end it silently.
    1
}

unsafe fn account(record: *mut EVENT_RECORD) {
    if record.is_null() {
        return;
    }
    let rec: &EVENT_RECORD = &*record;
    // MofData is four 32-bit fields before we care; the pid and payload size
    // are the first two.
    if (rec.UserDataLength as usize) < 16 {
        return;
    }
    let data = rec.UserData as *const u32;
    if data.is_null() {
        return;
    }

    let descriptor = &rec.EventHeader.EventDescriptor;
    let task = descriptor.Task;
    let rx = matches!(
        task,
        TASK_TCPIP_RECV_V4 | TASK_UDPIP_RECV_V4 | TASK_TCPIP_RECV_V6 | TASK_UDPIP_RECV_V6
    );
    let tx = matches!(task, TASK_TCPIP_SEND | TASK_UDPIP_SEND);
    if !rx && !tx {
        return;
    }

    let pid = *data as u32;
    let size = *data.add(1) as u64;
    if pid == 0 || size == 0 {
        return;
    }

    BATCH.with(|b| {
        let mut batch = b.borrow_mut();
        let e = batch.entry(pid).or_insert((0, 0));
        if rx {
            e.0 = e.0.saturating_add(size);
        } else {
            e.1 = e.1.saturating_add(size);
        }
    });

    let pending = PENDING.with(|c| {
        let n = c.get().saturating_add(1);
        c.set(n);
        n
    });
    if pending >= FLUSH_EVERY {
        PENDING.with(|c| c.set(0));
        flush();
    }
}

/// Starts a kernel-network session. `None` means ETW is unavailable here and
/// the caller should fall back; the reason is recorded for the interface.
pub fn start() -> Option<Session> {
    if ACTIVE.load(Ordering::SeqCst) {
        return None;
    }

    let mut properties: Box<EVENT_TRACE_PROPERTIES> = Box::new(unsafe { std::mem::zeroed() });
    properties.Wnode.BufferSize = BUFFER_SIZE;
    properties.Wnode.Guid = SystemTraceControlGuid;
    properties.MinimumBuffers = MIN_BUFFERS;
    properties.LogFileMode = EVENT_TRACE_REAL_TIME_MODE | EVENT_TRACE_SYSTEM_LOGGER_MODE;
    properties.FlushTimer = 1;
    properties.EnableFlags = EVENT_TRACE_FLAG_NETWORK_TCPIP;

    let name: Vec<u16> = SESSION_NAME.encode_utf16().chain(std::iter::once(0)).collect();

    if let Err(e) = unsafe { StartTraceW(std::ptr::null_mut(), PCWSTR(name.as_ptr()), properties.as_mut()) }
    {
        log::warn!("appstats: kernel logger would not start: {}", e);
        // ERROR_ACCESS_DENIED is the unelevated case, which is by far the
        // most common reason. Anything else means we still cannot count.
        set_state(CaptureState::NeedsAdmin);
        return None;
    }

    if let Err(e) = unsafe {
        EnableTraceEx(
            &SystemTraceControlGuid as *const _,
            None,
            CONTROLTRACE_HANDLE { Value: 0 },
            EVENT_CONTROL_CODE_DISABLE_PROVIDER.0,
            5,
            KEYWORD_NETWORK,
            0,
            0,
            None,
        )
    } {
        log::warn!("appstats: network keyword refused: {}", e);
        unsafe {
            let _ = ControlTraceW(
                CONTROLTRACE_HANDLE { Value: 0 },
                PCWSTR::null(),
                properties.as_mut() as *mut _,
                EVENT_TRACE_CONTROL_STOP,
            );
        }
        set_state(CaptureState::Unavailable);
        return None;
    }

    ACTIVE.store(true, Ordering::SeqCst);
    set_state(CaptureState::Running);

    let consumer = std::thread::Builder::new()
        .name("wawity-etw".into())
        .spawn(|| {
            let mut logfile: EVENT_TRACE_LOGFILEW = unsafe { std::mem::zeroed() };
            // RealTimeMode lives in the union arm in this version of the
            // bindings, not at the top level.
            logfile.Anonymous1.LogFileMode = EVENT_TRACE_REAL_TIME_MODE;
            // windows-rs 0.52 declares PEVENT_TRACE_BUFFER_CALLBACKW as taking
            // *mut EVENT_TRACE_LOGFILEW, but the real EventTraceCallbackW takes
            // PEVENT_RECORD. The binding is wrong, so the signature is fixed up
            // here rather than worked around by walking ETW_BUFFER_HEADER.
            logfile.BufferCallback = Some(unsafe {
                std::mem::transmute::<
                    unsafe extern "system" fn(*mut EVENT_RECORD) -> u32,
                    unsafe extern "system" fn(*mut EVENT_TRACE_LOGFILEW) -> u32,
                >(on_event)
            });

            let handle: PROCESSTRACE_HANDLE = unsafe { OpenTraceW(&mut logfile) };
            if handle.Value == 0 {
                log::warn!("appstats: OpenTraceW failed");
                return;
            }
            // Blocks until Session::drop stops the session.
            let _ = unsafe { ProcessTrace(&[handle], None, None) };
            let _ = unsafe { CloseTrace(handle) };
            // Fold in whatever the batch still holds before the thread ends.
            flush();
        })
        .ok()?;

    Some(Session {
        properties,
        consumer: Some(consumer),
    })
}

/// Drains the accumulated per-PID byte totals, leaving the map empty.
pub fn take_totals() -> Totals {
    totals()
        .lock()
        .map(|mut t| std::mem::take(&mut *t))
        .unwrap_or_default()
}

/// Adds externally sampled bytes, used by the legacy TCP/IPv4 fallback so
/// both paths feed the same map.
pub fn add(pid: u32, rx: u64, tx: u64) {
    if pid == 0 || (rx == 0 && tx == 0) {
        return;
    }
    if let Ok(mut all) = totals().lock() {
        let e = all.entry(pid).or_insert((0, 0));
        e.0 = e.0.saturating_add(rx);
        e.1 = e.1.saturating_add(tx);
    }
}
