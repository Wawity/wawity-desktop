use crate::config::parser::{
    Hysteria2Params, ProxyConfig, ShadowsocksParams, TrojanParams, TuicParams, VlessParams,
    VmessParams,
};
use crate::constants::LOCAL_PROXY_PORT;
use crate::error::VpnError;
use serde_json::{json, Value};
use std::path::PathBuf;
use zeroize::Zeroize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DpiProfile {
    Off,
    Soft,
    Medium,
    Hard,
}

impl DpiProfile {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "soft" | "light" | "low" => DpiProfile::Soft,
            "medium" | "mid" | "balanced" => DpiProfile::Medium,
            "hard" | "strong" | "max" | "aggressive" => DpiProfile::Hard,
            _ => DpiProfile::Off,
        }
    }

    pub fn mux_streams(self) -> (u64, u64) {
        match self {
            DpiProfile::Off => (4, 4),
            DpiProfile::Soft => (4, 6),
            DpiProfile::Medium => (6, 8),
            DpiProfile::Hard => (8, 12),
        }
    }

    pub fn packet_fragment(self) -> bool {
        !matches!(self, DpiProfile::Off)
    }

    pub fn record_fragment(self) -> bool {
        matches!(self, DpiProfile::Medium | DpiProfile::Hard)
    }

    pub fn fallback_delay(self) -> &'static str {
        match self {
            DpiProfile::Soft => "300ms",
            DpiProfile::Medium => "500ms",
            DpiProfile::Hard => "800ms",
            DpiProfile::Off => "",
        }
    }
}

pub struct ConfigGenerator {
    pub kill_switch: bool,
    pub block_ads: bool,
    pub bypass_apps: Vec<String>,
    pub tun_interface: String,
    pub quantum_resistant: bool,
    pub binary_path: String,
    pub app_exe_path: String,
    pub ads_ruleset_path: Option<String>,
    pub private_ruleset_path: Option<String>,
    pub default_interface: Option<String>,
    pub enable_padding: bool,
    pub strict_route: bool,
    pub allow_insecure: bool,
    pub tunnel_own_traffic: bool,
    pub bootstrap_ip: String,
    pub bootstrap_ip_alt: String,
    pub system_dns: Option<String>,
    pub dpi_profile: DpiProfile,
}

impl ConfigGenerator {
    pub fn new(
        kill_switch: bool,
        bypass_apps: Vec<String>,
        quantum_resistant: bool,
        binary_path: String,
        app_exe_path: String,
    ) -> Self {
        let normalized_bypass = crate::util::normalize_path_list(&bypass_apps);
        Self {
            kill_switch,
            block_ads: true,
            bypass_apps: normalized_bypass,
            tun_interface: "wawity-tun0".to_string(),
            quantum_resistant,
            binary_path: crate::util::normalize_windows_path(&binary_path),
            app_exe_path: crate::util::normalize_windows_path(&app_exe_path),
            ads_ruleset_path: None,
            private_ruleset_path: None,
            default_interface: None,
            enable_padding: true,
            strict_route: true,
            allow_insecure: false,
            tunnel_own_traffic: true,
            bootstrap_ip: "1.1.1.1".to_string(),
            bootstrap_ip_alt: "1.0.0.1".to_string(),
            system_dns: None,
            dpi_profile: DpiProfile::Off,
        }
    }

    pub fn with_ad_blocking(mut self, enabled: bool) -> Self {
        self.block_ads = enabled;
        self
    }

    pub fn with_local_rulesets(
        mut self,
        ads_path: Option<PathBuf>,
        private_path: Option<PathBuf>,
    ) -> Self {
        self.ads_ruleset_path = ads_path
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().replace('\\', "/"));
        self.private_ruleset_path = private_path
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().replace('\\', "/"));
        self
    }

    pub fn with_default_interface(mut self, iface: Option<String>) -> Self {
        self.default_interface = iface.filter(|s| !s.is_empty());
        self
    }

    pub fn with_padding(mut self, enabled: bool) -> Self {
        self.enable_padding = enabled;
        self
    }

    pub fn with_dpi(mut self, raw: &str) -> Self {
        self.dpi_profile = DpiProfile::parse(raw);
        self
    }

    pub fn with_privacy(
        mut self,
        strict_route: bool,
        allow_insecure: bool,
        tunnel_own_traffic: bool,
        bootstrap: &str,
    ) -> Self {
        self.strict_route = strict_route;
        self.allow_insecure = allow_insecure;
        self.tunnel_own_traffic = tunnel_own_traffic;
        let (a, b) = match bootstrap {
            "google" => ("8.8.8.8", "8.8.4.4"),
            "quad9" => ("9.9.9.9", "149.112.112.112"),
            _ => ("1.1.1.1", "1.0.0.1"),
        };
        self.bootstrap_ip = a.to_string();
        self.bootstrap_ip_alt = b.to_string();
        self
    }

    pub fn with_system_dns(mut self, server: Option<String>) -> Self {
        self.system_dns = server;
        self
    }

    fn bootstrap_resolver_tag(&self) -> &'static str {
        if self.system_dns.is_some() {
            "isp-dns"
        } else {
            "bootstrap-dns-doh"
        }
    }

    fn ads_enabled(&self) -> bool {
        self.block_ads && self.ads_ruleset_path.is_some()
    }

    fn private_ruleset_enabled(&self) -> bool {
        self.private_ruleset_path.is_some()
    }

    pub fn to_json(
        &self,
        exit: &ProxyConfig,
        entry: Option<&ProxyConfig>,
    ) -> Result<String, VpnError> {
        let cfg = self.build(exit, entry)?;
        serde_json::to_string_pretty(&cfg).map_err(|e| VpnError::JsonError(e.to_string()))
    }

    fn build(
        &self,
        exit: &ProxyConfig,
        entry: Option<&ProxyConfig>,
    ) -> Result<Value, VpnError> {
        let mut outbounds: Vec<Value> = Vec::new();

        if let Some(entry_cfg) = entry {
            outbounds.push(self.build_proxy_outbound(entry_cfg, "proxy-entry", None)?);
            outbounds.push(self.build_proxy_outbound(exit, "proxy", Some("proxy-entry"))?);
        } else {
            outbounds.push(self.build_proxy_outbound(exit, "proxy", None)?);
        }

        outbounds.push(self.build_direct_outbound());

        let log_path = std::env::temp_dir()
            .join("wawity.log")
            .to_string_lossy()
            .replace('\\', "/");

        let mut cfg = json!({
            "log": {
                "disabled": false,
                "level": "info",
                "output": log_path,
                "timestamp": true
            },
            "dns": self.build_dns(),
            "inbounds": [
                self.build_tun_inbound(),
                self.build_mixed_inbound()
            ],
            "outbounds": outbounds,
            "route": self.build_route_tuned()
        });

        let rule_sets = self.build_rule_sets();
        if !rule_sets.is_empty() {
            cfg["route"]["rule_set"] = json!(rule_sets);
        }

        Ok(cfg)
    }

    fn build_direct_outbound(&self) -> Value {
        let mut direct = json!({
            "type": "direct",
            "tag": "direct"
        });
        if let Some(iface) = &self.default_interface {
            direct["bind_interface"] = json!(iface);
        }
        direct
    }

    const VALVE_GAME_RANGES: [&'static str; 10] = [
        "155.133.0.0/16",
        "162.254.192.0/21",
        "146.66.152.0/21",
        "185.25.180.0/22",
        "205.196.6.0/24",
        "103.10.124.0/23",
        "103.28.54.0/24",
        "208.64.200.0/22",
        "192.69.96.0/22",
        "45.121.184.0/23"
    ];

    fn build_tun_inbound(&self) -> Value {
        let mut tun = json!({
            "type": "tun",
            "tag": "tun-in",
            "interface_name": self.tun_interface,
            "address": [
                "172.19.0.1/30",
                "fdfe:dcba:9876::1/126"
            ],
            "mtu": 1500,
            "auto_route": true,
            "strict_route": self.strict_route,
            "stack": "system"
        });
        if !self.bypass_apps.is_empty() {
            tun["route_exclude_address"] = json!(Self::VALVE_GAME_RANGES);
            crate::util::net_debug_log(&format!(
                "tun exclude: {} valve ranges kernel-bypassed",
                Self::VALVE_GAME_RANGES.len()
            ));
        }
        tun
    }

    fn build_mixed_inbound(&self) -> Value {
        json!({
            "type": "mixed",
            "tag": "mixed-in",
            "listen": "127.0.0.1",
            "listen_port": LOCAL_PROXY_PORT
        })
    }

    fn build_dns(&self) -> Value {
        let mut rules: Vec<Value> = Vec::new();

        if self.ads_enabled() {
            rules.push(json!({
                "rule_set": ["geosite-category-ads-all"],
                "action": "reject"
            }));
        }

        if !self.bypass_apps.is_empty() {
            rules.push(json!({
                "process_path": self.bypass_apps,
                "server": "direct-dns"
            }));
            let bypass_names = Self::bypass_process_names(&self.bypass_apps);
            if !bypass_names.is_empty() {
                rules.push(json!({
                    "process_name": bypass_names,
                    "server": "direct-dns"
                }));
            }
        }

        let mut bootstrap_doh = json!({
            "type": "https",
            "tag": "bootstrap-dns-doh",
            "server": self.bootstrap_ip,
            "server_port": 443
        });
        let mut bootstrap_alt = json!({
            "type": "https",
            "tag": "bootstrap-dns",
            "server": self.bootstrap_ip_alt,
            "server_port": 443
        });
        let mut plain_dns = json!({
            "type": "udp",
            "tag": "direct-dns",
            "server": self.bootstrap_ip,
            "server_port": 53
        });

        if self.default_interface.is_some() {
            bootstrap_doh["detour"] = json!("direct");
            bootstrap_alt["detour"] = json!("direct");
            plain_dns["detour"] = json!("direct");
        }

        let mut servers = vec![
            json!({
                "type": "https",
                "tag": "remote-dns-doh",
                "server": "1.1.1.1",
                "server_port": 443,
                "detour": "proxy"
            }),
            json!({
                "type": "tls",
                "tag": "remote-dns",
                "server": "1.1.1.1",
                "server_port": 853,
                "detour": "proxy"
            }),
            bootstrap_doh,
            bootstrap_alt,
            plain_dns,
        ];

        if let Some(isp) = &self.system_dns {
            let mut isp_server = json!({
                "type": "udp",
                "tag": "isp-dns",
                "server": isp,
                "server_port": 53
            });
            if self.default_interface.is_some() {
                isp_server["detour"] = json!("direct");
            }
            servers.push(isp_server);
        }

        json!({
            "servers": servers,
            "rules": rules,
            "final": "remote-dns-doh",
            "strategy": "ipv4_only",
            "independent_cache": true
        })
    }

    fn build_rule_sets(&self) -> Vec<Value> {
        let mut sets = Vec::new();
        if self.ads_enabled() {
            sets.push(json!({
                "tag": "geosite-category-ads-all",
                "type": "local",
                "format": "binary",
                "path": self.ads_ruleset_path.as_ref().unwrap()
            }));
        }
        if self.private_ruleset_enabled() {
            sets.push(json!({
                "tag": "geosite-private",
                "type": "local",
                "format": "binary",
                "path": self.private_ruleset_path.as_ref().unwrap()
            }));
        }
        sets
    }

    fn bypass_process_names(paths: &[String]) -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        let mut names = Vec::new();
        for path in paths {
            let Some(raw) = path.rsplit(['\\', '/']).next() else { continue };
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            for variant in [trimmed.to_string(), trimmed.to_lowercase()] {
                if seen.insert(variant.clone()) {
                    names.push(variant);
                }
            }
        }
        names
    }

    fn build_route_tuned(&self) -> Value {
        let mut route = self.build_route();
        let frag = self.dpi_profile.packet_fragment();
        let record = self.dpi_profile.record_fragment();
        if !frag && !record {
            return route;
        }

        let mut rule = json!({
            "action": "route-options",
            "network": ["tcp"],
            "port": [443, 8443]
        });
        if frag {
            rule["tls_fragment"] = json!(true);
            let delay = self.dpi_profile.fallback_delay();
            if !delay.is_empty() {
                rule["tls_fragment_fallback_delay"] = json!(delay);
            }
        }
        if record {
            rule["tls_record_fragment"] = json!(true);
        }

        match route.get_mut("rules").and_then(|r| r.as_array_mut()) {
            Some(rules) => rules.insert(0, rule),
            None => route["rules"] = json!([rule]),
        }
        route
    }

    fn build_route(&self) -> Value {
        let mut rules: Vec<Value> = Vec::new();

        if !self.bypass_apps.is_empty() {
            let bypass_names = Self::bypass_process_names(&self.bypass_apps);
            crate::util::net_debug_log(&format!(
                "split apply: {} paths {:?} names {:?}",
                self.bypass_apps.len(),
                self.bypass_apps,
                bypass_names
            ));
            rules.push(json!({
                "process_path": self.bypass_apps,
                "outbound": "direct"
            }));
            if !bypass_names.is_empty() {
                rules.push(json!({
                    "process_name": bypass_names,
                    "outbound": "direct"
                }));
            }
        } else {
            crate::util::net_debug_log("split apply: empty list");
        }

        rules.push(json!({
            "inbound": ["tun-in", "mixed-in"],
            "action": "sniff"
        }));

        rules.push(json!({
            "protocol": "dns",
            "action": "hijack-dns"
        }));

        rules.push(json!({
            "inbound": ["mixed-in"],
            "outbound": "proxy"
        }));

        let mut direct_paths = vec![self.binary_path.clone()];
        if !self.tunnel_own_traffic {
            direct_paths.push(self.app_exe_path.clone());
        }
        rules.push(json!({
            "process_path": direct_paths,
            "outbound": "direct"
        }));

        if self.ads_enabled() {
            rules.push(json!({
                "rule_set": ["geosite-category-ads-all"],
                "action": "reject"
            }));
        }

        rules.push(json!({
            "ip_is_private": true,
            "outbound": "direct"
        }));

        if self.private_ruleset_enabled() {
            rules.push(json!({
                "rule_set": ["geosite-private"],
                "outbound": "direct"
            }));
        }

        rules.push(json!({
            "inbound": ["mixed-in", "tun-in"],
            "outbound": "proxy"
        }));

        if self.kill_switch {
            rules.push(json!({
                "network": ["tcp", "udp"],
                "action": "reject",
                "method": "drop"
            }));
        }

        let final_out = if self.kill_switch { "proxy" } else { "direct" };

        let mut route = json!({
            "rules": rules,
            "final": final_out,
            "auto_detect_interface": self.default_interface.is_none(),
            "default_domain_resolver": self.bootstrap_resolver_tag(),
            "find_process": true
        });

        if let Some(iface) = &self.default_interface {
            route["default_interface"] = json!(iface);
        }

        route
    }
    fn supports_mux(proxy: &ProxyConfig) -> bool {
        match proxy {
            ProxyConfig::Vless(p) => p.flow.is_empty(),
            ProxyConfig::Vmess(_) | ProxyConfig::Trojan(_) | ProxyConfig::Shadowsocks(_) => true,
            ProxyConfig::Hysteria2(_) | ProxyConfig::Tuic(_) => false,
        }
    }
    fn build_proxy_outbound(
        &self,
        proxy: &ProxyConfig,
        tag: &str,
        detour: Option<&str>,
    ) -> Result<Value, VpnError> {
        let mut out = match proxy {
            ProxyConfig::Vless(p) => self.build_vless(p)?,
            ProxyConfig::Vmess(p) => self.build_vmess(p)?,
            ProxyConfig::Trojan(p) => self.build_trojan(p)?,
            ProxyConfig::Shadowsocks(p) => self.build_shadowsocks(p),
            ProxyConfig::Hysteria2(p) => self.build_hysteria2(p),
            ProxyConfig::Tuic(p) => self.build_tuic(p),
        };

        out["tag"] = json!(tag);
        out["domain_resolver"] = json!(self.bootstrap_resolver_tag());

        if let Some(d) = detour {
            out["detour"] = json!(d);
        }

        if self.enable_padding && Self::supports_mux(proxy) {
            let (links, streams) = self.dpi_profile.mux_streams();
            out["multiplex"] = json!({
                "enabled": true,
                "protocol": "h2mux",
                "max_connections": links,
                "min_streams": streams,
                "padding": true
            });
        }

        Ok(out)
    }

    fn pq_curves(&self) -> Value {
        json!(["X25519MLKEM768", "X25519"])
    }

    fn build_tls_plain(&self, sni: &str, fp: &str, insecure: bool) -> Value {
        let insecure = insecure && self.allow_insecure;
        if insecure {
            log::warn!(
                "TLS certificate validation bypass enabled by user for sni={}",
                sni
            );
        }

        let mut tls = json!({
            "enabled": true,
            "server_name": sni,
            "insecure": insecure,
            "utls": {
                "enabled": !fp.is_empty(),
                "fingerprint": if fp.is_empty() { "chrome" } else { fp }
            }
        });

        if self.quantum_resistant {
            tls["curve_preferences"] = self.pq_curves();
        }

        tls
    }

    fn build_tls_reality(&self, sni: &str, fp: &str, pubkey: &str, sid: &str) -> Value {
        json!({
            "enabled": true,
            "server_name": sni,
            "utls": {
                "enabled": true,
                "fingerprint": if fp.is_empty() { "chrome" } else { fp }
            },
            "reality": {
                "enabled": true,
                "public_key": pubkey,
                "short_id": sid
            }
        })
    }

    fn build_transport(
        &self,
        network: &str,
        path: &str,
        host: &str,
        svc: &str,
    ) -> Option<Value> {
        match network {
            "ws" => {
                let mut t = json!({
                    "type": "ws",
                    "path": if path.is_empty() { "/" } else { path }
                });
                if !host.is_empty() {
                    t["headers"] = json!({ "Host": host });
                }
                Some(t)
            }
            "grpc" => Some(json!({
                "type": "grpc",
                "service_name": svc
            })),
            "http" => {
                let mut t = json!({
                    "type": "http",
                    "path": if path.is_empty() { "/" } else { path }
                });
                if !host.is_empty() {
                    t["host"] = json!([host]);
                }
                Some(t)
            }
            "httpupgrade" => {
                let mut t = json!({
                    "type": "httpupgrade",
                    "path": if path.is_empty() { "/" } else { path }
                });
                if !host.is_empty() {
                    t["host"] = json!(host);
                }
                Some(t)
            }
            _ => None,
        }
    }

    fn build_vless(&self, p: &VlessParams) -> Result<Value, VpnError> {
        let mut out = json!({
            "type": "vless",
            "server": p.server,
            "server_port": p.port,
            "uuid": p.uuid,
            "packet_encoding": "xudp"
        });

        if !p.flow.is_empty() {
            out["flow"] = json!(p.flow);
        }

        out["tls"] = match p.security.as_str() {
            "reality" => self.build_tls_reality(
                &p.sni,
                &p.fingerprint,
                &p.public_key,
                &p.short_id,
            ),
            "tls" => self.build_tls_plain(&p.sni, &p.fingerprint, p.insecure),
            _ => json!({ "enabled": false }),
        };

        if let Some(t) = self.build_transport(
            &p.network,
            &p.path,
            &p.host,
            &p.service_name,
        ) {
            out["transport"] = t;
        }

        Ok(out)
    }

    fn build_vmess(&self, p: &VmessParams) -> Result<Value, VpnError> {
        let mut out = json!({
            "type": "vmess",
            "server": p.server,
            "server_port": p.port,
            "uuid": p.uuid,
            "alter_id": p.alter_id,
            "security": if p.security.is_empty() { "auto" } else { p.security.as_str() }
        });

        if p.tls {
            out["tls"] = self.build_tls_plain(&p.sni, "chrome", false);
        }

        if let Some(t) = self.build_transport(
            &p.network,
            &p.path,
            &p.host,
            &p.service_name,
        ) {
            out["transport"] = t;
        }

        Ok(out)
    }

    fn build_trojan(&self, p: &TrojanParams) -> Result<Value, VpnError> {
        let mut out = json!({
            "type": "trojan",
            "server": p.server,
            "server_port": p.port,
            "password": p.password,
            "tls": self.build_tls_plain(&p.sni, &p.fingerprint, p.insecure)
        });

        if let Some(t) = self.build_transport(
            &p.network,
            &p.path,
            &p.host,
            "",
        ) {
            out["transport"] = t;
        }

        Ok(out)
    }

    fn build_shadowsocks(&self, p: &ShadowsocksParams) -> Value {
        let mut out = json!({
            "type": "shadowsocks",
            "server": p.server,
            "server_port": p.port,
            "method": p.method,
            "password": p.password
        });

        if !p.plugin.is_empty() {
            out["plugin"] = json!(p.plugin);
            if !p.plugin_opts.is_empty() {
                out["plugin_opts"] = json!(p.plugin_opts);
            }
        }

        out
    }

    fn build_hysteria2(&self, p: &Hysteria2Params) -> Value {
        if p.insecure && !self.allow_insecure {
            log::warn!("subscription requested TLS bypass for hysteria2, ignored (sni={})", p.sni);
        }

        let mut tls = json!({
            "enabled": true,
            "server_name": p.sni,
            "insecure": p.insecure && self.allow_insecure
        });

        if self.quantum_resistant {
            tls["curve_preferences"] = self.pq_curves();
        }

        let mut out = json!({
            "type": "hysteria2",
            "server": p.server,
            "server_port": p.port,
            "password": p.password,
            "tls": tls
        });

        if !p.obfs.is_empty() {
            out["obfs"] = json!({
                "type": p.obfs,
                "password": p.obfs_password
            });
        }

        out
    }

    fn build_tuic(&self, p: &TuicParams) -> Value {
        if p.insecure && !self.allow_insecure {
            log::warn!("subscription requested TLS bypass for tuic, ignored (sni={})", p.sni);
        }

        let mut tls = json!({
            "enabled": true,
            "server_name": p.sni,
            "insecure": p.insecure && self.allow_insecure,
            "alpn": ["h3"]
        });

        if self.quantum_resistant {
            tls["curve_preferences"] = self.pq_curves();
        }

        json!({
            "type": "tuic",
            "server": p.server,
            "server_port": p.port,
            "uuid": p.uuid,
            "password": p.password,
            "congestion_control": if p.congestion_control.is_empty() {
                "bbr"
            } else {
                p.congestion_control.as_str()
            },
            "tls": tls
        })
    }
}

impl Drop for ConfigGenerator {
    fn drop(&mut self) {
        self.bypass_apps.iter_mut().for_each(|s| s.zeroize());
        self.binary_path.zeroize();
        self.app_exe_path.zeroize();
    }
}

