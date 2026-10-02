//! Guard process: keeps the Always-On lockdown enforced while the desktop app
//! is not running.
//!
//! It is launched by the `WawityGuard` scheduled task at boot, before any user
//! logs on, and runs until shutdown. Every tick it looks at the real state of
//! the machine rather than at anything the GUI believes:
//!
//! * a tunnel is up      -> stand down, the app owns the firewall
//! * always-on is armed  -> hold the lockdown and re-assert it if it drifts
//! * neither             -> make sure nothing of ours is left blocking
//!
//! It also heals a lockdown whose owner died, which is the case where a user
//! would otherwise be stuck without internet and with no GUI to click.

use crate::engine::load_blocked_apps_from_disk;
use crate::network::routing::RoutingManager;
use crate::process::ProcessManager;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const TICK: Duration = Duration::from_secs(2);
/// A tunnel that has been gone for this long means the app really disconnected
/// rather than being mid-restart.
const TUNNEL_GRACE: Duration = Duration::from_secs(6);

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

/// Set by the app right before it starts or stops a tunnel so the guard does
/// not fight it during the handover.
fn handover_flag() -> Option<PathBuf> {
    data_file("guard_handover.flag")
}

struct Guard {
    routing: RoutingManager,
    processes: Option<ProcessManager>,
    last_tunnel_seen: Option<Instant>,
    applied_blocked: Option<Vec<String>>,
}

impl Guard {
    fn new() -> Self {
        let processes = ProcessManager::new().ok();
        let mut routing = RoutingManager::new();
        // A lockdown from a previous boot is still in place; take ownership of
        // it instead of stacking another one on top.
        if routing.recover_stranded_lockdown() {
            log::warn!("guard: released a stranded lockdown before taking over");
        }
        let blocked = load_blocked_apps_from_disk();
        let _ = routing.set_blocked_apps(&blocked);
        Self {
            routing,
            processes,
            last_tunnel_seen: None,
            applied_blocked: None,
        }
    }

    fn tunnel_up(&mut self) -> bool {
        let up = self
            .processes
            .as_ref()
            .map(|p| p.is_running())
            .unwrap_or(false);
        if up {
            self.last_tunnel_seen = Some(Instant::now());
        } else if let Some(last) = self.last_tunnel_seen {
            if last.elapsed() > TUNNEL_GRACE {
                self.last_tunnel_seen = None;
            }
        }
        up || self
            .last_tunnel_seen
            .map(|l| l.elapsed() <= TUNNEL_GRACE)
            .unwrap_or(false)
    }

    fn tick(&mut self) {
        if handover_flag().map(|p| p.exists()).unwrap_or(false) {
            // The app is mid-operation; do not touch anything.
            return;
        }

        let tunnel = self.tunnel_up();
        if tunnel {
            // Lift per-app blocks: they would also stop packets on their way
            // into the TUN gateway.
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
            // Re-assert the block if anything (or anyone) changed it.
            if let Ok(true) = self.routing.verify_and_repair() {
                log::warn!("guard: firewall drift repaired");
            }
        }

        // Keep the per-app protection list in sync with the file the app writes.
        let wanted = load_blocked_apps_from_disk();
        if self.applied_blocked.as_deref() != Some(wanted.as_slice()) {
            if let Err(e) = self.routing.set_blocked_apps(&wanted) {
                log::error!("guard: per-app protection failed: {}", e);
            }
            self.applied_blocked = Some(wanted);
        }
    }
}

/// Blocking entry point for the scheduled task. Never returns normally.
pub fn run() -> ! {
    log::info!("guard: starting");
    let mut guard = Guard::new();
    loop {
        guard.tick();
        std::thread::sleep(TICK);
    }
}
