use crate::engine::load_blocked_apps_from_disk;
use crate::network::routing::RoutingManager;
use crate::process::ProcessManager;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime};

const TICK: Duration = Duration::from_secs(2);
const TUNNEL_GRACE: Duration = Duration::from_secs(6);
const HANDOVER_STALE: Duration = Duration::from_secs(150);
const SINGBOX_RECHECK: Duration = Duration::from_secs(20);

fn data_file(name: &str) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join("data").join(name))
}

fn always_on_armed() -> bool {
    data_file("always_on.flag")
        .map(|p| p.exists())
        .unwrap_or(false)
}

fn handover_in_progress() -> bool {
    let Some(flag) = data_file("guard_handover.flag") else {
        return false;
    };
    let Ok(meta) = std::fs::metadata(&flag) else {
        return false;
    };
    let age = meta
        .modified()
        .ok()
        .and_then(|stamp| SystemTime::now().duration_since(stamp).ok())
        .unwrap_or(Duration::ZERO);
    if age > HANDOVER_STALE {
        let _ = std::fs::remove_file(&flag);
        log::warn!("guard: dropped a stale handover flag, {}s old", age.as_secs());
        return false;
    }
    true
}

struct Guard {
    routing: RoutingManager,
    processes: Option<ProcessManager>,
    last_tunnel_seen: Option<Instant>,
    singbox_alive: bool,
    singbox_checked: Option<Instant>,
    applied_blocked: Option<Vec<String>>,
}

impl Guard {
    fn new() -> Self {
        let mut guard = Self {
            routing: RoutingManager::new(),
            processes: ProcessManager::new().ok(),
            last_tunnel_seen: None,
            singbox_alive: false,
            singbox_checked: None,
            applied_blocked: None,
        };
        if guard.tunnel_up() {
            log::info!("guard: started while a tunnel is live, standing down");
            return guard;
        }
        if guard.routing.recover_stranded_lockdown() {
            log::warn!("guard: released a stranded lockdown before taking over");
        }
        let blocked = load_blocked_apps_from_disk();
        let _ = guard.routing.set_blocked_apps(&blocked);
        guard
    }

    fn singbox_running(&mut self) -> bool {
        if let Some(at) = self.singbox_checked {
            if at.elapsed() < SINGBOX_RECHECK {
                return self.singbox_alive;
            }
        }
        let alive = self
            .processes
            .as_ref()
            .map(|p| p.orphans_alive())
            .unwrap_or(false);
        self.singbox_alive = alive;
        self.singbox_checked = Some(Instant::now());
        alive
    }

    fn tunnel_up(&mut self) -> bool {
        let up = if crate::network::netinfo::wintun_adapter_present() {
            self.singbox_running()
        } else {
            self.singbox_checked = None;
            false
        };
        if up {
            self.last_tunnel_seen = Some(Instant::now());
            return true;
        }
        match self.last_tunnel_seen {
            Some(last) if last.elapsed() <= TUNNEL_GRACE => true,
            Some(_) => {
                self.last_tunnel_seen = None;
                false
            }
            None => false,
        }
    }

    fn tick(&mut self) {
        if handover_in_progress() {
            return;
        }

        if self.tunnel_up() {
            if self.applied_blocked.is_some() {
                let _ = self.routing.set_tunnel_active(true);
                self.applied_blocked = None;
            }
            return;
        }

        let _ = self.routing.set_tunnel_active(false);

        let armed = always_on_armed();
        let is_always_on = self.routing.is_always_on_active();

        if armed && !is_always_on {
            match crate::engine::resolve_own_exe_path() {
                Ok(app_exe) => {
                    if let Err(e) = self.routing.enable_always_on(&app_exe) {
                        log::error!("guard: cannot engage the lockdown: {}", e);
                    } else {
                        log::info!("guard: lockdown engaged");
                    }
                }
                Err(e) => log::error!("guard: cannot resolve the app path: {}", e),
            }
        } else if !armed && is_always_on {
            let _ = self.routing.disable_always_on();
            log::info!("guard: lockdown released");
        } else if armed {
            if let Ok(true) = self.routing.verify_and_repair() {
                log::warn!("guard: firewall drift repaired");
            }
        }

        let wanted = load_blocked_apps_from_disk();
        if self.applied_blocked.as_deref() != Some(wanted.as_slice()) {
            if let Err(e) = self.routing.set_blocked_apps(&wanted) {
                log::error!("guard: per-app protection failed: {}", e);
            }
            self.applied_blocked = Some(wanted);
        }
    }
}

pub fn run() -> ! {
    log::info!("guard: starting");
    let mut guard = Guard::new();
    loop {
        guard.tick();
        std::thread::sleep(TICK);
    }
}
