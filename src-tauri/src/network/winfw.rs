use crate::error::VpnError;
use windows::core::BSTR;
use windows::Win32::Foundation::{VARIANT_FALSE, VARIANT_TRUE};
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INetFwPolicy2, INetFwRule, INetFwRules, NetFwPolicy2, NetFwRule, NET_FW_ACTION_ALLOW,
    NET_FW_ACTION_BLOCK, NET_FW_PROFILE2_DOMAIN, NET_FW_PROFILE2_PRIVATE, NET_FW_PROFILE2_PUBLIC,
    NET_FW_PROFILE_TYPE2, NET_FW_RULE_DIR_IN, NET_FW_RULE_DIR_OUT,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};

const ALL_PROFILES_MASK: i32 = 0x7FFF_FFFF;
const RULE_GROUP: &str = "Wawity";
const MAX_DUPLICATE_REMOVALS: usize = 64;

pub const PROTO_TCP: i32 = 6;
pub const PROTO_UDP: i32 = 17;

#[derive(Debug, Clone, Default)]
pub struct ProfileState {
    pub name: String,
    pub enabled: bool,
    pub inbound_block: bool,
    pub outbound_block: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FirewallRule {
    pub name: String,
    pub outbound: bool,
    pub allow: bool,
    pub program: Option<String>,
    pub protocol: Option<i32>,
    pub local_ports: Option<String>,
    pub remote_ports: Option<String>,
    pub local_addresses: Option<String>,
    pub remote_addresses: Option<String>,
}

fn wrap(context: &str, err: windows::core::Error) -> VpnError {
    VpnError::NetworkError(format!("{}: {}", context, err))
}

struct ComSession {
    owns_apartment: bool,
}

impl ComSession {
    fn open() -> Self {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self {
            owns_apartment: hr.is_ok(),
        }
    }
}

impl Drop for ComSession {
    fn drop(&mut self) {
        if self.owns_apartment {
            unsafe { CoUninitialize() };
        }
    }
}

fn open_policy() -> Result<(ComSession, INetFwPolicy2), VpnError> {
    let session = ComSession::open();
    let policy: INetFwPolicy2 = unsafe { CoCreateInstance(&NetFwPolicy2, None, CLSCTX_ALL) }
        .map_err(|e| wrap("open firewall policy", e))?;
    Ok((session, policy))
}

fn rules_of(policy: &INetFwPolicy2) -> Result<INetFwRules, VpnError> {
    unsafe { policy.Rules() }.map_err(|e| wrap("open firewall rule collection", e))
}

fn remove_all_with_name(rules: &INetFwRules, name: &str) {
    let key = BSTR::from(name);
    for _ in 0..MAX_DUPLICATE_REMOVALS {
        if unsafe { rules.Item(&key) }.is_err() {
            break;
        }
        if unsafe { rules.Remove(&key) }.is_err() {
            break;
        }
    }
}

pub fn add_rule(spec: &FirewallRule) -> Result<(), VpnError> {
    let (_com, policy) = open_policy()?;
    let rules = rules_of(&policy)?;
    remove_all_with_name(&rules, &spec.name);
    let rule: INetFwRule = unsafe { CoCreateInstance(&NetFwRule, None, CLSCTX_ALL) }
        .map_err(|e| wrap("create firewall rule object", e))?;
    unsafe {
        rule.SetName(&BSTR::from(spec.name.as_str()))
            .map_err(|e| wrap("rule name", e))?;
        rule.SetGrouping(&BSTR::from(RULE_GROUP))
            .map_err(|e| wrap("rule grouping", e))?;
        rule.SetDirection(if spec.outbound {
            NET_FW_RULE_DIR_OUT
        } else {
            NET_FW_RULE_DIR_IN
        })
        .map_err(|e| wrap("rule direction", e))?;
        if let Some(proto) = spec.protocol {
            rule.SetProtocol(proto).map_err(|e| wrap("rule protocol", e))?;
        }
        if let Some(v) = &spec.local_ports {
            rule.SetLocalPorts(&BSTR::from(v.as_str()))
                .map_err(|e| wrap("rule local ports", e))?;
        }
        if let Some(v) = &spec.remote_ports {
            rule.SetRemotePorts(&BSTR::from(v.as_str()))
                .map_err(|e| wrap("rule remote ports", e))?;
        }
        if let Some(v) = &spec.local_addresses {
            rule.SetLocalAddresses(&BSTR::from(v.as_str()))
                .map_err(|e| wrap("rule local addresses", e))?;
        }
        if let Some(v) = &spec.remote_addresses {
            rule.SetRemoteAddresses(&BSTR::from(v.as_str()))
                .map_err(|e| wrap("rule remote addresses", e))?;
        }
        if let Some(v) = &spec.program {
            rule.SetApplicationName(&BSTR::from(v.as_str()))
                .map_err(|e| wrap("rule program", e))?;
        }
        rule.SetProfiles(ALL_PROFILES_MASK)
            .map_err(|e| wrap("rule profiles", e))?;
        rule.SetAction(if spec.allow {
            NET_FW_ACTION_ALLOW
        } else {
            NET_FW_ACTION_BLOCK
        })
        .map_err(|e| wrap("rule action", e))?;
        rule.SetEnabled(VARIANT_TRUE)
            .map_err(|e| wrap("rule enable", e))?;
        rules.Add(&rule).map_err(|e| wrap("rule add", e))?;
    }
    Ok(())
}

pub fn remove_rule(name: &str) {
    if let Ok((_com, policy)) = open_policy() {
        if let Ok(rules) = rules_of(&policy) {
            remove_all_with_name(&rules, name);
        }
    }
}

pub fn remove_rules(names: &[&str]) {
    if let Ok((_com, policy)) = open_policy() {
        if let Ok(rules) = rules_of(&policy) {
            for name in names {
                remove_all_with_name(&rules, name);
            }
        }
    }
}

pub fn rule_exists(name: &str) -> bool {
    match open_policy() {
        Ok((_com, policy)) => match rules_of(&policy) {
            Ok(rules) => unsafe { rules.Item(&BSTR::from(name)) }.is_ok(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn profile_kinds() -> [(NET_FW_PROFILE_TYPE2, &'static str); 3] {
    [
        (NET_FW_PROFILE2_DOMAIN, "domain"),
        (NET_FW_PROFILE2_PRIVATE, "private"),
        (NET_FW_PROFILE2_PUBLIC, "public"),
    ]
}

fn kind_for_name(name: &str) -> Option<NET_FW_PROFILE_TYPE2> {
    profile_kinds()
        .into_iter()
        .find(|(_, n)| name.eq_ignore_ascii_case(n))
        .map(|(kind, _)| kind)
}

pub fn read_profiles() -> Result<Vec<ProfileState>, VpnError> {
    let (_com, policy) = open_policy()?;
    let mut out = Vec::with_capacity(3);
    for (kind, name) in profile_kinds() {
        unsafe {
            let enabled = policy
                .get_FirewallEnabled(kind)
                .map_err(|e| wrap("read firewall enabled", e))?;
            let inbound = policy
                .get_DefaultInboundAction(kind)
                .map_err(|e| wrap("read inbound action", e))?;
            let outbound = policy
                .get_DefaultOutboundAction(kind)
                .map_err(|e| wrap("read outbound action", e))?;
            out.push(ProfileState {
                name: name.to_string(),
                enabled: enabled.as_bool(),
                inbound_block: inbound == NET_FW_ACTION_BLOCK,
                outbound_block: outbound == NET_FW_ACTION_BLOCK,
            });
        }
    }
    Ok(out)
}

pub fn apply_profiles(states: &[ProfileState]) -> Result<(), VpnError> {
    let (_com, policy) = open_policy()?;
    for state in states {
        let Some(kind) = kind_for_name(&state.name) else {
            continue;
        };
        unsafe {
            policy
                .put_FirewallEnabled(kind, if state.enabled { VARIANT_TRUE } else { VARIANT_FALSE })
                .map_err(|e| wrap("restore firewall enabled", e))?;
            policy
                .put_DefaultInboundAction(
                    kind,
                    if state.inbound_block { NET_FW_ACTION_BLOCK } else { NET_FW_ACTION_ALLOW },
                )
                .map_err(|e| wrap("restore inbound action", e))?;
            policy
                .put_DefaultOutboundAction(
                    kind,
                    if state.outbound_block { NET_FW_ACTION_BLOCK } else { NET_FW_ACTION_ALLOW },
                )
                .map_err(|e| wrap("restore outbound action", e))?;
        }
    }
    Ok(())
}

pub fn apply_block_all() -> Result<(), VpnError> {
    let (_com, policy) = open_policy()?;
    for (kind, _) in profile_kinds() {
        unsafe {
            policy
                .put_FirewallEnabled(kind, VARIANT_TRUE)
                .map_err(|e| wrap("enable firewall", e))?;
            policy
                .put_DefaultInboundAction(kind, NET_FW_ACTION_BLOCK)
                .map_err(|e| wrap("set inbound block", e))?;
            policy
                .put_DefaultOutboundAction(kind, NET_FW_ACTION_BLOCK)
                .map_err(|e| wrap("set outbound block", e))?;
        }
    }
    Ok(())
}

pub fn apply_safe_defaults() -> Result<(), VpnError> {
    let (_com, policy) = open_policy()?;
    for (kind, _) in profile_kinds() {
        unsafe {
            policy
                .put_FirewallEnabled(kind, VARIANT_TRUE)
                .map_err(|e| wrap("enable firewall", e))?;
            policy
                .put_DefaultInboundAction(kind, NET_FW_ACTION_BLOCK)
                .map_err(|e| wrap("set inbound block", e))?;
            policy
                .put_DefaultOutboundAction(kind, NET_FW_ACTION_ALLOW)
                .map_err(|e| wrap("set outbound allow", e))?;
        }
    }
    Ok(())
}
