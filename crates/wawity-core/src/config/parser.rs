use crate::error::VpnError;
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlessParams {
    pub uuid: String,
    pub server: String,
    pub port: u16,
    pub flow: String,
    pub network: String,
    pub security: String,
    pub sni: String,
    pub fingerprint: String,
    pub public_key: String,
    pub short_id: String,
    pub path: String,
    pub host: String,
    pub service_name: String,
    pub insecure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmessParams {
    pub uuid: String,
    pub server: String,
    pub port: u16,
    pub alter_id: u16,
    pub security: String,
    pub network: String,
    pub tls: bool,
    pub sni: String,
    pub path: String,
    pub host: String,
    pub service_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrojanParams {
    pub password: String,
    pub server: String,
    pub port: u16,
    pub sni: String,
    pub fingerprint: String,
    pub network: String,
    pub path: String,
    pub host: String,
    pub insecure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowsocksParams {
    pub method: String,
    pub password: String,
    pub server: String,
    pub port: u16,
    pub plugin: String,
    pub plugin_opts: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hysteria2Params {
    pub password: String,
    pub server: String,
    pub port: u16,
    pub sni: String,
    pub insecure: bool,
    pub obfs: String,
    pub obfs_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuicParams {
    pub uuid: String,
    pub password: String,
    pub server: String,
    pub port: u16,
    pub sni: String,
    pub congestion_control: String,
    pub insecure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyConfig {
    Vless(VlessParams),
    Vmess(VmessParams),
    Trojan(TrojanParams),
    Shadowsocks(ShadowsocksParams),
    Hysteria2(Hysteria2Params),
    Tuic(TuicParams),
}

impl ProxyConfig {
    pub fn protocol_name(&self) -> &'static str {
        match self {
            ProxyConfig::Vless(_) => "vless",
            ProxyConfig::Vmess(_) => "vmess",
            ProxyConfig::Trojan(_) => "trojan",
            ProxyConfig::Shadowsocks(_) => "shadowsocks",
            ProxyConfig::Hysteria2(_) => "hysteria2",
            ProxyConfig::Tuic(_) => "tuic",
        }
    }

    pub fn server_host(&self) -> &str {
        match self {
            ProxyConfig::Vless(p) => &p.server,
            ProxyConfig::Vmess(p) => &p.server,
            ProxyConfig::Trojan(p) => &p.server,
            ProxyConfig::Shadowsocks(p) => &p.server,
            ProxyConfig::Hysteria2(p) => &p.server,
            ProxyConfig::Tuic(p) => &p.server,
        }
    }
}

pub fn parse_subscription(raw: &str) -> Result<ProxyConfig, VpnError> {
    let s = raw.trim();
    parse_single_url(s).or_else(|_| {
        let decoded = try_decode_base64(s)?;
        let first = decoded
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && is_supported_scheme(l))
            .ok_or_else(|| VpnError::ParseError("No valid proxy URL in subscription".into()))?;
        parse_single_url(first)
    })
}

fn try_decode_base64(s: &str) -> Result<String, VpnError> {
    let cleaned = s.replace(['\n', '\r', ' '], "");
    let padded = pad_base64(&cleaned);

    general_purpose::STANDARD
        .decode(&padded)
        .or_else(|_| general_purpose::URL_SAFE.decode(&padded))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(s.trim()))
        .map_err(|e| VpnError::ParseError(format!("base64 decode: {}", e)))
        .and_then(|b| String::from_utf8(b).map_err(|e| VpnError::ParseError(e.to_string())))
}

fn pad_base64(s: &str) -> String {
    let rem = s.len() % 4;
    if rem == 0 {
        s.to_string()
    } else {
        format!("{}{}", s, "=".repeat(4 - rem))
    }
}

fn is_supported_scheme(s: &str) -> bool {
    matches!(
        s.split("://").next().unwrap_or(""),
        "vless" | "vmess" | "trojan" | "ss" | "hysteria2" | "hy2" | "tuic"
    )
}

pub fn parse_all_from_subscription(raw: &str) -> Vec<(ProxyConfig, String)> {
    let content = if let Ok(decoded) = try_decode_base64(raw.trim()) {
        decoded
    } else {
        raw.to_string()
    };

    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && is_supported_scheme(l))
        .filter_map(|line| {
            let name = extract_fragment(line);
            parse_single_url(line).ok().map(|cfg| (cfg, name))
        })
        .collect()
}

fn extract_fragment(url_str: &str) -> String {
    if let Some(pos) = url_str.rfind('#') {
        let frag = &url_str[pos + 1..];
        urlencoding::decode(frag)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| frag.to_string())
    } else {
        String::new()
    }
}

fn parse_single_url(s: &str) -> Result<ProxyConfig, VpnError> {
    if s.starts_with("vless://") {
        parse_vless(s)
    } else if s.starts_with("vmess://") {
        parse_vmess(s)
    } else if s.starts_with("trojan://") {
        parse_trojan(s)
    } else if s.starts_with("ss://") {
        parse_shadowsocks(s)
    } else if s.starts_with("hysteria2://") || s.starts_with("hy2://") {
        parse_hysteria2(s)
    } else if s.starts_with("tuic://") {
        parse_tuic(s)
    } else {
        Err(VpnError::InvalidFormat(format!("Unknown scheme: {}", s.split("://").next().unwrap_or(""))))
    }
}

fn query_map(url: &Url) -> HashMap<String, String> {
    url.query_pairs()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn parse_vless(raw: &str) -> Result<ProxyConfig, VpnError> {
    let url = Url::parse(raw)?;

    let uuid = url.username();
    if uuid.is_empty() {
        return Err(VpnError::ParseError("VLESS: UUID is empty".into()));
    }

    let server = url
        .host_str()
        .ok_or_else(|| VpnError::ParseError("VLESS: missing host".into()))?
        .to_string();
    let port = url.port().unwrap_or(443);
    let q = query_map(&url);

    let network = q.get("type").cloned().unwrap_or_else(|| "tcp".into());
    let security = q.get("security").cloned().unwrap_or_else(|| "none".into());
    let sni = q.get("sni").cloned().unwrap_or_else(|| server.clone());
    let fingerprint = q.get("fp").cloned().unwrap_or_else(|| "chrome".into());
    let public_key = q.get("pbk").cloned().unwrap_or_default();
    let short_id = q.get("sid").cloned().unwrap_or_default();
    let flow = q.get("flow").cloned().unwrap_or_default();
    let path = q.get("path").cloned().unwrap_or_else(|| "/".into());
    let host = q.get("host").cloned().unwrap_or_else(|| server.clone());
    let service_name = q.get("serviceName").cloned().unwrap_or_default();
    let insecure = q.get("allowInsecure").map(|v| v == "1" || v == "true").unwrap_or(false);

    Ok(ProxyConfig::Vless(VlessParams {
        uuid: uuid.to_string(),
        server,
        port,
        flow,
        network,
        security,
        sni,
        fingerprint,
        public_key,
        short_id,
        path,
        host,
        service_name,
        insecure,
    }))
}

fn parse_vmess(raw: &str) -> Result<ProxyConfig, VpnError> {
    let b64 = raw.trim_start_matches("vmess://");
    let decoded = try_decode_base64(b64)?;

    #[derive(Deserialize)]
    struct VmessJson {
        id: String,
        add: String,
        port: serde_json::Value,
        #[serde(rename = "aid", default)]
        alter_id: serde_json::Value,
        scy: Option<String>,
        net: Option<String>,
        tls: Option<String>,
        sni: Option<String>,
        path: Option<String>,
        host: Option<String>,
        #[serde(rename = "type")]
        #[allow(dead_code)]
        header_type: Option<String>,
        #[serde(rename = "grpcServiceName")]
        grpc_service_name: Option<String>,
    }

    let v: VmessJson = serde_json::from_str(&decoded)
        .map_err(|e| VpnError::ParseError(format!("VMess JSON: {}", e)))?;

    let port = match &v.port {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(443) as u16,
        serde_json::Value::String(s) => s.parse::<u16>().unwrap_or(443),
        _ => 443,
    };

    let alter_id = match &v.alter_id {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as u16,
        serde_json::Value::String(s) => s.parse::<u16>().unwrap_or(0),
        _ => 0,
    };

    let tls_str = v.tls.as_deref().unwrap_or("none");
    let network = v.net.clone().unwrap_or_else(|| "tcp".into());
    let service_name = v.grpc_service_name.clone().unwrap_or_default();
    let sni = v.sni.clone().unwrap_or_else(|| v.add.clone());
    let host = v.host.clone().unwrap_or_else(|| v.add.clone());

    Ok(ProxyConfig::Vmess(VmessParams {
        uuid: v.id,
        server: v.add,
        port,
        alter_id,
        security: v.scy.unwrap_or_else(|| "auto".into()),
        network,
        tls: tls_str == "tls",
        sni,
        path: v.path.unwrap_or_else(|| "/".into()),
        host,
        service_name,
    }))
}

fn parse_trojan(raw: &str) -> Result<ProxyConfig, VpnError> {
    let url = Url::parse(raw)?;

    let password = url.username();
    if password.is_empty() {
        return Err(VpnError::ParseError("Trojan: password is empty".into()));
    }

    let server = url
        .host_str()
        .ok_or_else(|| VpnError::ParseError("Trojan: missing host".into()))?
        .to_string();
    let port = url.port().unwrap_or(443);
    let q = query_map(&url);

    let sni = q.get("sni").cloned().unwrap_or_else(|| server.clone());
    let fingerprint = q.get("fp").cloned().unwrap_or_else(|| "chrome".into());
    let network = q.get("type").cloned().unwrap_or_else(|| "tcp".into());
    let path = q.get("path").cloned().unwrap_or_else(|| "/".into());
    let host = q.get("host").cloned().unwrap_or_else(|| server.clone());
    let insecure = q.get("allowInsecure").map(|v| v == "1" || v == "true").unwrap_or(false);

    Ok(ProxyConfig::Trojan(TrojanParams {
        password: password.to_string(),
        server,
        port,
        sni,
        fingerprint,
        network,
        path,
        host,
        insecure,
    }))
}

fn parse_shadowsocks(raw: &str) -> Result<ProxyConfig, VpnError> {
    let body = raw.trim_start_matches("ss://");
    let body = body.split('#').next().unwrap_or(body);

    if !body.contains('@') {
        let blob = body.split('?').next().unwrap_or(body);
        let plain = try_decode_base64(blob)?;
        let (creds, addr) = plain.rsplit_once('@')
            .ok_or_else(|| VpnError::ParseError("SS: legacy form missing @".into()))?;
        let (method, password) = creds.split_once(':')
            .ok_or_else(|| VpnError::ParseError("SS: invalid legacy userinfo".into()))?;
        let (server, port_str) = addr.rsplit_once(':')
            .ok_or_else(|| VpnError::ParseError("SS: legacy form missing port".into()))?;
        let port = port_str.parse::<u16>()
            .map_err(|e| VpnError::ParseError(format!("SS: bad port: {}", e)))?;
        if method.is_empty() || password.is_empty() {
            return Err(VpnError::ParseError("SS: method or password is empty".into()));
        }
        if server.is_empty() {
            return Err(VpnError::ParseError("SS: missing host".into()));
        }
        return Ok(ProxyConfig::Shadowsocks(ShadowsocksParams {
            method: method.to_string(),
            password: password.to_string(),
            server: server.to_string(),
            port,
            plugin: String::new(),
            plugin_opts: String::new(),
        }));
    }

    let url = Url::parse(raw)?;

    let server = url
        .host_str()
        .ok_or_else(|| VpnError::ParseError("SS: missing host".into()))?
        .to_string();
    let port = url.port().unwrap_or(8388);
    let q = query_map(&url);

    let plugin = q.get("plugin").cloned().unwrap_or_default();
    let plugin_opts = q.get("plugin-opts").cloned().unwrap_or_default();

    let userinfo_raw = urlencoding::decode(url.username())
        .map(|s| s.to_string())
        .unwrap_or_else(|_| url.username().to_string());

    let userinfo = if userinfo_raw.contains(':') {
        userinfo_raw
    } else if let Some(pw) = url.password() {
        let pw_decoded = urlencoding::decode(pw)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| pw.to_string());
        format!("{}:{}", userinfo_raw, pw_decoded)
    } else {
        let padded = pad_base64(&userinfo_raw);
        general_purpose::STANDARD
            .decode(&padded)
            .or_else(|_| general_purpose::URL_SAFE.decode(&padded))
            .map_err(|e| VpnError::ParseError(format!("SS userinfo decode: {}", e)))
            .and_then(|b| String::from_utf8(b).map_err(|e| VpnError::ParseError(e.to_string())))?
    };

    let (method, password) = userinfo
        .split_once(':')
        .map(|(m, p)| (m.to_string(), p.to_string()))
        .ok_or_else(|| VpnError::ParseError("SS: invalid userinfo format".into()))?;

    if method.is_empty() || password.is_empty() {
        return Err(VpnError::ParseError("SS: method or password is empty".into()));
    }

    Ok(ProxyConfig::Shadowsocks(ShadowsocksParams {
        method,
        password,
        server,
        port,
        plugin,
        plugin_opts,
    }))
}
fn parse_hysteria2(raw: &str) -> Result<ProxyConfig, VpnError> {
    let normalized = raw
        .replacen("hysteria2://", "https://", 1)
        .replacen("hy2://", "https://", 1);
    let url = Url::parse(&normalized)?;

    let password = url.username();
    if password.is_empty() {
        return Err(VpnError::ParseError("Hysteria2: password is empty".into()));
    }

    let server = url
        .host_str()
        .ok_or_else(|| VpnError::ParseError("Hysteria2: missing host".into()))?
        .to_string();
    let port = url.port().unwrap_or(443);
    let q = query_map(&url);

    let sni = q.get("sni").cloned().unwrap_or_else(|| server.clone());
    let insecure = q.get("insecure").map(|v| v == "1" || v == "true").unwrap_or(false);
    let obfs = q.get("obfs").cloned().unwrap_or_default();
    let obfs_password = q.get("obfs-password").cloned().unwrap_or_default();

    Ok(ProxyConfig::Hysteria2(Hysteria2Params {
        password: password.to_string(),
        server,
        port,
        sni,
        insecure,
        obfs,
        obfs_password,
    }))
}

fn parse_tuic(raw: &str) -> Result<ProxyConfig, VpnError> {
    let url = Url::parse(raw)?;

    let uuid = url.username();
    let password = url.password().unwrap_or("");
    if uuid.is_empty() {
        return Err(VpnError::ParseError("TUIC: UUID is empty".into()));
    }

    let server = url
        .host_str()
        .ok_or_else(|| VpnError::ParseError("TUIC: missing host".into()))?
        .to_string();
    let port = url.port().unwrap_or(443);
    let q = query_map(&url);

    let sni = q.get("sni").cloned().unwrap_or_else(|| server.clone());
    let congestion_control = q.get("congestion_control").cloned().unwrap_or_else(|| "bbr".into());
    let insecure = q.get("allow_insecure").map(|v| v == "1" || v == "true").unwrap_or(false);

    Ok(ProxyConfig::Tuic(TuicParams {
        uuid: uuid.to_string(),
        password: password.to_string(),
        server,
        port,
        sni,
        congestion_control,
        insecure,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vless_reality() {
        let url = "vless://abcd-1234@1.2.3.4:443?security=reality&sni=yahoo.com&fp=chrome&pbk=pubkey123&sid=abc&type=tcp&flow=xtls-rprx-vision#Test";
        let r = parse_vless(url).unwrap();
        match r {
            ProxyConfig::Vless(p) => {
                assert_eq!(p.uuid, "abcd-1234");
                assert_eq!(p.security, "reality");
                assert_eq!(p.flow, "xtls-rprx-vision");
                assert_eq!(p.public_key, "pubkey123");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn vless_ws_tls() {
        let url = "vless://uuid@example.com:443?security=tls&type=ws&path=%2Fws&host=example.com&sni=example.com#WS";
        let r = parse_vless(url).unwrap();
        match r {
            ProxyConfig::Vless(p) => {
                assert_eq!(p.network, "ws");
                assert_eq!(p.security, "tls");
                assert_eq!(p.path, "/ws");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn trojan_basic() {
        let url = "trojan://mypassword@server.com:443?sni=server.com#Trojan";
        let r = parse_trojan(url).unwrap();
        match r {
            ProxyConfig::Trojan(p) => {
                assert_eq!(p.password, "mypassword");
                assert_eq!(p.port, 443);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn hysteria2_basic() {
        let url = "hysteria2://pass123@1.2.3.4:8443?sni=example.com&insecure=0#HY2";
        let r = parse_hysteria2(url).unwrap();
        match r {
            ProxyConfig::Hysteria2(p) => {
                assert_eq!(p.password, "pass123");
                assert_eq!(p.port, 8443);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn ss_base64_userinfo() {
        let userinfo = general_purpose::STANDARD.encode("chacha20-ietf-poly1305:mypassword");
        let url = format!("ss://{}@1.2.3.4:8388#SS", userinfo);
        let r = parse_shadowsocks(&url).unwrap();
        match r {
            ProxyConfig::Shadowsocks(p) => {
                assert_eq!(p.method, "chacha20-ietf-poly1305");
                assert_eq!(p.password, "mypassword");
            }
            _ => panic!("wrong variant"),
        }
    }
}