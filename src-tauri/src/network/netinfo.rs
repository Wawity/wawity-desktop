use crate::error::VpnError;
use std::thread;
use std::time::{Duration, Instant};
use windows::core::{GUID, PWSTR};
use windows::Win32::Foundation::{ERROR_BUFFER_OVERFLOW, ERROR_SUCCESS};
use windows::Win32::NetworkManagement::IpHelper::{
    FreeMibTable, GetAdaptersAddresses, GetIfEntry2, GetIfTable2, GetIpForwardTable2,
    GetIpInterfaceEntry, SetInterfaceDnsSettings, DNS_INTERFACE_SETTINGS,
    DNS_INTERFACE_SETTINGS_VERSION1, DNS_SETTING_NAMESERVER, GAA_FLAG_SKIP_ANYCAST,
    GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_UNICAST, IP_ADAPTER_ADDRESSES_LH, MIB_IF_ROW2,
    MIB_IF_TABLE2, MIB_IPFORWARD_TABLE2, MIB_IPINTERFACE_ROW,
};
use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
use windows::Win32::Networking::WinSock::{AF_INET, SOCKADDR_IN};

const TUN_ADAPTER_ALIAS: &str = "wawity-tun0";

const VIRTUAL_ADAPTER_MARKERS: &[&str] = &[
    "wintun",
    "wireguard",
    "tap",
    "tun ",
    "openvpn",
    "hyper-v",
    "vmware",
    "virtualbox",
    "loopback",
    "bluetooth",
    "hamachi",
    "radmin",
];

#[derive(Debug, Clone)]
pub struct AdapterInfo {
    pub alias: String,
    pub description: String,
    pub is_up: bool,
    pub guid: GUID,
}

fn wide_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

fn is_virtual_adapter(description: &str) -> bool {
    let lower = description.to_lowercase();
    VIRTUAL_ADAPTER_MARKERS.iter().any(|m| lower.contains(m))
}

pub fn list_adapters() -> Option<Vec<AdapterInfo>> {
    let mut table: *mut MIB_IF_TABLE2 = std::ptr::null_mut();
    if !unsafe { GetIfTable2(&mut table) }.is_ok() || table.is_null() {
        return None;
    }
    let mut rows = Vec::new();
    unsafe {
        let count = (*table).NumEntries as usize;
        let base = (*table).Table.as_ptr();
        for i in 0..count {
            let row = &*base.add(i);
            rows.push(AdapterInfo {
                alias: wide_to_string(&row.Alias),
                description: wide_to_string(&row.Description),
                is_up: row.OperStatus == IfOperStatusUp,
                guid: row.InterfaceGuid,
            });
        }
        FreeMibTable(table as *const _);
    }
    Some(rows)
}

pub fn wintun_adapter_present() -> bool {
    list_adapters()
        .map(|rows| {
            rows.iter()
                .any(|a| a.is_up && a.alias.eq_ignore_ascii_case(TUN_ADAPTER_ALIAS))
        })
        .unwrap_or(false)
}

pub fn adapter_summary() -> String {
    match list_adapters() {
        Some(rows) => rows
            .iter()
            .filter(|a| a.is_up)
            .map(|a| format!("{} [{}]", a.alias, a.description))
            .collect::<Vec<_>>()
            .join("; "),
        None => "interface table unavailable".into(),
    }
}

pub fn wait_for_wintun_teardown(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if !wintun_adapter_present() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(200));
    }
}

pub fn tun_interface_alias(candidate: &str) -> Option<String> {
    let rows = list_adapters()?;
    for adapter in &rows {
        if adapter.is_up && adapter.alias.eq_ignore_ascii_case(candidate) {
            return Some(adapter.alias.clone());
        }
    }
    for adapter in &rows {
        if adapter.is_up && adapter.description.to_lowercase().contains("wintun") {
            return Some(adapter.alias.clone());
        }
    }
    Some(candidate.to_string())
}

pub fn default_physical_interface() -> Option<String> {
    let mut table: *mut MIB_IPFORWARD_TABLE2 = std::ptr::null_mut();
    if !unsafe { GetIpForwardTable2(AF_INET, &mut table) }.is_ok() || table.is_null() {
        return None;
    }
    let mut best: Option<(u64, String)> = None;
    unsafe {
        let count = (*table).NumEntries as usize;
        let base = (*table).Table.as_ptr();
        for i in 0..count {
            let route = &*base.add(i);
            if route.DestinationPrefix.PrefixLength != 0 {
                continue;
            }
            let mut ifrow = MIB_IF_ROW2::default();
            ifrow.InterfaceLuid = route.InterfaceLuid;
            if !GetIfEntry2(&mut ifrow).is_ok() {
                continue;
            }
            if ifrow.OperStatus != IfOperStatusUp {
                continue;
            }
            let description = wide_to_string(&ifrow.Description);
            if is_virtual_adapter(&description) {
                continue;
            }
            let alias = wide_to_string(&ifrow.Alias);
            if alias.trim().is_empty() {
                continue;
            }
            if alias.eq_ignore_ascii_case(TUN_ADAPTER_ALIAS) {
                continue;
            }
            let mut ipif = MIB_IPINTERFACE_ROW::default();
            ipif.Family = AF_INET;
            ipif.InterfaceLuid = route.InterfaceLuid;
            let iface_metric = if GetIpInterfaceEntry(&mut ipif).is_ok() {
                ipif.Metric as u64
            } else {
                0
            };
            let total_metric = route.Metric as u64 + iface_metric;
            match &best {
                Some((metric, _)) if *metric <= total_metric => {}
                _ => best = Some((total_metric, alias)),
            }
        }
        FreeMibTable(table as *const _);
    }
    best.map(|(_, alias)| alias)
}

pub fn physical_dns_servers(alias: &str) -> Vec<String> {
    let mut servers: Vec<String> = Vec::new();
    let mut size: u32 = 16384;
    unsafe {
        for _ in 0..3 {
            let mut buf = vec![0u8; size as usize];
            let head = buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;
            let flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_UNICAST;
            let status = GetAdaptersAddresses(AF_INET.0 as u32, flags, None, Some(head), &mut size);
            if status == ERROR_BUFFER_OVERFLOW.0 {
                continue;
            }
            if status != ERROR_SUCCESS.0 {
                return servers;
            }
            let mut node = head;
            while !node.is_null() {
                let adapter = &*node;
                let name = adapter.FriendlyName.to_string().unwrap_or_default();
                if adapter.OperStatus == IfOperStatusUp
                    && name.eq_ignore_ascii_case(alias)
                    && !name.eq_ignore_ascii_case(TUN_ADAPTER_ALIAS)
                {
                    let mut dns_node = adapter.FirstDnsServerAddress;
                    while !dns_node.is_null() {
                        let sockaddr = (*dns_node).Address.lpSockaddr;
                        if !sockaddr.is_null() && (*sockaddr).sa_family == AF_INET {
                            let v4 = &*(sockaddr as *const SOCKADDR_IN);
                            let octets = v4.sin_addr.S_un.S_addr.to_ne_bytes();
                            let ip = format!("{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3]);
                            let usable = !ip.starts_with("127.")
                                && !ip.starts_with("169.254.")
                                && ip != "0.0.0.0"
                                && ip != "172.19.0.2"
                                && !servers.contains(&ip);
                            if usable {
                                servers.push(ip);
                            }
                        }
                        dns_node = (*dns_node).Next;
                    }
                }
                node = adapter.Next;
            }
            break;
        }
    }
    servers
}

pub fn set_interface_dns(alias: &str, nameserver: &str) -> Result<(), VpnError> {
    let rows = list_adapters()
        .ok_or_else(|| VpnError::NetworkError("interface table unavailable".into()))?;
    let target = rows
        .iter()
        .find(|a| a.alias.eq_ignore_ascii_case(alias))
        .ok_or_else(|| VpnError::NetworkError(format!("interface {} not found", alias)))?;
    let mut server_wide: Vec<u16> = nameserver.encode_utf16().chain(std::iter::once(0)).collect();
    let settings = DNS_INTERFACE_SETTINGS {
        Version: DNS_INTERFACE_SETTINGS_VERSION1,
        Flags: DNS_SETTING_NAMESERVER as u64,
        Domain: PWSTR::null(),
        NameServer: PWSTR(server_wide.as_mut_ptr()),
        SearchList: PWSTR::null(),
        RegistrationEnabled: 0,
        RegisterAdapterName: 0,
        EnableLLMNR: 0,
        QueryAdapterName: 0,
        ProfileNameServer: PWSTR::null(),
    };
    let status = unsafe { SetInterfaceDnsSettings(target.guid, &settings) };
    if status.is_ok() {
        Ok(())
    } else {
        Err(VpnError::NetworkError(format!(
            "set dns on {} failed: {:?}",
            alias, status
        )))
    }
}
