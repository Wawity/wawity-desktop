use crate::error::VpnError;
use crate::network::winfw::{self, FirewallRule, ProfileState, PROTO_TCP, PROTO_UDP};

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

pub struct RoutingManager {
    mode: FirewallMode,
    saved_policy: Vec<ProfileState>,
    active_bypass_count: usize,
    installed_app_path: Option<String>,
    installed_singbox_path: Option<String>,
    installed_bypass_paths: Vec<String>,
}

impl RoutingManager {
    pub fn new() -> Self {
        Self {
            mode: FirewallMode::Normal,
            saved_policy: Vec::new(),
            active_bypass_count: 0,
            installed_app_path: None,
            installed_singbox_path: None,
            installed_bypass_paths: Vec::new(),
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
        if let Err(e) = winfw::apply_block_all() {
            self.restore_policy();
            self.wipe_all_rules();
            self.installed_app_path = None;
            return Err(e);
        }
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
        if self.mode == FirewallMode::Normal {
            self.saved_policy = self.read_baseline_policy()?;
            self.wipe_all_rules();
            self.install_base_rules(app_exe_path)?;
        }
        self.install_singbox_rules(singbox_path)?;
        if self.mode == FirewallMode::Normal {
            winfw::apply_block_all()?;
        }
        Ok(())
    }

    pub fn commit_connection(&mut self) -> Result<(), VpnError> {
        if self.mode == FirewallMode::Connected {
            return Ok(());
        }
        if self.mode == FirewallMode::Normal {
            winfw::apply_block_all()?;
        }
        self.mode = FirewallMode::Connected;
        Ok(())
    }

    pub fn allow_server_endpoint(&mut self, host: &str, resolver: &str) -> Result<String, VpnError> {
        let trimmed = host.trim();
        if trimmed.is_empty() {
            return Err(VpnError::NetworkError("empty server host".into()));
        }
        let ips = crate::commands::resolve_server_ips_bootstrap(trimmed, resolver)
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
        if self.mode != FirewallMode::Connected {
            return Ok(());
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
        Ok(())
    }

    pub fn update_bypass_rules(&mut self, paths: &[String]) -> Result<(), VpnError> {
        let normalized = crate::util::normalize_path_list(paths);
        let old_count = self.active_bypass_count;
        self.remove_bypass_rules(old_count);
        self.active_bypass_count = 0;
        self.installed_bypass_paths.clear();

        if normalized.is_empty() {
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
        let profiles = winfw::read_profiles().unwrap_or_default();
        let policy_drifted = profiles.is_empty()
            || profiles
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
                if self.update_bypass_rules(&expected_bypass).is_ok() {
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
        self.installed_app_path = None;
        self.installed_singbox_path = None;
        self.installed_bypass_paths.clear();
        self.mode = FirewallMode::Normal;
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

    fn read_baseline_policy(&self) -> Result<Vec<ProfileState>, VpnError> {
        let mut policy = winfw::read_profiles()?;
        for p in policy.iter_mut() {
            p.outbound_block = false;
        }
        Ok(policy)
    }

    fn restore_policy(&mut self) {
        if self.saved_policy.is_empty() {
            let _ = winfw::apply_safe_defaults();
        } else {
            let _ = winfw::apply_profiles(&self.saved_policy);
        }
        self.saved_policy.clear();
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

    fn wipe_all_rules(&self) {
        let mut names: Vec<&str> = all_static_rule_names();
        names.extend_from_slice(LEGACY_RULE_NAMES);
        winfw::remove_rules(&names);
        self.remove_all_bypass_rules_by_prefix();
    }
}

impl Drop for RoutingManager {
    fn drop(&mut self) {
        self.force_cleanup();
    }
}

