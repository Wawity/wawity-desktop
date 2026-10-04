use crate::error::VpnError;
use crate::network::winfw::{self, FirewallRule, ProfileState, PROTO_TCP, PROTO_UDP};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const RULE_APP_OUT: &str = "WawityFW_AllowApp_Out";
const RULE_APP_IN: &str = "WawityFW_AllowApp_In";
const RULE_LOOPBACK_OUT_V4: &str = "WawityFW_AllowLoopbackV4_Out";
const RULE_LOOPBACK_IN_V4: &str = "WawityFW_AllowLoopbackV4_In";
const RULE_LOOPBACK_OUT_V6: &str = "WawityFW_AllowLoopbackV6_Out";
const RULE_LOOPBACK_IN_V6: &str = "WawityFW_AllowLoopbackV6_In";
const RULE_DHCP_OUT: &str = "WawityFW_AllowDhcp_Out";
const RULE_DHCP_IN: &str = "WawityFW_AllowDhcp_In";
const RULE_SINGBOX_OUT: &str = "WawityFW_AllowSingbox_Out";
const RULE_SINGBOX_IN: &str = "WawityFW_AllowSingbox_In";
const RULE_TUN_OUT_V4: &str = "WawityFW_AllowTunV4_Out";
const RULE_TUN_IN_V4: &str = "WawityFW_AllowTunV4_In";
const RULE_TUN_OUT_V6: &str = "WawityFW_AllowTunV6_Out";
const RULE_TUN_IN_V6: &str = "WawityFW_AllowTunV6_In";
const RULE_SERVER_OUT: &str = "WawityFW_AllowServer_Out";
const RULE_BOOTSTRAP_DNS_OUT: &str = "WawityFW_AllowBootstrapDns_Out";
const BYPASS_RULE_PREFIX_OUT: &str = "WawityFW_BypassApp_Out_";
const BYPASS_RULE_PREFIX_IN: &str = "WawityFW_BypassApp_In_";
const BLOCK_APP_RULE_PREFIX_OUT: &str = "WawityFW_BlockApp_Out_";
const BLOCK_APP_RULE_PREFIX_IN: &str = "WawityFW_BlockApp_In_";
const RULE_GUARD_DNS_UDP: &str = "WawityFW_BlockLanDns_Udp";
const RULE_GUARD_DNS_TCP: &str = "WawityFW_BlockLanDns_Tcp";
const RULE_GUARD_LLMNR: &str = "WawityFW_BlockLlmnr_Out";
const RULE_GUARD_MDNS: &str = "WawityFW_BlockMdns_Out";
const RULE_GUARD_NBNS: &str = "WawityFW_BlockNbns_Out";

const DNS_GUARD_RULE_NAMES: &[&str] = &[
    RULE_GUARD_DNS_UDP,
    RULE_GUARD_DNS_TCP,
    RULE_GUARD_LLMNR,
    RULE_GUARD_MDNS,
    RULE_GUARD_NBNS,
];

const BASE_RULE_NAMES: &[&str] = &[
    RULE_APP_OUT,
    RULE_APP_IN,
    RULE_LOOPBACK_OUT_V4,
    RULE_LOOPBACK_IN_V4,
    RULE_LOOPBACK_OUT_V6,
    RULE_LOOPBACK_IN_V6,
    RULE_DHCP_OUT,
    RULE_DHCP_IN,
];

const SINGBOX_TUN_RULE_NAMES: &[&str] = &[
    RULE_SINGBOX_OUT,
    RULE_SINGBOX_IN,
    RULE_TUN_OUT_V4,
    RULE_TUN_IN_V4,
    RULE_TUN_OUT_V6,
    RULE_TUN_IN_V6,
];

const LEGACY_RULE_NAMES: &[&str] = &[
    "WawityKS_AllowSingbox_Out",
    "WawityKS_AllowSingbox_In",
    "WawityKS_AllowApp_Out",
    "WawityKS_AllowApp_In",
    "WawityKS_AllowLoopbackV4_Out",
    "WawityKS_AllowLoopbackV4_In",
    "WawityKS_AllowLoopbackV6_Out",
    "WawityKS_AllowLoopbackV6_In",
    "WawityKS_AllowTunV4_Out",
    "WawityKS_AllowTunV4_In",
    "WawityKS_AllowTunV6_Out",
    "WawityKS_AllowTunV6_In",
    "WawityKS_AllowDhcp_Out",
    "WawityKS_AllowDhcp_In",
    "WawityKS_BlockAll_Out",
    "WawityKS_BlockAll_In",
];

const LAN_DNS_BLOCK_RANGES: &str =
    "10.0.0.0/8,172.16.0.0-172.18.255.255,172.19.0.4-172.31.255.255,192.168.0.0/16,169.254.0.0/16";
const TUN_SUBNET_V4: &str = "172.19.0.0/30";
const TUN_SUBNET_V6: &str = "fdfe:dcba:9876::/126";
const LOOPBACK_V4: &str = "127.0.0.0/8";
const LOOPBACK_V6: &str = "::1/128";
const MAX_STALE_BYPASS_RULES: usize = 128;
const MAX_BLOCK_APP_RULES: usize = 128;

fn all_static_rule_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = Vec::new();
    names.extend_from_slice(BASE_RULE_NAMES);
    names.extend_from_slice(SINGBOX_TUN_RULE_NAMES);
    names.extend_from_slice(DNS_GUARD_RULE_NAMES);
    names.push(RULE_SERVER_OUT);
    names.push(RULE_BOOTSTRAP_DNS_OUT);
    names
}

fn bypass_rule_name_out(idx: usize) -> String {
    format!("{}{}", BYPASS_RULE_PREFIX_OUT, idx)
}

fn bypass_rule_name_in(idx: usize) -> String {
    format!("{}{}", BYPASS_RULE_PREFIX_IN, idx)
}

fn block_app_rule_name_out(idx: usize) -> String {
    format!("{}{}", BLOCK_APP_RULE_PREFIX_OUT, idx)
}

fn block_app_rule_name_in(idx: usize) -> String {
    format!("{}{}", BLOCK_APP_RULE_PREFIX_IN, idx)
}

fn program_block_rule(name: &str, outbound: bool, program: &str) -> FirewallRule {
    FirewallRule {
        name: name.to_string(),
        outbound,
        allow: false,
        program: Some(program.to_string()),
        ..Default::default()
    }
}

fn program_allow_rule(name: &str, outbound: bool, program: &str) -> FirewallRule {
    FirewallRule {
        name: name.to_string(),
        outbound,
        allow: true,
        program: Some(program.to_string()),
        ..Default::default()
    }
}

fn addr_allow_rule(name: &str, outbound: bool, local: bool, addresses: &str) -> FirewallRule {
    let mut rule = FirewallRule {
        name: name.to_string(),
        outbound,
        allow: true,
        ..Default::default()
    };
    if local {
        rule.local_addresses = Some(addresses.to_string());
    } else {
        rule.remote_addresses = Some(addresses.to_string());
    }
    rule
}

fn dhcp_allow_rule(name: &str, outbound: bool) -> FirewallRule {
    FirewallRule {
        name: name.to_string(),
        outbound,
        allow: true,
        protocol: Some(PROTO_UDP),
        local_ports: Some("68".to_string()),
        remote_ports: Some("67".to_string()),
        ..Default::default()
    }
}

fn dns_block_rule(
    name: &str,
    protocol: i32,
    remote_ports: &str,
    remote_addresses: Option<&str>,
) -> FirewallRule {
    FirewallRule {
        name: name.to_string(),
        outbound: true,
        allow: false,
        protocol: Some(protocol),
        remote_ports: Some(remote_ports.to_string()),
        remote_addresses: remote_addresses.map(|v| v.to_string()),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FirewallMode {
    Normal,
    AlwaysOnIdle,
    Connected,
}

/// Crash-safe journal describing a lockdown we applied but have not released
/// yet. It is flushed to disk *before* the firewall profiles are flipped, so a
/// `taskkill /F`, a panic or a power loss still leaves behind everything needed
/// to put the machine back online on the next launch.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LockdownJournal {
    /// Schema marker so a future format change can be detected instead of
    /// being misread as "no baseline available".
    version: u32,
    armed: bool,
    baseline: Vec<ProfileState>,
}

const JOURNAL_VERSION: u32 = 1;
const JOURNAL_FILE: &str = "firewall_state.json";

fn journal_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    Some(dir.join("data").join(JOURNAL_FILE))
}

fn read_journal() -> Option<LockdownJournal> {
    let path = journal_path()?;
    let raw = std::fs::read_to_string(&path).ok()?;
    match serde_json::from_str::<LockdownJournal>(&raw) {
        Ok(j) if j.version == JOURNAL_VERSION && j.armed => Some(j),
        Ok(_) => {
            log::info!("firewall journal present but disarmed, ignoring");
            None
        }
        Err(e) => {
            log::error!("firewall journal unreadable ({}), falling back to safe defaults", e);
            None
        }
    }
}

fn write_journal(baseline: &[ProfileState]) {
    let Some(path) = journal_path() else { return };
    if let Some(dir) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            log::error!("cannot create {} for the firewall journal: {}", dir.display(), e);
            return;
        }
    }
    let journal = LockdownJournal {
        version: JOURNAL_VERSION,
        armed: true,
        baseline: baseline.to_vec(),
    };
    match serde_json::to_string_pretty(&journal) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::error!("cannot persist the firewall journal to {}: {}", path.display(), e);
            }
        }
        Err(e) => log::error!("cannot serialize the firewall journal: {}", e),
    }
}

fn clear_journal() {
    if let Some(path) = journal_path() {
        if let Err(e) = std::fs::remove_file(&path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                log::warn!("cannot remove the firewall journal {}: {}", path.display(), e);
            }
        }
    }
}

pub struct RoutingManager {
    mode: FirewallMode,
    saved_policy: Vec<ProfileState>,
    active_bypass_count: usize,
    include_mode: bool,
    installed_app_path: Option<String>,
    installed_singbox_path: Option<String>,
    installed_bypass_paths: Vec<String>,
    /// Programs denied internet whenever no tunnel is up. Unlike the bypass
    /// list these are *block* rules, so they must be lifted while a tunnel is
    /// up: a program block also stops the packet before it reaches the TUN
    /// gateway, which would cut the app off even through the VPN.
    blocked_apps: Vec<String>,
    active_block_app_count: usize,
    tunnel_active: bool,
}

impl RoutingManager {
    pub fn new() -> Self {
        Self {
            mode: FirewallMode::Normal,
            saved_policy: read_journal().map(|j| j.baseline).unwrap_or_default(),
            active_bypass_count: 0,
            include_mode: false,
            installed_app_path: None,
            installed_singbox_path: None,
            installed_bypass_paths: Vec::new(),
            blocked_apps: Vec::new(),
            active_block_app_count: 0,
            tunnel_active: false,
        }
    }

    /// Replaces the per-app protection list. Block rules are installed only
    /// while no tunnel is up, so this is safe to call at any time.
    pub fn set_blocked_apps(&mut self, paths: &[String]) -> Result<(), VpnError> {
        let normalized = crate::util::normalize_path_list(paths);
        self.remove_block_app_rules();
        self.blocked_apps = normalized.clone();
        if self.tunnel_active || normalized.is_empty() {
            return Ok(());
        }
        self.install_block_app_rules(&normalized)
    }

    pub fn blocked_apps(&self) -> Vec<String> {
        self.blocked_apps.clone()
    }

    /// Tells the manager whether a tunnel is carrying traffic. Lifts the
    /// per-app blocks on connect and re-arms them on disconnect.
    pub fn set_tunnel_active(&mut self, active: bool) -> Result<(), VpnError> {
        if self.tunnel_active == active {
            return Ok(());
        }
        self.tunnel_active = active;
        if active {
            self.remove_block_app_rules();
            return Ok(());
        }
        let paths = self.blocked_apps.clone();
        self.install_block_app_rules(&paths)
    }

    fn install_block_app_rules(&mut self, paths: &[String]) -> Result<(), VpnError> {
        self.active_block_app_count = 0;
        if self.tunnel_active {
            return Ok(());
        }
        for (idx, path) in paths.iter().take(MAX_BLOCK_APP_RULES).enumerate() {
            if path.trim().is_empty() {
                continue;
            }
            if let Err(e) =
                winfw::add_rule(&program_block_rule(&block_app_rule_name_out(idx), true, path))
            {
                self.active_block_app_count = idx;
                return Err(VpnError::NetworkError(format!(
                    "per-app protection failed: {}",
                    e
                )));
            }
            if let Err(e) =
                winfw::add_rule(&program_block_rule(&block_app_rule_name_in(idx), false, path))
            {
                self.active_block_app_count = idx;
                return Err(VpnError::NetworkError(format!(
                    "per-app protection failed: {}",
                    e
                )));
            }
        }
        self.active_block_app_count = paths.len().min(MAX_BLOCK_APP_RULES);
        Ok(())
    }

    fn remove_block_app_rules(&mut self) {
        let upto = self
            .active_block_app_count
            .max(self.blocked_apps.len().min(MAX_BLOCK_APP_RULES));
        for idx in 0..upto {
            winfw::remove_rule(&block_app_rule_name_out(idx));
            winfw::remove_rule(&block_app_rule_name_in(idx));
        }
        self.active_block_app_count = 0;
    }

    /// Rolls back a lockdown that outlived its process (crash, force kill,
    /// power loss, uninstall). Safe to call when no lockdown was left behind.
    /// Call this before re-engaging any deliberate lockdown.
    pub fn recover_stranded_lockdown(&mut self) -> bool {
        let Some(journal) = read_journal() else {
            // A previous version of the app could leave the profiles blocked
            // without ever writing a journal. Detect that on the live system.
            if winfw::live_outbound_blocked() {
                log::warn!(
                    "outbound traffic is blocked but no lockdown is tracked by this process, \
                     forcing firewall defaults back to allow"
                );
                self.restore_from(&Vec::new());
                self.wipe_all_rules();
                self.mode = FirewallMode::Normal;
                return true;
            }
            return false;
        };
        log::warn!("recovering a firewall lockdown left over from a previous run");
        self.saved_policy = journal.baseline;
        self.restore_policy();
        self.wipe_all_rules();
        self.active_bypass_count = 0;
        self.installed_app_path = None;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();
        self.mode = FirewallMode::Normal;
        true
    }

    /// Releases the live lockdown without touching process state. Used by the
    /// kill-switch toggle so that flipping the switch off is immediately
    /// effective instead of only applying to the next connection.
    pub fn disable_kill_switch(&mut self) -> Result<(), VpnError> {
        winfw::remove_rules(SINGBOX_TUN_RULE_NAMES);
        winfw::remove_rule(RULE_SERVER_OUT);
        self.remove_bypass_rules(self.active_bypass_count);
        self.active_bypass_count = 0;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();

        match self.mode {
            // Always-on is a separate, explicitly requested lockdown: leave it
            // in place.
            FirewallMode::AlwaysOnIdle => return Ok(()),
            FirewallMode::Connected => {
                winfw::remove_rules(BASE_RULE_NAMES);
                self.restore_policy();
                self.installed_app_path = None;
                self.mode = FirewallMode::Normal;
                Ok(())
            }
            FirewallMode::Normal => {
                if self.saved_policy.is_empty() && winfw::live_outbound_blocked() {
                    winfw::remove_rules(BASE_RULE_NAMES);
                    self.restore_policy();
                    self.installed_app_path = None;
                }
                Ok(())
            }
        }
    }

    pub fn enable_always_on(&mut self, app_exe_path: &str) -> Result<(), VpnError> {
        if self.mode != FirewallMode::Normal {
            return Ok(());
        }
        if app_exe_path.trim().is_empty() {
            return Err(VpnError::NetworkError("empty app executable path".into()));
        }
        self.saved_policy = self.read_baseline_policy()?;
        self.wipe_all_rules();
        if let Err(e) = self.install_base_rules(app_exe_path) {
            self.restore_policy();
            self.wipe_all_rules();
            return Err(e);
        }
        self.arm_lockdown()?;
        self.mode = FirewallMode::AlwaysOnIdle;
        Ok(())
    }

    pub fn disable_always_on(&mut self) -> Result<(), VpnError> {
        if self.mode != FirewallMode::AlwaysOnIdle {
            return Ok(());
        }
        self.restore_policy();
        self.wipe_all_rules();
        self.installed_app_path = None;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();
        self.mode = FirewallMode::Normal;
        Ok(())
    }

    pub fn stage_exceptions(&mut self, singbox_path: &str, app_exe_path: &str) -> Result<(), VpnError> {
        if self.mode == FirewallMode::Connected {
            return Ok(());
        }
        if singbox_path.trim().is_empty() {
            return Err(VpnError::NetworkError("empty sing-box path".into()));
        }
        if app_exe_path.trim().is_empty() {
            return Err(VpnError::NetworkError("empty app executable path".into()));
        }
        let previous_mode = self.mode;
        let staged_from_normal = previous_mode == FirewallMode::Normal;
        let result = (|| -> Result<(), VpnError> {
            if staged_from_normal {
                self.saved_policy = self.read_baseline_policy()?;
                self.wipe_all_rules();
                self.install_base_rules(app_exe_path)?;
            }
            self.install_singbox_rules(singbox_path)?;
            if staged_from_normal {
                self.arm_lockdown()?;
            }
            Ok(())
        })();
        if let Err(e) = result {
            // Never leave the firewall blocked (or the allow-list half
            // installed) after a failed stage.
            self.abort_staged_connection(previous_mode == FirewallMode::AlwaysOnIdle);
            return Err(e);
        }
        Ok(())
    }

    pub fn commit_connection(&mut self) -> Result<(), VpnError> {
        if self.mode == FirewallMode::Connected {
            return Ok(());
        }
        if self.mode == FirewallMode::Normal {
            self.arm_lockdown()?;
        }
        self.mode = FirewallMode::Connected;
        // Traffic now flows through sing-box: lift the per-app blocks so the
        // protected apps can still use the tunnel.
        self.set_tunnel_active(true)
    }

    /// Arms the kill switch on an already-running tunnel. Every step is rolled
    /// back on failure so a partially applied lockdown can never strand the
    /// machine without internet.
    pub fn engage_lockdown(
        &mut self,
        singbox_path: &str,
        app_exe_path: &str,
        bypass_paths: &[String],
        include_mode: bool,
    ) -> Result<(), VpnError> {
        if self.is_kill_switch_active() {
            return Ok(());
        }
        self.stage_exceptions(singbox_path, app_exe_path)?;
        if !bypass_paths.is_empty() {
            if let Err(e) = self.update_bypass_rules(bypass_paths, include_mode) {
                self.abort_staged_connection(false);
                return Err(e);
            }
        }
        if let Err(e) = self.commit_connection() {
            self.abort_staged_connection(false);
            return Err(e);
        }
        Ok(())
    }

    pub fn allow_server_endpoint(&mut self, host: &str, resolver: &str) -> Result<String, VpnError> {
        let trimmed = host.trim();
        if trimmed.is_empty() {
            return Err(VpnError::NetworkError("empty server host".into()));
        }
        let ips = crate::engine::resolve_server_ips_bootstrap(trimmed, resolver)
            .map_err(VpnError::NetworkError)?;
        self.allow_server_ips(&ips)
    }

    pub fn allow_server_ips(&mut self, ips: &[String]) -> Result<String, VpnError> {
        let clean: Vec<String> = ips
            .iter()
            .filter(|s| s.parse::<std::net::IpAddr>().is_ok())
            .cloned()
            .collect();
        if clean.is_empty() {
            return Err(VpnError::NetworkError("no valid server ips".into()));
        }
        winfw::remove_rule(RULE_SERVER_OUT);
        winfw::remove_rule(RULE_BOOTSTRAP_DNS_OUT);
        let resolved = clean.join(",");
        winfw::add_rule(&addr_allow_rule(RULE_SERVER_OUT, true, false, &resolved))?;
        log::info!("firewall allows server endpoint {}", resolved);
        Ok(resolved)
    }

    pub fn abort_staged_connection(&mut self, was_always_on_idle: bool) {
        winfw::remove_rules(DNS_GUARD_RULE_NAMES);
        winfw::remove_rules(SINGBOX_TUN_RULE_NAMES);
        winfw::remove_rule(RULE_SERVER_OUT);
        self.remove_bypass_rules(self.active_bypass_count);
        self.active_bypass_count = 0;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();
        if was_always_on_idle {
            self.mode = FirewallMode::AlwaysOnIdle;
        } else {
            winfw::remove_rules(BASE_RULE_NAMES);
            self.restore_policy();
            self.installed_app_path = None;
            self.mode = FirewallMode::Normal;
        }
    }

    pub fn end_connection(&mut self, keep_locked_down: bool) -> Result<(), VpnError> {
        // No tunnel any more: re-arm the per-app blocks.
        self.tunnel_active = false;
        if self.mode != FirewallMode::Connected {
            // The in-memory mode can be Normal while the machine is still
            // blocked (lockdown applied by an earlier run, or a stage that
            // failed midway). Never let a disconnect leave traffic blocked.
            if self.saved_policy.is_empty() && winfw::live_outbound_blocked() {
                log::warn!("disconnect found a live firewall lockdown with no tracked session, forcing release");
                self.restore_policy();
                self.wipe_all_rules();
            }
            let paths = self.blocked_apps.clone();
            return self.install_block_app_rules(&paths);
        }
        winfw::remove_rules(SINGBOX_TUN_RULE_NAMES);
        winfw::remove_rule(RULE_SERVER_OUT);
        self.remove_bypass_rules(self.active_bypass_count);
        self.active_bypass_count = 0;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();
        if keep_locked_down {
            self.mode = FirewallMode::AlwaysOnIdle;
        } else {
            winfw::remove_rules(BASE_RULE_NAMES);
            self.restore_policy();
            self.installed_app_path = None;
            self.mode = FirewallMode::Normal;
        }
        let paths = self.blocked_apps.clone();
        self.install_block_app_rules(&paths)
    }

    pub fn update_bypass_rules(&mut self, paths: &[String], include_mode: bool) -> Result<(), VpnError> {
        let normalized = crate::util::normalize_path_list(paths);
        let old_count = self.active_bypass_count;
        self.remove_bypass_rules(old_count);
        self.active_bypass_count = 0;
        self.installed_bypass_paths.clear();
        self.include_mode = include_mode;

        if normalized.is_empty() {
            super::qos::clear_async();
            return Ok(());
        }

        if include_mode {
            self.installed_bypass_paths = normalized;
            super::qos::clear_async();
            return Ok(());
        }

        let lockdown_active = matches!(self.mode, FirewallMode::AlwaysOnIdle | FirewallMode::Connected)
            || self.installed_singbox_path.is_some();
        if !lockdown_active {
            super::qos::refresh_async(normalized, self.installed_singbox_path.clone());
            return Ok(());
        }

        for (idx, path) in normalized.iter().enumerate() {
            if path.trim().is_empty() {
                continue;
            }
            if let Err(e) = winfw::add_rule(&program_allow_rule(&bypass_rule_name_out(idx), true, path)) {
                self.active_bypass_count = idx;
                return Err(VpnError::NetworkError(format!(
                    "bypass firewall rules failed: {}",
                    e
                )));
            }
            if let Err(e) = winfw::add_rule(&program_allow_rule(&bypass_rule_name_in(idx), false, path)) {
                self.active_bypass_count = idx + 1;
                return Err(VpnError::NetworkError(format!(
                    "bypass firewall rules failed: {}",
                    e
                )));
            }
        }

        self.active_bypass_count = normalized.len();
        self.installed_bypass_paths = normalized;
        super::qos::refresh_async(
            self.installed_bypass_paths.clone(),
            self.installed_singbox_path.clone(),
        );
        Ok(())
    }

    pub fn verify_and_repair(&mut self) -> Result<bool, VpnError> {
        if !matches!(self.mode, FirewallMode::AlwaysOnIdle | FirewallMode::Connected) {
            return Ok(false);
        }
        let mut repaired = false;
        let profiles = match winfw::read_profiles() {
            Ok(profiles) if !profiles.is_empty() => profiles,
            Ok(_) => {
                log::error!("firewall returned no profiles, skipping lockdown drift check");
                return Ok(false);
            }
            Err(e) => {
                // Never re-apply the block on an unreadable policy: guessing
                // here is what strands users without internet.
                log::error!("cannot read firewall profiles, skipping lockdown drift check: {}", e);
                return Ok(false);
            }
        };
        let policy_drifted = profiles
            .iter()
            .any(|p| !p.enabled || !p.inbound_block || !p.outbound_block);
        if policy_drifted && winfw::apply_block_all().is_ok() {
            repaired = true;
        }
        if !winfw::rule_exists(RULE_APP_OUT)
            || !winfw::rule_exists(RULE_LOOPBACK_OUT_V4)
            || !winfw::rule_exists(RULE_DHCP_OUT)
        {
            if let Some(app_path) = self.installed_app_path.clone() {
                if self.install_base_rules(&app_path).is_ok() {
                    repaired = true;
                }
            }
        }
        if self.mode == FirewallMode::Connected {
            if !winfw::rule_exists(RULE_SINGBOX_OUT) || !winfw::rule_exists(RULE_TUN_OUT_V4) {
                if let Some(singbox_path) = self.installed_singbox_path.clone() {
                    if self.install_singbox_rules(&singbox_path).is_ok() {
                        repaired = true;
                    }
                }
            }
            let expected_bypass = self.installed_bypass_paths.clone();
            if !expected_bypass.is_empty() && !winfw::rule_exists(&bypass_rule_name_out(0)) {
                if self.update_bypass_rules(&expected_bypass, self.include_mode).is_ok() {
                    repaired = true;
                }
            }
        }
        Ok(repaired)
    }

    pub fn enable_dns_leak_guard(&self) -> Result<(), VpnError> {
        winfw::remove_rules(DNS_GUARD_RULE_NAMES);
        winfw::add_rule(&dns_block_rule(
            RULE_GUARD_DNS_UDP,
            PROTO_UDP,
            "53",
            Some(LAN_DNS_BLOCK_RANGES),
        ))
        .map_err(|e| VpnError::NetworkError(format!("dns leak guard failed: {}", e)))?;
        winfw::add_rule(&dns_block_rule(
            RULE_GUARD_DNS_TCP,
            PROTO_TCP,
            "53",
            Some(LAN_DNS_BLOCK_RANGES),
        ))
        .map_err(|e| VpnError::NetworkError(format!("dns leak guard failed: {}", e)))?;
        winfw::add_rule(&dns_block_rule(RULE_GUARD_LLMNR, PROTO_UDP, "5355", None))
            .map_err(|e| VpnError::NetworkError(format!("dns leak guard failed: {}", e)))?;
        winfw::add_rule(&dns_block_rule(RULE_GUARD_MDNS, PROTO_UDP, "5353", None))
            .map_err(|e| VpnError::NetworkError(format!("dns leak guard failed: {}", e)))?;
        winfw::add_rule(&dns_block_rule(RULE_GUARD_NBNS, PROTO_UDP, "137,138", None))
            .map_err(|e| VpnError::NetworkError(format!("dns leak guard failed: {}", e)))?;
        Ok(())
    }

    pub fn disable_dns_leak_guard(&self) {
        winfw::remove_rules(DNS_GUARD_RULE_NAMES);
    }

    pub fn is_kill_switch_active(&self) -> bool {
        self.mode == FirewallMode::Connected
    }

    pub fn is_always_on_active(&self) -> bool {
        matches!(self.mode, FirewallMode::AlwaysOnIdle | FirewallMode::Connected)
    }

    pub fn force_cleanup(&mut self) {
        super::qos::clear_async();
        self.restore_policy();
        self.wipe_all_rules();
        self.active_bypass_count = 0;
        self.active_block_app_count = 0;
        self.tunnel_active = false;
        self.installed_app_path = None;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();
        self.mode = FirewallMode::Normal;
    }

    /// Public DoH resolvers, kept reachable while the lockdown is armed.
    ///
    /// These are exactly the endpoints `resolve_server_ips_bootstrap` needs in
    /// order to turn a server hostname into an address the allow-rule can
    /// name. The rule name existed but nothing ever created it, so switching
    /// servers while locked down could never resolve the new host: the DoH
    /// round trip was blocked by our own block-all, the system resolver was
    /// blocked by the DNS leak guard, and the switch failed with os error
    /// 10054 / 11001.
    const BOOTSTRAP_DNS_IPS: &'static str = "1.1.1.1,1.0.0.1,8.8.8.8,8.8.4.4,9.9.9.9,149.112.112.112,208.67.222.222,208.67.220.220";

    fn install_bootstrap_dns_rule(&self) {
        if let Err(e) = winfw::add_rule(&addr_allow_rule(
            RULE_BOOTSTRAP_DNS_OUT,
            true,
            false,
            Self::BOOTSTRAP_DNS_IPS,
        )) {
            log::warn!("could not allow the DoH resolvers through the firewall: {}", e);
        }
    }

    fn install_base_rules(&mut self, app_exe_path: &str) -> Result<(), VpnError> {
        let normalized_path = crate::util::normalize_windows_path(app_exe_path);
        winfw::add_rule(&program_allow_rule(RULE_APP_OUT, true, &normalized_path))
            .map_err(|e| VpnError::NetworkError(format!("base firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&program_allow_rule(RULE_APP_IN, false, &normalized_path))
            .map_err(|e| VpnError::NetworkError(format!("base firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&addr_allow_rule(RULE_LOOPBACK_OUT_V4, true, false, LOOPBACK_V4))
            .map_err(|e| VpnError::NetworkError(format!("base firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&addr_allow_rule(RULE_LOOPBACK_IN_V4, false, true, LOOPBACK_V4))
            .map_err(|e| VpnError::NetworkError(format!("base firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&dhcp_allow_rule(RULE_DHCP_OUT, true))
            .map_err(|e| VpnError::NetworkError(format!("base firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&dhcp_allow_rule(RULE_DHCP_IN, false))
            .map_err(|e| VpnError::NetworkError(format!("base firewall rule setup failed: {}", e)))?;
        if let Err(e) = winfw::add_rule(&addr_allow_rule(RULE_LOOPBACK_OUT_V6, true, false, LOOPBACK_V6)) {
            log::warn!("ipv6 loopback exception skipped: {}", e);
        }
        if let Err(e) = winfw::add_rule(&addr_allow_rule(RULE_LOOPBACK_IN_V6, false, true, LOOPBACK_V6)) {
            log::warn!("ipv6 loopback exception skipped: {}", e);
        }
        self.installed_app_path = Some(normalized_path);
        self.install_bootstrap_dns_rule();
        Ok(())
    }

    fn install_singbox_rules(&mut self, singbox_path: &str) -> Result<(), VpnError> {
        let normalized_path = crate::util::normalize_windows_path(singbox_path);
        winfw::add_rule(&program_allow_rule(RULE_SINGBOX_OUT, true, &normalized_path))
            .map_err(|e| VpnError::NetworkError(format!("sing-box firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&program_allow_rule(RULE_SINGBOX_IN, false, &normalized_path))
            .map_err(|e| VpnError::NetworkError(format!("sing-box firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&addr_allow_rule(RULE_TUN_OUT_V4, true, true, TUN_SUBNET_V4))
            .map_err(|e| VpnError::NetworkError(format!("sing-box firewall rule setup failed: {}", e)))?;
        winfw::add_rule(&addr_allow_rule(RULE_TUN_IN_V4, false, false, TUN_SUBNET_V4))
            .map_err(|e| VpnError::NetworkError(format!("sing-box firewall rule setup failed: {}", e)))?;
        if let Err(e) = winfw::add_rule(&addr_allow_rule(RULE_TUN_OUT_V6, true, true, TUN_SUBNET_V6)) {
            log::warn!("ipv6 tun exception skipped: {}", e);
        }
        if let Err(e) = winfw::add_rule(&addr_allow_rule(RULE_TUN_IN_V6, false, false, TUN_SUBNET_V6)) {
            log::warn!("ipv6 tun exception skipped: {}", e);
        }
        self.installed_singbox_path = Some(normalized_path);
        Ok(())
    }

    /// Captures the pre-lockdown firewall policy. A journal left behind by an
    /// older run is authoritative: reading live profiles while they are blocked
    /// would record "block" as the baseline and make the block unrecoverable.
    fn read_baseline_policy(&self) -> Result<Vec<ProfileState>, VpnError> {
        if !self.saved_policy.is_empty() {
            return Ok(self.saved_policy.clone());
        }
        let mut policy = winfw::read_profiles()?;
        for p in policy.iter_mut() {
            p.outbound_block = false;
        }
        Ok(policy)
    }

    /// Flushes the baseline to disk *before* flipping the profiles, so an
    /// abrupt termination still leaves a recoverable trace.
    fn arm_lockdown(&mut self) -> Result<(), VpnError> {
        if self.saved_policy.is_empty() {
            self.saved_policy = self.read_baseline_policy()?;
        }
        write_journal(&self.saved_policy);
        if let Err(e) = winfw::apply_block_all() {
            // The journal is already on disk, so startup recovery will undo it.
            return Err(e);
        }
        Ok(())
    }

    fn restore_from(&mut self, baseline: &[ProfileState]) {
        let result = if baseline.is_empty() {
            winfw::apply_safe_defaults()
        } else {
            winfw::apply_profiles(baseline)
        };
        match result {
            Ok(()) => {
                self.saved_policy.clear();
                clear_journal();
            }
            Err(e) => {
                // Keep the baseline and the journal so the next attempt (and
                // the next launch) can still put the machine back online.
                log::error!("firewall policy restore failed, keeping journal for retry: {}", e);
                winfw::force_unblock_outbound();
                if winfw::live_outbound_blocked() {
                    log::error!("outbound traffic is still blocked after restore failure");
                } else {
                    self.saved_policy.clear();
                    clear_journal();
                }
            }
        }
    }

    fn restore_policy(&mut self) {
        if self.saved_policy.is_empty() && !winfw::live_outbound_blocked() && read_journal().is_none() {
            // Nothing of ours is applied: leave the user's firewall setup
            // exactly as it is instead of forcing our own defaults.
            return;
        }
        let baseline = self.saved_policy.clone();
        self.restore_from(&baseline);
    }

    fn remove_bypass_rules(&self, count: usize) {
        for idx in 0..count {
            winfw::remove_rule(&bypass_rule_name_out(idx));
            winfw::remove_rule(&bypass_rule_name_in(idx));
        }
    }

    fn remove_all_bypass_rules_by_prefix(&self) {
        for idx in 0..MAX_STALE_BYPASS_RULES {
            winfw::remove_rule(&bypass_rule_name_out(idx));
            winfw::remove_rule(&bypass_rule_name_in(idx));
        }
    }

    /// Sweeps every per-app block slot, not just the ones this process
    /// installed, so a crash cannot leave a user locked out of an app.
    fn remove_all_block_app_rules(&self) {
        for idx in 0..MAX_BLOCK_APP_RULES {
            winfw::remove_rule(&block_app_rule_name_out(idx));
            winfw::remove_rule(&block_app_rule_name_in(idx));
        }
    }

    fn wipe_all_rules(&self) {
        let mut names: Vec<&str> = all_static_rule_names();
        names.extend_from_slice(LEGACY_RULE_NAMES);
        winfw::remove_rules(&names);
        self.remove_all_bypass_rules_by_prefix();
        self.remove_all_block_app_rules();
    }
}

impl Drop for RoutingManager {
    fn drop(&mut self) {
        self.force_cleanup();
    }
}

